use iced::widget::column;
use iced::widget::combo_box;
use iced::widget::horizontal_space;
use iced::widget::row;
use iced::Element;
use iced::Length;

use crate::app::Message;
use crate::styled_container;

pub fn options_view(app: &crate::app::GothicOrganizer) -> Element<Message> {
    let choice_theme = combo_box(
        &app.state.theme_choices,
        "Application theme",
        app.theme.as_ref(),
        Message::ThemeSwitch,
    );

    let group_theme = row!(choice_theme, horizontal_space(),).spacing(10);

    styled_container!(
        column!(group_theme).spacing(10).padding(10),
        border_width = 4.0,
        border_radius = 4.0
    )
    .padding(10)
    .align_top(Length::Fill)
    .into()
}
