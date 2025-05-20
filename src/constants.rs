use fltk_theme::SchemeType;
use fltk_theme::ThemeType;

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
pub enum Theme {
    /// Windows classic
    Classic,
    /// Windows 7
    Aero,
    /// Windows 8
    Metro,
    /// Classic MacOS
    AquaClassic,
    /// Xfce
    Greybird,
    /// Windows 2000
    Blue,
    /// Dark
    #[default]
    Dark,
    /// High Contrast
    HighContrast,
}

impl From<Theme> for ThemeType {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Classic => ThemeType::Classic,
            Theme::Aero => ThemeType::Aero,
            Theme::Metro => ThemeType::Metro,
            Theme::AquaClassic => ThemeType::AquaClassic,
            Theme::Greybird => ThemeType::Greybird,
            Theme::Blue => ThemeType::Blue,
            Theme::Dark => ThemeType::Dark,
            Theme::HighContrast => ThemeType::HighContrast,
        }
    }
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
