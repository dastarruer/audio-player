use fltk::{app, prelude::*, window};

fn main() {
    let a = app::App::default();
    let mut win = window::Window::new(100, 100, 400, 300, "My window");

    win.end();
    win.show();
    a.run().unwrap();
}
