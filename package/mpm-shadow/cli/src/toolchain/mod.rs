//! What:
//! The toolchain half of `mpm-shadow`:
//! the plan and lock files,
//! and the commands that read,
//! check,
//! install
//!           and update what they describe (`install --lockfile-only`,
//! `install --lockfile-only --frozen-lockfile`,
//! `install`,
//! `list`,
//! `update`,
//! `bin`).
//! Why:
//! mpm-shadow owns the toolchain plan (`toolchain.plan.jsonc`,
//! written by people) and the lock
//!           (`toolchain.lock.json`,
//! written only here),
//! so the modules live together under one directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // toolchain/index.ts
//! export * from './model.ts';
//! export * from './plan.ts';
//! export * from './lock.ts';
//! ```

/// The newest finished copy of a repository-published tool across both toolchain roots.
pub mod across_roots;

/// The plan and lock agreement validator and the offline projection.
pub mod agree;

/// `bin`:
/// the program directories as JSON.
pub mod bin_paths;

/// The command-line grammar and dispatch of the toolchain verbs.
pub mod commands;

/// The declared grammar of every toolchain verb:
/// its name,
/// visible aliases and argument builder,
/// re-exported by `commands`.
mod verb_grammar;

/// The bundle of seams every provider,
/// observer and resolver receives.
pub mod context;

/// The error type and its exit codes.
pub mod error;

/// The exposure directory,
/// postinstall links and postinstall commands.
pub mod exposure;

/// SHA-256 helpers.
pub mod hash;

/// The network seam and its real client.
pub mod http;

/// The switch from a tool's route to its installer.
pub mod install;

/// The `archive` route.
pub mod install_archive;

/// The `bin-sourced` route.
pub mod install_bin;

/// The cargo routes.
pub mod install_cargo;

/// Where `cargo-binstall` tools live,
/// and which version of a repository-published one runs.
pub mod cargo_installs;

/// The routes through pnpm,
/// uv and go.
pub mod install_managers;

/// The Rust toolchain and Android SDK routes.
pub mod install_rust_android;

/// Downloads,
/// extraction and links shared by the installers.
pub mod install_support;

/// The staging directory and completion marker of an archive install.
pub mod archive_staging;

/// The per-tree launcher for repository-published tools.
pub mod launcher;

/// The check that a launcher's records never lead back to the launcher or in a circle,
/// so a launcher never starts itself.
pub mod launcher_chain;

/// Whether a path is unreachable because the repository's link to the 860 drive dangles,
/// by asking the installed check once.
pub mod repository_link_cause;

/// What a launcher runs when its record's program is gone:
/// the program under `current`,
/// or the sentence that names both versions and the command that fixes it.
pub mod launcher_current_link;

/// Which `mpm-shadow` a launcher is a copy of:
/// the newest installed one,
/// or the running program when it belongs to no install.
pub mod launcher_source;

/// Where tools install and how their binaries are found.
pub mod layout;

/// Reading a scope's plan and lock together.
pub mod load;

/// The lock file:
/// build,
/// render,
/// parse,
/// digest.
pub mod lock;

/// `install --lockfile-only` and `install --lockfile-only --frozen-lockfile`.
pub mod lock_cmd;

/// The shared data model and its JSON decoding.
pub mod model;

/// Decoding the plan and lock JSON into the model.
pub mod model_decode;

/// The checks the decoders share.
pub mod model_decode_support;

/// Encoding the model back to JSON in the lock's key order.
pub mod model_encode;

/// What is installed now,
/// per route,
/// and the status report.
pub mod observe;

/// Cargo's own install record (`.crates.toml`) as crate,
/// version and binaries.
pub mod cargo_record;

/// Finding the old copies of this repository's own tools in the shared cargo bin directory,
/// and deciding which links move and which copies go.
pub mod old_copies;

/// Carrying out the plan for the old copies:
/// re-point the links,
/// then remove the copy and its cargo record.
pub mod old_copy_apply;

/// The places that can hold a link to an old copy,
/// and the scan for links to one file.
pub mod old_copy_places;

/// The removal of an old copy in the cargo bin directory and of an older managed version.
pub mod old_copy_remove;

/// The older managed copies of this repository's tools in the root that does not hold the newest copy.
pub mod managed_losers;

/// The worktrees of the repository and the lock file of each.
pub mod worktrees;

/// The cleanup of version directories no lock names.
pub mod prune_cargo;

/// The newest-only install layout of the archive-route tools this repository publishes.
pub mod published_archive;

/// The names the plan and the lock may use as directory names.
pub mod segment;

/// Readers for the text other tools print,
/// and the version comparison.
pub mod parse_output;

/// The plan file reader.
pub mod plan;

/// The request text of a plan entry.
pub mod request;

/// Revisions,
/// the scope's OS lock and the atomic publish of the lock.
pub mod publish;

/// The seam for starting programs.
pub mod runner;

/// The repository and global scopes and their paths.
pub mod scope;

/// The `list` command.
pub mod status_cmd;

/// The tool environment (`ANDROID_HOME` and the like) as a generated mise fragment.
pub mod tool_env;

/// The `install` command.
pub mod sync;

/// Running the cleanup of old copies after `install` and printing its lines.
pub mod sync_old_copies;
/// The wording of what `install` prints for a sync.
pub mod sync_report_text;

/// CI detection by the rules of `ci-info`,
/// which pnpm uses.
pub mod ci_detect;

