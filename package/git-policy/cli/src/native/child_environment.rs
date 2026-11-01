//! What:
//!  The environment variables cli-git adds for every real Git it starts.
//! Why:
//!  Native Git must write lock-owner PID files,
//!  and a wrapper must be able to
//!      notice that another wrapper selected it as "real Git".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // spawn(gitPath, args, { env: { ...process.env, ...childEnvironmentOverlay(process.env, gitPath) } });
//! ```

/// What:
///  `OsStr` is borrowed operating-system text;
///  `OsString` is its owned form.
///       They hold raw OS bytes (siblings `&str`/`String` must be valid UTF-8).
/// Why:
///   Environment names and values need not be UTF-8 and are passed through unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Node exposes process.env as strings; there is no byte-preserving equivalent.
/// ```
use std::ffi::{OsStr, OsString};
/// `Path` is a borrowed filesystem path made of the same raw OS bytes.
use std::path::Path;

/// Git's count variable for environment-supplied configuration.
pub const COUNT_VARIABLE: &str = "GIT_CONFIG_COUNT";

/// What:
///  The variable naming the executable this wrapper selected as real Git.
/// Why:
///   If the selected executable is itself a cli-git wrapper (for example another
///       build that file identity cannot recognise),
///  it finds its own path here and
///       stops instead of forwarding back and forth forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FORWARD_TARGET_VARIABLE = 'CLI_GIT_NATIVE_FORWARD_TARGET';
/// ```
pub const FORWARD_TARGET_VARIABLE: &str = "CLI_GIT_NATIVE_FORWARD_TARGET";

/// Key spelling written into the environment;
///  Git compares it case-insensitively.
const LOCKFILE_PID_KEY: &str = "core.lockfilePid";

/// What:
///  Largest count Git accepts (`INT_MAX`),
///  as an unsigned 64-bit integer.
///       Siblings:
///  `u32`,
///  `usize`,
///  `i64`.
/// Why:
///   `u64` holds every value C's `strtoul` can return on a 64-bit platform,
///  so
///       the comparison Git makes can be repeated without wrapping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GIT_MAX_CONFIG_COUNT = 2_147_483_647;
/// ```
const GIT_MAX_CONFIG_COUNT: u64 = 2_147_483_647;

/// What:
///  Look one variable up the way C's `getenv` does:
///  the first entry wins.
///       `&[(OsString, OsString)]` borrows a list of name/value pairs.
///       `Option<OsString>` is "an owned value or nothing".
/// Why:
///   The overlay must read exactly what the Git child will read.
///  Returning an
///       owned copy keeps the function free of borrow bookkeeping for the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function environmentValue(environment: [string, string][], name: string): string | undefined;
/// ```
pub fn environment_value(environment: &[(OsString, OsString)], name: &str) -> Option<OsString> {
    // What: `for (key, value) in environment` destructures each borrowed pair.
    // Why:  The list is small and scanned in its original order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const [key, value] of environment) { ... }
    // ```
    for (key, value) in environment {
        // `key == name` compares OS text with UTF-8 text without converting either.
        if key == name {
            // What: `Some(value.clone())` wraps an owned copy in the "present" variant.
            // Why:  The caller may keep the value after the environment list is gone.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return value;
            // ```
            return Some(value.clone());
        }
    }
    // `None` is the "absent" variant: the variable is not set.
    return None;
}

