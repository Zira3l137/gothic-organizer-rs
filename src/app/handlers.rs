use crate::app::message;
use crate::app::session;
use crate::app::state;
use crate::core::constants;
use crate::core::services;
use crate::lookup;

pub fn handle_profile_message(
    session: &mut session::ApplicationSession,
    state: &mut state::ApplicationState,
    message: message::ProfileMessage,
) -> iced::Task<message::Message> {
    let mut service = services::profile::ProfileService::new(session, state);

    match message {
        message::ProfileMessage::SetActive(profile_name) => {
            service.switch_profile(&profile_name).map(message::Message::from)
        }

        message::ProfileMessage::SetGameDir(path) => service.set_game_dir(path).map(message::Message::from),

        message::ProfileMessage::AddInstance => {
            service.add_instance_for_profile().map(message::Message::from)
        }

        message::ProfileMessage::SetActiveInstance(instance_name) => {
            let mut service = services::profile::ProfileService::new(session, state);
            service.switch_instance(&instance_name)
        }

        message::ProfileMessage::RemoveActiveInstance => service.remove_instance_from_profile(),

        message::ProfileMessage::UpdateInstanceNameField(input) => {
            state.profile.instance_name_field = input;
            iced::Task::none()
        }

        message::ProfileMessage::UpdateProfileDirField(input) => {
            state.profile.profile_dir_field = input;
            iced::Task::none()
        }
    }
}

pub fn handle_mod_message(
    session: &mut session::ApplicationSession,
    state: &mut state::ApplicationState,
    message: message::ModMessage,
) -> iced::Task<message::Message> {
    match message {
        message::ModMessage::Add(path) => {
            let mut service = services::mods::ModService::new(session, state);
            service.add_mod(path).map(message::Message::from)
        }

        message::ModMessage::Toggle(name, new_state) => {
            let mut profile_service = services::profile::ProfileService::new(session, state);
            if let Err(err) = profile_service.commit_session_files() {
                tracing::warn!("Couldn't update instance cache: {err}");
            }

            let mut service = services::mods::ModService::new(session, state);
            service.toggle_mod(name, new_state)
        }

        message::ModMessage::Uninstall(name) => {
            let mut service = services::mods::ModService::new(session, state);
            service.remove_mod(name).map(message::Message::from)
        }

        message::ModMessage::Reload => {
            let mut service = services::mods::ModService::new(session, state);
            service.reload_mods().map(message::Message::from)
        }

        message::ModMessage::SetModsDir(path) => {
            let mut service = services::profile::ProfileService::new(session, state);
            service.set_mods_dir(path).map(message::Message::from)
        }

        message::ModMessage::UpdateModsDirField(input) => {
            state.mod_management.mods_dir_field = input;
            iced::Task::none()
        }
    }
}

pub fn handle_ui_message(
    session: &mut session::ApplicationSession,
    state: &mut state::ApplicationState,
    message: message::UiMessage,
) -> iced::Task<message::Message> {
    match message {
        message::UiMessage::UpdateActiveDir(path) => {
            let mut profile_service = services::profile::ProfileService::new(session, state);
            if let Err(err) = profile_service.commit_session_files() {
                tracing::warn!("Couldn't update instance cache: {err}");
            }

            state.ui.current_dir = path.clone();
            let mut ui_service = services::ui::UiService::new(session, state);
            ui_service.reload_displayed_directory(Some(path))
        }

        message::UiMessage::ToggleFileEntry(entry_sate, path) => {
            let mut service = services::ui::UiService::new(session, state);
            service.set_entry_state_with_children(Some(entry_sate), Some(&path));
            iced::Task::none()
        }

        message::UiMessage::ToggleAllFileEntries => {
            let mut service = services::ui::UiService::new(session, state);
            service.set_entry_state_with_children(None, None);
            iced::Task::none()
        }

        message::UiMessage::ReloadDirEntries => {
            let mut service = services::ui::UiService::new(session, state);
            service.reload_displayed_directory(None)
        }

        message::UiMessage::SetTheme(theme) => {
            tracing::info!("Setting theme to {theme}");
            session.theme_selected = Some(theme);
            iced::Task::none()
        }

        message::UiMessage::SetOptionsMenu(menu) => {
            state.ui.active_options_menu = menu;
            iced::Task::none()
        }
    }
}

