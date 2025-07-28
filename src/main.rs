mod app;
use app::AudioApp;

fn main() {
    let mut app = AudioApp::new();
    app.run();
}
