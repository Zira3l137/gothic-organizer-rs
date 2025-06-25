use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::text;
use iced::advanced::widget::Tree;
use iced::advanced::Layout;
use iced::advanced::Widget;
use iced::alignment;
use iced::event;
use iced::widget::text::Fragment;
use iced::widget::text::LineHeight;
use iced::widget::text::Shaping;
use iced::widget::text::Wrapping;
use iced::Element;
use iced::Length;
use iced::Pixels;
use iced::Rectangle;
use iced::Size;
use iced::Theme;

pub struct ClickableText<'a, Renderer, Message>
where
    Renderer: text::Renderer,
{
    width: Length,
    height: Length,
    shaping: Shaping,
    wrapping: Wrapping,
    size: Option<Pixels>,
    fragment: Fragment<'a>,
    passed_message: Message,
    line_height: LineHeight,
    font: Option<Renderer::Font>,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
}

impl<'a, Renderer, Message> ClickableText<'a, Renderer, Message>
where
    Renderer: text::Renderer,
{
    pub fn new(fragment: impl text::IntoFragment<'a>, message: Message) -> Self {
        ClickableText {
            fragment: fragment.into_fragment(),
            size: None,
            line_height: LineHeight::default(),
            font: None,
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            shaping: Shaping::default(),
            wrapping: Wrapping::default(),
            passed_message: message,
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn align_x(mut self, alignment: impl Into<iced::alignment::Horizontal>) -> Self {
        self.horizontal_alignment = alignment.into();
        self
    }

    pub fn align_y(mut self, alignment: impl Into<iced::alignment::Vertical>) -> Self {
        self.vertical_alignment = alignment.into();
        self
    }

    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }
}

impl<'a, Renderer, Message> Widget<Message, Theme, Renderer> for ClickableText<'a, Renderer, Message>
where
    Renderer: text::Renderer,
    Message: Clone,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn state(&self) -> iced_core::widget::tree::State {
        iced_core::widget::tree::State::new(iced_core::widget::text::State::<Renderer::Paragraph>(
            iced_core::text::paragraph::Plain::default(),
        ))
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        iced_core::widget::text::layout(
            tree.state
                .downcast_mut::<iced::widget::text::State<Renderer::Paragraph>>(),
            renderer,
            limits,
            self.width,
            self.height,
            &self.fragment,
            self.line_height,
            self.size,
            self.font,
            self.horizontal_alignment,
            self.vertical_alignment,
            self.shaping,
            self.wrapping,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree
            .state
            .downcast_ref::<iced_core::widget::text::State<Renderer::Paragraph>>();
        let palette = theme.palette();
        let paragraph = state.0.raw();

        let bounds = layout.bounds();

        let x = match self.horizontal_alignment {
            alignment::Horizontal::Left => bounds.x,
            alignment::Horizontal::Center => bounds.center_x(),
            alignment::Horizontal::Right => bounds.x + bounds.width,
        };

        let y = match self.vertical_alignment {
            alignment::Vertical::Top => bounds.y,
            alignment::Vertical::Center => bounds.center_y(),
            alignment::Vertical::Bottom => bounds.y + bounds.height,
        };

        renderer.fill_paragraph(
            paragraph,
            iced_core::Point::new(x, y),
            if cursor.is_over(layout.bounds()) {
                palette.primary
            } else {
                palette.text
            },
            *viewport,
        );
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> iced::advanced::graphics::core::event::Status {
        if cursor.is_over(layout.bounds()) {
            match event {
                iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    shell.publish(self.passed_message.clone());
                    event::Status::Captured
                }
                _ => event::Status::Ignored,
            }
        } else {
            event::Status::Ignored
        }
    }
}

impl<'a, Renderer, Message> From<ClickableText<'a, Renderer, Message>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer + 'a,
    Message: Clone + 'a,
{
    fn from(widget: ClickableText<'a, Renderer, Message>) -> Self {
        Self::new(widget)
    }
}
