//! What:
//!  In-process repository reads:
//!  objects,
//!  packs,
//!  refs,
//!  layout and configuration without starting Git.
//! Why:
//!  Reading without a Git process is one capability.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // repo_read/index.ts: one `export * from './file'` per module below.
//! ```

/// Captured read-only real-Git queries over raw bytes.
pub mod git_metadata;

/// The Git starts of the layout questions for a user whose own file holds `core.fsmonitor = true`;
///  never part of the release executable.
#[cfg(test)]
#[cfg(unix)]
mod reader_fsmonitor_tests;

/// The one query for repository identity,
///  top level and the path below it.
pub mod repository_location;

/// The one query for the worktree a repository's shared settings belong to.
pub mod repo_read_main_worktree;

/// Lazily fetched repository facts behind one interface,
///  with the real-Git provider.
pub mod repository_facts;

/// What one `git add` selects,
///  counted from real Git's own listings.
pub mod repository_facts_staged_selection;

/// The worktree's ignored files whose paths a revision's tree tracks,
///  read beside the provider so its interface file stays within its line bound.
pub mod repository_facts_ignored_files;

/// What one abort form would discard besides the state of the operation it undoes.
pub mod repository_facts_abort_discard;

/// The work a guarded form would discard in a repository no worktree rule refuses:
///  the uncommitted changes and the files Git does not track.
pub mod repository_facts_discardable_paths;

/// Files-backend ref reads that answer without a Git process or decline.
pub mod ref_files;

/// Proof,
/// without a Git process,
/// that a sequencer marker names nothing on the files ref storage.
pub mod sequencer_marker;

/// The Git directories a command works in,
/// found without starting Git.
pub mod git_directory_roots;

/// The system configuration file a real Git executable reads,
/// learned once per executable identity.
pub mod git_system_config;

/// In-process repository reads:
/// the plain-text configuration reader that gives up on any spelling it cannot prove.
pub mod repo_read_config_text;

/// In-process repository reads:
/// proof that no configuration scope sets `core.bare` or `core.worktree`.
pub mod repo_read_config_scan;

/// In-process repository reads:
/// proof that every configuration value Git types at startup is one Git accepts silently.
pub mod repo_read_config_typed;

/// The table of which configuration keys Git reads at startup and what kind of value each takes.
pub mod repo_read_config_value_kinds;

/// In-process repository reads:
/// the location,
/// Git paths and object format found from the file system.
pub mod repo_read_layout;

/// The rules about the environment of a Git command:
/// which variables can move discovery,
/// and the effective environment.
pub mod repo_read_layout_environment_rules;

/// Small file system and command line facts a layout read needs:
/// owner,
/// device,
/// small file text,
/// the `-C` directory.
pub mod repo_read_layout_disk_and_directory_facts;

/// The layout reader's answer with some global options and variables left out.
pub mod repo_read_layout_without;

/// In-process repository reads:
/// the layout turned into the shapes the wrapper's existing queries answer with.
pub mod repo_read_adapter;

/// In-process repository reads:
/// object kinds,
/// names and the zlib step shared by loose objects and packs.
pub mod repo_read_object;

/// In-process repository reads:
/// one pack,
/// its index lookup and delta resolution.
#[cfg(unix)]
pub mod repo_read_pack;

/// In-process repository reads:
/// the object database of a repository,
/// its packs and alternates.
#[cfg(unix)]
pub mod repo_read_odb;

/// The abbreviated object name Git prints (`core.abbrev`,
/// the automatic length,
/// uniqueness),
/// computed without starting Git.
#[cfg(unix)]
pub mod object_abbreviation;

/// Small byte and name checks the abbreviation uses:
/// a NUL,
/// a hexadecimal digit,
/// a shared prefix length.
pub mod object_abbreviation_byte_and_prefix_checks;

/// In-process repository reads:
/// trees,
/// flattened trees and commit fields.
#[cfg(unix)]
pub mod repo_read_tree;

/// In-process repository reads:
/// the part of a tree that differs from an index,
/// found beside its cache-tree.
#[cfg(unix)]
pub mod repo_read_tree_cached;

/// In-process repository reads:
/// the entries of an index file,
/// versions 2 to 4.
pub mod repo_read_index;

/// In-process repository reads:
/// the difference between two sorted entry lists and the output spellings.
#[cfg(unix)]
pub mod repo_read_diff;

/// In-process repository reads:
/// the files ref storage,
/// one ref by name,
/// a symbolic chain and the list of all refs.
#[cfg(unix)]
pub mod repo_read_refs;

/// In-process repository reads:
/// the merged configuration listing with includes and environment entries.
pub mod repo_read_config_list;

/// In-process repository reads:
/// answers to the exact read-only command lines the wrapper issues.
#[cfg(unix)]
pub mod repo_read_query;

/// What the in-process Git queries share:
/// the request,
/// the answer,
/// the parsed command line and the query context.
pub mod repo_read_query_request_answer_and_context;

/// In-process repository reads:
/// answers for the reference and configuration command lines.
#[cfg(unix)]
pub mod repo_read_query_refs;

/// In-process repository reads:
/// the exit status of `diff-index --cached --quiet`.
#[cfg(unix)]
pub mod repo_read_query_quiet;

/// In-process repository reads:
/// the two runner entry points that try the answers before starting Git.
pub mod repo_read_hook;
