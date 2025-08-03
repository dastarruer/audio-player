use std::{sync::mpsc, time::Duration};

use fltk::{frame, misc::Progress, prelude::WidgetExt};

/// Stores the progress bar that shows the user how far into the audio track they are.
/// The user can also click on the progress bar in order seek to a specific point in the audio
pub struct ProgressBar {
    progress_bar: Progress,

    /// The receiver that will receive the audio's current position, and update accordingly
    audio_pos_receiver: mpsc::Receiver<Duration>,

    current_audio_pos: Duration,

    /// Display the audio's current position to the user
    current_audio_pos_text: frame::Frame,
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

        const TIMESTAMP_PADDING: i32 = 10;
        const TIMESTAMP_WIDTH: i32 = 30;
        const TIMESTAMP_HEIGHT: i32 = 1;

        // Create the timestamp to show the viewer the total duration of the audio
        frame::Frame::default()
            .with_size(TIMESTAMP_WIDTH, TIMESTAMP_HEIGHT)
            .right_of(&progress_bar, TIMESTAMP_PADDING)
            .center_y(&progress_bar)
            .with_label(&ProgressBar::format_duration(&audio_length));

        // Create the timestamp to show the viewer the current audio position
        let default_timestamp = "0:00";
        let current_audio_pos_text = frame::Frame::default()
            .with_size(TIMESTAMP_WIDTH, TIMESTAMP_HEIGHT)
            .left_of(&progress_bar, TIMESTAMP_PADDING)
            .center_y(&progress_bar)
            .with_label(default_timestamp);

        // Set the range to be from 0 - audio length so progress bar value can simply be set to current position without doing any calculations
        progress_bar.set_minimum(0.0);
        progress_bar.set_maximum(audio_length.as_millis() as f64);

        ProgressBar {
            progress_bar,
            audio_pos_receiver,
            current_audio_pos: Duration::from_secs(0),
            current_audio_pos_text,
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

    /// Format a Duration as mm:ss
    fn format_duration(duration: &Duration) -> String {
        let secs = duration.as_secs();
        let minutes = secs / 60;
        let seconds = secs % 60;

        // If minutes has two digits
        if minutes >= 10 {
            // Return as is
            return format!("{}:{}", minutes, seconds);
        } else {
            // If minutes has only one digit, add a leading zero
            return format!("{}:0{}", minutes, seconds);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_format_duration_minutes() {
        let duration = Duration::from_secs(61);
        assert_eq!("1:01", ProgressBar::format_duration(&duration));
    }

    #[test]
    fn test_format_duration_hours() {
        let duration = Duration::from_secs(3601);
        assert_eq!("1:00:01", ProgressBar::format_duration(&duration));
    }
}
