use std::path::{Path, PathBuf};

use crate::app;
use crate::core::profile;

pub fn load_files(app: &mut app::GothicOrganizer, root: Option<PathBuf>) {
    if let Some(profile_name) = app.profile_selected.as_ref()
        && let Some(profile) = app.profiles.get_mut(profile_name)
    {
        let root_dir = root.unwrap_or_else(|| profile.path.clone());
        app.state.current_directory = root_dir.clone();

        let get_current_dir_entries = |app_files: &profile::Lookup<PathBuf, profile::FileInfo>| {
            app_files
                .iter()
                .filter(|(path, _)| path.parent() == Some(&root_dir))
                .map(|(path, info)| (path.clone(), info.clone()))
                .collect::<Vec<(PathBuf, profile::FileInfo)>>()
        };

        let mut current_directory_entries: Vec<(PathBuf, crate::core::profile::FileInfo)>;

        if let Some(instance_name) = &app.instance_selected
            && let Some(instances) = &profile.instances
            && let Some(instance) = instances.get(instance_name)
            && let Some(instance_files) = &instance.files
        {
            log::trace!("Fetching files from current instance");
            app.files.extend(instance_files.clone());
            current_directory_entries = get_current_dir_entries(&app.files);
        } else {
            log::warn!("No instance selected, displaying only base files for current directory");
            current_directory_entries = get_current_dir_entries(&app.files);
        }

        current_directory_entries.sort_unstable_by_key(|(path, _)| !path.is_dir());
        app.state.current_directory_entries = current_directory_entries;
    }
}

pub fn toggle_state_recursive(app: &mut app::GothicOrganizer, path: Option<&Path>) {
    let paths_to_toggle: Vec<PathBuf> = path.map(|p| vec![p.to_path_buf()]).unwrap_or_else(|| {
        app.state
            .current_directory_entries
            .iter()
            .map(|(p, _)| p.clone())
            .collect()
    });

    paths_to_toggle.iter().for_each(|path_to_toggle| {
        if let Some(info) = app
            .state
            .current_directory_entries
            .iter_mut()
            .find_map(|(p, i)| (p == path_to_toggle).then_some(i))
        {
            info.enabled = !info.enabled;
            if path_to_toggle.is_dir() {
                app.files.insert(path_to_toggle.clone(), info.clone());
                app.files.iter_mut().for_each(|(p, i)| {
                    if p.starts_with(path_to_toggle) {
                        i.enabled = info.enabled;
                    }
                });
            }
        }
    });
}
