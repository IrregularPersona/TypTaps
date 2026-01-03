use iced::widget::{Space, button, column, container, row, scrollable, svg, text};
use iced::{Alignment, Element, Subscription, Task, time};
use iced_code_editor::{CodeEditor, Message as EditorMessage};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, Instant, SystemTime};

pub struct TypTaps {
    pub editor: CodeEditor,
    pub file: Option<PathBuf>,
    pub pages: Vec<svg::Handle>,
    pub is_rendering: bool,
    pub last_rendered_time: Option<SystemTime>,
    pub watch_process: Option<Child>,
    pub is_dirty: bool,
    pub last_save_time: Instant,
    // pub preview_scroll_id: scrollable::Id::unique(),
}

#[derive(Debug, Clone)]
pub enum Message {
    EditorEvent(EditorMessage),
    OpenFile,
    FileOpened(Result<PathBuf, String>),
    Tick,
}

impl Default for TypTaps {
    fn default() -> Self {
        Self {
            editor: CodeEditor::new("", "rust"),
            file: None,
            pages: Vec::new(),
            is_rendering: false,
            last_rendered_time: None,
            watch_process: None,
            is_dirty: false,
            last_save_time: Instant::now(),
        }
    }
}

impl TypTaps {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EditorEvent(event) => {
                let task = self.editor.update(&event).map(Message::EditorEvent);
                self.is_dirty = true;
                task
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
                    self.editor = CodeEditor::new(&content.as_str(), "typ");
                }

                self.pages.clear();
                self.is_rendering = false;
                self.last_rendered_time = None;

                if let Some(mut child) = self.watch_process.take() {
                    let _ = child.kill();
                }

                // Get cache directory for output
                let cache_dir = dirs::cache_dir()
                    .map(|p| p.join("typtaps"))
                    .unwrap_or_else(|| PathBuf::from(".cache/typtaps"));
                let stem = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "output".to_string());
                let output_template = cache_dir.join(format!("{}-{{p}}.svg", stem));

                println!("Watching {:?} -> {:?}", path, output_template);

                match Command::new("typst")
                    .arg("watch")
                    .arg(&path)
                    .arg(&output_template)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                {
                    Ok(child) => {
                        self.watch_process = Some(child);
                        self.is_rendering = true;
                        self.last_rendered_time = None;
                        Task::none()
                    }
                    Err(e) => {
                        eprintln!("Error starting typst: {}", e);
                        Task::none()
                    }
                }
            }
            Message::FileOpened(Err(e)) => {
                eprintln!("Error opening file: {}", e);
                Task::none()
            }
            Message::Tick => {
                if let Some(path) = &self.file {
                    let cache_dir = dirs::cache_dir()
                        .map(|p| p.join("typtaps"))
                        .unwrap_or_else(|| PathBuf::from(".cache/typtaps"));
                    let stem = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "output".to_string());

                    let first_page = cache_dir.join(format!("{}-1.svg", stem));

                    if first_page.exists() {
                        let metadata = std::fs::metadata(&first_page).ok();
                        let mtime = metadata.and_then(|m| m.modified().ok());

                        let needs_reload = match (mtime, self.last_rendered_time) {
                            (Some(mt), Some(rt)) => mt > rt,
                            (Some(_), None) => true,
                            _ => false,
                        };

                        if needs_reload {
                            let mut new_pages = Vec::new();
                            let mut i = 1;
                            loop {
                                let page_path = cache_dir.join(format!("{}-{}.svg", stem, i));
                                if page_path.exists() {
                                    if let Ok(content) = std::fs::read(&page_path) {
                                        new_pages.push(svg::Handle::from_memory(content));
                                    }
                                    i += 1;
                                } else {
                                    break;
                                }
                            }
                            if !new_pages.is_empty() {
                                self.pages = new_pages;
                                self.last_rendered_time = mtime;
                            }
                        }
                    } else {
                        let output_path = cache_dir.join(format!("{}.svg", stem));
                        if output_path.exists() {
                            let metadata = std::fs::metadata(&output_path).ok();
                            let mtime = metadata.and_then(|m| m.modified().ok());
                            if self
                                .last_rendered_time
                                .map_or(true, |rt| mtime.map_or(false, |mt| mt > rt))
                            {
                                if let Ok(content) = std::fs::read(&output_path) {
                                    self.pages = vec![svg::Handle::from_memory(content)];
                                    self.last_rendered_time = mtime;
                                }
                            }
                        }
                    }
                }

                // Autosave logic
                if self.is_dirty && self.last_save_time.elapsed() >= Duration::from_millis(90) {
                    if let Some(path) = &self.file {
                        let content = self.editor.content();
                        if let Err(e) = std::fs::write(path, content) {
                            eprintln!("Autosave error: {}", e);
                        } else {
                            self.is_dirty = false;
                            self.last_save_time = Instant::now();
                        }
                    }
                }

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let top_bar = container(row![
            Space::new().width(10),
            button(text("Open File").size(10))
                .on_press(Message::OpenFile)
                .padding(2),
        ])
        .width(iced::Length::Fill)
        .padding(iced::Padding {
            top: 2.0,
            right: 2.0,
            bottom: 0.0,
            left: 2.0,
        });

        let preview_content: Element<Message> = if !self.pages.is_empty() {
            scrollable(
                column(self.pages.iter().map(|handle| {
                    container(
                        svg(handle.clone())
                            .width(iced::Length::Fixed(1400.0))
                            .height(iced::Length::Fixed(1600.0)),
                    )
                    .padding(10)
                    .into()
                }))
                .spacing(10)
                .align_x(Alignment::Center)
                .width(iced::Length::Fill),
            )
            .height(iced::Length::Fill)
            .auto_scroll(true)
            .into()
        } else if self.file.is_some() {
            container(text("Waiting to finish render...").size(18))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        } else {
            container(text("No file loaded").size(18))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        };

        column![
            top_bar,
            container(self.editor.view().map(Message::EditorEvent))
                .padding(iced::Padding {
                    top: 0.0,
                    right: 20.0,
                    bottom: 20.0,
                    left: 20.0,
                })
                .width(iced::Length::Fill)
                .height(iced::Length::Fill),
            container(preview_content)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
        ]
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(60)).map(|_| Message::Tick)
    }
}
