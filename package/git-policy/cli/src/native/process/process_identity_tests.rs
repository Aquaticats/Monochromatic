//! Controls for birth identities:
//!  the `/proc` parser,
//!  the inspector formatting the incumbent
//! uses on macOS and Windows,
//!  and live processes on this host.

use super::*;

/// A `/proc/<pid>/stat` line with the given command and state,
///  start tick 8423337.
fn stat_line(command: &str, state: &str) -> String {
    return format!(
        "4242 ({command}) {state} 1 4242 4242 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 1 0 8423337 \
         1000 100 18446744073709551615 1 1 0 0 0 0 0 0 0 0 0 0 17 3 0 0 0 0 0\n"
    );
}

/// The start tick follows the last `)`,
///  whatever the command name holds.
#[test]
fn the_start_tick_follows_the_last_parenthesis() {
    assert_eq!(
        linux_identity_from_stat(stat_line("git", "S").as_str()),
        Ok(Some(String::from("linux:8423337")))
    );
    assert_eq!(
        linux_identity_from_stat(stat_line("a) b (c) d", "R").as_str()),
        Ok(Some(String::from("linux:8423337")))
    );
    assert_eq!(
        linux_identity_from_stat(stat_line("", "D").as_str()),
        Ok(Some(String::from("linux:8423337")))
    );
}

/// Zombie and dead processes have no identity.
#[test]
fn exited_states_have_no_identity() {
    assert_eq!(
        linux_identity_from_stat(stat_line("git", "Z").as_str()),
        Ok(None)
    );
    assert_eq!(
        linux_identity_from_stat(stat_line("git", "X").as_str()),
        Ok(None)
    );
    assert_eq!(
        linux_identity_from_stat(stat_line("git", "z").as_str()),
        Ok(Some(String::from("linux:8423337")))
    );
}

/// A record without a `)`,
///  without enough fields,
///  or with an empty start field is malformed.
#[test]
fn malformed_records_are_refused() {
    assert_eq!(
        linux_identity_from_stat("4242 git S 1 2 3"),
        Err(MalformedStat)
    );
    assert_eq!(
        linux_identity_from_stat("4242 (git) S 1 2 3"),
        Err(MalformedStat)
    );
    assert_eq!(linux_identity_from_stat("4242 (git)"), Err(MalformedStat));
    assert_eq!(linux_identity_from_stat(")"), Err(MalformedStat));
    // A double space leaves an empty piece where the start tick should be.
    let gap: String = stat_line("git", "S").replace(" 8423337", "  8423337");
    assert_eq!(linux_identity_from_stat(gap.as_str()), Err(MalformedStat));
}

/// Exactly 20 fields after the command name is the shortest record that has a start tick.
#[test]
fn the_twentieth_field_is_the_start_tick() {
    let mut fields: Vec<String> = Vec::new();
    for number in 3..=22 {
        fields.push(number.to_string());
    }
    let line: String = format!("1 (x) {}", fields.join(" "));
    assert_eq!(
        linux_identity_from_stat(line.as_str()),
        Ok(Some(String::from("linux:22")))
    );
    let short: String = format!("1 (x) {}", fields[..19].join(" "));
    assert_eq!(linux_identity_from_stat(short.as_str()), Err(MalformedStat));
}

/// Inspector output becomes `<prefix>:<trimmed>`,
///  and whitespace alone is no identity.
#[test]
fn inspector_output_is_prefixed_and_trimmed() {
    assert_eq!(
        command_identity("darwin", b"Mon Oct  6 16:34:01 2026\n"),
        Some(String::from("darwin:Mon Oct  6 16:34:01 2026"))
    );
    assert_eq!(
        command_identity("win32", b"\r\n638950000000000000\r\n"),
        Some(String::from("win32:638950000000000000"))
    );
    assert_eq!(command_identity("darwin", b" \n\t"), None);
    assert_eq!(command_identity("darwin", b""), None);
    assert_eq!(
        command_identity("x", b"a\xffb"),
        Some(String::from("x:a\u{fffd}b"))
    );
}

