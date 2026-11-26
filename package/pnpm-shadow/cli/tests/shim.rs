//! Integration tests:
//! the built binary started as `pnpm` and as `pnpm-shadow`,
//! against a fake pnpm on PATH.

/// Includes the launcher arrangement from its own file in this test's folder.
#[path = "shim/launcher.rs"]
mod launcher;
/// Includes the world builder from its own file in this test's folder.
#[path = "shim/world.rs"]
mod world;

/// Imports the launcher arrangement and its deadline helpers.
use crate::launcher::{
    TIMED_OUT_EXIT_CODE,
    put_a_launcher_in_front,
    run_limited,
};
/// Imports the world builder and its constants.
use crate::world::{
    CLEAN,
    DISTINCT_EXIT_CODE,
    FAKE_MARKER,
    KILL_VARIABLE,
    LITERAL,
    SHADOW_BINARY,
    SIGTERM_NUMBER,
    expected_version_line,
    make_world,
    path_of,
    run_shadow,
    write_workspace,
};
/// Imports `.signal()` on an exit status.
use std::os::unix::process::ExitStatusExt;
/// Imports the child-process builder.
use std::process::Command;

#[test]
fn a_violation_warns_on_stderr_and_pnpm_runs_with_its_own_output_and_status() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = run_shadow(
        &world,
        &path,
        &["install", "--frozen-lockfile"],
        DISTINCT_EXIT_CODE,
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stdout.trim_end(),
        format!("{FAKE_MARKER} install --frozen-lockfile")
    );
    assert!(stderr.contains("warning:"));
    // The wrapper names itself only after `--version` and `--help`, never in a warning.
    assert!(!stderr.contains("pnpm-shadow"));
    assert!(!stderr.to_lowercase().contains("shadow"));
    assert!(stderr.contains("left-pad"));
    assert_eq!(output.status.code(), Some(DISTINCT_EXIT_CODE));
}

#[test]
fn a_clean_workspace_is_silent_and_pnpm_runs() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = run_shadow(&world, &path, &["list"], 0);

    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim_end(),
        format!("{FAKE_MARKER} list")
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn version_prints_pnpms_text_then_one_line_naming_the_wrapper_and_the_real_pnpm() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let long = run_shadow(&world, &path, &["--version"], 0);
    let short = run_shadow(&world, &path, &["-v"], 0);

    let expected_long = format!("{FAKE_MARKER} --version\n{}", expected_version_line(&world));
    let expected_short = format!("{FAKE_MARKER} -v\n{}", expected_version_line(&world));
    assert_eq!(String::from_utf8_lossy(&long.stdout), expected_long);
    assert_eq!(String::from_utf8_lossy(&short.stdout), expected_short);
    assert!(long.stderr.is_empty());
}

#[test]
fn help_prints_pnpms_text_then_a_section_naming_the_wrapper() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    for word in ["--help", "-h", "help"] {
        let output = run_shadow(&world, &path, &[word], 0);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.starts_with(&format!("{FAKE_MARKER} {word}\n\n")),
            "word {word}"
        );
        assert!(
            stdout.contains(&expected_version_line(&world)),
            "word {word}"
        );
        assert!(stdout.contains("never blocks pnpm"), "word {word}");
    }
}

#[test]
fn a_commands_own_help_and_every_ordinary_run_print_exactly_pnpms_text() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let command_lines: [&[&str]; 3] = [&["install", "--help"], &["help", "install"], &["install"]];
    for command_line in command_lines {
        let output = run_shadow(&world, &path, command_line, 0);

        let expected = format!("{FAKE_MARKER} {}\n", command_line.join(" "));
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            expected,
            "{command_line:?}"
        );
    }
}

#[test]
fn a_lone_version_or_help_word_does_no_scan_even_in_a_workspace_with_a_violation() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    for word in ["--version", "-v", "--help", "-h", "help"] {
        let output = run_shadow(&world, &path, &[word], 0);

        assert!(output.stderr.is_empty(), "word {word} must not scan");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("shadowing pnpm at"),
            "word {word}"
        );
    }
}

#[test]
fn a_commands_own_help_does_no_scan_because_it_installs_nothing() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    for command_line in [["install", "--help"], ["add", "-h"]] {
        let output = run_shadow(&world, &path, &command_line, 0);

        assert!(output.stderr.is_empty(), "{command_line:?} must not scan");
    }
}

#[test]
fn only_commands_that_resolve_or_change_dependencies_warn() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let warns: [&[&str]; 7] = [
        &["add", "x"],
        &["i"],
        &["update"],
        &["remove", "x"],
        &["--filter", "app", "add", "y"],
        &["-r", "install"],
        &["recursive", "install"],
    ];
    for command_line in warns {
        let output = run_shadow(&world, &path, command_line, 0);

        assert!(
            String::from_utf8_lossy(&output.stderr).contains("left-pad"),
            "{command_line:?} must warn"
        );
    }

    let silent: [&[&str]; 6] = [
        &["run", "build"],
        &["list"],
        &["root"],
        &["exec", "ls"],
        &["test"],
        &["--filter", "install", "list"],
    ];
    for command_line in silent {
        let output = run_shadow(&world, &path, command_line, 0);

        assert!(output.stderr.is_empty(), "{command_line:?} must not scan");
    }
}

