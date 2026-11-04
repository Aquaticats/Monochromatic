//! Controls for process start times:
//!  the `/proc` stat and uptime parsers,
//!  the `ps -o lstart=`
//! parser,
//!  and the Linux reader run against a fixture proc root and against this process.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// A `/proc/<pid>/stat` line with the given command and state;
///  start tick 4200.
fn stat_line(command: &str, state: &str) -> String {
    return format!(
        "4242 ({command}) {state} 1 4242 4242 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 1 0 4200 \
         1000 100 18446744073709551615 1 1 0 0 0 0 0 0 0 0 0 0 17 3 0 0 0 0 0\n"
    );
}

/// Write a fake proc root with one process `pid` in `state` and an uptime of `uptime`.
fn plant_proc(root: &Path, pid: i64, stat: &str, uptime: &str) {
    let directory: PathBuf = root.join(pid.to_string());
    std::fs::create_dir_all(&directory).expect("fake process directory");
    std::fs::write(directory.join("stat"), stat).expect("fake stat");
    std::fs::write(root.join("uptime"), uptime).expect("fake uptime");
}

/// The start-tick field is found after the last `)`,
///  whatever the command name holds.
#[test]
fn the_start_tick_follows_the_last_parenthesis() {
    assert_eq!(
        parse_linux_stat(stat_line("git", "S").as_str()),
        Some((String::from("S"), 4200))
    );
    assert_eq!(
        parse_linux_stat(stat_line("a) b (c) d", "R").as_str()),
        Some((String::from("R"), 4200))
    );
    assert_eq!(
        parse_linux_stat(stat_line("", "D").as_str()),
        Some((String::from("D"), 4200))
    );
}

/// A stat line without a `)`,
///  with too few fields,
///  or with a non-digit start tick is refused.
#[test]
fn malformed_stat_lines_are_refused() {
    assert_eq!(parse_linux_stat("4242 git S 1 2 3"), None);
    assert_eq!(parse_linux_stat("4242 (git) S 1 2 3"), None);
    assert_eq!(parse_linux_stat("4242 (git)"), None);
    assert_eq!(parse_linux_stat(")"), None);
    assert_eq!(parse_linux_stat(stat_line("git", "S").replace("4200", "-4200").as_str()), None);
    assert_eq!(parse_linux_stat(stat_line("git", "S").replace("4200", "42x").as_str()), None);
}

/// A start tick above the largest safe integer is refused,
///  as the incumbent's `Number` read would lose it.
#[test]
fn a_start_tick_beyond_the_safe_integers_is_refused() {
    let beyond: String = stat_line("git", "S").replace("4200", "9007199254740992");
    assert_eq!(parse_linux_stat(beyond.as_str()), None);
    let largest: String = stat_line("git", "S").replace("4200", "9007199254740991");
    assert_eq!(
        parse_linux_stat(largest.as_str()),
        Some((String::from("S"), 9_007_199_254_740_991))
    );
}

/// The uptime is the first field of `/proc/uptime`,
///  and a non-finite or non-numeric one is refused.
#[test]
fn uptime_reads_its_first_field_only() {
    assert_eq!(parse_uptime("12345.67 890.12\n"), Some(12345.67));
    assert_eq!(parse_uptime(" 7.5 1.0"), Some(0.0));
    assert_eq!(parse_uptime("\t7.5 1.0"), Some(7.5));
    assert_eq!(parse_uptime("abc 1.0"), None);
    assert_eq!(parse_uptime(""), Some(0.0));
    assert_eq!(parse_uptime("inf 1.0"), None);
    assert_eq!(parse_uptime("NaN 1.0"), None);
}

/// The `ps -o lstart=` form reads its fields as the C locale prints them.
#[test]
fn lstart_fields_are_read_in_order() {
    assert_eq!(
        parse_lstart("Fri Sep 26 10:00:00 2026"),
        Some((2026, 8, 26, 10, 0, 0))
    );
    assert_eq!(
        parse_lstart("Mon Jan  5 09:08:07 2026"),
        Some((2026, 0, 5, 9, 8, 7))
    );
    assert_eq!(
        parse_lstart("Thu Dec 31 23:59:58 2026"),
        Some((2026, 11, 31, 23, 59, 58))
    );
}

