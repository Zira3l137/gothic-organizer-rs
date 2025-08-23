use std::path::PathBuf;

use iced::widget::combo_box;

use crate::config;
use crate::core::constants;
use crate::core::lookup;
use crate::core::profile;
use crate::core::services;
use crate::error;
use crate::gui::options;

#[derive(Debug, Default)]
pub struct GothicOrganizer {
    pub session: services::session::SessionService,
    pub state: ApplicationState,
}

#[derive(Debug, Default)]
pub struct ApplicationState {
    pub current_dir: PathBuf,
    pub instance_name_field: String,
    pub profile_dir_field: String,
    pub mods_dir_field: String,
    pub zspy_level_field: u8,
    pub active_options_menu: options::menu::OptionsMenu,
    pub profile_choices: combo_box::State<String>,
    pub theme_choices: combo_box::State<String>,
    pub instance_choices: combo_box::State<String>,
    pub renderer_choices: combo_box::State<config::RendererBackend>,
    pub themes: lookup::Lookup<String, iced::Theme>,
    pub dir_entries: Vec<(PathBuf, profile::FileMetadata)>,
}

#[derive(Debug, Default)]
pub struct WindowState {
    pub wnd_name: String,
    pub is_closed: bool,
}

impl GothicOrganizer {
    pub const WINDOW_TITLE: &str = "Gothic Organizer";
    pub const WINDOW_SIZE: (f32, f32) = (768.0, 768.0);

    pub fn new() -> (Self, iced::Task<Message>) {
        let mut app =
            Self { session: services::session::SessionService::new(), ..Default::default() };

        app.state.profile_choices =
            combo_box::State::new(app.session.profile_names.clone().unwrap_or_default());

        app.state.instance_choices =
            combo_box::State::new(app.session.instance_names.clone().unwrap_or_default());

        app.state.themes = services::session::SessionService::load_default_themes();

        app.state.theme_choices =
            combo_box::State::new(app.state.themes.iter().map(|(_, t)| t.to_string()).collect());

        app.state.renderer_choices = combo_box::State::new(
            config::RendererBackend::into_iter().cloned().collect::<Vec<_>>(),
        );

        app.state.zspy_level_field =
            app.session.active_zspy_config.get_or_insert_default().verbosity.into();

        (app, iced::Task::done(Message::RequestInitializeMainWindow))
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match &message {
            Message::RequestInitializeMainWindow => {
                return self.session.init_window();
            }

            Message::RequestDirEntriesReload => {
                let mut service = services::ui::UiService::new(&mut self.session, &mut self.state);
                service.reload_displayed_directory(None);
            }

            Message::RequestModsReload => {
                let mut service = services::mods::ModService::new(&mut self.session);
                return service.reload_mods();
            }

            Message::RequestModUninstall(name) => {
                let mut service = services::mods::ModService::new(&mut self.session);
                return service.remove_mod(name.clone());
            }

            Message::RequestOpenRepository => {
                if let Err(err) = services::browser_open(constants::APP_REPOSITORY) {
                    log::error!("Error opening repository: {err}");
                }
            }

            Message::RequestWindowClose(wnd_id) => {
                return self.session.close_window(wnd_id);
            }

            Message::RequestExitApplication => {
                if self.session.windows.iter().all(|(_, wnd_state)| wnd_state.is_closed) {
                    let mut profile_service =
                        services::profile::ProfileService::new(&mut self.session, &mut self.state);

                    if let Err(err) = profile_service.update_instance_from_cache() {
                        log::warn!("Couldn't update instance cache: {err}");
                    }

                    self.session.save_current_session();
                    return iced::exit();
                }
            }

            Message::RequestPanicWithErr(err) => {
                return self.session.exit_with_error(err.clone());
            }

            Message::RequestWindowOpen(name) => match name.as_str() {
                "options" => return self.session.invoke_options_window(),
                "overwrites" => return self.session.invoke_overwrites_window(),
                _ => return iced::Task::none(),
            },

            Message::RemoveActiveInstance => {
                let mut service =
                    services::profile::ProfileService::new(&mut self.session, &mut self.state);
                service.remove_instance_from_profile();
            }

            Message::SetUiTheme(theme) => {
                self.session.theme_selected = Some(theme.clone());
            }

            Message::SetActiveProfile(profile_name) => {
                let mut service =
                    services::profile::ProfileService::new(&mut self.session, &mut self.state);
                return service.switch_profile(profile_name);
            }

            Message::SetGameDir(path) => {
                let mut service =
                    services::profile::ProfileService::new(&mut self.session, &mut self.state);
                return service.set_game_dir(path.clone());
            }

            Message::SetModsDir(path) => {
                let mut service =
                    services::profile::ProfileService::new(&mut self.session, &mut self.state);
                return service.set_mods_dir(path.clone());
            }

            Message::SetActiveInstance(instance) => {
                let mut service =
                    services::profile::ProfileService::new(&mut self.session, &mut self.state);
                return service.switch_instance(instance);
            }

            Message::SetOptionsMenu(menu) => {
                self.state.active_options_menu = *menu;
            }

            Message::SetRendererBackend(backend) => {
                self.session.active_renderer_backend = Some(backend.clone());
                self.session.launch_options.get_or_insert_default().game_settings.renderer =
                    backend.clone();
            }

            Message::UpdateInstanceNameField(input) => {
                self.state.instance_name_field = input.clone();
            }

            Message::UpdateProfileDirField(input) => {
                self.state.profile_dir_field = input.clone();
            }

            Message::UpdateActiveUiDir(path) => {
                let mut profile_service =
                    services::profile::ProfileService::new(&mut self.session, &mut self.state);

                if let Err(err) = profile_service.update_instance_from_cache() {
                    log::warn!("Couldn't update instance cache: {err}");
                }
                self.state.current_dir = path.clone();
                let mut ui_service =
                    services::ui::UiService::new(&mut self.session, &mut self.state);
                ui_service.reload_displayed_directory(Some(path.clone()))
            }

            Message::UpdateModsDirField(input) => {
                self.state.mods_dir_field = input.clone();
            }

            Message::UpdateZspyLevelField(level) => {
                self.state.zspy_level_field = *level;
                self.session.active_zspy_config.get_or_insert_default().verbosity = (*level).into();
                self.session.launch_options.get_or_insert_default().game_settings.zspy.verbosity =
                    (*level).into();
            }

            Message::ToggleFileEntry(path) => {
                let mut service = services::ui::UiService::new(&mut self.session, &mut self.state);
                service.toggle_state_recursive(Some(path));
            }

            Message::ToggleAllFileEntries => {
                let mut service = services::ui::UiService::new(&mut self.session, &mut self.state);
                service.toggle_state_recursive(None);
            }

            Message::ToggleMod(name, new_state) => {
                let mut profile_service =
                    services::profile::ProfileService::new(&mut self.session, &mut self.state);

                if let Err(err) = profile_service.update_instance_from_cache() {
                    log::warn!("Couldn't update instance cache: {err}");
                }
                let mut service = services::mods::ModService::new(&mut self.session);
                service.toggle_mod(name.clone(), *new_state);
                return iced::Task::done(Message::RequestDirEntriesReload);
            }

            Message::ToggleMarvinMode(new_state) => {
                self.session
                    .launch_options
                    .get_or_insert_default()
                    .game_settings
                    .is_marvin_mode_enabled = *new_state;
            }

            Message::ToggleParserSetting(option, new_state) => {
                self.session.toggle_launch_option(option, *new_state);
            }

            Message::ToggleZSpyState(new_state) => {
                self.session.active_zspy_config.get_or_insert_default().is_enabled = *new_state;
                self.session.launch_options.get_or_insert_default().game_settings.zspy.is_enabled =
                    *new_state;
            }

            Message::AddMod(path) => {
                let mut service = services::mods::ModService::new(&mut self.session);
                return service.add_mod(path.clone());
            }

            Message::AddNewInstance(profile_name) => {
                let mut service =
                    services::profile::ProfileService::new(&mut self.session, &mut self.state);
                return service.add_instance_for_profile(profile_name);
            }

            Message::Idle => {
                return iced::Task::none();
            }
        }

        iced::Task::none()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen_with(|event, _, id| match event {
            iced::Event::Window(iced::window::Event::CloseRequested) => {
                Some(Message::RequestWindowClose(id))
            }
            iced::Event::Window(iced::window::Event::FileDropped(path)) => {
                Some(Message::AddMod(Some(path)))
            }
            _ => None,
        })
    }

