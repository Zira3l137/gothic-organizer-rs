/// Creates a container with optional arguments for styling
/// # Example
/// ```rust
/// let container = styled_container(
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
        $crate::helpers::styled_container(
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

/// This macro is a shortcut for
/// ```
/// fn save_session<P: AsRef<Path>>(
///     selected_profile: Option<String>,
///     selected_instance: Option<String>,
///     custom_path: Option<P>
/// ) -> Result<(), std::io::Error>
/// ```
/// where `$selected_profile` and `$selected_instance` are the selected profile and instance **names**.
/// Optionally `$custom_path` can be provided.
#[macro_export]
macro_rules! save_session {
    ($selected_profile: expr, $selected_instance: expr, $custom_path: expr) => {
        $crate::helpers::save_session($selected_profile, $selected_instance, $custom_path)
    };
    ($selected_profile: expr, $selected_instance: expr) => {
        $crate::helpers::save_session::<String>($selected_profile, $selected_instance, None)
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
        $crate::helpers::load_session::<String>(None)
    };

    ($custom_path: expr) => {
        $crate::helpers::load_session($custom_path)
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
        $crate::helpers::load_profile::<String>($name, None)
    };
    ($name: expr, $custom_path: expr) => {
        $crate::helpers::load_profile($name, $custom_path)
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
        $crate::helpers::save_profile::<String>($profile, None)
    };
    ($profile: expr, $custom_path: expr) => {
        $crate::helpers::save_profile($profile, $custom_path)
    };
}

/// Returns the path to the local profiles directory.
#[macro_export]
macro_rules! local_profiles_dir {
    () => {
        std::path::PathBuf::from($crate::profile::local_app_data()).join($crate::constants::APP_NAME)
    };
}
