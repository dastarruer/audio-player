use fltk::frame::Frame;
use fltk::image::{JpegImage, PngImage, SharedImage};
use fltk::prelude::{ImageExt, WidgetBase, WidgetExt};
use lofty::error::{ErrorKind, LoftyError};
use lofty::file::TaggedFileExt;
use lofty::picture::MimeType;
use lofty::read_from_path;
use lofty::tag::{Accessor, Tag};

pub struct NowPlaying {}

impl NowPlaying {
    pub fn new(path: &str) -> NowPlaying {
        let mut cover_image_widget = Frame::new(0, 0, 100, 100, "");

        let metadata_tag = NowPlaying::parse_file(path).unwrap();
        let cover_image = NowPlaying::extract_cover_image_from_tag(&metadata_tag);

        // Load image from file
        let img = SharedImage::from_image(&cover_image).expect("Could not load image");

        // Assign the image to the frame
        // Use set_image_scaled so that the image scales to the widget's size
        cover_image_widget.set_image_scaled(Some(img));

        NowPlaying {}
    }

    /// Parse an audio file's metadata, and return the primary tag. If the primary tag is not found, it will return the first tag.
    /// These tags contain details about the audio, such as the title, artist, etc.
    /// # Errors
    /// - If `path` does not exist
    /// - If the reader contains invalid data
    /// - If the audio file does not contain a primary tag or a first tag
    fn parse_file(path: &str) -> Result<Tag, LoftyError> {
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

    fn extract_cover_image_from_tag(tag: &Tag) -> SharedImage {
        let cover = tag.pictures().first().unwrap();
        let cover_bytes = cover.data();

        // Return different SharedImage's depending on what filetype the cover is
        match cover.mime_type().unwrap() {
            MimeType::Png => {
                SharedImage::from_image(&PngImage::from_data(cover_bytes).unwrap()).unwrap()
            }
            MimeType::Jpeg => {
                SharedImage::from_image(&JpegImage::from_data(cover_bytes).unwrap()).unwrap()
            }
            _ => panic!("Unsupported mime type"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_FILES: &str = "./src/app/ui/tests/files";

    mod parse_file {
        use std::path::Path;

        use super::*;
        use lofty::tag::Accessor;

        /// A testing function to check that the metadata for an audio file is being parsed properly
        fn assert_metadata(filename: &str, expected_title: &str, expected_artist: &str) {
            let relative_path = format!("{}/audio/with-metadata/{}", TEST_FILES, filename);

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
            let relative_path = format!("{}/audio/without-metadata/{}", TEST_FILES, filename);

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
            let binding = format!("{}/does_not_exist.mp3", TEST_FILES);
            let path = binding.as_str();

            let invalid_file = NowPlaying::parse_file(path);
            assert!(invalid_file.is_err());
        }
    }

    mod extract_cover_image_from_tag {
        use std::{fs, path::Path};

        use fltk::prelude::ImageExt;
        use lofty::{
            picture::{MimeType, Picture, PictureType},
            tag::TagType,
        };

        use super::*;

        #[test]
        fn test_png() {
            let relative_path_cover = format!("{}/images/covers/{}", TEST_FILES, "test_cover.png");

            let full_path_cover = Path::new(&relative_path_cover)
                .canonicalize()
                .expect("Failed to resolve absolute path");

            let mut tag = Tag::new(TagType::Id3v2);

            // Add a front cover
            // Read the file bytes
            let data = fs::read(full_path_cover.clone()).expect("Failed to read image file");

            // Infer MIME type from extension
            let mime_type = MimeType::Png;

            let front_cover =
                Picture::new_unchecked(PictureType::CoverFront, Some(mime_type), None, data);

            tag.push_picture(front_cover);

            let expected_img = SharedImage::load(full_path_cover).unwrap();
            let img = NowPlaying::extract_cover_image_from_tag(&tag);

            assert_eq!(expected_img.width(), img.width());
            assert_eq!(expected_img.height(), img.height());
            assert_eq!(expected_img.to_rgb_data(), img.to_rgb_data());
        }

        #[test]
        fn test_jpg() {
            let relative_path_cover = format!("{}/images/covers/{}", TEST_FILES, "test_cover.jpg");

            let full_path_cover = Path::new(&relative_path_cover)
                .canonicalize()
                .expect("Failed to resolve absolute path");

            let mut tag = Tag::new(TagType::Id3v2);

            // Add a front cover
            // Read the file bytes
            let data = fs::read(full_path_cover.clone()).expect("Failed to read image file");

            // Infer MIME type from extension
            let mime_type = MimeType::Jpeg;

            let front_cover =
                Picture::new_unchecked(PictureType::CoverFront, Some(mime_type), None, data);

            tag.push_picture(front_cover);

            let expected_img = SharedImage::load(full_path_cover).unwrap();
            let img = NowPlaying::extract_cover_image_from_tag(&tag);

            assert_eq!(expected_img.width(), img.width());
            assert_eq!(expected_img.height(), img.height());
            assert_eq!(expected_img.to_rgb_data(), img.to_rgb_data());
        }
    }
}
