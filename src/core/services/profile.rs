use std::path;

use iced::Task;

use crate::app::message;
use crate::app::session;
use crate::app::state;
use crate::core;
use crate::core::lookup;
use crate::core::profile;
use crate::core::services::ApplicationContext;
use crate::core::services::context;
use crate::error;

pub struct ProfileService<'a> {
    session: &'a mut session::ApplicationSession,
    state: &'a mut state::ApplicationState,
}

crate::impl_app_context!(ProfileService);

impl<'a> ProfileService<'a> {
    pub fn new(
        session: &'a mut session::ApplicationSession,
        app_state: &'a mut state::ApplicationState,
    ) -> Self {
        Self { session, state: app_state }
    }

    /// Switch profile to `profile_name` after comitting session files for the current one.
    ///
    /// Emits error message if failed to commit session files.
    pub fn switch_profile(&mut self, profile_name: &str) -> Task<message::Message> {
        let mut tasks: Vec<iced::Task<message::Message>> = Vec::new();
        if let Err(err) = self.commit_session_files() {
            tracing::error!("Failed to commit session files: {err}");
            tasks.push(Task::done(
                message::ErrorMessage::Handle(
                    error::ErrorContext::builder()
                        .error(err)
                        .suggested_action("Make sure the active profile is selected and valid")
                        .build(),
                )
                .into(),
            ));
        }

        if let Some(next_profile) = self.state.profile.profiles.get(profile_name) {
            tracing::info!("Switching to profile: {}", next_profile.name);
            self.session.active_profile = Some(profile_name.to_owned());
            self.session.active_instance = None;

            let instances =
                next_profile.instances.as_ref().map(|i| i.keys().cloned().collect()).unwrap_or_default();

            self.state.profile.instance_choices = iced::widget::combo_box::State::new(instances);

            if !next_profile.path.as_os_str().is_empty() {
                tracing::info!("Reloading UI files.");
                tasks.push(Task::done(message::UiMessage::ReloadDirEntries.into()));
            }
        }

        Task::batch(tasks)
    }

    /// Extend session with file state from UI and write to instance.
    /// Returns error if failed to get context.
    pub fn commit_session_files(&mut self) -> Result<(), error::Error> {
        if self.session.active_profile.is_none() {
            return Ok(());
        }

        tracing::info!("Updating Session with UI files.");
        self.session.files.extend(self.state.ui.dir_entries.iter().cloned());

        let cached_files = self.session.files.clone();
        let mut context = self.context()?;

        tracing::info!("Updating Instance with Session files.");
        context.set_instance_files(Some(cached_files));
        Ok(())
    }

    /// Add new instance for active profile and populate it with base files from profile path.
    /// Does nothing if instance name is empty, instance with that name already exists or profile
    /// is invalid.
    ///
    /// After adding instance, switch to it.
    ///
    /// Emits error message if failed to get context.
    pub fn add_instance_for_profile(&mut self) -> Task<message::Message> {
        let instance_name = self.state.profile.instance_name_field.clone();
        let mut context: context::Context;
        match self.context() {
            Ok(ctx) => context = ctx,
            Err(err) => {
                tracing::error!("Failed to get context: {err}");
                return Task::done(message::ErrorMessage::Handle(err.into()).into());
            }
        }

        let profile_path = context.active_profile.path.clone();
        if instance_name.is_empty() {
            tracing::warn!("Instance name is empty");
            return Task::none();
        }

        let base_files = ignore::WalkBuilder::new(&profile_path)
            .ignore(false)
            .build()
            .filter_map(|e| match e {
                Ok(e) if e.path() != profile_path => Some((
                    e.path().to_path_buf(),
                    profile::FileMetadata::default().with_source_path(e.path()).with_enabled(true),
                )),
                _ => None,
            })
            .collect::<lookup::Lookup<path::PathBuf, profile::FileMetadata>>();

        let new_instance =
            core::profile::Instance::default().with_name(&instance_name).with_files(Some(base_files));

        if context.contains_instance(&instance_name) {
            tracing::warn!("Instance name already exists");
            return Task::none();
        }

        tracing::info!("Adding instance: {instance_name}");
        context.insert_instance(new_instance);
        self.state.profile.instance_choices = iced::widget::combo_box::State::new(context.instance_names());
        self.state.profile.instance_name_field.clear();

        self.switch_instance(&instance_name)
    }

