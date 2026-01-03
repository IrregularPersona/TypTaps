mod app;
<<<<<<< HEAD
<<<<<<< HEAD
mod message;
mod pdfviewer;
mod ui;
mod utils;

use app::TypTaps;

pub fn main() -> iced::Result {
    iced::application(TypTaps::default, TypTaps::update, TypTaps::view)
        .title("TypTaps - Minimal Typst Editor")
        .theme(TypTaps::theme)
=======
use app::TypTaps;
use std::fs;

fn main() -> iced::Result {
    if let Some(cache_dir) = dirs::cache_dir() {
        let typtaps_cache = cache_dir.join("typtaps");
        if !typtaps_cache.exists() {
            if let Err(e) = fs::create_dir_all(&typtaps_cache) {
                eprintln!("Error creating cache directory: {}", e);
            }
        }
    }

    iced::application(TypTaps::default, TypTaps::update, TypTaps::view)
>>>>>>> refactor/minimal-form
        .subscription(TypTaps::subscription)
        .run()
}
