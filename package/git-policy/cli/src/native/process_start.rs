//! What:
//!  The wall-clock start time of a process,
//!  with its resolution,
//!  for comparing a Git lock
//!       PID file's owner with the lock's change time.
//! Why:
//!  A PID file names a live owner only when that process started no later than the lock
//!      changed;
//!  a younger process reused the PID.
//!  Linux derives the start from
//!      `/proc/<pid>/stat` field 22,
//!  `/proc/uptime` and the current time;
//!  macOS reads
//!      `ps -o lstart=`;
//!  Windows reads `Get-Process` `StartTime`
//!      (`src/index-lock/process-start-time.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const start = await resolveProcessStart({ pid });
//! ```

/// Process existence probes.
#[cfg(unix)]
use super::process_identity::signal_probe;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Clock ticks per second in `/proc/<pid>/stat`,
///  the kernel's fixed `USER_HZ`.
pub const LINUX_USER_HZ: i64 = 100;

/// Resolution of a Linux start time:
///  one tick plus the centisecond resolution of `/proc/uptime`.
pub const LINUX_RESOLUTION_MS: i64 = 2 * (1000 / LINUX_USER_HZ);

/// Resolution of `ps -o lstart=`,
///  which prints whole seconds.
pub const DARWIN_RESOLUTION_MS: i64 = 1000;

/// Resolution granted to Windows start times.
pub const WINDOWS_RESOLUTION_MS: i64 = 1;

/// Index of the start-tick field among the fields after the command name (field 22 overall).
pub const START_TICKS_FIELD_INDEX: usize = 19;

/// What:
///  When a process started,
///  or why that is not known.
/// Why:
///   `ProcessStart`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProcessStart = { kind: 'running'; startedAtMs; resolutionMs } | { kind: 'missing' } | { kind: 'unknown'; reason };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum ProcessStart {
    /// The process runs.
    Running {
        /// Start time in milliseconds since the epoch.
        started_at_ms: f64,
        /// Clock granularity to allow for when comparing.
        resolution_ms: i64,
    },
    /// No such process runs,
    ///  or it is a zombie.
    Missing,
    /// The process may run but its start time is unreadable.
    Unknown(
        /// Why the start time is not known.
        String,
    ),
}

/// What:
///  The state letter and start ticks of a `/proc/<pid>/stat` line.
/// Why:
///   `parseLinuxStat`:
///  the command name ends at the last `)`;
///  field 22 must be a safe
///       integer as `Number` reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseLinuxStat(stat: string): { state: string; startTicks: number }; // throws TypeError
/// ```
pub fn parse_linux_stat(stat: &str) -> Option<(String, i64)> {
    let command_end: usize = stat.rfind(')')?;
    let after: &str = stat.get(command_end + 2..).unwrap_or("");
    let fields: Vec<&str> = super::js_text::trim_javascript(after).split(' ').collect();
    let state: &str = fields.first().copied().unwrap_or("");
    let ticks_text: &str = fields.get(START_TICKS_FIELD_INDEX).copied()?;
    if state.is_empty() || ticks_text.is_empty() || !ticks_text.bytes().all(is_digit) {
        return None;
    }
    let ticks: i64 = ticks_text.parse::<i64>().ok()?;
    if ticks > super::json_record::MAX_SAFE_INTEGER {
        return None;
    }
    return Some((String::from(state), ticks));
}

/// What:
///  Whether a byte is an ASCII digit.
/// Why:
///   A named predicate keeps the scan free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte >= 0x30 && byte <= 0x39
/// ```
fn is_digit(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// What:
///  The seconds since boot in `/proc/uptime` text,
///  its first field.
/// Why:
///   The start is `now - uptime + ticks / USER_HZ`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(text.split(' ')[0])
/// ```
pub fn parse_uptime(text: &str) -> Option<f64> {
    let first: &str = text.split(' ').next()?;
    let trimmed: &str = super::js_text::trim_javascript(first);
    // `Number('')` is zero, so an empty first field reads as no time since boot.
    let seconds: f64 = if trimmed.is_empty() { 0.0 } else { trimmed.parse::<f64>().ok()? };
    if !seconds.is_finite() {
        return None;
    }
    return Some(seconds);
}

/// What:
///  The current time in milliseconds since the epoch.
/// Why:
///   `Date.now()`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Date.now()
/// ```
pub fn now_ms() -> f64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => return elapsed.as_secs_f64() * 1000.0,
        Err(_) => return 0.0,
    }
}

