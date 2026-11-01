//! What:
//!  Process-birth identities:
//!  a string that names one process for its whole life and
//!       changes when its PID is reused,
//!  in the incumbent's exact spelling.
//! Why:
//!  Every lock and transaction records its owner's PID and birth identity;
//!  an owner is
//!      alive only while its PID names a process with the recorded identity.
//!  Both wrappers
//!      judge each other's locks,
//!  so the strings must agree byte for byte
//!      (`src/policy-engine/commit-transaction-process-identity.ts`;
//!  the decision in
//!      `doc/handover/cli-git-rust-implementation.md`,
//!  "Adopted without a question").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identity = await resolveProcessBirthIdentity(pid); // 'linux:8423337' or PROCESS_IDENTITY_ABSENT
//! ```

/// Text trimming with the incumbent's `.trim()` semantics.
use super::js_text::trim_javascript;
/// `OnceLock` holds a value computed at most once per process.
use std::sync::OnceLock;

/// What:
///  Why a birth identity could not be decided.
///  Absence (the process exited) is not an
///       error;
///  these are states the incumbent also throws on.
/// Why:
///   A lock acquirer that cannot tell whether an owner lives must stop,
///  not guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ProcessIdentityError extends TypeError {}
/// ```
#[derive(Debug)]
pub enum ProcessIdentityError {
    /// `/proc/<pid>/stat` has no `)` or no start-time field.
    MalformedStat {
        /// The process whose record was read.
        pid: i64,
    },
    /// The record could not be read for a reason other than the process being gone.
    Io {
        /// The process whose record was read.
        pid: i64,
        /// The operating system's refusal.
        error: std::io::Error,
    },
    /// This platform has no identity source.
    UnsupportedPlatform,
    /// The current process has no identity,
    ///  so it cannot own a lock.
    CurrentUnavailable,
}

/// What:
///  Human-readable text for each failure.
/// Why:
///   Diagnostics name the process and the reason in plain words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.message
/// ```
impl std::fmt::Display for ProcessIdentityError {
    /// Writes one sentence naming the process and what went wrong.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedStat { pid } => {
                return write!(
                    formatter,
                    "the Linux process record of PID {pid} is malformed"
                );
            }
            Self::Io { pid, error } => {
                return write!(
                    formatter,
                    "the process record of PID {pid} could not be read: {error}"
                );
            }
            Self::UnsupportedPlatform => {
                return write!(
                    formatter,
                    "this platform has no process birth identity source"
                );
            }
            Self::CurrentUnavailable => {
                return write!(
                    formatter,
                    "the birth identity of the current process is unavailable"
                );
            }
        }
    }
}

/// What:
///  The one way a `/proc/<pid>/stat` text can be malformed for this purpose.
/// Why:
///   The caller knows the PID and turns this into a named error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MalformedStat extends TypeError {}
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MalformedStat;

/// Linux process states of a process that has exited:
///  zombie and dead.
const EXITED_LINUX_STATES: [&str; 2] = ["Z", "X"];

/// Index of the start-time field among the fields after the command name:
///  field 22 is the
/// 20th field counted from field 3,
///  the state.
const LINUX_START_FIELD_INDEX: usize = 19;

/// What:
///  The birth identity in one `/proc/<pid>/stat` text,
///  or nothing for an exited process.
///       `Result<Option<String>, MalformedStat>` is "an identity,
///  nothing,
///  or a malformed record".
/// Why:
///   The incumbent cuts after the last `)` (the command name may hold spaces and `)`),
///       skips `) `,
///  trims,
///  splits on single spaces,
///  treats states `Z` and `X` as exited
///       (a zombie under a container init that never reaps keeps its PID forever),
///  and
///       answers `linux:` plus the start tick.
///  An empty start field is malformed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linuxIdentityFromStat(stat: string): string | typeof ABSENT; // throws when malformed
/// ```
pub fn linux_identity_from_stat(stat: &str) -> Result<Option<String>, MalformedStat> {
    // `rfind` gives the byte position of the last `)`, the end of the command name.
    let Some(command_end) = stat.rfind(')') else {
        return Err(MalformedStat);
    };
    // `get(a..)` is the text from byte `a` on, or nothing past the end; `) ` is two bytes.
    let after: &str = stat.get(command_end + 2..).unwrap_or("");
    // `.split(' ')` yields every piece between single spaces, empty pieces included.
    let fields: Vec<&str> = trim_javascript(after).split(' ').collect();
    // `fields[0]` always exists: splitting yields at least one piece.
    if EXITED_LINUX_STATES.contains(&fields[0]) {
        return Ok(None);
    }
    match fields.get(LINUX_START_FIELD_INDEX) {
        Some(tick) if !tick.is_empty() => return Ok(Some(format!("linux:{tick}"))),
        Some(_) | None => return Err(MalformedStat),
    }
}

