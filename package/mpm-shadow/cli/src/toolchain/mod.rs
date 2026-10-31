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

/// The error type and its exit codes.
pub mod error;

/// SHA-256 helpers.
pub mod hash;

/// The lock file: build, render, parse, digest.
pub mod lock;

/// The shared data model and its JSON decoding.
pub mod model;

/// The plan file reader.
pub mod plan;

/// Test fixtures shared by the toolchain unit tests.
#[cfg(test)]
mod fixtures;

/// The unit tests of the plan reader.
#[cfg(test)]
mod plan_tests;

/// The unit tests of the lock file and the agreement validator.
#[cfg(test)]
mod lock_tests;
