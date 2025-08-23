use iced::widget::column;
use iced::widget::container;
use iced::widget::row;

use crate::app::Message;

pub fn overwrites_view(app: &crate::app::GothicOrganizer) -> iced::Element<Message> {
    column![].spacing(10).padding(10).into()
}
