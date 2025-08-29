use std::process;

pub mod context;
pub mod mods;
pub mod profile;
pub mod session;
pub mod ui;

pub trait Service {
    fn context(&mut self) -> Result<context::Context, crate::error::Error>;
}

pub fn execute_cmd(cmd: &str, args: &[&str]) -> Result<String, crate::error::Error> {
    let output = process::Command::new(cmd).args(args).output()?;
    if !output.stderr.is_empty() {
        Err(crate::error::Error::system(
            String::from_utf8_lossy(&output.stderr).to_string(),
            "Execute Command".to_string(),
        ))
    } else if !output.stdout.is_empty() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Ok(String::new())
    }
}

pub fn browser_open(url: &str) -> Result<String, crate::error::Error> {
    #[cfg(target_os = "windows")]
    {
        execute_cmd("explorer", &[url])
    }

    #[cfg(target_os = "linux")]
    {
        execute_cmd("xdg-open", &[url])
    }
}
