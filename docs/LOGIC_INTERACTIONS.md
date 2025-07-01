# Logic and `GothicOrganizer` Interaction Cheatsheet

This document provides a detailed explanation of how the modules within `src/core/logic` interact with the main `GothicOrganizer` struct. It's intended as a reference for developers to understand the flow of data and state modifications.

## `GothicOrganizer` Struct Overview

The `GothicOrganizer` struct is the central state holder for the entire application. Its fields can be categorized into core data, session state, and UI state.

-   **Core Data & Session State:**
    -   `profiles: Lookup<String, profile::Profile>`: Holds all loaded game profiles. This is the primary source of truth for profiles, instances, and their configurations. It's persisted to disk.
    -   `files: Lookup<PathBuf, profile::FileInfo>`: A flat cache of all file information for the *currently active context*. If an instance is selected, this holds the merged view of base game files and all active mod files for that instance. If no instance is selected, it holds only the base game files for the selected profile. This is a critical, mutable piece of state that is frequently updated.
    -   `profile_selected: Option<String>`: The name of the currently active profile.
    -   `instance_selected: Option<String>`: The name of the currently active instance.
    -   `mod_storage_dir: Option<PathBuf>`: The root directory where mod files are stored.
    -   `theme: Option<String>`: The name of the active UI theme.

-   **UI State (`InnerState` struct):**
    -   `current_directory: PathBuf`: The absolute path of the directory currently being displayed in the file editor view.
    -   `current_directory_entries: Vec<(PathBuf, profile::FileInfo)>`: A filtered and sorted list of items from `GothicOrganizer::files` that are direct children of `current_directory`. This is what is directly rendered in the UI.
    -   Other fields in `InnerState` are mostly related to `iced` widget state (like `combo_box::State`) or temporary user input.

## Interaction by Module

### `app_lifecycle.rs`

This module manages the application's startup, shutdown, and session persistence.

-   **`try_reload_last_session(app: &mut GothicOrganizer)`**
    -   **Purpose:** To restore the application to its last saved state on startup.
    -   **Interactions:**
        -   **Modifies `app.profiles` and `app.state.profile_choices`**: Pre-loads all profiles from disk.
        -   **Modifies `app.profile_selected`, `app.instance_selected`, `app.files`**: Reads `session.json` and sets the active profile and instance. If an instance was active, it loads the file cache (`instance.files`) into the main `app.files` state.
        -   **Modifies `app.theme`, `app.mod_storage_dir`**: Reads `config.json` to set the theme and mod storage path.

-   **`save_current_session(app: &GothicOrganizer)`**
    -   **Purpose:** To persist the current application state to disk.
    -   **Interactions:**
        -   **Reads `app.profiles`**: Iterates through all profiles and saves each one to its respective `profile.json`.
        -   **Reads `app.profile_selected`, `app.instance_selected`, `app.files`**: Saves the current session information to `session.json`. It intelligently decides whether to save the `app.files` cache. The cache is saved only if no instances are being used for the current profile, otherwise, the file cache is persisted within the instance itself.
        -   **Reads `app.theme`, `app.mod_storage_dir`**: Saves the application configuration to `config.json`.

-   **`exit(app: &mut GothicOrganizer, ...)`**
    -   **Purpose:** To gracefully shut down the application.
    -   **Interactions:**
        -   Triggers `profile_management::write_changes_to_instance(app)` to ensure any pending UI changes are saved to the data model.
        -   Triggers `save_current_session(app)` to persist everything to disk.

### `profile_management.rs`

This module handles all logic related to creating, modifying, and switching between profiles and instances.

-   **`switch_profile(...)` & `select_instance(...)`**
    -   **Purpose:** To change the active profile or instance.
    -   **Interactions:**
        -   **Crucial Nuance:** Before switching, it calls `write_changes_to_instance(app)` to save the state of the *outgoing* instance.
        -   **Modifies `app.profile_selected` and `app.instance_selected`**.
        -   **Modifies `app.state.instance_choices`** to reflect the instances available in the new profile.
        -   Dispatches `Message::RefreshFiles` to trigger `ui_logic::load_files`, which will load the file view for the new context.

