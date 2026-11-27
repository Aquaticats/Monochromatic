//! What:
//! The `tests/shim.rs` tests that put a launcher between `pnpm` and the shadow,
//! as the toolchain does:
//! `pnpm` links to a separate executable that starts the shadow under the name `pnpm`,
//! with a record beside it naming the shadow.
//! Why:
//! The launcher chain is the arrangement that made the first release loop;
//! keeping its tests together lets a reader see what the guard bounds without
//! reading the plain pass-through tests in between.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('through a launcher both start paths reach the real pnpm', () => { /* timeout-bound run */ });
//! ```

/// Imports the launcher arrangement and its deadline helpers.
use crate::launcher::{
    TIMED_OUT_EXIT_CODE,
    put_a_launcher_in_front,
    run_limited,
};
/// Imports the world builder and the names these tests ask pnpm to use.
use crate::world::{
    CLEAN,
    FAKE_MARKER,
    LITERAL,
    expected_version_line,
    make_world,
    path_of,
    write_workspace,
};

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
