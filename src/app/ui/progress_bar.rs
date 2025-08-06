use std::{sync::mpsc, time::Duration};

use fltk::{
    draw,
    enums::{Color, Font, FrameType},
    misc::Progress,
    output,
    prelude::{WidgetBase, WidgetExt},
};

/// Stores the progress bar that shows the user how far into the audio track they are.
/// The user can also click on the progress bar in order seek to a specific point in the audio
pub struct ProgressBar {
    progress_bar: Progress,

    /// The receiver that will receive the audio's current position, and update accordingly
    audio_pos_receiver: mpsc::Receiver<Duration>,

    audio_length: Duration,

    current_audio_pos: Duration,

    /// Display the audio's current position to the user
    current_audio_pos_timestamp: output::Output,
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
        progress_bar.set_value(0.0);

        let (current_audio_pos_timestamp, _) =
            ProgressBar::create_timestamps(&progress_bar, audio_length);

        ProgressBar {
            progress_bar,
            audio_pos_receiver,
            current_audio_pos: Duration::from_secs(0),
            audio_length,
            current_audio_pos_timestamp,
        }
    }

    /// Update the progress bar based on the audio's current position.
    pub fn update(&mut self) {
        // Drain all available positions and keep the newest one, so the progress bar never lags behind
        while let Ok(pos) = self.audio_pos_receiver.try_recv() {
            if self.audio_length < pos {
                self.current_audio_pos = self.audio_length;
            } else {
                self.current_audio_pos = pos;
            }
        }

        // Draw the knob first so that it is drawn over the progress bar
        let diameter = 10;
        let knob_x = self.knob_x();
        println!("{}", knob_x);
        let knob_y = self.progress_bar.y() - 2;
        self.progress_bar.draw(move |_| {
            draw::draw_circle_fill(knob_x, knob_y, diameter, Color::Blue);
        });

        self.current_audio_pos_timestamp
            .set_label(&ProgressBar::format_duration(self.current_audio_pos));
        self.progress_bar
            .set_value(self.current_audio_pos.as_millis() as f64);
    }

    /// Create the timestamps on both sides of the progress bar.
    fn create_timestamps(
        progress_bar: &Progress,
        audio_length: Duration,
    ) -> (output::Output, output::Output) {
        const TIMESTAMP_WIDTH: i32 = 30;
        const TIMESTAMP_HEIGHT: i32 = 1;

        // The padding is different for both because god willed it to be
        const CURRENT_AUDIO_POS_TIMESTAMP_PADDING: i32 = -25;
        const TOTAL_AUDIO_DURATION_TIMESTAMP_PADDING: i32 = 40;

        // Create the timestamp to show the viewer the total duration of the audio
        let foramtted_duration = &ProgressBar::format_duration(audio_length);
        let mut total_audio_duration_timestamp = output::Output::default()
            .with_size(TIMESTAMP_WIDTH, TIMESTAMP_HEIGHT)
            .right_of(progress_bar, TOTAL_AUDIO_DURATION_TIMESTAMP_PADDING)
            .center_y(progress_bar)
            .with_label(foramtted_duration);
        total_audio_duration_timestamp.set_label_font(Font::Helvetica);
        total_audio_duration_timestamp.set_frame(FrameType::NoBox);

        // Create the timestamp to show the viewer the current audio position
        let default_timestamp = "0:00";
        let mut current_audio_pos_timestamp = output::Output::default()
            .with_size(TIMESTAMP_WIDTH, TIMESTAMP_HEIGHT)
            .left_of(progress_bar, CURRENT_AUDIO_POS_TIMESTAMP_PADDING)
            .center_y(progress_bar)
            .with_label(default_timestamp);
        current_audio_pos_timestamp.set_label_font(Font::Helvetica);
        current_audio_pos_timestamp.set_frame(FrameType::NoBox);

        (current_audio_pos_timestamp, total_audio_duration_timestamp)
    }

    /// Format a Duration as mm:ss
    fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();

        let hours = total_secs / 3600;
        let rem_secs = total_secs % 3600;

        let minutes = rem_secs / 60;
        let seconds = rem_secs % 60;

        if hours > 0 {
            return format!("{}:{:02}:{:02}", hours, minutes, seconds);
        }
        format!("{}:{:02}", minutes, seconds)
    }

    /// Get the x position of the progress bar knob
    fn knob_x(&self) -> i32 {
        let progress = (self.current_audio_pos.as_millis() as i32) / (self.audio_length.as_millis() as i32) as i32;
        let progress_bar_width = self.progress_bar.width() as f64;

        let progress_offset = (progress as f64 * progress_bar_width) as i32;

        self.progress_bar.x() + progress_offset
    }
}

