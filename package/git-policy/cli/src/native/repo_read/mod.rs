//! What:
//!  In-process repository reads:
//!  objects,
//!  packs,
//!  refs,
//!  layout and configuration without starting Git.
//! Why:
//!  Reading without a Git process is one capability.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `repo_read_object_format.rs`:
//!   the object name hash a repository's configuration selects,
//!   SHA-1 or SHA-256,
//!   and the name and length each one gives.
//! - `git_directory_roots.rs`:
//!   the Git directories a command works in,
//!   found without starting Git.
//! - `git_system_config.rs`:
//!   the system configuration file a real Git executable reads,
//!   learned once per executable identity.
//! - `git_metadata.rs`:
//!   captured read-only real-Git queries over raw bytes.
//! - `object_abbreviation_byte_and_prefix_checks.rs`:
//!   small byte and name checks the abbreviation uses:
//!   a NUL,
//!   a hexadecimal digit,
//!   a shared prefix length.
//! - `ref_files.rs`:
//!   files-backend ref reads that answer without a Git process or decline.
//! - `repo_read_config_text.rs`:
//!   in-process repository reads:
//!   the plain-text configuration reader that gives up on any spelling it cannot prove.
//! - `repo_read_config_scan.rs`:
//!   in-process repository reads:
//!   proof that no configuration scope sets `core.bare` or `core.worktree`.
//! - `repo_read_config_list.rs`:
//!   in-process repository reads:
//!   the merged configuration listing with includes and environment entries.
//! - `repo_read_config_value_words.rs`:
//!   the words a configuration value may be spelled with:
//!   the booleans,
//!   the per-driver and per-remote names,
//!   the `color.grep.<slot>` slots and the color and attribute words.
//! - `repo_read_config_key_kinds_table.rs`:
//!   the configuration keys whose value is a plain boolean,
//!   any text,
//!   or a path.
//! - `repo_read_config_value_kinds.rs`:
//!   the kinds of value a configuration key takes,
//!   and the lookup from a key to its kind.
//! - `repo_read_config_typed_value_spellings.rs`:
//!   the spelling of each typed value Git reads silently:
//!   a boolean,
//!   a number with its unit letters,
//!   a color,
//!   and a ref name or a word from a list.
//! - `repo_read_config_typed.rs`:
//!   in-process repository reads:
//!   proof that every configuration value Git types at startup is one Git accepts silently.
//! - `repo_read_diff.rs`:
//!   in-process repository reads:
//!   the difference between two sorted entry lists and the output spellings.
//! - `repo_read_hook.rs`:
//!   in-process repository reads:
//!   the two runner entry points that try the answers before starting Git.
//! - `repo_read_index.rs`:
//!   in-process repository reads:
//!   the entries of an index file,
//!   versions 2 to 4.
//! - `repo_read_layout_disk_and_directory_facts.rs`:
//!   small file system and command line facts a layout read needs:
//!   owner,
//!   device,
//!   small file text,
//!   the `-C` directory.
//! - `repo_read_layout_environment_rules.rs`:
//!   the rules about the environment of a Git command:
//!   which variables can move discovery,
//!   and the effective environment.
//! - `repo_read_layout_repository_format.rs`:
//!   the repository format keys a layout read needs:
//!   the object format,
//!   whether `extensions.worktreeConfig` is on,
//!   and the configuration entries once the last `core.bare` has settled.
//! - `repo_read_layout_git_entry_and_head.rs`:
//!   the `.git` entry a directory names,
//!   and the `HEAD` text a plain layout must hold.
//! - `repo_read_layout.rs`:
//!   in-process repository reads:
//!   the location,
//!   Git paths and object format found from the file system.
//! - `repo_read_main_worktree.rs`:
//!   the one query for the worktree a repository's shared settings belong to.
//! - `repo_read_object.rs`:
//!   in-process repository reads:
//!   object kinds,
//!   names and the zlib step shared by loose objects and packs.
//! - `repo_read_odb.rs`:
//!   in-process repository reads:
//!   the object database of a repository,
//!   its packs and alternates.
//! - `object_abbreviation_pack_index.rs`:
//!   one pack index and one multi-pack-index of an object directory,
//!   read for the count and the sorted names they hold.
//! - `object_abbreviation.rs`:
//!   the abbreviated object name Git prints (`core.abbrev`,
//!   the automatic length,
//!   uniqueness),
//!   computed without starting Git.
//! - `repo_read_pack.rs`:
//!   in-process repository reads:
//!   one pack,
//!   its index lookup and delta resolution.
//! - `repo_read_query.rs`:
//!   in-process repository reads:
//!   answers to the exact read-only command lines the wrapper issues.
//! - `repo_read_query_request_answer_and_context.rs`:
//!   what the in-process Git queries share:
//!   the request,
//!   the answer,
//!   the parsed command line and the query context.
//! - `repo_read_refs.rs`:
//!   in-process repository reads:
//!   the files ref storage,
//!   one ref by name,
//!   a symbolic chain and the list of all refs.
//! - `repo_read_query_quiet.rs`:
//!   in-process repository reads:
//!   the exit status of `diff-index --cached --quiet`.
//! - `repo_read_query_refs.rs`:
//!   in-process repository reads:
//!   answers for the reference and configuration command lines.
//! - `repo_read_tree.rs`:
//!   in-process repository reads:
//!   trees,
//!   flattened trees and commit fields.
//! - `repo_read_tree_cached.rs`:
//!   in-process repository reads:
//!   the part of a tree that differs from an index,
//!   found beside its cache-tree.
//! - `repository_location.rs`:
//!   the one query for repository identity,
//!   top level and the path below it.
//! - `repository_facts_git_provider.rs`:
//!   the provider that asks real Git for a fact:
//!   the values it holds,
//!   the one query every fact goes through,
//!   and the small messages its questions report.
//! - `repository_facts_abort_discard_query_arguments_and_readings.rs`:
//!   the questions one abort form asks Git,
//!   the argument list of each,
//!   and the reading of its answer.
//! - `repository_facts_abort_discard_record.rs`:
//!   the record of the commit an abort restores,
//!   read in bounded time from a regular file only,
//!   read beside the reading that asks for it.
//! - `the_facts_a_repository_reads.rs`:
//!   the interface a repository-fact provider answers,
//!   and the two shapes the question about an abort form answers in.
//! - `repository_facts_abort_discard_worktree_entries.rs`:
//!   whether the worktree still holds an entry at a path a listing named,
//!   read beside the reading that asks for it.
//! - `repository_facts_ignored_files.rs`:
//!   the worktree's ignored files whose paths a revision's tree tracks,
//!   read beside the provider so its interface file stays within its line bound.
//! - `repository_facts_staged_entries.rs`:
//!   the listed paths whose staged content an abort would discard,
//!   read beside the provider so its interface file stays within its line bound.
//! - `repository_facts_staged_selection.rs`:
//!   what one `git add` selects,
//!   counted from real Git's own listings.
//! - `repository_facts_untracked_files.rs`:
//!   the worktree's untracked files whose paths a revision's tree tracks,
//!   read beside the provider so its interface file stays within its line bound.
//! - `repository_facts_abort_discard.rs`:
//!   what one abort form would discard besides the state of the operation it undoes.
//! - `repo_read_adapter.rs`:
//!   in-process repository reads:
//!   the layout turned into the shapes the wrapper's existing queries answer with.
//! - `sequencer_marker.rs`:
//!   proof,
//!   without a Git process,
//!   that a sequencer marker names nothing on the files ref storage.
//! - `repository_facts.rs`:
//!   lazily fetched repository facts behind one interface,
//!   with the real-Git provider.
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

