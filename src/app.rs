use iced::widget::text_editor;
use iced::{Task, Theme};
use std::path::PathBuf;
use crate::{message::Message, ui};

pub struct TypTaps {
    pub content: text_editor::Content,
    pub file: Option<PathBuf>,
    pub cursor_line: usize,
    pub cursor_column: usize,
}

impl Default for TypTaps {
    fn default() -> Self {
        Self {
            content: text_editor::Content::with_text(""),
            file: None,
            cursor_line: 1,
            cursor_column: 1,
        }
    }
}

impl TypTaps {
    pub fn theme(&self) -> Theme {
        Theme::GruvboxDark
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        ui::view(self)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                self.content.perform(action);
                
                self.cursor_line = self.content.text().lines().count().max(1);
                self.cursor_column = self
                    .content
                    .text()
                    .lines()
                    .last()
                    .map(|line| line.len() + 1)
                    .unwrap_or(1);
                
                Task::none()
            }
            Message::OpenFile => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Open Typst File")
                        .add_filter("Typst", &["typ"])
                        .pick_file()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                        .ok_or("Dialog Complete".to_string())
                },
                |res| Message::FileOpened(res.map_err(|_| "Error".to_string())),
            ),
            Message::FileOpened(Ok(path)) => {
                self.file = Some(path.clone());

                if let Ok(content) = std::fs::read_to_string(&path) {
                    self.content = text_editor::Content::with_text(&content);
                }
                Task::none()
            }
            Message::FileOpened(Err(_)) => Task::none(),
            Message::OpenDir => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Open Directory")
                        .pick_folder()
                        .await
                        .map(|handle| handle.path().to_path_buf())
                        .ok_or("Dialog Cancelled".to_string())
                },
                |res| Message::DirOpened(res),
            ),
            Message::DirOpened(Ok(_path)) => {
                // Directory support could be added here (e.g. populating a file tree)
                Task::none()
            }
            Message::DirOpened(Err(_)) => Task::none(),
            Message::SaveFile => {
                if let Some(path) = &self.file {
                    let content = self.content.text();
                    // For now, we'll do a simple synchronous write for simplicity.
                    // In a larger app, you'd want this to be a Task.
                    let _ = std::fs::write(path, content);
                }
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen().map(|event| {
            if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
                if let iced::keyboard::Key::Character(s) = key {
                    if s == "s" && modifiers.command() {
                        return Some(Message::SaveFile);
                    }
                }
            }
            None
        })
        .filter_map(|msg| msg)
    }
}
