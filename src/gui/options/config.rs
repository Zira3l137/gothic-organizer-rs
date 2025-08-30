use iced::widget;

use crate::app::message;
use crate::styled_container;

pub fn config_menu(app: &crate::app::GothicOrganizer) -> iced::Element<message::Message> {
    let theme_setting = theme_setting(app);
    let profile_setting = game_directory_setting(app);
    let mods_dir_setting = mods_dir_setting(app);
    let notifications_setting = notifications_setting(app);

    styled_container!(
        widget::column!(theme_setting, profile_setting, mods_dir_setting, notifications_setting)
            .spacing(10)
            .padding(10),
        border_width = 4.0,
        border_radius = 4.0
    )
    .padding(10)
    .align_top(iced::Length::Fill)
    .into()
}

pub fn mods_dir_setting(app: &crate::app::GothicOrganizer) -> iced::Element<message::Message> {
    let label_mods_dir = widget::text!("Mods directory:");
    let input_mods_dir: iced::Element<message::Message> =
        widget::text_input("Mods directory", app.state.mod_management.mods_dir_field.as_ref())
            .on_input_maybe(
                app.session
                    .active_profile
                    .clone()
                    .map(|_| |input| message::ModMessage::UpdateModsDirField(input).into()),
            )
            .on_submit(
                message::ModMessage::SetModsDir(Some(app.state.mod_management.mods_dir_field.clone().into()))
                    .into(),
            )
            .into();

    let button_browse_mods_dir =
        widget::button("...").on_press_maybe(if app.session.active_profile.is_some() {
            Some(message::ModMessage::SetModsDir(None).into())
        } else {
            None
        });

    widget::row!(label_mods_dir, iced::widget::horizontal_space(), input_mods_dir, button_browse_mods_dir)
        .spacing(10)
        .into()
}

pub fn game_directory_setting(app: &crate::app::GothicOrganizer) -> iced::Element<message::Message> {
    let label_profile_dir = widget::text!("Game directory:");
    let input_profile_dir: iced::Element<message::Message> =
        widget::text_input("Game directory", app.state.profile.profile_dir_field.as_ref())
            .on_input_maybe(
                app.session
                    .active_profile
                    .clone()
                    .map(|_| |input| message::ProfileMessage::UpdateProfileDirField(input).into()),
            )
            .on_submit(
                message::ProfileMessage::SetGameDir(Some(app.state.profile.profile_dir_field.clone().into()))
                    .into(),
            )
            .into();

    let button_browse_profile_dir =
        widget::button("...").on_press_maybe(if app.session.active_profile.is_some() {
            Some(message::ProfileMessage::SetGameDir(None).into())
        } else {
            None
        });

    widget::row!(label_profile_dir, widget::horizontal_space(), input_profile_dir, button_browse_profile_dir)
        .spacing(10)
        .into()
}

pub fn theme_setting(app: &crate::app::GothicOrganizer) -> iced::Element<message::Message> {
    let label_theme = widget::text!("Application theme:");

    let choice_theme = widget::combo_box(
        &app.state.settings.theme_choices,
        "Application theme",
        app.session.theme_selected.as_ref(),
        |theme| message::UiMessage::SetTheme(theme).into(),
    );

    widget::row!(label_theme, iced::widget::horizontal_space(), choice_theme).spacing(10).into()
}

pub fn notifications_setting(app: &crate::app::GothicOrganizer) -> iced::Element<message::Message> {
    let notify_on_error_checkbox =
        widget::checkbox("Notify on error", app.session.error_notifications_enabled)
            .on_toggle(|new_state| message::SettingsMessage::ToggleErrorNotifications(new_state).into());

    widget::row!(notify_on_error_checkbox).spacing(10).into()
}
