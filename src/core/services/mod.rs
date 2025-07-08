pub mod context;
pub mod mod_service;
pub mod profile_service;
pub mod session_service;
pub mod ui_service;

pub trait Service {
    fn context(&mut self) -> Result<context::Context, crate::error::GothicOrganizerError>;
}
