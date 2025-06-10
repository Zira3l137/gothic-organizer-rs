use fltk::enums::CallbackTrigger;
use fltk::tree::TreeItem;
use fltk::tree::TreeReason;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use fltk::button::Button;
use fltk::enums::Align;
use fltk::enums::FrameType;
use fltk::group::Flex;
use fltk::group::GridAlign;
use fltk::input::Input;
use fltk::menu::Choice;
use fltk::prelude::*;
use fltk::tree::Tree;
use fltk::tree::TreeSelect;
use fltk::tree::TreeSort;

use crate::application::AnyWidget;
use crate::application::ApplicationSettings;
use crate::application::GothicOrganizerWindow;
use crate::constants::game_profile_list;
use crate::constants::ColorScheme;
use crate::constants::Style;
use crate::error::GuiError;
use crate::impl_widget_name_enum;
use crate::load_profile;
use crate::profile::FileNode;
use crate::profile::Instance;
use crate::profile::ModInfo;
use crate::profile::Profile;
use crate::profile::Session;

/// A shortcut for a `Rc<RefCell<T>>`
type MutRc<T> = Rc<RefCell<T>>;

#[derive(Default)]
pub struct EditorWindow {
    previous_session: Session,
    widgets: HashMap<WidgetName, AnyWidget>,
    profiles: MutRc<Vec<MutRc<Profile>>>,
    instances: MutRc<Vec<MutRc<Instance>>>,
    current_profile: MutRc<Profile>,
    current_instance: MutRc<Instance>,
    file_tree_cache: IndexMap<PathBuf, FileNode>,
    _selected_file_index: i32,
    _selected_profile_index: i32,
    _selected_instance_index: i32,
}

impl GothicOrganizerWindow for EditorWindow {
    type Message = Message;
    type Task = Task;
    type WidgetName = WidgetName;

