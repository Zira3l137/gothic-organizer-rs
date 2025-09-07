use std::process;

pub mod mods;
pub mod profile;
pub mod session;
pub mod ui;

pub fn execute_cmd(cmd: &str, args: &[&str]) -> Result<String, crate::error::Error> {
    let output = process::Command::new(cmd).args(args).output()?;
    if !output.stderr.is_empty() {
        Err(crate::error::Error::system(
            String::from_utf8_lossy(&output.stderr).to_string(),
            "Execute Command",
        ))
    } else if !output.stdout.is_empty() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Ok(String::new())
    }
}
