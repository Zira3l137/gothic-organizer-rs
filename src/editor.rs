use std::path::PathBuf;

use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::widget::button;
use iced::widget::checkbox;
use iced::widget::column;
use iced::widget::combo_box;
use iced::widget::combo_box::State;
use iced::widget::container;
use iced::widget::horizontal_space;
use iced::widget::image;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::text;
use iced::widget::Column;
use iced::widget::Container;
use iced::widget::Row;
use iced::window;
use iced::Element;
use iced::Length;
use iced::Task;

use rfd::FileDialog;

use chrono::Local;

use crate::constants::APP_TITLE;
use crate::constants::APP_VERSION;
use crate::constants::GAME_PROFILES;
use crate::cutstom_widgets::clickable_text::ClickableText;
use crate::load_profile;
use crate::load_session;
use crate::profile::Instance;
use crate::profile::Lookup;
use crate::profile::Profile;
use crate::save_profile;
use crate::save_session;
use crate::styled_container;

#[derive(Debug, Default)]
pub struct Editor {
    files_buffer: Option<Lookup<PathBuf, bool>>,
    profile_selected: Option<String>,
    instance_selected: Option<String>,
    profiles: Lookup<String, Profile>,
    state: InnerState,
}

#[derive(Debug, Default)]
pub struct InnerState {
    instance_input: Option<String>,
    profile_choices: State<String>,
    instance_choices: State<String>,
}

impl Editor {
    pub const WINDOW_TITLE: &str = "Startup Window";
    pub const WINDOW_SIZE: (f32, f32) = (768.0, 768.0);

