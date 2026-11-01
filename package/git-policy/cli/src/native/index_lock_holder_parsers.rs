//! What:
//!  Parsers of the holder-scan output that is not `/proc`:
//!  `lsof -F pcDi` records and the
//!       Restart Manager script lines,
//!  both matched to a lock by device and inode.
//! Why:
//!  The parsers are pure text functions,
//!  so their controls run on every host,
//!  whichever scan
//!      produced the text (`src/index-lock/index-lock-holders-darwin.ts`,
//!  `-win32.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! parseLsofFields(output); parseRestartManagerOutput(output);
//! ```

/// The holder record type the parsers produce.
use super::index_lock_holders::Holder;

/// What:
///  One `lsof -F pcDi` process record:
///  its PID,
///  command and the device and inode of every
///       listed file (each field present when parsable).
/// Why:
///   `LsofProcess`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LsofProcess = { pid: number; command?: string; files: { device?: bigint; inode?: bigint }[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LsofProcess {
    /// Process ID as `Number` reads it;
    ///  `None` when it is not a number.
    pub pid: Option<i64>,
    /// Command name.
    pub command: Option<String>,
    /// Device and inode of each listed file.
    pub files: Vec<(Option<u128>, Option<u128>)>,
}

/// What:
///  An `lsof` number as `BigInt` reads it:
///  decimal,
///  or `0x`,
///  `0o` or `0b` prefixed.
/// Why:
///   Device numbers come `0x`-prefixed,
///  inodes decimal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// BigInt(text)
/// ```
pub fn parse_lsof_number(text: &str) -> Option<u128> {
    let trimmed: &str = super::js_text::trim_javascript(text);
    if trimmed.is_empty() {
        return Some(0);
    }
    let (digits, radix) = match trimmed.get(..2) {
        Some("0x" | "0X") => (&trimmed[2..], 16),
        Some("0o" | "0O") => (&trimmed[2..], 8),
        Some("0b" | "0B") => (&trimmed[2..], 2),
        _ => (trimmed, 10),
    };
    if digits.is_empty() || digits.starts_with(['+', '-']) {
        return None;
    }
    return u128::from_str_radix(digits, radix).ok();
}

/// What:
///  The first field with this identifier among lines,
///  parsed as a number.
/// Why:
///   `numberField`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// numberField({ lines, identifier })
/// ```
fn number_field(lines: &[&str], identifier: char) -> Option<u128> {
    for line in lines {
        if let Some(rest) = line.strip_prefix(identifier) {
            return parse_lsof_number(rest);
        }
    }
    return None;
}

/// What:
///  Lines grouped under each line starting with `identifier`;
///  earlier lines are dropped.
/// Why:
///   `groupsStartingAt`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// groupsStartingAt({ lines, identifier })
/// ```
fn groups<'a>(lines: &[&'a str], identifier: char) -> Vec<Vec<&'a str>> {
    let mut grouped: Vec<Vec<&'a str>> = Vec::new();
    for line in lines {
        if line.starts_with(identifier) {
            grouped.push(vec![*line]);
        } else if let Some(last) = grouped.last_mut() {
            last.push(*line);
        }
    }
    return grouped;
}

/// What:
///  Parse `lsof -F pcDi` output into process records.
/// Why:
///   `parseLsofFields`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseLsofFields('p42\ncgit\nf3\nD0x1000011\ni77\n')
/// ```
pub fn parse_lsof_fields(output: &str) -> Vec<LsofProcess> {
    let lines: Vec<&str> = output.split('\n').collect();
    let mut processes: Vec<LsofProcess> = Vec::new();
    for group in groups(lines.as_slice(), 'p') {
        let mut command: Option<String> = None;
        for line in &group {
            if let Some(rest) = line.strip_prefix('c') {
                command = Some(String::from(rest));
                break;
            }
        }
        let mut files: Vec<(Option<u128>, Option<u128>)> = Vec::new();
        for file in groups(group.as_slice(), 'f') {
            files.push((number_field(file.as_slice(), 'D'), number_field(file.as_slice(), 'i')));
        }
        processes.push(LsofProcess {
            pid: super::recovery_evidence::decimal_attempt(&group[0][1..]),
            command,
            files,
        });
    }
    return processes;
}

/// What:
///  Processes whose listed file matches the lock's device and inode.
/// Why:
///   `matchingHolders`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// matchingHolders({ processes, device, inode })
/// ```
pub fn matching_holders(processes: &[LsofProcess], device: u64, inode: u64) -> Vec<Holder> {
    let mut holders: Vec<Holder> = Vec::new();
    for process in processes {
        let Some(pid) = process.pid else {
            continue;
        };
        let wanted: (Option<u128>, Option<u128>) = (Some(u128::from(device)), Some(u128::from(inode)));
        if process.files.contains(&wanted) {
            holders.push(Holder {
                pid,
                command: process.command.clone(),
            });
        }
    }
    return holders;
}

/// What:
///  Parse the Restart Manager script's `<pid>\t<application name>` lines.
/// Why:
///   `parseRestartManagerOutput`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseRestartManagerOutput('4242\tgit.exe\n')
/// ```
pub fn parse_restart_manager_output(output: &str) -> Vec<Holder> {
    let mut holders: Vec<Holder> = Vec::new();
    for raw in output.split('\n') {
        let line: &str = super::js_text::trim_javascript(raw);
        if line.is_empty() {
            continue;
        }
        let (pid_text, command) = match line.split_once('\t') {
            Some((pid, name)) => (pid, name),
            None => (line, ""),
        };
        let Some(pid) = super::recovery_evidence::decimal_attempt(super::js_text::trim_javascript(pid_text)) else {
            continue;
        };
        holders.push(Holder {
            pid,
            command: if command.is_empty() { None } else { Some(String::from(command)) },
        });
    }
    return holders;
}
