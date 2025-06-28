use iced::widget::column;
use iced::widget::combo_box;
use iced::widget::row;
use iced::widget::text;
use iced::widget::text_input;
use iced::Element;
use iced::Length;

use crate::app::Message;
use crate::styled_container;

pub fn options_view(app: &crate::app::GothicOrganizer) -> Element<Message> {
    let label_theme = text("Application theme:");

    let choice_theme = combo_box(
        &app.state.theme_choices,
        "Application theme",
        app.theme.as_ref(),
        Message::ThemeSwitch,
    );

    let group_theme = row!(label_theme, choice_theme).spacing(10);

    let label_profile_dir = text("Game directory:");
    let input_profile_dir: Element<Message> = text_input("Game directory", app.state.profile_directory_input.as_ref())
        .on_input(Message::ProfileDirInput)
        .on_submit(Message::BrowseGameDir(
            None,
            Some(app.state.profile_directory_input.clone().into()),
        ))
        .into();

    let group_profile_dir = row!(label_profile_dir, input_profile_dir).spacing(10);

    styled_container!(
        column!(group_theme, group_profile_dir)
            .spacing(10)
            .padding(10),
        border_width = 4.0,
        border_radius = 4.0
    )
    .padding(10)
    .align_top(Length::Fill)
    .into()
}
