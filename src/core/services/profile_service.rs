use std::path;

use iced::Task;

use crate::app;
use crate::core;
use crate::core::lookup;
use crate::core::profile;
use crate::core::services::Service;
use crate::error;

pub struct ProfileService<'a> {
    session: &'a mut core::services::session_service::SessionService,
    app_state: &'a mut app::InnerState,
}

impl super::Service for ProfileService<'_> {
    fn context(&mut self) -> Result<super::context::Context, crate::error::GothicOrganizerError> {
        let profile = self
            .session
            .active_profile
            .as_mut()
            .and_then(|p| self.session.profiles.get_mut(&p.clone()))
            .ok_or_else(|| crate::error::GothicOrganizerError::Other("No active profile".into()))?;

        let instance_name = self
            .session
            .active_instance
            .as_ref()
            .ok_or_else(|| crate::error::GothicOrganizerError::Other("No active instance".into()))?;

        Ok(super::context::Context::new(profile, instance_name))
    }
}

impl<'a> ProfileService<'a> {
    pub fn new(session: &'a mut core::services::session_service::SessionService, app_state: &'a mut app::InnerState) -> Self {
        Self { session, app_state }
    }

    pub fn switch_profile(&mut self, profile_name: &str) -> Task<app::Message> {
        if let Err(err) = self.update_instance_from_cache() {
            log::error!("Failed to update instance from cache: {err}");
        }

        if let Some(next_profile) = self.session.profiles.get(profile_name) {
            self.session.active_profile = Some(profile_name.to_owned());
            self.session.active_instance = None;

            let instances = next_profile
                .instances
                .as_ref()
                .map(|i| i.keys().cloned().collect())
                .unwrap_or_default();

            self.app_state.instance_choices = iced::widget::combo_box::State::new(instances);

            if !next_profile.path.as_os_str().is_empty() {
                return Task::done(app::Message::CurrentDirectoryUpdated);
            }
        }

        Task::none()
    }

    pub fn update_instance_from_cache(&mut self) -> Result<(), error::GothicOrganizerError> {
        self.session
            .files
            .extend(self.app_state.current_directory_entries.iter().cloned());

        let cached_files = self.session.files.clone();
        let mut context = self.context()?;

        context.instance_mut(None).files = Some(cached_files);
        Ok(())
    }

    pub fn add_instance_for_profile(&mut self, profile_name: &str) -> Task<app::Message> {
        if let Some(profile) = self.session.profiles.get_mut(profile_name) {
            let instance_name = self.app_state.instance_input.clone();
            if instance_name.is_empty() {
                return Task::none();
            }

            let base_files = ignore::WalkBuilder::new(&profile.path)
                .ignore(false)
                .build()
                .filter_map(Result::ok)
                .map(|entry| {
                    (
                        entry.path().to_path_buf(),
                        profile::FileInfo::default()
                            .with_source_path(entry.path())
                            .with_enabled(true),
                    )
                })
                .collect::<lookup::Lookup<path::PathBuf, profile::FileInfo>>();

            let new_instance = core::profile::Instance::default()
                .with_name(&instance_name)
                .with_files(Some(base_files));

            let instances = profile.instances.get_or_insert_with(Default::default);
            if instances.contains_key(&instance_name) {
                return Task::none();
            }

            instances.insert(instance_name.clone(), new_instance);
            self.app_state.instance_choices = iced::widget::combo_box::State::new(instances.keys().cloned().collect());
            self.app_state.instance_input = String::new();

            return self.switch_instance(&instance_name);
        }

        Task::none()
    }

    pub fn remove_instance_from_profile(&mut self, profile_name: &str) {
        if let Some(profile) = self.session.profiles.get_mut(profile_name)
            && let Some(selected_instance_name) = self.session.active_instance.clone()
            && let Some(instances) = profile.instances.as_mut()
        {
            instances.remove(&selected_instance_name);
            self.app_state.instance_choices = iced::widget::combo_box::State::new(instances.keys().cloned().collect());
            self.session.active_instance = None;
            self.app_state.instance_input = String::new();
            if instances.is_empty() {
                profile.instances = None;
            }
        }
    }

    pub fn switch_instance(&mut self, instance_name: &str) -> Task<app::Message> {
        if let Err(err) = self.update_instance_from_cache() {
            log::error!("Failed to update instance from cache: {err}");
        }
        self.session.active_instance = Some(instance_name.to_owned());
        Task::done(app::Message::CurrentDirectoryUpdated)
    }

    pub fn set_game_dir(&mut self, profile_name: Option<String>, path: Option<path::PathBuf>) -> Task<app::Message> {
        if let Some(profile_name) = profile_name.or_else(|| self.session.active_profile.clone())
            && let Some(path) = path.or_else(|| {
                rfd::FileDialog::new()
                    .set_title(format!("Select {} directory", &profile_name))
                    .pick_folder()
            })
            && path.is_dir()
            && let Some(profile) = self.session.profiles.get_mut(&profile_name)
        {
            log::trace!("Setting {} directory to {}", profile_name, path.display());
            profile.path = path.clone();
            self.app_state.current_directory = path.clone();

            self.session.files.extend(
                ignore::WalkBuilder::new(path)
                    .ignore(false)
                    .build()
                    .filter_map(Result::ok)
                    .map(|entry| {
                        (
                            entry.path().to_path_buf(),
                            core::profile::FileInfo::default()
                                .with_source_path(entry.path())
                                .with_enabled(true),
                        )
                    }),
            );

            return Task::done(app::Message::CurrentDirectoryUpdated);
        }

        Task::none()
    }

    pub fn set_mods_dir(&mut self, profile_name: Option<String>, path: Option<path::PathBuf>) -> Task<app::Message> {
        if let Some(profile_name) = profile_name.or_else(|| self.session.active_profile.clone())
            && let Some(path) = path.or_else(|| {
                rfd::FileDialog::new()
                    .set_title(format!("Select {} directory", &profile_name))
                    .pick_folder()
            })
            && path.is_dir()
            && self.session.profiles.get(&profile_name).is_some()
        {
            self.session.mod_storage_dir = Some(path.clone());

            if let Err(err) = std::fs::create_dir_all(&path) {
                return Task::done(app::Message::ErrorReturned(error::SharedError::new(err)));
            } else {
                return Task::done(app::Message::CurrentDirectoryUpdated);
            }
        }

        Task::none()
    }
}