/// The inspector command lines are exactly the incumbent's.
#[test]
fn inspector_arguments_are_the_incumbents() {
    assert_eq!(
        darwin_identity_arguments(42),
        ["-o", "lstart=", "-p", "42"].map(String::from).to_vec()
    );
    assert_eq!(
        windows_identity_arguments(42),
        [
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-Process -Id 42).StartTime.ToUniversalTime().Ticks"
        ]
        .map(String::from)
        .to_vec()
    );
}

/// Running an inspector reads its trimmed output;
///  a failing or missing one means absent.
#[cfg(unix)]
#[test]
fn inspectors_run_in_the_c_locale() {
    let printed: Vec<String> = vec![
        String::from("-c"),
        String::from("printf ' %s \\n' \"$LC_ALL\""),
    ];
    assert_eq!(
        inspector_identity("sh", printed.as_slice(), "probe"),
        Some(String::from("probe:C"))
    );
    assert_eq!(inspector_identity("false", &[], "probe"), None);
    assert_eq!(
        inspector_identity("/nonexistent/inspector", &[], "probe"),
        None
    );
    let blank: Vec<String> = vec![String::from("-c"), String::from("printf '  '")];
    assert_eq!(inspector_identity("sh", blank.as_slice(), "probe"), None);
}

/// This process has an identity,
///  and it is computed once.
#[cfg(target_os = "linux")]
#[test]
fn the_current_process_has_a_stable_identity() {
    let own: String = current_birth_identity().expect("own identity");
    assert!(own.starts_with("linux:"));
    assert_eq!(current_birth_identity().expect("again"), own);
    assert_eq!(
        process_birth_identity(i64::from(std::process::id())).expect("probe"),
        Some(own)
    );
}

