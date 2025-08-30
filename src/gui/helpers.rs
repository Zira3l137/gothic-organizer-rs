use iced::Background;
use iced::Color;
use iced::Shadow;
use iced::Theme;
use iced::Vector;
use iced::border::Border;
use iced::border::Radius;
use iced::widget::Button;
use iced::widget::Container;
use iced::widget::Svg;
use iced::widget::button::Status as ButtonStatus;
use iced::widget::button::Style as ButtonStyle;
use iced::widget::container::Style as ContainerStyle;
use iced::widget::svg::Status as SvgStatus;
use iced::widget::svg::Style as SvgStyle;

#[allow(clippy::too_many_arguments)]
pub fn styled_container<'a, Message, C, R, V, B>(
    content: impl Into<iced::Element<'a, Message>>,
    border_width: Option<f32>,
    border_color: Option<C>,
    border_radius: Option<R>,
    shadow_blur_radius: Option<f32>,
    shadow_color: Option<C>,
    shadow_offset: Option<V>,
    text_color: Option<C>,
    background: Option<B>,
) -> Container<'a, Message>
where
    B: Into<Background> + Clone + 'a,
    C: Into<Color> + Clone + 'a,
    R: Into<Radius> + Clone + 'a,
    V: Into<Vector<f32>> + Clone + 'a,
{
    Container::<Message>::new(content).style(move |theme| {
        let palette_ext = theme.extended_palette();

        let border_color = match border_color.clone() {
            Some(color) => color.into(),
            None => palette_ext.primary.base.color,
        };

        let border_radius = match border_radius.clone() {
            Some(radius) => radius.into(),
            None => Radius::default(),
        };

        let shadow_color = match shadow_color.clone() {
            Some(color) => color.into(),
            None => palette_ext.background.weak.color,
        };

        let shadow_offset = match shadow_offset.clone() {
            Some(offset) => offset.into(),
            None => iced::Vector::new(1.0, 1.0),
        };

        let text_color = match text_color.clone() {
            Some(color) => color.into(),
            None => palette_ext.background.base.text,
        };

        let background = match background.clone() {
            Some(background) => background.into(),
            None => palette_ext.background.base.color.into(),
        };

        ContainerStyle {
            background: Some(background),
            text_color: Some(text_color),

            border: Border { color: border_color, width: border_width.unwrap_or(1.0), radius: border_radius },

            shadow: Shadow {
                color: shadow_color,
                offset: shadow_offset,
                blur_radius: shadow_blur_radius.unwrap_or(1.0),
            },
        }
    })
}

