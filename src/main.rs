use fltk::{app, button, enums::Color, prelude::*, window};
use fltk_theme::{ThemeType, WidgetTheme};

fn main() {
    let a = app::App::default().with_scheme(app::Scheme::Gtk);

    // Create a new window
    let win_width = 400;
    let win_height = 300;
    let mut win = window::Window::default()
        .with_size(win_width, win_height)
        .with_label("My window");
    win.set_color(Color::White);

    // Create a button
    let mut btn = button::Button::default()
        .with_size(30, 30)
        .with_pos(155, 200)
        .with_label("");

    theme_button(&mut btn);

    // Show the window
    win.end();
    win.show();

    // Add an action to the button
    btn.set_callback(move |btn| match btn.label().as_str() {
        "" => btn.set_label(""),
        "" => btn.set_label(""),
        _ => unreachable!(),
    });

    a.run().unwrap();
}

fn theme_button(btn: &mut button::Button) {
    // Remove focus border around button
    btn.clear_visible_focus();

    // Remove button background
    btn.set_frame(fltk::enums::FrameType::NoBox);
}
