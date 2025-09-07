use std::path;
use std::path::Path;

use iced::Task;

use crate::app::message;
use crate::app::session;
use crate::app::state;
use crate::core;
use crate::error::ErrorContext;

pub struct UiService<'a> {
    session: &'a mut session::ApplicationSession,
    state: &'a mut state::ApplicationState,
}

impl<'a> UiService<'a> {
    pub fn new(session: &'a mut session::ApplicationSession, state: &'a mut state::ApplicationState) -> Self {
        Self { session, state }
    }

    pub fn reload_displayed_directory(&mut self, root: Option<&Path>) -> Task<message::Message> {
        match self.try_reload_displayed_directory(root) {
            Ok(()) => Task::none(),
            Err(err) => Task::done(message::ErrorMessage::Handle(err).into()),
        }
    }

    fn try_reload_displayed_directory(&mut self, root: Option<&Path>) -> Result<(), ErrorContext> {
        if self.session.active_profile.is_none() {
            return Ok(());
        }
        let active_profile_name = &self.session.active_profile.clone().unwrap();
        let active_profile = self.state.profile.profiles.get_mut(active_profile_name).unwrap();
        let active_instance_name = &self.session.active_instance.clone().unwrap();
        let active_instance =
            active_profile.instances.as_mut().unwrap().get_mut(active_instance_name).unwrap();
        let active_instance_files = active_instance.files.as_ref();
        let profile_path = active_profile.path.clone();

        let root_dir = root.unwrap_or(&profile_path);
        let mut current_directory_entries: Vec<(path::PathBuf, core::profile::FileMetadata)>;

        if let Some(instance_files) = active_instance_files.cloned() {
            self.session.files.clear();
            self.session.files.extend(instance_files);
        }

        current_directory_entries = self
            .session
            .files
            .iter()
            .filter_map(|(path, info)| {
                (path.parent() == Some(root_dir)).then_some((path.clone(), info.clone()))
            })
            .collect::<Vec<(path::PathBuf, core::profile::FileMetadata)>>();

        current_directory_entries.sort_unstable_by_key(|(path, _)| !path.is_dir());

        tracing::info!("Reloading entries for UI from: \"{}\"", root_dir.display());
        self.state.ui.current_dir = root_dir.to_path_buf();
        self.state.ui.dir_entries = current_directory_entries;

        Ok(())
    }

    pub fn set_entry_state_with_children(&mut self, state: Option<bool>, path: Option<&path::Path>) {
        let paths_to_toggle: Vec<path::PathBuf> = path
            .map(|p| vec![p.to_path_buf()])
            .unwrap_or_else(|| self.state.ui.dir_entries.iter().map(|(p, _)| p.clone()).collect());

        paths_to_toggle.iter().for_each(|path_to_toggle| {
            if let Some(info) =
                self.state.ui.dir_entries.iter_mut().find_map(|(p, i)| (p == path_to_toggle).then_some(i))
            {
                info.enabled = state.unwrap_or(!info.enabled);
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
