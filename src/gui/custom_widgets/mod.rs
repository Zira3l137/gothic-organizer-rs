pub mod clickable_text;

pub fn clickable_text<'a, Message, Renderer>(
    content: impl iced::widget::text::IntoFragment<'a>,
) -> clickable_text::ClickableText<'a, Renderer, Message>
where
    Renderer: iced_core::text::Renderer,
{
    clickable_text::ClickableText::new(content)
}
