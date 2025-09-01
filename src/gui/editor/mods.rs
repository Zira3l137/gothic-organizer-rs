use iced::alignment;
use iced::theme::palette;
use iced::widget;

use crate::app::message;
use crate::clickable_text;
use crate::core::profile;
use crate::styled_button;
use crate::styled_container;
use crate::svg_with_color;

pub fn mods_menu<'a>(
    palette_ext: &palette::Extended,
    instance_selected: Option<&'a profile::Instance>,
) -> iced::Element<'a, message::Message> {
    let mut container_bg_color = palette_ext.primary.weak.color;
    container_bg_color.a = 0.3;

    let mods_view = mods_view(instance_selected);

    let icon_add = svg_with_color!(
        "./resources/add_mod.svg",
        color_idle = palette_ext.primary.strong.text,
        color_hovered = palette_ext.primary.strong.text
    )
    .height(20)
    .width(20);

    let button_add_mod = styled_button!(
        icon_add,
        background = palette_ext.success.base.color,
        hover_background = palette_ext.success.strong.color,
        pressed_background = palette_ext.success.base.color,
        disabled_background = palette_ext.success.weak.color,
    )
    .on_press(message::ModMessage::Add(None).into());

    let group_mod_controls =
        styled_container!(widget::row!(button_add_mod).spacing(10), border_width = 1.0, border_radius = 4.0)
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
    current_instance: Option<&'a crate::core::profile::Instance>,
) -> iced::Element<'a, message::Message> {
    if let Some(instance) = current_instance
        && let Some(mods) = &instance.mods
    {
        mods.iter()
            .fold(widget::Column::new(), |column, mod_info| {
                let mod_name: iced::Element<_> = widget::text(mod_info.name.clone()).into();

                let checkbox = widget::checkbox("", mod_info.enabled).on_toggle(|new_state| {
                    message::ModMessage::Toggle(mod_info.name.clone(), new_state).into()
                });

                let button_uninstall = clickable_text!("Uninstall")
                    .on_press(message::ModMessage::Uninstall(mod_info.name.clone()).into());

                let mod_entry = styled_container!(
                    widget::row![checkbox, mod_name, widget::horizontal_space(), button_uninstall],
                    border_width = 1.0,
                    border_radius = 4.0
                )
                .padding(5)
                .align_left(iced::Length::Fill);

                column.push(mod_entry)
            })
            .into()
    } else {
        widget::Column::new().into()
    }
}
