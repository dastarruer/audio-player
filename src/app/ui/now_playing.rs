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
    pub fn parse_file(path: &str) -> Result<Tag, LoftyError> {
        let tagged_file = read_from_path(path)?;

        // If the primary tag doesn't exist, or the tag types
        // don't matter, the primary tag can be retrieved
        let primary_tag = tagged_file
            .primary_tag()
            .ok_or_else(|| LoftyError::new(ErrorKind::FakeTag))?;

        Ok(primary_tag.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod parse_file {
        use super::*;
        use lofty::tag::Accessor;

        const TEST_DATA: &str = "./src/app/ui/test_files";

        #[test]
        fn parse_valid_mp3_file() {
            let binding = format!("{}/test.mp3", TEST_DATA);
            let path = binding.as_str();

            let primary_tag = NowPlaying::parse_file(path).unwrap();

            assert_eq!(primary_tag.title().unwrap(), "less than lovers");
            assert_eq!(primary_tag.artist().unwrap(), "Kensuke Ushio");
        }

        #[test]
        fn parse_valid_flac_file() {
            let binding = format!("{}/test.flac", TEST_DATA);
            let path = binding.as_str();

            let primary_tag = NowPlaying::parse_file(path).unwrap();

            assert_eq!(primary_tag.title().unwrap(), "less than lovers");
            assert_eq!(primary_tag.artist().unwrap(), "Kensuke Ushio");
        }

        #[test]
        fn parse_non_existent_file() {
            let binding = format!("{}/does_not_exist.mp3", TEST_DATA);
            let path = binding.as_str();

            let invalid_file = NowPlaying::parse_file(path);
            assert!(invalid_file.is_err());
        }
    }
}
