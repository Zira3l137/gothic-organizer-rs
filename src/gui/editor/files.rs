use iced::widget;

use crate::app;
use crate::clickable_text;
use crate::core::profile;
use crate::styled_container;
use crate::svg_with_color;

pub fn files_menu<'a>(
    app: &'a crate::app::GothicOrganizer,
    palette_ext: &iced::theme::palette::Extended,
    current_profile: Option<&profile::Profile>,
    instance_selected: Option<&profile::Instance>,
) -> iced::Element<'a, app::Message> {
    let mut container_bg_color = palette_ext.primary.weak.color;
    container_bg_color.a = 0.3;
    let file_view_controls = file_view_controls(app, current_profile);
    let file_view = file_view(app, instance_selected);

    styled_container!(
        widget::column!(file_view_controls, widget::scrollable(file_view)).spacing(10),
        border_width = 2.0,
        border_radius = 4.0,
        background = container_bg_color
    )
    .padding(10)
    .align_top(iced::Length::Fill)
    .into()
}

pub fn file_view_controls<'a>(
    app: &'a app::GothicOrganizer,
    current_profile: Option<&profile::Profile>,
) -> iced::Element<'a, app::Message> {
    let icon_back = svg_with_color!("./resources/back.svg").height(20).width(20);
    let icon_home = svg_with_color!("./resources/home.svg").height(20).width(20);
    let button_back_message = current_profile.and_then(|profile| {
        if profile.path == app.state.current_dir {
            return None;
        };
        Some(app::Message::UpdateActiveUiDir(
            app.state.current_dir.clone().parent().unwrap_or(profile.path.as_ref()).to_path_buf(),
        ))
    });
    let button_home_message = current_profile.and_then(|profile| {
        if profile.path == app.state.current_dir {
            return None;
        };
        Some(app::Message::UpdateActiveUiDir(profile.path.clone()))
    });

    let button_back = widget::button(icon_back).on_press_maybe(button_back_message);
    let button_home = widget::button(icon_home).on_press_maybe(button_home_message);
    let button_toggle_all =
        widget::button("Toggle all").on_press(app::Message::ToggleAllFileEntries);

    styled_container!(
        widget::row!(button_back, button_home, button_toggle_all).spacing(10),
        border_width = 1.0,
        border_radius = 4.0
    )
    .padding(10)
    .align_left(iced::Length::Fill)
    .into()
}

pub fn file_view<'a>(
    app: &'a app::GothicOrganizer,
    instance_selected: Option<&profile::Instance>,
) -> iced::Element<'a, app::Message> {
    if instance_selected.is_none() {
        return widget::Column::new().into();
    };

    app.state
        .dir_entries
        .iter()
        .fold(widget::Column::new(), |column, (path, info)| {
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let parent_name = info.parent_name.clone().unwrap_or(String::from("Default"));
            let tooltip_text = widget::text(format!("Supplied by: {parent_name}"));
            let is_dir = path.is_dir();
            let icon_path = match &is_dir {
                true => "./resources/directory.svg",
                false => "./resources/file.svg",
            };

            let label: iced::Element<_> = match &is_dir {
                true => clickable_text!("{file_name}")
                    .on_press(app::Message::UpdateActiveUiDir(path.clone()))
                    .into(),
                false => widget::text(file_name).into(),
            };

            let icon: iced::Element<_> = svg_with_color!(icon_path).height(20).width(20).into();
            let tooltip_body =
                styled_container!(tooltip_text, border_width = 1.0, border_radius = 4.0).padding(5);
            let checkbox = widget::checkbox("", info.enabled)
                .on_toggle(|_| app::Message::ToggleFileEntry(path.clone()));

            let file_entry = widget::tooltip(
                styled_container!(
                    widget::row![checkbox, icon, label],
                    border_width = 1.0,
                    border_radius = 4.0
                )
                .padding(5)
                .align_left(iced::Length::Fill),
                tooltip_body,
                widget::tooltip::Position::Left,
            );

            column.push(file_entry)
        })
        .into()
}