    fn settings(&self) -> ApplicationSettings {
        ApplicationSettings::new()
            .with_title("Gothic Organizer: Editor")
            .with_width(768)
            .with_height(768)
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
        let wnd_settings = self.settings();
        let profile_group = self.add_widget(
            WidgetName::ProfileGroup,
            Flex::default()
                .row()
                .with_align(Align::Left)
                .with_size(wnd_settings.width - 20, wnd_settings.height / 12),
        );

        {
            let mut profile_group = profile_group.borrow_mut();
            profile_group.set_frame(FrameType::DownBox);
            profile_group.set_spacing(5);
            profile_group.set_margins(10, 20, 10, 10);
            profile_group.begin();

            let profile_choice = self.add_widget(
                WidgetName::ProfileChoice,
                Choice::default()
                    .with_size(100, 30)
                    .with_align(Align::TopLeft)
                    .with_label("Profile:"),
            );

            {
                let mut profile_choice = profile_choice.borrow_mut();
                self.profiles.borrow().iter().for_each(|p| {
                    let profile_name = &p.borrow().name;
                    profile_choice.add_choice(profile_name);
                });
                let selected_profile_index = self.previous_session.selected_profile.unwrap_or(0);
                profile_choice.set_value(selected_profile_index);
                self.current_profile = self.profiles.borrow()[selected_profile_index as usize].clone();
                profile_choice.set_callback(move |w| {
                    let index = w.value();
                    sender.send(Message::SelectProfile(index));
                });
            }

            let instance_choice = self.add_widget(
                WidgetName::InstanceChoice,
                Choice::default()
                    .with_size(100, 30)
                    .with_align(Align::TopLeft)
                    .with_label("Instance:"),
            );

            {
                let mut instance_choice = instance_choice.borrow_mut();
                self.instances.borrow().iter().for_each(|i| {
                    let instance_name = &i.borrow().name;
                    instance_choice.add_choice(instance_name);
                });
                let selected_instance_index = self.previous_session.selected_instance.unwrap_or(0);
                instance_choice.set_value(selected_instance_index);
                self.current_instance = self.instances.borrow()[selected_instance_index as usize].clone();
                instance_choice.set_callback(move |w| {
                    let index = w.value();
                    sender.send(Message::SelectInstance(index));
                });
            }

            let add_instance_button = self.add_widget(
                WidgetName::AddInstanceButton,
                Button::default()
                    .with_size(30, 30)
                    .with_align(Align::Center)
                    .with_label("+"),
            );

            {
                let mut add_instance_button = add_instance_button.borrow_mut();
                add_instance_button.emit(sender, Message::AddInstance);
            }

            let remove_instance_button = self.add_widget(
                WidgetName::RemoveInstanceButton,
                Button::default()
                    .with_size(30, 30)
                    .with_align(Align::Center)
                    .with_label("-"),
            );

            {
                let mut remove_instance_button = remove_instance_button.borrow_mut();
                remove_instance_button.emit(sender, Message::RemoveInstance);
            }

            let run_button = self.add_widget(
                WidgetName::RunButton,
                Button::default()
                    .with_size(30, 30)
                    .with_align(Align::Center)
                    .with_label("Run"),
            );

            {
                let mut run_button = run_button.borrow_mut();
                run_button.emit(sender, Message::Run);
            }

            let settings_button = self.add_widget(
                WidgetName::SettingsButton,
                Button::default()
                    .with_size(30, 30)
                    .with_align(Align::Center)
                    .with_label("Settings"),
            );

            {
                let mut settings_button = settings_button.borrow_mut();
                settings_button.emit(sender, Message::Settings);
            }

            profile_group.end();

            let main_group = self.add_widget(
                WidgetName::MainGroup,
                Flex::default().row().with_align(Align::Left).with_size(
                    wnd_settings.width - 20,
                    wnd_settings.height - wnd_settings.height / 7,
                ),
            );

            let mut main_group = main_group.borrow_mut();
            main_group.set_frame(FrameType::DownBox);
            main_group.set_spacing(5);
            main_group.set_margins(10, 20, 10, 10);
            main_group.begin();

            let mods_group = self.add_widget(
                WidgetName::ModsGroup,
                Flex::default().column().with_align(Align::Left).with_size(
                    (wnd_settings.width / 2) - 20,
                    wnd_settings.height - wnd_settings.height / 7,
                ),
            );

            let mut mods_group = mods_group.borrow_mut();
            mods_group.set_spacing(5);
            mods_group.set_frame(FrameType::DownBox);
            mods_group.begin();

            mods_group.end();

            let files_group = self.add_widget(
                WidgetName::FilesGroup,
                Flex::default().column().with_align(Align::Right).with_size(
                    (wnd_settings.width / 2) - 20,
                    wnd_settings.height - wnd_settings.height / 7,
                ),
            );

            let mut files_group = files_group.borrow_mut();
            files_group.set_spacing(15);
            files_group.set_margin(10);
            files_group.set_frame(FrameType::DownBox);
            files_group.begin();

            let file_controls_group = self.add_widget(
                WidgetName::FileControlsGroup,
                Flex::default().row().with_align(Align::Left),
            );

            {
                let mut file_controls_group = file_controls_group.borrow_mut();
                file_controls_group.set_spacing(5);
                file_controls_group.set_frame(FrameType::DownBox);
                file_controls_group.begin();

                let view_overrides_button = self.add_widget(
                    WidgetName::ViewOverridesButton,
                    Button::default()
                        .with_align(Align::Center)
                        .with_label("View Overrides"),
                );

                let include_button = self.add_widget(
                    WidgetName::IncludeButton,
                    Button::default()
                        .with_align(Align::Center)
                        .with_label("Include"),
                );

                let exclude_button = self.add_widget(
                    WidgetName::ExcludeButton,
                    Button::default()
                        .with_align(Align::Center)
                        .with_label("Exclude"),
                );

                file_controls_group.end();

                let files_tree = self.add_widget(WidgetName::FilesList, Tree::default_fill());

                {
                    let mut files_tree = files_tree.borrow_mut();

                    let current_instance = self.current_instance.borrow();
                    let game_dir: &PathBuf = &self.current_profile.borrow().game_path;

                    let instance_files = &current_instance.files;
                    let mod_files = &current_instance.mods;

                    if let Some(files) = instance_files {
                        let nodes = assume_target_structure(files, mod_files.as_deref(), game_dir)?;
                        self.file_tree_cache = nodes.clone();
                        let tree_items = prepare_tree_items(&files_tree, game_dir, &nodes)?;
                        populate_file_tree(&mut files_tree, &tree_items)?;
                    }

                    files_tree.set_root_label(&game_dir.file_stem().unwrap_or_default().to_string_lossy());
                    files_tree.set_sort_order(TreeSort::None);
                    files_tree.set_select_mode(TreeSelect::Multi);
                    files_tree.set_callback(move |w| {
                        if let TreeReason::Selected = w.callback_reason() {
                            let mut selected_file_names: Vec<String> = Vec::new();

                            let Some(selected_items) = w.get_selected_items() else {
                                return;
                            };

                            selected_items.iter().for_each(|item| {
                                collect_children_recursively(item, &mut selected_file_names);
                                selected_file_names.iter().for_each(|item_name| {
                                    let _ = w.select(item_name, false);
                                    if !w.is_open(item_name) {
                                        let _ = w.open(item_name, false);
                                    }
                                });
                            });

                            sender.send(Message::FilesSelected(selected_file_names));
                        }
                    });
                }

                let search_bar = self.add_widget(
                    WidgetName::SearchBar,
                    Input::default()
                        .with_label("Search")
                        .with_align(Align::TopLeft),
                );

                {
                    let mut search_bar = search_bar.borrow_mut();
                    files_group.fixed(search_bar.deref_mut(), 30);

                    search_bar.set_trigger(CallbackTrigger::EnterKey);
                    search_bar.set_callback(move |w| {
                        let search_term = w.value();
                        sender.send(Message::SearchForItem(search_term));
                    });
                }
            }

            files_group.end();
            files_group.fixed(file_controls_group.borrow_mut().deref_mut(), 30);
            main_group.end();

            if let Some(mut grid_layout) = layout.as_grid_mut() {
                grid_layout.set_layout(6, 1);
                grid_layout.set_widget_ext(profile_group.deref_mut(), 0, 0, GridAlign::PROPORTIONAL)?;
                grid_layout.set_widget_ext(main_group.deref_mut(), 1, 0, GridAlign::PROPORTIONAL)?;
            }
        }

        Ok(())
    }