/// What:
///  A Linux process's start from a proc root.
/// Why:
///   `linuxStart`;
///  a vanished stat file or a zombie is a missing process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function linuxStart({ pid, procRoot, now }): Promise<ProcessStart>;
/// ```
pub fn linux_start(pid: i64, proc_root: &Path, now: f64) -> ProcessStart {
    let stat_path: PathBuf = proc_root.join(pid.to_string()).join("stat");
    let stat: String = match std::fs::read(&stat_path) {
        Ok(bytes) => String::from_utf8_lossy(bytes.as_slice()).into_owned(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return ProcessStart::Missing,
        #[cfg(unix)]
        Err(error) if error.raw_os_error() == Some(libc::ESRCH) => return ProcessStart::Missing,
        Err(error) => return ProcessStart::Unknown(format!("reading {} failed: {error}", stat_path.display())),
    };
    let Some((state, ticks)) = parse_linux_stat(stat.as_str()) else {
        return ProcessStart::Unknown(String::from("Malformed Linux process stat."));
    };
    if state == "Z" || state == "X" {
        return ProcessStart::Missing;
    }
    let uptime_path: PathBuf = proc_root.join("uptime");
    let uptime: f64 = match std::fs::read_to_string(&uptime_path).ok().as_deref().and_then(parse_uptime) {
        Some(seconds) => seconds,
        None => return ProcessStart::Unknown(String::from("Malformed /proc/uptime.")),
    };
    // Ticks are at most 2^53 - 1, so the conversion to a double is exact.
    let ticks_ms: f64 = (ticks as f64) * 1000.0 / (LINUX_USER_HZ as f64);
    return ProcessStart::Running {
        started_at_ms: (now - uptime * 1000.0) + ticks_ms,
        resolution_ms: LINUX_RESOLUTION_MS,
    };
}

/// What:
///  Month numbers of the C locale's abbreviated month names.
/// Why:
///   `ps -o lstart=` prints `Fri Sep 26 10:00:00 2026`.
const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// What:
///  The fields of a `ps -o lstart=` line:
///  year,
///  month (0 to 11),
///  day,
///  hour,
///  minute,
///  second.
/// Why:
///   `Date.parse` reads this local-time form;
///  the conversion to an instant happens apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Date.parse('Fri Sep 26 10:00:00 2026')
/// ```
pub fn parse_lstart(text: &str) -> Option<(i32, i32, i32, i32, i32, i32)> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() != 5 {
        return None;
    }
    let month: i32 = month_number(words[1])?;
    let day: i32 = words[2].parse::<i32>().ok()?;
    let clock: Vec<&str> = words[3].split(':').collect();
    if clock.len() != 3 {
        return None;
    }
    let hour: i32 = clock[0].parse::<i32>().ok()?;
    let minute: i32 = clock[1].parse::<i32>().ok()?;
    let second: i32 = clock[2].parse::<i32>().ok()?;
    let year: i32 = words[4].parse::<i32>().ok()?;
    return Some((year, month, day, hour, minute, second));
}

/// What:
///  The month number (0 to 11) of a C-locale abbreviated month name.
/// Why:
///   A named search keeps the parser free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// MONTHS.indexOf(name)
/// ```
fn month_number(name: &str) -> Option<i32> {
    for (index, month) in MONTHS.iter().enumerate() {
        if *month == name {
            return i32::try_from(index).ok();
        }
    }
    return None;
}