/// Other word counts,
///  unknown month names,
///  short clocks and non-numeric fields are refused.
#[test]
fn malformed_lstart_text_is_refused() {
    assert_eq!(parse_lstart(""), None);
    assert_eq!(parse_lstart("Fri Sep 26 10:00:00"), None);
    assert_eq!(parse_lstart("Fri Smarch 26 10:00:00 2026"), None);
    assert_eq!(parse_lstart("Fri Sep 26 10:00 2026"), None);
    assert_eq!(parse_lstart("Fri Sep xx 10:00:00 2026"), None);
    assert_eq!(parse_lstart("Fri Sep 26 10:00:xx 2026"), None);
    assert_eq!(parse_lstart("Fri Sep 26 10:00:00 year"), None);
}

/// A Linux start is the boot-relative time subtracted from now,
///  plus the start ticks.
#[test]
fn a_running_process_starts_at_now_minus_uptime_plus_its_ticks() {
    let root: PathBuf = fixture("process-start-running");
    plant_proc(&root, 4242, stat_line("git", "S").as_str(), "100.00 0.00\n");
    // Start at 4200 ticks is 42 seconds after boot; boot was 100 seconds before `now`.
    let start: ProcessStart = linux_start(4242, root.as_path(), 1_000_000.0);
    assert_eq!(
        start,
        ProcessStart::Running {
            started_at_ms: 1_000_000.0 - 100_000.0 + 42_000.0,
            resolution_ms: LINUX_RESOLUTION_MS,
        }
    );
    remove(root.as_path());
}

/// A zombie or dead process is missing,
///  whichever state letter it shows.
#[test]
fn zombie_and_dead_processes_are_missing() {
    let root: PathBuf = fixture("process-start-exited");
    plant_proc(&root, 4242, stat_line("git", "Z").as_str(), "100.00 0.00\n");
    assert_eq!(linux_start(4242, root.as_path(), 1.0), ProcessStart::Missing);
    plant_proc(&root, 4242, stat_line("git", "X").as_str(), "100.00 0.00\n");
    assert_eq!(linux_start(4242, root.as_path(), 1.0), ProcessStart::Missing);
    remove(root.as_path());
}

/// A PID with no stat file is missing,
///  and an unreadable directory entry is unknown.
#[test]
fn an_absent_stat_file_is_missing() {
    let root: PathBuf = fixture("process-start-absent");
    assert_eq!(linux_start(4242, root.as_path(), 1.0), ProcessStart::Missing);
    remove(root.as_path());
}

/// A malformed stat line or uptime file is unknown,
///  never a guessed start time.
#[test]
fn malformed_proc_files_are_unknown() {
    let root: PathBuf = fixture("process-start-malformed");
    plant_proc(&root, 4242, "4242 git S 1\n", "100.00 0.00\n");
    assert_eq!(
        linux_start(4242, root.as_path(), 1.0),
        ProcessStart::Unknown(String::from("Malformed Linux process stat."))
    );
    plant_proc(&root, 4242, stat_line("git", "S").as_str(), "not-a-number\n");
    assert_eq!(
        linux_start(4242, root.as_path(), 1.0),
        ProcessStart::Unknown(String::from("Malformed /proc/uptime."))
    );
    remove(root.as_path());
}

/// This test process is a live process with a known Linux start,
///  and no missing PID is live.
#[test]
#[cfg(target_os = "linux")]
fn this_process_has_a_start_on_the_host() {
    let own: i64 = i64::from(std::process::id());
    assert!(process_exists(own));
    match resolve_process_start(own) {
        ProcessStart::Running {
            started_at_ms,
            resolution_ms,
        } => {
            assert!(started_at_ms <= now_ms() + (LINUX_RESOLUTION_MS as f64));
            assert_eq!(resolution_ms, LINUX_RESOLUTION_MS);
        }
        other => panic!("this process should be running, got {other:?}"),
    }
    assert_eq!(resolve_process_start(4_000_000_000), ProcessStart::Missing);
    assert!(!process_exists(4_000_000_000));
}
