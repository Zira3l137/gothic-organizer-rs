use iced::widget;

use crate::app;
use crate::core::profile;
use crate::styled_container;

pub fn profile_controls<'a>(
    app: &'a crate::app::GothicOrganizer,
    palette_ext: &iced::theme::palette::Extended,
    current_profile: Option<&profile::Profile>,
) -> iced::Element<'a, app::Message> {
    let mut container_bg_color = palette_ext.primary.weak.color;
    container_bg_color.a = 0.3;
    let instance_controls = instance_controls(app, current_profile);
    let button_browse = widget::button("Browse").on_press(app::Message::SetGameDir(None));
    let choice_profile = widget::combo_box(
        &app.state.profile_choices,
        "Profile",
        app.session.active_profile.as_ref(),
        app::Message::SetActiveProfile,
    );

    let instance_element = match current_profile {
        Some(profile) if no_profile_path(profile) => {
            let button_browse: iced::Element<_> = button_browse.into();
            button_browse
        }
        Some(profile) if !no_profile_path(profile) => instance_controls,
        _ => widget::horizontal_space().into(),
    };

    styled_container!(
        widget::row!(choice_profile, instance_element).spacing(10),
        border_width = 2.0,
        border_radius = 4.0,
        background = container_bg_color
    )
    .padding(10)
    .center_x(iced::Length::Fill)
    .into()
}

pub fn instance_controls<'a>(
    app: &'a app::GothicOrganizer,
    current_profile: Option<&profile::Profile>,
) -> iced::Element<'a, app::Message> {
    let choice_instance = widget::combo_box(
        &app.state.instance_choices,
        "Instance",
        app.session.active_instance.as_ref(),
        app::Message::SetActiveInstance,
    )
    .on_input(app::Message::UpdateInstanceNameField);

    let button_add_message = current_profile.and_then(|p| {
        if no_profile_path(p) {
            return None;
        };
        Some(app::Message::AddNewInstance(p.name.clone()))
    });
    let button_remove_message = current_profile.and_then(|p| {
        if no_profile_path(p) {
            return None;
        };
        Some(app::Message::RemoveActiveInstance)
    });

    let button_add = widget::button("Add").on_press_maybe(button_add_message);
    let button_remove = widget::button("Remove").on_press_maybe(button_remove_message);
    widget::container(widget::row!(choice_instance, button_add, button_remove)).into()
}

fn no_profile_path(p: &crate::core::profile::Profile) -> bool {
    p.path.as_os_str().is_empty()
}
