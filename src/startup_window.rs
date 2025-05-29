use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::fs::read_dir;
use std::fs::remove_dir_all;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::rc::Rc;

use fltk::browser::HoldBrowser;
use fltk::button::Button;
use fltk::dialog::NativeFileChooser;
use fltk::dialog::NativeFileChooserType;
use fltk::enums::Align;
use fltk::enums::CallbackTrigger;
use fltk::group::GridAlign;
use fltk::input::Input;
use fltk::menu::Choice;
use fltk::prelude::*;

use crate::application::AnyWidget;
use crate::application::ApplicationSettings;
use crate::application::GothicOrganizerWindow;
use crate::constants::game_profile_list;
use crate::constants::ColorScheme;
use crate::constants::Style;
use crate::error::GuiError;
use crate::impl_widget_name_enum;
use crate::local_instances;
use crate::profile::Instance;
use crate::profile::Profile;
use crate::save_profile;

/// A shortcut for a `Rc<RefCell<T>>`
type MutRc<T> = Rc<RefCell<T>>;

#[derive(Default)]
pub struct StartupWindow {
    widgets: HashMap<WidgetName, AnyWidget>,
    profiles: MutRc<Vec<MutRc<Profile>>>,
    instances: MutRc<Vec<MutRc<Instance>>>,
    current_profile: MutRc<Profile>,
    instance_name_input: String,
    selected_profile_index: i32,
    selected_instance_index: i32,
    pub canceled: bool,
}

impl GothicOrganizerWindow for StartupWindow {
    type Message = Message;
    type Task = Task;
    type WidgetName = WidgetName;

    fn settings(&self) -> ApplicationSettings {
        ApplicationSettings::new()
            .with_title("Gothic Organizer: Startup")
            .with_width(340)
            .with_height(580)
            .with_style(Style::Fluent)
            .with_colors(ColorScheme::Dark2)
            .with_icon("resources/icon.ico")
            .centered()
            .resizable()
    }

    fn widgets(&self) -> &HashMap<WidgetName, AnyWidget> {
        &self.widgets
    }

    fn widgets_mut(&mut self) -> &mut HashMap<WidgetName, AnyWidget> {
        &mut self.widgets
    }

