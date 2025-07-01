/// Creates a container with optional arguments for styling
/// # Example
/// ```rust
/// let container = styled_container!(
///     text("Hello"),
///     border_width = 2.0,
///     border_color = iced::Color::BLACK,
///     border_radius = 4.0,
///     shadow_blur_radius = 2.0,
///     shadow_color = iced::Color::BLACK,
///     shadow_offset = iced::Vector::new(2.0, 2.0),
///     text_color = iced::Color::BLACK,
///     background = iced::Background::Color(iced::Color::WHITE),
/// );
///```
#[macro_export]
macro_rules! styled_container {
    (
        $content:expr
        $(, border_width = $border_width:expr)?
        $(, border_color = $border_color:expr)?
        $(, border_radius = $border_radius:expr)?
        $(, shadow_blur_radius = $shadow_blur_radius:expr)?
        $(, shadow_color = $shadow_color:expr)?
        $(, shadow_offset = $shadow_offset:expr)?
        $(, text_color = $text_color:expr)?
        $(, background = $background:expr)?
        $(,)?
    ) => {
        $crate::core::helpers::styled_container(
            $content,
            styled_container!(@some_opt $($border_width)?; f32),
            styled_container!(@some_opt $($border_color)?; iced::Color),
            styled_container!(@some_opt $($border_radius)?; iced::border::Radius),
            styled_container!(@some_opt $($shadow_blur_radius)?; f32),
            styled_container!(@some_opt $($shadow_color)?; iced::Color),
            styled_container!(@some_opt $($shadow_offset)?; iced::Vector<f32>),
            styled_container!(@some_opt $($text_color)?; iced::Color),
            styled_container!(@some_opt $($background)?; iced::Background),
        )
    };

    // Helper to wrap with Some(...) or None::<Type>
    (@some_opt $val:expr; $ty:ty) => {
        Some($val)
    };
    (@some_opt ; $ty:ty) => {
        None::<$ty>
    };
}

/// Creates an svg widget with optional arguments for color
/// # Example
/// ```rust
/// let svg = svg_with_color!(
///     "ferris_nude_pic.svg",
///     color_idle = iced::Color::BLACK,
///     color_hovered = iced::Color::WHITE
/// );
///```
#[macro_export]
macro_rules! svg_with_color {
    (
        $content:expr
        $(, color_idle = $color_idle:expr)?
        $(, color_hovered = $color_hovered:expr)?
        $(,)?
    ) => {
        $crate::core::helpers::svg_with_color(
            $content,
            styled_container!(@some_opt $($color_idle)?; iced::Color),
            styled_container!(@some_opt $($color_hovered)?; iced::Color),
        )
    };

    // Helper to wrap with Some(...) or None::<Type>
    (@some_opt $val:expr; $ty:ty) => {
        Some($val)
    };
    (@some_opt ; $ty:ty) => {
        None::<$ty>
    };
}

/// This macro is a shortcut for
/// ```
/// pub fn save_config<P: AsRef<Path>>(
///    theme: Option<String>,
///    mod_storage_dir: Option<PathBuf>,
///    custom_path: Option<P>,
///) -> Result<(), error::GothicOrganizerError>
/// ```
/// where `$theme` and `$mod_storage_dir` are the selected theme name and mod storage directory path.
/// Optionally `$custom_path` can be provided for config.
#[macro_export]
macro_rules! save_config {
    ($theme: expr, $mod_storage_dir: expr, $custom_path: expr) => {
        $crate::helpers::save_config($theme, $mod_storage_dir, $custom_path)
    };
    ($theme: expr, $mod_storage_dir: expr) => {
        $crate::core::helpers::save_config::<String>($theme, $mod_storage_dir, None)
    };
}

/// This macro is a shortcut for
/// ```
/// pub fn load_config<P: AsRef<Path>>(custom_path: Option<P>) -> Option<config::AppConfig>
/// ```
/// Optionally `$custom_path` can be provided for config.
/// Returns an `Option<config::AppConfig>`
#[macro_export]
macro_rules! load_config {
    () => {
        $crate::core::helpers::load_config::<String>(None)
    };

    ($custom_path: expr) => {
        $crate::core::helpers::load_config($custom_path)
    };
}

/// This macro is a shortcut for
/// ```
/// pub fn save_session<P: AsRef<Path>>(
///     selected_profile: Option<String>,
///     selected_instance: Option<String>,
///     cache: Option<Lookup<PathBuf, profile::FileInfo>>,
///     custom_path: Option<P>,
/// ) -> Result<(), std::io::Error>
/// ```
/// where `$selected_profile` and `$selected_instance` are the selected profile and instance **names**.
/// Optionally `$custom_path` can be provided.
#[macro_export]
macro_rules! save_session {
    ($selected_profile: expr, $selected_instance: expr, $cache: expr, $custom_path: expr) => {
        $crate::helpers::save_session($selected_profile, $selected_instance, $cache, $custom_path)
    };
    ($selected_profile: expr, $selected_instance: expr, $cache: expr) => {
        $crate::core::helpers::save_session::<String>($selected_profile, $selected_instance, $cache, None)
    };
}

/// This macro is a shortcut for
/// ```
/// fn load_session<P: AsRef<Path>>(custom_path: Option<P>) -> Option<Session>
/// ```
/// Optionally `$custom_path` can be provided.
/// Returns an `Option<Session>`
#[macro_export]
macro_rules! load_session {
    () => {
        $crate::core::helpers::load_session::<String>(None)
    };

    ($custom_path: expr) => {
        $crate::core::helpers::load_session($custom_path)
    };
}

/// This macro is a shortcut for
/// ```
/// fn load_profile<P: AsRef<Path>>(name: &str, custom_path: Option<P>) -> Option<Profile>
/// ```
/// where `$name` is the name of the profile.
/// Optionally `$custom_path` can be provided.
/// Returns an `Option<Profile>`
#[macro_export]
macro_rules! load_profile {
    ($name: expr) => {
        $crate::core::helpers::load_profile::<String>($name, None)
    };
    ($name: expr, $custom_path: expr) => {
        $crate::core::helpers::load_profile($name, $custom_path)
    };
}

/// This macro is a shortcut for
/// ```
/// fn save_profile<P: AsRef<Path>>(profile: &Profile, custom_path: Option<P>) -> Result<(),
/// std::io::Error>
/// ```
/// where `$profile` is the profile.
/// Optionally `$custom_path` can be provided.
/// Saves the profile on a disk and returns a `Result<(), std::io::Error>`
#[macro_export]
macro_rules! save_profile {
    ($profile: expr) => {
        $crate::core::helpers::save_profile::<String>($profile, None)
    };
    ($profile: expr, $custom_path: expr) => {
        $crate::core::helpers::save_profile($profile, $custom_path)
    };
}
