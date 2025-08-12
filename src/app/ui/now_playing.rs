use lofty::error::{ErrorKind, LoftyError};
use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use lofty::tag::Tag;

struct NowPlaying {}

impl NowPlaying {
    pub fn new() -> NowPlaying {
        NowPlaying {}
    }

    /// Parse an audio file's metadata, and return the primary tag. This will contain details about the audio, such as the title, artist, etc.
    /// # Errors
    /// - If `path` does not exist
    /// - If the reader contains invalid data
    /// - If the audio file does not contain a primary tag
    pub fn parse_file(path: &str) -> Result<&Tag, LoftyError> {
        let tagged_file = read_from_path(path)?;

        // If the primary tag doesn't exist, or the tag types
        // don't matter, the primary tag can be retrieved
        let primary_tag = tagged_file
            .primary_tag()
            .ok_or_else(|| LoftyError::new(ErrorKind::FakeTag))?;

        Ok(primary_tag)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod parse_file {
        use lofty::tag::Accessor;

        use super::*;

        #[test]
        fn parse_file() {
            let primary_tag = NowPlaying::parse_file("./test.mp3").unwrap();

            assert_eq!(primary_tag.title().unwrap(), "less than lovers")
        }
    }
}
