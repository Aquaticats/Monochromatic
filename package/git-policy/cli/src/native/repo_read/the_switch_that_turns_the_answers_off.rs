//! What:
//!  The environment switch that turns the in-process answers off,
//!  and the overlay entry a control writes to set it.
//! Why:
//!  The hook that tries an in-process answer reads the switch,
//!  and so does the layout reader,
//!  which declines its own answers when the switch is off:
//!  while the switch lived in the hook's file,
//!  those two files used each other in a loop (`AGENTS.md` rule `LYF`).
//!  The switch belongs to neither:
//!  it says whether the in-process readers run at all,
//!  and every file that asks is a caller of this one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const SWITCH_VARIABLE = 'CLI_GIT_IN_PROCESS_READS';
//! const enabled = (variables: [string, string][]) => /* not switched off here or for this run */;
//! ```

/// Import the owned operating-system text a variable name and value are.
use std::ffi::OsString;

/// What:
///  A name bound to a borrowed string literal:
///  `&str` is a read-only view of UTF-8 bytes baked into the binary,
///  and its sibling `String` would be an owned buffer on the heap.
/// Why:
///  The variable's name is a fixed word this release reads,
///  so nothing here needs to grow or be freed,
///  and both the reader and the controls write the same constant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SWITCH_VARIABLE = "CLI_GIT_IN_PROCESS_READS";
/// ```
pub const SWITCH_VARIABLE: &str = "CLI_GIT_IN_PROCESS_READS";

/// What:
///  Whether the answers are enabled in this process and for this run's added variables.
/// Why:
///   A person who sets the switch in their own environment,
///  and a run whose overlay entries carry it turned off,
///  both get Git started for every read,
///  which is what a measurement of both answers compares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function enabled(variables: [string, string][]): boolean
/// ```
pub fn enabled(variables: &[(OsString, OsString)]) -> bool {
    for (name, value) in variables {
        if name == SWITCH_VARIABLE && value == "off" {
            return false;
        }
    }
    return match std::env::var_os(SWITCH_VARIABLE) {
        Some(value) => value != "off",
        None => true,
    };
}

/// What:
///  The overlay entry that makes a runner start Git for every command line.
/// Why:
///   Controls that count the starts of a stand-in Git,
/// or that need an unstartable Git to fail,
/// measure the process
///  path,
/// which stays the fallback for everything the readers decline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const switchOff = [['CLI_GIT_IN_PROCESS_READS', 'off']];
/// ```
#[cfg(test)]
pub fn switch_off() -> [(OsString, OsString); 1] {
    return [(OsString::from(SWITCH_VARIABLE), OsString::from("off"))];
}
