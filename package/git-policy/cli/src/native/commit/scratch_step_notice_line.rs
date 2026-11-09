//! What:
//!  The line a commit prints while it waits for the `git commit` of its scratch repository:
//!  what the step is,
//!  whether it signs the commit,
//!  which program runs below it,
//!  how long it has waited,
//!  and when the ceiling stops it.
//! Why:
//!  Every other wait of the wrapper says what it waits for (issue 1811),
//! and this step runs with the terminal attached and no watch,
//!  because an editor may open in it.
//!  The first line adds what to do when the step looks stuck;
//! the repeats stay short,
//!  so a long wait does not bury the person's terminal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! noticeLine({ signs, waited, first, below, ceiling })
//! ```

/// The kernel's name,
/// state and processor time of one process.
use crate::process_tree::ProcessUse;
/// Time spans,
/// for the two figures the line names.
use std::time::Duration;

/// The programs below the step other than Git itself,
/// which the notice line names.
pub fn other_programs(below: &[(i32, ProcessUse)]) -> Vec<(i32, String)> {
    let mut others: Vec<(i32, String)> = Vec::new();
    for (pid, used) in below {
        if used.name != "git" {
            others.push((*pid, used.name.clone()));
        }
    }
    return others;
}

/// What:
///  One notice line.
/// Why:
///   The first one adds what to do when the step looks stuck;
/// the repeats stay short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noticeLine({ signs, waited, first, below, canStop })
/// ```
pub fn notice_line(signs: bool, waited: Duration, first: bool, below: &[(i32, String)], ceiling: Duration, can_stop: bool) -> String {
    let signing: String = if signs { String::from(", which signs the commit because `commit.gpgsign` or `-S` asks for it") } else { String::new() };
    let signer: String = match below.first() {
        Some((pid, name)) => format!("; `{name}` (process {pid}) runs below it"),
        None => String::new(),
    };
    let hint: &str = if first { " Hint: it is still running; if it looks stuck, check it with `ps` and report it." } else { "" };
    // A step whose start gave no descriptor cannot be named again, so the ceiling cannot end it, and the line says so
    // rather than promising a stop (findings 1 and 4 of the third review).
    let ending: String = if can_stop {
        format!("it is stopped if it lasts {} seconds in all.", ceiling.as_secs())
    } else {
        String::from("it cannot be stopped: no descriptor was made for it when it started, so this commit waits until it ends.")
    };
    return format!(
        "monochromatic-git-policy-cli: still waiting for the `git commit` this commit runs in its scratch repository{signing}{signer}, {} seconds so far; \
         {ending}{hint}\n",
        waited.as_secs()
    );
}

/// The line and its controls stay out of the release executable.
#[cfg(test)]
#[path = "scratch_step_notice_line_tests.rs"]
mod tests;
