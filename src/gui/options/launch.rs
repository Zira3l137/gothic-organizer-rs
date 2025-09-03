use iced::widget;

use crate::app::message;
use crate::app::session;
use crate::core::profile::Lookup;
use crate::styled_container;

fn parser_settings(
    app: &crate::app::GothicOrganizer,
    launch_options: Option<session::GameLaunchConfiguration>,
) -> iced::Element<message::Message> {
    let theme = app.theme();
    let palette_ext = theme.extended_palette();
    let mut container_bg_color = palette_ext.primary.weak.color;
    container_bg_color.a = 0.3;

    let mut parser_settings: Lookup<session::ParserCommand, bool> = Lookup::default();
    if let Some(launch_options) = launch_options {
        parser_settings = launch_options.parser_settings.commands.clone();
    }

    styled_container!(
        session::ParserCommand::into_iter().fold(
            widget::column![
                styled_container!(
                    widget::text("Parser Settings"),
                    border_width = 2.0,
                    border_radius = 4.0,
                    background = container_bg_color
                )
                .padding(10)
                .align_left(iced::Length::Fill)
            ]
            .spacing(10)
            .padding(10),
            |container, option| {
                container.push(
                    widget::Checkbox::new(
                        format!("Reparse {option}"),
                        *parser_settings.get(option).unwrap_or(&false),
                    )
                    .on_toggle(|new_state| {
                        message::SettingsMessage::ToggleParserSetting(option.clone(), new_state).into()
                    }),
                )
            }
        ),
        border_width = 2.0,
        border_radius = 4.0
    )
    .align_top(iced::Length::Fill)
    .align_left(iced::Length::Fill)
    .padding(10)
    .into()
}

fn game_settings(
    app: &crate::app::GothicOrganizer,
    launch_options: Option<session::GameLaunchConfiguration>,
) -> iced::Element<message::Message> {
    let theme = app.theme();
    let palette_ext = theme.extended_palette();
    let mut container_bg_color = palette_ext.primary.weak.color;
    container_bg_color.a = 0.3;

    let mut game_settings = session::GameSettings::default();
    if let Some(launch_options) = launch_options {
        game_settings = launch_options.game_settings.clone();
    }

    let zspy_level_label: iced::Element<message::Message> =
        widget::Text::new(format!("ZSpy verbosity Level: {}", game_settings.zspy.verbosity)).into();

    let zspy_slider: iced::Element<message::Message> = widget::Slider::new(
        std::ops::RangeInclusive::new(0, 10),
        app.state.settings.zspy_level_field,
        |value| {
            if let Some(zspy_cfg) = &app.session.active_zspy_config
                && zspy_cfg.is_enabled
            {
                message::SettingsMessage::UpdateZspyLevel(value).into()
            } else {
                message::SystemMessage::Idle.into()
            }
        },
    )
    .into();

    let renderer_switcher: iced::Element<message::Message> = widget::ComboBox::new(
        &app.state.settings.renderer_choices,
        "Renderer Backend",
        app.session.active_renderer_backend.as_ref(),
        |renderer| message::SettingsMessage::SetRendererBackend(renderer).into(),
    )
    .into();

    let column = widget::column![
        styled_container!(
            widget::text("Game Settings"),
            border_width = 2.0,
            border_radius = 4.0,
            background = container_bg_color
        )
        .padding(10)
        .align_left(iced::Length::Fill),
        renderer_switcher,
        widget::Checkbox::new("Enable MARVIN mode", game_settings.is_marvin_mode_enabled)
            .on_toggle(|new_state| { message::SettingsMessage::ToggleMarvinMode(new_state).into() }),
        widget::Checkbox::new("Enable zSpy", game_settings.zspy.is_enabled)
            .on_toggle(|new_state| { message::SettingsMessage::ToggleZSpyState(new_state).into() }),
        zspy_level_label,
        zspy_slider
    ]
    .spacing(10)
    .padding(10);

    styled_container!(column, border_width = 2.0, border_radius = 4.0)
        .padding(10)
        .align_top(iced::Length::Fill)
        .align_left(iced::Length::Fill)
        .into()
}

pub fn launch_menu(app: &crate::app::GothicOrganizer) -> iced::Element<message::Message> {
    let launch_options = &app.session.launch_options;
    styled_container!(
        widget::row![
            parser_settings(app, launch_options.clone()),
            game_settings(app, launch_options.clone())
        ]
        .spacing(10)
        .padding(10),
        border_width = 4.0,
        border_radius = 4.0
    )
    .padding(10)
    .align_top(iced::Length::Fill)
    .align_left(iced::Length::Fill)
    .into()
}