#[allow(clippy::too_many_arguments)]
pub fn styled_button<'a, Message, C, R, V, B>(
    content: impl Into<iced::Element<'a, Message>>,
    border_width: Option<f32>,
    border_color: Option<C>,
    border_radius: Option<R>,
    shadow_blur_radius: Option<f32>,
    shadow_color: Option<C>,
    shadow_offset: Option<V>,
    text_color: Option<C>,
    hover_text_color: Option<C>,
    pressed_text_color: Option<C>,
    disabled_text_color: Option<C>,
    background: Option<B>,
    hover_background: Option<B>,
    pressed_background: Option<B>,
    disabled_background: Option<B>,
) -> Button<'a, Message>
where
    B: Into<Background> + Clone + 'a,
    C: Into<Color> + Clone + 'a,
    R: Into<Radius> + Clone + 'a,
    V: Into<Vector<f32>> + Clone + 'a,
    Message: Clone + 'a,
{
    let button = Button::new(content);

    button.style(move |theme, status| {
        let palette_ext = theme.extended_palette();

        let base_border_color = match border_color.clone() {
            Some(color) => color.into(),
            None => palette_ext.primary.base.color,
        };

        let base_border_radius = match border_radius.clone() {
            Some(radius) => radius.into(),
            None => Radius::from(1.0),
        };

        let base_shadow_color = match shadow_color.clone() {
            Some(color) => color.into(),
            None => Color::from_rgba(0.0, 0.0, 0.0, 0.1),
        };

        let base_shadow_offset = match shadow_offset.clone() {
            Some(offset) => offset.into(),
            None => Vector::new(0.0, 1.0),
        };

        let base_text_color = match text_color.clone() {
            Some(color) => color.into(),
            None => palette_ext.primary.strong.text,
        };

        let base_background = match background.clone() {
            Some(bg) => Some(bg.into()),
            None => Some(palette_ext.primary.base.color.into()),
        };

        match status {
            ButtonStatus::Active => ButtonStyle {
                background: base_background,
                text_color: base_text_color,
                border: Border {
                    color: base_border_color,
                    width: border_width.unwrap_or(0.0),
                    radius: base_border_radius,
                },
                shadow: Shadow {
                    color: base_shadow_color,
                    offset: base_shadow_offset,
                    blur_radius: shadow_blur_radius.unwrap_or(0.0),
                },
            },
            ButtonStatus::Hovered => {
                let hover_bg = match hover_background.clone() {
                    Some(bg) => Some(bg.into()),
                    None => base_background.map(|bg| match bg {
                        Background::Color(color) => Background::Color(Color { a: color.a * 0.9, ..color }),
                        other => other,
                    }),
                };

                let hover_text = match hover_text_color.clone() {
                    Some(color) => color.into(),
                    None => base_text_color,
                };

                ButtonStyle {
                    background: hover_bg,
                    text_color: hover_text,
                    border: Border {
                        color: base_border_color,
                        width: border_width.unwrap_or(0.0),
                        radius: base_border_radius,
                    },
                    shadow: Shadow {
                        color: base_shadow_color,
                        offset: base_shadow_offset + Vector::new(0.0, 1.0),
                        blur_radius: shadow_blur_radius.unwrap_or(0.0) + 2.0,
                    },
                }
            }
            ButtonStatus::Pressed => {
                let pressed_bg = match pressed_background.clone() {
                    Some(bg) => Some(bg.into()),
                    None => base_background.map(|bg| match bg {
                        Background::Color(color) => Background::Color(Color {
                            r: color.r * 0.8,
                            g: color.g * 0.8,
                            b: color.b * 0.8,
                            a: color.a,
                        }),
                        other => other,
                    }),
                };

                let pressed_text = match pressed_text_color.clone() {
                    Some(color) => color.into(),
                    None => base_text_color,
                };

                ButtonStyle {
                    background: pressed_bg,
                    text_color: pressed_text,
                    border: Border {
                        color: base_border_color,
                        width: border_width.unwrap_or(0.0),
                        radius: base_border_radius,
                    },
                    shadow: Shadow { color: base_shadow_color, offset: Vector::default(), blur_radius: 0.0 },
                }
            }
            ButtonStatus::Disabled => {
                let disabled_bg = match disabled_background.clone() {
                    Some(bg) => Some(bg.into()),
                    None => base_background.map(|bg| match bg {
                        Background::Color(color) => Background::Color(Color { a: color.a * 0.5, ..color }),
                        other => other,
                    }),
                };

                let disabled_text = match disabled_text_color.clone() {
                    Some(color) => color.into(),
                    None => Color { a: base_text_color.a * 0.5, ..base_text_color },
                };

                ButtonStyle {
                    background: disabled_bg,
                    text_color: disabled_text,
                    border: Border {
                        color: Color { a: base_border_color.a * 0.5, ..base_border_color },
                        width: border_width.unwrap_or(0.0),
                        radius: base_border_radius,
                    },
                    shadow: Shadow { color: Color::TRANSPARENT, offset: Vector::default(), blur_radius: 0.0 },
                }
            }
        }
    })
}

#[allow(clippy::too_many_arguments)]
pub fn svg_with_color<'a, C>(
    handle: impl Into<iced_core::svg::Handle>,
    color_idle: Option<C>,
    color_hovered: Option<C>,
) -> Svg<'a>
where
    C: Into<Color> + Clone + 'a,
{
    Svg::new(handle).style(move |theme: &Theme, status| {
        let palette_ext = theme.extended_palette();

        let idle_color = match color_idle.clone() {
            Some(idle_color) => idle_color.into(),
            None => palette_ext.primary.base.text,
        };

        let hovered_color = match color_hovered.clone() {
            Some(hovered_color) => hovered_color.into(),
            None => palette_ext.primary.strong.text,
        };

        SvgStyle {
            color: match status {
                SvgStatus::Idle => Some(idle_color),
                SvgStatus::Hovered => Some(hovered_color),
            },
        }
    })
}
