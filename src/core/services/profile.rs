use std::path;
use std::path::Path;

use iced::Task;

use crate::app::message;
use crate::app::session;
use crate::app::state;
use crate::core;
use crate::core::profile;
use crate::core::profile::Lookup;
use crate::error;
use crate::error::ErrorContext;

pub struct ProfileService<'a> {
    session: &'a mut session::ApplicationSession,
    state: &'a mut state::ApplicationState,
}

impl<'a> ProfileService<'a> {
    pub fn new(
        session: &'a mut session::ApplicationSession,
        app_state: &'a mut state::ApplicationState,
    ) -> Self {
        Self { session, state: app_state }
    }

    pub fn add_instance(&mut self) -> Task<message::Message> {
        let new_instance_name = self.state.profile.instance_name_field.clone();
        match self.try_add_instance(&new_instance_name) {
            Ok(()) => self.switch_instance(&new_instance_name),
            Err(err) => Task::done(message::ErrorMessage::Handle(err).into()),
        }
    }

    pub fn remove_instance(&mut self) -> Task<message::Message> {
        match self.try_remove_instance() {
            Ok(()) => Task::none(),
            Err(err) => Task::done(message::ErrorMessage::Handle(err).into()),
        }
    }

    pub fn switch_profile(&mut self, profile_name: &str) -> Task<message::Message> {
        match self.try_switch_profile(profile_name) {
            Ok(()) => Task::done(message::UiMessage::ReloadDirEntries.into()),
            Err(err) => Task::done(message::ErrorMessage::Handle(err).into()),
        }
    }

    pub fn switch_instance(&mut self, instance_name: &str) -> Task<message::Message> {
        match self.try_switch_instance(instance_name) {
            Ok(()) => Task::done(message::UiMessage::ReloadDirEntries.into()),
            Err(err) => Task::done(message::ErrorMessage::Handle(err).into()),
        }
    }

    pub fn commit_session_files(&mut self) -> Task<message::Message> {
        match self.try_commit_changes() {
            Ok(()) => Task::none(),
            Err(err) => Task::done(message::ErrorMessage::Handle(err).into()),
        }
    }

    pub fn set_game_dir(&mut self, path: Option<path::PathBuf>) -> Task<message::Message> {
        let Some(path) =
            path.or_else(|| rfd::FileDialog::new().set_title("Select game directory").pick_folder())
        else {
            tracing::warn!("No path selected");
            return Task::none();
        };

        match self.try_set_game_dir(&path) {
            Ok(_) => Task::batch(vec![
                Task::done(message::ModMessage::Reload.into()),
                Task::done(message::UiMessage::ReloadDirEntries.into()),
                Task::done(message::ProfileMessage::UpdateProfileDirField(path.display().to_string()).into()),
            ]),
            Err(err) => Task::done(message::ErrorMessage::Handle(err).into()),
        }
    }

    pub fn set_mods_dir(&mut self, path: Option<path::PathBuf>) -> Task<message::Message> {
        let Some(path) =
            path.or_else(|| rfd::FileDialog::new().set_title("Select mod storage directory").pick_folder())
        else {
            tracing::warn!("No path selected");
            return Task::none();
        };

        match self.try_set_mods_dir(&path) {
            Ok(_) => Task::batch([
                Task::done(message::UiMessage::ReloadDirEntries.into()),
                Task::done(message::ModMessage::UpdateModsDirField(path.display().to_string()).into()),
            ]),
            Err(err) => Task::done(message::ErrorMessage::Handle(err).into()),
        }
    }

    fn try_add_instance(&mut self, instance_name: &str) -> Result<(), ErrorContext> {
        if self.session.active_profile.is_none() {
            tracing::warn!("No active profile");
            return Ok(());
        }

        let active_profile_name = &self.session.active_profile.clone().unwrap();
        let active_profile = self.state.profile.profiles.get_mut(active_profile_name).unwrap();
        let active_profile_instances = active_profile.instances.get_or_insert_default();
        let profile_path = active_profile.path.clone();

        if active_profile_instances.contains_key(instance_name) {
            tracing::warn!("Instance already exists: {instance_name}");
            return Ok(());
        }

        let new_instance = Self::create_new_instance(instance_name, &profile_path);
        tracing::info!("Adding instance: {instance_name}");
        active_profile_instances.insert(instance_name.to_owned(), new_instance);
        let instance_names = active_profile_instances.keys().cloned().collect();
        self.state.profile.instance_choices = iced::widget::combo_box::State::new(instance_names);
        self.state.profile.instance_name_field.clear();

        Ok(())
    }

    fn try_remove_instance(&mut self) -> Result<(), ErrorContext> {
        self.validate_context("Remove Instance", false)?;
        let active_profile_name = &self.session.active_profile.clone().unwrap();
        let active_profile = self.state.profile.profiles.get_mut(active_profile_name).unwrap();
        let active_profile_instances = active_profile.instances.get_or_insert_default();
        let active_instance_name = &self.session.active_instance.clone().unwrap();

        tracing::info!("Removing instance: {active_instance_name}",);
        let instance_names: Option<Vec<String>>;
        if active_profile_instances.remove(active_instance_name).is_none() {
            tracing::warn!("Tried to remove instance that does not exist: {active_instance_name}");
            return Ok(());
        };

        if active_profile_instances.is_empty() {
            active_profile.instances = None;
            instance_names = None;
        } else {
            instance_names = Some(active_profile_instances.keys().cloned().collect::<Vec<String>>());
        }

        self.state.profile.instance_choices =
            iced::widget::combo_box::State::new(instance_names.unwrap_or_default());
        self.state.profile.instance_name_field.clear();
        self.session.active_instance = None;

        Ok(())
    }

