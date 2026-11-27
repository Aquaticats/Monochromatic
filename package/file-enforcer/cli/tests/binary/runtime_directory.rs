//! What:
//!  The binary test for where the server keeps its socket.
//! Why:
//! The socket lives under `XDG_RUNTIME_DIR`,
//! and a command started without that variable falls back to a directory only its owner can enter.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe("where the server keeps its socket", () => { it("...", () => { ... }); });
//! ```

use crate::support::{
    CONFIG,
    Harness,
};
use std::os::unix::fs::PermissionsExt;

#[test]
fn an_unset_runtime_directory_falls_back_to_an_owner_only_directory_under_the_cache() {
    let harness = Harness::new();
    harness.write("file-enforcer.config.jsonc", CONFIG);
    harness.write("AGENTS.md", "body\n");
    let mut command = harness.command(&["check"]);
    command
        .env_remove("XDG_RUNTIME_DIR")
        .env("XDG_CACHE_HOME", harness.home.path());
    let output = command.output().expect("runs");
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let fallback = harness.home.path().join("file-enforcer").join("run");
    let mode = std::fs::metadata(&fallback)
        .expect("the fallback exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o700);
    assert_eq!(
        harness.server_pids().len(),
        1,
        "the server listens in the fallback directory"
    );
    assert!(
        harness.servers_gone_within(10),
        "and exits when idle, so a container step ends deterministically"
    );
}
