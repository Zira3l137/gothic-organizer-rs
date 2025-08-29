use crate::styled_container;
use iced::widget;

pub fn logs_view(app: &crate::app::GothicOrganizer) -> iced::Element<crate::app::message::Message> {
    let theme = app.theme();
    let palette_ext = theme.extended_palette();
    let messages_column: widget::Column<crate::app::message::Message> = app
        .state
        .errors
        .error_history
        .iter()
        .fold(widget::column![], |column, ctx| {
            let message_contents = format!("{}: ERROR: {}", ctx.timestamp_string(), ctx.error);
            let message_text = widget::text(message_contents).color(palette_ext.danger.strong.color);
            let message_container = styled_container!(message_text, border_width = 2.0, border_radius = 0.0)
                .padding(10)
                .center_x(iced::Length::Fill);

            let tooltip_contents = format!("Try this: {}", ctx.suggested_action.clone());
            let tooltip_text = widget::text(tooltip_contents);
            let tooltip_body =
                styled_container!(tooltip_text, border_width = 1.0, border_radius = 4.0).padding(5);

            column.push(widget::tooltip(message_container, tooltip_body, widget::tooltip::Position::Bottom))
        })
        .padding(10)
        .spacing(10);

    let messages_container =
        styled_container!(widget::scrollable(messages_column), border_width = 4.0, border_radius = 4.0)
            .center_x(iced::Length::Fill)
            .align_top(iced::Length::Fill);

    let logs_view = widget::column![messages_container];

    styled_container!(logs_view, border_width = 4.0, border_radius = 4.0)
        .center(iced::Length::Fill)
        .align_top(iced::Length::Fill)
        .into()
}
