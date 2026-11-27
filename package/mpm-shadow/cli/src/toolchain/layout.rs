//! What:
//! Where things are:
//! the host's platform id,
//! the install directory of one tool version,
//! the directories each
//!           route puts its binaries in,
//! and the bounded search for an executable inside them.
//! Why:
//! `list`,
//! `install`,
//! the exposure farm and `bin` must agree on where a tool lives;
//! they all ask here.
//! The answers are one file each,
//! named for what it holds:
//! the host's platform id and OS family (`platform_ids`),
//! the directories one tool version installs and downloads into (`install_directories`),
//! the package managers' home and bin directories (`manager_homes`),
//! and the bounded search for a tool's executable (`bin_search`).
//! A caller imports from this file and never from a part.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // layout.ts
//! export * from './layout/platform_ids.ts';
//! export * from './layout/install_directories.ts';
//! export * from './layout/manager_homes.ts';
//! export * from './layout/bin_search.ts';
//! ```

/// The search for a tool's executable.
mod bin_search;
/// The directories one tool version installs and downloads into.
mod install_directories;
/// The package managers' home and bin directories.
mod manager_homes;
/// The host's platform id and OS family.
mod platform_ids;

/// Re-exports the search directories,
/// the executable check and the tool-bin search.
pub use self::bin_search::{
    bin_search_dirs,
    find_bin_under,
    find_tool_bin,
    is_executable,
};
/// Re-exports the install directory of one tool version,
/// the Go `bin` directory and the downloads' directory.
pub use self::install_directories::{
    GO_BIN_DIRECTORY_NAME,
    go_bin_dir_of,
    install_dir_name,
    tool_directory,
    tool_downloads_dir,
    tool_install_dir,
};
/// Re-exports the package managers' home and bin directories.
pub use self::manager_homes::{
    cargo_bin_dir,
    cargo_home_dir,
    manager_native_bin_dirs,
    physical_pnpm_home,
};
/// Re-exports the host's platform id,
/// its OS family and the OS filter's verdict.
pub use self::platform_ids::{
    applies_to_family,
    current_family,
    current_platform_id,
    platform_id,
};
