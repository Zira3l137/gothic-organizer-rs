use std::path::PathBuf;

use fltk::app::App;
use fltk::image::IcoImage;
use fltk::prelude::*;
use fltk::window::Window;

use fltk_theme::ColorTheme;
use fltk_theme::WidgetScheme;

use crate::constants::ColorScheme;
use crate::constants::Style;
use crate::error::GuiError;

pub trait GothicOrganizerWindow {
    type Message;

    fn window(settings: &ApplicationSettings) -> Window {
        let mut wnd = Window::default()
            .with_size(settings.width, settings.height)
            .with_pos(settings.resolution.0, settings.resolution.1)
            .with_label(&settings.title);

        if settings.centered {
            wnd = wnd.center_screen();
        }

        if settings.resizable {
            wnd.make_resizable(true);
        }

        if let Some(icon) = &settings.icon {
            wnd.set_icon(IcoImage::load(icon).ok())
        }

        wnd
    }

    fn app(settings: &ApplicationSettings) -> App {
        let app = App::default();
        WidgetScheme::new(settings.style.into()).apply();
        ColorTheme::new(settings.colors.into()).apply();
        app
    }

    fn run(&mut self) -> Result<(), GuiError>;
}

#[derive(Debug, Default)]
pub struct ApplicationSettings {
    pub icon: Option<PathBuf>,
    pub title: String,
    pub width: i32,
    pub height: i32,
    pub centered: bool,
    pub resolution: (i32, i32),
    pub resizable: bool,
    pub style: Style,
    pub colors: ColorScheme,
}
