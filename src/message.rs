use iced::widget::text_editor;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    // text box
    Edit(text_editor::Action),

    // file dialog
    OpenFile,
    FileOpened(Result<PathBuf, String>),
    OpenDir,
    DirOpened(Result<PathBuf, String>),

    // we need to change this later
    SaveFile,
    ReloadRequested(std::time::Instant),

    // file tree
    ToggleDir(PathBuf),
    DirLoaded(PathBuf, Vec<crate::app::TreeEntry>),
}
