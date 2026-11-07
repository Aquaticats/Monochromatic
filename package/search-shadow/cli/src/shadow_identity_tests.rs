//! The unit tests of `shadow_identity.rs`:
//! reading the identity out of a program file,
//! following launcher records,
//! vetting a chosen program,
//! and the arrival marker.
//! The test program itself links this crate,
//! so it carries the identity and is the shadow-shaped fixture;
//! `/usr/bin/true` and `/usr/bin/echo` are the compiled programs that are not shadows.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::shadow_identity::{
    ARRIVAL_EXIT_CODE, ARRIVAL_VARIABLE, Arrival, IDENTITY_PREFIX, IDENTITY_SECTION, MAX_LAUNCHER_HOPS, StartMode, arrival_line, arrival_value,
    birth_in, identity_of_open_file, is_meant_for, is_same_file, own_identity, parse_arrival, process_birth, shadow_behind, vet,
};
use crate::test_scratch::scratch;

/// The running test program, which carries the identity because it links this crate.
fn test_program() -> PathBuf {
    return std::env::current_exe().expect("the test program's own path");
}

/// A copy of a program in a folder, executable.
fn copy_program(source: &Path, target: &Path) -> PathBuf {
    fs::create_dir_all(target.parent().expect("a parent folder")).expect("make the folder");
    fs::copy(source, target).expect("copy the program");
    fs::set_permissions(target, fs::Permissions::from_mode(0o755)).expect("make it executable");
    return target.to_path_buf();
}

/// Writes a launcher record beside a launcher: the shape `mpm-shadow` writes.
fn write_record(launcher: &Path, real: &Path) {
    let mut name: std::ffi::OsString = launcher.as_os_str().to_os_string();
    name.push(".launch");
    fs::write(name, format!(r#"{{"toolId":"t","lockFile":"/dev/null","real":{},"scope":"repo"}}"#, serde_json::Value::String(real.display().to_string()))).expect("write the record");
}

#[test]
fn the_running_program_carries_the_identity_it_says_it_has() {
    let file = fs::File::open(test_program()).expect("open the test program");

    let found: Option<String> = identity_of_open_file(&file);

    assert_eq!(found.as_deref(), Some(own_identity()));
    assert!(own_identity().starts_with(concat!(env!("CARGO_PKG_NAME"), " ")), "{}", own_identity());
}

#[test]
fn the_identity_text_starts_with_the_prefix_and_names_the_crate_and_its_version() {
    let carried: String = format!("{IDENTITY_PREFIX}{}", own_identity());

    assert_eq!(carried, format!("{IDENTITY_PREFIX}{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")));
}

#[test]
fn a_compiled_program_that_is_not_a_shadow_carries_no_identity() {
    for program in ["/usr/bin/true", "/usr/bin/echo", "/usr/bin/grep"] {
        let file = fs::File::open(program).expect("open the program");

        assert_eq!(identity_of_open_file(&file), None, "{program}");
    }
}

#[test]
fn a_text_file_that_quotes_the_identity_is_not_a_shadow() {
    let folder = scratch();
    let path: PathBuf = folder.path().join("quote");
    fs::write(&path, format!("{IDENTITY_PREFIX}monochromatic-example-shadow 0.0.1\n")).expect("write the file");

    assert_eq!(identity_of_open_file(&fs::File::open(&path).expect("open")), None);
    assert_eq!(shadow_behind(&path), None);
}

#[test]
fn an_elf_file_whose_section_has_another_name_is_not_a_shadow() {
    let folder = scratch();
    let mut bytes: Vec<u8> = fs::read(test_program()).expect("read the test program");
    let needle: &[u8] = IDENTITY_SECTION.as_bytes();
    let mut renamed: Vec<u8> = needle.to_vec();
    let last: usize = renamed.len() - 1;
    renamed[last] = b'X';
    let mut index: usize = 0;
    while index + needle.len() <= bytes.len() {
        if &bytes[index..index + needle.len()] == needle {
            bytes[index..index + needle.len()].copy_from_slice(&renamed);
            index += needle.len();
        } else {
            index += 1;
        }
    }
    let path: PathBuf = folder.path().join("renamed");
    fs::write(&path, bytes).expect("write the copy");

    assert_eq!(identity_of_open_file(&fs::File::open(&path).expect("open")), None);
}

#[test]
fn damaged_or_short_program_files_are_not_shadows_and_never_panic() {
    let folder = scratch();
    let whole: Vec<u8> = fs::read(test_program()).expect("read the test program");
    let header_length: usize = 64;
    let table_offset_field: std::ops::Range<usize> = 0x28..0x30;
    let count_field: std::ops::Range<usize> = 0x3C..0x3E;
    let mut huge_count: Vec<u8> = whole[..header_length].to_vec();
    huge_count[count_field].copy_from_slice(&u16::MAX.to_le_bytes());
    let mut table_past_the_end: Vec<u8> = whole[..header_length].to_vec();
    table_past_the_end[table_offset_field].copy_from_slice(&u64::MAX.to_le_bytes());
    let mut big_endian: Vec<u8> = whole[..header_length].to_vec();
    big_endian[5] = 2;
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("empty", Vec::new()),
        ("three bytes", whole[..3].to_vec()),
        ("only the header", whole[..header_length].to_vec()),
        ("cut in the middle", whole[..whole.len() / 2].to_vec()),
        ("more sections than allowed", huge_count),
        ("section table past the end", table_past_the_end),
        ("big-endian", big_endian),
    ];
    for (label, bytes) in cases {
        let path: PathBuf = folder.path().join(label.replace(' ', "-"));
        fs::write(&path, bytes).expect("write the case");

        assert_eq!(identity_of_open_file(&fs::File::open(&path).expect("open")), None, "{label}");
    }
}

