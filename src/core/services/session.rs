use iced::Task;

use crate::app::message;
use crate::app::state;
use crate::core::profile::Lookup;
use crate::error;
use crate::save_app_session;
use crate::save_profile;

pub struct SessionService<'a> {
    session: &'a mut crate::app::session::ApplicationSession,
    state: &'a mut crate::app::state::ApplicationState,
}

impl<'a> SessionService<'a> {
    pub fn new(
        session: &'a mut crate::app::session::ApplicationSession,
        state: &'a mut crate::app::state::ApplicationState,
    ) -> Self {
        Self { session, state }
    }

    /// Attempts to serialize and save all profile states and the current application session.
    ///
    /// Emits error messages for each failed save operation.
    pub fn save_current_session(&self) -> Task<message::Message> {
        let mut profile_saving_error_tasks: Vec<Task<message::Message>> = vec![Task::none()];
        let mut session_save_error_task: Task<message::Message> = Task::none();

        self.state.profile.profiles.values().for_each(|p| {
            tracing::info!("Saving profile: {}", p.name);
            if let Err(e) = save_profile!(p) {
                tracing::error!("Failed saving {} profile: {e}", p.name);
                profile_saving_error_tasks.push(Task::done(
                    message::ErrorMessage::Handle(error::ErrorContext::from(error::Error::from(e))).into(),
                ));
            }
        });

        tracing::info!("Saving session");
        if let Err(e) = save_app_session!(self.session) {
            tracing::error!("Failed saving session: {e}");
            session_save_error_task = Task::done(
                message::ErrorMessage::Handle(error::ErrorContext::from(error::Error::from(e))).into(),
            );
        }

        Task::batch(profile_saving_error_tasks).chain(session_save_error_task)
    }

    /// Exits the application if all windows are closed or if the editor window is closed otherwise
    /// does nothing.
    pub fn exit_application(&mut self) -> Task<message::Message> {
        let windows = self
            .state
            .ui
            .windows
            .iter()
            .map(|(_, wnd_state)| (wnd_state.name.as_str(), wnd_state))
            .collect::<Lookup<_, _>>();
        if windows.iter().all(|(_, wnd_state)| wnd_state.is_closed)
            || windows.get("editor").unwrap().is_closed
        {
            tracing::info!("Exiting application");
            return self.save_current_session().chain(iced::exit());
        }

        iced::Task::none()
    }

    /// Closes the window with the given id. Exits the application if all windows are closed.
    pub fn close_window(&mut self, wnd_id: &iced::window::Id) -> Task<message::Message> {
        if let Some(wnd_state) = self.state.ui.windows.get_mut(&Some(*wnd_id)) {
            tracing::info!("Closing window: {}", wnd_state.name);
            wnd_state.is_closed = true;
        }

        iced::Task::chain(
            iced::window::close(*wnd_id),
            Task::done(message::SystemMessage::ExitApplication.into()),
        )
    }

    /// Initializes the editor window. Reloads the directory entries for UI upon completion.
    pub fn init_window(&mut self) -> Task<message::Message> {
        let wnd_size = crate::app::GothicOrganizer::WINDOW_SIZE;
        self.invoke_window("editor", None, Some(iced::Size { width: wnd_size.0, height: wnd_size.1 }))
            .chain(Task::done(message::UiMessage::ReloadDirEntries.into()))
    }

    /// Creates a new window with the given name, position and size.
    pub fn invoke_window(
        &mut self,
        name: &str,
        position: Option<iced::window::Position>,
        size: Option<iced::Size>,
    ) -> Task<message::Message> {
        let (id, task) = iced::window::open(iced::window::Settings {
            position: position.unwrap_or(iced::window::Position::Centered),
            size: size.unwrap_or(iced::Size { width: 512.0, height: 512.0 }),
            icon: iced::window::icon::from_file("./resources/icon.ico").ok(),
            exit_on_close_request: false,
            ..Default::default()
        });

        tracing::info!("Opening window: {name}");
        self.state.ui.windows.insert(Some(id), state::WindowInfo { name: name.to_owned(), is_closed: false });
        task.then(|_| Task::none())
    }
}
