use lofty::error::{ErrorKind, LoftyError};
use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use lofty::tag::{Accessor, Tag};

struct NowPlaying {}

impl NowPlaying {
    pub fn new() -> NowPlaying {
        NowPlaying {}
    }

    /// Parse an audio file's metadata, and return the primary tag. If the primary tag is not found, it will return the first tag.
    /// These tags contain details about the audio, such as the title, artist, etc.
    /// # Errors
    /// - If `path` does not exist
    /// - If the reader contains invalid data
    /// - If the audio file does not contain a primary tag or a first tag
    pub fn parse_file(path: &str) -> Result<Tag, LoftyError> {
        let tagged_file = read_from_path(path)?;

        // Get the primary tag, and if primary tag is not found, fall back to first tag
        let tag = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())
            .ok_or_else(|| LoftyError::new(ErrorKind::FakeTag))?;

        // Fail if no title or no artist
        if tag.title().is_none() || tag.artist().is_none() {
            return Err(LoftyError::new(ErrorKind::FakeTag));
        }

        Ok(tag.clone())
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
            let full_path = format!("{}/with-metadata/{}", TEST_DATA, filename);
            let primary_tag = NowPlaying::parse_file(&full_path).unwrap();

            assert_eq!(primary_tag.title().unwrap(), expected_title);
            assert_eq!(primary_tag.artist().unwrap(), expected_artist);
        }

        /// Check that a file with no metadata returns the correct error
        fn assert_no_metadata(filename: &str) {
            let full_path = format!("{}/without-metadata/{}", TEST_DATA, filename);
            let result = NowPlaying::parse_file(&full_path);

            assert!(result.is_err());

            if let Err(err) = result {
                match err.kind() {
                    ErrorKind::FakeTag => (),
                    _ => panic!("Error was incorrect: {:?}", err.kind()),
                }
            }
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
        fn parse_no_metadata_mp3_file() {
            assert_no_metadata("test.mp3");
        }

        #[test]
        fn parse_no_metadata_ogg_file() {
            assert_no_metadata("test.ogg");
        }

        #[test]
        fn parse_no_metadata_wav_file() {
            assert_no_metadata("test.wav");
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
