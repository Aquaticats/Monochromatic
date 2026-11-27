//! Removes an image,
//! and starts nothing.
//!
//! This file is a fixture of `rust-sources-in-the-scan.unit.test.ts`:
//! it names the program in code,
//! so the ledger reads it,
//! and the answer the scan must give for it is no start at all.

/// Removes the image the scan must find no start in.
pub fn remove(name: &str) -> std::process::Output {
    return Command::new("podman")
        .args([
            "rmi",
            name,
        ])
        .output()
        .expect("podman runs");
}