    pub fn theme(&self) -> iced::Theme {
        match &self.session.theme_selected {
            Some(theme) => self.state.themes.get(theme).cloned().unwrap_or_else(|| {
                log::warn!("Theme {theme} not found, defaulting to dark");
                iced::Theme::Dark
            }),
            None => iced::Theme::Dark,
        }
    }

    pub fn view(&self, id: iced::window::Id) -> iced::Element<Message> {
        if let Some((_, wnd_state)) =
            self.session.windows.iter().find(|(wnd_id, _)| **wnd_id == Some(id))
        {
            match wnd_state.wnd_name.as_str() {
                "options" => crate::gui::options::options_view(self),
                "overwrites" => crate::gui::overwrites::overwrites_view(self),
                _ => crate::gui::editor::editor_view(self),
            }
        } else {
            iced::widget::container(iced::widget::text("no window")).into()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AddNewInstance(String),
    AddMod(Option<PathBuf>),
    SetModsDir(Option<PathBuf>),
    SetGameDir(Option<PathBuf>),
    SetActiveProfile(String),
    SetActiveInstance(String),
    SetOptionsMenu(options::menu::OptionsMenu),
    SetRendererBackend(config::RendererBackend),
    SetUiTheme(String),
    UpdateInstanceNameField(String),
    UpdateProfileDirField(String),
    UpdateModsDirField(String),
    UpdateActiveUiDir(PathBuf),
    UpdateZspyLevelField(u8),
    ToggleFileEntry(PathBuf),
    ToggleParserSetting(config::ParserCommand, bool),
    ToggleMarvinMode(bool),
    ToggleZSpyState(bool),
    ToggleMod(String, bool),
    ToggleAllFileEntries,
    RequestWindowClose(iced::window::Id),
    RequestWindowOpen(String),
    RequestModUninstall(String),
    RequestOpenRepository,
    RequestDirEntriesReload,
    RequestModsReload,
    RequestInitializeMainWindow,
    RequestExitApplication,
    RemoveActiveInstance,
    RequestPanicWithErr(error::SharedError),
    Idle,
}
