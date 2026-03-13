use iced::{Font, widget::{row, text}};

use crate::app::Message;

pub fn metric_row(label: impl Into<String>, value: Option<String>) -> iced::widget::Row<'static, Message> {
    let label = label.into();
    
    row![
        text(label)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::DEFAULT
            })
            .size(16),
        text(value.unwrap_or_else(|| "Not available yet".to_string()))
            .size(16)
    ]
    .spacing(5)
}