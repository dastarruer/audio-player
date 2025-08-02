use rodio::Sink;
use rodio::{Decoder, OutputStream, Source};
use std::fs::File;
use std::io::BufReader;
use std::process::exit;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use crate::app::Message;

/// Store the functionality for playing audio and other functions.
// Note that pub(crate) means that AudioHandler can only be used by files in `app/`
pub(crate) struct AudioHandler {
    /// Audio sink to control playback
    sink: Arc<Mutex<Option<Sink>>>,

    /// The audio that is playing
    stream: Arc<Mutex<Option<OutputStream>>>,
}

impl AudioHandler {
    /// Return an empty instance of AudioPlayer.
    pub(crate) fn new() -> AudioHandler {
        // Use None for now; this will become populated in self.play_audio
        let sink = Arc::new(Mutex::new(None));
        let stream = Arc::new(Mutex::new(None));

        AudioHandler { sink, stream }
    }

    /// Load audio source from file, returning the Decoder and its total duration (if available).
    pub(crate) fn load_audio(file_path: &str) -> (Decoder<BufReader<File>>, Option<Duration>) {
        // Load sound file
        let file = File::open(file_path).unwrap_or_else(|_| {
            eprintln!("File path does not exist. Exiting...");
            exit(1);
        });

        let byte_len = file.metadata().map(|m| m.len()).unwrap_or(0);
        let file = BufReader::new(file);

        let decoder = Decoder::builder()
            .with_data(file)
            // Specify the length of the audio source for reliable seeking
            .with_byte_len(byte_len)
            // Essential to allow for seeking backwards
            .with_seekable(true)
            .build()
            .unwrap_or_else(|_| {
                eprintln!("File format not supported. Exiting...");
                exit(1);
            });

        let duration = decoder.total_duration();

        (decoder, duration)
    }

    /// Play audio and initialize self.sink and self.stream.
    pub(crate) fn play_audio(
        &self,
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        audio_pos_sender: mpsc::Sender<Duration>,
        decoder: Decoder<BufReader<File>>,
    ) {
        let sink_ref = Arc::clone(&self.sink);
        let stream_ref = Arc::clone(&self.stream);

        thread::spawn(move || {
            // Get an output stream handle to the default physical sound device.
            let stream_handle = AudioHandler::open_output_stream();

            // Create a new audio sink, which will be used to control playback of audio
            let sink = AudioHandler::create_sink(&stream_handle);

            // Play the sound directly on the device
            sink.append(decoder);

            // Add sink to self.sink so that it can be accessed by other methods
            *sink_ref.lock().unwrap() = Some(sink);

            // Add stream_handle to self.stream_handle so that it outlives the current thread and keeps playing audio
            *stream_ref.lock().unwrap() = Some(stream_handle);

            // Send the audio's current position to the progress bar
            AudioHandler::send_audio_pos(audio_pos_sender.clone(), Arc::clone(&sink_ref));

            // Continuously scan for new messages sent by the AudioApp
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                AudioHandler::handle_messages(message, &sink_ref, &audio_pos_sender);
            }
        });
    }

    /// Create a new thread to send the audio's current position to the progress bar
    fn send_audio_pos(
        audio_pos_sender: mpsc::Sender<Duration>,
        new_sink_ref: Arc<Mutex<Option<Sink>>>,
    ) {
        thread::spawn(move || {
            loop {
                let current_pos = AudioHandler::with_sink(&new_sink_ref, |sink| {
                    // Delay before sending position, otherwise a completely wrong position will be sent
                    thread::sleep(Duration::from_millis(100));
                    sink.get_pos()
                });

                // Send the current position to the progress bar
                match audio_pos_sender.send(current_pos) {
                    Ok(_) => (),
                    Err(_) => break,
                };
            }
        });
    }

    fn create_sink(stream_handle: &OutputStream) -> Sink {
        rodio::Sink::connect_new(&stream_handle.mixer())
    }

    fn open_output_stream() -> OutputStream {
        rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream")
    }

    /// A function that handles messages sent to the audio thread.
    fn handle_messages(
        message: Message,
        sink_ref: &Arc<Mutex<Option<Sink>>>,
        audio_pos_sender: &mpsc::Sender<Duration>,
    ) {
        match message {
            Message::Play => AudioHandler::with_sink(&sink_ref, |sink| {
                sink.play();
            }),
            Message::Pause => AudioHandler::with_sink(&sink_ref, |sink| {
                sink.pause();
            }),
            Message::FastForward(duration_secs) => AudioHandler::with_sink(&sink_ref, |sink| {
                AudioHandler::fast_forward(&audio_pos_sender, duration_secs, sink);
            }),
            Message::Rewind(duration_secs) => AudioHandler::with_sink(&sink_ref, |sink| {
                AudioHandler::rewind(audio_pos_sender, duration_secs, sink);
            }),
        }
    }

    fn fast_forward(
        audio_pos_sender: &mpsc::Sender<Duration>,
        duration_secs: Duration,
        sink: &Sink,
    ) {
        let current_pos = sink.get_pos();
        let target_pos = current_pos + duration_secs;
        println!("{:?}", target_pos);

        match sink.try_seek(target_pos) {
            Ok(_) => (),
            Err(e) => eprintln!("Unable to fast-forward: {:?}", e),
        };

        // Send the new position to the progress bar
        match audio_pos_sender.send(target_pos) {
            Ok(_) => (),
            Err(e) => eprintln!("Unable to send position to progress bar: {:?}", e),
        };
    }

    fn rewind(audio_pos_sender: &mpsc::Sender<Duration>, duration_secs: Duration, sink: &Sink) {
        let current_pos = sink.get_pos();

        // Subtract `duration_secs` from `current_pos`, and if result is negative, default to Duration::ZERO
        let target_pos = current_pos
            .checked_sub(duration_secs)
            .unwrap_or(Duration::ZERO);

        match sink.try_seek(target_pos) {
            Ok(_) => (),
            Err(e) => eprintln!("Unable to rewind: {:?}", e),
        };

        // Send the new position to the progress bar
        match audio_pos_sender.send(target_pos) {
            Ok(_) => (),
            Err(e) => eprintln!("Unable to send position to progress bar: {:?}", e),
        };
    }

    /// Run a closure that operates on `sink` for audio playback control by extracting `sink` from `sink_ref`.
    fn with_sink<F, R>(sink_ref: &Arc<Mutex<Option<Sink>>>, f: F) -> R
    where
        F: FnOnce(&Sink) -> R,
    {
        let guard = sink_ref.lock().unwrap();
        let sink = guard.as_ref().expect("Sink not initialized");

        // Run the passed closure, passing `sink` as an argument
        f(sink)
    }
}