    fn try_switch_profile(&mut self, profile_name: &str) -> Result<(), ErrorContext> {
        if self.session.active_instance.is_some() {
            self.try_commit_changes()?;
        }

        if let Some(next_profile) = self.state.profile.profiles.get(profile_name) {
            tracing::info!("Switching to profile: {}", next_profile.name);
            self.session.active_profile = Some(profile_name.to_owned());
            self.session.active_instance = None;

            let instances =
                next_profile.instances.as_ref().map(|i| i.keys().cloned().collect()).unwrap_or_default();

            self.state.profile.instance_choices = iced::widget::combo_box::State::new(instances);
            if !next_profile.path.as_os_str().is_empty() {
                tracing::warn!("Active profile has no path.");
            }

            Ok(())
        } else {
            Err(ErrorContext::builder()
                .error(error::Error::new("Profile not found", "Profile Service", "Switch"))
                .suggested_action("Make sure to select a valid profile")
                .build())
        }
    }

    fn try_switch_instance(&mut self, instance_name: &str) -> Result<(), ErrorContext> {
        if self.session.active_instance.is_some() {
            self.try_commit_changes()?;
        }

        tracing::info!("Switching to instance: {instance_name}");
        self.session.active_instance = Some(instance_name.to_owned());
        Ok(())
    }

    fn try_commit_changes(&mut self) -> Result<(), ErrorContext> {
        self.validate_context("Commit Changes", false)?;
        let active_profile_name = &self.session.active_profile.clone().unwrap();
        let active_profile = self.state.profile.profiles.get_mut(active_profile_name).unwrap();
        let active_instance_name = &self.session.active_instance.clone().unwrap();
        let active_instance =
            active_profile.instances.as_mut().unwrap().get_mut(active_instance_name).unwrap();

        tracing::info!("Updating Session with UI files.");
        self.session.files.extend(self.state.ui.dir_entries.iter().cloned());
        let cached_files = self.session.files.clone();

        tracing::info!("Updating Instance with Session files.");
        active_instance.files = cached_files;
        Ok(())
    }

    fn try_set_game_dir(&mut self, path: &Path) -> Result<(), ErrorContext> {
        if self.session.active_profile.is_none() {
            tracing::warn!("No active profile");
            return Ok(());
        }

        tracing::info!("Setting game directory to: {}", path.display());
        let active_profile_name = &self.session.active_profile.clone().unwrap();
        let active_profile = self.state.profile.profiles.get_mut(active_profile_name).unwrap();

        active_profile.path = path.to_path_buf();
        self.state.ui.current_dir = path.to_path_buf();
        self.session.files.clear();

        self.session.files.extend(ignore::WalkBuilder::new(path).ignore(false).build().filter_map(|entry| {
            match entry {
                Ok(e) if e.path() != path => Some((
                    e.path().to_path_buf(),
                    core::profile::FileMetadata::default().with_source_path(e.path()).with_enabled(true),
                )),
                _ => None,
            }
        }));

        if self.session.active_instance.is_some() {
            self.try_commit_changes()?;
        }

        Ok(())
    }

    fn try_set_mods_dir(&mut self, path: &Path) -> Result<(), ErrorContext> {
        self.validate_context("Set Mods Dir", false)?;

        tracing::info!("Setting mod storage directory to: {}", path.display());
        self.session.mod_storage_dir = Some(path.to_path_buf());
        std::fs::create_dir_all(path).map_err(|err| {
            error::ErrorContext::builder()
                .error(err.into())
                .suggested_action("Check directory write permissions")
                .build()
        })?;

        Ok(())
    }

    fn create_new_instance(name: &str, base_path: &Path) -> core::profile::Instance {
        let base_files = ignore::WalkBuilder::new(base_path)
            .ignore(false)
            .build()
            .filter_map(|e| match e {
                Ok(e) if e.path() != base_path => Some((
                    e.path().to_path_buf(),
                    profile::FileMetadata::default()
                        .with_source_path(e.path())
                        .with_target_path(e.path())
                        .with_enabled(true),
                )),
                _ => None,
            })
            .collect::<Lookup<path::PathBuf, profile::FileMetadata>>();

        core::profile::Instance::default().with_name(name).with_files(base_files)
    }

    fn validate_context(&self, operation: &str, ignore_instance: bool) -> Result<(), ErrorContext> {
        if self.session.active_profile.is_none() {
            Err(ErrorContext::builder()
                .error(error::Error::new("No active profile", "Profile Service", operation))
                .suggested_action("Select a profile and try again")
                .build())
        } else if !ignore_instance && self.session.active_instance.is_none() {
            Err(ErrorContext::builder()
                .error(error::Error::new("No active instance", "Profile Service", operation))
                .suggested_action("Select an instance and try again")
                .build())
        } else {
            Ok(())
        }
    }
}
