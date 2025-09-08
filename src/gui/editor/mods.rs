use iced::alignment;
use iced::theme::palette;
use iced::widget;

use crate::app::message;
use crate::core::constants::OPEN_PATH_COMMAND;
use crate::core::profile;
use crate::styled_button;
use crate::styled_container;
use crate::svg_with_color;

pub fn mods_menu<'a>(
    app: &crate::app::GothicOrganizer,
    palette_ext: &palette::Extended,
    instance_selected: Option<&'a profile::Instance>,
) -> iced::Element<'a, message::Message> {
    let mut container_bg_color = palette_ext.primary.weak.color;
    container_bg_color.a = 0.3;

    let mods_view = mods_view(app, instance_selected, palette_ext);

    let icon_add = svg_with_color!(
        "./resources/add_mod.svg",
        color_idle = palette_ext.primary.strong.text,
        color_hovered = palette_ext.primary.strong.text
    )
    .height(20)
    .width(20);

    let button_add_mod: iced::Element<'a, message::Message> = styled_button!(
        icon_add,
        background = palette_ext.success.base.color,
        hover_background = palette_ext.success.strong.color,
        pressed_background = palette_ext.success.base.color,
        disabled_background = palette_ext.success.weak.color,
    )
    .on_press(message::ModMessage::Add(None).into())
    .into();

    let button_remove: iced::Element<'a, message::Message> = styled_button!(
        svg_with_color!(
            "./resources/remove_mod.svg",
            color_idle = palette_ext.primary.strong.text,
            color_hovered = palette_ext.primary.strong.text
        )
        .width(20)
        .height(20),
        background = palette_ext.danger.base.color,
        hover_background = palette_ext.danger.strong.color,
        pressed_background = palette_ext.danger.base.color,
        disabled_background = palette_ext.danger.weak.color
    )
    .on_press_maybe(
        app.session.mod_selected.map(|mod_index| message::ModMessage::Uninstall(mod_index).into()),
    )
    .into();

    let button_browse: iced::Element<'a, message::Message> = styled_button!(
        svg_with_color!(
            "./resources/browse_mod.svg",
            color_idle = palette_ext.primary.strong.text,
            color_hovered = palette_ext.primary.strong.text
        )
        .width(20)
        .height(20),
        background = palette_ext.secondary.base.color,
        hover_background = palette_ext.secondary.strong.color,
        pressed_background = palette_ext.secondary.base.color,
        disabled_background = palette_ext.secondary.weak.color
    )
    .on_press_maybe(match app.session.mod_selected {
        Some(mod_index) => instance_selected.map(|i| &i.mods).and_then(|m| m.get(mod_index)).map(|m| {
            message::SystemMessage::ExecuteCommand(
                OPEN_PATH_COMMAND.to_owned(),
                vec![m.path.to_string_lossy().into_owned()],
            )
            .into()
        }),
        None => None,
    })
    .into();

    let button_conflicts: iced::Element<'a, message::Message> = styled_button!(
        svg_with_color!(
            "./resources/open_conflicts.svg",
            color_idle = palette_ext.primary.strong.text,
            color_hovered = palette_ext.primary.strong.text
        )
        .width(20)
        .height(20),
        background = palette_ext.secondary.base.color,
        hover_background = palette_ext.secondary.strong.color,
        pressed_background = palette_ext.secondary.base.color,
        disabled_background = palette_ext.secondary.weak.color
    )
    .on_press_maybe(app.session.mod_selected.map(|_| message::SystemMessage::Idle.into())) // TODO: Implement conflicts window
    .into();

    let group_mod_controls = styled_container!(
        widget::row!(button_add_mod, button_remove, button_conflicts, button_browse).spacing(10),
        border_width = 1.0,
        border_radius = 4.0
    )
    .padding(10)
    .align_left(iced::Length::Fill);

    styled_container!(
        widget::column!(group_mod_controls, widget::scrollable(mods_view)),
        border_width = 2.0,
        border_radius = 4.0,
        background = container_bg_color
    )
    .padding(10)
    .align_y(alignment::Vertical::Top)
    .center_x(iced::Length::Fill)
    .into()
}

pub fn mods_view<'a>(
    app: &crate::app::GothicOrganizer,
    current_instance: Option<&'a crate::core::profile::Instance>,
    palette_ext: &palette::Extended,
) -> iced::Element<'a, message::Message> {
    if let Some(instance) = current_instance {
        instance
            .mods
            .iter()
            .enumerate()
            .fold(widget::Column::new(), |column, (mod_index, mod_info)| {
                let toggle: iced::Element<'a, message::Message> = styled_button!(
                    if mod_info.enabled { "ON" } else { "OFF" },
                    background = if mod_info.enabled {
                        palette_ext.success.base.color
                    } else {
                        palette_ext.danger.base.color
                    },
                    hover_background = if mod_info.enabled {
                        palette_ext.success.strong.color
                    } else {
                        palette_ext.danger.strong.color
                    },
                    pressed_background = if mod_info.enabled {
                        palette_ext.success.base.color
                    } else {
                        palette_ext.danger.base.color
                    },
                    disabled_background = if mod_info.enabled {
                        palette_ext.success.weak.color
                    } else {
                        palette_ext.danger.weak.color
                    }
                )
                .on_press(message::ModMessage::Toggle(mod_index, !mod_info.enabled).into())
                .into();

                let mod_label: iced::Element<'a, message::Message> = widget::Text::new(&mod_info.name)
                    .align_y(alignment::Vertical::Center)
                    .line_height(widget::text::LineHeight::Relative(2.0))
                    .into();

                let mod_selected = app.session.mod_selected == Some(mod_index);

                let mod_entry = styled_button!(
                    widget::row![toggle, mod_label, widget::horizontal_space(),].spacing(10).padding(5),
                    border_width = 1.0,
                    border_color = palette_ext.primary.base.color,
                    border_radius = 4.0,
                    background =
                        if !mod_selected { iced::Color::TRANSPARENT } else { palette_ext.primary.base.color },
                    hover_background = palette_ext.primary.strong.color.scale_alpha(0.5),
                    pressed_background = palette_ext.primary.base.color.scale_alpha(0.5),
                    disabled_background = palette_ext.primary.weak.color.scale_alpha(0.5)
                )
                .padding(5)
                .on_press(message::ModMessage::ToggleSelection(mod_index).into());

                column.push(
                    widget::Container::new(mod_entry)
                        .align_y(alignment::Vertical::Center)
                        .align_left(iced::Length::Fill),
                )
            })
            .into()
    } else {
        widget::Column::new().into()
    }
}
