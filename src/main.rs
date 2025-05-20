mod application;
mod constants;
mod error;
mod profile;
mod startup_window;

use crate::startup_window::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    println!("Hello, Auronen!");

    let previous_session = load_session!();
    let session = match previous_session {
        Some(s) => s,

        None => {
            let mut startup_window = StartupWindow::new();
            startup_window.run()?;

            if startup_window.canceled {
                return Ok(());
            }

            let Some(selected_profile) = &startup_window.selected_profile else {
                return Ok(());
            };

            let Some(selected_instance) = &startup_window.selected_instance else {
                return Ok(());
            };

            save_session!(
                Some(selected_profile.clone()),
                Some(selected_instance.clone())
            )?;

            load_session!().unwrap()
        }
    };

    println!("\nSession: {:?}", session);

    Ok(())
}
