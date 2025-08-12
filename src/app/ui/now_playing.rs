use lofty::error::{ErrorKind, LoftyError};
use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use lofty::tag::Accessor;

struct NowPlaying {}

impl NowPlaying {
    pub fn new() -> NowPlaying {
        NowPlaying {}
    }

    /// Parse an audio file's metadata, and return the first tag. This will contain details about the audio, such as the title, artist, etc.
    /// # Errors
    /// - If `path` does not exist
    /// - If the reader contains invalid data
    /// - If the audio file does not contain a first tag
    pub fn parse_file(path: &str) -> Result<(), LoftyError> {
        let tagged_file = read_from_path(path)?;

        // If the primary tag doesn't exist, or the tag types
        // don't matter, the first tag can be retrieved
        let unknown_first_tag = tagged_file
            .first_tag()
            .ok_or_else(|| LoftyError::new(ErrorKind::FakeTag))?;

        println!("unknown first tag: {}", unknown_first_tag.title().unwrap());

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod parse_file {
        use super::*;

        #[test]
        fn parse_file() {
            NowPlaying::parse_file("./test.mp3");
            assert_eq!(1, 1)
        }
    }
}
