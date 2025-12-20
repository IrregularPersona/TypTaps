use crate::app::{TreeEntry, TypTaps};
use crate::message::Message;
use crate::utils::get_icons;
use iced::widget::image::viewer;
use iced::widget::{Space, button, column, container, row, scrollable, text, text_editor};
use iced::{Alignment, Element, Length, Theme};

pub fn view(app: &TypTaps) -> Element<'_, Message> {
    let top_bar = container(row![
        button(text("Open File").size(10))
            .on_press(Message::OpenFile)
            .width(Length::Fixed(75.0)),
        Space::new().width(10),
        button(text("Open Folder").size(10))
            .on_press(Message::OpenDir)
            .width(Length::Fixed(85.0)),
    ])
    .width(Length::Fill);

    let file_tree_content = if app.file_tree.is_empty() {
        container(text("No folder open").size(14).color([0.7, 0.7, 0.7])).padding(10)
    } else {
        container(scrollable(
            column(app.file_tree.iter().map(|entry| view_tree_entry(entry, 0))).spacing(2),
        ))
        .padding(5)
    };

    let file_tree_panel = container(file_tree_content)
        .width(Length::Fixed(200.0)) // Increased width for better tree visibility
        .height(Length::FillPortion(8))
        .style(panel_style);

    let editor_content: Element<'_, Message> = if app.file.is_some() {
        text_editor(&app.content)
            .on_action(Message::Edit)
            .height(Length::Fill)
            .into()
    } else {
        container(
            column![
                text("TypTaps").size(40),
                text("A minimal Typst editor")
                    .size(18)
                    .color([0.6, 0.6, 0.6]),
                Space::new().height(20),
                row![
                    button(text("Open File").size(14))
                        .on_press(Message::OpenFile)
                        .padding(10),
                    button(text("Open Folder").size(14))
                        .on_press(Message::OpenDir)
                        .padding(10),
                ]
                .spacing(20),
            ]
            .spacing(10)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    };

    let code_panel = container(editor_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(15)
        .style(panel_style);

    let status_bar = container(row![
        text(format!(
            "Line: {} | Column: {}",
            app.cursor_line, app.cursor_column
        ))
        .size(12)
        .color([0.7, 0.7, 0.7]),
        Space::new().width(Length::Fill),
        text(if app.file.is_some() { "Typst" } else { "" })
            .size(12)
            .color([0.7, 0.7, 0.7]),
    ])
    .width(Length::Fill)
    .padding(5)
    .style(panel_style);

    let preview_content: Element<Message> = if let Some(handle) = &app.image_handle {
        viewer(handle.clone()).min_scale(1.0).max_scale(10.0).into()
    } else {
        iced::widget::text("Waiting for Typst...").into()
    };

    let preview_col = container(preview_content)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .padding(10);

    column![
        top_bar,
        Space::new().height(3),
        row![
            file_tree_panel,
            Space::new().width(5),
            column![code_panel, Space::new().height(5), status_bar,].height(Length::Fill),
            Space::new().width(5),
            preview_col,
        ]
        .spacing(10)
        .height(Length::Fill),
    ]
    .spacing(10)
    .padding(10)
    .into()
}

pub fn panel_style(theme: &Theme) -> container::Style {
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

fn view_tree_entry(entry: &TreeEntry, depth: u32) -> Element<'_, Message> {
    match entry {
        TreeEntry::File { name, path } => {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let (_, file_icon) = get_icons(ext);
            let file_text = format!("{} {}", file_icon, name);
            button(text(file_text).size(14).wrapping(text::Wrapping::None))
                .on_press(Message::FileOpened(Ok(path.clone())))
                .style(button::text)
                .padding(iced::Padding {
                    top: 2.0,
                    right: 5.0,
                    bottom: 2.0,
                    left: 5.0 + (depth * 12) as f32,
                })
                .width(Length::Fill)
                .into()
        }
        TreeEntry::Directory {
            name,
            path,
            is_expanded,
            children,
        } => {
            let icon = if *is_expanded { "▼" } else { "▶" };
            let header = button(
                text(format!("{} {}", icon, name))
                    .size(16)
                    .wrapping(text::Wrapping::None),
            )
            .on_press(Message::ToggleDir(path.clone()))
            .style(button::text)
            .padding(iced::Padding {
                top: 2.0,
                right: 5.0,
                bottom: 2.0,
                left: 5.0 + (depth * 12) as f32,
            })
            .width(Length::Fill);

            if *is_expanded {
                let mut col = column![header].spacing(2);
                for child in children {
                    col = col.push(view_tree_entry(child, depth + 1));
                }
                col.into()
            } else {
                header.into()
            }
        }
    }
}
