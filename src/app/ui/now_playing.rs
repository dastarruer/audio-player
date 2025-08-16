use fltk::draw::{self};
use fltk::enums::{Font, FrameType};
use fltk::frame::Frame;
use fltk::image::{JpegImage, PngImage, SharedImage};
use fltk::output::Output;
use fltk::prelude::{InputExt, WidgetBase, WidgetExt};
use lofty::error::{ErrorKind, LoftyError};
use lofty::file::TaggedFileExt;
use lofty::picture::{MimeType, PictureType};
use lofty::read_from_path;
use lofty::tag::{Accessor, Tag};
use std::borrow::Cow;
use std::path::Path;

pub struct NowPlaying {}

impl NowPlaying {
    pub fn new(path: &str) -> NowPlaying {
        let metadata_tag = NowPlaying::parse_file(path).unwrap();

        let cover_widget = NowPlaying::create_cover_widget(&metadata_tag);
        let title_widget = NowPlaying::create_title_widget(&metadata_tag, &cover_widget);
        NowPlaying::create_artist_widget(&metadata_tag, &cover_widget, &title_widget);

        NowPlaying {}
    }

    fn create_title_widget(metadata_tag: &Tag, cover_widget: &Frame) -> Output {
        const FONT: Font = Font::HelveticaBold;
        const FONTSIZE: i32 = 14;

        let title = NowPlaying::extract_title_from_tag(metadata_tag);

        // TODO: Remove getting text width twice
        draw::set_font(FONT, FONTSIZE);
        let (text_width, _) = draw::measure(&title, false);

        // Center X position
        let center_x = NowPlaying::text_center_x_of_widget(cover_widget, &title, FONT, FONTSIZE);
        let pos_y = NowPlaying::get_title_widget_y(cover_widget);

        let title_widget_width = text_width + 10;
        let title_widget_height = 20;

        let mut title_widget =
            Output::new(center_x, pos_y, title_widget_width, title_widget_height, "");

        title_widget.set_value(&title);
        title_widget.set_text_font(FONT);
        title_widget.set_frame(FrameType::NoBox);

        title_widget
    }

    fn create_artist_widget(metadata_tag: &Tag, cover_widget: &Frame, title_widget: &Output) {
        const FONT: Font = Font::Helvetica;
        const FONTSIZE: i32 = 14;

        let artist = NowPlaying::extract_artist_from_tag(metadata_tag);

        // TODO: Remove getting text width twice
        draw::set_font(FONT, FONTSIZE);
        let (text_width, _) = draw::measure(&artist, false);

        let artist_widget_x =
            NowPlaying::text_center_x_of_widget(cover_widget, &artist, FONT, FONTSIZE);
        let artist_widget_y = NowPlaying::get_artist_widget_y(title_widget);

        let artist_widget_width = text_width + 10;
        let artist_widget_height = 20;

        let mut artist_widget = Output::new(
            artist_widget_x,
            artist_widget_y,
            artist_widget_width,
            artist_widget_height,
            "",
        );

        artist_widget.set_value(&artist);
        artist_widget.set_text_font(FONT);
        artist_widget.set_frame(FrameType::NoBox);
    }

    fn get_artist_widget_y(title_widget: &Output) -> i32 {
        let title_y = title_widget.y();
        let title_h = title_widget.h();

        // Place just below the title
        const PADDING_Y: i32 = 0;
        title_y + title_h + PADDING_Y
    }

    /// Extract the title from a given metadata tag.
    ///
    /// This function determines what title to show in the Now Playing section
    /// of the audio player.
    ///
    /// # Returns
    /// A default title if any of the following conditions is met:
    /// - There is no title in the tag
    fn extract_title_from_tag(tag: &Tag) -> String {
        let default_title = "Untitled audio";
        tag.title()
            .unwrap_or(Cow::Borrowed(default_title))
            .to_string()
    }

    /// Extract the artist from a given metadata tag.
    ///
    /// This function determines what artist to show in the Now Playing section
    /// of the audio player.
    ///
    /// # Returns
    /// A default artist if any of the following conditions is met:
    /// - There is no artist in the tag
    fn extract_artist_from_tag(tag: &Tag) -> String {
        let default_artist = "No artist";
        tag.artist()
            .unwrap_or(Cow::Borrowed(default_artist))
            .to_string()
    }

