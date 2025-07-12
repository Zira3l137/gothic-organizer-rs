use std::process;

pub mod context;
pub mod mods;
pub mod profile;
pub mod session;
pub mod ui;

pub trait Service {
    fn context(&mut self) -> Result<context::Context, crate::error::GothicOrganizerError>;
}

pub fn execute_cmd(cmd: &str, args: &[&str]) -> Result<String, crate::error::GothicOrganizerError> {
    let output = process::Command::new(cmd).args(args).output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(crate::error::GothicOrganizerError::Other(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

pub fn browser_open(url: &str) -> Result<String, crate::error::GothicOrganizerError> {
    #[cfg(target_os = "windows")]
    {
        execute_cmd("explorer", &[url])
    }

    #[cfg(target_os = "linux")]
    {
        execute_cmd("xdg-open", &[url])
    }
}
