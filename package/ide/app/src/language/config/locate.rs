//! Deciding whether a server's program exists:
//!  the reasons a server is unavailable,
//!  and the lookup that finds the real program or explains its absence.

/// The TypeScript server is the project's own copy.
use super::typescript;
/// The lookup in the directories the toolchain provides,
///  and the state that says whether the toolchain serves the project.
use crate::language::{
    launch::resolve_executable,
    toolchain::{
        ToolchainState,
        search::{Missed, find_program},
    },
};
/// What:
///  `Path` and `PathBuf` are borrowed and owned filesystem paths.
/// Why:
///  A program is found as an absolute path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Path = string;
/// ```
use std::path::{Path, PathBuf};

/// What:
///  Why a configured server cannot be started.
///  An `enum` with data is a tagged union.
/// Why:
///  A missing program and a refused launch lead to different states and remedies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Unavailable = { kind: 'missing'; reason: string } | { kind: 'refused'; reason: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::language) enum Unavailable {
    /// The server's own program was not found,
    ///  or the project's copy is unusable.
    Missing(
        /// Which program was looked for and what the remedy is.
        String,
    ),
    /// The launch policy refused;
    ///  nothing is spawned instead.
    Refused(
        /// What the launch policy reported.
        String,
    ),
}

/// What:
///  The reason a program was not found,
///  naming where it was looked for.
///  `Missed` says whether a shim was passed over.
/// Why:
///  A project the toolchain serves is searched in its directories first,
///  and a reader must be able to tell a program that is absent from one that only exists as a shim,
///  and from a tool the lookup lists but that is not installed (its own warning says so).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingReason(server: string, command: string, toolchain: ToolchainState, missed: Missed): string
/// ```
fn missing_reason(
    server: &str,
    command: &str,
    toolchain: &ToolchainState,
    missed: &Missed,
) -> String {
    let places = match toolchain {
        ToolchainState::Provided(_) => {
            "in the tool directories the toolchain provides for this project or on PATH"
        }
        _ => "on PATH",
    };
    let mut reason = format!("the program '{command}' for {server} was not found {places}");
    if let Some(shim) = &missed.shim {
        reason.push_str(&format!(
            ", and the only match is the mise shim {}, which cannot run inside the sandbox",
            shim.display()
        ));
    }
    let warnings = toolchain.warnings();
    if !warnings.is_empty() {
        reason.push_str(&format!(". the toolchain lookup reported:\n{warnings}"));
    }
    return reason;
}

/// What:
///  Decide whether a server's program exists,
///  returning its absolute path or the reason it is unavailable.
///  `Option<&Path>` is the displayed document,
///  or nothing.
/// Why:
///  The check must see the real server,
///  never a wrapper placed in front of it.
///  The TypeScript server is the project's own copy,
///  looked up from the document's folder upward.
///  When a toolchain applies,
/// the lookup also uses its directories and refuses shims.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function locate(server: string, command: string, root: string, document?: string, toolchain: ToolchainState): string // throws when missing
/// ```
pub(in crate::language) fn locate(
    server: &str,
    command: &str,
    root: &Path,
    document: Option<&Path>,
    toolchain: &ToolchainState,
) -> Result<PathBuf, Unavailable> {
    if server == typescript::SERVER {
        // `map_err` wraps the text reason in the `Missing` variant.
        return typescript::locate(root, document).map_err(Unavailable::Missing);
    }
    if toolchain.serves_project() {
        return match find_program(command, root, toolchain.directories()) {
            Ok(found) => Ok(found),
            Err(missed) => Err(Unavailable::Missing(missing_reason(
                server, command, toolchain, &missed,
            ))),
        };
    }
    // Nothing found becomes a `Missing` reason built from an empty search record.
    return match resolve_executable(command, root) {
        Some(found) => Ok(found),
        None => Err(Unavailable::Missing(missing_reason(
            server,
            command,
            toolchain,
            &Missed::default(),
        ))),
    };
}