    /// Remove instance from profile if it exists, clear active instance choice and update UI. Does nothing if instance does not exist.
    ///
    /// Emits error message if failed to get context.
    pub fn remove_instance_from_profile(&mut self) -> Task<message::Message> {
        let mut context: context::Context;
        match self.context() {
            Ok(ctx) => context = ctx,
            Err(err) => {
                tracing::error!("Failed to get context: {err}");
                return Task::done(message::ErrorMessage::Handle(err.into()).into());
            }
        }

        let active_instance_name = context.active_instance_name.clone();
        tracing::info!("Removing instance: {active_instance_name}",);

        if context.remove_instance(&active_instance_name).is_none() {
            tracing::warn!("Tried to remove instance that does not exist: {active_instance_name}");
            return Task::none();
        };
        if context.instances_empty() {
            context.active_profile.instances = None;
        }

        self.state.profile.instance_choices = iced::widget::combo_box::State::new(context.instance_names());
        self.state.profile.instance_name_field.clear();
        self.session.active_instance = None;
        Task::none()
    }

    /// Switch active instance to `instance_name` after comitting session files for the current one.
    ///
    /// Emits error message if failed to commit session files.
    pub fn switch_instance(&mut self, instance_name: &str) -> Task<message::Message> {
        if let Err(err) = self.commit_session_files() {
            tracing::error!("Failed to commit session files: {err}");
            return Task::done(
                message::ErrorMessage::Handle(
                    error::ErrorContext::builder()
                        .error(err)
                        .suggested_action("Make sure the active profile is selected and valid")
                        .build(),
                )
                .into(),
            );
        }

        tracing::info!("Switching to instance: {instance_name}");
        self.session.active_instance = Some(instance_name.to_owned());
        tracing::info!("Reloading UI files.");
        Task::done(message::UiMessage::ReloadDirEntries.into())
    }

    /// Set game directory for active profile and reload mods and UI files. Does nothing if path is
    /// empty.
    ///
    /// Emits error message if failed to get context or failed to commit session files.
    pub fn set_game_dir(&mut self, path: Option<path::PathBuf>) -> Task<message::Message> {
        let mut tasks: Vec<iced::Task<message::Message>> = Vec::new();
        let mut context: context::Context;
        match self.context() {
            Ok(ctx) => context = ctx,
            Err(err) => {
                tracing::error!("Failed to get context: {err}");
                return Task::done(message::ErrorMessage::Handle(err.into()).into());
            }
        }

        let Some(path) = path.or_else(|| {
            rfd::FileDialog::new()
                .set_title(format!("Select {} directory", &context.active_profile.name))
                .pick_folder()
        }) else {
            tracing::warn!("No path selected");
            return Task::none();
        };

        tracing::info!("Setting game directory to: {}", path.display());
        context.active_profile.path = path.clone();
        context.set_instance_files(None);
        self.state.ui.current_dir = path.clone();

        self.session.files.clear();
        self.session.files.extend(ignore::WalkBuilder::new(&path).ignore(false).build().filter_map(
            |entry| match entry {
                Ok(e) if e.path() != path => Some((
                    e.path().to_path_buf(),
                    core::profile::FileMetadata::default().with_source_path(e.path()).with_enabled(true),
                )),
                _ => None,
            },
        ));

        if let Err(err) = self.commit_session_files() {
            tracing::error!("Failed to commit session files: {err}");
            tasks.push(Task::done(
                message::ErrorMessage::Handle(
                    error::ErrorContext::builder()
                        .error(err)
                        .suggested_action("Make sure the active profile is selected and valid")
                        .build(),
                )
                .into(),
            ));
        }

        tracing::info!("Reloading mods");
        tracing::info!("Reloading UI files.");
        tasks.extend(vec![
            Task::done(message::ModMessage::Reload.into()),
            Task::done(message::UiMessage::ReloadDirEntries.into()),
        ]);

        Task::batch(tasks)
    }

    /// Set mod storage directory for active profile and reload UI files. Does nothing if path is
    /// empty.
    ///
    /// Emits error message if failed to get context.
    pub fn set_mods_dir(&mut self, path: Option<path::PathBuf>) -> Task<message::Message> {
        let context = match self.context() {
            Ok(ctx) => ctx,
            Err(err) => {
                tracing::error!("Failed to get context: {err}");
                return Task::done(message::ErrorMessage::Handle(err.into()).into());
            }
        };

        let Some(path) = path.or_else(|| {
            rfd::FileDialog::new()
                .set_title(format!("Select mod storage directory for {}", &context.active_profile.name))
                .pick_folder()
        }) else {
            tracing::warn!("No path selected");
            return Task::none();
        };

        tracing::info!("Setting mod storage directory to: {}", path.display());
        self.session.mod_storage_dir = Some(path.clone());

        match std::fs::create_dir_all(&path).map_err(|err| error::ErrorContext::from(error::Error::from(err)))
        {
            Ok(_) => {
                tracing::info!("Reloading UI files.");
                Task::done(message::UiMessage::ReloadDirEntries.into())
            }
            Err(err) => {
                tracing::error!("Failed to create mod storage directory: {err}");
                Task::done(message::ErrorMessage::Handle(err).into())
            }
        }
    }
}
