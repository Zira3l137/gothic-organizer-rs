use iced::widget;

use crate::app;
use crate::styled_container;

pub fn config_menu(app: &app::GothicOrganizer) -> iced::Element<app::Message> {
    let theme_setting = theme_setting(app);
    let profile_setting = game_directory_setting(app);
    let mods_dir_setting = mods_dir_setting(app);

    styled_container!(
        widget::column!(theme_setting, profile_setting, mods_dir_setting).spacing(10).padding(10),
        border_width = 4.0,
        border_radius = 4.0
    )
    .padding(10)
    .align_top(iced::Length::Fill)
    .into()
}

pub fn mods_dir_setting(app: &app::GothicOrganizer) -> iced::Element<app::Message> {
    let label_mods_dir = widget::text!("Mods directory:");
    let input_mods_dir: iced::Element<app::Message> =
        widget::text_input("Mods directory", app.state.mods_dir_field.as_ref())
            .on_input_maybe(if app.session.active_profile.is_some() {
                Some(app::Message::UpdateModsDirField)
            } else {
                None
            })
            .on_submit(app::Message::SetModsDir(Some(app.state.mods_dir_field.clone().into())))
            .into();

    let button_browse_mods_dir =
        widget::button("...").on_press_maybe(if app.session.active_profile.is_some() {
            Some(app::Message::SetModsDir(None))
        } else {
            None
        });

    widget::row!(
        label_mods_dir,
        iced::widget::horizontal_space(),
        input_mods_dir,
        button_browse_mods_dir
    )
    .spacing(10)
    .into()
}

pub fn game_directory_setting(app: &app::GothicOrganizer) -> iced::Element<app::Message> {
    let label_profile_dir = widget::text!("Game directory:");
    let input_profile_dir: iced::Element<app::Message> =
        widget::text_input("Game directory", app.state.profile_dir_field.as_ref())
            .on_input_maybe(if app.session.active_profile.is_some() {
                Some(app::Message::UpdateProfileDirField)
            } else {
                None
            })
            .on_submit(app::Message::SetGameDir(Some(app.state.profile_dir_field.clone().into())))
            .into();

    let button_browse_profile_dir =
        widget::button("...").on_press_maybe(if app.session.active_profile.is_some() {
            Some(app::Message::SetGameDir(None))
        } else {
            None
        });

    widget::row!(
        label_profile_dir,
        widget::horizontal_space(),
        input_profile_dir,
        button_browse_profile_dir
    )
    .spacing(10)
    .into()
}

pub fn theme_setting(app: &app::GothicOrganizer) -> iced::Element<app::Message> {
    let label_theme = widget::text!("Application theme:");

    let choice_theme = widget::combo_box(
        &app.state.theme_choices,
        "Application theme",
        app.session.theme_selected.as_ref(),
        app::Message::SetUiTheme,
    );

    widget::row!(label_theme, iced::widget::horizontal_space(), choice_theme).spacing(10).into()
}
