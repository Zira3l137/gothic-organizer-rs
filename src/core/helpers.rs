use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::write;
use std::path::Path;
use std::path::PathBuf;

use crate::app::session;
use crate::core::constants;
use crate::core::profile;

fn default_path<P: AsRef<Path>>(custom_path: Option<P>) -> PathBuf {
    match custom_path {
        Some(p) => p.as_ref().to_path_buf().join(constants::APP_NAME),
        None => crate::core::constants::local_app_data_path().join(constants::APP_NAME),
    }
}

pub fn save_app_session<P: AsRef<Path>>(
    session: &session::ApplicationSession,
    custom_path: Option<P>,
) -> Result<(), std::io::Error> {
    let default_path = default_path(custom_path);
    let session_string = serde_json::to_string_pretty(session)?;
    tracing::info!("Writing to {}", default_path.join("session.json").display());
    write(default_path.join("session.json"), session_string)?;

    Ok(())
}

pub fn load_app_session<P: AsRef<Path>>(custom_path: Option<P>) -> Option<session::ApplicationSession> {
    let default_path = default_path(custom_path);
    if !default_path.exists() {
        return None;
    }

    tracing::info!("Reading from {}", default_path.join("session.json").display());
    let session_json = read_to_string(default_path.join("session.json")).ok()?;

    let Ok(session): Result<session::ApplicationSession, _> = serde_json::from_str(&session_json) else {
        return None;
    };

    Some(session)
}

pub fn save_profile<P: AsRef<Path>>(
    profile: &profile::Profile,
    custom_path: Option<P>,
) -> Result<(), std::io::Error> {
    let default_profile_path = default_path(custom_path);
    let this_profile_path = default_profile_path.join(&profile.name);
    let profile_json = serde_json::to_string_pretty(&profile).map_err(std::io::Error::other)?;

    create_dir_all(&this_profile_path).map_err(|e| std::io::Error::new(e.kind(), e))?;

    tracing::info!("Writing to {}", this_profile_path.join("profile.json").display());
    write(default_profile_path.join(&profile.name).join("profile.json"), profile_json)?;

    Ok(())
}

pub fn load_profile<P: AsRef<Path>>(name: &str, custom_path: Option<P>) -> Option<profile::Profile> {
    let default_profile_path = default_path(custom_path);
    let this_profile_path = default_profile_path.join(name);

    tracing::info!("Reading from {}", this_profile_path.join("profile.json").display());
    let profile_json = read_to_string(this_profile_path.join("profile.json")).ok()?;

    let Ok(profile): Result<profile::Profile, _> = serde_json::from_str(&profile_json) else {
        return None;
    };

    Some(profile)
}

pub fn default_themes<'a>() -> [(&'a str, iced::Theme); 22] {
    [
        ("Light", iced::Theme::Light),
        ("Dark", iced::Theme::Dark),
        ("Dracula", iced::Theme::Dracula),
        ("Nord", iced::Theme::Nord),
        ("Solarized Light", iced::Theme::SolarizedLight),
        ("Solarized Dark", iced::Theme::SolarizedDark),
        ("Gruvbox Light", iced::Theme::GruvboxLight),
        ("Gruvbox Dark", iced::Theme::GruvboxDark),
        ("Catppuccin Latte", iced::Theme::CatppuccinLatte),
        ("Catppuccin Frappé", iced::Theme::CatppuccinFrappe),
        ("Catppuccin Macchiato", iced::Theme::CatppuccinMacchiato),
        ("Catppuccin Mocha", iced::Theme::CatppuccinMocha),
        ("Tokyo Night", iced::Theme::TokyoNight),
        ("Tokyo Night Storm", iced::Theme::TokyoNightStorm),
        ("Tokyo Night Light", iced::Theme::TokyoNightLight),
        ("Kanagawa Wave", iced::Theme::KanagawaWave),
        ("Kanagawa Dragon", iced::Theme::KanagawaDragon),
        ("Kanagawa Lotus", iced::Theme::KanagawaLotus),
        ("Moonfly", iced::Theme::Moonfly),
        ("Nightfly", iced::Theme::Nightfly),
        ("Oxocarbon", iced::Theme::Oxocarbon),
        ("Ferra", iced::Theme::Ferra),
    ]
}
