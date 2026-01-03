use crate::pdfviewer::render_pdf_to_pages;
use crate::{message::Message, ui};
use iced::widget::{image, text_editor};
use iced::{Task, Theme, time};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, SystemTime};

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
    pub pages: Vec<image::Handle>,
    pub pdf_child: Option<Child>,
    pub is_rendering: bool,
    pub last_rendered_time: Option<SystemTime>,
    pub zoom: f32,
}

impl Default for TypTaps {
    fn default() -> Self {
        Self {
            content: text_editor::Content::with_text(""),
            file: None,
            cursor_line: 1,
            cursor_column: 1,
            file_tree: Vec::new(),
            pages: Vec::new(),
            pdf_child: None,
            is_rendering: false,
            last_rendered_time: None,
            zoom: 1.0,
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

                if let Some(path) = &self.file {
                    let content = self.content.text();
                    let _ = std::fs::write(path, content);
                }

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

                if let Some(mut child) = self.pdf_child.take() {
                    let _ = child.kill();
                }

                self.pages.clear();
                self.is_rendering = false;
                self.last_rendered_time = None;

                match Command::new("typst").arg("watch").arg(&path).spawn() {
                    Ok(child) => {
                        self.pdf_child = Some(child);
                    }
                    Err(e) => {
                        eprintln!("Failed to start typst watch: {:?}", e);
                        self.is_rendering = false;
                    }
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
            Message::ReloadRequested(_) => {
                if self.is_rendering {
                    return Task::none();
                }

                if let Some(path) = &self.file {
                    let pdf_path = path.with_extension("pdf");
                    if pdf_path.exists() {
                        let metadata = std::fs::metadata(&pdf_path).ok();
                        let mtime = metadata.and_then(|m| m.modified().ok());

                        let needs_reload = match (mtime, self.last_rendered_time) {
                            (Some(mt), Some(rt)) => mt > rt,
                            (Some(_), None) => true,
                            _ => false,
                        };

                        if needs_reload {
                            self.is_rendering = true;
                            let path_to_render = pdf_path.clone();
                            return Task::perform(
                                async move {
                                    let res: Result<Vec<image::Handle>, String> =
                                        tokio::task::spawn_blocking(move || {
                                            render_pdf_to_pages(&path_to_render)
                                                .map_err(|e| e.to_string())
                                        })
                                        .await
                                        .map_err(|e| e.to_string())?;
                                    res
                                },
                                Message::PdfRendered,
                            );
                        }
                    }
                }
                Task::none()
            }
            Message::PdfRendered(result) => {
                self.is_rendering = false;
                self.last_rendered_time = Some(SystemTime::now());
                if let Ok(pages) = result {
                    self.pages = pages;
                }
                Task::none()
            }
            Message::ZoomIn => {
                self.zoom = (self.zoom + 0.2).min(10.0);
                Task::none()
            }
            Message::ZoomOut => {
                self.zoom = (self.zoom - 0.2).max(0.1);
                Task::none()
            }
            Message::ResetZoom => {
                self.zoom = 1.0;
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch(vec![
            iced::event::listen()
                .map(|event| {
                    if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key,
                        modifiers,
                        ..
                    }) = event
                    {
                        if let iced::keyboard::Key::Character(s) = key {
                            if modifiers.command() {
                                match s.as_str() {
                                    "+" | "=" => return Some(Message::ZoomIn),
                                    "-" | "_" => return Some(Message::ZoomOut),
                                    ")" | "0" => return Some(Message::ResetZoom),
                                    _ => {}
                                }
                            }
                        }
                    }
                    None
                })
                .filter_map(|msg: Option<Message>| msg),
            time::every(Duration::from_millis(100)).map(Message::ReloadRequested),
        ])
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

    // maybe we need this, maybe not, idk
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
