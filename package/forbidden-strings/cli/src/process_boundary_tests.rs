//! What:
//!  Process-level positive controls for redacted panic output.
//! Why:
//!  A passing absence assertion is meaningful only if the default hook demonstrably prints the synthetic payload.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Run the same named panicking operation in separate default-hook and protected child processes.
//! ```

/// Import the actual process boundary and ordinary native subprocess API.
use super::{omit_panic_payload, run};
use std::process::{Command, ExitCode, Output};

/// Deliberate fault exists only in the test binary,
///  never behind a production environment trigger.
fn synthetic_panic() -> anyhow::Result<i32> {
    panic!("SYNTHETIC_PRIVATE_PAYLOAD_7f1c");
}

/// Ordinary setup errors keep their established CLI diagnostic.
fn ordinary_failure() -> anyhow::Result<i32> {
    return Err(anyhow::anyhow!("ordinary fixture failure"));
}

/// A successful callback retains its policy exit code.
fn ordinary_success() -> anyhow::Result<i32> {
    return Ok(1);
}

/// Child entry selected by its exact test name;
///  no hook is changed in the ordinary parent test process.
#[test]
fn isolated_boundary_probe() {
    let Ok(mode) = std::env::var("FORBIDDEN_STRINGS_BOUNDARY_TEST") else { return; };
    if mode == "protected" {
        std::panic::set_hook(Box::new(omit_panic_payload));
        assert_eq!(run(synthetic_panic), ExitCode::from(2));
    } else if mode == "default" {
        assert_eq!(run(synthetic_panic), ExitCode::from(2));
    } else if mode == "ordinary" {
        std::panic::set_hook(Box::new(omit_panic_payload));
        assert_eq!(run(ordinary_failure), ExitCode::from(2));
        assert_eq!(run(ordinary_success), ExitCode::from(1));
    } else {
        panic!("unknown fixture mode");
    }
}

/// Invoke the exact child test with its own process-wide hook state.
fn probe(mode: &str) -> Output {
    return Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", "process_boundary::tests::isolated_boundary_probe", "--nocapture"])
        .env("FORBIDDEN_STRINGS_BOUNDARY_TEST", mode)
        .output()
        .expect("isolated boundary probe");
}

/// Default output is a positive control;
///  protected output retains the error without private payloads.
#[test]
fn process_owner_redacts_payloads_without_turning_panics_into_success() {
    let default: Output = probe("default");
    assert!(default.status.success());
    assert!(String::from_utf8_lossy(&default.stdout).contains("1 passed"));
    assert!(String::from_utf8_lossy(&default.stderr).contains("SYNTHETIC_PRIVATE_PAYLOAD_7f1c"));
    let protected: Output = probe("protected");
    assert!(protected.status.success());
    assert!(String::from_utf8_lossy(&protected.stdout).contains("1 passed"));
    let stderr: String = String::from_utf8_lossy(&protected.stderr).into_owned();
    assert!(!stderr.contains("SYNTHETIC_PRIVATE_PAYLOAD_7f1c"));
    assert!(stderr.contains("scanner operation panicked; no complete scan result is available."));
    let ordinary: Output = probe("ordinary");
    assert!(ordinary.status.success());
    assert!(String::from_utf8_lossy(&ordinary.stderr).contains("forbidden-strings: ordinary fixture failure"));
}
