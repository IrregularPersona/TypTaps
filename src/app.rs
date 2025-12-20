use crate::{message::Message, ui};
use iced::widget::text_editor;
use iced::{Task, Theme};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum TreeEntry {
    File {
        path: PathBuf,
        name: String,
    },
    Directory {
        path: PathBuf,
        name: String,
        is_expanded: bool,
        children: Vec<TreeEntry>,
    },
}

impl TreeEntry {
    pub fn name(&self) -> &str {
        match self {
            TreeEntry::File { name, .. } => name,
            TreeEntry::Directory { name, .. } => name,
        }
    }

    pub fn path(&self) -> &PathBuf {
        match self {
            TreeEntry::File { path, .. } => path,
            TreeEntry::Directory { path, .. } => path,
        }
    }
}

pub struct TypTaps {
    pub content: text_editor::Content,
    pub file: Option<PathBuf>,
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub file_tree: Vec<TreeEntry>,
}

impl Default for TypTaps {
    fn default() -> Self {
        Self {
            content: text_editor::Content::with_text(""),
            file: None,
            cursor_line: 1,
            cursor_column: 1,
            file_tree: Vec::new(),
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
            Message::DirOpened(Ok(path)) => Task::perform(load_directory(path), |(p, entries)| {
                Message::DirLoaded(p, entries)
            }),
            Message::DirOpened(Err(_)) => Task::none(),
            Message::SaveFile => {
                if let Some(path) = &self.file {
                    let content = self.content.text();
                    let _ = std::fs::write(path, content);
                }
                Task::none()
            }
            Message::ToggleDir(target_path) => {
                let mut found_and_empty = false;
                toggle_dir_recursive(&mut self.file_tree, &target_path, &mut found_and_empty);

                if found_and_empty {
                    Task::perform(load_directory(target_path), |(p, entries)| {
                        Message::DirLoaded(p, entries)
                    })
                } else {
                    Task::none()
                }
            }
            Message::DirLoaded(path, entries) => {
                update_dir_children_recursive(&mut self.file_tree, &path, entries);
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen()
            .map(|event| {
                if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    ..
                }) = event
                {
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

async fn load_directory(path: PathBuf) -> (PathBuf, Vec<TreeEntry>) {
    let mut entries = Vec::new();

    if let Ok(read_dir) = std::fs::read_dir(&path) {
        for entry in read_dir.flatten() {
            let entry_path = entry.path();
            let name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();

            if entry_path.is_dir() {
                entries.push(TreeEntry::Directory {
                    path: entry_path,
                    name,
                    is_expanded: false,
                    children: Vec::new(),
                });
            } else {
                entries.push(TreeEntry::File {
                    path: entry_path,
                    name,
                });
            }
        }
    }

    // Sort: Directories first, then files, both alphabetically
    entries.sort_by(|a, b| match (a, b) {
        (TreeEntry::Directory { .. }, TreeEntry::File { .. }) => std::cmp::Ordering::Less,
        (TreeEntry::File { .. }, TreeEntry::Directory { .. }) => std::cmp::Ordering::Greater,
        _ => a.name().to_lowercase().cmp(&b.name().to_lowercase()),
    });

    (path, entries)
}

fn toggle_dir_recursive(entries: &mut [TreeEntry], target: &PathBuf, found_and_empty: &mut bool) {
    for entry in entries {
        if let TreeEntry::Directory {
            path,
            is_expanded,
            children,
            ..
        } = entry
        {
            if path == target {
                *is_expanded = !*is_expanded;
                if *is_expanded && children.is_empty() {
                    *found_and_empty = true;
                }
                return;
            }
            toggle_dir_recursive(children, target, found_and_empty);
        }
    }
}

fn update_dir_children_recursive(
    entries: &mut Vec<TreeEntry>,
    target: &PathBuf,
    new_children: Vec<TreeEntry>,
) {
    if entries.is_empty() {
        *entries = new_children;
        return;
    }

    for entry in entries.iter_mut() {
        if let TreeEntry::Directory { path, children, .. } = entry {
            if path == target {
                *children = new_children;
                return;
            }
            update_dir_children_recursive(children, target, new_children.clone());
        }
    }
}
