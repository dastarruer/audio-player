use fltk::{app, button, prelude::*, window};

fn main() {
    let a = app::App::default();

    // Create a new window
    let win_width = 400;
    let win_height = 300;
    let mut win = window::Window::default()
        .with_size(win_width, win_height)
        .with_label("My window");

    // Create a button
    let mut btn = button::Button::default()
        .with_size(80, 30)
        .with_pos(155, 200)
        .with_label("Pause");

    // Show the window
    win.end();
    win.show();

    // Add an action to the button
    btn.set_callback(move |btn| match btn.label().as_str() {
        "Play" => btn.set_label("Pause"),
        "Pause" => btn.set_label("Play"),
        _ => unreachable!(),
    });

    a.run().unwrap();
}
