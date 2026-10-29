//! What:
//!  The Git LFS upload before a push, run as a supported `pre-push` configuration hook the wrapper puts in effect
//!  for its own pushes, and the `git cli-git lfs-pre-push` entry that hook starts.
//! Why:
//!  Traditional hook files are unsupported, so the `pre-push` shim `git lfs install` writes cannot be relied on.
//!  Without an upload step a push publishes pointer files whose objects the LFS server never received.
//!  The wrapper therefore adds `hook.cli-git-lfs-pre-push.event=pre-push` and a command naming itself to every real
//!  push it starts, only when the Git configuration holds LFS keys.
//!  Git runs that hook with its own arguments (`<remote> <url>`) and ref-update lines on standard input, before it
//!  sends anything, and a non-zero status stops the push; this module adds no copy of that computation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const args = usesLfs(snapshot) ? [...lfsHookArguments(executable), ...pushArguments] : pushArguments;
//! ```

/// Whether the configuration snapshot holds LFS keys.
use super::config_snapshot::ConfigSnapshot;
/// How the `git-lfs` child ended, as a status the hook returns.
use super::forwarding::{ChildOutcome, exit_code, outcome_of};
/// The retried start of the `git-lfs` child.
use super::start_retry::RetryStart;
/// Owned operating-system text and its byte view for the shell quoting of the hook command.
use std::ffi::{OsStr, OsString};
/// Where the hook's diagnostics go.
use std::io::Write;
/// Byte access to operating-system text.
use std::os::unix::ffi::{OsStrExt, OsStringExt};
/// The executable path the hook command names.
use std::path::Path;
/// The `git-lfs` child.
use std::process::Command;

/// The friendly name of the injected hook; `hook.<name>.enabled=false` on the command line turns it off.
pub const HOOK_NAME: &str = "cli-git-lfs-pre-push";

/// The management words the injected hook command starts the wrapper with.
pub const HOOK_ENTRY: [&str; 2] = ["cli-git", "lfs-pre-push"];

/// The program the hook runs, found on `PATH` as the shim `git lfs install` wrote found it.
const LFS_PROGRAM: &str = "git-lfs";

/// Key prefixes whose presence means the repository is set up for Git LFS.
const LFS_KEY_PREFIXES: [&str; 2] = ["filter.lfs.", "lfs."];

/// What:
///  Whether any configuration entry belongs to Git LFS.
/// Why:
///   `git lfs install` writes `filter.lfs.*` and `lfs.repositoryformatversion`, and a server is named by `lfs.url`;
///  one such key, in any scope, is the sign that pushed pointers need their objects uploaded first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.some(({ key }) => key.startsWith('filter.lfs.') || key.startsWith('lfs.'));
/// ```
pub fn uses_lfs(snapshot: &ConfigSnapshot) -> bool {
    return snapshot.entries().iter().any(|entry| {
        return LFS_KEY_PREFIXES.iter().any(|prefix| {
            return entry.key.starts_with(prefix);
        });
    });
}

/// What:
///  Quote operating-system text as one word of a POSIX shell command.
/// Why:
///   Git runs a `hook.<name>.command` that holds a space through `sh -c`, so a path with spaces or quotes must stay
///  one word; single quotes keep every byte literal, and an embedded single quote is closed, escaped and reopened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `'${text.replaceAll("'", "'\\''")}'`
/// ```
pub fn shell_quote(text: &OsStr) -> OsString {
    let mut quoted: Vec<u8> = vec![b'\''];
    for byte in text.as_bytes() {
        if *byte == b'\'' {
            quoted.extend_from_slice(b"'\\''");
        } else {
            quoted.push(*byte);
        }
    }
    quoted.push(b'\'');
    return OsString::from_vec(quoted);
}

/// What:
///  The four global-option tokens that put the LFS hook in effect for one Git process.
/// Why:
///   The hook command names the running executable, so an upgraded or moved wrapper never leaves a stale path in
///  the repository configuration; nothing is written to any configuration file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['-c', `hook.${name}.event=pre-push`, '-c', `hook.${name}.command='${exe}' cli-git lfs-pre-push`]
/// ```
pub fn hook_arguments(executable: &Path) -> Vec<OsString> {
    let mut command: OsString = OsString::from(format!("hook.{HOOK_NAME}.command="));
    command.push(shell_quote(executable.as_os_str()));
    for word in HOOK_ENTRY {
        command.push(" ");
        command.push(word);
    }
    return vec![
        OsString::from("-c"),
        OsString::from(format!("hook.{HOOK_NAME}.event=pre-push")),
        OsString::from("-c"),
        command,
    ];
}

/// What:
///  Put the LFS hook in front of a Git command line.
/// Why:
///   Global options may precede the subcommand, so the hook definition goes first and a `-c` the caller wrote after
///  it (for example `-c hook.cli-git-lfs-pre-push.enabled=false`) still wins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [...hookArguments(executable), ...arguments]
/// ```
pub fn with_lfs_hook(arguments: &[OsString], executable: &Path) -> Vec<OsString> {
    let mut combined: Vec<OsString> = hook_arguments(executable);
    combined.extend(arguments.iter().cloned());
    return combined;
}