#[cfg(test)]
mod test {
    mod format_duration {
        use super::super::*;

        #[test]
        fn test_format_duration_minutes() {
            let duration = Duration::from_secs(61);
            assert_eq!("1:01", ProgressBar::format_duration(duration));

            let duration = Duration::from_secs(158);
            assert_eq!("2:38", ProgressBar::format_duration(duration));
        }

        #[test]
        fn test_format_duration_hours() {
            let duration = Duration::from_secs(3601);
            assert_eq!("1:00:01", ProgressBar::format_duration(duration));
        }
    }

    mod knob_x {
        use super::super::*;

        // const DIAMETER: i32 = 10;

        #[test]
        fn test_0_progress() {
            let (_, rx) = mpsc::channel();
            let progress = ProgressBar::new(400, Duration::from_secs(15), rx);

            assert_eq!(progress.knob_x(), 75);
        }

        #[test]
        fn test_25_progress() {
            let (_, rx) = mpsc::channel();
            let mut progress = ProgressBar::new(400, Duration::from_secs(100), rx);
            progress.progress_bar.set_value(25.0);

            assert_eq!(progress.knob_x(), 137);
        }

        #[test]
        fn test_50_progress() {
            let (_, rx) = mpsc::channel();
            let mut progress = ProgressBar::new(400, Duration::from_secs(100), rx);
            progress.progress_bar.set_value(50.0);

            assert_eq!(progress.knob_x(), 200);
        }

        #[test]
        fn test_75_progress() {
            let (_, rx) = mpsc::channel();
            let mut progress = ProgressBar::new(400, Duration::from_secs(100), rx);
            progress.progress_bar.set_value(75.0);

            assert_eq!(progress.knob_x(), 262);
        }

        #[test]
        fn test_100_progress() {
            let (_, rx) = mpsc::channel();
            let mut progress = ProgressBar::new(400, Duration::from_secs(100), rx);
            progress.progress_bar.set_value(100.0);

            assert_eq!(progress.knob_x(), 325);
        }
    }

    mod progress_as_decimal {
        use super::super::*;

        #[test]
        fn test_0_percent() {
            let (_, rx) = mpsc::channel();
            let progress = ProgressBar::new(400, Duration::from_secs(100), rx);

            assert_eq!(progress.progress_as_decimal(), 0.0);
        }

        #[test]
        fn test_25_percent() {
            let (_, rx) = mpsc::channel();
            let mut progress = ProgressBar::new(400, Duration::from_secs(100), rx);
            progress.progress_bar.set_value(25.0);

            assert_eq!(progress.progress_as_decimal(), 0.25);
        }

        #[test]
        fn test_50_percent() {
            let (_, rx) = mpsc::channel();
            let mut progress = ProgressBar::new(400, Duration::from_secs(100), rx);
            progress.progress_bar.set_value(50.0);

            assert_eq!(progress.progress_as_decimal(), 0.50);
        }

        #[test]
        fn test_75_percent() {
            let (_, rx) = mpsc::channel();
            let mut progress = ProgressBar::new(400, Duration::from_secs(100), rx);
            progress.progress_bar.set_value(75.0);

            assert_eq!(progress.progress_as_decimal(), 0.75);
        }

        #[test]
        fn test_100_percent() {
            let (_, rx) = mpsc::channel();
            let mut progress = ProgressBar::new(400, Duration::from_secs(100), rx);
            progress.progress_bar.set_value(100.0);

            assert_eq!(progress.progress_as_decimal(), 1.0);
        }
    }
}
