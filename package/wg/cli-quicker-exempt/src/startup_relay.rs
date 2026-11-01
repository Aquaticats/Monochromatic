//! Relays what the new watcher wrote to its log while it started to the terminal that ran `up`.
//! The log stays the full record;
//! the relay prints the loud lines,
//! at most a bounded number,
//! and says how many more there are,
//! so no finding of the start is dropped without a word.

/// The log of one watcher.
use crate::application_watch_state::watch_log_path;
/// Escaping,
/// the non-panicking write,
/// and the marker of a notice about the log.
use crate::log_line::{sanitize, write_unchecked, LOG_NOTICE_MARKER};
/// The marker of a loud report.
use crate::update_report::POLICY_NOT_IN_FORCE_MARKER;
/// Reading the log.
use std::io;

/// Most bytes of the watcher log read to relay its startup reports.
const MAX_RELAY_BYTES: u64 = 1_048_576;

/// Reads at most `MAX_RELAY_BYTES` of the watcher log from byte `offset` on,
/// so a long-lived log is never read whole.
fn read_watch_log_since(key: &str, offset: u64) -> io::Result<String> {
    let path = watch_log_path(key)?;
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(String::new()),
        Err(error) => return Err(error),
    };
    io::Seek::seek(&mut file, io::SeekFrom::Start(offset))?;
    let mut bytes: Vec<u8> = Vec::new();
    io::Read::read_to_end(&mut io::Read::take(file, MAX_RELAY_BYTES), &mut bytes)?;
    return Ok(String::from_utf8_lossy(&bytes).into_owned());
}

/// The loud lines in the part of `log` that starts at byte `offset`,
/// which is what the new watcher wrote since the start began.
/// Earlier lines belong to earlier runs and are left out.
pub fn failure_lines_since(log: &str, offset: u64) -> Vec<String> {
    let start = usize::try_from(offset).unwrap_or(usize::MAX).min(log.len());
    let fresh = String::from_utf8_lossy(&log.as_bytes()[start..]).into_owned();
    let mut lines: Vec<String> = Vec::new();
    for line in fresh.lines() {
        if line.contains(POLICY_NOT_IN_FORCE_MARKER) || line.contains(LOG_NOTICE_MARKER) {
            lines.push(line.to_string());
        }
    }
    return lines;
}

/// Most loud lines the start prints to the terminal;
/// the rest are counted and left in the log.
const MAX_RELAYED_LINES: usize = 200;

/// The lines the start prints to the terminal for the loud lines of the log:
/// at most `MAX_RELAYED_LINES`,
/// escaped,
/// then one line saying how many more there are in the log,
/// so a start with many findings never drops one without saying so.
pub fn relayed_text(lines: &[String], log_path: &str) -> Vec<String> {
    let mut printed: Vec<String> = Vec::new();
    for line in lines.iter().take(MAX_RELAYED_LINES) {
        printed.push(sanitize(line));
    }
    if lines.len() > MAX_RELAYED_LINES {
        printed.push(format!(
            "wg-quicker-exempt: {} more loud line(s) from the start are in the watcher log {}",
            lines.len() - MAX_RELAYED_LINES,
            sanitize(log_path)
        ));
    }
    return printed;
}

/// Prints the relayed text to the terminal.
fn relay_lines(lines: &[String], log_path: &str) {
    for line in relayed_text(lines, log_path) {
        write_unchecked(&line);
    }
}

/// Prints the loud lines the new watcher wrote to its log since `offset`,
/// so a failure at startup reaches the terminal that ran `up`.
/// The log stays the full record.
pub fn forward_startup_failures(key: &str, offset: u64) -> io::Result<()> {
    let text = read_watch_log_since(key, offset)?;
    relay_lines(&failure_lines_since(&text, 0), &watch_log_path(key)?.display().to_string());
    return Ok(());
}
