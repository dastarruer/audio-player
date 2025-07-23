use fltk::{app, button, enums::Color, prelude::*, window};

/// Stores the components of the GUI
pub struct App {
    app: app::App,
    window: window::DoubleWindow,
    play_button: button::Button,
}

impl App {
    /// Create the new App
    pub fn new() -> App {
        let app = app::App::default().with_scheme(app::Scheme::Gtk);

        // Create a new window
        const WIN_WIDTH: i32 = 400;
        const WIN_HEIGHT: i32 = 300;
        let window = App::create_window(WIN_WIDTH, WIN_HEIGHT);

        const BTN_SIZE: i32 = 30;
        const BTN_X: i32 = (WIN_WIDTH - BTN_SIZE) / 2; // Center the button horizontally
        const BTN_Y: i32 = 200;
        let play_button = App::create_play_button(BTN_SIZE, BTN_X, BTN_Y);

        App {
            app,
            window,
            play_button,
        }
    }

    /// Show the GUI to the user
    pub fn show_window(&mut self) {
        self.window.end();
        self.window.show();

        self.app.run().unwrap();
    }

    /// Create the window and theme it
    fn create_window(width: i32, height: i32) -> window::DoubleWindow {
        let mut win = window::Window::default()
            .with_size(width, height)
            .with_label("My window");
        win.set_color(Color::White);
        win
    }

    /// Create the play button and theme it
    fn create_play_button(size: i32, x: i32, y: i32) -> button::Button {
        let mut btn = button::Button::default()
            .with_size(size, size)
            .with_pos(x, y)
            .with_label("");

        // Remove focus border around button
        btn.clear_visible_focus();

        // Remove button background
        btn.set_frame(fltk::enums::FrameType::NoBox);

        // Switch between play/pause icons on button click
        btn.set_callback(move |btn| match btn.label().as_str() {
            "" => btn.set_label(""),
            "" => btn.set_label(""),
            _ => unreachable!(),
        });

        btn
    }
}
