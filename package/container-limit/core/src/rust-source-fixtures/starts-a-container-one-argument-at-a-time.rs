//! Starts a container,
//! with one argument per call,
//! which is the other shape a Rust program writes a start in.
//!
//! This file is a fixture of `rust-sources-in-the-scan.unit.test.ts`:
//! the answer the scan must give for it is one start,
//! whose verb is `run`.
//! It carries no limit,
//! because it is no ledger entry,
//! and no test expects a bound of it.

/// Starts the container the scan must report,
/// with the verb in the first of its `arg` calls.
pub fn start(image: &str) -> std::process::Output {
    return Command::new("podman")
        .arg("run")
        .arg("--rm")
        .arg(image)
        .output()
        .expect("podman runs");
}
