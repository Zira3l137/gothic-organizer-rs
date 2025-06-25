pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_TITLE: &str = "Gothic Organizer";
pub const APP_AUTHOR: &str = "Zira3l137";
pub const APP_REPOSITORY: &str = "https://github.com/Zira3l137/gothic-organizer-rs";

pub fn app_title_full() -> String {
    format!("{} v{}", APP_TITLE, APP_VERSION)
}

pub fn app_info() -> String {
    format!(
        "{}\nVersion: {}\nAuthor: {}\nRepository: {}",
        APP_TITLE, APP_VERSION, APP_AUTHOR, APP_REPOSITORY
    )
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
