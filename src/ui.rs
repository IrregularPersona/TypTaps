use iced::widget::{Space, button, column, container, row, text, text_editor};
use iced::{Alignment, Element, Length, Theme};
use crate::message::Message;
use crate::app::TypTaps;

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

    let status_bar = container(
        row![
            text(format!("Line: {} | Column: {}", app.cursor_line, app.cursor_column))
                .size(12)
                .color([0.7, 0.7, 0.7]),
            Space::new().width(Length::Fill),
            text(if app.file.is_some() { "Typst" } else { "" })
                .size(12)
                .color([0.7, 0.7, 0.7]),
        ]
    )
    .width(Length::Fill)
    .padding(5)
    .style(panel_style);

    column![
        top_bar,
        Space::new().height(3),
        row![
            file_tree_panel,
            Space::new().width(5),
            column![
                code_panel,
                Space::new().height(5),
                status_bar,
            ]
            .height(Length::Fill)
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
