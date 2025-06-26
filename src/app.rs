use std::path::Path;
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
    pub const WINDOW_TITLE: &str = "Gothic Organizer";
    pub const WINDOW_SIZE: (f32, f32) = (768.0, 768.0);

    pub fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();

        if let Err(err) = Self::try_reload_last_session(&mut app) {
            eprintln!("{err}");
        }

        (app, Task::done(Message::RefreshFiles))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match &message {
            Message::ProfileSelected(profile_name) => {
                self.print_debug();
                return self.switch_profile(profile_name);
            }

            Message::InstanceSelected(instance) => {
                self.print_debug();
                return self.select_instance(instance);
            }

            Message::InstanceInput(input) => {
                self.state.instance_input = Some(input.clone());
            }

            Message::InstanceAdd(profile_name) => {
                self.print_debug();
                return self.add_instance_for_profile(profile_name);
            }

            Message::InstanceRemove(profile_name) => {
                self.remove_instance_from_profile(profile_name);
            }

            Message::FileToggle(path) => {
                self.toggle_state_recursive(Some(path));
            }

            Message::FileToggleAll => {
                self.toggle_state_recursive(None);
            }

            Message::BrowseGameDir(profile_name) => {
                self.print_debug();
                return self.browse_game_dir(profile_name);
            }

            Message::TraverseIntoDir(path) => {
                self.write_current_changes();
                self.state.current_directory = path.clone();
                self.refresh_files(Some(path.clone()))
            }

            Message::RefreshFiles => {
                self.refresh_files(None);
            }

            Message::Exit => {
                self.write_current_changes();
                self.save_current_session();
                self.print_debug();
                return window::get_latest().and_then(window::close);
            }
        }

        self.print_debug();
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

        let selected_profile_instances = selected_profile.instances.ok_or_else(|| {
            app.files = last_session.cache.unwrap_or_default();
            GothicOrganizerError::new("no instances for selected profile")
        })?;

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

        if let Some(instances) = current_profile.instances.as_mut()
            && let Some(current_instance) = instances.get_mut(&self.instance_selected.clone().unwrap_or_default())
        {
            current_instance.files = Some(self.files.clone());
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
            return Task::done(Message::RefreshFiles);
        };

        if instances.contains_key(&instance_name) {
            return Task::none();
        }

        instances.insert(instance_name.to_owned(), new_instance.clone());
        self.state.instance_choices = State::new(instances.keys().cloned().collect::<Vec<String>>());

        Task::done(Message::RefreshFiles)
    }

    fn remove_instance_from_profile(&mut self, profile_name: &str) {
        let profile_name = profile_name.to_owned();
        let selected_instance_name = self.instance_selected.clone().unwrap_or_default();

        if let Some(profile) = self.profiles.get_mut(&profile_name)
            && let Some(instances) = profile.instances.as_mut()
        {
            instances.remove(&selected_instance_name);
            self.state.instance_choices = State::new(instances.keys().cloned().collect::<Vec<String>>());
            self.instance_selected = None;
            self.state.instance_input = None;
            if instances.is_empty() {
                profile.instances = None;
            }
        }
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

    fn refresh_files(&mut self, root: Option<PathBuf>) {
        let Some(current_profile) = self
            .profiles
            .get_mut(&self.profile_selected.clone().unwrap_or_default())
        else {
            return;
        };

        let root_dir = root.unwrap_or_else(|| current_profile.path.clone());
        self.state.current_directory = root_dir.clone();

        let Ok(root_dir_iter) = root_dir.read_dir() else {
            return;
        };

        let profile_dir_entries: Vec<(PathBuf, bool)> = root_dir_iter
            .flatten()
            .map(|entry| (entry.path(), true))
            .collect();

        if let Some(selected_instance) = &self.instance_selected
            && let Some(instances) = &current_profile.instances
            && let Some(current_instance) = instances.get(selected_instance)
        {
            if let Some(instance_files) = &current_instance.files
                && !instance_files.is_empty()
            {
                for (path, enabled) in instance_files.iter() {
                    self.files.insert(path.clone(), *enabled);
                }
            }

            self.state.current_directory_entries.clear();

            for (path, _) in &profile_dir_entries {
                if let Some(displayed_state) = self.files.get(path) {
                    self.state
                        .current_directory_entries
                        .push((path.clone(), *displayed_state));
                }
            }
        } else {
            self.state.current_directory_entries = profile_dir_entries;
        }

        self.state
            .current_directory_entries
            .sort_unstable_by_key(|(path, _)| !path.is_dir());
    }

    fn save_current_session(&self) {
        self.profiles.values().for_each(|p| match save_profile!(p) {
            Ok(_) => {}
            Err(e) => eprintln!("Failed saving profile: {e}"),
        });

        let cache = match self
            .profiles
            .get(&self.profile_selected.clone().unwrap_or_default())
        {
            Some(current_profile) if current_profile.instances.is_some() => None,
            _ => Some(self.files.clone()),
        };

        if let Err(e) = save_session!(
            self.profile_selected.clone(),
            self.instance_selected.clone(),
            cache
        ) {
            eprintln!("Failed saving session: {e}");
        }
    }

    fn get_instance_name(&self, profile_name: &str) -> String {
        self.state
            .instance_input
            .clone()
            .unwrap_or_else(|| format!("{}_instance_{}", profile_name, Local::now().timestamp()))
    }

    fn toggle_state_recursive(&mut self, path: Option<&Path>) {
        if let Some(path) = path
            && let Some(old_state) = self
                .state
                .current_directory_entries
                .iter_mut()
                .find_map(|(p, s)| if p == path { Some(s) } else { None })
        {
            let new_state = !(*old_state);
            *old_state = new_state;
            if path.is_dir() {
                self.files.insert(path.to_path_buf(), new_state);
                self.files.iter_mut().for_each(|(p, s)| {
                    if p.starts_with(path) {
                        *s = !(*s);
                    }
                })
            }
        } else {
            for (path, state) in self.state.current_directory_entries.iter_mut() {
                let new_state = !(*state);
                *state = new_state;
                if path.is_dir() {
                    self.files.insert(path.clone(), new_state);
                    self.files.iter_mut().for_each(|(p, s)| {
                        if p.starts_with(path.clone()) {
                            *s = new_state;
                        }
                    })
                }
            }
        }
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

    fn print_debug(&self) {
        println!(
            "{}\nSelected profile: {:#?}\nSelected instance: {:#?}\nFiles: {:#?}\nCurrent directory entries: {:#?}\nCurrent directory: {:#?}\n{}\n",
            "-----".repeat(10),
            self.profile_selected,
            self.instance_selected,
            self.files.iter().count(),
            self.state.current_directory_entries.len(),
            self.state.current_directory,
            "-----".repeat(10),
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
    FileToggle(PathBuf),
    TraverseIntoDir(PathBuf),
    FileToggleAll,
    RefreshFiles,
    Exit,
}