    fn populate_ui(&mut self, sender: fltk::app::Sender<Self::Message>, layout: &mut crate::application::AnyGroup) -> Result<(), GuiError> {
        let profile_choice = self.add_widget(
            WidgetName::ProfileChoice,
            Choice::default()
                .with_size(300, 30)
                .with_align(Align::TopLeft)
                .with_label("Profile:"),
        );

        {
            let mut profile_choice = profile_choice.borrow_mut();
            self.profiles.borrow().iter().for_each(|p| {
                profile_choice.add_choice(&p.borrow().name);
            });
            profile_choice.set_callback(move |w| {
                let index = w.value();
                sender.send(Message::SelectProfile(index));
            });
        }

        let browse_button = self.add_widget(
            WidgetName::BrowseButton,
            Button::default()
                .with_size(300, 30)
                .with_align(Align::Center)
                .with_label("Browse game directory..."),
        );

        {
            let mut browse_button = browse_button.borrow_mut();
            browse_button.emit(sender, Message::SelectProfileDirectory);
            browse_button.deactivate();
        }

        let instance_entry = self.add_widget(
            WidgetName::InstanceEntry,
            Input::default()
                .with_size(300, 30)
                .with_align(Align::TopLeft)
                .with_label("New instance name:"),
        );

        {
            let mut instance_entry = instance_entry.borrow_mut();
            instance_entry.set_trigger(CallbackTrigger::EnterKeyChanged);
            instance_entry.set_callback(move |i| sender.send(Message::InputInstanceName(i.value().to_string())));
            instance_entry.deactivate();
        }

        let instance_selector = self.add_widget(
            WidgetName::InstanceSelector,
            HoldBrowser::default()
                .with_size(300, 200)
                .with_align(Align::TopLeft)
                .with_label("Instances:"),
        );

        {
            let mut instance_selector = instance_selector.borrow_mut();
            self.instances.borrow().iter().for_each(|i| {
                instance_selector.add(&i.borrow().name);
            });
            instance_selector.set_callback(move |w| {
                let index = w.value();
                sender.send(Message::SelectInstance(index));
            });
            instance_selector.deactivate();
        }

        let add_instance_button = self.add_widget(
            WidgetName::AddInstanceButton,
            Button::default()
                .with_size(300, 30)
                .with_align(Align::Center)
                .with_label("Add instance"),
        );

        {
            let mut add_instance_button = add_instance_button.borrow_mut();
            add_instance_button.emit(sender, Message::AddInstance);
            add_instance_button.deactivate();
        }

        let remove_instance_button = self.add_widget(
            WidgetName::RemoveInstanceButton,
            Button::default()
                .with_size(300, 30)
                .with_align(Align::Center)
                .with_label("Remove instance"),
        );

        {
            let mut remove_instance_button = remove_instance_button.borrow_mut();
            remove_instance_button.emit(sender, Message::RemoveInstance);
            remove_instance_button.deactivate();
        }

        let start_button = self.add_widget(
            WidgetName::StartButton,
            Button::default()
                .with_size(300, 30)
                .with_align(Align::Center)
                .with_label("Start"),
        );

        {
            let mut start_button = start_button.borrow_mut();
            start_button.emit(sender, Message::Start);
            start_button.deactivate();
        }

        let cancel_button = self.add_widget(
            WidgetName::CancelButton,
            Button::default()
                .with_size(300, 30)
                .with_align(Align::Center)
                .with_label("Cancel"),
        );

        cancel_button.borrow_mut().emit(sender, Message::Exit);

        if let Some(mut grid_layout) = layout.as_grid_mut() {
            grid_layout.set_widget_ext(
                profile_choice.borrow_mut().deref_mut(),
                0,
                0,
                GridAlign::PROPORTIONAL,
            )?;
            grid_layout.set_widget_ext(
                browse_button.borrow_mut().deref_mut(),
                1,
                0,
                GridAlign::PROPORTIONAL,
            )?;
            grid_layout.set_widget_ext(
                instance_entry.borrow_mut().deref_mut(),
                2,
                0,
                GridAlign::PROPORTIONAL,
            )?;
            grid_layout.set_widget_ext(
                add_instance_button.borrow_mut().deref_mut(),
                3,
                0,
                GridAlign::PROPORTIONAL,
            )?;
            grid_layout.set_widget_ext(
                remove_instance_button.borrow_mut().deref_mut(),
                4,
                0,
                GridAlign::PROPORTIONAL,
            )?;
            grid_layout.set_widget_ext(
                instance_selector.borrow_mut().deref_mut(),
                5,
                0,
                GridAlign::PROPORTIONAL,
            )?;
            grid_layout.set_widget_ext(
                start_button.borrow_mut().deref_mut(),
                6,
                0,
                GridAlign::PROPORTIONAL,
            )?;
            grid_layout.set_widget_ext(
                cancel_button.borrow_mut().deref_mut(),
                7,
                0,
                GridAlign::PROPORTIONAL,
            )?;
        }

        Ok(())
    }

