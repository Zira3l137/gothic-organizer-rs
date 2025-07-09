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

crate::impl_service!(ProfileService);

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

        context.set_instance_files(cached_files);
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

    pub fn remove_instance_from_profile(&mut self) {
        let Ok(mut context) = self.context() else {
            log::error!("Failed to get context");
            return;
        };

        let active_instance_name = context.active_instance_name.clone();
        let mut instance_names: Vec<String> = Vec::new();

        if let Some(instances) = context.instances_mut() {
            instances.remove(&active_instance_name);
            instance_names.extend(instances.keys().cloned());
            if instances.is_empty() {
                context.active_profile.instances = None;
            }
        } else {
            log::error!("Failed to get instances");
            return;
        }

        self.app_state.instance_choices = iced::widget::combo_box::State::new(instance_names);
        self.app_state.instance_input = String::new();
        self.session.active_instance = None;
    }

    pub fn switch_instance(&mut self, instance_name: &str) -> Task<app::Message> {
        if let Err(err) = self.update_instance_from_cache() {
            log::error!("Failed to update instance from cache: {err}");
        }
        self.session.active_instance = Some(instance_name.to_owned());
        Task::done(app::Message::CurrentDirectoryUpdated)
    }

    pub fn set_game_dir(&mut self, path: Option<path::PathBuf>) -> Task<app::Message> {
        let Ok(context) = self.context() else {
            log::error!("Failed to get context");
            return Task::none();
        };

        let Some(path) = path.or_else(|| {
            rfd::FileDialog::new()
                .set_title(format!("Select {} directory", &context.active_profile.name))
                .pick_folder()
        }) else {
            return Task::none();
        };

        context.active_profile.path = path.clone();
        self.app_state.current_directory = path.clone();

        self.session.files.clear();
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

        if let Err(err) = self.update_instance_from_cache() {
            log::error!("Failed to update instance from cache: {err}");
        }

        Task::batch(vec![
            Task::done(app::Message::LoadModsRequested),
            Task::done(app::Message::CurrentDirectoryUpdated),
        ])
    }

    pub fn set_mods_dir(&mut self, path: Option<path::PathBuf>) -> Task<app::Message> {
        let Ok(context) = self.context() else {
            log::error!("Failed to get context");
            return Task::none();
        };

        let Some(path) = path.or_else(|| {
            rfd::FileDialog::new()
                .set_title(format!("Select {} directory", &context.active_profile.name))
                .pick_folder()
        }) else {
            log::error!("Failed to get mod storage path");
            return Task::none();
        };

        self.session.mod_storage_dir = Some(path.clone());

        if let Err(err) = std::fs::create_dir_all(&path) {
            Task::done(app::Message::ErrorReturned(error::SharedError::new(err)))
        } else {
            Task::done(app::Message::CurrentDirectoryUpdated)
        }
    }
}
