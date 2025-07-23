use fltk::{app, button, enums::Color, prelude::*, window};
use fltk_theme::{ThemeType, WidgetTheme};

fn main() {
    let a = app::App::default().with_scheme(app::Scheme::Gtk);

    // Create a new window
    const WIN_WIDTH: i32 = 400;
    const WIN_HEIGHT: i32 = 300;
    let mut win = window::Window::default()
        .with_size(WIN_WIDTH, WIN_HEIGHT)
        .with_label("My window");
    win.set_color(Color::White);

    // Create a button
    const BTN_SIZE: i32 = 30;
    const BTN_X: i32 = (WIN_WIDTH - BTN_SIZE) / 2; // Center the button horizontally
    const BTN_Y: i32 = 200;
    let mut btn = button::Button::default()
        .with_size(BTN_SIZE, BTN_SIZE)
        .with_pos(BTN_X, BTN_Y)
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
