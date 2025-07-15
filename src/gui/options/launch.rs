use iced::alignment;
use iced::widget;

use crate::app;
use crate::styled_container;

pub fn launch_menu(_app: &app::GothicOrganizer) -> iced::Element<app::Message> {
    styled_container!(
        widget::column![].align_x(alignment::Horizontal::Center).spacing(10).padding(10),
        border_width = 4.0,
        border_radius = 4.0
    )
    .padding(10)
    .center(iced::Length::Fill)
    .align_top(iced::Length::Fill)
    .into()
}
