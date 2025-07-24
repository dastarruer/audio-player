use fltk::{app, button, enums::Color, prelude::*, window};

use rodio::Sink;
use rodio::{Decoder, OutputStream, source::Source};
use std::fs::File;
use std::io::BufReader;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;

/// Stores the components of the GUI
pub struct App {
    app: app::App,
    window: window::DoubleWindow,

    /// Button to play/pause audio
    play_button: button::Button,

    /// Audio sink to control playback
    sink: Arc<Mutex<Option<Sink>>>,
}

impl App {
    /// Create the new App
    pub fn new() -> App {
        let app = app::App::default().with_scheme(app::Scheme::Gtk);

        // Create a new window
        const WIN_WIDTH: i32 = 400;
        const WIN_HEIGHT: i32 = 300;
        let window = App::create_window(WIN_WIDTH, WIN_HEIGHT);

        // Create a placeholder value to use for `sink` in App struct
        let sink = Arc::new(Mutex::new(None));

        const BTN_SIZE: i32 = 30;
        const BTN_X: i32 = (WIN_WIDTH - BTN_SIZE) / 2; // Center the button horizontally
        const BTN_Y: i32 = 200;
        let play_button = App::create_play_button(BTN_SIZE, BTN_X, BTN_Y, &sink);


        App {
            app,
            window,
            play_button,
            sink,
        }
    }

    /// Run the app
    pub fn run(&mut self) {
        // Show the window
        self.window.end();
        self.window.show();

        // Play the audio
        self.play_audio();

        // Run the app
        self.app.run().unwrap();
    }

    /// Play audio and set self.sink to an audio sink
    fn play_audio(&self) {
        let sink_ref = Arc::clone(&self.sink);

        thread::spawn(move || {
            // Get an output stream handle to the default physical sound device.
            let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
                .expect("open default audio stream");

            // Load sound from file
            let file_path = "test.mp3";
            let file = match File::open(file_path) {
                Ok(file) => BufReader::new(file),
                Err(_) => {
                    println!("File path does not exist. Exiting...");
                    exit(1);
                }
            };

            // Play audio
            let sink = match rodio::play(&stream_handle.mixer(), file) {
                Ok(sink) => sink,
                Err(_) => {
                    println!("Invalid file type.");
                    exit(1);
                }
            };

            // Since audio plays in seperate thread, block current thread from terminating
            sink.sleep_until_end();

            let mut mutex_guard = sink_ref.lock().unwrap();
            *mutex_guard = Some(sink);
        });
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
            let sink = sink_ref.lock().unwrap();

            if let Some(sink) = &*sink {
                // Play/pause audio on button click
                match btn.label().as_str() {
                    PLAY_BUTTON => {
                        btn.set_label(PAUSE_BUTTON);
                        sink.pause();
                    }
                    PAUSE_BUTTON => {
                        btn.set_label(PLAY_BUTTON);
                        sink.play();
                    }
                    _ => unreachable!(),
                }
            }
        });

        btn
    }
}
