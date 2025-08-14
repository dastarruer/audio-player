use fltk::frame::Frame;
use fltk::image::SharedImage;
use fltk::prelude::{WidgetBase, WidgetExt};
use lofty::error::{ErrorKind, LoftyError};
use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use lofty::tag::{Accessor, Tag};

pub struct NowPlaying {}

impl NowPlaying {
    pub fn new() -> NowPlaying {
        // Load image from file
        let img = SharedImage::load("test.png").expect("Could not load image");

        let mut frame = Frame::new(0, 0, 100, 100, "");

        // Assign the image to the frame
        // Use set_image_scaled so that the image scales to the widget's size
        frame.set_image_scaled(Some(img));

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
        use std::path::Path;

        use super::*;
        use lofty::tag::Accessor;

        const TEST_DATA: &str = "./src/app/ui/test_files";

        /// A testing function to check that the metadata for an audio file is being parsed properly
        fn assert_metadata(filename: &str, expected_title: &str, expected_artist: &str) {
            let relative_path = format!("{}/with-metadata/{}", TEST_DATA, filename);

            let full_path = Path::new(&relative_path)
                .canonicalize()
                .expect("Failed to resolve absolute path");

            let primary_tag = NowPlaying::parse_file(full_path.to_str().unwrap()).unwrap();

            assert_eq!(primary_tag.title().unwrap(), expected_title);
            assert_eq!(primary_tag.artist().unwrap(), expected_artist);

            assert_ne!(primary_tag.picture_count(), 0);
        }

        /// Check that a file with no metadata returns the correct error
        fn assert_no_metadata(filename: &str) {
            let relative_path = format!("{}/without-metadata/{}", TEST_DATA, filename);

            let full_path = Path::new(&relative_path)
                .canonicalize()
                .expect("Failed to resolve absolute path");

            let result = NowPlaying::parse_file(full_path.to_str().unwrap());

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
        fn parse_no_metadata_mp3_file() {
            assert_no_metadata("test.mp3");
        }

        #[test]
        fn parse_no_metadata_ogg_file() {
            assert_no_metadata("test.ogg");
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
