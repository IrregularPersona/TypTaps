mod app;
mod message;
mod pdfviewer;
mod ui;
mod utils;

use app::TypTaps;

pub fn main() -> iced::Result {
    iced::application(TypTaps::default, TypTaps::update, TypTaps::view)
        .title("TypTaps - Minimal Typst Editor")
        .theme(TypTaps::theme)
        .subscription(TypTaps::subscription)
        .run()
}
