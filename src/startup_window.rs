use std::collections::hash_map::HashMap;
use std::fs::read_dir;
use std::fs::remove_dir_all;
use std::ops::DerefMut;

use fltk::browser::HoldBrowser;
use fltk::button::Button;
use fltk::enums::Align;
use fltk::enums::CallbackTrigger;
use fltk::group::GridAlign;
use fltk::input::Input;
use fltk::menu::Choice;
use fltk::prelude::*;

use rfd::FileDialog;

use crate::application::AnyWidget;
use crate::application::ApplicationSettings;
use crate::application::GothicOrganizerWindow;
use crate::constants::game_profile_list;
use crate::constants::ColorScheme;
use crate::constants::Style;
use crate::error::GuiError;
use crate::load_profile;
use crate::local_instances;
use crate::profile::init_profile;
use crate::profile::Instance;
use crate::profile::Profile;
use crate::save_profile;

#[derive(Default)]
pub struct StartupWindow {
    widgets: HashMap<String, AnyWidget>,
    profile_choices: Vec<String>,
    instance_name_input: String,
    instance_choices: Option<Vec<String>>,
    current_profile: Option<Profile>,
    selected_profile_index: i32,
    selected_instance_index: i32,
    pub selected_directory: Option<String>,
    pub selected_profile: Option<String>,
    pub selected_instance: Option<String>,
    pub canceled: bool,
}

impl GothicOrganizerWindow for StartupWindow {
    type Message = Message;
    type Task = Task;

    fn settings(&self) -> ApplicationSettings {
        ApplicationSettings {
            title: "Gothic Organizer: Startup".to_string(),
            width: 340,
            height: 580,
            centered: true,
            resizable: true,
            style: Style::Fluent,
            colors: ColorScheme::Dark2,
            icon: Some("resources/icon.ico".into()),
            ..Default::default()
        }
    }

    fn widgets_mut(&mut self) -> &mut HashMap<String, AnyWidget> {
        &mut self.widgets
    }

    fn populate_ui(&mut self, sender: fltk::app::Sender<Self::Message>, grid: &mut fltk::group::Grid) -> Result<(), GuiError> {
        let profile_choice = self.add_widget(
            "profile_choice",
            Choice::default()
                .with_size(300, 30)
                .with_align(Align::TopLeft)
                .with_label("Profile:"),
        );

        self.profile_choices.iter().for_each(|p| {
            profile_choice.borrow_mut().add_choice(p);
        });

        profile_choice.borrow_mut().set_callback(move |w| {
            let index = w.value();
            sender.send(Message::SelectProfile(index));
        });

        let browse_button = self.add_widget(
            "browse_button",
            Button::default()
                .with_size(300, 30)
                .with_align(Align::Center)
                .with_label("Browse game directory..."),
        );

        browse_button
            .borrow_mut()
            .emit(sender, Message::SelectProfileDirectory);
        browse_button.borrow_mut().deactivate();

        let instance_entry = self.add_widget(
            "instance_entry",
            Input::default()
                .with_size(300, 30)
                .with_align(Align::TopLeft)
                .with_label("New instance name:"),
        );

        instance_entry
            .borrow_mut()
            .set_trigger(CallbackTrigger::EnterKeyChanged);
        instance_entry
            .borrow_mut()
            .set_callback(move |i| sender.send(Message::InputInstanceName(i.value().to_string())));
        instance_entry.borrow_mut().deactivate();

        let instance_selector = self.add_widget(
            "instance_selector",
            HoldBrowser::default()
                .with_size(300, 200)
                .with_align(Align::TopLeft)
                .with_label("Instances:"),
        );

        instance_selector.borrow_mut().set_callback(move |w| {
            let index = w.value();
            sender.send(Message::SelectInstance(index));
        });

        if let Some(available) = &self.instance_choices {
            available.iter().for_each(|i| {
                instance_selector.borrow_mut().add(i);
            });
        }
        instance_selector.borrow_mut().deactivate();

        let add_instance_button = self.add_widget(
            "add_instance_button",
            Button::default()
                .with_size(300, 30)
                .with_align(Align::Center)
                .with_label("Add instance"),
        );

        add_instance_button
            .borrow_mut()
            .emit(sender, Message::AddInstance);
        add_instance_button.borrow_mut().deactivate();

        let remove_instance_button = self.add_widget(
            "remove_instance_button",
            Button::default()
                .with_size(300, 30)
                .with_align(Align::Center)
                .with_label("Remove instance"),
        );

        remove_instance_button
            .borrow_mut()
            .emit(sender, Message::RemoveInstance);
        remove_instance_button.borrow_mut().deactivate();

        let start_button = self.add_widget(
            "start_button",
            Button::default()
                .with_size(300, 30)
                .with_align(Align::Center)
                .with_label("Start"),
        );

        start_button.borrow_mut().emit(sender, Message::Start);
        start_button.borrow_mut().deactivate();

        let cancel_button = self.add_widget(
            "cancel_button",
            Button::default()
                .with_size(300, 30)
                .with_align(Align::Center)
                .with_label("Cancel"),
        );

        cancel_button.borrow_mut().emit(sender, Message::Exit);

        grid.set_widget_ext(
            profile_choice.borrow_mut().deref_mut(),
            0,
            0,
            GridAlign::PROPORTIONAL,
        )?;
        grid.set_widget_ext(
            browse_button.borrow_mut().deref_mut(),
            1,
            0,
            GridAlign::PROPORTIONAL,
        )?;
        grid.set_widget_ext(
            instance_entry.borrow_mut().deref_mut(),
            2,
            0,
            GridAlign::PROPORTIONAL,
        )?;
        grid.set_widget_ext(
            add_instance_button.borrow_mut().deref_mut(),
            3,
            0,
            GridAlign::PROPORTIONAL,
        )?;
        grid.set_widget_ext(
            remove_instance_button.borrow_mut().deref_mut(),
            4,
            0,
            GridAlign::PROPORTIONAL,
        )?;
        grid.set_widget_ext(
            instance_selector.borrow_mut().deref_mut(),
            5,
            0,
            GridAlign::PROPORTIONAL,
        )?;
        grid.set_widget_ext(
            start_button.borrow_mut().deref_mut(),
            6,
            0,
            GridAlign::PROPORTIONAL,
        )?;
        grid.set_widget_ext(
            cancel_button.borrow_mut().deref_mut(),
            7,
            0,
            GridAlign::PROPORTIONAL,
        )?;

        Ok(())
    }