#[test]
fn a_signal_that_kills_pnpm_reaches_the_caller_as_that_signal() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = Command::new(world.shadow_directory.join("pnpm"))
        .arg("install")
        .current_dir(&world.workspace)
        .env("PATH", &path)
        .env(KILL_VARIABLE, "1")
        .output()
        .expect("start the shadow");

    // The shadow replaced itself with pnpm, so the caller sees pnpm's own death by signal and no exit code.
    assert_eq!(output.status.signal(), Some(SIGTERM_NUMBER));
    assert_eq!(output.status.code(), None);
}

#[test]
fn through_a_launcher_both_start_paths_reach_the_real_pnpm_without_looping() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    put_a_launcher_in_front(&world);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let exec_path = run_limited(&world, &path, &["install"]);
    let spawn_path = run_limited(&world, &path, &["--version"]);

    assert_ne!(
        exec_path.status.code(),
        Some(TIMED_OUT_EXIT_CODE),
        "the exec path looped"
    );
    assert_ne!(
        spawn_path.status.code(),
        Some(TIMED_OUT_EXIT_CODE),
        "the spawn path looped"
    );
    assert_eq!(
        String::from_utf8_lossy(&exec_path.stdout),
        format!("{FAKE_MARKER} install\n")
    );
    assert_eq!(exec_path.status.code(), Some(0));
    // The notice appears once, after pnpm's own line, whatever the chain between the link and pnpm.
    let version_stdout = String::from_utf8_lossy(&spawn_path.stdout);
    assert_eq!(
        version_stdout,
        format!("{FAKE_MARKER} --version\n{}", expected_version_line(&world))
    );
    assert_eq!(spawn_path.status.code(), Some(0));
}

#[test]
fn through_a_launcher_a_violation_warns_exactly_once() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    put_a_launcher_in_front(&world);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = run_limited(&world, &path, &["install"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr.matches("problem in the package manifests").count(),
        1,
        "stderr: {stderr}"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_pnpm_that_pnpm_itself_runs_still_finds_the_real_pnpm_so_the_guard_does_not_leak() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    put_a_launcher_in_front(&world);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    // The fake pnpm runs `pnpm inner` through PATH, which is the launcher chain again, from inside the real pnpm.
    let output = run_limited(&world, &path, &["nested"]);

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("{FAKE_MARKER} nested\n{FAKE_MARKER} inner\n"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_launcher_with_no_real_pnpm_behind_it_ends_with_127_and_does_not_loop() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    put_a_launcher_in_front(&world);
    let path = path_of(&[&world.shadow_directory]);

    for arguments in [["install"], ["--version"]] {
        let output = run_limited(&world, &path, &arguments);

        assert_eq!(output.status.code(), Some(127), "{arguments:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("no real pnpm found on PATH"),
            "{arguments:?}"
        );
    }
}

#[test]
fn a_failing_version_run_gets_no_notice() {
    let world = make_world();
    write_workspace(&world, CLEAN);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = run_shadow(&world, &path, &["--version"], DISTINCT_EXIT_CODE);

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("{FAKE_MARKER} --version\n")
    );
    assert_eq!(output.status.code(), Some(DISTINCT_EXIT_CODE));
}

#[test]
fn a_directory_outside_any_workspace_is_silent_and_pnpm_runs() {
    let world = make_world();
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let output = run_shadow(&world, &path, &["help"], 0);

    assert!(output.stderr.is_empty());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn the_warning_repeats_on_every_run() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.shadow_directory, &world.fake_directory]);

    let first = run_shadow(&world, &path, &["install"], 0);
    let second = run_shadow(&world, &path, &["install"], 0);

    assert!(String::from_utf8_lossy(&first.stderr).contains("warning:"));
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn with_no_real_pnpm_on_path_it_exits_127_after_the_warning() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.shadow_directory]);

    let output = run_shadow(&world, &path, &["install"], 0);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("left-pad"));
    assert!(stderr.contains("no real pnpm found on PATH"));
    assert_eq!(output.status.code(), Some(127));
}

#[test]
fn started_under_its_own_name_it_answers_version_and_help_and_runs_no_pnpm() {
    let world = make_world();
    write_workspace(&world, LITERAL);
    let path = path_of(&[&world.fake_directory]);

    let version = Command::new(SHADOW_BINARY)
        .arg("--version")
        .current_dir(&world.workspace)
        .env("PATH", &path)
        .output()
        .expect("start the shadow");
    let help = Command::new(SHADOW_BINARY)
        .arg("--help")
        .current_dir(&world.workspace)
        .env("PATH", &path)
        .output()
        .expect("start the shadow");

    assert!(String::from_utf8_lossy(&version.stdout).contains(env!("CARGO_PKG_VERSION")));
    assert!(String::from_utf8_lossy(&help.stdout).contains("Shadows pnpm"));
    assert!(version.stderr.is_empty());
    assert!(!String::from_utf8_lossy(&help.stdout).contains(FAKE_MARKER));
}
