//! What:
//!  Controls for the panic hook:
//!  its exact notice,
//!  and in a separate process that a
//!       caught panic's message never reaches standard error once it is installed.
//! Why:
//!  A panic message can hold bytes of a scanned file.
//!  Only a process whose hook was
//!      replaced shows whether the message is kept back,
//!  and only a positive control with
//!      the default hook shows that the probe would see a leaked message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const child = spawnSync(testBinary, ['--exact', name], { env: { MODE: 'hook' } }); expect(child.stderr).not.toContain(secret);
//! ```

/// Import the module under test.
use super::{install_payload_free_panic_hook, panic_notice};
use std::process::{Command, Output};

/// The variable that makes the re-run test act as the child,
///  and says which hook it uses.
const CHILD_VARIABLE: &str = "CLI_GIT_NATIVE_PANIC_HOOK_CHILD";

/// A payload built at run time,
///  so no source file holds it whole.
fn secret() -> String {
    return ["PANIC", "PAYLOAD", "SECRET"].join("_");
}

/// The notice names the location when there is one,
///  and nothing else.
#[test]
fn the_notice_names_only_the_location() {
    let location: &std::panic::Location<'static> = std::panic::Location::caller();
    let with_location: String = panic_notice(Some(location));
    assert!(
        with_location.starts_with(
            format!(
                "cli-git: internal error at {}:{}:{}; its message is not shown because it may contain repository content.",
                location.file(),
                location.line(),
                location.column()
            )
            .as_str()
        ),
        "{with_location}"
    );
    assert!(with_location.contains(" Hint: if you were running a Git command, run it again;"), "{with_location}");
    assert!(with_location.ends_with("report this location.\n"), "{with_location}");
    let without_location: String = panic_notice(None);
    assert!(
        without_location.starts_with(
            "cli-git: internal error; its message is not shown because it may contain repository content."
        ),
        "{without_location}"
    );
    assert!(without_location.contains(" Hint: if you were running a Git command, run it again;"), "{without_location}");
    assert!(without_location.ends_with("report the command you ran.\n"), "{without_location}");
}

/// What:
///  In the child,
///  install the hook when asked,
///  then panic with the secret and catch it.
/// Why:
///   The test runner's capture is off in the child,
///  so what the hook writes is what
///       the child's standard error holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.env.MODE === 'hook') installHook(); try { throw secret; } catch {}
/// ```
#[test]
fn a_caught_panic_shows_only_the_notice_in_its_own_process() {
    if let Some(mode) = std::env::var_os(CHILD_VARIABLE) {
        if mode == "payload-free" {
            install_payload_free_panic_hook();
        }
        // `catch_unwind` runs the named function and turns its panic into an `Err`.
        let caught: std::thread::Result<()> = std::panic::catch_unwind(panic_with_secret);
        assert!(caught.is_err(), "the panic was caught");
        return;
    }
    let mut stderr_by_mode: Vec<String> = Vec::new();
    for mode in ["payload-free", "default"] {
        let output: Output = Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "panic_notice::tests::a_caught_panic_shows_only_the_notice_in_its_own_process",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD_VARIABLE, mode)
            .output()
            .expect("start the child");
        let stdout: String = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(output.status.success(), "{mode}: {stdout}");
        assert!(
            stdout.contains("test result: ok. 1 passed; 0 failed"),
            "{mode}: the child ran exactly one test: {stdout}"
        );
        stderr_by_mode.push(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    // With the hook: the notice and its location, never the payload.
    assert!(
        stderr_by_mode[0].contains("cli-git: internal error at ")
            && stderr_by_mode[0].contains("panic_notice_tests.rs"),
        "{}",
        stderr_by_mode[0]
    );
    assert!(
        !stderr_by_mode[0].contains(secret().as_str()),
        "{}",
        stderr_by_mode[0]
    );
    // Positive control: the default hook prints the payload, so the probe can see a leak.
    assert!(
        stderr_by_mode[1].contains(secret().as_str()),
        "{}",
        stderr_by_mode[1]
    );
    assert!(!stderr_by_mode[1].contains("cli-git: internal error"));
}

/// What:
///  Panic with the secret as the message.
/// Why:
///   A named function for `catch_unwind`,
///  because the repository bans anonymous functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function panicWithSecret(): never { throw new Error(secret()); }
/// ```
fn panic_with_secret() {
    panic!("{}", secret());
}
