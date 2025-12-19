use iced::widget::text_editor;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    Edit(text_editor::Action),
    OpenFile,
    FileOpened(Result<PathBuf, String>),
    OpenDir,
    DirOpened(Result<PathBuf, String>),
    SaveFile,
}
