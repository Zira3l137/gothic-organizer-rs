use std::path::PathBuf;

use iced::widget::combo_box::State;
use iced::window;
use iced::Element;
use iced::Task;

use rfd::FileDialog;

use ignore::WalkBuilder;

use chrono::Local;

use crate::core::profile::Instance;
use crate::core::profile::Lookup;
use crate::core::profile::Profile;
use crate::error::GothicOrganizerError;
use crate::load_profile;
use crate::load_session;
use crate::save_profile;
use crate::save_session;

#[derive(Debug, Default)]
pub struct GothicOrganizer {
    pub profile_selected: Option<String>,
    pub instance_selected: Option<String>,
    pub profiles: Lookup<String, Profile>,
    pub files: Lookup<PathBuf, bool>,
    pub state: InnerState,
}

#[derive(Debug, Default)]
pub struct InnerState {
    pub instance_input: Option<String>,
    pub profile_choices: State<String>,
    pub instance_choices: State<String>,
    pub current_directory_entries: Vec<(PathBuf, bool)>,
    pub current_directory: PathBuf,
}

impl GothicOrganizer {
    pub const WINDOW_TITLE: &str = "Startup Window";
    pub const WINDOW_SIZE: (f32, f32) = (768.0, 768.0);

    pub fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();

        if let Err(err) = Self::try_reload_last_session(&mut app) {
            eprintln!("{}", err);
        }

        (app, Task::done(Message::RefreshFiles))
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
                if let Some(state) = self
                    .state
                    .current_directory_entries
                    .iter_mut()
                    .find(|(p, _)| p == path)
                {
                    state.1 = *new_state;
                }
            }

            Message::BrowseGameDir(profile_name) => {
                return self.browse_game_dir(profile_name);
            }

            Message::TraverseIntoDir(path) => {
                self.write_current_changes();
                self.state.current_directory = path.clone();
                return self.refresh_files(Some(path.clone()));
            }

            Message::RefreshFiles => {
                return self.refresh_files(None);
            }

            Message::Exit => {
                self.write_current_changes();
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
        crate::gui::editor_view::editor_view(self)
    }

    pub fn try_reload_last_session(app: &mut Self) -> Result<(), GothicOrganizerError> {
        let profiles = Self::preload_profiles();
        app.profiles = profiles.clone();
        app.state.profile_choices = State::new(profiles.keys().cloned().collect());

        let last_session = load_session!().ok_or(GothicOrganizerError::new("failed to load last session"))?;

        let selected_profile_name = last_session
            .selected_profile
            .ok_or(GothicOrganizerError::new("no selected profile"))?
            .clone();

        app.profile_selected = Some(selected_profile_name.clone());

        let selected_profile = profiles
            .get(&selected_profile_name)
            .ok_or(GothicOrganizerError::new(&format!(
                "no profile with name {}",
                &selected_profile_name
            )))?
            .clone();

        let selected_profile_instances = selected_profile.instances.ok_or(GothicOrganizerError::new(
            "no instances for selected profile",
        ))?;

        app.state.instance_choices = State::new(selected_profile_instances.keys().cloned().collect());

        let selected_instance_name = last_session
            .selected_instance
            .ok_or(GothicOrganizerError::new("no selected instance"))?
            .clone();

        app.instance_selected = Some(selected_instance_name.clone());

        let selected_instance = selected_profile_instances
            .get(&selected_instance_name)
            .ok_or(GothicOrganizerError::new(&format!(
                "no instance with name {} for profile {}",
                &selected_instance_name, &selected_profile_name
            )))?
            .clone();

        app.files = selected_instance
            .files
            .ok_or(GothicOrganizerError::new("no files for selected instance"))?;

        Ok(())
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

        self.state
            .current_directory_entries
            .iter()
            .for_each(|(path, enabled)| {
                self.files.insert(path.clone(), *enabled);
            });

        if let Some(instances) = current_profile.instances.as_mut() {
            if let Some(current_instance) = instances.get_mut(&self.instance_selected.clone().unwrap_or_default()) {
                current_instance.files = Some(self.files.clone());
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

        Task::done(Message::RefreshFiles)
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
        self.state.current_directory = path.clone();

        WalkBuilder::new(path)
            .ignore(false)
            .build()
            .filter_map(Result::ok)
            .for_each(|entry| {
                self.files.insert(entry.path().to_path_buf(), true);
            });

        Task::done(Message::RefreshFiles)
    }

    fn refresh_files(&mut self, root: Option<PathBuf>) -> Task<Message> {
        let Some(current_profile) = self
            .profiles
            .get_mut(&self.profile_selected.clone().unwrap_or_default())
        else {
            return Task::none();
        };

        let root_dir = match root {
            Some(root) => root,
            None => current_profile.path.clone(),
        };

        self.state.current_directory = root_dir.clone();

        let Ok(root_dir) = root_dir.read_dir() else {
            return Task::none();
        };

        // Get all the files in the profile directory
        let profile_dir_entries = root_dir
            .flatten()
            .map(|entry| (entry.path(), true))
            .collect::<Vec<(PathBuf, bool)>>();

        // If not instance is selected, just show the profile directory
        let Some(selected_instance) = &self.instance_selected else {
            self.state.current_directory_entries = profile_dir_entries;
            return Task::none();
        };

        if let Some(instances) = &current_profile.instances {
            // If couldn't find the selected instance, just show the profile directory
            let Some(current_instance) = instances.get(selected_instance) else {
                self.state.current_directory_entries = profile_dir_entries;
                return Task::none();
            };

            // If current instance has any files listed, update the files cache with them
            if let Some(instance_files) = &current_instance.files {
                if !instance_files.is_empty() {
                    instance_files.iter().for_each(|(path, enabled)| {
                        self.files.insert(path.clone(), *enabled);
                    });
                }
            }

            // Update the current directory buffer
            self.state.current_directory_entries.clear();
            profile_dir_entries.iter().for_each(|(path, _)| {
                if let Some(displayed_state) = self.files.get(path) {
                    self.state
                        .current_directory_entries
                        .push((path.clone(), *displayed_state));
                }
            })
        }

        self.state
            .current_directory_entries
            .sort_unstable_by_key(|(path, _)| !path.is_dir());

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
            crate::core::constants::Profile::into_iter()
                .map(|profile_name| match load_profile!((*profile_name).into()) {
                    Some(p) => (profile_name.to_string(), p),
                    None => (
                        profile_name.to_string(),
                        Profile::default().with_name((*profile_name).into()),
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
    Exit,
}