pub fn handle_settings_message(
    session: &mut session::ApplicationSession,
    state: &mut state::ApplicationState,
    message: message::SettingsMessage,
) -> iced::Task<message::Message> {
    match message {
        message::SettingsMessage::SetRendererBackend(backend) => {
            session.active_renderer_backend = Some(backend.clone());
            session.launch_options.get_or_insert_default().game_settings.renderer = backend;
            iced::Task::none()
        }

        message::SettingsMessage::UpdateZspyLevel(level) => {
            state.settings.zspy_level_field = level;
            session.active_zspy_config.get_or_insert_default().verbosity = level.into();
            session.launch_options.get_or_insert_default().game_settings.zspy.verbosity = level.into();
            iced::Task::none()
        }

        message::SettingsMessage::ToggleMarvinMode(new_state) => {
            session.launch_options.get_or_insert_default().game_settings.is_marvin_mode_enabled = new_state;
            iced::Task::none()
        }

        message::SettingsMessage::ToggleParserSetting(option, new_state) => {
            if let Some(options) = session.launch_options.as_mut() {
                options.parser_settings.commands.insert(option.clone(), new_state);
            } else {
                session.launch_options = Some(session::GameLaunchConfiguration {
                    parser_settings: session::ParserSettings {
                        commands: lookup![(option.clone() => new_state)],
                    },
                    ..Default::default()
                });
            }
            iced::Task::none()
        }

        message::SettingsMessage::ToggleZSpyState(new_state) => {
            let config = session.active_zspy_config.get_or_insert_default();
            config.is_enabled = new_state;
            session.launch_options.get_or_insert_default().game_settings.zspy.is_enabled = new_state;
            iced::Task::none()
        }

        message::SettingsMessage::ToggleErrorNotifications(new_state) => {
            session.error_notifications_enabled = new_state;
            iced::Task::none()
        }
    }
}

pub fn handle_window_message(
    session: &mut session::ApplicationSession,
    state: &mut state::ApplicationState,
    message: message::WindowMessage,
) -> iced::Task<message::Message> {
    match message {
        message::WindowMessage::Close(wnd_id) => {
            let mut session_service = services::session::SessionService::new(session, state);
            session_service.close_window(&wnd_id).map(message::Message::from)
        }

        message::WindowMessage::Open(name) => {
            let open_windows = state
                .ui
                .windows
                .iter()
                .filter_map(|(id, info)| (!info.is_closed).then_some((info.name.clone(), id.unwrap())))
                .collect::<crate::core::lookup::Lookup<_, _>>();

            let mut session_service = services::session::SessionService::new(session, state);

            if let Some(open_window_id) = open_windows.get(&name) {
                return iced::Task::done(message::WindowMessage::Close(*open_window_id).into());
            }

            match name.as_str() {
                "options" => session_service
                    .invoke_window("options", None, Some(iced::Size { width: 768.0, height: 460.0 }))
                    .map(message::Message::from),
                "overwrites" => session_service
                    .invoke_window("overwrites", None, Some(iced::Size { width: 460.0, height: 768.0 }))
                    .map(message::Message::from),
                "logs" => session_service
                    .invoke_window("logs", None, Some(iced::Size { width: 768.0, height: 460.0 }))
                    .map(message::Message::from),
                _ => iced::Task::none(),
            }
        }

        message::WindowMessage::Initialize => {
            let mut session_service = services::session::SessionService::new(session, state);
            session_service.init_window().map(message::Message::from)
        }
    }
}

pub fn handle_system_message(
    session: &mut session::ApplicationSession,
    state: &mut state::ApplicationState,
    message: message::SystemMessage,
) -> iced::Task<message::Message> {
    match message {
        message::SystemMessage::OpenRepository => {
            if let Err(err) = services::browser_open(constants::APP_REPOSITORY) {
                tracing::error!("Error opening repository: {err}");
            }
            iced::Task::none()
        }

        message::SystemMessage::ExitApplication => {
            let mut error_task: iced::Task<message::Message> = iced::Task::none();
            let mut profile_service = services::profile::ProfileService::new(session, state);
            if let Err(err) = profile_service.commit_session_files() {
                tracing::error!("Failed to commit session files: {err}");
                error_task = iced::Task::done(
                    message::ErrorMessage::Handle(
                        crate::error::ErrorContext::builder()
                            .error(err)
                            .suggested_action("Make sure the active profile is selected and valid")
                            .build(),
                    )
                    .into(),
                );
            }

            let mut session_service = services::session::SessionService::new(session, state);
            error_task.chain(session_service.exit_application())
        }

        message::SystemMessage::Idle => iced::Task::none(),
    }
}

pub fn handle_error_message(
    _session: &mut session::ApplicationSession,
    state: &mut state::ApplicationState,
    message: message::ErrorMessage,
) -> iced::Task<message::Message> {
    match message {
        message::ErrorMessage::Handle(error_ctx) => {
            state.errors.add_error(error_ctx);
            iced::Task::none()
        }

        message::ErrorMessage::Dismiss(error_id) => {
            state.errors.dismiss_error(error_id);
            iced::Task::none()
        }

        message::ErrorMessage::ShowDetails(_error_id) => {
            // TODO:  Could open a detailed error dialog
            iced::Task::none()
        }

        message::ErrorMessage::ClearAll => {
            state.errors.clear_all();
            iced::Task::none()
        }
    }
}
