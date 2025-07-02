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
use iced::widget::svg;
use iced::widget::text;
use iced::widget::tooltip;
use iced::widget::Column;
use iced::widget::Row;
use iced::Element;
use iced::Length;

use crate::app::Message;
use crate::core::constants::app_title_full;
use crate::gui::custom_widgets::clickable_text::ClickableText;
use crate::styled_container;
use crate::svg_with_color;

pub fn editor_view(app: &crate::app::GothicOrganizer) -> Element<Message> {
    /////////////////////////[States]/////////////////////////////
    let current_profile = app
        .profile_selected
        .as_ref()
        .and_then(|s| app.profiles.get(s));

    let instance_selected = app
        .instance_selected
        .as_ref()
        .and_then(|s| current_profile.and_then(|p| p.instances.as_ref().and_then(|i| i.get(s))));

    let current_instance = app
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

    let button_options_icon: svg::Svg<_> = svg("./resources/options.svg").height(20).width(20);
    let button_options = button(button_options_icon).on_press(Message::InvokeOptionsMenu);
    let header: Row<_> = row!(logo, title, horizontal_space(), button_options).spacing(10);
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

    let button_browse = button("Browse").on_press(Message::SetGameDir(None, None));
    let group_instance = container(row!(choice_instance, button_add, button_remove));

    let profile_controls = styled_container!(
        row!(
            choice_profile,
            match current_profile {
                Some(profile) if profile.path.display().to_string() == "" => {
                    let button_browse: Element<_> = button_browse.into();
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
        border_width = 2.0,
        border_radius = 4.0
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

    let button_toggle_all = button("Toggle all").on_press(Message::FileToggleAll);

    let controls_files = styled_container!(
        row!(button_back, button_home, button_toggle_all).spacing(10),
        border_width = 1.0,
        border_radius = 4.0
    )
    .padding(10)
    .align_left(Length::Fill);

    let mut column_files: Column<_> = Column::new();

    if instance_selected.is_some() {
        column_files = app
            .state
            .current_directory_entries
            .iter()
            .fold(Column::new(), |column, (path, info)| {
                let is_dir = path.is_dir();

                let file_name = path.file_name().unwrap().to_string_lossy().to_string();

                let label: Element<_> = match &is_dir {
                    true => ClickableText::new(file_name)
                        .on_press(|| Message::TraverseIntoDir(path.clone()))
                        .into(),
                    false => text(file_name).into(),
                };

                let icon_path = match &is_dir {
                    true => "./resources/directory.svg",
                    false => "./resources/file.svg",
                };

                let icon: Element<_> = svg_with_color!(icon_path).height(20).width(20).into();

                let parent_name = info.parent_name.clone().unwrap_or(String::from("Default"));
                let tooltip_text = text(format!("Supplied by: {parent_name}"));
                let tooltip_body = styled_container!(tooltip_text, border_width = 1.0, border_radius = 4.0).padding(5);

                let checkbox = checkbox("", info.enabled).on_toggle(|_| Message::FileToggle(path.clone()));

                let group_file = tooltip(
                    styled_container!(
                        row![checkbox, icon, label],
                        border_width = 1.0,
                        border_radius = 4.0
                    )
                    .padding(5)
                    .align_left(Length::Fill),
                    tooltip_body,
                    tooltip::Position::Left,
                );

                column.push(group_file)
            });
    }

    let files_menu = styled_container!(
        column!(controls_files, scrollable(column_files)).spacing(10),
        border_width = 2.0,
        border_radius = 4.0
    )
    .padding(10)
    .align_top(Length::Fill);

    //////////////////////////////////////////////////////////////
    /////////////////////////[Left-side Mods List]////////////////

    let icon_add = svg_with_color!(
        "./resources/add_mod.svg",
        color_idle = iced::Color::from_rgb(0.0, 230.0, 0.0),
        color_hovered = iced::Color::from_rgb(0.0, 255.0, 0.0)
    )
    .height(20)
    .width(20);
    let button_add_mod = button(icon_add).on_press(Message::ModAdd(None));

    let group_mod_controls = styled_container!(
        row!(button_add_mod),
        border_width = 1.0,
        border_radius = 4.0
    )
    .padding(10)
    .center_x(Length::Fill);

    let mut column_mods: Column<_> = Column::new();

    if let Some(instance) = current_instance
        && let Some(mods) = &instance.mods
    {
        column_mods = mods.iter().fold(Column::new(), |column, mod_info| {
            let mod_name: Element<_> = text(mod_info.name.clone()).into();
            let checkbox = checkbox("", mod_info.enabled).on_toggle(|new_state| Message::ModToggle(mod_info.name.clone(), new_state));
            let button_uninstall = ClickableText::new("Uninstall").on_press(|| Message::ModUninstall(mod_info.name.clone()));

            let group_mod = styled_container!(
                row![checkbox, mod_name, horizontal_space(), button_uninstall],
                border_width = 1.0,
                border_radius = 4.0
            )
            .padding(5)
            .align_left(Length::Fill);

            column.push(group_mod)
        })
    }

    let mods_menu = styled_container!(
        column!(group_mod_controls, scrollable(column_mods)),
        border_width = 2.0,
        border_radius = 4.0
    )
    .padding(10)
    .align_y(Vertical::Top)
    .center_x(Length::Fill);

    //////////////////////////////////////////////////////////////
    /////////////////////////[Final Layout]///////////////////////

    let group_editor = container(row!(mods_menu, files_menu).spacing(10)).align_top(Length::Fill);

    column![header, profile_controls, group_editor]
        .spacing(10)
        .padding(10)
        .into()
}