#[test]
fn a_launcher_is_judged_by_the_program_its_record_leads_to() {
    let folder = scratch();
    let shadow: PathBuf = copy_program(&test_program(), &folder.path().join("other-version").join("search-shadow"));
    let real: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("real").join("tool"));
    let to_shadow: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("launchers").join("to-shadow"));
    let to_real: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("launchers").join("to-real"));
    write_record(&to_shadow, &shadow);
    write_record(&to_real, &real);

    assert_eq!(shadow_behind(&to_shadow).as_deref(), Some(own_identity()));
    assert_eq!(shadow_behind(&to_real), None);
}

#[test]
fn a_launcher_that_is_itself_a_copy_of_a_shadow_is_judged_by_its_record_not_by_its_own_bytes() {
    // The toolchain's launcher is a hard link of mpm-shadow, which carries the identity; it must not make every tool look like a shadow.
    let folder = scratch();
    let real: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("real").join("tool"));
    let launcher: PathBuf = copy_program(&test_program(), &folder.path().join("launchers").join("tool"));
    write_record(&launcher, &real);

    assert_eq!(shadow_behind(&launcher), None);
}

#[test]
fn a_launcher_chain_is_followed_and_a_chain_that_never_ends_leads_to_a_shadow() {
    let folder = scratch();
    let shadow: PathBuf = copy_program(&test_program(), &folder.path().join("other-version").join("search-shadow"));
    let first: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("launchers").join("first"));
    let second: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("launchers").join("second"));
    write_record(&first, &second);
    write_record(&second, &shadow);
    let again: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("launchers").join("again"));
    let other: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("launchers").join("other"));
    write_record(&again, &other);
    write_record(&other, &again);
    let itself: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("launchers").join("itself"));
    write_record(&itself, &itself);

    assert_eq!(shadow_behind(&first).as_deref(), Some(own_identity()));
    let loop_verdict: String = shadow_behind(&again).expect("a loop of records is a shadow");
    assert!(loop_verdict.contains(&MAX_LAUNCHER_HOPS.to_string()), "{loop_verdict}");
    assert!(shadow_behind(&itself).is_some());
}

