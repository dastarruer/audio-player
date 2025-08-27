# audio_player

A minimal audio player written in Rust.

# Usage

For now, you will have to add a `test.mp3` file to the project's _root directory_:

```rust
AUDIO_PLAYER/
├── assets/
├── src/
├── target/
├── ...
└── test.mp3
```

Then run:

```sh
cargo run --release
```

This will play `test.mp3` with the audio player.
