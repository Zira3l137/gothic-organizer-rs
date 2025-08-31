use iced::widget;

use crate::app::message;
use crate::clickable_text;
use crate::core::profile;
use crate::styled_container;
use crate::svg_with_color;

pub fn files_menu<'a>(
    app: &'a crate::app::GothicOrganizer,
    palette_ext: &iced::theme::palette::Extended,
    current_profile: Option<&profile::Profile>,
    instance_selected: Option<&profile::Instance>,
) -> iced::Element<'a, message::Message> {
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
    app: &'a crate::app::GothicOrganizer,
    current_profile: Option<&profile::Profile>,
) -> iced::Element<'a, message::Message> {
    let theme = app.theme();
    let palette_ext = theme.extended_palette();
    let icon_back = svg_with_color!(
        "./resources/back.svg",
        color_idle = palette_ext.primary.strong.text,
        color_hovered = palette_ext.primary.strong.text
    )
    .height(20)
    .width(20);
    let icon_home = svg_with_color!(
        "./resources/home.svg",
        color_idle = palette_ext.primary.strong.text,
        color_hovered = palette_ext.primary.strong.text
    )
    .height(20)
    .width(20);
    let button_back_message = current_profile.and_then(|profile| {
        if profile.path == app.state.ui.current_dir {
            return None;
        };
        Some(
            message::UiMessage::UpdateActiveDir(
                app.state.ui.current_dir.clone().parent().unwrap_or(profile.path.as_ref()).to_path_buf(),
            )
            .into(),
        )
    });
    let button_home_message = current_profile.and_then(|profile| {
        if profile.path == app.state.ui.current_dir {
            return None;
        };
        Some(message::UiMessage::UpdateActiveDir(profile.path.clone()).into())
    });

    let button_back = widget::button(icon_back).on_press_maybe(button_back_message);
    let button_home = widget::button(icon_home).on_press_maybe(button_home_message);
    let button_toggle_all =
        widget::button("Toggle all").on_press(message::UiMessage::ToggleAllFileEntries.into());

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
    app: &'a crate::app::GothicOrganizer,
    instance_selected: Option<&profile::Instance>,
) -> iced::Element<'a, message::Message> {
    let theme = app.theme();
    let palette_ext = theme.extended_palette();

    if instance_selected.is_none() {
        return widget::Column::new().into();
    };

    app.state
        .ui
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
                    .on_press(message::UiMessage::UpdateActiveDir(path.clone()).into())
                    .into(),
                false => widget::text(file_name).into(),
            };

            let icon: iced::Element<_> = svg_with_color!(
                icon_path,
                color_idle = palette_ext.primary.base.color,
                color_hovered = palette_ext.primary.strong.color
            )
            .height(20)
            .width(20)
            .into();
            let tooltip_body =
                styled_container!(tooltip_text, border_width = 1.0, border_radius = 4.0).padding(5);
            let checkbox = widget::checkbox("", info.enabled).on_toggle(|entry_state| {
                message::UiMessage::ToggleFileEntry(entry_state, path.clone()).into()
            });

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
