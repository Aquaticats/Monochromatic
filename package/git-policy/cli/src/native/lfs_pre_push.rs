//! What:
//!  The Git LFS upload before a push, run as a supported `pre-push` configuration hook the wrapper puts in effect
//!  for its own pushes, and the explanations the wrapper (the parent that owns the push) adds around it.
//! Why:
//!  Traditional hook files are unsupported, so the `pre-push` shim `git lfs install` writes cannot be relied on.
//!  Without an upload step a push publishes pointer files whose objects the LFS server never received.
//!  The wrapper therefore adds `hook.cli-git-lfs-pre-push.event=pre-push` and the plain command `git-lfs pre-push` to
//!  every real push it starts, only when the Git configuration holds LFS keys.
//!  Git runs that command itself, with its own arguments (`<remote> <url>`) and the ref-update lines on standard input,
//!  before it sends anything, and a non-zero status stops the push; this module adds no copy of that computation.
//!  The command holds no shell syntax and names no wrapper executable: the wrapper never starts itself and never
//!  writes shell text (`doc/planning/cli-git-no-self-invocation.md`).
//!  What a failed upload means is said by the parent, in `lfs_hook_trace`, from Git's own trace of the hook.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const args = usesLfs(snapshot) ? [...lfsHookArguments(), ...pushArguments] : pushArguments;
//! ```

/// Whether the configuration snapshot holds LFS keys.
use super::config_snapshot::ConfigSnapshot;
/// Owned operating-system text for the arguments.
use std::ffi::{OsStr, OsString};
/// Where `PATH` entries are looked at.
use std::path::Path;

/// The friendly name of the injected hook; `hook.<name>.enabled=false` on the command line turns it off.
pub const HOOK_NAME: &str = "cli-git-lfs-pre-push";

/// The command Git runs for the hook: two plain words that Git starts directly, with no shell syntax.
pub const HOOK_COMMAND: &str = "git-lfs pre-push";

/// The program the hook runs, found on `PATH` as the shim `git lfs install` wrote found it.
pub const LFS_PROGRAM: &str = "git-lfs";

/// The status `wait` reports for a program ended by signal `n` is this base plus `n`.
pub const SIGNAL_STATUS_BASE: i64 = 128;

/// The highest signal number a status can stand for.
pub const MAX_SIGNAL: i64 = 64;

/// Key prefixes whose presence means the repository is set up for Git LFS.
const LFS_KEY_PREFIXES: [&str; 2] = ["filter.lfs.", "lfs."];

/// Scopes that describe the machine or the account, not this repository.
const MACHINE_SCOPES: [&str; 2] = ["system", "global"];

/// What:
///  Whether a configuration entry of this repository, its worktree or the command line belongs to Git LFS.
/// Why:
///   `git lfs install` in an account writes `filter.lfs.*` to the global file, which every repository then sees, so a
///  global key says nothing about this repository.
///  The filter's first run in a repository writes `lfs.repositoryformatversion` to its local file, and a server is
///  named by `lfs.url`; one such key in the local, worktree or command scope is the sign that pushed pointers need
///  their objects uploaded first, without making every repository of the machine an LFS repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.some(({ scope, key }) => !['system', 'global'].includes(scope) && (key.startsWith('filter.lfs.') || key.startsWith('lfs.')));
/// ```
pub fn uses_lfs(snapshot: &ConfigSnapshot) -> bool {
    return snapshot.entries().iter().any(|entry| {
        return !MACHINE_SCOPES.contains(&entry.scope.as_str())
            && LFS_KEY_PREFIXES.iter().any(|prefix| {
                return entry.key.starts_with(prefix);
            });
    });
}

