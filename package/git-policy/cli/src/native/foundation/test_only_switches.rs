//! What:
//!  The one read every test-only switch of this crate goes through:
//!  the value a `CLI_GIT_TEST_ONLY_*` variable holds,
//!  taken from this process's environment or from an environment a caller already holds.
//! Why:
//!  A variable that arms a fault can kill or stall the commit service,
//!  and a unit's `Environment=` line can set one,
//!  so a build the release ships reads none of them:
//!  without the `test-only-switches` feature the reads below answer nothing at all,
//!  and only a build made for a control turns the feature on
//!  (`doc/decision/cli-git-test-only-switches-compiled-out-of-release.md`,
//!  as `start_probe/probe_deadline.rs` gates the probe's raised deadline).
//!  The feature decides what is compiled,
//!  never a check after the read,
//!  so a release build holds no read to gate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function text(variable: string): string | undefined // undefined in a release build
//! ```

use std::ffi::OsString;

/// What:
///  The raw value switch `variable` holds in this process's environment,
///  or nothing.
/// Why:
///  Most switches name a path,
///  which need not be text,
///  so the raw value comes first and the text form is built from it;
///  a build made without the feature answers nothing here,
///  which is every release build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function value(variable: string): OsString | undefined { return process.env[variable]; }
/// ```
#[cfg(feature = "test-only-switches")]
pub fn value(variable: &str) -> Option<OsString> {
    return std::env::var_os(variable);
}

/// What:
///  The same read in a build made without the feature:
///  no test-only switch is read,
///  so this answers nothing.
/// Why:
///  The release must hold no read of these variables,
///  and a function that is not compiled is a read that does not exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function value(variable: string): undefined { return undefined; }
/// ```
#[cfg(not(feature = "test-only-switches"))]
pub fn value(_variable: &str) -> Option<OsString> {
    return None;
}

/// What:
///  The text switch `variable` holds,
///  or nothing.
/// Why:
///  A switch that names a step,
///  a mode or a number is written as text,
///  and a value that is not text is no switch at all,
///  which is what a build without the feature always answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function text(variable: string): string | undefined { return value(variable)?.toStringIfText(); }
/// ```
pub fn text(variable: &str) -> Option<String> {
    // `Option<OsString>` is the raw value above: either the bytes the variable holds or nothing.
    let raw: Option<OsString> = value(variable);
    // `let Some(raw) = raw else { ... }` takes the value out when there is one and runs the block when there is none;
    // the block returns, so `raw` is the value from here on.
    let Some(raw) = raw else {
        return None;
    };
    // `.ok()` turns the `Result<String, _>` of "these bytes are text" into an `Option<String>`, dropping a value that
    // is not text rather than failing the run.
    return raw.into_string().ok();
}

/// What:
///  The value switch `variable` holds in `environment`,
///  which is the environment a command was started with,
///  or nothing.
/// Why:
///  Two switches are read from the environment a caller already holds rather than from this process's own,
///  so a control's value arrives the way a person's does;
///  a build without the feature answers nothing here too,
///  reading not even the list it is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function valueIn(environment: [string, string][], variable: string): OsString | undefined
/// ```
#[cfg(feature = "test-only-switches")]
pub fn value_in(environment: &[(OsString, OsString)], variable: &str) -> Option<OsString> {
    // The last value the environment holds under this name, as `seconds_variable` also takes it: an environment may
    // carry one name twice, and the later entry is the one the command read last.
    let mut found: Option<OsString> = None;
    for (name, held) in environment {
        if name.to_str() == Some(variable) {
            // `.clone()` copies the value, so the answer owns it and the caller's environment stays as it is.
            found = Some(held.clone());
        }
    }
    return found;
}

/// What:
///  The same read of a caller's environment in a build made without the feature,
///  which answers nothing and reads no entry.
/// Why:
///  A release build must hold no path by which a test-only value reaches the code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function valueIn(environment: [string, string][], variable: string): undefined { return undefined; }
/// ```
#[cfg(not(feature = "test-only-switches"))]
pub fn value_in(_environment: &[(OsString, OsString)], _variable: &str) -> Option<OsString> {
    return None;
}
