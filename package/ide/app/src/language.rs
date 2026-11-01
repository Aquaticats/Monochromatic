//! Headless language intelligence:
//!  definitions,
//!  references,
//!  hover,
//!  inlay hints,
//!  and diagnostics
//! for the displayed file,
//!  through Helix's language-server client,
//!  with no project-file writes.
//!
//! The native layer owns one `LanguageWorker`.
//!  It sends non-blocking commands (`open`,
//!  `reload`,
//! `close`,
//!  `request`,
//!  `request_hints`) and polls latest-value state and one-shot replies
//! (`try_take_status`,
//!  `try_take_diagnostics`,
//!  `try_take_hints`,
//!  `try_take_reply`).
//!  Every result
//! carries the file generation,
//!  content revision,
//!  and server process it answers;
//!  the handle
//! drops results that no longer describe the displayed text.

/// Errors name the operation that failed.
use anyhow::Result;
/// What:
///  `Path` is a borrowed filesystem path.
/// Why:
///  The project root is supplied by the caller and only read here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Path = string;
/// ```
use std::path::Path;

/// Displayed-text and server-process identities carried by every result.
pub mod identity;

/// Character offsets and server positions are converted in exactly one place.
pub mod position;

/// Server-returned locations are validated before anything is read or opened.
pub mod target;

/// Position requests and their one-shot replies.
pub mod reply;

/// Latest-value server and document states.
pub mod status;

/// Results for another file,
///  revision,
///  or server process are dropped when they are read.
pub mod fence;

/// Pushed and pulled diagnostics with version checks and the unversioned freshness hold.
pub mod diagnostics;

/// Inlay request ranges and label shaping.
pub mod hints;

/// Replies to requests that servers send to the client;
///  every edit is refused.
pub mod incoming;

/// The launch seam:
///  what is spawned for a server,
///  and where its private directories may be.
pub mod launch;

/// The production launch policy:
///  bubblewrap confinement,
///  refusing rather than running unconfined.
pub mod confine;

/// Servers that the project's toolchain provides,
///  looked up in the sandbox the servers get.
pub mod toolchain;

/// The language registry,
///  built in code from Helix's built-in definitions and the application's overrides.
pub mod config;

/// What the interface thread tells the worker about the displayed document.
pub mod sync;

/// Which directory Helix would root a server at,
///  checked before anything is spawned.
mod root;

/// Answers a server still owes for the displayed text,
///  and the bounded rule for asking again.
mod owed;

/// The reason a displayed file has no hover,
///  hints,
///  or definitions,
///  published as one `Information` entry of its diagnostics.
pub mod notice;

/// What the worker thread remembers.
mod session;

/// Server processes:
///  start,
///  deadline,
///  readiness,
///  exit.
mod attach;

/// Document synchronization notifications.
mod lifecycle;

/// The five feature requests.
mod request;

/// Send gathered file changes to the servers that asked for them.
mod forward;
/// Server-to-client traffic.
mod traffic;
/// The file watchers servers registered,
///  and the changes gathered for them.
mod watched_files;

/// The wait until every server process the worker started has been reaped.
mod reap;

/// The worker thread and its loop.
mod worker;

/// The interface thread's handle.
mod handle;

/// The handle the native layer owns.
pub use handle::LanguageWorker;

/// Log directive that shows helix-lsp's warnings and errors without its message bodies.
/// helix-lsp logs every protocol message in full at `info`;
///  keep its target at `warn`.
pub const HELIX_LOG_DIRECTIVE: &str = "helix_lsp=warn";

/// What:
///  Make the project root the process working directory.
///  `Result<()>` is success without
///       a value,
///  or an error.
/// Why:
///  Helix derives every language server's root from the process working directory,
///  reads
///      it once,
///  and keeps it.
///  Call this exactly once,
///  at application startup,
///  before any
///      thread is started and before the first Helix call (which includes creating the syntax
///      engine and this module's worker).
///  It changes process-wide state,
///  so library tests must
///      never call it;
///  tests that start servers run in a child process whose working directory
///      was set when it was spawned.
///  The worker itself never changes the working directory:
///  it
///      checks what Helix will compute and refuses to start a server whose root would be
///      outside the project.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function enterProjectDirectory(root: string): void { process.chdir(root); }
/// ```
pub fn enter_project_directory(root: &Path) -> Result<()> {
    // What: `anyhow::Error::new(error).context(...)` attaches a message built only on failure.
    // Why: A failure here means language support cannot work, and the message must name the directory.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { process.chdir(root); } catch (error) { throw new Error(`Cannot enter ${root}`, { cause: error }); }
    // ```
    if let Err(error) = std::env::set_current_dir(root) {
        return Err(anyhow::Error::new(error).context(format!(
            "Cannot make {} the working directory",
            root.display()
        )));
    }
    return Ok(());
}
