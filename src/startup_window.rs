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

#[allow(dead_code)]
use fltk::app::App;
use fltk::browser::HoldBrowser;
use fltk::button::Button;
use fltk::enums::Align;
use fltk::enums::CallbackTrigger;
use fltk::group::Grid;
use fltk::group::GridAlign;
use fltk::input::Input;
use fltk::menu::Choice;
use fltk::prelude::*;
use fltk::window::Window;

use rfd::FileDialog;

use crate::application::ApplicationSettings;
use crate::application::GothicOrganizerWindow;
use crate::constants::game_profile_list;
use crate::constants::Style;
use crate::constants::Theme;
use crate::error::GuiError;
use crate::load_profile;
use crate::profile::init_profile;
use crate::profile::Instance;
use crate::profile::Profile;
use crate::save_profile;

#[derive(Default, Debug)]
pub struct StartupWindow {
    window: Window,
    app: App,
    profile_choices: Vec<String>,
    instance_name_input: String,
    instance_choices: Option<Vec<String>>,
    current_profile: Option<Profile>,
    pub selected_directory: Option<String>,
    pub selected_profile: Option<String>,
    pub selected_instance: Option<String>,
    pub canceled: bool,
}

impl GothicOrganizerWindow for StartupWindow {
    type Message = Message;

