use iced::widget;

use crate::app;
use crate::config;
use crate::core::lookup::Lookup;
use crate::styled_container;

fn parser_settings(
    _app: &app::GothicOrganizer,
    launch_options: Option<config::LaunchOptions>,
) -> iced::Element<app::Message> {
    let mut parser_settings: Lookup<config::ParserCommand, bool> = Lookup::new();
    if let Some(launch_options) = launch_options {
        parser_settings = launch_options.parser_settings.commands.clone();
    }

    styled_container!(
        config::ParserCommand::into_iter().fold(
            widget::column![
                styled_container!(
                    widget::text("Parser Settings"),
                    border_width = 2.0,
                    border_radius = 4.0
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
                        app::Message::ToggleParserSetting(option.clone(), new_state)
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
    app: &app::GothicOrganizer,
    launch_options: Option<config::LaunchOptions>,
) -> iced::Element<app::Message> {
    let mut game_settings = config::GameSettings::default();
    if let Some(launch_options) = launch_options {
        game_settings = launch_options.game_settings.clone();
    }

    let zspy_level_label: iced::Element<app::Message> =
        widget::Text::new(format!("ZSpy verbosity Level: {}", game_settings.zspy.verbosity)).into();

    let zspy_slider: iced::Element<app::Message> = widget::Slider::new(
        std::ops::RangeInclusive::new(0, 10),
        app.state.zspy_level_input,
        |value| {
            if let Some(zspy_cfg) = &app.session.active_zspy_config
                && zspy_cfg.enabled
            {
                app::Message::ZSpyLevelChanged(value)
            } else {
                app::Message::None
            }
        },
    )
    .into();

    let renderer_switcher: iced::Element<app::Message> = widget::ComboBox::new(
        &app.state.renderer_choices,
        "Renderer Backend",
        app.session.active_renderer_backend.as_ref(),
        app::Message::OptionsRendererSwitched,
    )
    .into();

    let column = widget::column![
        styled_container!(widget::text("Game Settings"), border_width = 2.0, border_radius = 4.0)
            .padding(10)
            .align_left(iced::Length::Fill),
        renderer_switcher,
        widget::Checkbox::new("Enable MARVIN mode", game_settings.marvin_mode)
            .on_toggle(|new_state| { app::Message::ToggleMarvinMode(new_state) }),
        widget::Checkbox::new("Enable zSpy", game_settings.zspy.enabled)
            .on_toggle(|new_state| { app::Message::ToggleZSpy(new_state) }),
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

pub fn launch_menu(app: &app::GothicOrganizer) -> iced::Element<app::Message> {
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
