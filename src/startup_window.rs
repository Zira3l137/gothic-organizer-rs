/* TODO: Create a startup window for the app.
*
* Startup window should contain:
* - Logo
* - App name
* - Combo box for allowed game profiles (Gothic, Gothic II NOTR, Gothic II Classic?, Gothic
* Sequel?)
* - Selectable list box with available instances for the game profile
* - Button to browse for select game directory
* - Button to add an instance for the game profile into the list
* - Button to remove an instance from the list
* - Button to start the app
*
*     After user runs the main app, startup window should appear. User then will have to select a
* desired game profile and provide a path to the game directory. Then, he should either create a
* new instance of the game with an arbitrary name or select an existing one from the list. After
* that, he should be able to start the main app.
*
*     Upon startup window creation, the list of instances should be loaded from the config file if
* it exists, otherwise a new one should be initiliazed.
*/

use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::widget::button;
use iced::widget::column;
use iced::widget::combo_box;
use iced::widget::image;
use iced::widget::row;
use iced::widget::text;
use iced::window;
use iced::Element;
use iced::Task;

use rfd::FileDialog;

use chrono::Local;

use crate::constants::APP_TITLE;
use crate::constants::APP_VERSION;
use crate::constants::GAME_PROFILES;
use crate::profile::Instance;
use crate::profile::Profile;

#[derive(Debug, Default)]
pub struct StartupWindow {
    allowed_games: combo_box::State<String>,
    available_instances: combo_box::State<String>,
    game_selected: Option<String>,
    instance_selected: Option<String>,
    instance_input: Option<String>,
    profiles: Vec<Profile>,
}

impl StartupWindow {
    pub const WINDOW_TITLE: &str = "Startup Window";
    pub const WINDOW_SIZE: (f32, f32) = (400.0, 260.0);

    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                allowed_games: combo_box::State::new(
                    GAME_PROFILES
                        .iter()
                        .map(|s| String::from(*s))
                        .collect::<Vec<String>>(),
                ),
                available_instances: combo_box::State::new(Vec::new()),
                game_selected: None,
                instance_selected: None,
                instance_input: None,
                profiles: Vec::new(),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match &message {
            Message::GameSelected(game) => {
                self.game_selected = Some(game.clone());
            }
            Message::InstanceSelected(instance) => {
                self.instance_selected = Some(instance.clone());
            }
            Message::InstanceInput(input) => {
                self.instance_input = Some(input.clone());
            }
            Message::BrowseGameDir(game) => {
                if let Some(path) = FileDialog::new()
                    .set_title(format!("Select {} directory", game))
                    .pick_folder()
                {
                    self.profiles.push(Profile {
                        name: game.clone(),
                        game_path: path,
                        instances: None,
                    });
                }
            }
            Message::InstanceAddForGame(profile) => {
                if let Some(p) = self.profiles.iter_mut().find(|p| p.name == *profile) {
                    match p.instances.as_mut() {
                        None => {
                            let instances = vec![Instance {
                                name: self
                                    .instance_input
                                    .clone()
                                    .unwrap_or(Local::now().to_string()),
                                mods: None,
                                downloads: None,
                            }];
                            p.instances = Some(instances);
                        }
                        Some(instances) => instances.push(Instance {
                            name: self
                                .instance_input
                                .clone()
                                .unwrap_or(Local::now().to_string()),
                            mods: None,
                            downloads: None,
                        }),
                    }
                    self.available_instances = combo_box::State::new(
                        p.instances
                            .as_ref()
                            .unwrap_or(&Vec::new())
                            .iter()
                            .map(|i| i.name.clone())
                            .collect::<Vec<String>>(),
                    )
                }
            }
            Message::InstanceRemoveForGame(profile) => {
                if let Some(p) = self.profiles.iter_mut().find(|p| p.name == *profile) {
                    match p.instances.as_mut() {
                        None => (),
                        Some(instances) => {
                            instances.retain(|i| i.name != *self.instance_selected.as_ref().unwrap());
                            self.available_instances = combo_box::State::new(
                                instances
                                    .iter()
                                    .filter_map(|i| match i.name.clone() {
                                        name if name
                                            != *self
                                                .instance_selected
                                                .as_ref()
                                                .unwrap_or(&String::default()) =>
                                        {
                                            Some(name)
                                        }
                                        _ => None,
                                    })
                                    .collect::<Vec<String>>(),
                            )
                        }
                    }
                    self.instance_selected = None;
                    self.instance_input = None;
                }
            }
            Message::StartApp(_profile, _instance) => (),
            Message::Exit => {
                return window::get_latest().and_then(window::close);
            }
        }

        if cfg!(debug_assertions) {
            println!();
            println!("Message: {:?}", message);
            println!("Profiles: {:?}", self.profiles);
            println!(
                "Selected game: {}",
                self.game_selected.as_ref().unwrap_or(&String::default())
            );
            println!(
                "Selected instance: {}",
                self.instance_selected
                    .as_ref()
                    .unwrap_or(&String::default())
            );
            println!(
                "Instance input: {}",
                self.instance_input.as_ref().unwrap_or(&String::default())
            );
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        column![
            row!(
                image("./resources/icon.ico"),
                text!("{} v{}", APP_TITLE, APP_VERSION)
                    .align_y(Vertical::Center)
                    .align_x(Horizontal::Center)
                    .size(30)
            )
            .spacing(10),
            row!(
                combo_box(
                    &self.allowed_games,
                    "select a game",
                    self.game_selected.as_ref(),
                    Message::GameSelected,
                ),
                button("...").on_press_maybe(
                    self.game_selected
                        .as_ref()
                        .map(|s| Message::BrowseGameDir(s.clone()))
                ),
            )
            .spacing(10),
            row!(
                combo_box(
                    &self.available_instances,
                    "select an instance",
                    self.instance_selected.as_ref(),
                    Message::InstanceSelected,
                )
                .on_input(Message::InstanceInput),
                button("Add").on_press_maybe(self.game_selected.as_ref().and_then(|s| {
                    self.profiles
                        .iter()
                        .find(|p| p.name == *s)
                        .map(|_| Message::InstanceAddForGame(s.clone()))
                })),
                button("Remove").on_press_maybe(self.game_selected.as_ref().and_then(|s| {
                    self.profiles
                        .iter()
                        .find(|p| p.name == *s)
                        .map(|_| Message::InstanceRemoveForGame(s.clone()))
                })),
            )
            .spacing(10),
            row!(
                button("Proceed").on_press_maybe(self.game_selected.as_ref().and_then(|s| {
                    self.profiles
                        .iter()
                        .find(|p| p.name == *s)
                        .and_then(|p| match self.instance_selected.as_ref() {
                            Some(instance) => p
                                .instances
                                .as_ref()
                                .and_then(|instances| instances.iter().find(|i| i.name == *instance))
                                .map(|i| (p, i)),
                            None => None,
                        })
                        .map(|(p, i)| Message::StartApp(p.clone(), i.clone()))
                })),
                button("Cancel").on_press(Message::Exit),
            )
            .spacing(10)
            .padding(10)
        ]
        .spacing(10)
        .padding(10)
        .into()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    GameSelected(String),
    BrowseGameDir(String),
    InstanceSelected(String),
    InstanceAddForGame(String),
    InstanceRemoveForGame(String),
    InstanceInput(String),
    StartApp(Profile, Instance),
    Exit,
}