#[test]
fn a_record_that_cannot_be_followed_is_not_a_verdict() {
    let folder = scratch();
    let missing: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("launchers").join("missing"));
    write_record(&missing, &folder.path().join("deleted-folder").join("tool"));
    let broken: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("launchers").join("broken"));
    fs::write(folder.path().join("launchers").join("broken.launch"), "{not json").expect("write the record");

    assert_eq!(shadow_behind(&missing), None);
    assert_eq!(shadow_behind(&broken), None);
    assert_eq!(shadow_behind(&folder.path().join("no-such-file")), None);
}

#[test]
fn vet_refuses_a_shadow_and_a_launcher_that_leads_to_one_and_accepts_a_real_program() {
    let folder = scratch();
    let shadow: PathBuf = copy_program(&test_program(), &folder.path().join("other-version").join("grep"));
    let launcher: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("launchers").join("grep"));
    write_record(&launcher, &shadow);
    let real: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("real").join("grep"));

    let refused_shadow: String = vet(&shadow).err().expect("a shadow is refused");
    let refused_launcher: String = vet(&launcher).err().expect("a launcher of a shadow is refused");

    assert!(refused_shadow.contains(own_identity()) && refused_shadow.contains("a shadow"), "{refused_shadow}");
    assert!(refused_launcher.contains("leads to") || refused_launcher.contains("or leads"), "{refused_launcher}");
    assert!(vet(&real).is_ok());
}

#[test]
fn vet_accepts_a_launcher_that_is_a_copy_of_a_shadow_when_its_record_leads_to_a_real_program() {
    // The toolchain's launcher carries the identity (it is a copy of mpm-shadow) and must not make every tool look like a shadow.
    let folder = scratch();
    let real: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("real").join("tool"));
    let launcher: PathBuf = copy_program(&test_program(), &folder.path().join("launchers").join("tool"));
    write_record(&launcher, &real);

    assert!(vet(&launcher).is_ok());
}

#[test]
fn a_compiled_program_is_started_through_the_file_that_was_checked_even_when_its_path_names_a_shadow_afterwards() {
    let folder = scratch();
    let program: PathBuf = copy_program(Path::new("/usr/bin/echo"), &folder.path().join("real").join("echo"));
    let vetted = vet(&program).expect("a real program is accepted");
    // The path is made to name a shadow after the check: a rename over the checked file.
    let shadow: PathBuf = copy_program(&test_program(), &folder.path().join("real").join("swapped"));
    fs::rename(&shadow, &program).expect("swap the path");

    let output = vetted.command().arg("checked file ran").output().expect("start the program");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "checked file ran\n");
}

#[test]
fn a_script_is_started_by_its_path() {
    let folder = scratch();
    let script: PathBuf = folder.path().join("script");
    fs::write(&script, "#!/bin/sh\nprintf 'script ran\\n'\n").expect("write the script");
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).expect("make it executable");

    let output = vet(&script).expect("a script is accepted").command().output().expect("start the script");

    assert_eq!(String::from_utf8_lossy(&output.stdout), "script ran\n");
}

#[test]
fn a_program_that_cannot_be_read_is_still_started_by_its_path() {
    let folder = scratch();
    // A compiled program runs without read permission (a script does not: its interpreter must read it).
    let program: PathBuf = copy_program(Path::new("/usr/bin/echo"), &folder.path().join("execute-only"));
    fs::set_permissions(&program, fs::Permissions::from_mode(0o111)).expect("make it execute-only");
    // The test may run as a user for whom the owner bits do not hide the file (root); only the readable case is a skip.
    if fs::File::open(&program).is_ok() {
        return;
    }

    let vetted = vet(&program).expect("an execute-only program is accepted");

    assert_eq!(String::from_utf8_lossy(&vetted.command().arg("ran unread").output().expect("start it").stdout), "ran unread\n");
}

#[test]
fn a_missing_program_is_refused_with_its_path() {
    let folder = scratch();
    let missing: PathBuf = folder.path().join("no-such-program");

    let reason: String = vet(&missing).err().expect("a missing program is refused");

    assert!(reason.contains("no-such-program") && reason.contains("cannot be opened"), "{reason}");
}

