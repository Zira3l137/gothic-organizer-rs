mod application;
mod constants;
mod editor;
mod error;
mod profile;
mod startup_window;

use crate::editor::prelude::*;
use crate::startup_window::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    println!("Hello, Auronen!");

    let previous_session = load_session!();
    let session = match previous_session {
        Some(s) => s,

        None => {
            let mut startup_wnd = StartupWindow::new();
            startup_wnd.run()?;

            if startup_wnd.canceled {
                return Ok(());
            }

            save_session!(
                Some(startup_wnd.selected_profile()),
                Some(startup_wnd.selected_instance()),
                Some(startup_wnd.available_profiles()),
                Some(startup_wnd.available_instances())
            )?;

            load_session!().unwrap()
        }
    };

    let mut editor = EditorWindow::new(session);
    editor.run()?;

    Ok(())
}
