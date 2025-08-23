use crate::app;
use crate::core::constants;
use crate::styled_container;
use crate::svg_with_color;
use iced::alignment;
use iced::widget;

pub fn header<'a>(palette_ext: &iced::theme::palette::Extended) -> iced::Element<'a, app::Message> {
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
    let button_options_icon = svg_with_color!("./resources/options.svg").height(20).width(20);
    let button_options = widget::button(button_options_icon)
        .on_press(app::Message::RequestWindowOpen("options".into()));

    let header = widget::row!(title, widget::horizontal_space(), button_options)
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