/// The provider that asks real Git for a repository fact,
///  beside the interface it implements.
pub mod repository_facts_git_provider;

/// Lazily fetched repository facts behind one interface,
///  with the real-Git provider.
pub mod repository_facts;

/// What one `git add` selects,
///  counted from real Git's own listings.
pub mod repository_facts_staged_selection;

/// The worktree's ignored files whose paths a revision's tree tracks,
///  read beside the provider so its interface file stays within its line bound.
pub mod repository_facts_ignored_files;

/// The worktree's untracked files whose paths a revision's tree tracks,
///  read beside the provider so its interface file stays within its line bound.
pub mod repository_facts_untracked_files;

/// The listed paths whose staged content an abort would discard,
///  read beside the provider so its interface file stays within its line bound.
pub mod repository_facts_staged_entries;

/// The interface a repository-fact provider answers,
/// and the two shapes the question about an abort form answers in.
pub mod the_facts_a_repository_reads;

/// What one abort form would discard besides the state of the operation it undoes.
pub mod repository_facts_abort_discard;

/// The questions one abort form asks Git,
/// the argument list of each,
/// and the reading of its answer.
pub mod repository_facts_abort_discard_query_arguments_and_readings;

/// The record of the commit an abort restores,
/// read in bounded time from a regular file only,
///  read beside the reading that asks for it.
pub mod repository_facts_abort_discard_record;

/// Whether the worktree still holds an entry at a path a listing named,
///  read beside the reading that asks for it.
pub mod repository_facts_abort_discard_worktree_entries;

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

/// The spelling of each typed value Git reads silently:
/// a boolean,
/// a number with its unit letters,
/// a color,
/// and a ref name or a word from a list.
pub mod repo_read_config_typed_value_spellings;

/// In-process repository reads:
/// proof that every configuration value Git types at startup is one Git accepts silently.
pub mod repo_read_config_typed;

/// The words a configuration value may be spelled with:
/// the booleans,
/// the per-driver and per-remote names,
/// the `color.grep.<slot>` slots and the color and attribute words.
pub mod repo_read_config_value_words;

/// The configuration keys whose value is a plain boolean,
/// any text,
/// or a path.
pub mod repo_read_config_key_kinds_table;

/// The kinds of value a configuration key takes,
/// and the lookup from a key to its kind.
pub mod repo_read_config_value_kinds;

/// The object name hash a repository's configuration selects,
/// SHA-1 or SHA-256,
/// and the name and length each one gives.
pub mod repo_read_object_format;

/// The repository format keys a layout read needs:
/// the object format,
/// whether `extensions.worktreeConfig` is on,
/// and the configuration entries once the last `core.bare` has settled.
pub mod repo_read_layout_repository_format;

/// The `.git` entry a directory names,
/// and the `HEAD` text a plain layout must hold.
pub mod repo_read_layout_git_entry_and_head;

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

/// One pack index and one multi-pack-index of an object directory,
/// read for the count and the sorted names they hold.
#[cfg(unix)]
pub mod object_abbreviation_pack_index;

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