    fn handle_message(&mut self, msg: Self::Message) -> Result<Task, GuiError> {
        match msg {
            Message::_Exit => {
                return Ok(Task::CloseWindow);
            }

            Message::Run => {
                todo!()
            }

            Message::Settings => {
                todo!()
            }

            Message::SelectProfile(_index) => {
                todo!()
            }

            Message::SelectInstance(_index) => {
                todo!()
            }

            Message::_SelectProfileDirectory => {
                todo!()
            }

            Message::AddInstance => {
                todo!()
            }

            Message::RemoveInstance => {
                todo!()
            }

            Message::FilesSelected(_files) => {}

            Message::SearchForItem(term) => {
                if let AnyWidget::Tree(files_tree) = self.widget(&WidgetName::FilesList)? {
                    let mut files_tree = files_tree.borrow_mut();

                    if term.is_empty() {
                        files_tree.clear();
                        let new_items = prepare_tree_items(
                            &files_tree,
                            &self.current_profile.borrow().game_path,
                            &self.file_tree_cache,
                        )?;
                        populate_file_tree(&mut files_tree, &new_items)?;
                        return Ok(Task::None);
                    }

                    let found_nodes = self
                        .file_tree_cache
                        .iter()
                        .filter_map(|(path, node)| {
                            let name = &node.name;
                            if name.to_lowercase().contains(&term.to_lowercase()) {
                                Some((path.clone(), node.clone()))
                            } else {
                                None
                            }
                        })
                        .collect::<IndexMap<PathBuf, FileNode>>();

                    if found_nodes.is_empty() {
                        files_tree.clear();
                        files_tree.redraw();
                        return Ok(Task::None);
                    }

                    let new_items = prepare_tree_items(
                        &files_tree,
                        &self.current_profile.borrow().game_path,
                        &found_nodes,
                    )?;

                    populate_file_tree(&mut files_tree, &new_items)?;
                };
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

impl EditorWindow {
    pub fn new(session: Session) -> Self {
        let profiles = session
            .as_ref()
            .available_profiles
            .as_ref()
            .unwrap_or(&game_profile_list().to_vec())
            .iter()
            .map(|p| {
                Rc::new(RefCell::new(match load_profile!(p) {
                    Some(p) => p,
                    None => Profile::new(p, PathBuf::new()),
                }))
            })
            .collect::<Vec<MutRc<Profile>>>();

        let profiles = Rc::new(RefCell::new(profiles));

        let instances = profiles
            .borrow()
            .get(session.selected_profile.unwrap() as usize)
            .unwrap()
            .borrow()
            .instances
            .clone()
            .unwrap()
            .iter()
            .map(|i| Rc::new(RefCell::new(i.clone())))
            .collect::<Vec<MutRc<Instance>>>();

        let instances = Rc::new(RefCell::new(instances));

        let previous_session = session.clone();

        Self {
            profiles,
            instances,
            previous_session,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectProfile(i32),
    SelectInstance(i32),
    _SelectProfileDirectory,
    FilesSelected(Vec<String>),
    SearchForItem(String),
    RemoveInstance,
    AddInstance,
    Run,
    Settings,
    _Exit,
}

#[derive(Debug, Clone)]
pub enum Task {
    CloseWindow,
    None,
}

impl_widget_name_enum!(
    ProfileGroup,
    MainGroup,
    ModsGroup,
    FilesGroup,
    FileControlsGroup,
    IncludeButton,
    ExcludeButton,
    ViewOverridesButton,
    SearchBar,
    FilesList,
    ProfileChoice,
    InstanceChoice,
    BrowseButton,
    AddInstanceButton,
    RemoveInstanceButton,
    RunButton,
    SettingsButton,
);

fn collect_children_recursively(item: &TreeItem, storage: &mut Vec<String>) {
    let item_path = get_tree_item_path(item);
    if item.has_children() {
        storage.push(item_path);
        for i in 0..item.children() {
            let Some(child) = item.child(i) else {
                continue;
            };

            collect_children_recursively(&child, storage);
        }
    } else {
        storage.push(item_path);
    }
}

fn get_tree_item_path(item: &TreeItem) -> String {
    let mut path = Vec::new();
    let mut current = Some(item.clone());

    while let Some(it) = current {
        if let Some(label) = it.label() {
            path.push(label);
        } else {
            break;
        }
        current = it.parent();
    }

    path.reverse();
    path.join("/")
}

fn populate_file_tree(tree: &mut Tree, items: &IndexMap<String, TreeItem>) -> Result<(), GuiError> {
    tree.clear();
    items.iter().for_each(|(path, item)| {
        let tree_item = item.clone();
        tree.add_item(path, &tree_item);
    });
    tree.redraw();
    Ok(())
}

fn prepare_tree_items(tree: &Tree, root: &Path, nodes: &IndexMap<PathBuf, FileNode>) -> Result<IndexMap<String, TreeItem>, GuiError> {
    let mut tree_items: IndexMap<String, TreeItem> = IndexMap::with_capacity(nodes.len());

    for (path, node) in nodes {
        let target_path = path
            .strip_prefix(root)?
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("/");

        tree_items.insert(target_path.clone(), node.clone().into_tree_item(tree));
    }

    Ok(tree_items)
}

fn assume_target_structure(files: &[FileNode], mods: Option<&[ModInfo]>, root: &Path) -> Result<IndexMap<PathBuf, FileNode>, GuiError> {
    let base_root = root.to_path_buf();

    let mut target_file_tree: Vec<&FileNode> = files.iter().collect();
    target_file_tree.sort_by_key(|f| (!f.is_dir(), f.length()));

    let mut base: IndexMap<PathBuf, FileNode> = IndexMap::with_capacity(target_file_tree.len());

    for f in target_file_tree {
        base.insert(f.path.clone(), f.clone());
    }

    if let Some(mods) = mods {
        mods.iter().filter(|m| m.config.enabled).for_each(|m| {
            let Some(mod_files) = &m.config.files else {
                return;
            };

            mod_files.iter().for_each(|f| {
                let Ok(relative_path) = f.path.strip_prefix(&m.path) else {
                    return;
                };

                let destination_path = base_root.join(relative_path);
                base.entry(destination_path)
                    .and_modify(|existing| {
                        existing.override_by(f.name.clone(), f.path.clone());
                    })
                    .or_insert_with(|| f.clone());
            });
        });
    }

    Ok(base)
}

pub mod prelude {
    pub use crate::application::GothicOrganizerWindow;
    pub use crate::editor::EditorWindow;
}
