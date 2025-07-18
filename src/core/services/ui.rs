use std::path;

use crate::app;
use crate::core;
use crate::core::services::Service;

pub struct UiService<'a> {
    session: &'a mut core::services::session::SessionService,
    state: &'a mut app::InnerState,
}

crate::impl_service!(UiService);

impl<'a> UiService<'a> {
    pub fn new(
        session: &'a mut core::services::session::SessionService,
        state: &'a mut app::InnerState,
    ) -> Self {
        Self { session, state }
    }

    pub fn reload_displayed_directory(&mut self, root: Option<path::PathBuf>) {
        let Ok(context) = self.context() else {
            return;
        };

        let profile_path = context.active_profile.path.clone();
        let instance_files = context.instance_files().cloned();

        let root_dir = root.unwrap_or_else(|| profile_path.clone());
        let mut current_directory_entries: Vec<(path::PathBuf, core::profile::FileInfo)>;

        if let Some(instance_files) = instance_files {
            self.session.files.clear();
            self.session.files.extend(instance_files);
        } else {
            log::warn!("No instance selected, displaying only base files for current directory");
        }

        current_directory_entries = self
            .session
            .files
            .iter()
            .filter(|(path, _)| path.parent() == Some(&root_dir))
            .map(|(path, info)| (path.clone(), info.clone()))
            .collect::<Vec<(path::PathBuf, core::profile::FileInfo)>>();

        current_directory_entries.sort_unstable_by_key(|(path, _)| !path.is_dir());

        self.state.current_directory = root_dir.clone();
        self.state.current_directory_entries = current_directory_entries;
    }

    pub fn toggle_state_recursive(&mut self, path: Option<&path::Path>) {
        let paths_to_toggle: Vec<path::PathBuf> =
            path.map(|p| vec![p.to_path_buf()]).unwrap_or_else(|| {
                self.state.current_directory_entries.iter().map(|(p, _)| p.clone()).collect()
            });

        paths_to_toggle.iter().for_each(|path_to_toggle| {
            if let Some(info) = self
                .state
                .current_directory_entries
                .iter_mut()
                .find_map(|(p, i)| (p == path_to_toggle).then_some(i))
            {
                info.enabled = !info.enabled;
                if path_to_toggle.is_dir() {
                    self.session.files.insert(path_to_toggle.clone(), info.clone());
                    self.session.files.iter_mut().for_each(|(p, i)| {
                        if p.starts_with(path_to_toggle) {
                            i.enabled = info.enabled;
                        }
                    });
                }
            }
        });
    }
}
