//! What:
//! A candidate on `PATH` that is a launcher is judged by the program its record names,
//! never by its own bytes:
//! skipped when the record leads to a shadow,
//! started when it leads to a real tool.
//! Why:
//! The toolchain puts a launcher (a copy of the `mpm-shadow` program,
//! which carries the identity text) between a link and the tool it starts,
//! so its own bytes would name every launcher a shadow.
//! A record that names a shadow is the only sign that the launcher leads to one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeFile(join(launchers, 'grep.launch'), JSON.stringify({ real: secondShadow }));
//! assert(runBounded(grepOf(first, [first, launchers, real])).stdout === 'REAL grep needle haystack\n');
//! ```

/// Imports the file-system module that writes a launcher record.
use std::fs;

/// Imports the borrowed path type used to name the copied programs.
use std::path::Path;

/// Imports the owned path type one test keeps the shadow link in.
use std::path::PathBuf;

/// Imports the fixtures and the deadline this part of the rule runs on.
use super::{
    SHADOW_EXECUTABLE,
    command_for,
    copy_of_another_version,
    copy_program,
    finished,
    link_as,
    run_bounded,
    scratch,
    write_real_script,
};

#[test]
fn a_launcher_record_that_names_another_shadow_version_is_skipped() {
    let root = scratch();
    let (first, second, launchers, real) = (
        root.path().join("first"),
        root.path().join("second"),
        root.path().join("launchers"),
        root.path().join("real"),
    );
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    copy_of_another_version(&second.join("search-shadow"));
    let link: PathBuf = link_as(&first, "grep");
    // The launcher of the toolchain is a program of its own with a record beside it that names the program it starts.
    // Its own bytes are not the shadow's; only its record leads to the other shadow.
    copy_program(Path::new("/usr/bin/true"), &launchers.join("grep"));
    fs::write(
        launchers.join("grep.launch"),
        format!(r#"{{"toolId":"cargo:monochromatic-search-shadow","lockFile":"/dev/null","real":"{}","scope":"repo"}}"#, second.join("search-shadow").display()),
    )
    .expect("write the record");
    write_real_script(&real, "grep");

    let output = finished(run_bounded(command_for(
        &link,
        &[&first, &launchers, &real],
        &["needle", "haystack"],
    )));

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "REAL grep needle haystack\n",
        "the launcher that leads to a shadow was started"
    );
}

#[test]
fn a_launcher_record_that_names_a_real_tool_is_not_skipped() {
    let root = scratch();
    let (first, launchers) = (root.path().join("first"), root.path().join("launchers"));
    copy_program(Path::new(SHADOW_EXECUTABLE), &first.join("search-shadow"));
    let link: PathBuf = link_as(&first, "grep");
    copy_program(Path::new("/usr/bin/grep"), &launchers.join("grep"));
    fs::write(
        launchers.join("grep.launch"),
        r#"{"toolId":"grep","lockFile":"/dev/null","real":"/usr/bin/grep","scope":"repo"}"#,
    )
    .expect("write the record");
    let file = root.path().join("haystack.txt");
    fs::write(&file, "one needle here\nand none here\n").expect("write the haystack");

    let output = finished(run_bounded(command_for(
        &link,
        &[&first, &launchers],
        &["--count", "needle", &file.display().to_string()],
    )));

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1\n");
}
