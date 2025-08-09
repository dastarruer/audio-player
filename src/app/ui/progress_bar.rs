use std::{cell::RefCell, rc::Rc, sync::mpsc, time::Duration};

use fltk::{
    app::{self, MouseButton},
    draw,
    enums::{Color, Event, Font, FrameType},
    frame::Frame,
    misc::Progress,
    output,
    prelude::{WidgetBase, WidgetExt},
};

use crate::app::Message;

/// Stores the progress bar that shows the user how far into the audio track they are.
/// The user can also click on the progress bar in order seek to a specific point in the audio
pub struct ProgressBar {
    progress_bar: Rc<RefCell<Progress>>,

    /// The receiver that will receive the audio's current position, and update accordingly
    audio_pos_receiver: mpsc::Receiver<Duration>,

    audio_length: Duration,

    current_audio_pos: Rc<RefCell<Duration>>,

    /// Display the audio's current position to the user
    current_audio_pos_timestamp: output::Output,

    /// The overlay that is used to draw the knob on top of the progress bar
    knob_overlay: Rc<RefCell<Frame>>,

    /// The sender that will be used to rewind or fast-forward the audio when the progress bar is clicked
    audio_sender: mpsc::Sender<Message>,
}

impl ProgressBar {
    pub fn new(
        win_width: i32,
        audio_length: Duration,
        audio_pos_receiver: mpsc::Receiver<Duration>,
        audio_sender: mpsc::Sender<Message>,
    ) -> ProgressBar {
        const WIDTH: i32 = 250;
        const PROGRESS_BAR_Y: i32 = 190;
        const KNOB_Y_OFFSET: i32 = -2;

        let progress_bar_x = (win_width - WIDTH) / 2; // Center the progress bar horizontally

        let progress_bar = Rc::new(RefCell::new(
            Progress::default()
                .with_pos(progress_bar_x, PROGRESS_BAR_Y)
                .with_size(WIDTH, 5),
        ));

        // Set the range to be from 0 - audio length so progress bar value can simply be set to current position without doing any calculations
        progress_bar.borrow_mut().set_minimum(0.0);
        progress_bar
            .borrow_mut()
            .set_maximum(audio_length.as_millis() as f64);
        progress_bar.borrow_mut().set_value(0.0);

        let (current_audio_pos_timestamp, _) =
            ProgressBar::create_timestamps(&progress_bar.borrow(), audio_length);

        let knob_overlay = Rc::new(RefCell::new(Frame::new(
            progress_bar_x,
            PROGRESS_BAR_Y - 10,
            WIDTH,
            20,
            "",
        )));

        let current_audio_pos = Rc::new(RefCell::new(Duration::from_secs(0)));

        let progress = ProgressBar {
            progress_bar,
            audio_pos_receiver,
            current_audio_pos,
            audio_length,
            current_audio_pos_timestamp,
            knob_overlay,
            audio_sender,
        };

        let diameter = 10;
        let knob_y = progress.progress_bar.borrow().y() + KNOB_Y_OFFSET;

        // Clone/copy a bunch of values that will be moved into the handle closure
        let knob_overlay = Rc::clone(&progress.knob_overlay);
        let audio_sender = progress.audio_sender.clone();
        let audio_length = progress.audio_length;
        let current_audio_pos = Rc::clone(&progress.current_audio_pos);
        let progress_bar = Rc::clone(&progress.progress_bar);

        // Handle hovering over progress bar
        progress
            .knob_overlay
            .borrow_mut()
            .handle(move |_, event| match event {
                Event::Enter => {
                    // Clone the reference again so it can then be moved into the draw closure
                    let progress_bar = Rc::clone(&progress_bar);

                    // Update the knob overlay's draw function to draw the knob
                    knob_overlay.borrow_mut().draw(move |_| {
                        // Update knob_x
                        let knob_x = ProgressBar::knob_x(&progress_bar.borrow());

                        // Draw the knob
                        draw::draw_circle_fill(knob_x, knob_y, diameter, Color::gray_ramp(1));
                    });
                    true
                }
                Event::Leave => {
                    // Update the knob overlay's draw function to draw nothing
                    knob_overlay.borrow_mut().draw(move |_| {});
                    true
                }
                Event::Push if app::event_mouse_button() == MouseButton::Left => {
                    // Borrow progress_bar and current_audio_pos once here so we don't have to do it multiple times
                    let progress_bar = progress_bar.borrow();
                    let current_audio_pos = *current_audio_pos.borrow();

                    let mouse_x = app::event_x();
                    let progress_bar_x = progress_bar.x();
                    let progress_bar_width = progress_bar.width();

                    // Get position relative to progress bar, and ensure value is never less than 0 or bigger than progress bar width
                    let rel_x = (mouse_x - progress_bar_x).max(0).min(progress_bar_width);
                    let percentage = rel_x as f64 / progress_bar_width as f64;

                    // Convert percentage to target position
                    let position_to_seek = audio_length.mul_f64(percentage);

                    // Compute how far to jump (positive = forward, negative = backward)
                    if position_to_seek > current_audio_pos {
                        let distance = position_to_seek - current_audio_pos;
                        match audio_sender.send(Message::FastForward(distance)) {
                            Ok(_) => true,
                            Err(e) => {
                                eprintln!("Unable to seek to position: {}", e);
                                false
                            }
                        }
                    } else {
                        let distance = current_audio_pos - position_to_seek;
                        match audio_sender.send(Message::Rewind(distance)) {
                            Ok(_) => true,
                            Err(e) => {
                                eprintln!("Unable to seek to position: {}", e);
                                false
                            }
                        }
                    }
                }
                _ => false,
            });

        progress
    }