/// The `update` command.
pub mod update;

/// Which plan tools a named `update` resolves.
pub mod update_select;

/// The Android command-line tools and conda package resolvers.
pub mod update_android_conda;

/// The GitHub release resolver.
pub mod update_github;

/// Choosing the release asset of one platform.
pub mod update_github_assets;

/// The Go runtime,
/// Go module and Zig resolvers.
pub mod update_go_zig;

/// The Temurin JDK and AWS CLI resolvers.
pub mod update_java_aws;

/// The npm,
/// PyPI and crates.io resolvers.
pub mod update_registry;

/// The resolver types,
/// fetch helpers and the route switch.
pub mod update_resolve;

/// The Rust channel and Node.js resolvers.
pub mod update_rust_node;

/// Small named helpers shared by the toolchain modules.
pub mod util;

/// The `verify-archive` command.
pub mod verify_archive;

/// Test fixtures shared by the toolchain unit tests.
#[cfg(test)]
mod fixtures;

/// The unit tests of the network seam.
#[cfg(test)]
mod http_tests;

/// The unit tests of the program-start seam.
#[cfg(test)]
mod runner_tests;

/// The unit tests of `list`.
#[cfg(test)]
mod status_tests;

/// Test doubles for the runner and the network.
#[cfg(test)]
mod fakes;

/// The unit tests of the plan reader.
#[cfg(test)]
mod plan_tests;

/// The unit tests of the lock file and the agreement validator.
#[cfg(test)]
mod lock_tests;

/// The unit tests of scopes,
/// layout and the binary search.
#[cfg(test)]
mod layout_tests;

/// The unit tests of `bin`.
#[cfg(test)]
mod bin_paths_tests;

/// The unit tests of `install --lockfile-only`,
/// `install --lockfile-only --frozen-lockfile` and publishing.
#[cfg(test)]
mod lock_cmd_tests;

/// The unit tests of the output readers.
#[cfg(test)]
mod parse_output_tests;

/// The unit tests of the observers and the status report.
#[cfg(test)]
mod observe_tests;

/// The unit tests of every installer.
#[cfg(test)]
mod install_tests;

/// The unit tests of the Rust component and target reconciler.
#[cfg(test)]
mod install_rust_android_tests;

/// The unit tests of `install`,
/// the exposure directory and the postinstall steps.
#[cfg(test)]
mod sync_tests;

/// The unit tests of the farm link to the running `mpm-shadow`.
#[cfg(test)]
mod exposure_tests;

/// The unit tests of every resolver.
#[cfg(test)]
mod update_resolve_tests;
/// The unit tests of the `github:` resolver's `tagPrefix` option.
#[cfg(test)]
mod update_github_prefix_tests;

/// The unit tests of `update`.
#[cfg(test)]
mod update_tests;

/// The unit tests of the CI detection and of `install` resolving or refusing a plan that is ahead of the lock.
#[cfg(test)]
mod sync_resolve_tests;

/// The unit tests of `update <tool>...`.
#[cfg(test)]
mod update_named_tests;

/// The unit tests of the request text.
#[cfg(test)]
mod request_tests;

/// The unit tests of `verify-archive`.
#[cfg(test)]
mod verify_archive_tests;

/// The unit tests of the shared helpers.
#[cfg(test)]
mod util_tests;

/// Test doubles for `cargo binstall --root`.
#[cfg(test)]
mod fakes_cargo;

/// The unit tests of the version-keyed cargo installs and the older-lock warnings.
#[cfg(test)]
mod cargo_installs_tests;

/// The record of postinstall links made,
/// and the retirement of those the lock no longer declares.
pub mod postinstall_links;

/// The failure of a repository crate whose `cargo binstall` found no usable release archive.
pub mod binstall_no_archive;

/// The unit tests of the retirement of stale postinstall links.
#[cfg(all(test, unix))]
mod postinstall_links_tests;

/// The unit tests of the `cargo binstall` command:
/// which crates may fall back to compiling.
#[cfg(test)]
mod install_cargo_tests;

/// The unit tests of the newest-only layout for archive-route tools this repository publishes.
#[cfg(test)]
mod published_archive_tests;

/// The unit tests of the launcher's decision.
#[cfg(test)]
mod launcher_tests;

/// The unit tests of which `mpm-shadow` a launcher is a copy of.
#[cfg(test)]
mod launcher_source_tests;

/// The unit tests of the cleanup of unnamed cargo versions.
#[cfg(test)]
mod prune_cargo_tests;

/// The unit tests of the names that become directory names.
#[cfg(test)]
mod segment_tests;

/// The unit tests of the reader of cargo's install record.
#[cfg(test)]
mod cargo_record_tests;

/// The unit tests of the cleanup of old copies of this repository's tools.
#[cfg(all(test, unix))]
mod old_copies_tests;

/// The unit tests of the places searched for links to an old copy.
#[cfg(all(test, unix))]
mod old_copy_places_tests;

/// The unit tests of the newest-across-both-roots rule for this repository's tools.
#[cfg(all(test, unix))]
mod across_roots_tests;

/// The unit tests of how an archive install becomes visible (staging,
/// the completion marker,
/// and what a failed install leaves).
#[cfg(test)]
mod archive_install_tests;

/// The unit tests of the tool environment fragment.
#[cfg(test)]
mod tool_env_tests;
