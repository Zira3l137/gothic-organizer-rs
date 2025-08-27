use iced::widget::column;
use iced::widget::container;
use iced::widget::row;

use crate::app::message;

pub mod files;
pub mod header;
pub mod mods;
pub mod profile;

pub fn editor_view(app: &crate::app::GothicOrganizer) -> iced::Element<message::Message> {
    let current_profile =
        app.session.active_profile.as_ref().and_then(|s| app.session.profiles.get(s));

    let instance_selected =
        app.session.active_instance.as_ref().and_then(|s| {
            current_profile.and_then(|p| p.instances.as_ref().and_then(|i| i.get(s)))
        });

    let theme = app.theme();
    let palette_ext = theme.extended_palette();

    let header = header::header(palette_ext);
    let profile_controls = profile::profile_controls(app, palette_ext, current_profile);
    let files_menu = files::files_menu(app, palette_ext, current_profile, instance_selected);
    let mods_menu = mods::mods_menu(palette_ext, instance_selected);
    let workspace =
        container(row!(mods_menu, files_menu).spacing(10)).align_top(iced::Length::Fill);

    column![header, profile_controls, workspace].spacing(10).padding(10).into()
}