    fn handle_message(&mut self, msg: Self::Message) -> Result<Task, GuiError> {
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
                self.activate_widget("instance_entry")?;
                self.activate_widget("add_instance_button")?;
                self.activate_widget("remove_instance_button")?;
                self.activate_widget("instance_selector")?;
            }
            Message::SelectProfile(index) => {
                let profile = self
                    .profile_choices
                    .get(index as usize)
                    .map(|p| p.to_string());

                self.selected_profile_index = index;
                self.selected_profile = profile;
                self.activate_widget("browse_button")?;
            }
            Message::SelectInstance(index) => {
                self.selected_instance_index = index;
                if let Some(available) = &self.instance_choices {
                    if index != 0 && index - 1 < available.len() as i32 && !available.is_empty() {
                        let instance = available.get((index as usize).saturating_sub(1)).cloned();
                        self.selected_instance = instance;
                    }
                }
                self.activate_widget("start_button")?;
            }
            Message::InputInstanceName(name) => {
                self.instance_name_input = name;
            }
            Message::AddInstance => {
                if self.instance_name_input.clone().is_empty() {
                    return Ok(Task::None);
                }

                if let Some(available) = &mut self.instance_choices {
                    if available.contains(&self.instance_name_input) {
                        return Ok(Task::None);
                    }

                    available.push(self.instance_name_input.clone());
                } else {
                    self.instance_choices = Some(vec![self.instance_name_input.clone()]);
                }

                if let AnyWidget::HoldBrowser(instance_selector) = self
                    .widgets
                    .get("instance_selector")
                    .ok_or(GuiError::WidgetNotFound("instance_selector".to_owned()))?
                {
                    instance_selector
                        .borrow_mut()
                        .add(&self.instance_name_input);
                }

                self.current_profile
                    .as_mut()
                    .unwrap()
                    .add_instance(Instance::new(self.instance_name_input.clone()));
                save_profile!(self.current_profile.clone().unwrap())?;
            }

            Message::RemoveInstance => {
                let index = self.selected_instance_index;
                let Some(selected_instance_name) = self.selected_instance.clone() else {
                    return Ok(Task::None);
                };

                if let Some(available) = &mut self.instance_choices {
                    if index != 0 && index - 1 < available.len() as i32 && !available.is_empty() {
                        available.remove((index as usize).saturating_sub(1));
                        if let AnyWidget::HoldBrowser(instance_selector) = self
                            .widgets
                            .get("instance_selector")
                            .ok_or(GuiError::WidgetNotFound("instance_selector".to_owned()))?
                        {
                            instance_selector.borrow_mut().remove(index);
                        }
                    }

                    if available.is_empty() {
                        self.deactivate_widget("start_button")?;
                        self.instance_choices = None;
                    }
                }

                self.current_profile
                    .as_mut()
                    .unwrap()
                    .remove_instance(selected_instance_name.clone());

                read_dir(local_instances!(self.selected_profile.clone().unwrap()))?.find_map(|d| {
                    d.ok().and_then(|e| {
                        if e.file_name().to_string_lossy().to_lowercase() == selected_instance_name.to_lowercase() {
                            remove_dir_all(e.path()).ok()
                        } else {
                            None
                        }
                    })
                });
            }
            Message::Start => {
                return Ok(Task::CloseWindow);
            }
            Message::Exit => {
                self.canceled = true;
                return Ok(Task::CloseWindow);
            }
        }
        Ok(Task::None)
    }

    fn event_loop(
        &mut self,
        app: &mut fltk::app::App,
        window: &mut fltk::window::Window,
        receiver: fltk::app::Receiver<<Self as GothicOrganizerWindow>::Message>,
    ) -> Result<(), GuiError> {
        while app.wait() {
            if let Some(msg) = receiver.recv() {
                match self.handle_message(msg)? {
                    Task::CloseWindow => {
                        window.hide();
                        break;
                    }
                    Task::None => (),
                }
            }
        }

        Ok(())
    }
}

impl StartupWindow {
    pub fn new() -> Self {
        Self {
            canceled: false,
            profile_choices: game_profile_list().to_vec(),
            ..Default::default()
        }
    }

    pub fn _with_instances(instances: Option<Vec<String>>) -> Self {
        let mut sw = Self::new();
        sw.instance_choices = instances;
        sw
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectProfileDirectory,
    SelectProfile(i32),
    SelectInstance(i32),
    RemoveInstance,
    InputInstanceName(String),
    AddInstance,
    Start,
    Exit,
}

#[derive(Debug, Clone)]
pub enum Task {
    CloseWindow,
    None,
}

pub mod prelude {
    pub use crate::application::GothicOrganizerWindow;
    pub use crate::startup_window::StartupWindow;
}