    /// Update the progress bar based on the audio's current position.
    /// This function is intended to be called continuously in the app's main loop.
    pub fn update(&mut self) {
        // Drain all available positions and keep the newest one, so the progress bar never lags behind
        while let Ok(pos) = self.audio_pos_receiver.try_recv() {
            // Ensure that current_audio_pos never goes over audio_length
            *self.current_audio_pos.borrow_mut() = pos.clamp(Duration::ZERO, self.audio_length);
        }

        // Draw the knob
        self.knob_overlay.borrow_mut().redraw();

        // Update the timestamp
        self.current_audio_pos_timestamp
            .set_label(&ProgressBar::format_duration(*self.current_audio_pos.borrow()));

        // Update the progress bar
        self.progress_bar
            .borrow_mut()
            .set_value(self.current_audio_pos.borrow().as_millis() as f64);
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
    fn knob_x(progress_bar: &Progress) -> i32 {
        let progress = progress_bar.value() / progress_bar.maximum();
        let progress_bar_width = progress_bar.width() as f64;
        let progress_offset = (progress * progress_bar_width) as i32;

        progress_bar.x() + progress_offset
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl Default for ProgressBar {
        /// Initialize a dummy ProgressBar for testing
        fn default() -> ProgressBar {
            let (_, rx) = mpsc::channel();
            let (tx, _) = mpsc::channel();

            ProgressBar::new(400, Duration::from_millis(100), rx, tx)
        }
    }
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
            let progress = ProgressBar::default();

            assert_eq!(ProgressBar::knob_x(&progress.progress_bar.borrow()), 75);
        }

        #[test]
        fn test_25_progress() {
            let progress = ProgressBar::default();
            progress.progress_bar.borrow_mut().set_value(25.0);

            assert_eq!(ProgressBar::knob_x(&progress.progress_bar.borrow()), 137);
        }

        #[test]
        fn test_50_progress() {
            let progress = ProgressBar::default();
            progress.progress_bar.borrow_mut().set_value(50.0);

            assert_eq!(ProgressBar::knob_x(&progress.progress_bar.borrow()), 200);
        }

        #[test]
        fn test_75_progress() {
            let progress = ProgressBar::default();
            progress.progress_bar.borrow_mut().set_value(75.0);

            assert_eq!(ProgressBar::knob_x(&progress.progress_bar.borrow()), 262);
        }

        #[test]
        fn test_100_progress() {
            let progress = ProgressBar::default();
            progress.progress_bar.borrow_mut().set_value(100.0);

            assert_eq!(ProgressBar::knob_x(&progress.progress_bar.borrow()), 325);
        }
    }
}
