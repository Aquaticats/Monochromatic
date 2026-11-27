//! What:
//!  Everything the index update reads besides the index file:
//!  the layout,
//!  the settings,
//!  the object database,
//!  and the environment Git would be started with.
//! Why:
//!  Any index form,
//!  configuration or hook the port does not reproduce returns nothing and the
//!  caller runs Git's own command,
//!  so the update reads the layout and the settings first,
//!  declines on a hook that Git would run and on a monitor variable whose
//!  daemon this update never asks,
//!  and hands the one value its caller needs rather than three.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const around = surroundings(git, repository); // { layout, settings, database } | undefined
//! ```

/// The hook Git runs after it writes an index,
/// which this update never runs.
use super::index_reset_file_names_and_checksums::POST_INDEX_HOOK;
use crate::index_settings::{
    IndexSettings,
    read_settings,
};
use crate::repo_read_config_list::list_config;
use crate::repo_read_config_scan::ScopeEnvironment;
/// The hook the update declines on.
use crate::repo_read_hook::enabled;
/// The layout,
/// the configuration listing and the settings made from it.
use crate::repo_read_layout::{
    RepoLayout,
    read_layout_with,
};
/// The objects the reset's entries are read from.
use crate::repo_read_odb::ObjectDatabase;
/// The environment Git would be started with,
/// which the update reads instead of starting it.
use crate::transaction_git::GitContext;
/// The variables a run adds to this process's own,
/// and the paths it works on.
use std::ffi::OsString;
use std::path::Path;
/// Everything the update reads besides the index file:
/// the layout,
/// the settings and the object database.
pub(super) struct Surroundings {
    /// Where the repository is.
    pub(super) layout: RepoLayout,
    /// What the configuration makes of the extensions.
    pub(super) settings: IndexSettings,
    /// The objects.
    pub(super) database: ObjectDatabase,
}

/// This process's environment with `added` laid over it,
/// minus the two variables that would point Git at other files.
fn effective_environment(added: Vec<(OsString, OsString)>) -> Vec<(OsString, OsString)> {
    let mut effective: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in std::env::vars_os() {
        // A variable the run adds replaces the one of this process.
        let mut replaced: bool = false;
        for (own, _) in &added {
            if *own == name {
                replaced = true;
                break;
            }
        }
        if !replaced {
            effective.push((name, value));
        }
    }
    effective.extend(added);
    let mut kept: Vec<(OsString, OsString)> = Vec::new();
    for (name, value) in effective {
        if name != "GIT_INDEX_FILE" && name != "GIT_OBJECT_DIRECTORY" {
            kept.push((name, value));
        }
    }
    return kept;
}

/// Whether the environment sets `GIT_TEST_FSMONITOR` to something,
/// which would turn a file system monitor on.
fn turns_monitor_on(effective: &[(OsString, OsString)]) -> bool {
    for (name, value) in effective {
        if name == "GIT_TEST_FSMONITOR" && !value.is_empty() {
            return true;
        }
    }
    return false;
}

/// Read the layout,
/// the configuration and the objects,
/// or decline.
pub(super) fn surroundings(
    git: &GitContext,
    repository: &Path,
) -> Option<Surroundings> {
    let added: Vec<(OsString, OsString)> =
        crate::git_system_config::with_real_git(git.overlay.as_slice(), git.real_git.as_path());
    if !enabled(added.as_slice()) {
        return None;
    }
    let effective: Vec<(OsString, OsString)> = effective_environment(added);
    // Git reads this variable to turn a monitor on; its effect depends on a daemon this update never asks.
    if turns_monitor_on(effective.as_slice()) {
        return None;
    }
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(effective.as_slice());
    let layout: RepoLayout = read_layout_with(repository, effective.as_slice(), &scope)?;
    let listed = list_config(
        layout.common_dir.as_path(),
        layout.git_dir.as_path(),
        &scope,
        effective.as_slice(),
    )?;
    let settings: IndexSettings = read_settings(listed.as_slice())?;
    if layout
        .common_dir
        .join("hooks")
        .join(POST_INDEX_HOOK)
        .symlink_metadata()
        .is_ok()
    {
        return None;
    }
    let database: ObjectDatabase =
        ObjectDatabase::open_where_the_layout_proved_the_store(&layout, None)?;
    return Some(Surroundings {
        layout,
        settings,
        database,
    });
}
