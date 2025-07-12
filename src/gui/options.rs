use iced::widget;

use crate::app;

pub mod about;
pub mod config;
pub mod launch;
pub mod menu;

pub fn options_view(app: &app::GothicOrganizer) -> iced::Element<app::Message> {
    let theme = app.theme();
    let palette_ext = theme.extended_palette();
    let menu_bar = menu::menu_bar(app);

    let options_menu = match app.state.current_options_menu {
        menu::OptionsMenu::Config => config::config_menu(app),
        menu::OptionsMenu::Launch => launch::launch_menu(app),
        menu::OptionsMenu::About => about::about_menu(palette_ext),
    };

    widget::column![menu_bar, options_menu].spacing(10).padding(10).into()
}