    pub fn new() -> (Self, Task<Message>) {
        let last_session = load_session!();
        let profiles = Self::preload_profiles();
        let profile_choices = State::new(profiles.keys().cloned().collect());

        let mut instance_choices = State::new(Vec::new());
        let mut profile_selected = None;
        let mut instance_selected = None;
        let mut files_buffer = None;

        if let Some(session) = last_session {
            if let Some(selected_profile_name) = &session.selected_profile {
                if let Some(profile) = profiles.get(selected_profile_name) {
                    profile_selected = Some(selected_profile_name.clone());
                    if let Some(instances) = &profile.instances {
                        instance_choices = State::new(instances.keys().cloned().collect());
                        if let Some(selected_instance_name) = &session.selected_instance {
                            if let Some(instance) = instances.get(selected_instance_name) {
                                instance_selected = Some(selected_instance_name.clone());
                                files_buffer = instance.files.clone();
                            }
                        }
                    }
                }
            }
        }

        let app = Self {
            profiles,
            profile_selected,
            instance_selected,
            files_buffer,
            state: InnerState {
                profile_choices,
                instance_choices,
                ..Default::default()
            },
        };

        (app, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match &message {
            Message::ProfileSelected(profile_name) => {
                return self.switch_profile(profile_name);
            }

            Message::InstanceSelected(instance) => {
                return self.select_instance(instance);
            }

            Message::InstanceInput(input) => {
                self.state.instance_input = Some(input.clone());
            }

            Message::InstanceAdd(profile_name) => {
                return self.add_instance_for_profile(profile_name);
            }

            Message::InstanceRemove(profile_name) => {
                return self.remove_instance_from_profile(profile_name);
            }

            Message::FileToggle(path, new_state) => {
                let Some(files) = self.files_buffer.as_mut() else {
                    return Task::none();
                };

                if let Some(state) = files.get_mut(path) {
                    *state = *new_state;
                }
            }

            Message::BrowseGameDir(profile_name) => {
                return self.browse_game_dir(profile_name);
            }

            Message::TraverseIntoDir(path) => {
                println!("traverse into dir: {}", path.display());
            }

            Message::RefreshFiles => {
                return self.refresh_files();
            }

            Message::ExtendFilesFromInstance => {
                return self.extend_from_instance();
            }

            Message::Exit => {
                self.save_current_session();
                return window::get_latest().and_then(window::close);
            }
        }

        Task::none()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen_with(|event, _, _| {
            if let iced::Event::Window(iced::window::Event::CloseRequested) = event {
                Some(Message::Exit)
            } else {
                None
            }
        })
    }

    pub fn view(&self) -> Element<Message> {
        let current_profile = self
            .profile_selected
            .as_ref()
            .and_then(|s| self.profiles.get(s));

        let header: Row<_> = row!(
            image("./resources/icon.ico"),
            text!("{} v{}", APP_TITLE, APP_VERSION)
                .align_y(Vertical::Center)
                .align_x(Horizontal::Left)
                .size(30)
        )
        .spacing(10);

        let instance_controls: Container<_> = container(row!(
            combo_box(
                &self.state.instance_choices,
                "Instance",
                self.instance_selected.as_ref(),
                Message::InstanceSelected,
            )
            .on_input(Message::InstanceInput),
            button("Add").on_press_maybe(self.profile_selected.as_ref().and_then(|s| {
                let profile = self.profiles.get(s)?;
                if profile.path.display().to_string() != "" {
                    Some(Message::InstanceAdd(s.clone()))
                } else {
                    None
                }
            })),
            button("Remove").on_press_maybe(self.profile_selected.as_ref().and_then(|s| {
                let profile = self.profiles.get(s)?;
                if profile.path.display().to_string() != "" {
                    Some(Message::InstanceRemove(s.clone()))
                } else {
                    None
                }
            })),
        ));

        let profile_controls: Container<_> = container(
            row!(
                combo_box(
                    &self.state.profile_choices,
                    "Profile",
                    self.profile_selected.as_ref(),
                    Message::ProfileSelected,
                ),
                if let Some(profile) = current_profile {
                    if profile.path.display().to_string() == "" {
                        container(button("Browse").on_press(Message::BrowseGameDir(profile.name.clone())))
                    } else {
                        instance_controls
                    }
                } else {
                    container(horizontal_space())
                }
            )
            .spacing(10),
        )
        .center_x(Length::Fill);

        let mut files_column: Column<_> = Column::new();

        if self.instance_selected.is_some() {
            if let Some(files) = self.files_buffer.as_ref() {
                files_column = files.iter().fold(Column::new(), |column, (path, enabled)| {
                    let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                    let label: Element<'_, Message>;
                    let icon: Element<'_, Message>;

                    match path.is_dir() {
                        false => {
                            icon = image("./resources/asset.png").into();
                            label = text(file_name).into();
                        }
                        true => {
                            icon = image("./resources/dir.png").into();
                            label = ClickableText::new(file_name, Message::TraverseIntoDir(path.clone())).into();
                        }
                    };

                    column.push(
                        styled_container!(
                            row![
                                checkbox("", *enabled).on_toggle(move |new_state| Message::FileToggle(path.clone(), new_state)),
                                icon,
                                label
                            ],
                            border_width = 2.0,
                            border_radius = 4.0
                        )
                        .padding(5)
                        .align_left(Length::Fill),
                    )
                });
            }
        }

        let mods_menu: Container<_> = styled_container!(
            column!(text("mods menu")),
            border_width = 4.0,
            border_radius = 8.0
        )
        .center(Length::Fill);

        let files_menu: Container<_> = styled_container!(
            column!(scrollable(files_column)).spacing(10),
            border_width = 4.0,
            border_radius = 8.0
        )
        .padding(10)
        .align_y(Vertical::Top)
        .center_x(Length::Fill);

        let editor_space: Container<_> = container(row!(mods_menu, files_menu).spacing(10)).center(Length::Fill);

        column![header, profile_controls, editor_space]
            .spacing(10)
            .padding(10)
            .into()
    }

    fn switch_profile(&mut self, profile_name: &str) -> Task<Message> {
        self.write_current_changes();
        let next_profile_name = profile_name.to_owned();

        let Some(next_profile) = self.profiles.get(&next_profile_name) else {
            return Task::none();
        };

        self.profile_selected = Some(next_profile_name.clone());
        self.instance_selected = None;

        if let Some(instances) = &next_profile.instances {
            self.state.instance_choices = State::new(instances.keys().cloned().collect::<Vec<String>>());
        } else {
            self.state.instance_choices = State::new(Vec::new());
        };

        if next_profile.path.display().to_string().is_empty() {
            Task::none()
        } else {
            Task::done(Message::RefreshFiles)
        }
    }

    fn write_current_changes(&mut self) {
        let Some(current_profile) = self
            .profiles
            .get_mut(&self.profile_selected.clone().unwrap_or_default())
        else {
            return;
        };

        if let Some(instances) = current_profile.instances.as_mut() {
            if let Some(current_instance) = instances.get_mut(&self.instance_selected.clone().unwrap_or_default()) {
                current_instance.files = self.files_buffer.clone();
            }
        }
    }

    fn add_instance_for_profile(&mut self, profile_name: &str) -> Task<Message> {
        let profile_name = profile_name.to_owned();
        let instance_name = self.get_instance_name(&profile_name);
        let new_instance = Instance::default().with_name(&instance_name);

        let Some(current_profile) = self.profiles.get_mut(&profile_name) else {
            return Task::none();
        };

        let Some(instances) = current_profile.instances.as_mut() else {
            let new_instances = Lookup::from(vec![new_instance]);
            self.state.instance_choices = State::new(new_instances.keys().cloned().collect::<Vec<String>>());
            current_profile.instances = Some(new_instances);
            return Task::none();
        };

        if instances.contains_key(&instance_name) {
            return Task::none();
        }

        instances.insert(instance_name.to_owned(), new_instance.clone());
        self.state.instance_choices = State::new(instances.keys().cloned().collect::<Vec<String>>());

        Task::none()
    }

    fn remove_instance_from_profile(&mut self, profile_name: &str) -> Task<Message> {
        let profile_name = profile_name.to_owned();
        let selected_instance_name = self.instance_selected.clone().unwrap_or_default();

        let Some(profile) = self.profiles.get_mut(&profile_name) else {
            return Task::none();
        };

        let Some(instances) = profile.instances.as_mut() else {
            return Task::none();
        };

        instances.remove(&selected_instance_name);

        self.state.instance_choices = State::new(instances.keys().cloned().collect::<Vec<String>>());
        self.instance_selected = None;
        self.state.instance_input = None;

        Task::none()
    }

    fn select_instance(&mut self, instance_name: &str) -> Task<Message> {
        self.write_current_changes();
        let instance_name = instance_name.to_owned();
        self.instance_selected = Some(instance_name.clone());
        Task::done(Message::RefreshFiles)
    }

    fn browse_game_dir(&mut self, profile_name: &str) -> Task<Message> {
        let profile_name = profile_name.to_owned();
        let Some(path) = FileDialog::new()
            .set_title(format!("Select {} directory", &profile_name))
            .pick_folder()
        else {
            return Task::none();
        };

        if !path.exists() || !path.is_dir() {
            return Task::none();
        };

        let Some(profile) = self.profiles.get_mut(&profile_name) else {
            return Task::none();
        };

        profile.path = path.clone();

        Task::done(Message::RefreshFiles)
    }

    fn refresh_files(&mut self) -> Task<Message> {
        let Some(current_profile) = self
            .profiles
            .get_mut(&self.profile_selected.clone().unwrap_or_default())
        else {
            return Task::none();
        };

        let Ok(profile_dir) = current_profile.path.read_dir() else {
            return Task::none();
        };

        match self.files_buffer.as_mut() {
            Some(files) => {
                files.clear();
                profile_dir.flatten().for_each(|entry| {
                    files.insert(entry.path(), true);
                });
            }
            None => {
                let mut files: Lookup<PathBuf, bool> = Lookup::default();
                profile_dir.flatten().for_each(|entry| {
                    files.insert(entry.path(), true);
                });
                self.files_buffer = Some(files);
            }
        }

        Task::done(Message::ExtendFilesFromInstance)
    }

    fn extend_from_instance(&mut self) -> Task<Message> {
        let Some(current_profile) = self
            .profiles
            .get_mut(&self.profile_selected.clone().unwrap_or_default())
        else {
            return Task::none();
        };

        if let Some(instances) = &current_profile.instances {
            let Some(current_instance) = instances.get(&self.instance_selected.clone().unwrap_or_default()) else {
                return Task::none();
            };

            if let Some(instance_files) = &current_instance.files {
                if let Some(files_buffer) = self.files_buffer.as_mut() {
                    instance_files.iter().for_each(|(path, enabled)| {
                        files_buffer.insert(path.clone(), *enabled);
                    })
                }
            }
        }

        Task::none()
    }

    fn save_current_session(&self) {
        self.profiles.values().for_each(|p| match save_profile!(p) {
            Ok(_) => {}
            Err(e) => eprintln!("Failed saving profile: {}", e),
        });

        if let Err(e) = save_session!(
            self.profile_selected.clone(),
            self.instance_selected.clone()
        ) {
            eprintln!("Failed saving session: {}", e);
        }
    }

    fn get_instance_name(&self, profile_name: &str) -> String {
        self.state
            .instance_input
            .clone()
            .unwrap_or_else(|| format!("{}_instance_{}", profile_name, Local::now().timestamp()))
    }

    fn preload_profiles() -> Lookup<String, Profile> {
        Lookup::from(
            GAME_PROFILES
                .iter()
                .map(|profile_name| match load_profile!(profile_name) {
                    Some(p) => (profile_name.to_string(), p),
                    None => (
                        profile_name.to_string(),
                        Profile::default().with_name(profile_name),
                    ),
                })
                .collect::<Vec<(String, Profile)>>(),
        )
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    BrowseGameDir(String),
    ProfileSelected(String),
    InstanceSelected(String),
    InstanceAdd(String),
    InstanceRemove(String),
    InstanceInput(String),
    FileToggle(PathBuf, bool),
    TraverseIntoDir(PathBuf),
    RefreshFiles,
    ExtendFilesFromInstance,
    Exit,
}
