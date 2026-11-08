//! What:
//! The free-space figures and the summary lines of a lean-folder run.
//! Why:
//! The run reports the space it freed per file system,
//! because the 860 drive also holds the repository move's space margin
//! (`package/repository-move/cli`:
//! 150 GiB must stay free).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function freeBytes(path: string): number
//! ```

/// Paths.
use std::path::Path;

/// File system statistics.
use rustix::fs::statvfs;

/// What:
/// `pub fn free_bytes(path: &Path) -> Option<u64>` is the space an unprivileged user can still write on the file system of a path.
/// Why:
/// The report compares it before and after the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const freeBytes = (path: string): number => statfsSync(path).bavail * statfsSync(path).bsize;
/// ```
pub fn free_bytes(path: &Path) -> Option<u64> {
    let stats = statvfs(path).ok()?;
    return Some(stats.f_bavail.saturating_mul(stats.f_frsize));
}

/// What:
/// `pub fn gib(bytes: u64) -> String` writes bytes as gibibytes with two decimals.
/// Why:
/// Reports compare figures in one unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const gib = (bytes: number): string => (bytes / 2 ** 30).toFixed(2);
/// ```
pub fn gib(bytes: u64) -> String {
    return format!("{:.2} GiB", bytes as f64 / (1_u64 << 30) as f64);
}

/// A named place whose file system is measured.
#[derive(Clone, Debug)]
pub struct Place {
    /// What to call it in the report.
    pub label: String,
    /// A path on its file system.
    pub path: std::path::PathBuf,
    /// Free bytes before the run.
    pub before: Option<u64>,
}

/// What:
/// `pub fn measure_places(places: &[(String, std::path::PathBuf)]) -> Vec<Place>` measures the free space of each place now.
/// Why:
/// The first reading is the "before".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function measurePlaces(places: [string, string][]): Place[]
/// ```
pub fn measure_places(places: &[(String, std::path::PathBuf)]) -> Vec<Place> {
    let mut measured: Vec<Place> = Vec::new();
    for (label, path) in places {
        measured.push(Place { label: label.clone(), path: path.clone(), before: free_bytes(path) });
    }
    return measured;
}

/// What:
/// `pub fn space_lines(places: &[Place]) -> Vec<String>` writes the before and after free space of each place.
/// Why:
/// The after reading is taken here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function spaceLines(places: Place[]): string[]
/// ```
pub fn space_lines(places: &[Place]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for place in places {
        let after: Option<u64> = free_bytes(&place.path);
        let text: String = match (place.before, after) {
            (Some(before), Some(now)) => format!(
                "free space on {} ({}): {} before, {} after, {}{} changed",
                place.label,
                place.path.display(),
                gib(before),
                gib(now),
                if now >= before { "+" } else { "-" },
                gib(now.abs_diff(before))
            ),
            _ => format!("free space on {} ({}): not readable", place.label, place.path.display()),
        };
        lines.push(text);
    }
    return lines;
}

/// What:
/// `pub fn log_line(text: &str)` writes one line to standard error and ignores a write that fails.
/// Why:
/// A closed pipe (`| head`) must not turn a finished removal into a panic in the middle of a run:
/// `eprintln!` panics when standard error is closed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function logLine(text: string): void { try { process.stderr.write(`${text}\n`); } catch {} }
/// ```
pub fn log_line(text: &str) {
    /// Writes lines.
    use std::io::Write;
    let _ = writeln!(std::io::stderr(), "cargo-shadow lean: {text}");
}

/// What:
/// `pub fn summary_line(text: &str)` writes one line to standard output and ignores a write that fails.
/// Why:
/// The same reason as `log_line`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function summaryLine(text: string): void
/// ```
pub fn summary_line(text: &str) {
    /// Writes lines.
    use std::io::Write;
    let _ = writeln!(std::io::stdout(), "{text}");
}

/// What:
/// `pub fn emit(log: &mut Vec<String>, line: String)` writes a removal's log line to standard error at once and keeps it.
/// Why:
/// The proof of "not in use" must reach the log when the removal happens,
/// not when a pass of hours ends,
/// so a run that is stopped has still said what it did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function emit(log: string[], line: string): void { logLine(line); log.push(line); }
/// ```
pub fn emit(log: &mut Vec<String>, line: String) {
    log_line(&line);
    log.push(line);
}
