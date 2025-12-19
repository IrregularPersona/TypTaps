use iced::border::top_left;
use iced::futures::task::SpawnError;
use iced::widget::{Space, button, column, container, row, text, text_editor};
use iced::{Element, Length, Task, Theme};
use std::path::PathBuf;

pub fn main() -> iced::Result {
    iced::application(TypTaps::default, TypTaps::update, TypTaps::view)
        .title("minimal text editor")
        .theme(TypTaps::theme)
        .run()
}

struct TypTaps {
    content: text_editor::Content,
    file: Option<PathBuf>,
    cursor_line: usize,
    cursor_column: usize,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    OpenFile,
    FileOpened(Result<PathBuf, String>),
    // OpenDir,
    // DirOpened(Result<PathBuf, String>),
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
    fn theme(&self) -> Theme {
        Theme::GruvboxDark
    }

    fn update(&mut self, message: Message) -> Task<Message> {
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
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let top_bar = container(
            row![
                button(text("Open File").size(10))
                    .on_press(Message::OpenFile)
                    .width(Length::Fixed(75.0)),
                // button("Open Folder").on_press(Message::OpenDir).width(Length::Fill),
            ], // .spacing(10),
        )
        .width(Length::Fixed(75.0));

        let file_tree_box = container(
            text("File Tree\n(Coming Soon)")
                .size(14)
                .color([0.7, 0.7, 0.7]),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10);

        let file_tree_panel = container(file_tree_box)
            .width(Length::Fixed(120.0))
            .height(Length::FillPortion(8))
            // .padding(10)
            .style(Self::panel_style);

        let editor_box = text_editor(&self.content)
            .on_action(Message::Edit)
            .height(Length::Fill);

        let code_panel = container(editor_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(15);
        // .style(Self::panel_style);

        let status_bar = container(
            text(format!("{}|{}", self.cursor_line, self.cursor_column))
                .size(12)
                .color([0.7, 0.7, 0.7]),
        )
        .width(Length::Fill)
        .padding(5)
        .style(Self::panel_style);

        column![
            top_bar,
            Space::new().width(3),
            row![
                file_tree_panel,
                Space::new().width(5),
                code_panel,
                Space::new().width(5)
            ]
            .spacing(10),
            status_bar
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn panel_style(theme: &Theme) -> container::Style {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.base.color.into()),
            border: iced::Border {
                color: palette.background.strong.color,
                width: 1.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        }
    }
}