    fn run(&mut self) -> Result<(), GuiError> {
        let (sender, receiver) = fltk::app::channel::<self::Message>();
        self.window.begin();

        let mut layout_grid = Grid::default_fill();
        layout_grid.set_layout(10, 1);
        layout_grid.set_margin(10, 20, 10, 10);
        layout_grid.set_gap(20, 10);

        let mut profile_choice = Choice::default()
            .with_size(self.window.width() - 50, 30)
            .with_align(Align::TopLeft)
            .with_label("Profile:");

        self.profile_choices.iter().for_each(|p| {
            profile_choice.add_choice(p);
        });

        profile_choice.emit(sender, Message::SelectProfile);

        let mut browse_button = Button::default()
            .with_size(self.window.width() - 50, 30)
            .with_align(Align::Center)
            .with_label("Browse game directory...");

        browse_button.emit(sender, Message::SelectProfileDirectory);
        browse_button.deactivate();

        let mut instance_entry = Input::default()
            .with_size(self.window.width() - 50, 30)
            .with_align(Align::TopLeft)
            .with_label("New instance name:");

        instance_entry.set_trigger(CallbackTrigger::EnterKeyChanged);
        instance_entry.set_callback(move |i| sender.send(Message::InputInstanceName(i.value().to_string())));
        instance_entry.deactivate();

        let mut add_instance_button = Button::default()
            .with_size(self.window.width() - 50, 30)
            .with_align(Align::Center)
            .with_label("Add instance");

        add_instance_button.emit(sender, Message::AddInstance);
        add_instance_button.deactivate();

        let mut remove_instance_button = Button::default()
            .with_size(self.window.width() - 50, 30)
            .with_align(Align::Center)
            .with_label("Remove instance");

        remove_instance_button.emit(sender, Message::RemoveInstance);
        remove_instance_button.deactivate();

        let mut instance_selector = HoldBrowser::default()
            .with_size(self.window.width() - 50, 200)
            .with_align(Align::TopLeft)
            .with_label("Instances:");

        instance_selector.emit(sender, Message::SelectInstance);
        if let Some(available) = &self.instance_choices {
            available.iter().for_each(|i| {
                instance_selector.add(i);
            });
        }
        instance_selector.deactivate();

        let mut start_button = Button::default()
            .with_size(self.window.width() - 50, 30)
            .with_align(Align::Center)
            .with_label("Start");

        start_button.emit(sender, Message::Start);
        start_button.deactivate();

        let mut cancel_button = Button::default()
            .with_size(self.window.width() - 50, 30)
            .with_align(Align::Center)
            .with_label("Cancel");

        cancel_button.emit(sender, Message::Exit);

        layout_grid.set_widget_ext(&mut profile_choice, 0, 0, GridAlign::PROPORTIONAL)?;
        layout_grid.set_widget_ext(&mut browse_button, 1, 0, GridAlign::PROPORTIONAL)?;
        layout_grid.set_widget_ext(&mut instance_entry, 2, 0, GridAlign::PROPORTIONAL)?;
        layout_grid.set_widget_ext(&mut add_instance_button, 3, 0, GridAlign::PROPORTIONAL)?;
        layout_grid.set_widget_ext(&mut remove_instance_button, 4, 0, GridAlign::PROPORTIONAL)?;
        layout_grid.set_widget_ext(&mut instance_selector, 5, 0, GridAlign::PROPORTIONAL)?;
        layout_grid.set_widget_ext(&mut start_button, 6, 0, GridAlign::PROPORTIONAL)?;
        layout_grid.set_widget_ext(&mut cancel_button, 7, 0, GridAlign::PROPORTIONAL)?;

        self.window.end();
        self.window.show();

        while self.app.wait() {
            if let Some(msg) = receiver.recv() {
                match msg {
                    Message::SelectProfileDirectory => {
                        let dir = FileDialog::new()
                            .pick_folder()
                            .map(|p| p.to_str().unwrap().to_string());

                        self.selected_directory = dir;
                        self.current_profile = match load_profile!(&self.selected_profile.clone().unwrap()) {
                            Some(p) => Some(p),
                            None => {
                                init_profile(
                                    &self.selected_profile.clone().unwrap(),
                                    self.selected_directory.clone().unwrap(),
                                    None,
                                )?;
                                load_profile!(&self.selected_profile.clone().unwrap())
                            }
                        };
                    }
                    Message::SelectProfile => {
                        let profile = self
                            .profile_choices
                            .get(profile_choice.value() as usize)
                            .map(|p| p.to_string());

                        self.selected_profile = profile;
                        browse_button.activate();
                        instance_entry.activate();
                        add_instance_button.activate();
                        remove_instance_button.activate();
                        instance_selector.activate();
                    }
                    Message::SelectInstance => {
                        if let Some(available) = &self.instance_choices {
                            let value_index = instance_selector.value();
                            if value_index - 1 < available.len() as i32 && !available.is_empty() {
                                let instance = available
                                    .get((instance_selector.value() as usize).saturating_sub(1))
                                    .cloned();
                                self.selected_instance = instance;
                            }
                        }
                        start_button.activate();
                    }
                    Message::InputInstanceName(name) => {
                        self.instance_name_input = name;
                    }
                    Message::AddInstance => {
                        if self.instance_name_input.clone().is_empty() {
                            continue;
                        }

                        if let Some(available) = &mut self.instance_choices {
                            if available.contains(&self.instance_name_input) {
                                continue;
                            }

                            available.push(self.instance_name_input.clone());
                        } else {
                            self.instance_choices = Some(vec![self.instance_name_input.clone()]);
                        }

                        instance_selector.add(&self.instance_name_input);
                        self.current_profile
                            .as_mut()
                            .unwrap()
                            .add_instance(Instance::new(self.instance_name_input.clone()));
                        save_profile!(self.current_profile.clone().unwrap())?;
                    }
                    Message::RemoveInstance => {
                        if let Some(available) = &mut self.instance_choices {
                            if available.is_empty() {
                                continue;
                            }

                            let value_index = instance_selector.value();
                            if value_index != 0 && value_index - 1 < available.len() as i32 && !available.is_empty() {
                                available.remove((value_index as usize).saturating_sub(1));
                                instance_selector.remove(value_index);
                            }

                            if available.is_empty() {
                                start_button.deactivate();
                            }
                        }
                    }
                    Message::Start => {
                        self.window.hide();
                    }
                    Message::Exit => {
                        self.canceled = true;
                        self.window.hide();
                    }
                }
            }
        }

        Ok(())
    }
}

impl StartupWindow {
    pub fn new() -> Self {
        let settings = ApplicationSettings {
            title: "Gothic Organizer: Startup".to_string(),
            width: 340,
            height: 580,
            centered: true,
            resizable: true,
            theme: Theme::Dark,
            style: Style::Fluent,
            icon: Some("resources/icon.ico".into()),
            ..Default::default()
        };

        Self {
            window: Self::window(&settings),
            app: Self::app(&settings),
            canceled: false,
            profile_choices: game_profile_list().to_vec(),
            ..Default::default()
        }
    }

    pub fn with_instances(instances: Option<Vec<String>>) -> Self {
        let mut sw = Self::new();
        sw.instance_choices = instances;
        sw
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectProfileDirectory,
    SelectProfile,
    SelectInstance,
    InputInstanceName(String),
    AddInstance,
    RemoveInstance,
    Start,
    Exit,
}

pub mod prelude {
    pub use crate::application::GothicOrganizerWindow;
    pub use crate::startup_window::StartupWindow;
}
