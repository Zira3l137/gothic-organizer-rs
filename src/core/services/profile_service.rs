use std::path;

use iced::Task;

use crate::app;
use crate::core;
use crate::error;

pub struct ProfileService<'a> {
    session: &'a mut core::services::session_service::SessionService,
    app_state: &'a mut app::InnerState,
}

impl<'a> ProfileService<'a> {
    pub fn new(session: &'a mut core::services::session_service::SessionService, app_state: &'a mut app::InnerState) -> Self {
        Self { session, app_state }
    }

    pub fn switch_profile(&mut self, profile_name: &str) -> Task<app::Message> {
        self.update_instance_from_cache();

        if let Some(next_profile) = self.session.profiles.get(profile_name) {
            log::trace!("Loading profile {profile_name}");
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

    pub fn update_instance_from_cache(&mut self) {
        if let Some(profile_name) = self.session.active_profile.clone()
            && let Some(profile) = self.session.profiles.get_mut(&profile_name)
            && let Some(instance_name) = self.session.active_instance.clone()
            && let Some(instances) = profile.instances.as_mut()
            && let Some(instance) = instances.get_mut(&instance_name)
        {
            self.session
                .files
                .extend(self.app_state.current_directory_entries.iter().cloned());
            instance.files = Some(self.session.files.clone());
        }
    }

    pub fn add_instance_for_profile(&mut self, profile_name: &str) -> Task<app::Message> {
        if let Some(profile) = self.session.profiles.get_mut(profile_name) {
            let instance_name = self.app_state.instance_input.clone();
            if instance_name.is_empty() {
                return Task::none();
            }

            let new_instance = core::profile::Instance::default().with_name(&instance_name);

            let instances = profile.instances.get_or_insert_with(Default::default);
            if instances.contains_key(&instance_name) {
                return Task::none();
            }

            instances.insert(instance_name.clone(), new_instance);
            self.app_state.instance_choices = iced::widget::combo_box::State::new(instances.keys().cloned().collect());
            self.app_state.instance_input = String::new();

            // Automatically select the new instance
            return self.select_instance(&instance_name);
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

    pub fn select_instance(&mut self, instance_name: &str) -> Task<app::Message> {
        self.session.active_instance = Some(instance_name.to_owned());
        log::trace!("Loading files for instance {instance_name}");
        Task::done(app::Message::ModsLoadingRequested)
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
            log::trace!(
                "Setting {profile_name} mods directory to {}",
                path.display()
            );
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