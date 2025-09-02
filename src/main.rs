mod app;
use app::AudioApp;
use fltk_flex::Flex;
fn main() {
    // TODO: Remove this
    // This colors the flexbox so it is easier to know what is going on
    Flex::debug(true);

    let mut app = AudioApp::new();
    app.run();
}
