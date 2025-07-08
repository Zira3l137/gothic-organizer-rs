# Service Interactions and Logic

This document provides a detailed explanation of how the modules within `src/core/services` interact with the application's state, managed primarily by `SessionService`. It's intended as a reference for developers to understand the flow of data and state modifications.

## `SessionService` - The Central State Manager

The `SessionService` is the owner of the application's core state.

-   **Core State:**
    -   `profiles: Lookup<String, profile::Profile>`: Holds all loaded game profiles. This is the primary source of truth for profiles, instances, and their configurations. It's persisted to disk.
    -   `files: Lookup<PathBuf, profile::FileInfo>`: A flat cache of all file information for the *currently active context*. This is a critical, mutable piece of state that is frequently updated by other services.
    -   `active_profile: Option<String>`: The name of the currently active profile.
    -   `active_instance: Option<String>`: The name of the currently active instance.
    -   `mod_storage_dir: Option<PathBuf>`: The root directory where mod files are stored.
    -   `theme_selected: Option<String>`: The name of the active UI theme.

### Key `SessionService` Methods

-   **`try_reload_last_session()`**
    -   **Purpose:** To restore the application to its last saved state on startup.
    -   **Interactions:**
        -   Loads all profiles from disk into `self.profiles`.
        -   Reads `session.json` to set `self.active_profile` and `self.active_instance`.
        -   If an instance was active, it loads the file cache (`instance.files`) into the main `self.files` state.
        -   Reads `config.json` to set the theme and mod storage path.

-   **`save_current_session()`**
    -   **Purpose:** To persist the current application state to disk.
    -   **Interactions:**
        -   Saves each profile in `self.profiles` to its respective `profile.json`.
        -   Saves the current session information to `session.json`.
        -   Saves the application configuration to `config.json`.

-   **`exit()`**
    -   **Purpose:** To gracefully shut down the application.
    -   **Interactions:**
        -   Calls `save_current_session()` to persist everything to disk.

## The `Context` Object

Services do not get direct access to the entire `SessionService`. Instead, they create a short-lived `Context` object.

-   **Purpose:** The `Context` provides mutable, scoped access to the currently active profile and instance within the `SessionService`.
-   **Safety:** This prevents services from accidentally modifying state outside of the current user context (e.g., changing an inactive profile).

## `ProfileService`

Handles all logic related to creating, modifying, and switching between profiles and instances.

-   **`switch_profile(...)` & `switch_instance(...)`**
    -   **Purpose:** To change the active profile or instance.
    -   **Interactions:**
        -   **Crucial Nuance:** Before switching, it calls `update_instance_from_cache()` to save the state of the *outgoing* instance.
        -   Modifies `session.active_profile` and `session.active_instance`.
        -   Updates the UI choices for instances.
        -   Dispatches `Message::CurrentDirectoryUpdated` to trigger `UiService` to reload the file view.

-   **`add_instance_for_profile(...)`**
    -   **Purpose:** To create a new, clean instance for a profile.
    -   **Interactions:**
        -   **Initializes a new `Instance` with a fresh copy of the base game files**, ensuring no state is carried over from a previously active instance.
        -   Adds the new instance to the current profile.

-   **`update_instance_from_cache()`**
    -   **Purpose:** To persist changes made in the UI (like toggling files) from the temporary UI state back into the main data model. This is a critical "commit" step that happens before any context switch or shutdown.
    -   **Interactions:**
        -   Merges `app.state.current_directory_entries` back into `session.files`.
        -   Saves the entire `session.files` cache into `instance.files` for the active instance.

-   **`set_game_dir(...)`**
    -   **Purpose:** To associate a game directory with a profile.
    -   **Interactions:**
        -   Modifies `profile.path` for the current profile.
        -   **Clears and Repopulates `session.files`**: It walks the entire new game directory and populates the `session.files` cache from scratch.

## `ModService`

Handles adding, removing, and applying mods to an instance.

-   **`add_mod(...)`**
    -   **Purpose:** To add a new mod to the selected instance.
    -   **Interactions:**
        -   Moves the mod files to a designated, instance-specific storage location: `$mod_storage_dir\$profile_name\$instance_name`.
        -   Creates a new `ModInfo` struct and adds it to the instance's `mods` list.
        -   Calls `apply_mod_files` to apply the new mod's files to the instance's file cache.

-   **`toggle_mod(...)`**
    -   **Purpose:** To enable or disable a mod.
    -   **Interactions:**
        -   **Idempotent:** Checks if the mod's state is already the desired state and returns early if so.
        -   Calls `apply_mod_files` or `unapply_mod_files` to modify the instance's file cache.
        -   Updates the `enabled` flag on the `ModInfo` struct.

-   **`remove_mod(...)`**
    -   **Purpose:** To remove a mod from an instance and delete its files.
    -   **Interactions:**
        -   Calls `toggle_mod` to disable the mod first, ensuring its files are unapplied cleanly.
        -   Removes the mod's files from the storage directory.
        -   Removes the `ModInfo` struct from the instance's `mods` list.
        -   Removes the mod's overwrites from the instance's `overwrites` map.

-   **`reload_mods()`**
    -   **Purpose:** To re-apply the files from all active mods to the instance's file cache. This is where file overwrites are handled.
    -   **Interactions:**
        -   Clears `instance.overwrites`.
        -   Iterates through each **enabled** `ModInfo` in `instance.mods` and applies its files using `apply_mod_files`. This correctly updates the `instance.files` and `instance.overwrites` data.

## `UiService`

Acts as the view-model layer, preparing data from the core model for display in the GUI.

-   **`reload_displayed_directory(...)`**
    -   **Purpose:** To populate the file list (`current_directory_entries`) that the user sees.
    -   **Interactions:**
        -   Gets the active instance's files (`instance.files`).
        -   Clears and extends the main `session.files` cache with the instance's files.
        -   Filters `session.files` to get only the items in the `current_directory`, sorts them, and stores the result in `app.state.current_directory_entries` for the UI to render.

-   **`toggle_state_recursive(...)`**
    -   **Purpose:** To handle the user checking/unchecking a file or directory in the UI.
    -   **Interactions:**
        -   **Modifies `app.state.current_directory_entries`**: It immediately flips the `enabled` flag on the item in the list the user is looking at, providing instant visual feedback.
        -   **Modifies `session.files`**: If the toggled item is a directory, it performs a recursive update on the main `session.files` cache, ensuring the change is captured in the underlying data model.