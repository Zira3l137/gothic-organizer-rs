use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::widget::button;
use iced::widget::checkbox;
use iced::widget::column;
use iced::widget::combo_box;
use iced::widget::container;
use iced::widget::horizontal_space;
use iced::widget::image;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::text;
use iced::widget::Column;
use iced::widget::Row;
use iced::Element;
use iced::Length;

use crate::core::constants::app_title_full;
use crate::gui::app::Message;
use crate::gui::custom_widgets::clickable_text::ClickableText;
use crate::styled_container;
use crate::svg_with_color;

pub fn editor_view(app: &crate::gui::app::GothicOrganizer) -> Element<Message> {
    /////////////////////////[States]/////////////////////////////
    let current_profile = app
        .profile_selected
        .as_ref()
        .and_then(|s| app.profiles.get(s));

    let instance_selected = app
        .instance_selected
        .as_ref()
        .and_then(|s| current_profile.and_then(|p| p.instances.as_ref().and_then(|i| i.get(s))));

    //////////////////////////////////////////////////////////////
    /////////////////////////[Top Header]/////////////////////////
    let logo = image("./resources/icon.ico");

    let mut title = text!("{}", app_title_full());
    title = title.align_y(Vertical::Center);
    title = title.align_x(Horizontal::Left);
    title = title.size(30);

    let header: Row<_> = row!(logo, title).spacing(10);
    //////////////////////////////////////////////////////////////
    /////////////////////////[Top Profile Controls]///////////////
    let choice_profile = combo_box(
        &app.state.profile_choices,
        "Profile",
        app.profile_selected.as_ref(),
        Message::ProfileSelected,
    );

    let choice_instance = combo_box(
        &app.state.instance_choices,
        "Instance",
        app.instance_selected.as_ref(),
        Message::InstanceSelected,
    )
    .on_input(Message::InstanceInput);

    let button_add: button::Button<Message> = button("Add").on_press_maybe(app.profile_selected.as_ref().and_then(|s| {
        let profile = app.profiles.get(s)?;
        if profile.path.display().to_string() != "" {
            Some(Message::InstanceAdd(s.clone()))
        } else {
            None
        }
    }));

    let button_remove = button("Remove").on_press_maybe(app.profile_selected.as_ref().and_then(|s| {
        let profile = app.profiles.get(s)?;
        if profile.path.display().to_string() != "" {
            Some(Message::InstanceRemove(s.clone()))
        } else {
            None
        }
    }));

    let button_browse = |profile: &crate::core::profile::Profile| button("Browse").on_press(Message::BrowseGameDir(profile.name.clone()));
    let group_instance = container(row!(choice_instance, button_add, button_remove));

    let profile_controls = styled_container!(
        row!(
            choice_profile,
            match current_profile {
                Some(profile) if profile.path.display().to_string() == "" => {
                    let button_browse: Element<_> = button_browse(profile).into();
                    button_browse
                }
                Some(profile) if profile.path.display().to_string() != "" => {
                    group_instance.into()
                }
                _ => {
                    horizontal_space().into()
                }
            }
        )
        .spacing(10),
        border_width = 4.0,
        border_radius = 8.0
    )
    .padding(10)
    .center_x(Length::Fill);

    //////////////////////////////////////////////////////////////
    /////////////////////////[Right-side File List]///////////////

    let icon_back = svg_with_color!("./resources/back.svg").height(20).width(20);
    let button_back_message = current_profile.and_then(|profile| {
        if profile.path == app.state.current_directory {
            return None;
        };
        Some(Message::TraverseIntoDir(
            app.state
                .current_directory
                .clone()
                .parent()
                .unwrap_or(profile.path.as_ref())
                .to_path_buf(),
        ))
    });
    let button_back = button(icon_back).on_press_maybe(button_back_message);

    let icon_home = svg_with_color!("./resources/home.svg").height(20).width(20);
    let button_home_message = current_profile.and_then(|profile| {
        if profile.path == app.state.current_directory {
            return None;
        };
        Some(Message::TraverseIntoDir(profile.path.clone()))
    });
    let button_home = button(icon_home).on_press_maybe(button_home_message);

    let controls_files = styled_container!(
        row!(button_back, button_home).spacing(10),
        border_width = 1.0,
        border_radius = 4.0
    )
    .padding(10)
    .center_x(Length::Fill);

    let mut column_files: Column<_> = Column::new();

    if instance_selected.is_some() {
        column_files = app
            .state
            .current_directory_entries
            .iter()
            .fold(Column::new(), |column, (path, enabled)| {
                let is_dir = path.is_dir();

                let file_name = path.file_name().unwrap().to_string_lossy().to_string();

                let label: Element<_> = match &is_dir {
                    true => ClickableText::new(file_name, Message::TraverseIntoDir(path.clone())).into(),
                    false => text(file_name).into(),
                };

                let icon_path = match &is_dir {
                    true => "./resources/directory.svg",
                    false => "./resources/file.svg",
                };

                let icon: Element<_> = svg_with_color!(icon_path).height(20).width(20).into();

                let checkbox = checkbox("", *enabled).on_toggle(move |new_state| Message::FileToggle(path.clone(), new_state));

                let group_file = styled_container!(
                    row![checkbox, icon, label],
                    border_width = 1.0,
                    border_radius = 4.0
                )
                .padding(5)
                .align_left(Length::Fill);

                column.push(group_file)
            });
    }

    let files_menu = styled_container!(
        column!(controls_files, scrollable(column_files)).spacing(10),
        border_width = 4.0,
        border_radius = 8.0
    )
    .padding(10)
    .align_y(Vertical::Top)
    .center_x(Length::Fill);

    //////////////////////////////////////////////////////////////
    /////////////////////////[Left-side Mods List]////////////////
    // TODO: Implement mod list
    let mods_menu = styled_container!(
        column!(text("mods menu")),
        border_width = 4.0,
        border_radius = 8.0
    )
    .center(Length::Fill);

    //////////////////////////////////////////////////////////////
    /////////////////////////[Final Layout]///////////////////////

    let group_editor = container(row!(mods_menu, files_menu).spacing(10)).center(Length::Fill);

    column![header, profile_controls, group_editor]
        .spacing(10)
        .padding(10)
        .into()
}