/// What:
///  Whether a process with this PID exists;
///  a permission denial means it does.
/// Why:
///   `processExists`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function processExists(pid: number): boolean;
/// ```
pub fn process_exists(pid: i64) -> bool {
    #[cfg(unix)]
    {
        match signal_probe(pid) {
            Ok(alive) => return alive,
            Err(error) => return error.kind() == std::io::ErrorKind::PermissionDenied,
        }
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = pid;
        return true;
    }
}

/// What:
///  Whether a PID names a running process and when it started.
/// Why:
///   `resolveProcessStart` with the host's mechanism.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveProcessStart({ pid }): Promise<ProcessStart>;
/// ```
pub fn resolve_process_start(pid: i64) -> ProcessStart {
    if !process_exists(pid) {
        return ProcessStart::Missing;
    }
    #[cfg(target_os = "linux")]
    {
        return linux_start(pid, Path::new("/proc"), now_ms());
    }
    #[cfg(target_os = "macos")]
    {
        return darwin_start(pid);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(windows)]
    {
        return windows_start(pid);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        return ProcessStart::Unknown(String::from("no start-time source on this platform"));
    }
}

/// What:
///  A macOS process's start through `ps -o lstart=`,
///  read as local time.
/// Why:
///   `darwinStart`;
///  `mktime` applies the local time zone as `Date.parse` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function darwinStart(pid: number): Promise<ProcessStart>;
/// ```
#[cfg(target_os = "macos")]
fn darwin_start(pid: i64) -> ProcessStart {
    let output: std::process::Output = match std::process::Command::new("ps")
        .args(["-o", "lstart=", "-p", pid.to_string().as_str()])
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
    {
        Ok(finished) => finished,
        Err(error) => return ProcessStart::Unknown(format!("ps could not start: {error}")),
    };
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
    let Some((year, month, day, hour, minute, second)) = parse_lstart(text.as_str()) else {
        return ProcessStart::Unknown(format!("unparsable ps start time {text:?}"));
    };
    // SAFETY: `tm` is plain data, fully initialized here; `mktime` only reads and normalizes it.
    let mut broken: libc::tm = unsafe { std::mem::zeroed() };
    broken.tm_year = year - 1900;
    broken.tm_mon = month;
    broken.tm_mday = day;
    broken.tm_hour = hour;
    broken.tm_min = minute;
    broken.tm_sec = second;
    broken.tm_isdst = -1;
    // SAFETY: `broken` is a valid, exclusively borrowed `tm`.
    let seconds: libc::time_t = unsafe { libc::mktime(&mut broken) };
    if seconds == -1 {
        return ProcessStart::Unknown(format!("unparsable ps start time {text:?}"));
    }
    return ProcessStart::Running {
        started_at_ms: (seconds as f64) * 1000.0,
        resolution_ms: DARWIN_RESOLUTION_MS,
    };
}

/// What:
///  A Windows process's start through PowerShell,
///  in .NET ticks.
/// Why:
///   `win32Start`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function win32Start(pid: number): Promise<ProcessStart>;
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
fn windows_start(pid: i64) -> ProcessStart {
    let command: String = format!("(Get-Process -Id {pid}).StartTime.ToUniversalTime().Ticks");
    let output: std::process::Output = match std::process::Command::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", command.as_str()])
        .stdin(std::process::Stdio::null())
        .output()
    {
        Ok(finished) => finished,
        Err(error) => return ProcessStart::Unknown(format!("powershell could not start: {error}")),
    };
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
    let Ok(ticks) = text.parse::<i128>() else {
        return ProcessStart::Unknown(format!("unparsable start ticks {text:?}"));
    };
    let milliseconds: i128 = (ticks - 621_355_968_000_000_000) / 10_000;
    return ProcessStart::Running {
        started_at_ms: milliseconds as f64,
        resolution_ms: WINDOWS_RESOLUTION_MS,
    };
}

/// Start-time controls stay out of the release executable.
#[cfg(test)]
#[path = "process_start_tests.rs"]
mod tests;
