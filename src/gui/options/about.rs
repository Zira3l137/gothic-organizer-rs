use iced::alignment;
use iced::theme::palette;
use iced::widget;

use crate::app;
use crate::clickable_text;
use crate::core::constants;
use crate::styled_container;
use crate::svg_with_color;

pub fn about_menu<'a>(palette_ext: &palette::Extended) -> iced::Element<'a, app::Message> {
    let logo = svg_with_color!(
        "./resources/logo.svg",
        color_idle = palette_ext.primary.strong.color,
        color_hovered = palette_ext.primary.strong.color
    )
    .height(60)
    .width(60);

    let title = widget::text!("{}", constants::app_title_full()).size(30);

    let header =
        widget::row!(logo, title).spacing(10).padding(10).align_y(alignment::Vertical::Center);

    let link = widget::row![
        widget::text!("Repository: "),
        clickable_text!("{}", constants::APP_REPOSITORY)
            .on_press(app::Message::OpenRepository)
            .color(palette_ext.primary.base.color)
            .color_hovered(palette_ext.primary.strong.color)
    ];

    styled_container!(
        widget::column!(
            header,
            widget::text!("Version: {}", constants::APP_VERSION),
            widget::text!("Authors: {}", constants::APP_AUTHORS),
            link
        )
        .align_x(alignment::Horizontal::Center)
        .spacing(10)
        .padding(10),
        border_width = 4.0,
        border_radius = 4.0
    )
    .padding(10)
    .center(iced::Length::Fill)
    .align_top(iced::Length::Fill)
    .into()
}
