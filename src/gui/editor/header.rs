use iced::alignment;
use iced::widget;

use crate::app::message;
use crate::core::constants;
use crate::styled_container;
use crate::svg_with_color;

pub fn header<'a>(
    app: &crate::app::GothicOrganizer,
    palette_ext: &iced::theme::palette::Extended,
) -> iced::Element<'a, message::Message> {
    let mut container_bg_color = palette_ext.primary.weak.color;
    container_bg_color.a = 0.3;
    let logo = svg_with_color!(
        "./resources/logo.svg",
        color_idle = palette_ext.primary.strong.color,
        color_hovered = palette_ext.primary.strong.color
    )
    .height(60)
    .width(60);

    let title = widget::text!("{}", constants::app_title_full()).size(30);

    let button_options_icon = svg_with_color!(
        "./resources/options.svg",
        color_idle = palette_ext.primary.strong.text,
        color_hovered = palette_ext.primary.strong.text
    )
    .height(20)
    .width(20);
    let button_options =
        widget::button(button_options_icon).on_press(message::WindowMessage::Open("options".into()).into());

    let (button_logs_color_idle, buton_logs_color_hovered) = match app.state.errors.active_errors.len() {
        0 => (palette_ext.primary.strong.text, palette_ext.primary.strong.text),
        _ => (palette_ext.danger.strong.color, palette_ext.danger.strong.color),
    };

    let button_logs_icon = svg_with_color!(
        "./resources/logs.svg",
        color_idle = button_logs_color_idle,
        color_hovered = buton_logs_color_hovered
    )
    .height(20)
    .width(20);

    let button_logs =
        widget::button(button_logs_icon).on_press(message::WindowMessage::Open("logs".into()).into());

    let header = widget::row!(title, widget::horizontal_space(), button_logs, button_options)
        .spacing(10)
        .padding(10)
        .align_y(alignment::Vertical::Center);

    styled_container!(
        widget::row![logo, header].spacing(10),
        border_width = 2.0,
        border_radius = 4.0,
        background = container_bg_color
    )
    .padding(10)
    .align_y(alignment::Vertical::Center)
    .center_x(iced::Length::Fill)
    .into()
}
