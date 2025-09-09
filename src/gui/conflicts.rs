use std::path::PathBuf;

use iced::widget;

use crate::styled_container;

pub fn conflicts_view(app: &crate::app::GothicOrganizer) -> iced::Element<crate::app::message::Message> {
    if let Some(active_profile_name) = app.session.active_profile.as_ref()
        && let Some(active_instance_name) = app.session.active_instance.as_ref()
        && let Some(active_profile) = app.state.profile.profiles.get(active_profile_name)
        && let Some(active_profile_instances) = active_profile.instances.as_ref()
        && let Some(active_instance) = active_profile_instances.get(active_instance_name)
        && let Some(active_mod_index) = app.session.mod_selected.as_ref()
        && let Some(active_mod) = active_instance.mods.get(*active_mod_index)
        && let Some(active_mod_priority) = active_instance.load_order.get(&active_mod.name)
        && !active_instance.conflicts.is_empty()
    {
        let active_mod_conflict_entries = active_instance.conflicts.iter().filter(|(dst_path, _)| {
            let relative_path = dst_path.strip_prefix(&active_profile.path).unwrap();
            let mod_relative_path = active_mod.path.join(relative_path);
            active_mod.files.contains_key(&mod_relative_path)
        });

        let mut files_from_other_mods_data: Vec<(PathBuf, String)> = vec![];
        let mut files_from_this_mod_data: Vec<(PathBuf, String)> = vec![];

        active_mod_conflict_entries.for_each(|(dst_path, conflict)| {
            conflict.iter().for_each(|(priority, metadata)| match priority.cmp(active_mod_priority) {
                std::cmp::Ordering::Less => {
                    files_from_other_mods_data.push((dst_path.clone(), metadata.parent_name.clone()));
                }
                std::cmp::Ordering::Greater => {
                    files_from_this_mod_data.push((dst_path.clone(), metadata.parent_name.clone()));
                }
                _ => (),
            });
        });

        let files_from_other_mods = files_from_other_mods_data
            .iter()
            .fold(widget::column![], |col, (dst_path, parent_name)| {
                col.push(widget::row![
                    widget::text(dst_path.to_string_lossy().into_owned()),
                    widget::horizontal_space(),
                    widget::text(parent_name.clone())
                ])
            })
            .spacing(5)
            .padding(10);

        let files_from_this_mod = files_from_this_mod_data
            .iter()
            .fold(widget::column![], |col, (dst_path, parent_name)| {
                col.push(widget::row![
                    widget::text(dst_path.to_string_lossy().into_owned()),
                    widget::horizontal_space(),
                    widget::text(parent_name.clone())
                ])
            })
            .spacing(5)
            .padding(10);

        styled_container!(
            widget::column![
                widget::column![
                    widget::text("Files from other mods replaced by this mod:"),
                    styled_container!(
                        widget::scrollable(files_from_other_mods),
                        border_width = 2.0,
                        border_radius = 4.0
                    )
                    .center(iced::Length::Fill)
                    .align_top(iced::Length::Fill),
                ]
                .spacing(5),
                widget::column![
                    widget::text("Files from this mod replaced by other mods:"),
                    styled_container!(
                        widget::scrollable(files_from_this_mod),
                        border_width = 2.0,
                        border_radius = 4.0
                    )
                    .center(iced::Length::Fill)
                    .align_top(iced::Length::Fill),
                ]
                .spacing(5),
            ]
            .padding(20)
            .spacing(10),
            border_width = 4.0,
            border_radius = 4.0
        )
        .padding(10)
        .center(iced::Length::Fill)
        .align_top(iced::Length::Fill)
        .into()
    } else {
        styled_container!(
            widget::column![
                widget::column![
                    widget::text("Files from other mods replaced by this mod:"),
                    styled_container!(widget::column![], border_width = 2.0, border_radius = 4.0)
                        .center(iced::Length::Fill)
                        .align_top(iced::Length::Fill),
                ]
                .spacing(5),
                widget::column![
                    widget::text("Files from this mod replaced by other mods:"),
                    styled_container!(widget::column![], border_width = 2.0, border_radius = 4.0)
                        .center(iced::Length::Fill)
                        .align_top(iced::Length::Fill),
                ]
                .spacing(5),
            ]
            .padding(20)
            .spacing(10),
            border_width = 4.0,
            border_radius = 4.0
        )
        .padding(10)
        .center(iced::Length::Fill)
        .align_top(iced::Length::Fill)
        .into()
    }
}
