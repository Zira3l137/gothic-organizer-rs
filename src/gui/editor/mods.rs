use iced::alignment;
use iced::theme::palette;
use iced::widget;

use crate::app;
use crate::clickable_text;
use crate::core::profile;
use crate::styled_container;
use crate::svg_with_color;

pub fn mods_menu<'a>(
    palette_ext: &palette::Extended,
    instance_selected: Option<&'a profile::Instance>,
) -> iced::Element<'a, app::Message> {
    let mut container_bg_color = palette_ext.primary.weak.color;
    container_bg_color.a = 0.3;

    let mods_view = mods_view(instance_selected);

    let icon_add = svg_with_color!(
        "./resources/add_mod.svg",
        color_idle = iced::Color::from_rgb(0.0, 230.0, 0.0),
        color_hovered = iced::Color::from_rgb(0.0, 255.0, 0.0)
    )
    .height(20)
    .width(20);

    let button_add_mod = widget::button(icon_add).on_press(app::Message::ModAdd(None));

    let group_mod_controls =
        styled_container!(widget::row!(button_add_mod), border_width = 1.0, border_radius = 4.0)
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
) -> iced::Element<'a, app::Message> {
    if let Some(instance) = current_instance
        && let Some(mods) = &instance.mods
    {
        mods.iter()
            .fold(widget::Column::new(), |column, mod_info| {
                let mod_name: iced::Element<_> = widget::text(mod_info.name.clone()).into();

                let checkbox = widget::checkbox("", mod_info.enabled).on_toggle(|new_state| {
                    app::Message::ModToggle(mod_info.name.clone(), new_state)
                });

                let button_uninstall = clickable_text!("Uninstall")
                    .on_press(app::Message::ModUninstall(mod_info.name.clone()));

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
