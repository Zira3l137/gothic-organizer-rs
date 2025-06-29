use std::fs::create_dir_all;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::write;
use std::path::Path;
use std::path::PathBuf;

use iced::border::Radius;
use iced::widget::Container;
use iced::widget::Svg;
use iced::Border;
use iced::Shadow;

use crate::core::constants::APP_NAME;
use crate::core::profile::FileInfo;
use crate::core::profile::Profile;
use crate::core::profile::Session;

fn default_path<P: AsRef<Path>>(custom_path: Option<P>) -> PathBuf {
    match custom_path {
        Some(p) => p.as_ref().to_path_buf(),
        None => PathBuf::from(crate::core::constants::local_app_data()).join(APP_NAME),
    }
}

pub fn save_session<P: AsRef<Path>>(
    selected_profile: Option<String>,
    selected_instance: Option<String>,
    cache: Option<crate::core::profile::Lookup<PathBuf, FileInfo>>,
    theme: Option<String>,
    custom_path: Option<P>,
) -> Result<(), std::io::Error> {
    let session = Session {
        selected_profile,
        selected_instance,
        theme,
        cache,
    };

    let default_path = default_path(custom_path);
    let session_string = serde_json::to_string_pretty(&session)?;
    write(default_path.join("session.json"), session_string)?;

    Ok(())
}

pub fn load_session<P: AsRef<Path>>(custom_path: Option<P>) -> Option<Session> {
    let default_path = default_path(custom_path);
    if !default_path.exists() {
        return None;
    }

    let session_json = read_to_string(default_path.join("session.json")).ok()?;

    let Ok(session): Result<Session, _> = serde_json::from_str(&session_json) else {
        return None;
    };

    Some(session)
}

pub fn save_profile<P: AsRef<Path>>(profile: &Profile, custom_path: Option<P>) -> Result<(), std::io::Error> {
    let default_profile_path = default_path(custom_path);
    let profile_json = serde_json::to_string_pretty(&profile).map_err(std::io::Error::other)?;

    create_dir_all(default_profile_path.join(&profile.name)).map_err(|e| std::io::Error::new(e.kind(), e))?;
    write(
        default_profile_path
            .join(&profile.name)
            .join("profile.json"),
        profile_json,
    )?;

    Ok(())
}

pub fn load_profile<P: AsRef<Path>>(name: &str, custom_path: Option<P>) -> Option<Profile> {
    let default_profile_path = default_path(custom_path);
    let mut entries = read_dir(default_profile_path).ok()?;

    let profile = entries.find_map(|e| {
        let entry = e.ok()?;

        if !entry.path().is_dir() {
            return None;
        }

        if entry.file_name().to_string_lossy().to_lowercase() != name.to_lowercase() {
            return None;
        }

        let mut profile_dir = read_dir(entry.path()).ok()?;

        let profile_str = profile_dir.find_map(|e| {
            let entry = e.ok()?;

            if entry.path().is_dir() {
                return None;
            }

            if entry.file_name().to_string_lossy().to_lowercase() != "profile.json" {
                return None;
            }

            let profile_str = read_to_string(entry.path()).ok()?;

            Some(profile_str)
        })?;

        let Ok(profile): Result<Profile, _> = serde_json::from_str(&profile_str) else {
            return None;
        };

        Some(profile)
    })?;

    Some(profile)
}

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
    B: Into<iced::Background> + Clone + 'a,
    C: Into<iced::Color> + Clone + 'a,
    R: Into<Radius> + Clone + 'a,
    V: Into<iced::Vector<f32>> + Clone + 'a,
{
    Container::<Message>::new(content).style(move |theme| {
        let palette = theme.palette();
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
            None => palette.text,
        };

        let background = match background.clone() {
            Some(background) => background.into(),
            None => palette.background.into(),
        };

        iced::widget::container::Style {
            background: Some(background),
            text_color: Some(text_color),

            border: Border {
                color: border_color,
                width: border_width.unwrap_or(1.0),
                radius: border_radius,
            },

            shadow: Shadow {
                color: shadow_color,
                offset: shadow_offset,
                blur_radius: shadow_blur_radius.unwrap_or(1.0),
            },
        }
    })
}

#[allow(clippy::too_many_arguments)]
pub fn svg_with_color<'a, C>(handle: impl Into<iced_core::svg::Handle>, color_idle: Option<C>, color_hovered: Option<C>) -> Svg<'a>
where
    C: Into<iced::Color> + Clone + 'a,
{
    Svg::new(handle).style(move |theme: &iced::Theme, status| {
        let palette = theme.palette();

        let idle_color = match color_idle.clone() {
            Some(idle_color) => idle_color.into(),
            None => palette.text,
        };

        let hovered_color = match color_hovered.clone() {
            Some(hovered_color) => hovered_color.into(),
            None => palette.text,
        };

        iced::widget::svg::Style {
            color: match status {
                iced::widget::svg::Status::Idle => Some(idle_color),
                iced::widget::svg::Status::Hovered => Some(hovered_color),
            },
        }
    })
}
