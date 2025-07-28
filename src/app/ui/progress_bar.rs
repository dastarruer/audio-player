use std::{sync::mpsc, thread, time::Duration};

use fltk::{misc::Progress, prelude::WidgetExt};

/// Stores the progress bar that shows the user how far into the audio track they are.
/// The user can also click on the progress bar in order seek to a specific point in the audio
pub struct ProgressBar {
    _progress_bar: Progress,

    /// The receiver that will receive the audio's current position, and update accordingly
    audio_pos_receiver: mpsc::Receiver<Duration>,
}

impl ProgressBar {
    pub fn new(win_width: i32, audio_pos_receiver: mpsc::Receiver<Duration>) -> ProgressBar {
        const WIDTH: i32 = 250;
        const PROGRESS_BAR_Y: i32 = 195;

        let progress_bar_x = (win_width - WIDTH) / 2; // Center the progress bar horizontally

        let mut _progress_bar = Progress::default()
            .with_pos(progress_bar_x, PROGRESS_BAR_Y)
            .with_size(WIDTH, 5);

        _progress_bar.set_value(15.0);

        ProgressBar { _progress_bar, audio_pos_receiver }
    }

    /// Run the progress bar in a seperate thread. This will initiate the progress updating logic based on the audio's current position
    pub fn run(&self) {
        thread::spawn(|| {
            loop {
                println!("i am progressing as we speak...");
            };
        });
    }
}
