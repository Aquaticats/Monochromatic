//! What:
//! The toolchain half of `mpm-shadow`:
//! the plan and lock files,
//! and the commands that read,
//! check,
//! install
//!           and update what they describe (`lock`,
//! `lock --check`,
//! `sync`,
//! `status`,
//! `update`,
//! `bin-paths`).
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

/// The plan and lock agreement validator and the offline projection.
pub mod agree;

/// `bin-paths`:
/// the program directories as JSON.
pub mod bin_paths;

/// The command-line grammar and dispatch of the toolchain verbs.
pub mod commands;

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

/// The routes through pnpm,
/// uv and go.
pub mod install_managers;

/// The Rust toolchain and Android SDK routes.
pub mod install_rust_android;

/// Downloads,
/// extraction and links shared by the installers.
pub mod install_support;

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

/// `lock` and `lock --check`.
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

/// The `status` command.
pub mod status_cmd;

/// The `sync` command.
pub mod sync;

/// The `update` command.
pub mod update;

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

/// The unit tests of `status`.
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

/// The unit tests of `bin-paths`.
#[cfg(test)]
mod bin_paths_tests;

/// The unit tests of `lock`,
/// `lock --check` and publishing.
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

/// The unit tests of `sync`,
/// the exposure directory and the postinstall steps.
#[cfg(test)]
mod sync_tests;

/// The unit tests of every resolver.
#[cfg(test)]
mod update_resolve_tests;

/// The unit tests of `update`.
#[cfg(test)]
mod update_tests;

/// The unit tests of the request text.
#[cfg(test)]
mod request_tests;

/// The unit tests of `verify-archive`.
#[cfg(test)]
mod verify_archive_tests;

/// The unit tests of the shared helpers.
#[cfg(test)]
mod util_tests;
