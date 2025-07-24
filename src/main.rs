extern crate audio_player;
use audio_player::App;

use rodio::{Decoder, OutputStream, source::Source};
use std::fs::File;
use std::io::BufReader;
use std::thread;

fn main() {
    let file_path = "test.mp3";
    play_audio(file_path);

    let mut app = App::new();
    app.run();
}

fn play_audio(file_path: &'static str) {
    thread::spawn(move || {
        // Get an output stream handle to the default physical sound device.
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");

        // Load sound from file
        let file = BufReader::new(File::open(file_path).unwrap());

        let _sink = rodio::play(&stream_handle.mixer(), file).unwrap();

        // Since audio plays in seperate thread, block current thread from terminating
        std::thread::sleep(std::time::Duration::from_secs(5));
    });
}