/// What:
///  The arguments after `cli-git lfs-pre-push`, when the invocation is the hook entry.
/// Why:
///   The entry is started by Git with exactly the management words and then `<remote> <url>`; anything else, such as
///  global options in front, is an ordinary invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// args[0] === 'cli-git' && args[1] === 'lfs-pre-push' ? args.slice(2) : undefined
/// ```
pub fn hook_entry_arguments(arguments: &[OsString]) -> Option<&[OsString]> {
    let [first, second, rest @ ..] = arguments else {
        return None;
    };
    if first.as_os_str() == HOOK_ENTRY[0] && second.as_os_str() == HOOK_ENTRY[1] {
        return Some(rest);
    }
    return None;
}

/// What:
///  The refusal printed when `git-lfs` cannot be started.
/// Why:
///   Without the program the objects cannot be uploaded, and pushing anyway would publish pointers the server cannot
///  serve; every valid way forward is named, including leaving LFS if the repository does not need it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: this push was stopped before any ref changed: ...`
/// ```
pub fn missing_program_message(error: &std::io::Error) -> String {
    return format!(
        "cli-git: this push was stopped before it sent anything. The Git configuration of this repository has Git LFS \
         keys (`filter.lfs.*` or `lfs.*`), so the wrapper runs `{LFS_PROGRAM} pre-push` to upload the large files the \
         pushed commits point to before it publishes the pointer files, but `{LFS_PROGRAM}` could not be started: \
         {error}. Pushing without that upload would publish pointers whose content the LFS server does not have. \
         Ways forward: install Git LFS so that `{LFS_PROGRAM}` is on PATH and push again; or, if this repository does not \
         use Git LFS, remove its configuration (`git lfs uninstall`, or delete the `filter.lfs.*` and `lfs.*` keys); \
         or accept pointers without objects for this push with `git push --no-verify` and upload the objects later with \
         `git lfs push --all <remote>` once Git LFS is installed.\n"
    );
}

/// What:
///  The explanation printed after `git-lfs pre-push` failed.
/// Why:
///   `git-lfs` prints its own error first (a refused credential, an unreachable server, a missing local object);
///  this text says what that means for the push and every way forward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: git-lfs pre-push exited with status ${code}; ...`
/// ```
pub fn failed_message(status: &str) -> String {
    return format!(
        "cli-git: `{LFS_PROGRAM} pre-push` {status}, so this push was stopped before any ref changed and nothing was \
         published. The lines above come from `{LFS_PROGRAM}`; they usually name the cause: the LFS server \
         (`lfs.url`) is unreachable or refused the credentials, or a large file the pushed commits point to is not \
         in this clone's `.git/lfs`. Ways forward: fix that cause and push again, because objects the server already \
         has are not sent twice; upload by hand with `git lfs push --all <remote>` and push again; or publish the \
         pointer files without their objects, which nobody can download until the objects are uploaded, with \
         `git push --no-verify` (or `GIT_LFS_SKIP_PUSH=1`).\n"
    );
}

/// What:
///  Run `program pre-push <arguments>` with this process's standard streams and report Git's verdict for the hook.
/// Why:
///   Standard input carries the ref-update lines Git computed, so the child inherits it untouched; only a failure to
///  start or a non-zero status adds this wrapper's explanation, and the child's status is the hook's status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { status } = spawnSync(program, ['pre-push', ...args], { stdio: 'inherit' });
/// ```
pub fn run_with_program(program: &OsStr, arguments: &[OsString], diagnostics: &mut dyn Write) -> i32 {
    let started = Command::new(program)
        .arg("pre-push")
        .args(arguments)
        .status_retrying();
    let status = match started {
        Ok(finished) => finished,
        Err(error) => {
            // Terminal output error: the exit status already blocks the push; a closed stderr cannot change that.
            let _written = diagnostics.write_all(missing_program_message(&error).as_bytes());
            return 1;
        }
    };
    if status.success() {
        return 0;
    }
    let outcome: ChildOutcome = outcome_of(status);
    let description: String = match outcome {
        ChildOutcome::Exited(code) => format!("exited with status {code}"),
        ChildOutcome::Signaled(signal) => format!("was ended by signal {signal}"),
    };
    let code: i32 = exit_code(outcome);
    // Terminal output error: see above.
    let _written = diagnostics.write_all(failed_message(description.as_str()).as_bytes());
    return if code == 0 { 1 } else { code };
}

/// What:
///  The hook entry: run `git-lfs pre-push` and print diagnostics on standard error.
/// Why:
///   Git starts this through the injected `hook.cli-git-lfs-pre-push.command` with `<remote> <url>` and the update
///  lines on standard input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.exitCode = runWithProgram('git-lfs', args, process.stderr);
/// ```
pub fn run_hook_entry(arguments: &[OsString]) -> i32 {
    return run_with_program(OsStr::new(LFS_PROGRAM), arguments, &mut std::io::stderr());
}

/// Pure parts and the hook entry are controlled in the test module.
#[cfg(test)]
#[path = "lfs_pre_push_tests.rs"]
mod tests;