/// What:
///  The four global-option tokens that put the LFS hook in effect for one Git process.
/// Why:
///   The command is two plain words, so Git starts `git-lfs` itself without a shell and nothing is quoted;
///  nothing is written to any configuration file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['-c', `hook.${name}.event=pre-push`, '-c', `hook.${name}.command=git-lfs pre-push`]
/// ```
pub fn hook_arguments() -> Vec<OsString> {
    return vec![
        OsString::from("-c"),
        OsString::from(format!("hook.{HOOK_NAME}.event=pre-push")),
        OsString::from("-c"),
        OsString::from(format!("hook.{HOOK_NAME}.command={HOOK_COMMAND}")),
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
/// [...hookArguments(), ...arguments]
/// ```
pub fn with_lfs_hook(arguments: &[OsString]) -> Vec<OsString> {
    let mut combined: Vec<OsString> = hook_arguments();
    combined.extend(arguments.iter().cloned());
    return combined;
}

/// What:
///  Whether the global options before `push` turn the injected hook off for this push.
/// Why:
///   The caller's `-c hook.cli-git-lfs-pre-push.enabled=false` is a documented way to push without the upload, so a
///  missing `git-lfs` must not stop such a push; the last such `-c` decides, as Git reads them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lastEnabledSetting(prefix, `hook.${name}.enabled`) === 'false'
/// ```
pub fn hook_switched_off(global_prefix: &[OsString]) -> bool {
    let key: String = format!("hook.{HOOK_NAME}.enabled=");
    let mut off: bool = false;
    let mut tokens = global_prefix.iter();
    while let Some(token) = tokens.next() {
        if token.as_os_str() != "-c" {
            continue;
        }
        let Some(setting) = tokens.next().and_then(|value| {
            return value.to_str();
        }) else {
            continue;
        };
        if let Some(value) = setting.strip_prefix(key.as_str()) {
            // Git reads these spellings, and an empty value, as false.
            off = ["false", "no", "off", "0", ""].iter().any(|spelling| {
                return value.eq_ignore_ascii_case(spelling);
            });
        }
    }
    return off;
}

/// What:
///  Whether `git-lfs` is an executable file in one of the directories of a `PATH` value.
/// Why:
///   A stat per directory answers it without starting any process; an empty entry means the current directory, as
///  `PATH` lookup treats it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.split(':').some((dir) => isExecutableFile(join(dir || '.', 'git-lfs')))
/// ```
pub fn program_on_path(path: &OsStr) -> bool {
    use std::os::unix::fs::PermissionsExt;
    return std::env::split_paths(path).any(|directory| {
        let base: &Path = if directory.as_os_str().is_empty() { Path::new(".") } else { directory.as_path() };
        return std::fs::metadata(base.join(LFS_PROGRAM))
            .is_ok_and(|found| {
                return found.is_file() && found.permissions().mode() & 0o111 != 0;
            });
    });
}

/// What:
///  The refusal printed when `git-lfs` is not on `PATH`.
/// Why:
///   Without the program the objects cannot be uploaded, and pushing anyway would publish pointers the server cannot
///  serve; every valid way forward is named, including leaving LFS if the repository does not need it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: this push was stopped before it sent anything: ...`
/// ```
pub fn missing_program_message() -> String {
    return format!(
        "cli-git: this push was stopped before it sent anything. The Git configuration of this repository has Git LFS \
         keys (`filter.lfs.*` or `lfs.*`), so the wrapper has Git run `{LFS_PROGRAM} pre-push` to upload the large \
         files the pushed commits point to before it publishes the pointer files, but `{LFS_PROGRAM}` was not found in \
         any directory of PATH. Pushing without that upload would publish pointers whose content the LFS server does \
         not have. Ways forward: install Git LFS so that `{LFS_PROGRAM}` is on PATH and push again; or, if this \
         repository does not use Git LFS, remove its configuration (`git lfs uninstall`, or delete the `filter.lfs.*` \
         and `lfs.*` keys); or accept pointers without objects for this push with `git push --no-verify` and upload \
         the objects later with `git lfs push --all <remote>` once Git LFS is installed.\n"
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
///  How the hook's status reads in `failed_message`.
/// Why:
///   Git's trace records the status `wait` gave, where a program ended by signal `n` reads `128 + n`; a program can
///  also exit with such a status itself, so both readings are named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// code > 128 ? `exited with status ${code} (ended by signal ${code - 128} when it did not exit by itself)` : `exited with status ${code}`
/// ```
pub fn status_description(code: i64) -> String {
    if code > SIGNAL_STATUS_BASE && code <= SIGNAL_STATUS_BASE + MAX_SIGNAL {
        return format!(
            "exited with status {code} (that is signal {} when it was ended by one)",
            code - SIGNAL_STATUS_BASE
        );
    }
    return format!("exited with status {code}");
}

/// Pure parts are controlled in the test module.
#[cfg(test)]
#[path = "lfs_pre_push_tests.rs"]
mod tests;
