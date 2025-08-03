use std::{sync::mpsc, time::Duration};

use fltk::{misc::Progress, prelude::WidgetExt};

/// Stores the progress bar that shows the user how far into the audio track they are.
/// The user can also click on the progress bar in order seek to a specific point in the audio
pub struct ProgressBar {
    progress_bar: Progress,

    /// The receiver that will receive the audio's current position, and update accordingly
    audio_pos_receiver: mpsc::Receiver<Duration>,

    current_audio_pos: Duration,
}

impl ProgressBar {
    pub fn new(
        win_width: i32,
        audio_length: Duration,
        audio_pos_receiver: mpsc::Receiver<Duration>,
    ) -> ProgressBar {
        const WIDTH: i32 = 250;
        const PROGRESS_BAR_Y: i32 = 190;

        let progress_bar_x = (win_width - WIDTH) / 2; // Center the progress bar horizontally

        let mut progress_bar = Progress::default()
            .with_pos(progress_bar_x, PROGRESS_BAR_Y)
            .with_size(WIDTH, 5);

        // Set the range to be from 0 - audio length so progress bar value can simply be set to current position without doing any calculations
        progress_bar.set_minimum(0.0);
        progress_bar.set_maximum(audio_length.as_millis() as f64);

        ProgressBar {
            progress_bar,
            audio_pos_receiver,
            current_audio_pos: Duration::from_secs(0),
        }
    }

    /// Update the progress bar based on the audio's current position.
    pub fn update(&mut self) {
        // Drain all available positions and keep the newest one, so the progress bar never lags behind
        while let Ok(pos) = self.audio_pos_receiver.try_recv() {
            self.current_audio_pos = pos;
        }

        self.progress_bar
            .set_value(self.current_audio_pos.as_millis() as f64);
    }
}
