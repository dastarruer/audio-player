struct NowPlaying {}

impl NowPlaying {
    pub fn new() -> NowPlaying {
        NowPlaying {  }
    }

    pub fn parse_file(_filename: &str) {
        println!("Hello");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod parse_file {
        use super::*;

        #[test]
        fn parse_file() {
            NowPlaying::parse_file("filename");
            assert_eq!(1, 1)
        }
    }
}
