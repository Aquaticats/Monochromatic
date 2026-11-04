//! What:
//!  Which processes of the table carry a given process number inside a PID namespace nested in the reader's.
//! Why:
//!  A pid file holds the number its writer had in the writer's own PID namespace; Git records no namespace. A reader in
//!  the initial namespace sees every process of the machine, but a writer in a nested namespace is listed under a number
//!  of the reader's namespace, and the number in the pid file is a different one. "No process has this number" therefore
//!  proves the writer gone only when no process of the table also has the number as its number inside a nested
//!  namespace. The kernel lists every number a process has, outermost first, on the `NSpid` line of
//!  `/proc/<pid>/status` (`proc_pid_status(5)`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const holders = nestedHoldersOf('/proc', 4242); // [] means no nested namespace has a process numbered 4242
//! ```

use std::path::Path;

/// The name of the line of `status` that lists a process's numbers, one per PID namespace, outermost first.
const NAMESPACE_PID_LINE: &str = "NSpid:";

/// The numbers a process has in the namespaces it is in, outermost first, from the text of its `status` file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function numbersOf(status: string): number[] | undefined;
/// ```
fn numbers_of(status: &str) -> Option<Vec<i64>> {
    let line: &str = status.lines().find_map(|line| { return line.strip_prefix(NAMESPACE_PID_LINE); })?;
    let mut numbers: Vec<i64> = Vec::new();
    for field in line.split_whitespace() {
        numbers.push(field.parse::<i64>().ok()?);
    }
    return Some(numbers);
}

/// What:
///  The processes of the table that have `number` as their number inside a namespace nested in the reader's.
/// Why:
///   Such a process may be the writer of a pid file that names `number`, whatever the reader's own table says about
///  that number. An error means the table could not be read completely, so the question is open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nestedHoldersOf(procRoot: string, number: number): number[]; // throws when the table is unreadable
/// ```
pub fn nested_holders_of(proc_root: &Path, number: i64) -> Result<Vec<i64>, String> {
    let listing: std::fs::ReadDir = std::fs::read_dir(proc_root).map_err(|error| { return format!("{}: {error}", proc_root.display()); })?;
    let mut holders: Vec<i64> = Vec::new();
    for item in listing.flatten() {
        let Some(outer) = item.file_name().to_str().and_then(|name| { return name.parse::<i64>().ok(); }) else {
            continue;
        };
        let status: String = match std::fs::read_to_string(item.path().join("status")) {
            Ok(text) => text,
            // A process that ended between the listing and the read cannot be the writer of a lock that still exists.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH) => continue,
            Err(error) => return Err(format!("the status of process {outer} could not be read: {error}")),
        };
        let Some(numbers) = numbers_of(status.as_str()) else {
            return Err(format!("the status of process {outer} lists no `{NAMESPACE_PID_LINE}` line"));
        };
        // The first number is the reader's own; any later one is the process's number inside a nested namespace.
        if numbers.iter().skip(1).any(|nested| { return *nested == number; }) {
            holders.push(outer);
        }
    }
    holders.sort_unstable();
    return Ok(holders);
}

/// Controls for the reading of namespace numbers are in the test module.
#[cfg(test)]
#[path = "stale_ref_lock_nested_numbers_tests.rs"]
mod tests;
