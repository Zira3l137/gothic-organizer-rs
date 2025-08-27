use iced::widget::column;

use crate::app::Message;
use crate::styled_container;

pub fn overwrites_view(app: &crate::app::GothicOrganizer) -> iced::Element<Message> {
    let mut mod_sections: iced::widget::Column<Message> = column![];

    if let Some(profile_name) = app.session.active_profile.as_ref()
        && let Some(instance_name) = app.session.active_instance.as_ref()
        && let Some(profile) = app.session.profiles.get(profile_name)
        && let Some(instances) = profile.instances.as_ref()
        && let Some(instance) = instances.get(instance_name)
        && let Some(overwrites) = instance.overwrites.as_ref()
    {
        let get_relative_path = |path: std::path::PathBuf| {
            path.strip_prefix(app.session.mod_storage_dir.clone().unwrap().join(profile_name))
                .unwrap_or_else(|err| panic!("Failed to get relative path: {path:?}\n{err}"))
                .to_path_buf()
        };

        mod_sections = overwrites
            .iter()
            .fold(column![], |col, (mod_name, file_data)| {
                let overwritten_count = file_data.len();

                let mod_info: iced::Element<Message> = iced::widget::text(format!(
                    "Mod - {mod_name}: {overwritten_count} files overwritten"
                ))
                .into();

                let files = file_data
                    .iter()
                    .fold(column![], |col, (replaces, replaced)| {
                        let (active, original) = (
                            get_relative_path(replaces.source_path.clone()),
                            get_relative_path(replaced.source_path.clone()),
                        );
                        let file_info: iced::Element<Message> = iced::widget::text(format!(
                            "\"{}\" overwrites \"{}\"",
                            active.display(),
                            original.display(),
                        ))
                        .into();
                        col.push(file_info)
                    })
                    .padding(10)
                    .spacing(10);

                col.push(
                    styled_container!(
                        column![mod_info, files].padding(10).spacing(10),
                        border_width = 2.0,
                        border_radius = 4.0
                    )
                    .center_x(iced::Length::Fill),
                )
            })
            .spacing(10)
            .padding(10);
    }

    styled_container!(mod_sections, border_width = 4.0, border_radius = 4.0)
        .center(iced::Length::Fill)
        .align_top(iced::Length::Fill)
        .into()
}