/// What:
///  Parse `GIT_CONFIG_COUNT` exactly as Git 2.56.0 `config.c` does with
///       `strtoul(env, &endp, 10)` followed by its `*endp` and `INT_MAX` checks.
///       `Option<u64>` is `Some(count)` when Git accepts the value,
///  `None` when Git
///       would report "bogus count" or "too many entries".
/// Why:
///   The overlay must append after the entries Git will really read,
///  and must
///       leave a value Git rejects untouched so Git reports the caller's own mistake.
///       `strtoul` skips leading whitespace and accepts one sign;
///  a negated nonzero
///       value wraps above `INT_MAX`,
///  so only `-0` survives with a minus sign.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseConfigCount(value: string): number | undefined;
/// ```
pub fn parse_config_count(value: &OsStr) -> Option<u64> {
    // What: `&[u8]` borrows the value's raw bytes (`u8` is one byte, 0 to 255).
    // Why:  Only ASCII whitespace, signs and digits are meaningful; anything else rejects.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const bytes = Buffer.from(value);
    // ```
    let bytes: &[u8] = value.as_encoded_bytes();
    // An empty string converts nothing and ends at once: Git reads it as zero entries.
    if bytes.is_empty() {
        return Some(0);
    }
    // `usize` is the index type of every Rust list; `mut` allows advancing it.
    let mut index: usize = 0;
    // C `isspace`: space, tab, line feed, vertical tab, form feed, carriage return.
    while index < bytes.len() && [b' ', b'\t', b'\n', 0x0b, 0x0c, b'\r'].contains(&bytes[index]) {
        index += 1;
    }
    let mut negative: bool = false;
    if index < bytes.len() && (bytes[index] == b'+' || bytes[index] == b'-') {
        negative = bytes[index] == b'-';
        index += 1;
    }
    let first_digit: usize = index;
    let mut count: u64 = 0;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        // What: `checked_mul`/`checked_add` return `None` on overflow; a trailing `?`
        //       returns that `None` to our caller, or unwraps the number.
        // Why:  A value too large for `strtoul` saturates far above `INT_MAX`, which
        //       Git rejects; overflow therefore means "rejected".
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // count = count * 10n + BigInt(digit); // BigInt cannot overflow
        // ```
        count = count
            .checked_mul(10)?
            .checked_add(u64::from(bytes[index] - b'0'))?;
        index += 1;
    }
    // No digit converted, or bytes remain after the digits: Git's `*endp` check fails.
    if index == first_digit || index != bytes.len() {
        return None;
    }
    if negative && count != 0 {
        return None;
    }
    if count > GIT_MAX_CONFIG_COUNT {
        return None;
    }
    return Some(count);
}

/// What:
///  Report whether a configuration value is one of the true spellings cli-git
///       itself writes or recognises (`true`,
///  `yes`,
///  `on`,
///  `1`,
///  any ASCII case).
/// Why:
///   When the last numbered `core.lockfilePid` entry already reads as true,
///  a
///       nested cli-git must not grow the list again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isTrueSpelling(value: string): boolean;
/// ```
fn is_true_spelling(value: &OsStr) -> bool {
    let bytes: &[u8] = value.as_encoded_bytes();
    // `b"true"` is a byte-string literal; `.as_slice()` makes the array a borrowed list.
    for spelling in [b"true".as_slice(), b"yes", b"on", b"1"] {
        if bytes.eq_ignore_ascii_case(spelling) {
            return true;
        }
    }
    return false;
}