-   **`write_changes_to_instance(app: &mut GothicOrganizer)`**
    -   **Purpose:** To persist changes made in the UI (like toggling files) from the temporary UI state back into the main data model. This is a critical "commit" step that happens before any context switch or shutdown.
    -   **Interactions:**
        -   Finds the currently active profile and instance within `app.profiles`.
        -   **Merges `app.state.current_directory_entries` back into `app.files`**. This updates the main file cache with any changes made in the current view.
        -   **Saves the entire `app.files` cache into `instance.files`** within the `app.profiles` data structure.

-   **`set_game_dir(...)`**
    -   **Purpose:** To associate a game directory with a profile.
    -   **Interactions:**
        -   **Modifies `profile.path`** for the current profile in `app.profiles`.
        -   **Modifies `app.state.current_directory`** to the new game path.
        -   **Clears and Repopulates `app.files`**: It walks the entire new game directory and populates the `app.files` cache from scratch with default `FileInfo`.
        -   Dispatches `Message::RefreshFiles`.

### `mod_management.rs`

This module handles adding, removing, and applying mods to an instance.

-   **`add_mod(...)`**
    -   **Purpose:** To add a new mod to the selected instance.
    -   **Interactions:**
        -   Reads `app.mod_storage_dir` and `app.profile_selected` to determine the correct storage location for the mod's files.
        -   **Modifies `app.profiles`**: It finds the current instance and adds a new `ModInfo` struct to its `mods` list. The `mod_info.files` field is populated by walking the extracted mod archive.
        -   Dispatches `Message::LoadMods`.

-   **`load_mods(app: &mut GothicOrganizer)`**
    -   **Purpose:** To apply the files from all active mods to the instance's file cache. This is where file overwrites are handled.
    -   **Interactions:**
        -   Finds the current instance in `app.profiles`.
        -   **Modifies `instance.files` and `instance.overwrites`**: It iterates through each `ModInfo` in `instance.mods`. For every file in a mod, it attempts to insert it into `instance.files`. If a file already exists at that path, the original `FileInfo` is moved to the `instance.overwrites` lookup, and the new mod file takes its place.
        -   **Nuance:** This function does *not* directly touch `app.files`. It prepares the `instance.files` cache. The UI is updated subsequently when `ui_logic::load_files` is called (usually via a `RefreshFiles` message), which then loads this prepared `instance.files` into the active `app.files`.
        -   Dispatches `Message::RefreshFiles`.

-   **`remove_mod(...)`**
    -   **Purpose:** To remove a mod from an instance and delete its files.
    -   **Interactions:**
        - **Work In Progress...

### `ui_logic.rs`

This module acts as the view-model layer, preparing data from the core model for display in the GUI.

-   **`load_files(app: &mut GothicOrganizer, ...)`**
    -   **Purpose:** To populate the file list (`current_directory_entries`) that the user sees.
    -   **Interactions:**
        -   **Modifies `app.state.current_directory`**.
        -   **Modifies `app.files`**: This is the crucial step where the active file cache is updated. It loads the `instance.files` from the current instance in `app.profiles` into the main `app.files` state, effectively switching the context.
        -   **Modifies `app.state.current_directory_entries`**: It filters the now-updated `app.files` to get only the items in the `current_directory`, sorts them, and stores the result for the UI to render.

-   **`toggle_state_recursive(app: &mut GothicOrganizer, ...)`**
    -   **Purpose:** To handle the user checking/unchecking a file or directory.
    -   **Interactions:**
        -   **Modifies `app.state.current_directory_entries`**: It immediately flips the `enabled` flag on the item in the list the user is looking at, providing instant visual feedback.
        -   **Modifies `app.files`**: If the toggled item is a directory, it performs a recursive update on the main `app.files` cache, setting the `enabled` state for all children of that directory. This ensures the change is captured in the underlying data model, not just the current view.
