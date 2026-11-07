//! What:
//!  The Rust copy of the repository link check (W1,
//! `package/repository-link/check`):
//! is the repository path a usable link to the 860 drive right now,
//! and the gate that asks it before a Git command touches the repository.
//! Why:
//!  After the repository moves to the 860 drive its old path is a link to the copy there.
//! A Git command that names the repository must never create anything in the wrong place when that link dangles
//! (the drive is not mounted),
//! so every command whose paths lie under the link,
//! its approved root or the old copy is checked first and stops with a plain message and status 128.
//! The plan is `doc/planning/repository-on-the-860-drive.md`,
//! section "Work items per tool",
//! W1 and W2.
//! The folder moves with the layout plan of issue 6261
//! (`doc/planning/cli-git-source-layout.md`,
//! "Folders to move"),
//! where it gets its `uses/` folder like every other folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gate = repositoryLinkGate({ home, paths }); // { kind: 'clear' } | { kind: 'stop', stderr }
//! ```

/// The answers of the check and their plain messages.
pub mod repository_link_answer;

/// The identity file the cut-over writes and the check reads.
pub mod repository_link_identity;

/// The filesystem questions the check asks,
/// and the probe that answers them from the running system.
pub mod repository_link_probe;

/// The check itself,
/// with its deadline.
pub mod repository_link_check_walk;

/// The paths an invocation names,
/// and whether one of them lies under the repository link.
pub mod repository_link_paths;

/// The gate that runs the check for the commands that touch the repository.
pub mod repository_link_gate;

/// The line the doctor prints about the link to the 860 drive.
pub mod repository_link_doctor_line;

/// The one table of fixture states both the TypeScript and the Rust check run.
#[cfg(test)]
#[path = "repository_link_shared_table_tests.rs"]
mod shared_table_tests;

/// A probe that answers from an in-memory world and counts its calls;
/// never part of the release executable.
#[cfg(test)]
pub(crate) mod repository_link_fixture_probe;
