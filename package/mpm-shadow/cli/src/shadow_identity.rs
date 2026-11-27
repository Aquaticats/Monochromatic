//! What:
//! The rule "a shadow never starts a shadow,
//! of any version,
//! as its real program",
//! in three parts:
//! an identity text that every shadow program carries inside its own file,
//! a check that reads that text from a candidate (through launcher records when the candidate is a launcher),
//! and an arrival marker in the environment that stops a shadow which a shadow started anyway.
//! Why:
//! A shadow sits first on `PATH` under the real tool's name.
//! Judging a candidate by its path or by this program's own path cannot tell another copy or another release of a shadow,
//! so two shadows at different paths chose each other as the real tool and replaced themselves with each other without end
//! (measured 2026-11-07,
//! `search-shadow` 0.2.2 against another copy:
//! about 105 starts per second under `strace`).
//! The owner's rule:
//! "We should never invoke ourselves recursively.
//! That's bad design."
//! What a file is does not depend on its path or its version:
//! every shadow carries the same identity text in a section of its own,
//! so a shadow skips such a candidate without starting it.
//! What no test of a file can recognize (a script that runs a shadow,
//! a launcher with no record)
//! is stopped by the start itself:
//! a shadow marks the environment of the program it starts with its own process number and birth identity,
//! and a shadow that finds the mark left for its own process,
//! or for its direct parent,
//! was started by a shadow and stops at once.
//! Anything started further down (a script the real tool runs,
//! which may call the shadow again) has another process in between and passes.
//!
//! This file declares the parts and re-exports what they answer,
//! so every caller keeps reading `shadow_identity::<name>`:
//! `the_carried_identity_text` (the words this program carries in its own file),
//! `what_a_file_is` (the identity in a file,
//! and two paths that name one file),
//! `launcher_records` (a launcher judged by the program it leads to),
//! `the_program_checked_while_open` (the open-and-check step,
//! and the start through that file),
//! and `the_arrival_marker` (the marker,
//! its reading and the line a stopped shadow prints).
//!
//! This file and its five parts are byte-identical copies in four shadow crates
//! (`cargo-shadow`,
//! `mpm-shadow`,
//! `pnpm-shadow`,
//! `search-shadow`),
//! because the four crates share no dependency and a crates.io footprint that stays small is the owner's decision
//! (`doc/planning/one-release-each-batch-2026-11.md`,
//! "What the owner decided").
//! `tests/shadow_identity_copies.rs` of `search-shadow` fails when any of the six files differs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shadow-identity/index.ts
//! export * from './the-carried-identity-text.ts';
//! export * from './what-a-file-is.ts';
//! export * from './launcher-records.ts';
//! export * from './the-program-checked-while-open.ts';
//! export * from './the-arrival-marker.ts';
//! ```
//!
//! The calls themselves are unchanged:
//! `ownIdentity()`,
//! `identityOfOpenFile(file)`,
//! `shadowBehind(candidate)`,
//! `vet(program).command()`,
//! `arrivalAtThisProcess()`,
//! `arrivalLine(tool, arrival)`.

/// What:
/// `mod the_carried_identity_text;` declares a private submodule whose code is in
/// `src/shadow_identity/the_carried_identity_text.rs`.
/// Why:
/// The parts stay private and only what this file re-exports is public,
/// so every caller's path is the one it had before the split.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as theCarriedIdentityText from './the-carried-identity-text.ts';
/// ```
mod the_carried_identity_text;

/// The identity text in a file,
/// and whether two paths name one file.
mod what_a_file_is;

/// The record beside a launcher,
/// and the program it leads to.
mod launcher_records;

/// The program opened once,
/// checked while open,
/// and started through that file.
mod the_program_checked_while_open;

/// The marker a shadow leaves for the program it starts,
/// and the arrival stop.
mod the_arrival_marker;

/// The environment variable that carries the arrival marker from a shadow to the program it starts.
pub use the_arrival_marker::{
    ARRIVAL_EXIT_CODE,
    ARRIVAL_VARIABLE,
    Arrival,
    StartMode,
    arrival_at_this_process,
    arrival_line,
    arrival_value,
    birth_in,
    is_meant_for,
    mark_start,
    parse_arrival,
    process_birth,
};

/// The identity text this program carries,
/// its section name and its prefix.
pub use the_carried_identity_text::{
    IDENTITY_PREFIX,
    IDENTITY_SECTION,
    own_identity,
};

/// The record suffix and hop bound of a launcher,
/// and the identity of the shadow behind a candidate.
pub use launcher_records::{
    LAUNCH_RECORD_SUFFIX,
    MAX_LAUNCHER_HOPS,
    identity_of_open_file_unless_launcher,
    shadow_behind,
};

/// The program checked while open,
/// and the check itself.
pub use the_program_checked_while_open::{
    Vetted,
    vet,
};

/// The identity in an open file,
/// and whether two paths name one file.
pub use what_a_file_is::{
    identity_of_open_file,
    is_same_file,
};
