use crate::app::message;
use crate::app::state;
use crate::core::constants;
use crate::core::services;

pub fn handle_profile_message(
    session: &mut services::session::SessionService,
    state: &mut state::ApplicationState,
    message: message::ProfileMessage,
) -> iced::Task<message::Message> {
    let mut service = services::profile::ProfileService::new(session, state);

    match message {
        message::ProfileMessage::SetActive(profile_name) => {
            service.switch_profile(&profile_name).map(message::Message::from)
        }

        message::ProfileMessage::SetGameDir(path) => {
            service.set_game_dir(path).map(message::Message::from)
        }

        message::ProfileMessage::AddInstance(profile_name) => {
            service.add_instance_for_profile(&profile_name).map(message::Message::from)
        }

        message::ProfileMessage::SetActiveInstance(instance_name) => {
            let mut service = services::profile::ProfileService::new(session, state);
            service.switch_instance(&instance_name)
        }

        message::ProfileMessage::RemoveActiveInstance => {
            service.remove_instance_from_profile();
            iced::Task::none()
        }

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
    session: &mut services::session::SessionService,
    state: &mut state::ApplicationState,
    message: message::ModMessage,
) -> iced::Task<message::Message> {
    match message {
        message::ModMessage::Add(path) => {
            let mut service = services::mods::ModService::new(session);
            service.add_mod(path).map(message::Message::from)
        }

        message::ModMessage::Toggle(name, new_state) => {
            let mut profile_service = services::profile::ProfileService::new(session, state);
            if let Err(err) = profile_service.update_instance_from_cache() {
                log::warn!("Couldn't update instance cache: {err}");
            }

            let mut service = services::mods::ModService::new(session);
            service.toggle_mod(name, new_state);
            iced::Task::done(message::Message::UI(message::UiMessage::ReloadDirEntries))
        }

        message::ModMessage::Uninstall(name) => {
            let mut service = services::mods::ModService::new(session);
            service.remove_mod(name).map(message::Message::from)
        }

        message::ModMessage::Reload => {
            let mut service = services::mods::ModService::new(session);
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
    session: &mut services::session::SessionService,
    state: &mut state::ApplicationState,
    message: message::UiMessage,
) -> iced::Task<message::Message> {
    match message {
        message::UiMessage::UpdateActiveDir(path) => {
            let mut profile_service = services::profile::ProfileService::new(session, state);
            if let Err(err) = profile_service.update_instance_from_cache() {
                log::warn!("Couldn't update instance cache: {err}");
            }

            state.ui.current_dir = path.clone();
            let mut ui_service = services::ui::UiService::new(session, state);
            ui_service.reload_displayed_directory(Some(path));
            iced::Task::none()
        }

        message::UiMessage::ToggleFileEntry(path) => {
            let mut service = services::ui::UiService::new(session, state);
            service.toggle_state_recursive(Some(&path));
            iced::Task::none()
        }

        message::UiMessage::ToggleAllFileEntries => {
            let mut service = services::ui::UiService::new(session, state);
            service.toggle_state_recursive(None);
            iced::Task::none()
        }

        message::UiMessage::ReloadDirEntries => {
            let mut service = services::ui::UiService::new(session, state);
            service.reload_displayed_directory(None);
            iced::Task::none()
        }

        message::UiMessage::SetTheme(theme) => {
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
    session: &mut services::session::SessionService,
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
            session.launch_options.get_or_insert_default().game_settings.zspy.verbosity =
                level.into();
            iced::Task::none()
        }

        message::SettingsMessage::ToggleMarvinMode(new_state) => {
            session.launch_options.get_or_insert_default().game_settings.is_marvin_mode_enabled =
                new_state;
            iced::Task::none()
        }

        message::SettingsMessage::ToggleParserSetting(option, new_state) => {
            session.toggle_launch_option(&option, new_state);
            iced::Task::none()
        }

        message::SettingsMessage::ToggleZSpyState(new_state) => {
            let config = session.active_zspy_config.get_or_insert_default();
            config.is_enabled = new_state;
            session.launch_options.get_or_insert_default().game_settings.zspy.is_enabled =
                new_state;
            iced::Task::none()
        }
    }
}

pub fn handle_window_message(
    session: &mut services::session::SessionService,
    _state: &mut state::ApplicationState,
    message: message::WindowMessage,
) -> iced::Task<message::Message> {
    match message {
        message::WindowMessage::Close(wnd_id) => {
            session.close_window(&wnd_id).map(message::Message::from)
        }

        message::WindowMessage::Open(name) => match name.as_str() {
            "options" => session.invoke_options_window().map(message::Message::from),
            "overwrites" => session.invoke_overwrites_window().map(message::Message::from),
            _ => iced::Task::none(),
        },

        message::WindowMessage::Initialize => session.init_window().map(message::Message::from),
    }
}

pub fn handle_system_message(
    session: &mut services::session::SessionService,
    state: &mut state::ApplicationState,
    message: message::SystemMessage,
) -> iced::Task<message::Message> {
    match message {
        message::SystemMessage::OpenRepository => {
            if let Err(err) = services::browser_open(constants::APP_REPOSITORY) {
                log::error!("Error opening repository: {err}");
            }
            iced::Task::none()
        }

        message::SystemMessage::ExitApplication => {
            if session.windows.iter().all(|(_, wnd_state)| wnd_state.is_closed) {
                let mut profile_service = services::profile::ProfileService::new(session, state);
                if let Err(err) = profile_service.update_instance_from_cache() {
                    log::warn!("Couldn't update instance cache: {err}");
                }
                session.save_current_session();
                iced::exit()
            } else {
                iced::Task::none()
            }
        }

        message::SystemMessage::Idle => iced::Task::none(),
    }
}

pub fn handle_error_message(
    _session: &mut services::session::SessionService,
    state: &mut state::ApplicationState,
    message: message::ErrorMessage,
) -> iced::Task<message::Message> {
    match message {
        message::ErrorMessage::Handle(error_ctx) => {
            log::error!("{}", error_ctx.error);
            iced::Task::none()
        }
        message::ErrorMessage::Dismiss(error_id) => {
            state.errors.dismiss_error(error_id);
            iced::Task::none()
        }
        message::ErrorMessage::ShowDetails(_) => {
            // Could open a detailed error dialog
            iced::Task::none()
        }
        message::ErrorMessage::ClearAll => {
            state.errors.clear_all();
            iced::Task::none()
        }
    }
}