/// What:
///  The identity a platform inspector printed,
///  as `<prefix>:<trimmed output>`,
///  or nothing
///       when it printed only whitespace.
///       `&[u8]` borrows the raw standard output.
/// Why:
///   On macOS and Windows the incumbent runs a command and trims its output;
///  empty output
///       means the process is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commandIdentity(prefix: string, stdout: string): string | typeof ABSENT;
/// ```
pub fn command_identity(prefix: &str, stdout: &[u8]) -> Option<String> {
    // `from_utf8_lossy` decodes as Node does for a child's output, replacing invalid bytes.
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(stdout);
    let value: &str = trim_javascript(text.as_ref());
    if value.is_empty() {
        return None;
    }
    return Some(format!("{prefix}:{value}"));
}

/// What:
///  The arguments the incumbent passes to PowerShell for one PID on Windows.
///       `Vec<String>` is an owned list of owned text.
/// Why:
///   The Windows identity is whatever this exact command prints,
///  so running the same
///       command reproduces it by construction;
///  the conversion through `GetProcessTimes`
///       that would avoid PowerShell is unproven (`doc/handover/cli-git-native-transactions.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['-NoLogo', '-NoProfile', '-NonInteractive', '-Command', `(Get-Process -Id ${pid}).StartTime.ToUniversalTime().Ticks`]
/// ```
pub fn windows_identity_arguments(pid: i64) -> Vec<String> {
    return vec![
        String::from("-NoLogo"),
        String::from("-NoProfile"),
        String::from("-NonInteractive"),
        String::from("-Command"),
        format!("(Get-Process -Id {pid}).StartTime.ToUniversalTime().Ticks"),
    ];
}

/// What:
///  The arguments the incumbent passes to `ps` for one PID on macOS.
/// Why:
///   `ps -o lstart=` prints the start time in the C locale;
///  running it reproduces the
///       incumbent's string by construction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['-o', 'lstart=', '-p', String(pid)]
/// ```
pub fn darwin_identity_arguments(pid: i64) -> Vec<String> {
    return vec![
        String::from("-o"),
        String::from("lstart="),
        String::from("-p"),
        pid.to_string(),
    ];
}

/// What:
///  Run a platform inspector in the C locale and read its identity line.
/// Why:
///   The incumbent's `nano-spawn` call adds `LC_ALL=C` to the inherited environment and
///       maps every failure to start or finish (a non-zero exit) to "absent".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveCommandIdentity({ command, args, prefix }): Promise<string | typeof ABSENT>;
/// ```
#[cfg_attr(
    all(target_os = "linux", not(test)),
    allow(
        dead_code,
        reason = "the release build uses it on macOS and Windows only"
    )
)]
fn inspector_identity(command: &str, arguments: &[String], prefix: &str) -> Option<String> {
    let output: std::process::Output = std::process::Command::new(command)
        .args(arguments)
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    return command_identity(prefix, output.stdout.as_slice());
}

