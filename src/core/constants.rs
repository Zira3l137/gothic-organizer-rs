use std::path::PathBuf;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_TITLE: &str = "Gothic Organizer";
pub const APP_AUTHOR: &str = "Zira3l137";
pub const APP_REPOSITORY: &str = "https://github.com/Zira3l137/gothic-organizer-rs";

pub fn app_title_full() -> String {
    format!("{APP_TITLE} v{APP_VERSION}")
}

pub fn app_info() -> String {
    format!("{APP_TITLE}\nVersion: {APP_VERSION}\nAuthor: {APP_AUTHOR}\nRepository: {APP_REPOSITORY}")
}

pub fn local_app_data() -> String {
    #[cfg(windows)]
    {
        std::env::var("LOCALAPPDATA").unwrap_or(String::from(""))
    }
    #[cfg(unix)]
    {
        std::env::var("XDG_DATA_HOME").unwrap_or(String::from("~/.local/share"))
    }
}

pub fn local_profiles_dir() -> PathBuf {
    PathBuf::from(local_app_data()).join(APP_NAME)
}

pub fn mod_storage_dir() -> PathBuf {
    PathBuf::from("./mods")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Gothic,
    Gothic2Classic,
    Gothic2NightOfTheRaven,
    GothicSequel,
}

impl Profile {
    pub fn into_iter() -> std::slice::Iter<'static, Profile> {
        static PROFILES: [Profile; 4] = [
            Profile::Gothic,
            Profile::Gothic2Classic,
            Profile::Gothic2NightOfTheRaven,
            Profile::GothicSequel,
        ];
        PROFILES.iter()
    }
}

impl From<Profile> for &'static str {
    fn from(value: Profile) -> Self {
        match value {
            Profile::Gothic => "Gothic",
            Profile::Gothic2Classic => "Gothic 2 Classic",
            Profile::Gothic2NightOfTheRaven => "Gothic 2 Night of the Raven",
            Profile::GothicSequel => "Gothic Sequel",
        }
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Profile::Gothic => write!(f, "Gothic"),
            Profile::Gothic2Classic => write!(f, "Gothic 2 Classic"),
            Profile::Gothic2NightOfTheRaven => write!(f, "Gothic 2 Night of the Raven"),
            Profile::GothicSequel => write!(f, "Gothic Sequel"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
}

impl Theme {
    pub fn into_iter() -> std::slice::Iter<'static, Theme> {
        static THEMES: [Theme; 22] = [
            Theme::Light,
            Theme::Dark,
            Theme::Dracula,
            Theme::Nord,
            Theme::SolarizedLight,
            Theme::SolarizedDark,
            Theme::GruvboxLight,
            Theme::GruvboxDark,
            Theme::CatppuccinLatte,
            Theme::CatppuccinFrappe,
            Theme::CatppuccinMacchiato,
            Theme::CatppuccinMocha,
            Theme::TokyoNight,
            Theme::TokyoNightStorm,
            Theme::TokyoNightLight,
            Theme::KanagawaWave,
            Theme::KanagawaDragon,
            Theme::KanagawaLotus,
            Theme::Moonfly,
            Theme::Nightfly,
            Theme::Oxocarbon,
            Theme::Ferra,
        ];
        THEMES.iter()
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Light => write!(f, "Light"),
            Theme::Dark => write!(f, "Dark"),
            Theme::Dracula => write!(f, "Dracula"),
            Theme::Nord => write!(f, "Nord"),
            Theme::SolarizedLight => write!(f, "Solarized Light"),
            Theme::SolarizedDark => write!(f, "Solarized Dark"),
            Theme::GruvboxLight => write!(f, "Gruvbox Light"),
            Theme::GruvboxDark => write!(f, "Gruvbox Dark"),
            Theme::CatppuccinLatte => write!(f, "Catppuccin Latte"),
            Theme::CatppuccinFrappe => write!(f, "Catppuccin Frappe"),
            Theme::CatppuccinMacchiato => write!(f, "Catppuccin Macchiato"),
            Theme::CatppuccinMocha => write!(f, "Catppuccin Mocha"),
            Theme::TokyoNight => write!(f, "Tokyo Night"),
            Theme::TokyoNightStorm => write!(f, "Tokyo Night Storm"),
            Theme::TokyoNightLight => write!(f, "Tokyo Night Light"),
            Theme::KanagawaWave => write!(f, "Kanagawa Wave"),
            Theme::KanagawaDragon => write!(f, "Kanagawa Dragon"),
            Theme::KanagawaLotus => write!(f, "Kanagawa Lotus"),
            Theme::Moonfly => write!(f, "Moonfly"),
            Theme::Nightfly => write!(f, "Nightfly"),
            Theme::Oxocarbon => write!(f, "Oxocarbon"),
            Theme::Ferra => write!(f, "Ferra"),
        }
    }
}

