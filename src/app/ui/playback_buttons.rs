use std::{sync::mpsc, time::Duration};

use fltk::{button, prelude::*};

use crate::app::Message;

/// A struct to create the playback buttons: the play, fast-forward, and rewind buttons.
pub struct PlaybackButtons {}

impl PlaybackButtons {
    const SEEK_DURATION_SECS: u64 = 5;

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
    fn style_button(mut btn: button::Button) -> button::Button {
        // Remove focus border around button
        btn.clear_visible_focus();

        // Remove button background
        btn.set_frame(fltk::enums::FrameType::NoBox);

        btn
    }

    /// Create the play button and theme it.
    fn create_play_button(btn_size: i32, btn_x: i32, btn_y: i32, sender: mpsc::Sender<Message>) {
        const PLAY_BUTTON: &str = "";
        const PAUSE_BUTTON: &str = "";

        let mut btn = PlaybackButtons::style_button(
            button::Button::default()
                .with_size(btn_size, btn_size)
                .with_pos(btn_x, btn_y)
                .with_label(PAUSE_BUTTON),
        );

        // Define a function to execute once the button is clicked
        btn.set_callback(move |btn| {
            // Play/pause audio
            match btn.label().as_str() {
                PAUSE_BUTTON => {
                    btn.set_label(PLAY_BUTTON);

                    // Send a message to the audio thread to pause the audio
                    match sender.send(Message::Pause) {
                        Ok(_) => (),
                        Err(e) => println!("Unable to play audio: {:?}", e),
                    };
                }
                PLAY_BUTTON => {
                    btn.set_label(PAUSE_BUTTON);

                    // Send a message to the audio thread to play the audio
                    match sender.send(Message::Play) {
                        Ok(_) => (),
                        Err(e) => println!("Unable to play audio: {:?}", e),
                    };
                }
                _ => unreachable!(),
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
            button::Button::default()
                .with_size(btn_size, btn_size)
                .with_pos(btn_x, btn_y)
                .with_label("󰵱"),
        );

        seek_forwards_btn.set_callback(move |_| {
            // Send a fast-forward message to the audio thread
            match sender.send(Message::FastForward(Duration::from_secs(
                Self::SEEK_DURATION_SECS,
            ))) {
                Ok(_) => (),
                Err(e) => println!("Unable to fast-forward: {:?}", e),
            }
        });
    }

    /// Create the rewind button.
    fn create_rewind_button(btn_size: i32, btn_x: i32, btn_y: i32, sender: mpsc::Sender<Message>) {
        let mut seek_backwards_btn = PlaybackButtons::style_button(
            button::Button::default()
                .with_size(btn_size, btn_size)
                .with_pos(btn_x, btn_y)
                .with_label("󰴪"),
        );

        seek_backwards_btn.set_callback(move |_| {
            // Send a rewind message to the audio thread
            match sender.send(Message::Rewind(Duration::from_secs(
                Self::SEEK_DURATION_SECS,
            ))) {
                Ok(_) => (),
                Err(e) => println!("Unable to rewind: {:?}", e),
            }
        });
    }
}
