use fltk::image::IcoImage;
use fltk::image::PngImage;
use fltk_theme::color_themes::fleet::*;
use fltk_theme::ColorMap;
use fltk_theme::SchemeType;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_TITLE: &str = "Gothic Organizer";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameProfile {
    Gothic,
    Gothic2,
    Gothic2Classic,
    GothicSequel,
}

impl std::fmt::Display for GameProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameProfile::Gothic => write!(f, "Gothic"),
            GameProfile::Gothic2 => write!(f, "Gothic II Night of Raven"),
            GameProfile::Gothic2Classic => write!(f, "Gothic II Classic"),
            GameProfile::GothicSequel => write!(f, "Gothic Sequel"),
        }
    }
}

pub fn game_profile_list() -> [String; 4] {
    [
        GameProfile::Gothic.to_string(),
        GameProfile::Gothic2.to_string(),
        GameProfile::Gothic2Classic.to_string(),
        GameProfile::GothicSequel.to_string(),
    ]
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Style {
    /// A scheme mimicking modern Aqua
    Aqua,
    /// Taken from the NTK fork
    Clean,
    /// Taken from the NTK fork
    Crystal,
    /// Windows 10
    Fluent,
    /// Taken from the NTK fork, a modification of the FLTK Gleam scheme
    Gleam,
    /**
    Draws the following FrameTypes using scalable vector graphics:
    - RoundedFrame
    - RoundedBox
    - RFlatBox
    - OvalBox
    - OvalFrame
    - OFlatFrame
    */
    SvgBased,
    /// A scheme mimicking the Sweet theme for GNOME/KDE
    Sweet,
    /// A 3D scheme designed for good looks in both dark and light colors
    #[default]
    Fleet1,
    /// A gradient scheme designed for good looks in both dark and light colors
    Fleet2,
}

impl From<Style> for SchemeType {
    fn from(style: Style) -> Self {
        match style {
            Style::Aqua => SchemeType::Aqua,
            Style::Clean => SchemeType::Clean,
            Style::Crystal => SchemeType::Crystal,
            Style::Fluent => SchemeType::Fluent,
            Style::Gleam => SchemeType::Gleam,
            Style::SvgBased => SchemeType::SvgBased,
            Style::Sweet => SchemeType::Sweet,
            Style::Fleet1 => SchemeType::Fleet1,
            Style::Fleet2 => SchemeType::Fleet2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorScheme {
    GruvboxDark,
    Monokai,
    SolarizedLight,
    Light,
    #[default]
    Dark1,
    Dark2,
    Tan,
    DarkTan,
    Nord,
    Marine,
    Blueish,
    HighContrast,
    Forest,
    PurpleDusk,
    SolarizedDark,
    GruvboxLight,
    Dracula,
    OceanicNext,
    Minimalist,
    Autumn,
    Cyberpunk,
    MaterialDark,
    Mint,
    Vintage,
    Gray,
}

impl From<ColorScheme> for &[ColorMap] {
    fn from(theme: ColorScheme) -> Self {
        match theme {
            ColorScheme::GruvboxDark => &GRUVBOX_DARK,
            ColorScheme::Monokai => &MONOKAI,
            ColorScheme::SolarizedLight => &SOLARIZED_LIGHT,
            ColorScheme::Light => &LIGHT,
            ColorScheme::Dark1 => &DARK1,
            ColorScheme::Dark2 => &DARK2,
            ColorScheme::Tan => &TAN,
            ColorScheme::DarkTan => &DARK_TAN,
            ColorScheme::Nord => &NORD,
            ColorScheme::Marine => &MARINE,
            ColorScheme::Blueish => &BLUEISH,
            ColorScheme::HighContrast => &HIGH_CONTRAST,
            ColorScheme::Forest => &FOREST,
            ColorScheme::PurpleDusk => &PURPLE_DUSK,
            ColorScheme::SolarizedDark => &SOLARIZED_DARK,
            ColorScheme::GruvboxLight => &GRUVBOX_LIGHT,
            ColorScheme::Dracula => &DRACULA,
            ColorScheme::OceanicNext => &OCEANIC_NEXT,
            ColorScheme::Minimalist => &MINIMALIST,
            ColorScheme::Autumn => &AUTUMN,
            ColorScheme::Cyberpunk => &CYBERPUNK,
            ColorScheme::MaterialDark => &MATERIAL_DARK,
            ColorScheme::Mint => &MINT,
            ColorScheme::Vintage => &VINTAGE,
            ColorScheme::Gray => &GRAY,
        }
    }
}

pub fn app_icon() -> Option<IcoImage> {
    IcoImage::load("resources/icon.ico").ok()
}

pub fn checked_icon() -> Option<PngImage> {
    PngImage::load("resources/checked.png").ok()
}

pub fn unchecked_icon() -> Option<PngImage> {
    PngImage::load("resources/unchecked.png").ok()
}

pub fn dir_icon() -> Option<PngImage> {
    PngImage::load("resources/dir.png").ok()
}