/// What:
///  Read `/proc/<pid>/stat` and decide the Linux identity.
/// Why:
///   A missing record (`ENOENT`) or a process that exits mid-read (`ESRCH`) is an exited
///       process;
///  any other read failure is an error,
///  as in the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveLinuxIdentity(pid: number): Promise<string | typeof ABSENT>;
/// ```
#[cfg(target_os = "linux")]
fn linux_identity(pid: i64) -> Result<Option<String>, ProcessIdentityError> {
    let bytes: Vec<u8> = match std::fs::read(format!("/proc/{pid}/stat")) {
        Ok(read) => read,
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                || error.raw_os_error() == Some(libc::ESRCH) =>
        {
            return Ok(None);
        }
        Err(error) => return Err(ProcessIdentityError::Io { pid, error }),
    };
    // Node reads the file as UTF-8, replacing invalid bytes in the command name.
    let stat: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
    match linux_identity_from_stat(stat.as_ref()) {
        Ok(identity) => return Ok(identity),
        Err(MalformedStat) => return Err(ProcessIdentityError::MalformedStat { pid }),
    }
}

/// What:
///  The birth identity of the process `pid` names now,
///  or nothing when none runs.
///       `i64` holds any PID a record can carry.
/// Why:
///   This is the one question every liveness check asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveProcessBirthIdentity(pid: number): Promise<string | typeof PROCESS_IDENTITY_ABSENT>;
/// ```
pub fn process_birth_identity(pid: i64) -> Result<Option<String>, ProcessIdentityError> {
    #[cfg(target_os = "linux")]
    {
        return linux_identity(pid);
    }
    #[cfg(target_os = "macos")]
    {
        return Ok(inspector_identity(
            "ps",
            darwin_identity_arguments(pid).as_slice(),
            "darwin",
        ));
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(windows)]
    {
        return Ok(inspector_identity(
            "powershell.exe",
            windows_identity_arguments(pid).as_slice(),
            "win32",
        ));
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        let _ = pid;
        return Err(ProcessIdentityError::UnsupportedPlatform);
    }
}

/// The current process's identity,
///  computed once:
///  it never changes while the process runs.
static CURRENT_IDENTITY: OnceLock<String> = OnceLock::new();

/// What:
///  The birth identity of this process,
///  computed once per process.
/// Why:
///   Every lock this process takes records it,
///  and on macOS and Windows each computation
///       starts a process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const own = await resolveProcessBirthIdentity(process.pid);
/// ```
pub fn current_birth_identity() -> Result<String, ProcessIdentityError> {
    if let Some(known) = CURRENT_IDENTITY.get() {
        return Ok(known.clone());
    }
    let computed: String = process_birth_identity(i64::from(std::process::id()))?
        .ok_or(ProcessIdentityError::CurrentUnavailable)?;
    // Another thread may have stored it meanwhile; both values are the same, so a lost race
    // to store is harmless.
    let _ = CURRENT_IDENTITY.set(computed.clone());
    return Ok(computed);
}

/// What:
///  Whether a signal could be sent to `pid` (it names some process),
///  on Unix.
///       `Ok(false)` means no such process;
///  an error is a refusal such as `EPERM`.
/// Why:
///   The incumbent's `processIsAlive` asks `kill(pid, 0)` before comparing identities;
///       a process of another account answers `EPERM`,
///  which callers judge themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function processIsAlive(pid: number): boolean { try { process.kill(pid, 0); return true; } catch (e) { if (e.code === 'ESRCH') return false; throw e; } }
/// ```
#[cfg(unix)]
pub fn signal_probe(pid: i64) -> std::io::Result<bool> {
    // A PID beyond `pid_t` cannot name a process.
    let Ok(target) = libc::pid_t::try_from(pid) else {
        return Ok(false);
    };
    if target <= 0 {
        return Ok(false);
    }
    // SAFETY: `kill` with signal 0 sends nothing; it only checks that the target exists and
    // may be signaled. It reads no memory of this process.
    let status: libc::c_int = unsafe { libc::kill(target, 0) };
    if status == 0 {
        return Ok(true);
    }
    let error: std::io::Error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        return Ok(false);
    }
    return Err(error);
}

/// Parser controls and live-process probes stay out of the release executable.
#[cfg(test)]
#[path = "process_identity_tests.rs"]
mod tests;