/// What:
///  Compute the variables that append `core.lockfilePid=true` to an environment's
///       numbered Git configuration.
///  `Vec<(OsString, OsString)>` is an owned list of
///       name/value pairs;
///  it is empty when nothing must change.
/// Why:
///   Native Git then writes an owner PID file beside each lock it takes,
///  which
///       lock-ownership checks rely on.
///  Numbered entries are read before
///       `GIT_CONFIG_PARAMETERS`,
///  so a caller's explicit `-c core.lockfilePid=false`
///       still wins.
///  A count Git would reject,
///  or a numbered entry Git would report
///       as missing,
///  is left untouched so Git reports the caller's own mistake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockfilePidOverlay(environment: [string, string][]): [string, string][];
/// ```
pub fn lockfile_pid_overlay(environment: &[(OsString, OsString)]) -> Vec<(OsString, OsString)> {
    // What: `match` picks by variant: an unset variable is zero entries; a set one is parsed.
    // Why:  Only a count Git accepts tells us where the next free index is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const count = raw === undefined ? 0 : parseConfigCount(raw);
    // ```
    let count: u64 = match environment_value(environment, COUNT_VARIABLE) {
        None => 0,
        Some(raw) => {
            // `let Some(x) = ... else { return ... };` unwraps the accepted count or exits.
            let Some(parsed) = parse_config_count(raw.as_os_str()) else {
                // `Vec::new()` is the empty owned list: nothing to add.
                return Vec::<(OsString, OsString)>::new();
            };
            parsed
        }
    };
    // The value of the last numbered `core.lockfilePid` entry, which is the one Git applies.
    let mut effective: Option<OsString> = None;
    let mut index: u64 = 0;
    while index < count {
        let key: Option<OsString> =
            environment_value(environment, format!("GIT_CONFIG_KEY_{index}").as_str());
        let value: Option<OsString> =
            environment_value(environment, format!("GIT_CONFIG_VALUE_{index}").as_str());
        // A missing numbered entry makes Git fail; stopping here also bounds this loop
        // by the real environment size instead of by an arbitrary declared count.
        let (Some(present_key), Some(present_value)) = (key, value) else {
            return Vec::<(OsString, OsString)>::new();
        };
        if present_key
            .as_encoded_bytes()
            .eq_ignore_ascii_case(b"core.lockfilepid")
        {
            effective = Some(present_value);
        }
        index += 1;
    }
    // `if let Some(value) = &effective && ...` borrows the stored value when one exists
    // and then also requires the second condition, like `a !== undefined && check(a)`.
    if let Some(value) = &effective
        && is_true_spelling(value.as_os_str())
    {
        return Vec::<(OsString, OsString)>::new();
    }
    // `vec![...]` builds an owned list; `OsString::from` copies text into OS form.
    return vec![
        (
            OsString::from(COUNT_VARIABLE),
            OsString::from((count + 1).to_string()),
        ),
        (
            OsString::from(format!("GIT_CONFIG_KEY_{count}")),
            OsString::from(LOCKFILE_PID_KEY),
        ),
        (
            OsString::from(format!("GIT_CONFIG_VALUE_{count}")),
            OsString::from("true"),
        ),
    ];
}

/// What:
///  Compute every variable cli-git sets for a real Git child:
///  the lock PID
///       injection plus the forward-target marker naming `real_git`.
/// Why:
///   Every Git the wrapper starts,
///  forwarded or queried,
///  gets the same additions;
///       the rest of the caller's environment is inherited unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function childEnvironmentOverlay(environment: [string, string][], realGit: string): [string, string][];
/// ```
pub fn child_environment_overlay(
    environment: &[(OsString, OsString)],
    real_git: &Path,
) -> Vec<(OsString, OsString)> {
    let mut overlay: Vec<(OsString, OsString)> = lockfile_pid_overlay(environment);
    // `.as_os_str().to_os_string()` copies the path's raw bytes without any decoding.
    overlay.push((
        OsString::from(FORWARD_TARGET_VARIABLE),
        real_git.as_os_str().to_os_string(),
    ));
    return overlay;
}

/// What:
///  The complete environment of a Git child that is started with a cleared environment:
///  the caller's variables, with every name the overlay sets replaced by the overlay's value.
/// Why:
///   The incumbent's push coordination, push and manual-push probe inherit the whole process environment
///  (`HOME`, `PATH`, `SSH_AUTH_SOCK`, the user's `GIT_*` variables) plus the overlay,
///  and a credential helper or transport needs them;
///  the overlay alone would start Git with an almost empty environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ ...process.env, ...Object.fromEntries(overlay) })
/// ```
pub fn environment_with_overlay(
    environment: &[(OsString, OsString)],
    overlay: &[(OsString, OsString)],
) -> Vec<(OsString, OsString)> {
    // `mut` lets the overlay entries be appended after the surviving caller entries.
    let mut result: Vec<(OsString, OsString)> = Vec::with_capacity(environment.len() + overlay.len());
    for (name, value) in environment {
        let replaced: bool = overlay.iter().any(|(overlaid, _value)| {
            return overlaid == name;
        });
        if !replaced {
            result.push((name.clone(), value.clone()));
        }
    }
    result.extend(overlay.iter().cloned());
    return result;
}

/// Lookup and count-parsing controls,
///  checked against real Git.
#[cfg(test)]
#[path = "child_environment_count_tests.rs"]
mod count_tests;

/// Overlay controls and the shared environment builder stay out of the release executable.
#[cfg(test)]
#[path = "child_environment_tests.rs"]
mod tests;
