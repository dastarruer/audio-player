mod audio_handler;
mod ui;

use fltk::{app, enums::Color, prelude::*, window};

use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use audio_handler::AudioHandler;
use ui::playback_buttons::PlaybackButtons;

use crate::app::ui::progress_bar::ProgressBar;

/// A message to be sent to the audio thread
pub enum Message {
    Play,
    Pause,
    FastForward(Duration),
    Rewind(Duration),
}

/// Stores the components of the GUI.
pub struct AudioApp {
    app: app::App,
    window: window::DoubleWindow,

    /// Buttons to control playback. These are the pause, rewind, and fast-forward buttons.
    playback_buttons: Option<PlaybackButtons>,

    /// Progress bar to show the user the current timestamp of the audio. Also allows them to seek to a certain position.
    progress_bar: Option<ProgressBar>,

    /// An AudioHandler, which will handle audio related functions such as playing audio.
    audio_handler: AudioHandler,
}

impl AudioApp {
    const WIN_WIDTH: i32 = 400;
    const WIN_HEIGHT: i32 = 300;

    /// Create the new App.
    pub fn new() -> AudioApp {
        let app = app::App::default().with_scheme(app::Scheme::Gtk);
        let audio_handler = AudioHandler::new();

        // Create a new window
        let window = AudioApp::create_window();

        AudioApp {
            app,
            window,
            playback_buttons: None,
            progress_bar: None,
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
        self.playback_buttons = Some(PlaybackButtons::new(AudioApp::WIN_WIDTH, sender));
        self.progress_bar = Some(ProgressBar::new(AudioApp::WIN_WIDTH));
    }

    /// Create the window and theme it.
    fn create_window() -> window::DoubleWindow {
        let mut win = window::Window::default()
            .with_size(AudioApp::WIN_WIDTH, AudioApp::WIN_HEIGHT)
            .with_label("My window");
        win.set_color(Color::White);
        win
    }
}
