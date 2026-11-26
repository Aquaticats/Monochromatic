//! What:
//!  The Git arguments of the manual-push probe's dry-run push,
//!  and the destination locations it confirms.
//! Why:
//!   The probe's own file holds the discovery,
//!  the confirmation and the answers it carries to the Git LFS upload plan,
//! and the arguments
//!  and locations sit beside them because both together are more than the package's length limit allows
//!  one file (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! probeArgs({ globalPrefix, args }); await git(['remote', 'get-url', '--push', '--all', remote]);
//! ```

/// The probe's Git runner and its bounded queries.
use crate::auto_push_single_flight::{
    PushGit,
    run_query,
};
/// The push's own reading of its destination.
use crate::command_options::{
    Boundary,
    DEFAULT_MODE,
    parse_options,
};
use crate::command_push::PUSH_TABLE;
/// Owned OS strings for the probe's Git arguments.
use std::ffi::OsString;

/// What:
///  The probe's Git arguments:
/// the invocation's global options,
/// the switches that keep every hook and full object
///  names,
/// then `push` with `--dry-run --porcelain` placed before any `--` or `--end-of-options` terminator.
/// Why:
///   The `--` terminator is the user's;
/// the options Git interprets must stay before it.
/// The probe publishes nothing and
///  must not run the upload hook of the caller's configuration,
/// so configured `pre-push` hooks are off and the
///  traditional hooks directory is one that never exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// probeArgs({ globalPrefix, args });
/// ```
pub fn probe_arguments(
    global_prefix: &[OsString],
    post_subcommand: &[OsString],
) -> Vec<OsString> {
    // The separator is the one Git reads: a `--` that is the value of an option such as `--receive-pack` is not it. A region
    //  Git refuses has no reading, and the probe then stops at the first `--`, where Git refuses it as well.
    let separator = match parse_options(post_subcommand, PUSH_TABLE, DEFAULT_MODE, &[]) {
        Ok(parsed) => match parsed.boundary {
            Boundary::DashDash(at) | Boundary::EndOfOptions(at) => Some(at),
            Boundary::End | Boundary::NonOption(_) => None,
        },
        Err(_) => {
            let mut first: Option<usize> = None;
            for (position, argument) in post_subcommand.iter().enumerate() {
                if argument.as_os_str() == "--" {
                    first = Some(position);
                    break;
                }
            }
            first
        }
    };
    let insertion = separator.unwrap_or(post_subcommand.len());
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    for setting in [
        format!("core.hooksPath={}", crate::hooks_off::NO_HOOKS_DIRECTORY),
        String::from("hook.pre-push.enabled=false"),
        String::from("core.abbrev=no"),
    ] {
        arguments.push(OsString::from("-c"));
        arguments.push(OsString::from(setting));
    }
    arguments.push(OsString::from("push"));
    arguments.extend(post_subcommand[..insertion].iter().cloned());
    // A `-q` written by the caller silences the report this probe reads, so it is cancelled here; the caller's own
    //  push keeps its `-q`.
    arguments.push(OsString::from("--no-quiet"));
    arguments.push(OsString::from("--dry-run"));
    arguments.push(OsString::from("--porcelain"));
    arguments.extend(post_subcommand[insertion..].iter().cloned());
    return arguments;
}

/// Whether text typed as the repository is an URL,
/// an scp-style address or a path,
/// not a remote name.
pub(crate) fn looks_like_location(typed: &str) -> bool {
    if typed.contains("://")
        || typed.starts_with('/')
        || typed.starts_with("./")
        || typed.starts_with("../")
    {
        return true;
    }
    return match typed.split_once(':') {
        Some((front, _)) => !front.contains('/'),
        None => false,
    };
}

/// One Git query of the probe:
/// its trimmed standard output when it exits with status zero.
pub(crate) fn answer(
    git: &PushGit,
    arguments: &[OsString],
) -> Option<String> {
    let output = run_query(git, arguments).ok()?;
    if output.code != Some(0) {
        return None;
    }
    return Some(
        String::from_utf8_lossy(output.stdout.as_slice())
            .trim()
            .to_owned(),
    );
}

/// The destination locations of a remote name,
/// one per `To` block Git printed:
/// the push URLs of a configured remote,
///  or the name itself when it is not one (a URL or a path typed in place of a remote).
pub(crate) fn locations_of(
    git: &PushGit,
    global_prefix: &[OsString],
    remote: &str,
) -> Vec<String> {
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    arguments.extend(["remote", "get-url", "--push", "--all", remote].map(OsString::from));
    return match answer(git, arguments.as_slice()) {
        Some(listing) if !listing.is_empty() => listing.lines().map(str::to_owned).collect(),
        Some(_) | None => vec![remote.to_owned()],
    };
}
