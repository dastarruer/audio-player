use rodio::Sink;
use rodio::{Decoder, OutputStream};
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

    /// Play audio and initialize self.sink and self.stream.
    pub(crate) fn play_audio(
        &self,
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        audio_pos_sender: mpsc::Sender<Duration>,
    ) {
        let sink_ref = Arc::clone(&self.sink);
        let stream_ref = Arc::clone(&self.stream);

        thread::spawn(move || {
            // Get an output stream handle to the default physical sound device.
            let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
                .expect("open default audio stream");

            // Create a new audio sink, which will be used to control playback of audio
            let sink = rodio::Sink::connect_new(&stream_handle.mixer());

            // Load sound file
            let file_path = "../test.mp3";
            let file = match File::open(file_path) {
                Ok(file) => file,
                Err(_) => {
                    eprintln!("File path does not exist. Exiting...");
                    exit(1);
                }
            };

            // Get the byte length of the file
            let byte_len = match file.metadata() {
                Ok(metadata) => metadata.len(),
                Err(_) => {
                    eprintln!("Unable to determine file byte length. Exiting...");
                    exit(1);
                }
            };

            // I have no idea what this does
            let file = BufReader::new(file);

            // Decode that sound file into a source
            let source = match Decoder::builder()
                .with_data(file)
                // Specify the length of the audio source for reliable seeking
                .with_byte_len(byte_len)
                // Essential to allow for seeking backwards
                .with_seekable(true)
                .build()
            {
                Ok(source) => source,
                Err(_) => {
                    eprintln!("File format not supported. Exiting...");
                    exit(1);
                }
            };

            // Play the sound directly on the device
            sink.append(source);

            // Add sink to self.sink so that it outlives the current thread
            *sink_ref.lock().unwrap() = Some(sink);

            // Add stream_handle to self.stream_handle so that it outlives the current thread and keeps playing audio
            *stream_ref.lock().unwrap() = Some(stream_handle);

            // Create a new thread to send the audio's current position to the progress bar
            let new_sink_ref = Arc::clone(&sink_ref);
            thread::spawn(move || {
                loop {
                    let current_pos = AudioHandler::with_sink(&new_sink_ref, |sink| sink.get_pos());

                    // Send the current position to the progress bar
                    match audio_pos_sender.send(current_pos) {
                        Ok(_) => (),
                        Err(_) => break, // Break the loop because if audio_pos_sender.send returns an error, it means the receiving end does not exist anymore
                    };
                }
            });

            // Continuously scan for new messages sent by the App
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                AudioHandler::handle_messages(message, &sink_ref);
            }
        });
    }

    /// A function that handles messages sent to the audio thread.
    fn handle_messages(message: Message, sink_ref: &Arc<Mutex<Option<Sink>>>) {
        match message {
            Message::Play => AudioHandler::with_sink(&sink_ref, |sink| {
                sink.play();
            }),
            Message::Pause => AudioHandler::with_sink(&sink_ref, |sink| {
                sink.pause();
            }),
            Message::FastForward(duration_secs) => AudioHandler::with_sink(&sink_ref, |sink| {
                let current_pos = sink.get_pos();
                let target_pos = current_pos + duration_secs;

                match sink.try_seek(target_pos) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Unable to fast-forward: {:?}", e),
                };
            }),
            Message::Rewind(duration_secs) => AudioHandler::with_sink(&sink_ref, |sink| {
                let current_pos = sink.get_pos();

                // Subtract `duration_secs` from `current_pos`, and if result is negative, default to Duration::ZERO
                let target_pos = current_pos
                    .checked_sub(duration_secs)
                    .unwrap_or(Duration::ZERO);

                // Seek to the target position
                match sink.try_seek(target_pos) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Unable to rewind: {:?}", e),
                };
            }),
        }
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
