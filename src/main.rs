extern crate audio_player;
use audio_player::App;

use rodio::{Decoder, OutputStream, source::Source};
use std::fs::File;
use std::io::BufReader;

fn main() {
    let mut app = App::new();
    app.run();

    // Get an output stream handle to the default physical sound device.
    // Note that the playback stops when the stream_handle is dropped.
    let stream_handle =
        rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");

    // Load a sound from a file, using a path relative to Cargo.toml
    let file = BufReader::new(File::open("test.mp3").unwrap());
    // Note that the playback stops when the sink is dropped
    let sink = rodio::play(&stream_handle.mixer(), file).unwrap();

    // The sound plays in a separate audio thread,
    // so we need to keep the main thread alive while it's playing.
    std::thread::sleep(std::time::Duration::from_secs(5));
}
