use std::path;

use iced::Task;

use crate::app::message;
use crate::app::session;
use crate::app::state;
use crate::core;
use crate::core::services::ApplicationContext;

pub struct UiService<'a> {
    session: &'a mut session::ApplicationSession,
    state: &'a mut state::ApplicationState,
}

crate::impl_app_context!(UiService);

impl<'a> UiService<'a> {
    pub fn new(session: &'a mut session::ApplicationSession, state: &'a mut state::ApplicationState) -> Self {
        Self { session, state }
    }

    /// Reloads the directory displayed in the UI from the active profile directory.
    /// If root is provided, it will be used instead.
    ///
    /// Emits an error message if the context could not be obtained.
    pub fn reload_displayed_directory(&mut self, root: Option<path::PathBuf>) -> Task<message::Message> {
        if self.session.active_profile.is_none() {
            return Task::none();
        }

        let context = match self.context() {
            Ok(ctx) => ctx,
            Err(err) => {
                tracing::error!("Failed to get context: {err}");
                return Task::done(message::ErrorMessage::Handle(err.into()).into());
            }
        };

        let profile_path = context.active_profile.path.clone();
        let instance_files = context.instance_files().cloned();

        let root_dir = root.unwrap_or(profile_path.clone());
        let mut current_directory_entries: Vec<(path::PathBuf, core::profile::FileMetadata)>;

        if let Some(instance_files) = instance_files {
            self.session.files.clear();
            self.session.files.extend(instance_files);
        }

        current_directory_entries = self
            .session
            .files
            .iter()
            .filter_map(|(path, info)| {
                (path.parent() == Some(&root_dir)).then_some((path.clone(), info.clone()))
            })
            .collect::<Vec<(path::PathBuf, core::profile::FileMetadata)>>();

        current_directory_entries.sort_unstable_by_key(|(path, _)| !path.is_dir());

        tracing::info!("Reloading entries for UI from: \"{}\"", root_dir.display());
        self.state.ui.current_dir = root_dir.clone();
        self.state.ui.dir_entries = current_directory_entries;

        Task::none()
    }

    /// Sets the enabled state of a directory entry and its children to `state`. If a `path` is provided, it will be toggled
    /// otherwise, all directory entries will be toggled.
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
