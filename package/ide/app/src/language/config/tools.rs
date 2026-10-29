//! How the registry uses the toolchain:
//!  the request the lookup is asked with,
//!  and the environment and reads a server gets from its answer.

/// The setup says which policy resolves the toolchain and where private state lives.
use super::setup::LanguageSetup;
/// Helix's spelling of the project root,
///  which servers are given.
use crate::language::{
    root::RootView,
    toolchain::{ToolchainRequest, ToolchainState, search::search_path},
};
/// What:
///  `HashMap` is a key-value table;
///  `Path` and `PathBuf` are borrowed and owned filesystem paths.
/// Why:
///  A server definition's environment is a table of variables.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Environment = Record<string, string>;
/// ```
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// What:
///  Helix's spelling of the root,
///  when it differs from the canonical one.
///  `ok()` drops the refusal,
///  which the start reports on its own.
/// Why:
///  Helix gives servers this spelling,
///  so a confined server must find the project there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spellings = [discoverRoot(root)?.helix].filter(spelling => spelling && spelling !== root);
/// ```
pub(super) fn spellings(root: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    // Helix's spelling is listed only when it differs from the canonical root.
    if let Ok(view) = RootView::discover(root) {
        let spelling = view.helix().to_path_buf();
        if spelling.as_path() != root {
            found.push(spelling);
        }
    }
    return found;
}

/// What:
///  Ask the setup's toolchain policy about the project.
/// Why:
///  The answer is kept for the registry:
///  its directories are searched for servers and placed on their `PATH`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolve(root: string, setup: LanguageSetup): ToolchainState
/// ```
pub(super) fn resolve(root: &Path, setup: &LanguageSetup) -> ToolchainState {
    let request = ToolchainRequest {
        project_root: root.to_path_buf(),
        project_spellings: spellings(root),
        state_root: setup.state_root.clone(),
    };
    let state = (setup.toolchain)(&request);
    tracing::debug!(root = %root.display(), state = ?state, "asked the toolchain about the project");
    return state;
}

/// What:
///  The variables a server gets:
///  its definition's own,
///  with `PATH` set to the toolchain's directories followed by the definition's or the application's `PATH`
///  when the toolchain provides directories.
/// Why:
///  A Node server's launcher starts with `#!/usr/bin/env node`,
///  which needs the project's `node` inside the sandbox.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function serverEnvironment(own: Record<string, string>, toolchain: ToolchainState): Record<string, string>
/// ```
pub(super) fn server_environment(
    own: &HashMap<String, String>,
    toolchain: &ToolchainState,
) -> HashMap<String, String> {
    let mut environment = own.clone();
    if toolchain.directories().is_empty() {
        return environment;
    }
    let mut inherited = own.get("PATH").map(std::ffi::OsString::from);
    if inherited.is_none() {
        inherited = std::env::var_os("PATH");
    }
    if let Some(path) = search_path(toolchain.directories(), inherited) {
        environment.insert("PATH".to_string(), path);
    }
    return environment;
}

/// The directories a server may read but not write:
///  the toolchain's directories,
///  shown again read-only when the sandbox replaced their location.
pub(super) fn read_only_binds(toolchain: &ToolchainState) -> Vec<PathBuf> {
    return toolchain.directories().to_vec();
}

/// Where servers are found,
///  the environment they get,
///  and when the toolchain is asked again.
#[cfg(test)]
#[path = "tools_tests.rs"]
mod tests;
