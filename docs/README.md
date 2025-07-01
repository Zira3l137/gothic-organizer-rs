# Gothic Organizer Documentation

This document provides an overview of the Gothic Organizer project, its structure, logic, and data handling model.

## Project Structure

The project is organized into the following main directories:

- `src`: Contains the source code of the application.
- `resources`: Contains static assets like icons and images.
- `docs`: Contains this documentation.

### `src` Directory

The `src` directory is further divided into the following modules:

- `app.rs`: The main application file, containing the `GothicOrganizer` struct, which holds the application's state, and the `Message` enum, which defines all possible user interactions.
- `config.rs`: Defines the structures for application configuration (`AppConfig`) and session data (`Session`), which are serialized to and deserialized from JSON files.
- `core`: Contains the core logic of the application.
- `error.rs`: Defines the custom error types used throughout the application.
- `gui`: Contains the UI components of the application, built with the `iced` framework.
- `main.rs`: The entry point of the application, responsible for parsing command-line arguments, setting up the logger, and running the `iced` application.
- `macros.rs`: Contains utility macros to simplify common tasks like creating styled widgets and saving/loading configuration and session data.

### `core` Directory

The `core` directory contains the heart of the application's logic and is structured as follows:

- `constants.rs`: Defines application-wide constants, such as the application name, version, and paths to data directories.
- `helpers.rs`: Provides helper functions for loading and saving configuration, session data, and profiles. It also includes helpers for creating styled UI components.
- `logic`: Contains the business logic of the application, separated into modules for different concerns.
- `lookup.rs`: Implements a custom `Lookup` data structure, which is a wrapper around `hashbrown::HashMap` with a more convenient API for the application's needs.
- `profile.rs`: Defines the data structures for profiles, instances, mods, and file information.
- `utils.rs`: Contains utility functions for file system operations, such as copying files recursively and extracting zip archives.

### `gui` Directory

The `gui` directory contains the UI components of the application:

- `editor_view.rs`: Defines the main view of the application, where users can manage their game profiles, instances, and mods.
- `options_view.rs`: Defines the options view, where users can configure the application's theme and other settings.
- `custom_widgets`: Contains custom `iced` widgets, such as `ClickableText`.

## Logic and Data Handling

The application's logic is primarily handled within the `core::logic` module, which is divided into the following sub-modules:

- `app_lifecycle.rs`: Manages the application's lifecycle, including initialization, session loading/saving, and exiting.
- `mod_management.rs`: Handles the logic for adding, removing, and loading mods.
- `profile_management.rs`: Manages game profiles and instances, including creating, deleting, and switching between them.
- `ui_logic.rs`: Contains logic related to the user interface, such as loading and displaying files and directories.

### Data Model

The application's data is organized around the concept of **profiles**. A profile represents a specific game installation (e.g., Gothic, Gothic 2 Classic). Each profile can have multiple **instances**, which are different configurations of the game with their own set of mods and file overrides.

The main data structures are defined in `core/profile.rs`:

- `Profile`: Represents a game profile, containing its name, path to the game directory, and a collection of instances.
- `Instance`: Represents a specific configuration of a profile, containing its name, a list of mods, and a cache of file information.
- `ModInfo`: Contains information about a mod, including its name, path, and a list of its files.
- `FileInfo`: Represents a single file, containing its source and target paths, and whether it is enabled or not.

### Displaying Directory Entries and Mods

The application displays directory entries and mods in the `editor_view`. The logic for this is handled in `core::logic::ui_logic.rs` and `gui::editor_view.rs`.

When a user selects a profile and an instance, the `load_files` function in `ui_logic.rs` is called. This function populates the `current_directory_entries` field in the application's state with the files and directories of the currently selected instance.

The `editor_view` then iterates over `current_directory_entries` and creates a UI element for each entry. Directories are displayed with a folder icon and are clickable, allowing the user to navigate the file system. Files are displayed with a file icon.

Mods are displayed in a separate list in the `editor_view`. The `mod_management.rs` module handles the logic for adding, removing, and loading mods. When a mod is added, its files are extracted to a dedicated storage directory, and a `ModInfo` struct is created to track the mod's information.

The `FileInfo` for each file contains a `parent_name` field, which indicates which mod the file belongs to. This information is displayed in a tooltip when the user hovers over a file in the `editor_view`. This allows users to easily see which files are part of the base game and which are from mods.
