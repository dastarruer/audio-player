use fltk::{app, button, enums::Color, prelude::*, window};

use rodio::Sink;
use rodio::{Decoder, OutputStream, source::Source};
use std::fs::File;
use std::io::BufReader;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;

/// Store the functionality for playing audio and other functions
struct AudioHandler {
    /// Audio sink to control playback
    sink: Arc<Mutex<Option<Sink>>>,

    /// Audio stream
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
    fn play_audio(&self) {
        let sink_ref = Arc::clone(&self.sink);
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
                    println!("File path does not exist. Exiting...");
                    exit(1);
                }
            };

            // Decode that sound file into a source
            let source = match Decoder::try_from(file) {
                Ok(source) => source,
                Err(_) => {
                    println!("File format not supported. Exiting...");
                    exit(1);
                }
            };

            // Play the sound directly on the device
            sink.append(source);

            // Add sink to self.sink
            *sink_ref.lock().unwrap() = Some(sink);

            // Add stream_handle to self._stream_handle
            *stream_ref.lock().unwrap() = Some(stream_handle);

            // Block thread until sink is empty (when audio is finished)
            while !sink_ref.lock().unwrap().as_ref().unwrap().empty() {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    }
}

/// Stores the components of the GUI
pub struct App {
    app: app::App,
    window: window::DoubleWindow,

    /// Button to play/pause audio
    play_button: button::Button,

    /// An AudioHandler, which will handle audio related functions such as playing audio
    audio_handler: AudioHandler,
}

impl App {
    /// Create the new App
    pub fn new() -> App {
        let app = app::App::default().with_scheme(app::Scheme::Gtk);
        let audio_handler = AudioHandler::new();
        // Create a new window
        const WIN_WIDTH: i32 = 400;
        const WIN_HEIGHT: i32 = 300;
        let window = App::create_window(WIN_WIDTH, WIN_HEIGHT);

        const BTN_SIZE: i32 = 30;
        const BTN_X: i32 = (WIN_WIDTH - BTN_SIZE) / 2; // Center the button horizontally
        const BTN_Y: i32 = 200;
        let play_button = App::create_play_button(BTN_SIZE, BTN_X, BTN_Y, &audio_handler.sink);

        App {
            app,
            window,
            play_button,
            audio_handler,
        }
    }

    /// Run the app
    pub fn run(&mut self) {
        // Show the window
        self.window.end();
        self.window.show();

        // Play the audio
        self.audio_handler.play_audio();

        // Run the app
        self.app.run().unwrap();
    }

    /// Create the window and theme it
    fn create_window(width: i32, height: i32) -> window::DoubleWindow {
        let mut win = window::Window::default()
            .with_size(width, height)
            .with_label("My window");
        win.set_color(Color::White);
        win
    }

    /// Create the play button and theme it
    fn create_play_button(
        size: i32,
        x: i32,
        y: i32,
        sink: &Arc<Mutex<Option<Sink>>>,
    ) -> button::Button {
        const PLAY_BUTTON: &str = "";
        const PAUSE_BUTTON: &str = "";

        // Clone the reference to the sink
        let sink_ref = Arc::clone(&sink);

        let mut btn = button::Button::default()
            .with_size(size, size)
            .with_pos(x, y)
            .with_label(PAUSE_BUTTON);

        // Remove focus border around button
        btn.clear_visible_focus();

        // Remove button background
        btn.set_frame(fltk::enums::FrameType::NoBox);

        // Define a function to execute once the button is clicked
        btn.set_callback(move |btn| {
            // Get the audio sink
            let binding = sink_ref.lock().unwrap();
            let sink = binding.as_ref().unwrap();

            // Play/pause audio on button click
            match sink.is_paused() {
                true => {
                    btn.set_label(PAUSE_BUTTON);
                    sink.play();
                }
                false => {
                    btn.set_label(PLAY_BUTTON);
                    sink.pause();
                }
            }
        });

        btn
    }
}
