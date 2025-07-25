use fltk::button::Button;
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

    /// Create the new App
    pub fn new() -> App {
        let app = app::App::default().with_scheme(app::Scheme::Gtk);
        let audio_handler = AudioHandler::new();

        // Create a new window
        let window = App::create_window();

        let mut app = App {
            app,
            window,
            play_button: None,
            seek_buttons: None,
            audio_handler,
        };

        app.play_button = Some(app.create_play_button());
        app.seek_buttons = Some(app.create_seek_buttons());

        app
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
    fn create_play_button(&self) -> button::Button {
        const PLAY_BUTTON: &str = "";
        const PAUSE_BUTTON: &str = "";

        // Clone the reference to the sink
        let sink_ref = self.audio_handler.get_sink_ref();

        let mut btn = App::style_button(
            button::Button::default()
                .with_size(Self::PLAY_BTN_SIZE, Self::PLAY_BTN_SIZE)
                .with_pos(Self::PLAY_BTN_X, Self::PLAY_BTN_Y)
                .with_label(PAUSE_BUTTON),
        );

        // Define a function to execute once the button is clicked
        btn.set_callback(move |btn| {
            // Play/pause audio
            AudioHandler::with_sink(&sink_ref, |sink| match sink.is_paused() {
                true => {
                    btn.set_label(PAUSE_BUTTON);
                    sink.play();
                }
                false => {
                    btn.set_label(PLAY_BUTTON);
                    sink.pause();
                }
            });
        });

        btn
    }

    /// Create the seek buttons to fast-forward and rewind
    fn create_seek_buttons(&self) -> [button::Button; 2] {
        let seek_forwards_btn = App::style_button(
            button::Button::default()
                .with_size(Self::PLAY_BTN_SIZE, Self::PLAY_BTN_SIZE)
                .with_pos(Self::PLAY_BTN_X + 100, Self::PLAY_BTN_Y)
                .with_label("󰵱"),
        );
        let seek_backwards_btn = App::style_button(
            button::Button::default()
                .with_size(Self::PLAY_BTN_SIZE, Self::PLAY_BTN_SIZE)
                .with_pos(Self::PLAY_BTN_X - 100, Self::PLAY_BTN_Y)
                .with_label("󰴪"),
        );

        [seek_backwards_btn, seek_forwards_btn]
    }
}
