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

/// Store the functionality for playing audio and other functions
struct AudioHandler {
    /// Audio sink to control playback
    sink: Arc<Mutex<Option<Sink>>>,

    /// Audio stream
    /// We store this here so that the audio lives as long as the app
    stream: Arc<Mutex<Option<OutputStream>>>,
}

impl AudioHandler {
    /// Return an empty instance of AudioPlayer
    fn new() -> AudioHandler {
        // Use None for now; this will become populated in self.play_audio
        let sink = Arc::new(Mutex::new(None));
        let stream = Arc::new(Mutex::new(None));

        AudioHandler { sink, stream }
    }

    /// Play audio and initialize self.sink and self.stream
    fn play_audio(&self, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) {
        let sink_ref = self.get_sink_ref();
        let stream_ref = Arc::clone(&self.stream);

        thread::spawn(move || {
            // Get an output stream handle to the default physical sound device.
            let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
                .expect("open default audio stream");

            // Create a new audio sink
            let sink = rodio::Sink::connect_new(&stream_handle.mixer());

            // Load sound file
            let file_path = "test.mp3";
            let file = match File::open(file_path) {
                Ok(file) => BufReader::new(file),
                Err(_) => {
                    eprintln!("File path does not exist. Exiting...");
                    exit(1);
                }
            };

            // Decode that sound file into a source
            let source = match Decoder::try_from(file) {
                Ok(source) => source,
                Err(_) => {
                    eprintln!("File format not supported. Exiting...");
                    exit(1);
                }
            };

            // Play the sound directly on the device
            sink.append(source);

            // Add sink to self.sink
            *sink_ref.lock().unwrap() = Some(sink);

            // Add stream_handle to self.stream_handle
            *stream_ref.lock().unwrap() = Some(stream_handle);

            loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                AudioHandler::handle_messages(message, &sink_ref);
            }
        });
    }

    /// A function that handles messages sent to the audio thread
    fn handle_messages(message: Message, sink_ref: &Arc<Mutex<Option<Sink>>>) {
        match message {
            Message::Play => AudioHandler::with_sink(&sink_ref, |sink| {
                sink.play();
            }),
            Message::Pause => AudioHandler::with_sink(&sink_ref, |sink| {
                sink.pause();
            }),
            Message::FastForward(duration) => AudioHandler::with_sink(&sink_ref, |sink| {
                let current_pos = sink.get_pos();
                match sink.try_seek(current_pos + duration) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Unable to fast-forward: {:?}", e),
                };
            }),
            Message::Rewind(duration) => AudioHandler::with_sink(&sink_ref, |sink| {
                let current_pos = sink.get_pos();

                // Ensure that current_pos is not smaller than 10, which would panic if current_pos - Duration::from_secs(10) were to be called
                if current_pos < duration {
                    match sink.try_seek(Duration::from_secs(0)) {
                        Ok(_) => (),
                        Err(e) => eprintln!("Unable to rewind: {:?}", e),
                    };
                } else {
                    match sink.try_seek(current_pos - duration) {
                        Ok(_) => (),
                        Err(e) => eprintln!("Unable to rewind: {:?}", e),
                    };
                }
            }),
        }
    }

    /// Returns a clones reference for `self.sink`.
    fn get_sink_ref(&self) -> Arc<Mutex<Option<Sink>>> {
        Arc::clone(&self.sink)
    }

    /// Run a closure that extracts `sink` from `sink_ref` (which can be obtained using `get_sink_ref()`).
    fn with_sink<F, R>(sink_ref: &Arc<Mutex<Option<Sink>>>, f: F) -> R
    where
        F: FnOnce(&Sink) -> R,
    {
        let guard = sink_ref.lock().unwrap();
        let sink = guard.as_ref().expect("Sink not initialized");
        f(sink)
    }
}

/// Stores the components of the GUI
pub struct App {
    app: app::App,
    window: window::DoubleWindow,

    /// Button to play/pause audio
    play_button: Option<button::Button>,

    /// Buttons to seek forwards and backwards
    seek_buttons: Option<[button::Button; 2]>,

    /// An AudioHandler, which will handle audio related functions such as playing audio
    audio_handler: AudioHandler,
}

impl App {
    const WIN_WIDTH: i32 = 400;
    const WIN_HEIGHT: i32 = 300;

    const PLAY_BTN_SIZE: i32 = 30;
    const PLAY_BTN_X: i32 = (Self::WIN_WIDTH - Self::PLAY_BTN_SIZE) / 2; // Center the button horizontally
    const PLAY_BTN_Y: i32 = 200;

    const SEEK_DURATION_SECS: u64 = 5;

    /// Create the new App
    pub fn new() -> App {
        let app = app::App::default().with_scheme(app::Scheme::Gtk);
        let audio_handler = AudioHandler::new();

        // Create a new window
        let window = App::create_window();

        App {
            app,
            window,
            play_button: None,
            seek_buttons: None,
            audio_handler,
        }
    }

    /// Run the app
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

    /// Create all the necessary app components, such as the pause button, fast-forward and rewind buttons, etc.
    fn create_app_components(&mut self, sender: mpsc::Sender<Message>) {
        self.play_button = Some(self.create_play_button(sender.clone()));
        self.seek_buttons = Some(self.create_seek_buttons(sender.clone()));
    }

    /// Create the window and theme it
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

    /// Create the play button and theme it
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

    /// Create the seek buttons to fast-forward and rewind
    fn create_seek_buttons(&self, sender: mpsc::Sender<Message>) -> [button::Button; 2] {
        let seek_forwards_btn = self.create_fast_forward_button(sender.clone());
        let seek_backwards_btn = self.create_rewind_button(sender.clone());

        [seek_backwards_btn, seek_forwards_btn]
    }

    /// Create the fast-forwards button
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

    /// Create the rewind button
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