/// A running child has an identity;
///  once it exits and is reaped it has none,
///  and while it
/// is an unreaped zombie it has none either.
#[cfg(target_os = "linux")]
#[test]
fn children_lose_their_identity_when_they_exit() {
    let mut running: std::process::Child = crate::test_support::spawn_owned(std::process::Command::new("sleep").arg("30"))
        .expect("start sleep");
    let pid: i64 = i64::from(running.id());
    let identity: String = process_birth_identity(pid)
        .expect("probe running")
        .expect("a running child has an identity");
    assert!(identity.starts_with("linux:"));
    assert!(signal_probe(pid).expect("signal probe"));
    running.kill().expect("kill");
    // Until `wait`, the killed child is a zombie: it keeps its PID but has no identity.
    let mut zombie: bool = false;
    for _ in 0..200 {
        let stat: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
        if stat.contains(") Z ") {
            zombie = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(zombie, "the killed child became a zombie");
    assert_eq!(process_birth_identity(pid).expect("probe zombie"), None);
    running.wait().expect("reap");
    assert_eq!(process_birth_identity(pid).expect("probe reaped"), None);
    assert!(!signal_probe(pid).expect("signal probe reaped"));
}

/// PIDs that cannot name a process are not signalable.
#[cfg(unix)]
#[test]
fn impossible_pids_are_not_signalable() {
    assert!(!signal_probe(0).expect("zero"));
    assert!(!signal_probe(-1).expect("negative"));
    assert!(!signal_probe(i64::from(i32::MAX) + 1).expect("beyond pid_t"));
    assert!(signal_probe(i64::from(std::process::id())).expect("own"));
}

/// A process of another account is reported,
///  not judged.
#[cfg(target_os = "linux")]
#[test]
fn another_accounts_process_is_an_error_or_signalable() {
    // PID 1 belongs to the container's init or the host's; an unprivileged test sees `EPERM`.
    match signal_probe(1) {
        Ok(signalable) => assert!(signalable),
        Err(error) => assert_eq!(error.raw_os_error(), Some(libc::EPERM)),
    }
}

/// A missing process record is absent;
///  an unreadable path is an error with the PID named.
#[cfg(target_os = "linux")]
#[test]
fn missing_records_are_absent() {
    assert_eq!(
        process_birth_identity(i64::from(i32::MAX)).expect("probe"),
        None
    );
    assert_eq!(process_birth_identity(-7).expect("negative"), None);
}

/// Every failure names the process or the platform.
#[test]
fn failures_are_described() {
    assert_eq!(
        ProcessIdentityError::MalformedStat { pid: 7 }.to_string(),
        "the Linux process record of PID 7 is malformed"
    );
    assert_eq!(
        ProcessIdentityError::Io {
            pid: 7,
            error: std::io::Error::from(std::io::ErrorKind::PermissionDenied)
        }
        .to_string(),
        "the process record of PID 7 could not be read: permission denied"
    );
    assert_eq!(
        ProcessIdentityError::UnsupportedPlatform.to_string(),
        "this platform has no process birth identity source"
    );
    assert_eq!(
        ProcessIdentityError::CurrentUnavailable.to_string(),
        "the birth identity of the current process is unavailable"
    );
}

/// A record that cannot be read says the process is gone only when it is missing or the process exited during the read;
/// a refusal,
/// a record that is a directory,
/// and any other failure say nothing about the process,
/// and reading them as "gone" would let a lock of a running service be taken.
#[cfg(target_os = "linux")]
#[test]
fn only_a_missing_record_or_an_exit_during_the_read_says_the_process_is_gone() {
    use super::record_is_gone;
    assert!(record_is_gone(&std::io::Error::from(std::io::ErrorKind::NotFound)));
    assert!(record_is_gone(&std::io::Error::from_raw_os_error(libc::ENOENT)));
    assert!(record_is_gone(&std::io::Error::from_raw_os_error(libc::ESRCH)));
    for refusal in [libc::EACCES, libc::EPERM, libc::EISDIR, libc::EIO, libc::EMFILE, libc::EINTR] {
        assert!(!record_is_gone(&std::io::Error::from_raw_os_error(refusal)), "errno {refusal} says nothing about the process");
    }
    assert!(!record_is_gone(&std::io::Error::from(std::io::ErrorKind::PermissionDenied)));
    assert!(!record_is_gone(&std::io::Error::other("something else")));
}

/// A record that is missing or whose process exited during the read is an exited process;
/// a refusal and an I/O error are
/// errors naming the process,
/// and a record that reads is parsed.
#[cfg(target_os = "linux")]
#[test]
fn a_failure_to_read_the_record_is_an_exit_only_when_it_says_so() {
    use super::linux_identity_read_by;
    let failing = |code: i32| { return move |_: String| -> std::io::Result<Vec<u8>> { return Err(std::io::Error::from_raw_os_error(code)); }; };
    assert_eq!(linux_identity_read_by(4242, failing(libc::ENOENT)).expect("gone is not an error"), None);
    assert_eq!(linux_identity_read_by(4242, failing(libc::ESRCH)).expect("gone is not an error"), None);
    for code in [libc::EACCES, libc::EIO, libc::EISDIR] {
        match linux_identity_read_by(4242, failing(code)) {
            Err(ProcessIdentityError::Io { pid, error }) => {
                assert_eq!(pid, 4242);
                assert_eq!(error.raw_os_error(), Some(code));
            },
            other => panic!("errno {code} must be an error naming the process, got {other:?}"),
        }
    }
    let seen_path = std::cell::RefCell::new(String::new());
    let read = linux_identity_read_by(4242, |path| {
        *seen_path.borrow_mut() = path;
        return Ok(stat_line("git", "S").into_bytes());
    });
    assert_eq!(read.expect("a record that reads"), Some(String::from("linux:8423337")));
    assert_eq!(seen_path.borrow().as_str(), "/proc/4242/stat");
}
