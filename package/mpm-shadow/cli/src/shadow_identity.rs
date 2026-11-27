//! What:
//! The rule "a shadow never starts a shadow,
//! of any version,
//! as its real program",
//! in parts:
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
//! This module is one of four byte-identical copies,
//! one per shadow crate (`cargo-shadow`,
//! `mpm-shadow`,
//! `pnpm-shadow`,
//! `search-shadow`),
//! because the four crates share no dependency and a crates.io footprint that stays small is the owner's decision
//! (`doc/planning/one-release-each-batch-2026-11.md`,
//! "What the owner decided").
//! `tests/shadow_identity_copies.rs` of `search-shadow` fails when the copies differ.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shadow-identity.ts
//! export function identityOfFile(file: FileHandle): string | undefined { /* the ELF section ".monochromatic_shadow_identity" */ }
//! export function shadowBehind(candidate: string): string | undefined { /* follow `.launch` records, then identityOfFile */ }
//! export function arrivalAtThisProcess(value: string | undefined): Arrival | undefined { /* pid + birth identity */ }
//! ```
//!
//! The rule's parts are one file each:
//! the identity text and the carried section (`identity_text.rs`),
//! the section reader (`elf_section.rs`),
//! the launcher records (`launcher_chain.rs`),
//! the checked start (`vetted_program.rs`),
//! the arrival marker (`arrival_marker.rs`),
//! and the same-file comparison (`same_file.rs`).

/// The arrival marker that stops a shadow a shadow started.
mod arrival_marker;
/// The reader for the ELF section that holds the identity text.
mod elf_section;
/// The identity text and what this program says it is.
mod identity_text;
/// The launcher records that lead from a candidate to the program it starts.
mod launcher_chain;
/// The comparison of two paths by device and inode.
mod same_file;
/// The program chosen as the real tool,
/// opened and checked before it starts.
mod vetted_program;

/// Re-exports the arrival marker,
/// what it says and the rule that reads it.
pub use self::arrival_marker::{
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
/// Re-exports the reader for the identity section.
pub use self::elf_section::identity_of_open_file;
/// Re-exports the identity text's names and what this program says it is.
pub use self::identity_text::{
    IDENTITY_PREFIX,
    IDENTITY_SECTION,
    own_identity,
};
/// Re-exports the launcher record suffix,
/// the hop bound and the record follow.
pub use self::launcher_chain::{
    LAUNCH_RECORD_SUFFIX,
    MAX_LAUNCHER_HOPS,
    identity_of_open_file_unless_launcher,
    shadow_behind,
};
/// Re-exports the comparison of two paths by the file they name.
pub use self::same_file::is_same_file;
/// Re-exports the checked program, the check, and why it hands nothing back.
pub use self::vetted_program::{
    VetRefusal,
    Vetted,
    vet,
};
