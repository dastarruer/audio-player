mod audio_handler;
mod ui;

use fltk::{app, enums::Color, prelude::*, window};

use std::process::exit;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
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
        // Create a channel to send messages to the audio thread, allowing ui elements to do things such as pause, play, rewind, etc.
        let (sender, recevier) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(recevier));

        // Create the channel for the progress bar and audio sink to communicate the audio position to each other
        let (audio_pos_sender, audio_pos_receiver) = mpsc::channel::<Duration>();

        // Load the audio
        let (decoder, audio_length) = AudioHandler::load_audio(
            "/home/dastarruer/Documents/coding/rust/audio_player/test.mp3",
        );
        let audio_length = audio_length.unwrap_or_else(|| {
            eprintln!("Duration could not be determined. Exiting...");
            exit(1);
        });

        // Create the components
        self.create_app_components(sender, audio_length, audio_pos_receiver);

        // Show the window
        self.window.end();
        self.window.show();

        // Play the audio
        self.audio_handler
            .play_audio(Arc::clone(&receiver), audio_pos_sender, decoder);

        // Run the app
        while self.app.wait() {
            // Sleep thread so that fltk updates even when idling
            thread::sleep(Duration::from_millis(50));

            // Update progress bar
            if let Some(pb) = self.progress_bar.as_mut() {
                pb.update();
            }
        }
    }

    /// Create all the necessary app components, such as the playback buttons, etc.
    fn create_app_components(
        &mut self,
        sender: mpsc::Sender<Message>,
        audio_length: Duration,
        audio_pos_receiver: mpsc::Receiver<Duration>,
    ) {
        self.playback_buttons = Some(PlaybackButtons::new(AudioApp::WIN_WIDTH, sender.clone()));
        self.progress_bar = Some(ProgressBar::new(
            AudioApp::WIN_WIDTH,
            audio_length,
            audio_pos_receiver,
            sender,
        ));
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
