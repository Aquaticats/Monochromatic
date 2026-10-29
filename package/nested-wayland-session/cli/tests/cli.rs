//! Verify the real executable's help and usage behavior without any display.

/// What:
///  Command launches the compiled fixture and Output owns its exit/stdio.
/// Why:
///  Parser-only tests cannot prove main treats help as a successful exit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawnSync } from 'node:child_process';
/// ```
use std::process::{Command, Output};

/// Invoke the built consumer artifact with display discovery unavailable.
fn execute(arguments: &[&str]) -> Output {
    // What: Cargo supplies this binary path at compile time for integration tests.
    // Why: Test the artifact being built, not an older installed copy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const command = new Process(builtBinary);
    // ```
    let mut command = Command::new(env!("CARGO_BIN_EXE_monochromatic-nested-wayland-session"));
    command.args(arguments);
    command.env_remove("WAYLAND_DISPLAY");
    command.env_remove("DISPLAY");
    command.env_remove("XDG_RUNTIME_DIR");
    command.env_remove("DBUS_SESSION_BUS_ADDRESS");
    // output captures both streams; expect only handles failure to launch the test artifact.
    return command.output().expect("launch compiled CLI");
}

/// Both help spellings succeed without a display and write only to stdout.
#[test]
fn help_exits_successfully_without_startup() {
    for option in ["--help", "-h"] {
        let output = execute(&[option]);
        assert!(output.status.success());
        // UTF-8 conversion is expected for clap-generated help text.
        let text = String::from_utf8(output.stdout).expect("UTF-8 help");
        assert!(text.contains("Usage:"));
        assert!(text.contains("--socket"));
        assert!(text.contains("--color-scheme"));
        assert!(text.contains("--headless"));
        assert!(text.contains("COMMAND"));
        assert!(output.stderr.is_empty());
    }
}

/// Version reporting also bypasses native startup.
#[test]
fn version_exits_successfully_without_startup() {
    let output = execute(&["--version"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).expect("UTF-8 version");
    assert!(text.contains("monochromatic-nested-wayland-session"));
    assert!(output.stderr.is_empty());
}

/// Usage errors use clap's exit 2,
///  not a later missing-Wayland error.
#[test]
fn usage_errors_are_reported_before_startup() {
    for input in [vec![], vec!["--unknown"], vec!["--socket"],
        vec!["--size", "0x1", "app"]] {
        let output = execute(&input);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).expect("UTF-8 error");
        // clap's missing-value form gives --help guidance without repeating usage.
        assert!(text.contains("--help"), "{input:?}: {text}");
        if input.is_empty() || input[0] == "--unknown" {
            assert!(text.contains("Usage:"), "{input:?}: {text}");
        }
        if !input.is_empty() {
            assert!(text.contains(input[0]), "{input:?}: {text}");
        }
        assert!(!text.contains("WAYLAND_DISPLAY"));
        assert!(!text.contains("XDG_RUNTIME_DIR"));
    }
}

/// With no display and no `--headless`,
///  the nested window fails to start,
///  exits with a runtime failure,
///  and the message names the flag as an option.
#[test]
fn start_without_a_display_and_without_the_flag_names_the_headless_option() {
    let output = execute(&["--size", "400x300", "--", "true"]);
    assert_eq!(output.status.code(), Some(1), "{:?}", output);
    let text = String::from_utf8(output.stderr).expect("UTF-8 error");
    assert!(text.contains("--headless"), "{text}");
    assert!(text.contains("winit backend init failed"), "{text}");
}

/// A headless start without `XDG_RUNTIME_DIR` cannot create the Wayland listening socket,
///  so it exits 1 with a message naming the variable instead of panicking (a panic exits 101).
#[test]
fn start_without_a_runtime_directory_reports_an_error_not_a_panic() {
    // execute removes XDG_RUNTIME_DIR, so the listening socket has no directory to live in.
    let output = execute(&["--headless", "--", "true"]);
    let text = String::from_utf8(output.stderr.clone()).expect("UTF-8 error");
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("XDG_RUNTIME_DIR"), "{text}");
    assert!(!text.contains("panicked"), "{text}");
}

/// `--gpu` without `--headless` exits 1 before any startup and names `--headless`.
#[test]
fn gpu_without_headless_fails_naming_headless() {
    let output = execute(&["--gpu", "software", "--", "true"]);
    assert_eq!(output.status.code(), Some(1), "{:?}", output);
    let text = String::from_utf8(output.stderr).expect("UTF-8 error");
    assert!(text.contains("--gpu is only valid together with --headless"), "{text}");
}

/// A bad `--gpu` value is a usage error that names the choices.
#[test]
fn bad_gpu_value_is_a_usage_error_naming_the_choices() {
    let output = execute(&["--headless", "--gpu", "amd", "--", "true"]);
    assert_eq!(output.status.code(), Some(2), "{:?}", output);
    let text = String::from_utf8(output.stderr).expect("UTF-8 error");
    assert!(text.contains("auto, software"), "{text}");
    assert!(text.contains("--list-gpus"), "{text}");
}

/// `--list-gpus` prints the device lines and exits 0 with no command and no display.
#[test]
fn list_gpus_prints_the_devices_and_the_auto_choice() {
    let output = execute(&["--list-gpus"]);
    assert!(output.status.success(), "{:?}", output);
    let text = String::from_utf8(output.stdout).expect("UTF-8 listing");
    assert!(text.contains("auto uses:"), "{text}");
}
