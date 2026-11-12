//! Controls for the program search list a commit leaves for the service that pushes it:
//! which variables travel,
//! where the record lives,
//! and what a record this cannot read reads as.

use super::*;

/// A directory of this control's own,
/// which the fixture support removes when the control ends.
struct Scratch {
    /// The directory the worktree's Git directory stands for.
    git_dir: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Scratch {
        return Scratch { git_dir: crate::test_support::fixture_with_prefix("native-program-search", name) };
    }
}

/// An environment of the variables a shell sets.
fn environment(pairs: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    return pairs
        .iter()
        .map(|(name, value)| { return (OsString::from(name), OsString::from(value)); })
        .collect();
}

/// What:
///  The list a push carries is the shell's `PATH` and the program-naming variables,
///  and no variable another program of the host could read something else from.
/// Why:
///   The service's push must find the programs the shell that committed would,
/// and a variable this list leaves out keeps the service's own value.
#[test]
fn the_carried_list_is_the_shells_own_programs() {
    let shell: Vec<(OsString, OsString)> = environment(&[
        ("PATH", "/home/a/.local/share/mise/shims:/usr/bin"),
        ("GIT_SSH_COMMAND", "ssh -i /home/a/.ssh/id_work"),
        ("TERM", "xterm-256color"),
        ("AWS_SECRET_ACCESS_KEY", "not-carried"),
    ]);
    assert_eq!(
        carried(shell.as_slice()),
        vec![
            (String::from("PATH"), String::from("/home/a/.local/share/mise/shims:/usr/bin")),
            (String::from("GIT_SSH_COMMAND"), String::from("ssh -i /home/a/.ssh/id_work")),
        ],
        "the search list and the program a variable names travel, nothing else"
    );
}

/// What:
///  A variable whose value is not text is left out rather than written as replacement characters.
/// Why:
///   A record is JSON;
/// a lossy value could name a folder no shell searches,
/// and following it would start a program the
///  person never resolved.
#[test]
fn a_value_that_is_not_text_does_not_travel() {
    use std::os::unix::ffi::OsStringExt;
    let shell: Vec<(OsString, OsString)> = vec![
        (OsString::from("PATH"), OsString::from_vec(b"/a/not-text-\xff:/usr/bin".to_vec())),
        (OsString::from("GIT_SSH"), OsString::from("/usr/bin/ssh")),
    ];
    assert_eq!(
        carried(shell.as_slice()),
        vec![(String::from("GIT_SSH"), String::from("/usr/bin/ssh"))],
        "the unreadable value stays out and the readable one travels"
    );
}

/// What:
///  A written list reads back as it was written,
///  and the record sits where both sides find it.
/// Why:
///   The client writes it and the service reads it without either knowing the other's paths.
#[test]
fn a_written_list_reads_back_whole() {
    let scratch: Scratch = Scratch::new("round-trip");
    let shell: Vec<(OsString, OsString)> = environment(&[("PATH", "/opt/git-lfs:/usr/bin"), ("GIT_SSH", "/usr/bin/ssh")]);
    write(scratch.git_dir.as_path(), shell.as_slice()).expect("the record is written");
    assert_eq!(record_path(scratch.git_dir.as_path()), scratch.git_dir.join(FOLDER).join(FILE), "the record's place is the work folder's");
    let read_back: Vec<(OsString, OsString)> = read(scratch.git_dir.as_path()).expect("the record reads back");
    assert_eq!(read_back, shell, "the list the service searches is the list the shell left");
}

/// What:
///  A record that is missing,
///  half written,
///  of another version or without a `PATH` reads as no list.
/// Why:
///   The service then searches its own list and says so,
/// which refuses the push rather than choosing another program.
#[test]
fn a_record_this_cannot_read_is_no_list() {
    let scratch: Scratch = Scratch::new("unreadable");
    assert_eq!(read(scratch.git_dir.as_path()), None, "nothing was written");

    let record: PathBuf = record_path(scratch.git_dir.as_path());
    std::fs::create_dir_all(record.parent().expect("the record has a folder")).expect("the record's folder");
    std::fs::write(&record, b"{\"v\":1,\"variables\":{\"PATH\":\"/usr/bin\"}").expect("a half-written record");
    assert_eq!(read(scratch.git_dir.as_path()), None, "a torn record is no list");

    std::fs::write(&record, b"{\"v\":2,\"variables\":{\"PATH\":\"/usr/bin\"}}").expect("another version's record");
    assert_eq!(read(scratch.git_dir.as_path()), None, "another version's record is no list");

    std::fs::write(&record, b"{\"v\":1,\"variables\":{\"GIT_SSH\":\"/usr/bin/ssh\"}}").expect("a record without a PATH");
    assert_eq!(read(scratch.git_dir.as_path()), None, "a list without a program folder is no list");
}
