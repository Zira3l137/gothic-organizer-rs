use std::path;

use iced::Task;

use crate::app::message;
use crate::app::session;
use crate::app::state;
use crate::core;
use crate::core::lookup;
use crate::core::profile;
use crate::core::services::Service;
use crate::error;

pub struct ProfileService<'a> {
    session: &'a mut session::ApplicationSession,
    state: &'a mut state::ApplicationState,
}

crate::impl_service!(ProfileService);

impl<'a> ProfileService<'a> {
    pub fn new(
        session: &'a mut session::ApplicationSession,
        app_state: &'a mut state::ApplicationState,
    ) -> Self {
        Self { session, state: app_state }
    }

    pub fn switch_profile(&mut self, profile_name: &str) -> Task<message::Message> {
        if let Err(err) = self.update_instance_from_cache() {
            tracing::error!("Failed to update instance from cache: {err}");
        }

        if let Some(next_profile) = self.state.profile.profiles.get(profile_name) {
            self.session.active_profile = Some(profile_name.to_owned());
            self.session.active_instance = None;

            let instances =
                next_profile.instances.as_ref().map(|i| i.keys().cloned().collect()).unwrap_or_default();

            self.state.profile.instance_choices = iced::widget::combo_box::State::new(instances);

            if !next_profile.path.as_os_str().is_empty() {
                return Task::done(message::UiMessage::ReloadDirEntries.into());
            }
        }

        Task::none()
    }

    pub fn update_instance_from_cache(&mut self) -> Result<(), error::Error> {
        self.session.files.extend(self.state.ui.dir_entries.iter().cloned());

        let cached_files = self.session.files.clone();
        let mut context = self.context()?;

        context.set_instance_files(Some(cached_files));
        Ok(())
    }

    pub fn add_instance_for_profile(&mut self, profile_name: &str) -> Task<message::Message> {
        if let Some(profile) = self.state.profile.profiles.get_mut(profile_name) {
            let instance_name = self.state.profile.instance_name_field.clone();
            if instance_name.is_empty() {
                return Task::none();
            }

            let base_files = ignore::WalkBuilder::new(&profile.path)
                .ignore(false)
                .build()
                .filter_map(|e| {
                    let entry = e.clone().ok();
                    if let Some(entry) = entry
                        && entry.path() == profile.path
                    {
                        return None;
                    };
                    e.ok()
                })
                .map(|entry| {
                    (
                        entry.path().to_path_buf(),
                        profile::FileMetadata::default().with_source_path(entry.path()).with_enabled(true),
                    )
                })
                .collect::<lookup::Lookup<path::PathBuf, profile::FileMetadata>>();

            let new_instance =
                core::profile::Instance::default().with_name(&instance_name).with_files(Some(base_files));

            let instances = profile.instances.get_or_insert_with(Default::default);
            if instances.contains_key(&instance_name) {
                return Task::none();
            }

            instances.insert(instance_name.clone(), new_instance);
            self.state.profile.instance_choices =
                iced::widget::combo_box::State::new(instances.keys().cloned().collect());
            self.state.profile.instance_name_field = String::new();

            return self.switch_instance(&instance_name);
        }

        Task::none()
    }

    pub fn remove_instance_from_profile(&mut self) {
        let Ok(mut context) = self.context() else {
            tracing::error!("Failed to get context");
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
            tracing::error!("Failed to get instances");
            return;
        }

        self.state.profile.instance_choices = iced::widget::combo_box::State::new(instance_names);
        self.state.profile.instance_name_field = String::new();
        self.session.active_instance = None;
    }

    pub fn switch_instance(&mut self, instance_name: &str) -> Task<message::Message> {
        if let Err(err) = self.update_instance_from_cache() {
            tracing::error!("Failed to update instance from cache: {err}");
        }
        self.session.active_instance = Some(instance_name.to_owned());
        Task::done(message::UiMessage::ReloadDirEntries.into())
    }

    pub fn set_game_dir(&mut self, path: Option<path::PathBuf>) -> Task<message::Message> {
        let Ok(mut context) = self.context() else {
            tracing::error!("Failed to get context");
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
        context.set_instance_files(None);
        self.state.ui.current_dir = path.clone();

        self.session.files.clear();
        self.session.files.extend(
            ignore::WalkBuilder::new(path).ignore(false).build().filter_map(Result::ok).map(|entry| {
                (
                    entry.path().to_path_buf(),
                    core::profile::FileMetadata::default().with_source_path(entry.path()).with_enabled(true),
                )
            }),
        );

        if let Err(err) = self.update_instance_from_cache() {
            tracing::error!("Failed to update instance from cache: {err}");
        }

        Task::batch(vec![
            Task::done(message::ModMessage::Reload.into()),
            Task::done(message::UiMessage::ReloadDirEntries.into()),
        ])
    }

    pub fn set_mods_dir(&mut self, path: Option<path::PathBuf>) -> Task<message::Message> {
        let Ok(context) = self.context() else {
            tracing::error!("Failed to get context");
            return Task::none();
        };

        let Some(path) = path.or_else(|| {
            rfd::FileDialog::new()
                .set_title(format!("Select mod storage directory for {}", &context.active_profile.name))
                .pick_folder()
        }) else {
            tracing::error!("Failed to get mod storage path");
            return Task::none();
        };

        self.session.mod_storage_dir = Some(path.clone());

        if let Err(err) = std::fs::create_dir_all(&path) {
            tracing::error!("Failed to create mod storage directory: {err}");
            Task::none()
        } else {
            Task::done(message::UiMessage::ReloadDirEntries.into())
        }
    }
}