    fn handle_message(&mut self, msg: Self::Message) -> Result<Task, GuiError> {
        match msg {
            Message::Start => {
                return Ok(Task::CloseWindow);
            }

            Message::Exit => {
                self.canceled = true;
                return Ok(Task::CloseWindow);
            }

            Message::InputInstanceName(name) => {
                self.instance_name_input = name;
            }

            Message::SelectProfile(index) => {
                let profiles_clone = self.profiles.clone();
                let profiles_borrowed = profiles_clone.borrow();
                let Some(profile) = profiles_borrowed.get(index as usize) else {
                    return Ok(Task::None);
                };

                self.selected_profile_index = index;
                self.current_profile = profile.clone();
                self.activate_widget(&WidgetName::BrowseButton)?;
            }

            Message::SelectInstance(index) => {
                if index != 0 && index - 1 < self.instances.borrow().len() as i32 && !self.instances.borrow().is_empty() {
                    self.selected_instance_index = index;
                }

                self.activate_widget(&WidgetName::StartButton)?;
            }

            Message::SelectProfileDirectory => {
                let mut file_dialog = NativeFileChooser::new(NativeFileChooserType::BrowseDir);
                file_dialog.set_title("Select game directory");
                file_dialog.show();

                match file_dialog.filename() {
                    dir if dir.exists() => {
                        self.current_profile.borrow_mut().game_path = dir;

                        self.activate_widget(&WidgetName::InstanceEntry)?;
                        self.activate_widget(&WidgetName::AddInstanceButton)?;
                        self.activate_widget(&WidgetName::RemoveInstanceButton)?;
                        self.activate_widget(&WidgetName::InstanceSelector)?;
                    }
                    _ => {
                        self.deactivate_widget(&WidgetName::InstanceEntry)?;
                        self.deactivate_widget(&WidgetName::AddInstanceButton)?;
                        self.deactivate_widget(&WidgetName::RemoveInstanceButton)?;
                        self.deactivate_widget(&WidgetName::InstanceSelector)?;
                    }
                }
            }

            Message::AddInstance => {
                if self.instance_name_input.clone().is_empty() {
                    return Ok(Task::None);
                }

                if !self.instances.borrow().is_empty() {
                    let mut instances_borrowed = self.instances.borrow_mut();

                    match instances_borrowed
                        .iter()
                        .find(|i| i.borrow().name.to_lowercase() == self.instance_name_input.to_lowercase())
                    {
                        Some(_) => {
                            return Ok(Task::None);
                        }
                        None => {
                            instances_borrowed.push(Rc::new(RefCell::new(Instance::new(
                                &self.instance_name_input,
                            ))));
                        }
                    }
                } else {
                    self.instances.replace_with(|_| {
                        vec![Rc::new(RefCell::new(Instance::new(
                            &self.instance_name_input,
                        )))]
                    });
                }

                self.current_profile
                    .borrow_mut()
                    .add_instance(Instance::new(&self.instance_name_input));

                if let AnyWidget::HoldBrowser(instance_selector) = self.widget(&WidgetName::InstanceSelector)? {
                    instance_selector
                        .borrow_mut()
                        .add(&self.instance_name_input);
                }

                save_profile!(self.current_profile.borrow_mut().clone())?;
            }

            Message::RemoveInstance => {
                let index = self.selected_instance_index;

                let AnyWidget::HoldBrowser(instance_selector) = self.widget(&WidgetName::InstanceSelector)? else {
                    return Ok(Task::None);
                };

                let available = self.instances.clone();

                let selected_instance_name = match available.borrow().get(index.saturating_sub(1) as usize) {
                    Some(instance) => instance.borrow().name.clone(),
                    None => return Ok(Task::None),
                };

                if index != 0 && index - 1 < available.borrow().len() as i32 && !available.borrow().is_empty() {
                    available
                        .borrow_mut()
                        .remove((index as usize).saturating_sub(1));
                    instance_selector.borrow_mut().remove(index);
                }

                if available.borrow().is_empty() {
                    self.deactivate_widget(&WidgetName::StartButton)?;
                    self.instances = Rc::new(RefCell::new(Vec::new()));
                }

                self.current_profile
                    .borrow_mut()
                    .remove_instance(&selected_instance_name);

                read_dir(local_instances!(&self.current_profile.borrow().name))?.find_map(|d| {
                    d.ok().and_then(|e| {
                        if e.file_name().to_string_lossy().to_lowercase() == selected_instance_name.to_lowercase() {
                            remove_dir_all(e.path()).ok()
                        } else {
                            None
                        }
                    })
                });
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
        let profiles = game_profile_list()
            .iter()
            .map(|p| Rc::new(RefCell::new(Profile::new(p, PathBuf::new()))))
            .collect::<Vec<MutRc<Profile>>>();

        let profiles = Rc::new(RefCell::new(profiles));

        Self {
            canceled: false,
            profiles,
            ..Default::default()
        }
    }

    pub fn selected_profile(&self) -> Option<Rc<RefCell<Profile>>> {
        self.profiles
            .borrow()
            .get(self.selected_profile_index as usize)
            .cloned()
    }

    pub fn selected_instance(&self) -> Option<Rc<RefCell<Instance>>> {
        self.instances
            .borrow()
            .get(self.selected_instance_index.saturating_sub(1) as usize)
            .cloned()
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

impl_widget_name_enum!(
    ProfileSelector,
    ProfileChoice,
    InstanceSelector,
    InstanceEntry,
    AddInstanceButton,
    RemoveInstanceButton,
    StartButton,
    BrowseButton,
    CancelButton,
);

pub mod prelude {
    pub use crate::application::GothicOrganizerWindow;
    pub use crate::startup_window::StartupWindow;
}
