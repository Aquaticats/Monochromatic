//! What:
//!  A harness that runs the real `file-enforcer` binary in a scratch worktree with its own runtime directory.
//! Why:
//!  Every test that starts a server owns its end:
//! servers get a short idle time,
//! and a drop guard stops any that
//!  remain,
//! so nothing outlives the test and nothing lands in the real `$XDG_RUNTIME_DIR`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class Harness { run(args: string[]): { code: number; lines: object[] } }
//! ```

use serde_json::Value;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;

/// What:
///  One scratch worktree,
/// one runtime directory and one home directory,
/// plus the guard that stops stray servers.
/// Why:
///  The runtime directory sits under `${HOME}/temp/agent` so socket paths stay short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Harness { repo: string; runtime: string; home: string }
/// ```
pub struct Harness {
    pub repo: TempDir,
    pub runtime: TempDir,
    pub home: TempDir,
}

/// What:
///  Create a scratch directory under the build directory (or the short runtime base).
/// Why:
///  `clippy.toml` bans `std::env::temp_dir`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scratchIn(base: string): TempDir
/// ```
fn scratch_in(base: &Path) -> TempDir {
    std::fs::create_dir_all(base).expect("the scratch base is creatable");
    return TempDir::new_in(base).expect("a scratch directory is creatable");
}

impl Harness {
    /// Build a harness with an empty repository.
    pub fn new() -> Harness {
        let target_scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join("test-scratch");
        let home_base = std::env::var("HOME").expect("HOME is set");
        let runtime_base = Path::new(&home_base).join("temp/agent/file-enforcer-test");
        let harness = Harness {
            repo: scratch_in(&target_scratch),
            runtime: scratch_in(&runtime_base),
            home: scratch_in(&runtime_base),
        };
        // `TempDir::new_in` creates directories with the process umask's default mode here (0755), and the server
        // rightly refuses a runtime directory that group or others can enter, so the harness makes it owner-only.
        std::fs::set_permissions(harness.runtime.path(), std::fs::Permissions::from_mode(0o700)).expect("chmod");
        return harness;
    }

    /// Write a file under the repository,
    /// creating directories.
    pub fn write(&self, path: &str, text: &str) {
        let target = self.repo.path().join(path);
        std::fs::create_dir_all(target.parent().expect("a parent")).expect("directories");
        std::fs::write(target, text).expect("write");
    }

    /// Read a file under the repository.
    pub fn read(&self, path: &str) -> String {
        return std::fs::read_to_string(self.repo.path().join(path)).expect("read");
    }

    /// The command with this harness's environment:
    /// own runtime directory,
    /// own home,
    /// short idle time.
    pub fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_file-enforcer"));
        command
            .args(args)
            .current_dir(self.repo.path())
            .env("XDG_RUNTIME_DIR", self.runtime.path())
            .env("HOME", self.home.path())
            .env("FILE_ENFORCER_IDLE_SECONDS", "2")
            .stdin(Stdio::null());
        return command;
    }

    /// Run a command to completion;
    /// return the exit code and the JSON lines it printed.
    pub fn run(&self, args: &[&str]) -> (i32, Vec<Value>) {
        let output = self.command(args).output().expect("the binary runs");
        let lines = parse_lines(&String::from_utf8_lossy(&output.stdout));
        return (output.status.code().unwrap_or(-1), lines);
    }

    /// The pids of `file-enforcer serve` processes started with this harness's runtime directory.
    pub fn server_pids(&self) -> Vec<i32> {
        let marker = self.runtime.path().display().to_string();
        let home_marker = self.home.path().display().to_string();
        let mut pids: Vec<i32> = Vec::new();
        let Ok(entries) = std::fs::read_dir("/proc") else { return pids };
        for entry in entries.flatten() {
            let Ok(pid) = entry.file_name().to_string_lossy().parse::<i32>() else { continue };
            let Ok(raw) = std::fs::read(entry.path().join("cmdline")) else { continue };
            let text = String::from_utf8_lossy(&raw).replace('\0', " ");
            if text.contains(" serve ") && (text.contains(&marker) || text.contains(&home_marker)) {
                pids.push(pid);
            }
        }
        return pids;
    }

    /// Wait up to `seconds` for every server of this harness to exit;
    /// return whether they did.
    pub fn servers_gone_within(&self, seconds: u64) -> bool {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
        while std::time::Instant::now() < deadline {
            if self.server_pids().is_empty() {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        return self.server_pids().is_empty();
    }
}

impl Drop for Harness {
    /// Stop any server this test left,
    /// so none outlives the test.
    fn drop(&mut self) {
        for pid in self.server_pids() {
            // SAFETY: `kill` with a positive pid and SIGTERM has no memory-safety preconditions.
            unsafe {
                libc::kill(pid, libc::SIGTERM);
            }
        }
    }
}

/// Parse stdout into JSON values,
/// one per non-empty line.
pub fn parse_lines(text: &str) -> Vec<Value> {
    let mut values: Vec<Value> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        values.push(serde_json::from_str(line).unwrap_or_else(|error| panic!("not a JSON line: {line:?}: {error}")));
    }
    return values;
}

/// Count lines with a given `code`.
pub fn count_code(lines: &[Value], code: &str) -> usize {
    return lines.iter().filter(|line| return line["code"] == code).count();
}

/// The standard minimal config:
/// one concat rule from AGENTS.md to CLAUDE.md.
pub const CONFIG: &str = r#"{"rules": [{"kind": "concat", "id": "claude", "output": "CLAUDE.md", "parts": ["head\n", {"file": "AGENTS.md"}]}]}"#;

/// Path of the repo config inside a harness.
pub fn config_path(harness: &Harness) -> PathBuf {
    return harness.repo.path().join("file-enforcer.config.jsonc");
}

/// What:
///  Whether a process is alive and not just waiting to be reaped.
///  `i32` is a signed 32-bit integer (the type `libc::kill` takes for a process id).
/// Why:
///  A server killed after its parent exited can sit as a zombie for a moment;
/// a zombie holds no socket and serves nothing,
/// so it counts as stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function processIsRunning(pid: number): boolean
/// ```
pub fn process_is_running(pid: i32) -> bool {
    let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else { return false };
    // The state letter follows the parenthesised command name, which may itself contain spaces and parentheses.
    let after_name = stat.rsplit_once(')').map(|pair| return pair.1).unwrap_or_default();
    return after_name.trim_start().chars().next().is_some_and(|state| return state != 'Z');
}
