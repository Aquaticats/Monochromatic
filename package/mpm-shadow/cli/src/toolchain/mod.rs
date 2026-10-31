//! What:     The toolchain half of `mpm-shadow`: the plan and lock files, and the commands that read, check, install
//!           and update what they describe (`lock`, `lock --check`, `sync`, `status`, `update`, `bin-paths`).
//! Why:      mpm-shadow owns the toolchain plan (`toolchain.plan.jsonc`, written by people) and the lock
//!           (`toolchain.lock.json`, written only here), so the modules live together under one directory.
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

/// `bin-paths`: the program directories as JSON.
pub mod bin_paths;

/// The command-line grammar and dispatch of the toolchain verbs.
pub mod commands;

/// The bundle of seams every provider, observer and resolver receives.
pub mod context;

/// The error type and its exit codes.
pub mod error;

/// SHA-256 helpers.
pub mod hash;

/// The network seam and its real client.
pub mod http;

/// Where tools install and how their binaries are found.
pub mod layout;

/// Reading a scope's plan and lock together.
pub mod load;

/// The lock file: build, render, parse, digest.
pub mod lock;

/// `lock` and `lock --check`.
pub mod lock_cmd;

/// The shared data model and its JSON decoding.
pub mod model;

/// What is installed now, per route, and the status report.
pub mod observe;

/// Readers for the text other tools print, and the version comparison.
pub mod parse_output;

/// The plan file reader.
pub mod plan;

/// Revisions, the scope's OS lock and the atomic publish of the lock.
pub mod publish;

/// The seam for starting programs.
pub mod runner;

/// The repository and global scopes and their paths.
pub mod scope;

/// The `status` command.
pub mod status_cmd;

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

/// The unit tests of scopes, layout and the binary search.
#[cfg(test)]
mod layout_tests;

/// The unit tests of `bin-paths`.
#[cfg(test)]
mod bin_paths_tests;

/// The unit tests of `lock`, `lock --check` and publishing.
#[cfg(test)]
mod lock_cmd_tests;

/// The unit tests of the output readers.
#[cfg(test)]
mod parse_output_tests;

/// The unit tests of the observers and the status report.
#[cfg(test)]
mod observe_tests;
