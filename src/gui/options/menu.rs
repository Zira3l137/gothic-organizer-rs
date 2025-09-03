use iced::widget;

use crate::app::message;
use crate::svg_with_color;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum OptionsMenu {
    #[default]
    Config,
    Launch,
    About,
}

impl IntoIterator for OptionsMenu {
    type Item = OptionsMenu;
    type IntoIter = std::array::IntoIter<OptionsMenu, 3>;

    fn into_iter(self) -> Self::IntoIter {
        [OptionsMenu::Config, OptionsMenu::Launch, OptionsMenu::About].into_iter()
    }
}

impl std::fmt::Display for OptionsMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionsMenu::Config => write!(f, "Config"),
            OptionsMenu::Launch => write!(f, "Launch"),
            OptionsMenu::About => write!(f, "About"),
        }
    }
}

impl<'a, Message: 'a> From<OptionsMenu> for iced::Element<'a, Message> {
    fn from(menu: OptionsMenu) -> Self {
        widget::row![
            svg_with_color!(format!("./resources/{menu}.svg")).width(20).height(20),
            widget::text!("{menu}")
        ]
        .spacing(10)
        .into()
    }
}

pub fn menu_bar(app: &crate::app::GothicOrganizer) -> iced::Element<message::Message> {
    let current_menu = app.state.ui.active_options_menu;
    widget::container(OptionsMenu::into_iter(OptionsMenu::default()).fold(widget::row![], |bar, menu| {
        let menu_button = widget::button(menu)
            .on_press_maybe((current_menu != menu).then_some(message::UiMessage::SetOptionsMenu(menu).into()))
            .width(iced::Length::Fill);
        bar.push(menu_button)
    }))
    .center_x(iced::Length::Fill)
    .into()
}