#[test]
fn the_arrival_marker_survives_a_round_trip_and_names_this_process() {
    let value: String = arrival_value(StartMode::Spawn);

    let parsed: Arrival = parse_arrival(&value).expect("the marker parses");

    assert_eq!(parsed.mode, StartMode::Spawn);
    assert_eq!(parsed.pid, std::process::id());
    assert_eq!(parsed.birth, process_birth(std::process::id()));
    assert_eq!(parsed.starter, own_identity());
    assert_eq!(parse_arrival(&arrival_value(StartMode::Exec)).expect("parses").mode, StartMode::Exec);
}

#[test]
fn text_that_is_not_an_arrival_marker_is_ignored() {
    for text in ["", "exec", "exec 1", "exec 1 2", "fork 1 2 name", "exec x 2 name", "exec -1 2 name", "EXEC 1 2 name"] {
        assert_eq!(parse_arrival(text), None, "{text:?}");
    }
}

#[test]
fn a_marker_is_meant_for_the_process_it_names_through_the_way_it_started() {
    let exec = Arrival { mode: StartMode::Exec, pid: 10, birth: "500".to_string(), starter: "s".to_string() };
    let spawn = Arrival { mode: StartMode::Spawn, pid: 10, birth: "500".to_string(), starter: "s".to_string() };

    // After `exec` the program is the process the marker names.
    assert!(is_meant_for(&exec, (10, "500"), (1, "1")));
    // A child of that process is reached by the other mode only.
    assert!(is_meant_for(&spawn, (11, "600"), (10, "500")));
    assert!(!is_meant_for(&exec, (11, "600"), (10, "500")));
    assert!(!is_meant_for(&spawn, (10, "500"), (1, "1")));
    // A script that the real tool runs is further down: another process in between.
    assert!(!is_meant_for(&spawn, (12, "700"), (11, "600")));
    // The same number with another birth identity is a later process that reused the number.
    assert!(!is_meant_for(&exec, (10, "501"), (1, "1")));
    assert!(!is_meant_for(&spawn, (11, "600"), (10, "501")));
}

#[test]
fn the_birth_identity_is_counted_after_the_last_bracket_of_the_name() {
    // The name may hold spaces and brackets: field 22 is still the nineteenth word after the bracket that ends the name.
    let after_name: String = (0..25).map(|index| { return format!(" {}", 1000 + index); }).collect();
    let stat: String = format!("42 (we ) ird (name) S{after_name}");

    assert_eq!(birth_in(&stat), "1018");
    assert_eq!(birth_in("no brackets at all"), "?");
    assert_eq!(birth_in("1 (short) S 1 2"), "?");
    assert_ne!(process_birth(std::process::id()), "?");
    assert_eq!(process_birth(u32::MAX), "?");
}

#[test]
fn the_arrival_line_is_one_line_naming_both_shadows_the_tool_and_the_exit_status() {
    let arrival = Arrival { mode: StartMode::Exec, pid: 4242, birth: "1".to_string(), starter: "monochromatic-example-shadow 0.2.1".to_string() };

    let line: String = arrival_line("grep", &arrival);

    assert_eq!(line.lines().count(), 1, "{line}");
    assert!(line.contains(own_identity()) && line.contains("monochromatic-example-shadow 0.2.1") && line.contains("4242"), "{line}");
    assert!(line.contains("real `grep`") && line.contains(&ARRIVAL_EXIT_CODE.to_string()), "{line}");
    assert_ne!(ARRIVAL_EXIT_CODE, 0);
    assert!(ARRIVAL_VARIABLE.starts_with("MONOCHROMATIC_SHADOW_"));
}

#[test]
fn two_paths_to_one_file_are_the_same_file_and_two_copies_are_not() {
    let folder = scratch();
    let original: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("one"));
    let link: PathBuf = folder.path().join("two");
    fs::hard_link(&original, &link).expect("hard link");
    let copy: PathBuf = copy_program(Path::new("/usr/bin/true"), &folder.path().join("three"));

    assert!(is_same_file(&original, &link));
    assert!(!is_same_file(&original, &copy));
    assert!(!is_same_file(&original, &folder.path().join("missing")));
}
