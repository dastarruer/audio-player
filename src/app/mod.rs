mod ui;

use fltk::{app, button, enums::Color, prelude::*, window};

use rodio::Sink;
use rodio::{Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;
use std::process::exit;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

/// A message to be sent to the audio thread
enum Message {
    Play,
    Pause,
    FastForward(Duration),
    Rewind(Duration),
}

/// Store the functionality for playing audio and other functions.
struct AudioHandler {
    /// Audio sink to control playback
    sink: Arc<Mutex<Option<Sink>>>,

    /// The audio that is playing
    stream: Arc<Mutex<Option<OutputStream>>>,
}

impl AudioHandler {
    /// Return an empty instance of AudioPlayer.
    fn new() -> AudioHandler {
        // Use None for now; this will become populated in self.play_audio
        let sink = Arc::new(Mutex::new(None));
        let stream = Arc::new(Mutex::new(None));

        AudioHandler { sink, stream }
    }

    /// Play audio and initialize self.sink and self.stream.
    fn play_audio(&self, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) {
        let sink_ref = Arc::clone(&self.sink);
        let stream_ref = Arc::clone(&self.stream);

        thread::spawn(move || {
            // Get an output stream handle to the default physical sound device.
            let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
                .expect("open default audio stream");

            // Create a new audio sink, which will be used to control playback of audio
            let sink = rodio::Sink::connect_new(&stream_handle.mixer());

            // Load sound file
            let file_path = "../../test.mp3";
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

/// Stores the components of the GUI.
pub struct App {
    app: app::App,
    window: window::DoubleWindow,

    /// Buttons to control playback. These are the pause, rewind, and fast-forward buttons.
    playback_buttons: Option<[button::Button; 3]>,

    /// An AudioHandler, which will handle audio related functions such as playing audio.
    audio_handler: AudioHandler,
}

impl App {
    const WIN_WIDTH: i32 = 400;
    const WIN_HEIGHT: i32 = 300;

    const PLAY_BTN_SIZE: i32 = 30;
    const PLAY_BTN_X: i32 = (Self::WIN_WIDTH - Self::PLAY_BTN_SIZE) / 2; // Center the button horizontally
    const PLAY_BTN_Y: i32 = 200;

    const SEEK_DURATION_SECS: u64 = 5;

    /// Create the new App.
    pub fn new() -> App {
        let app = app::App::default().with_scheme(app::Scheme::Gtk);
        let audio_handler = AudioHandler::new();

        // Create a new window
        let window = App::create_window();

        App {
            app,
            window,
            playback_buttons: None,
            audio_handler,
        }
    }

    /// Run the app.
    pub fn run(&mut self) {
        // Create a channel to send messages to the audio thread
        let (sender, recevier) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(recevier));

        // Create the components
        self.create_app_components(sender);

        // Show the window
        self.window.end();
        self.window.show();

        // Play the audio
        self.audio_handler.play_audio(Arc::clone(&receiver));

        // Run the app
        self.app.run().unwrap();
    }

    /// Create all the necessary app components, such as the playback buttons, etc.
    fn create_app_components(&mut self, sender: mpsc::Sender<Message>) {
        self.playback_buttons = Some(self.create_playback_buttons(sender.clone()));
    }

    /// Create the window and theme it.
    fn create_window() -> window::DoubleWindow {
        let mut win = window::Window::default()
            .with_size(Self::WIN_WIDTH, Self::WIN_HEIGHT)
            .with_label("My window");
        win.set_color(Color::White);
        win
    }

    fn style_button(mut btn: button::Button) -> button::Button {
        // Remove focus border around button
        btn.clear_visible_focus();

        // Remove button background
        btn.set_frame(fltk::enums::FrameType::NoBox);

        btn
    }

    /// Create the play button and theme it.
    fn create_play_button(&self, sender: mpsc::Sender<Message>) -> button::Button {
        const PLAY_BUTTON: &str = "";
        const PAUSE_BUTTON: &str = "";

        let mut btn = App::style_button(
            button::Button::default()
                .with_size(Self::PLAY_BTN_SIZE, Self::PLAY_BTN_SIZE)
                .with_pos(Self::PLAY_BTN_X, Self::PLAY_BTN_Y)
                .with_label(PAUSE_BUTTON),
        );

        // Define a function to execute once the button is clicked
        btn.set_callback(move |btn| {
            // Play/pause audio
            match btn.label().as_str() {
                PAUSE_BUTTON => {
                    btn.set_label(PLAY_BUTTON);

                    // Send a message to the audio thread to pause the audio
                    match sender.send(Message::Pause) {
                        Ok(_) => (),
                        Err(e) => println!("Unable to play audio: {:?}", e),
                    };
                }
                PLAY_BUTTON => {
                    btn.set_label(PAUSE_BUTTON);

                    // Send a message to the audio thread to play the audio
                    match sender.send(Message::Play) {
                        Ok(_) => (),
                        Err(e) => println!("Unable to play audio: {:?}", e),
                    };
                }
                _ => unreachable!(),
            };
        });

        btn
    }

    /// Create playback buttons to rewind,, fast-forward and play/pause.
    fn create_playback_buttons(&self, sender: mpsc::Sender<Message>) -> [button::Button; 3] {
        let seek_forwards_btn = self.create_fast_forward_button(sender.clone());
        let seek_backwards_btn = self.create_rewind_button(sender.clone());
        let play_btn = self.create_play_button(sender.clone());

        [seek_backwards_btn, seek_forwards_btn, play_btn]
    }

    /// Create the fast-forwards button.
    fn create_fast_forward_button(&self, sender: mpsc::Sender<Message>) -> button::Button {
        let mut seek_forwards_btn = App::style_button(
            button::Button::default()
                .with_size(Self::PLAY_BTN_SIZE, Self::PLAY_BTN_SIZE)
                .with_pos(Self::PLAY_BTN_X + 100, Self::PLAY_BTN_Y)
                .with_label("󰵱"),
        );

        seek_forwards_btn.set_callback(move |_| {
            // Send a fast-forward message to the audio thread
            match sender.send(Message::FastForward(Duration::from_secs(
                Self::SEEK_DURATION_SECS,
            ))) {
                Ok(_) => (),
                Err(e) => println!("Unable to fast-forward: {:?}", e),
            }
        });

        seek_forwards_btn
    }

    /// Create the rewind button.
    fn create_rewind_button(&self, sender: mpsc::Sender<Message>) -> button::Button {
        let mut seek_backwards_btn = App::style_button(
            button::Button::default()
                .with_size(Self::PLAY_BTN_SIZE, Self::PLAY_BTN_SIZE)
                .with_pos(Self::PLAY_BTN_X - 100, Self::PLAY_BTN_Y)
                .with_label("󰴪"),
        );

        seek_backwards_btn.set_callback(move |_| {
            // Send a rewind message to the audio thread
            match sender.send(Message::Rewind(Duration::from_secs(
                Self::SEEK_DURATION_SECS,
            ))) {
                Ok(_) => (),
                Err(e) => println!("Unable to rewind: {:?}", e),
            }
        });

        seek_backwards_btn
    }
}