    /// Return the x position of a text box that would be needed to center it over another widget.
    fn text_center_x_of_widget(widget: &Frame, title: &str, font: Font, fontsize: i32) -> i32 {
        draw::set_font(font, fontsize);
        let (text_width, _) = draw::measure(title, false);

        // Get cover widget position and width
        let cover_x = widget.x();
        let cover_w = widget.w();

        // Return centered x position
        cover_x + (cover_w - text_width) / 2
    }

    fn get_title_widget_y(cover_widget: &Frame) -> i32 {
        let cover_y = cover_widget.y();
        let cover_h = cover_widget.h();

        // Place just below the cover
        const PADDING_Y: i32 = 0;
        cover_y + cover_h + PADDING_Y
    }

    fn create_cover_widget(metadata_tag: &Tag) -> Frame {
        let mut cover_widget = Frame::new(150, 40, 100, 100, "");

        // Extract the image from the metadata tag
        let cover_image = NowPlaying::extract_cover_image_from_tag(metadata_tag);

        // Assign the image to the frame
        // Use set_image_scaled so that the image scales to the widget's size
        cover_widget.set_image_scaled(Some(cover_image));

        cover_widget
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

    /// Extract the cover image from a given metadata tag.
    ///
    /// This function determines what image to show in the Now Playing section
    /// of the audio player.
    ///
    /// # Returns
    /// A default cover if any of the following conditions is met:
    /// - There are no images in the tag
    /// - The mime type does not exist
    /// - The mime type is not `MimeType::Png` or `MimeType::Jpeg`
    fn extract_cover_image_from_tag(tag: &Tag) -> SharedImage {
        // The path to the default cover, which will be displayed in case anything goes wrong while fetching the cover image
        let default_cover_path = Path::new("assets/default.png");

        // If there are no pictures, return the default cover
        if tag.picture_count() == 0 {
            return SharedImage::load(default_cover_path).unwrap();
        }

        let cover = tag
            .pictures()
            .iter()
            .find(|picture| picture.pic_type() == PictureType::CoverFront);

        // Unwrap if there is a front cover, otherwise just return the default cover
        let cover = if let Some(cover) = cover {
            cover
        } else {
            return SharedImage::load(default_cover_path).unwrap();
        };

        let cover_bytes = cover.data();

        // Return different SharedImage's depending on what filetype the cover is
        match cover
            .mime_type()
            .unwrap_or(&MimeType::Unknown("No mime type".to_string()))
        {
            MimeType::Png => {
                // Should not panic, because the mime type is determined from the start
                SharedImage::from_image(&PngImage::from_data(cover_bytes).unwrap()).unwrap()
            }
            MimeType::Jpeg => {
                SharedImage::from_image(&JpegImage::from_data(cover_bytes).unwrap()).unwrap()
            }
            _ => SharedImage::load(default_cover_path).unwrap(),
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
        use std::{
            fs,
            path::{Path, PathBuf},
        };

        use fltk::prelude::ImageExt;
        use lofty::{
            picture::{MimeType, Picture, PictureType},
            tag::TagType,
        };

        use super::*;

        fn create_picture(path: PathBuf, mime_type: MimeType, pic_type: PictureType) -> Picture {
            // Read the file bytes
            let data = fs::read(path).expect("Failed to read image file");

            Picture::new_unchecked(pic_type, Some(mime_type), None, data)
        }

        fn get_test_cover_path(filename: &str) -> PathBuf {
            let relative_cover_path = format!("{}/images/covers/{}", TEST_FILES, filename);

            // Return the full file path
            Path::new(&relative_cover_path)
                .canonicalize()
                .expect("Failed to resolve absolute path")
        }

        fn assert_test_cover_correct(filename: &str, mime_type: MimeType, pic_type: PictureType) {
            // The tag type shouldn't matter
            let mut tag = Tag::new(TagType::Id3v2);

            let full_cover_path = get_test_cover_path(filename);

            let front_cover = create_picture(full_cover_path.clone(), mime_type, pic_type);
            tag.push_picture(front_cover);

            let expected_cover = SharedImage::load(full_cover_path).unwrap();
            let cover = NowPlaying::extract_cover_image_from_tag(&tag);

            assert_eq!(expected_cover.width(), cover.width());
            assert_eq!(expected_cover.height(), cover.height());
            assert_eq!(expected_cover.to_rgb_data(), cover.to_rgb_data());
        }

        fn assert_default_cover_is_returned<F>(test: F)
        where
            F: Fn() -> SharedImage,
        {
            let img = test();
            let expected_img = SharedImage::load("./assets/default.png").unwrap();

            assert_eq!(expected_img.width(), img.width());
            assert_eq!(expected_img.height(), img.height());
            assert_eq!(expected_img.to_rgb_data(), img.to_rgb_data());
        }

        #[test]
        fn test_png() {
            assert_test_cover_correct("test_cover.png", MimeType::Png, PictureType::CoverFront);
        }

        #[test]
        fn test_jpg() {
            assert_test_cover_correct("test_cover.jpg", MimeType::Jpeg, PictureType::CoverFront);
        }

        #[test]
        fn test_non_existent_cover() {
            assert_default_cover_is_returned(|| {
                let tag = Tag::new(TagType::Id3v2);
                NowPlaying::extract_cover_image_from_tag(&tag)
            });
        }

        #[test]
        fn test_unsupported_gif_format() {
            assert_default_cover_is_returned(|| {
                let relative_path_cover =
                    format!("{}/images/covers/{}", TEST_FILES, "test_cover.gif");

                let full_cover_path = Path::new(&relative_path_cover)
                    .canonicalize()
                    .expect("Failed to resolve absolute path");

                let mut tag = Tag::new(TagType::Id3v2);

                // Read the file bytes
                let data = fs::read(full_cover_path.clone()).expect("Failed to read image file");

                // Add a front cover
                let front_cover = Picture::new_unchecked(
                    PictureType::CoverFront,
                    Some(MimeType::Gif),
                    None,
                    data,
                );

                tag.push_picture(front_cover);

                NowPlaying::extract_cover_image_from_tag(&tag)
            });
        }

        #[test]
        fn test_no_cover_image() {
            assert_default_cover_is_returned(|| {
                let full_cover_path = get_test_cover_path("test_cover.jpg");

                let mut tag = Tag::new(TagType::Id3v2);

                // Read the file bytes
                let data = fs::read(full_cover_path.clone()).expect("Failed to read image file");

                // Add an artist picture. This should not get returned
                let front_cover =
                    Picture::new_unchecked(PictureType::Artist, Some(MimeType::Jpeg), None, data);

                tag.push_picture(front_cover);

                NowPlaying::extract_cover_image_from_tag(&tag)
            });
        }

        #[test]
        fn test_multiple_cover_images() {
            let full_expected_cover_path = get_test_cover_path("test_cover.jpg");

            // The tag type shouldn't matter
            let mut tag = Tag::new(TagType::Id3v2);

            // Create first front cover. This is the one that should be returned
            let expected_front_cover = create_picture(
                full_expected_cover_path.clone(),
                MimeType::Jpeg,
                PictureType::CoverFront,
            );
            tag.push_picture(expected_front_cover);

            let full_cover_path = get_test_cover_path("test_cover.png");

            // Create second front cover. This should not be returned
            let front_cover = create_picture(
                full_cover_path.clone(),
                MimeType::Jpeg,
                PictureType::CoverFront,
            );
            tag.push_picture(front_cover);

            let expected_img = SharedImage::load(full_expected_cover_path).unwrap();
            let img = NowPlaying::extract_cover_image_from_tag(&tag);

            assert_eq!(expected_img.width(), img.width());
            assert_eq!(expected_img.height(), img.height());
            assert_eq!(expected_img.to_rgb_data(), img.to_rgb_data());
        }

        #[test]
        fn test_multiple_cover_image_types() {
            let full_artist_picture_path = get_test_cover_path("test_cover.jpg");

            // The tag type shouldn't matter
            let mut tag = Tag::new(TagType::Id3v2);

            // Create artist picture. This should not be returned
            let artist_picture = create_picture(
                full_artist_picture_path,
                MimeType::Jpeg,
                PictureType::Artist,
            );
            tag.push_picture(artist_picture);

            let full_cover_path = get_test_cover_path("test_cover.png");

            // Create front cover. This should be returned
            let front_cover = create_picture(
                full_cover_path.clone(),
                MimeType::Png,
                PictureType::CoverFront,
            );
            tag.push_picture(front_cover);

            let expected_img = SharedImage::load(full_cover_path).unwrap();
            let img = NowPlaying::extract_cover_image_from_tag(&tag);

            assert_eq!(expected_img.width(), img.width());
            assert_eq!(expected_img.height(), img.height());
            assert_eq!(expected_img.to_rgb_data(), img.to_rgb_data());
        }
    }

    // TODO: I don't know how to test this without getting constant errors from fltk
    // mod get_title_widget_x {
    //     use fltk::{
    //         enums::Font,
    //         frame::Frame,
    //         prelude::{WidgetBase, WidgetExt},
    //     };

    //     use crate::app::ui::now_playing::NowPlaying;

    //     fn test_title_centering(title: &str) {
    //         let cover_widget = Frame::new(150, 50, 100, 100, "");

    //         let text_width = {
    //             fltk::draw::set_font(Font::Helvetica, 14);
    //             let (w, _) = fltk::draw::measure(title, false);
    //             w
    //         };

    //         let cover_x = cover_widget.x();
    //         let cover_w = cover_widget.w();

    //         let title_x = NowPlaying::get_title_widget_x(&cover_widget, title);

    //         // Compute left & right margins
    //         let left_margin = title_x - cover_x;
    //         let right_margin = (cover_x + cover_w) - (title_x + text_width);

    //         // They should be almost equal
    //         assert!((left_margin - right_margin).abs() <= 1);
    //     }

    //     #[test]
    //     fn test_short_title() {
    //         test_title_centering("hello");
    //     }

    //     #[test]
    //     fn test_medium_title() {
    //         test_title_centering("hello world");
    //     }

    //     #[test]
    //     fn test_long_title() {
    //         test_title_centering("hello world today is the day");
    //     }
    // }

    mod extract_title_from_tag {
        use lofty::tag::{ItemKey, Tag};

        use crate::app::ui::now_playing::NowPlaying;

        fn test_default_title_is_returned<F>(test: F)
        where
            F: Fn() -> Tag,
        {
            let title = NowPlaying::extract_title_from_tag(&test());
            let expected_title = "Untitled audio";

            assert_eq!(title, expected_title);
        }

        #[test]
        fn extract_title() {
            let title = "less than lovers";

            let mut tag = Tag::new(lofty::tag::TagType::Id3v2);
            tag.insert_text(ItemKey::TrackTitle, title.to_string());

            assert_eq!(NowPlaying::extract_title_from_tag(&tag), title);
        }

        #[test]
        fn extract_title_from_tag_with_no_title() {
            test_default_title_is_returned(|| Tag::new(lofty::tag::TagType::Id3v2));
        }
    }

    mod extract_artist_from_tag {
        use lofty::tag::{ItemKey, Tag};

        use crate::app::ui::now_playing::NowPlaying;

        fn test_default_artist_is_returned<F>(test: F)
        where
            F: Fn() -> Tag,
        {
            let artist = NowPlaying::extract_artist_from_tag(&test());
            let expected_artist = "No artist";

            assert_eq!(artist, expected_artist);
        }

        #[test]
        fn extract_artist() {
            let artist = "Kensuke Ushio";

            let mut tag = Tag::new(lofty::tag::TagType::Id3v2);
            tag.insert_text(ItemKey::TrackArtist, artist.to_string());

            assert_eq!(NowPlaying::extract_artist_from_tag(&tag), artist);
        }

        #[test]
        fn extract_artist_from_tag_with_no_artist() {
            test_default_artist_is_returned(|| Tag::new(lofty::tag::TagType::Id3v2));
        }
    }
}
