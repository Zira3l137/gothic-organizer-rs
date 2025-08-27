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
            $crate::svg_with_color!(@some_opt $($color_idle)?; iced::Color),
            $crate::svg_with_color!(@some_opt $($color_hovered)?; iced::Color),
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

/// Creates a new [`ClickableText`] widget with the provided content.
///
/// [`ClickableText`]: gui::custom_widgets::clickable_text::ClickableText
///
/// This macro uses the same syntax as [`format!`], but creates a new [`ClickableText`] widget instead.
///
/// See [the formatting documentation in `std::fmt`](std::fmt)
/// for details of the macro argument syntax.
///
#[macro_export]
macro_rules! clickable_text {
    ($($arg:tt)*) => {
        $crate::gui::custom_widgets::clickable_text(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! save_app_preferences {
    ($theme: expr, $mod_storage_dir: expr, $custom_path: expr) => {
        $crate::helpers::save_app_preferences($theme, $mod_storage_dir, $custom_path)
    };
    ($theme: expr, $mod_storage_dir: expr) => {
        $crate::core::helpers::save_app_preferences::<String>($theme, $mod_storage_dir, None)
    };
}

#[macro_export]
macro_rules! load_app_preferences {
    () => {
        $crate::core::helpers::load_app_preferences::<String>(None)
    };

    ($custom_path: expr) => {
        $crate::core::helpers::load_app_preferences($custom_path)
    };
}

#[macro_export]
macro_rules! save_app_session {
    ($session: expr, $custom_path: expr) => {
        $crate::core::helpers::save_app_session($session, $custom_path)
    };
    ($session: expr) => {
        $crate::core::helpers::save_app_session::<String>($session, None)
    };
}

#[macro_export]
macro_rules! load_app_session {
    () => {
        $crate::core::helpers::load_app_session::<String>(None)
    };

    ($custom_path: expr) => {
        $crate::core::helpers::load_app_session($custom_path)
    };
}

#[macro_export]
macro_rules! save_profile {
    ($profile: expr) => {
        $crate::core::helpers::save_profile::<String>($profile, None)
    };
    ($profile: expr, $custom_path: expr) => {
        $crate::core::helpers::save_profile($profile, $custom_path)
    };
}

#[macro_export]
macro_rules! load_profile {
    ($name: expr) => {
        $crate::core::helpers::load_profile::<String>($name, None)
    };
    ($name: expr, $custom_path: expr) => {
        $crate::core::helpers::load_profile($name, $custom_path)
    };
}

#[macro_export]
macro_rules! impl_service {
    ($service:ident) => {
        impl Service for $service<'_> {
            fn context(
                &mut self,
            ) -> Result<$crate::core::services::context::Context, $crate::error::AppError> {
                let profile = self
                    .session
                    .active_profile
                    .as_mut()
                    .and_then(|p| self.state.profile.profiles.get_mut(&p.clone()))
                    .ok_or_else(|| $crate::error::AppError::ProfileService {
                        operation: $crate::error::ProfileOperation::Load,
                        details: "Failed to get active profile".to_string(),
                    })?;

                let instance_name = self.session.active_instance.clone().unwrap_or_default();

                Ok($crate::core::services::context::Context::new(profile, &instance_name))
            }
        }
    };
}

#[macro_export]
macro_rules! lookup {
    [($($key: expr => $value: expr),*)] => {
        {
            let mut lookup = $crate::core::lookup::Lookup::new();
            $(
                lookup.insert($key, $value);
            )*
            lookup
        }
    };
}
