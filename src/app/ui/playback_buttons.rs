use std::{sync::mpsc, time::Duration};

use fltk::{button::Button, prelude::*};

use crate::app::Message;

/// A struct to create the playback buttons: the play, fast-forward, and rewind buttons.
pub struct PlaybackButtons {}

impl PlaybackButtons {
    const SEEK_DURATION: Duration = Duration::from_secs(5);
    const PLAY_BUTTON: &str = "";
    const PAUSE_BUTTON: &str = "";

    /// Create new playback buttons
    pub fn new(win_width: i32, sender: mpsc::Sender<Message>) -> PlaybackButtons {
        const BTN_SIZE: i32 = 30;
        const BTN_Y: i32 = 200; // Since every button will be at the same y-coordinate, each button shares the same constant
        const BTN_OFFSET: i32 = 100;

        let play_btn_x = (win_width - BTN_SIZE) / 2; // Center the button horizontally
        let fast_forward_btn_x = play_btn_x + BTN_OFFSET;
        let rewind_btn_x = play_btn_x - BTN_OFFSET;

        PlaybackButtons::create_play_button(BTN_SIZE, play_btn_x, BTN_Y, sender.clone());

        PlaybackButtons::create_fast_forward_button(
            BTN_SIZE,
            fast_forward_btn_x,
            BTN_Y,
            sender.clone(),
        );

        PlaybackButtons::create_rewind_button(BTN_SIZE, rewind_btn_x, BTN_Y, sender);

        PlaybackButtons {}
    }

    /// Style each playback button with a unified style
    fn style_button(mut btn: Button) -> Button {
        // Remove focus border around button
        btn.clear_visible_focus();

        // Remove button background
        btn.set_frame(fltk::enums::FrameType::NoBox);

        btn
    }

    /// Create the play button and theme it.
    fn create_play_button(btn_size: i32, btn_x: i32, btn_y: i32, sender: mpsc::Sender<Message>) {
        let mut btn = PlaybackButtons::style_button(
            Button::default()
                .with_size(btn_size, btn_size)
                .with_pos(btn_x, btn_y)
                .with_label(Self::PAUSE_BUTTON),
        );

        // Define a function to execute once the button is clicked
        btn.set_callback(move |btn| {
            // Play/pause audio
            let current_label = btn.label();
            let (new_label, message) = PlaybackButtons::handle_play_pause_click(&current_label);

            // Update the button label
            btn.set_label(new_label);

            // Send a message to the audio thread to play/pause the audio
            if let Err(e) = sender.send(message) {
                eprintln!("Unable to play/pause audio: {:?}", e);
            };
        });
    }

    /// Create the fast-forwards button.
    fn create_fast_forward_button(
        btn_size: i32,
        btn_x: i32,
        btn_y: i32,
        sender: mpsc::Sender<Message>,
    ) {
        let mut seek_forwards_btn = PlaybackButtons::style_button(
            Button::default()
                .with_size(btn_size, btn_size)
                .with_pos(btn_x, btn_y)
                .with_label("󰵱"),
        );

        seek_forwards_btn.set_callback(move |_| {
            // Send a fast-forward message to the audio thread
            if let Err(e) = sender.send(Message::FastForward(Self::SEEK_DURATION)) {
                eprintln!("Unable to fast-forward: {:?}", e);
            }
        });
    }

    /// Create the rewind button.
    fn create_rewind_button(btn_size: i32, btn_x: i32, btn_y: i32, sender: mpsc::Sender<Message>) {
        let mut seek_backwards_btn = PlaybackButtons::style_button(
            Button::default()
                .with_size(btn_size, btn_size)
                .with_pos(btn_x, btn_y)
                .with_label("󰴪"),
        );

        seek_backwards_btn.set_callback(move |_| {
            // Send a rewind message to the audio thread
            if let Err(e) = sender.send(Message::Rewind(Self::SEEK_DURATION)) {
                eprintln!("Unable to rewind: {:?}", e)
            }
        });
    }

    /// Return the corresponding label and Message once the play/pause button is clicked.
    /// For instance, if the audio is paused, the function will return (Self::PAUSE_BUTTON, Message::Play).
    /// However, if the audio is playing, the function will return (Self::PLAY_BUTTON, Message::Pause).
    fn handle_play_pause_click(current_label: &str) -> (&str, Message) {
        match current_label {
            Self::PAUSE_BUTTON => (Self::PLAY_BUTTON, Message::Pause),
            Self::PLAY_BUTTON => (Self::PAUSE_BUTTON, Message::Play),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod handle_play_pause_click {
        use super::*;

        #[test]
        fn test_paused_btn() {
            assert_eq!(
                PlaybackButtons::handle_play_pause_click(PlaybackButtons::PLAY_BUTTON),
                (PlaybackButtons::PAUSE_BUTTON, Message::Play),
            );
        }

        #[test]
        fn test_play_btn() {
            assert_eq!(
                PlaybackButtons::handle_play_pause_click(PlaybackButtons::PAUSE_BUTTON),
                (PlaybackButtons::PLAY_BUTTON, Message::Pause)
            );
        }
    }
}
