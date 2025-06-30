use std::path::{Path, PathBuf};

use crate::{
    app::GothicOrganizer,
    core::profile::{self, FileInfo},
};

pub fn load_files(app: &mut GothicOrganizer, root: Option<PathBuf>) {
    let Some(current_profile) = app
        .profiles
        .get_mut(&app.profile_selected.clone().unwrap_or_default())
    else {
        return;
    };

    let root_dir = root.unwrap_or_else(|| current_profile.path.clone());
    app.state.current_directory = root_dir.clone();

    let current_dir_entries = |app_files: &profile::Lookup<PathBuf, FileInfo>| {
        app_files
            .iter()
            .filter_map(|(path, info)| {
                path.parent().and_then(|parent| {
                    if parent == root_dir {
                        Some((path.clone(), info.clone()))
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<(PathBuf, FileInfo)>>()
    };

    if let Some(selected_instance) = &app.instance_selected
        && let Some(instances) = &current_profile.instances
        && let Some(current_instance) = instances.get(selected_instance)
    {
        log::trace!("Fetching files from current instance");
        if let Some(instance_files) = &current_instance.files
            && !instance_files.is_empty()
        {
            for (path, info) in instance_files.iter() {
                app.files.insert(path.clone(), info.clone());
            }
        }

        log::trace!("Clearing current directory entries");
        app.state.current_directory_entries.clear();

        log::trace!("Displaying fetched files for current directory");
        current_dir_entries(&app.files)
            .iter()
            .for_each(|(path, info)| {
                app.state
                    .current_directory_entries
                    .push((path.clone(), info.clone()));
            })
    } else {
        log::warn!("No instance selected, displaying only base files for current directory");
        app.state.current_directory_entries = current_dir_entries(&app.files);
    }

    log::trace!("Sorting current directory entries");
    app.state
        .current_directory_entries
        .sort_unstable_by_key(|(path, _)| !path.is_dir());
}

// FIXME: This is a mess
pub fn toggle_state_recursive(app: &mut GothicOrganizer, path: Option<&Path>) {
    if let Some(path) = path
        && let Some(old_state) = app
            .state
            .current_directory_entries
            .iter_mut()
            .find_map(|(p, s)| if p == path { Some(s) } else { None })
    {
        let new_state = !(old_state.enabled);
        old_state.enabled = new_state;
        if path.is_dir() {
            app.files.insert(path.to_path_buf(), old_state.clone());
            app.files.iter_mut().for_each(|(p, s)| {
                if p.starts_with(path) {
                    s.enabled = !(s.enabled);
                }
            })
        }
    } else {
        for (path, state) in app.state.current_directory_entries.iter_mut() {
            let new_state = !(state.enabled);
            state.enabled = new_state;
            if path.is_dir() {
                app.files.insert(path.clone(), state.clone());
                app.files.iter_mut().for_each(|(p, s)| {
                    if p.starts_with(path.clone()) {
                        s.enabled = !(s.enabled);
                    }
                })
            }
        }
    }
}
