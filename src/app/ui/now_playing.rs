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

        /// A testing function to check that the metadata for an audio file is being parsed properly
        fn assert_metadata(filename: &str, expected_title: &str, expected_artist: &str) {
            let full_path = format!("{}/{}", TEST_DATA, filename);
            let primary_tag = NowPlaying::parse_file(&full_path).unwrap();

            assert_eq!(primary_tag.title().unwrap(), expected_title);
            assert_eq!(primary_tag.artist().unwrap(), expected_artist);
        }

        #[test]
        fn parse_valid_mp3_file() {
            assert_metadata("test.mp3", "less than lovers", "Kensuke Ushio");
        }

        #[test]
        fn parse_valid_ogg_file() {
            assert_metadata("test.flac", "less than lovers", "Kensuke Ushio");
        }

        #[test]
        fn parse_valid_wav_file() {
            assert_metadata("test.wav", "less than lovers", "Kensuke Ushio");
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
