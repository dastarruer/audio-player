use std::{sync::mpsc, time::Duration};

use fltk::{button::Button, group, prelude::*};

use crate::app::Message;

/// A struct to create the playback buttons: the play, fast-forward, and rewind buttons.
pub struct PlaybackButtons {}

impl PlaybackButtons {
    const SEEK_DURATION: Duration = Duration::from_secs(5);
    const PLAY_BUTTON: &str = "";
    const PAUSE_BUTTON: &str = "";

    /// Create new playback buttons
    pub fn new(win_width: i32, sender: mpsc::Sender<Message>) -> PlaybackButtons {
        const FLEX_WIDTH: i32 = 250;
        const FLEX_Y: i32 = 210;

        const BTN_OFFSET: i32 = 100;

        let play_btn_x = (win_width - 20) / 2; // Center the button horizontally
        let flex_x = play_btn_x - BTN_OFFSET;

        let mut flex = group::Flex::default()
            .with_pos(flex_x, FLEX_Y)
            .with_size(FLEX_WIDTH, 10)
            .row();

        let rewind_btn = PlaybackButtons::create_rewind_button(sender.clone());

        let play_btn = PlaybackButtons::create_play_button(sender.clone());

        let fast_forward_btn = PlaybackButtons::create_fast_forward_button(sender);

        flex.fixed(&rewind_btn, 55);
        flex.fixed(&play_btn, 100);
        flex.fixed(&fast_forward_btn, 55);
        flex.end();
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
    fn create_play_button(sender: mpsc::Sender<Message>) -> Button {
        let mut play_btn =
            PlaybackButtons::style_button(Button::default().with_label(Self::PAUSE_BUTTON));

        // Define a function to execute once the button is clicked
        play_btn.set_callback(move |btn| {
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

        play_btn
    }

    /// Create the fast-forwards button.
    fn create_fast_forward_button(sender: mpsc::Sender<Message>) -> Button {
        let mut seek_forwards_btn =
            PlaybackButtons::style_button(Button::default().with_label("󰵱"));

        seek_forwards_btn.set_callback(move |_| {
            // Send a fast-forward message to the audio thread
            if let Err(e) = sender.send(Message::FastForward(Self::SEEK_DURATION)) {
                eprintln!("Unable to fast-forward: {:?}", e);
            }
        });

        seek_forwards_btn
    }

    /// Create the rewind button.
    fn create_rewind_button(sender: mpsc::Sender<Message>) -> Button {
        let mut seek_backwards_btn =
            PlaybackButtons::style_button(Button::default().with_label("󰴪"));

        seek_backwards_btn.set_callback(move |_| {
            // Send a rewind message to the audio thread
            if let Err(e) = sender.send(Message::Rewind(Self::SEEK_DURATION)) {
                eprintln!("Unable to rewind: {:?}", e)
            }
        });

        seek_backwards_btn
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
