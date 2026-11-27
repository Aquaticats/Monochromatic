//! What:
//! A genuine compiled program without the identity text runs behind the shadow,
//! for the `grep` family and for `rg` alike.
//! Why:
//! The identity check must not turn away the real tools it exists to protect:
//! the system's own `grep` and a compiled program standing in for `rg` both run,
//! with the arguments and the output the caller asked for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const grepRun = runBounded(grepOf(first, [first, real], ['--count', 'needle', file]));
//! const rgRun = runBounded(rgOf(first, [first, real], ['needle', file]));
//! assert(grepRun.stdout === '1\n' && rgRun.status === 0);
//! ```

/// Imports the file-system module that writes the file the two runs search.
use std::fs;

/// Imports the borrowed path type used to name the copied programs.
use std::path::Path;

/// Imports the owned path type one test keeps the shadow links in.
use std::path::PathBuf;

/// Imports the fixtures and the deadline this part of the rule runs on.
use super::{
    SHADOW_EXECUTABLE,
    command_for,
    copy_program,
    finished,
    link_as,
    run_bounded,
    scratch,
};

#[test]
fn a_compiled_real_grep_and_a_compiled_real_rg_still_run_behind_the_shadow() {
    let root = scratch();
    let (first, real) = (root.path().join("first"), root.path().join("real"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    let grep_link: PathBuf = link_as(&first, "grep");
    let rg_link: PathBuf = link_as(&first, "rg");
    // Genuine compiled programs without the shadow's identity: the system's grep, and `true` standing in for ripgrep.
    copy_program(Path::new("/usr/bin/grep"), &real.join("grep"));
    copy_program(Path::new("/usr/bin/true"), &real.join("rg"));
    let file = root.path().join("haystack.txt");
    fs::write(&file, "one needle here\nand none here\n").expect("write the haystack");

    let grep_output = finished(run_bounded(command_for(
        &grep_link,
        &[&first, &real],
        &["--count", "needle", &file.display().to_string()],
    )));
    let rg_output = finished(run_bounded(command_for(
        &rg_link,
        &[&first, &real],
        &["needle", &file.display().to_string()],
    )));

    assert!(
        grep_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&grep_output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&grep_output.stdout), "1\n");
    assert!(
        rg_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&rg_output.stderr)
    );
}
