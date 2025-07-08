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
- `services`: Contains the business logic of the application, separated into service modules for different concerns.
- `lookup.rs`: Implements a custom `Lookup` data structure, which is a wrapper around `hashbrown::HashMap` with a more convenient API for the application's needs.
- `profile.rs`: Defines the data structures for profiles, instances, mods, and file information.
- `utils.rs`: Contains utility functions for file system operations, such as copying files recursively and extracting zip archives.

### `gui` Directory

The `gui` directory contains the UI components of the application:

- `editor_view.rs`: Defines the main view of the application, where users can manage their game profiles, instances, and mods.
- `options_view.rs`: Defines the options view, where users can configure the application's theme and other settings.
- `custom_widgets`: Contains custom `iced` widgets, such as `ClickableText`.

## Logic and Data Handling

The application's logic is handled by a set of services within the `core::services` module. This service-oriented architecture separates concerns and manages the application's state.

-   **`SessionService`**: The central state manager. It holds all profiles, the active profile/instance selection, and the main file cache. It's responsible for loading and saving the user's session.
-   **`ProfileService`**: Manages profiles and instances. It handles logic for creating, switching, and modifying them.
-   **`ModService`**: Manages mods for the selected instance, including adding, removing, enabling/disabling, and handling file overwrites.
-   **`UiService`**: Prepares data for the UI. It populates the file list view based on the current context (profile/instance).
-   **`Context`**: A temporary object that provides controlled, mutable access to the currently active profile and instance within the `SessionService`'s state. Services use it to ensure they are operating on the correct data.

### Data Model

The application's data is organized around the concept of **profiles**. A profile represents a specific game installation (e.g., Gothic, Gothic 2 Classic). Each profile can have multiple **instances**, which are different configurations of the game with their own set of mods and file overrides.

The main data structures are defined in `core/profile.rs`:

- `Profile`: Represents a game profile, containing its name, path to the game directory, and a collection of instances.
- `Instance`: Represents a specific configuration of a profile, containing its name, a list of mods, and a cache of file information.
- `ModInfo`: Contains information about a mod, including its name, path, and a list of its files.
- `FileInfo`: Represents a single file, containing its source and target paths, and whether it is enabled or not.

The application uses a custom `Lookup<K, V>` data structure, which is a wrapper around `hashbrown::HashMap` using the `ahash` algorithm for performance.

### File Overwrites

When a mod is loaded, its files may overwrite existing files from the base game or other mods. The application handles this by storing the overwritten files in the `overwrites` field of the `Instance` struct. This field is a `Lookup<String, Lookup<PathBuf, FileInfo>>`, which maps a mod's name to a lookup of the files it has overwritten. This allows the application to restore the original files when a mod is disabled or removed.

The `apply_mod_files` and `unapply_mod_files` functions in `ModService` are responsible for managing this process.