impl From<Theme> for iced::Theme {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => iced::Theme::Light,
            Theme::Dark => iced::Theme::Dark,
            Theme::Dracula => iced::Theme::Dracula,
            Theme::Nord => iced::Theme::Nord,
            Theme::SolarizedLight => iced::Theme::SolarizedLight,
            Theme::SolarizedDark => iced::Theme::SolarizedDark,
            Theme::GruvboxLight => iced::Theme::GruvboxLight,
            Theme::GruvboxDark => iced::Theme::GruvboxDark,
            Theme::CatppuccinLatte => iced::Theme::CatppuccinLatte,
            Theme::CatppuccinFrappe => iced::Theme::CatppuccinFrappe,
            Theme::CatppuccinMacchiato => iced::Theme::CatppuccinMacchiato,
            Theme::CatppuccinMocha => iced::Theme::CatppuccinMocha,
            Theme::TokyoNight => iced::Theme::TokyoNight,
            Theme::TokyoNightStorm => iced::Theme::TokyoNightStorm,
            Theme::TokyoNightLight => iced::Theme::TokyoNightLight,
            Theme::KanagawaWave => iced::Theme::KanagawaWave,
            Theme::KanagawaDragon => iced::Theme::KanagawaDragon,
            Theme::KanagawaLotus => iced::Theme::KanagawaLotus,
            Theme::Moonfly => iced::Theme::Moonfly,
            Theme::Nightfly => iced::Theme::Nightfly,
            Theme::Oxocarbon => iced::Theme::Oxocarbon,
            Theme::Ferra => iced::Theme::Ferra,
        }
    }
}

impl From<String> for Theme {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Light" => Theme::Light,
            "Dark" => Theme::Dark,
            "Dracula" => Theme::Dracula,
            "Nord" => Theme::Nord,
            "Solarized Light" => Theme::SolarizedLight,
            "Solarized Dark" => Theme::SolarizedDark,
            "Gruvbox Light" => Theme::GruvboxLight,
            "Gruvbox Dark" => Theme::GruvboxDark,
            "Catppuccin Latte" => Theme::CatppuccinLatte,
            "Catppuccin Frappe" => Theme::CatppuccinFrappe,
            "Catppuccin Macchiato" => Theme::CatppuccinMacchiato,
            "Catppuccin Mocha" => Theme::CatppuccinMocha,
            "Tokyo Night" => Theme::TokyoNight,
            "Tokyo Night Storm" => Theme::TokyoNightStorm,
            "Tokyo Night Light" => Theme::TokyoNightLight,
            "Kanagawa Wave" => Theme::KanagawaWave,
            "Kanagawa Dragon" => Theme::KanagawaDragon,
            "Kanagawa Lotus" => Theme::KanagawaLotus,
            "Moonfly" => Theme::Moonfly,
            "Nightfly" => Theme::Nightfly,
            "Oxocarbon" => Theme::Oxocarbon,
            "Ferra" => Theme::Ferra,
            _ => Theme::Light,
        }
    }
}
