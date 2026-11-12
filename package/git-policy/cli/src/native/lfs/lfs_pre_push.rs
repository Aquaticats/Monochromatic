//! What:
//!  When a push needs the Git LFS upload,
//! and the texts the parent prints around it.
//! Why:
//!  Traditional hook files are unsupported,
//! so the `pre-push` shim `git lfs install` writes cannot be relied on.
//!  Without an upload step a push publishes pointer files whose objects the LFS server never received.
//!  The wrapper therefore runs `git-lfs push <remote> <refs>` itself,
//! as a child it owns,
//! before it starts the push
//!  (`lfs_upload.rs`),
//! only when the Git configuration holds LFS keys;
//! the push carries no hook and no injected
//!  configuration,
//! the wrapper never starts itself and writes no shell text
//!  (`doc/planning/cli-git-no-self-invocation.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (usesLfs(snapshot) && !skipRequested(env)) { const code = await uploadObjects(plan); if (code) return code; }
//! ```

/// Whether the configuration snapshot holds LFS keys.
use crate::config_snapshot::ConfigSnapshot;
/// Owned operating-system text for the arguments.
use std::ffi::{OsStr, OsString};
/// Where `PATH` entries are looked at.
use std::path::Path;

/// The name the caller's switch uses:
/// `-c hook.cli-git-lfs-pre-push.enabled=false` skips the upload for one command,
///  as it turned off the hook an earlier version put in effect.
pub const SWITCH_NAME: &str = "cli-git-lfs-pre-push";

/// The program the upload runs,
/// found on `PATH` as the shim `git lfs install` wrote found it.
pub const LFS_PROGRAM: &str = "git-lfs";

/// Key prefixes whose presence means the repository is set up for Git LFS.
const LFS_KEY_PREFIXES: [&str; 2] = ["filter.lfs.", "lfs."];

/// Scopes that describe the machine or the account,
/// not this repository.
const MACHINE_SCOPES: [&str; 2] = ["system", "global"];

/// What:
///  Whether a configuration entry of this repository,
/// its worktree or the command line belongs to Git LFS.
/// Why:
///   `git lfs install` in an account writes `filter.lfs.*` to the global file,
/// which every repository then sees,
/// so a
///  global key says nothing about this repository.
///  The filter's first run in a repository writes `lfs.repositoryformatversion` to its local file,
/// and a server is
///  named by `lfs.url`;
/// one such key in the local,
/// worktree or command scope is the sign that pushed pointers need
///  their objects uploaded first,
/// without making every repository of the machine an LFS repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.some(({ scope, key }) => !['system', 'global'].includes(scope) && (key.startsWith('filter.lfs.') || key.startsWith('lfs.')));
/// ```
pub fn uses_lfs(snapshot: &ConfigSnapshot) -> bool {
    for entry in snapshot.entries() {
        if MACHINE_SCOPES.contains(&entry.scope.as_str()) {
            continue;
        }
        for prefix in LFS_KEY_PREFIXES {
            if entry.key.starts_with(prefix) {
                return true;
            }
        }
    }
    return false;
}

/// What:
///  The number the environment gives in `GIT_CONFIG_COUNT`:
/// how many `GIT_CONFIG_KEY_n`/`GIT_CONFIG_VALUE_n` pairs
///  it holds,
/// or zero when the variable is missing or is not a plain number.
/// Why:
///   Both the switch reader and the pair builder of the upload start from this count,
/// and a function stands where a
///  chain of closures stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const count = Number.parseInt(env.GIT_CONFIG_COUNT ?? '0', 10) || 0;
/// ```
pub fn environment_config_count(environment: &[(OsString, OsString)]) -> usize {
    let Some(value) = crate::child_environment::environment_value(environment, "GIT_CONFIG_COUNT") else {
        return 0;
    };
    let Some(text) = value.to_str() else {
        return 0;
    };
    return text.parse::<usize>().unwrap_or(0);
}

/// One numbered variable of the environment's configuration pairs:
/// `GIT_CONFIG_KEY_3` for kind `KEY` and index 3.
fn numbered_config_variable(environment: &[(OsString, OsString)], kind: &str, index: usize) -> Option<OsString> {
    return crate::child_environment::environment_value(environment, &format!("GIT_CONFIG_{kind}_{index}"));
}

/// Whether a configuration name is the switch:
/// Git reads the section and the variable case-insensitively and the
///  subsection exactly.
fn is_switch_key(name: &str) -> bool {
    const VARIABLE: &str = ".enabled";
    let Some(section) = name.get(..5) else {
        return false;
    };
    if !section.eq_ignore_ascii_case("hook.") {
        return false;
    }
    let rest: &str = &name[5..];
    if rest.len() <= VARIABLE.len() {
        return false;
    }
    if !rest.is_char_boundary(rest.len() - VARIABLE.len()) {
        return false;
    }
    let (subsection, variable) = rest.split_at(rest.len() - VARIABLE.len());
    return variable.eq_ignore_ascii_case(VARIABLE) && subsection == SWITCH_NAME;
}

/// Whether a value of the switch reads as false the way Git reads a boolean (an empty value counts as false).
fn reads_as_false(value: &str) -> bool {
    for spelling in ["false", "no", "off", "0", ""] {
        if value.eq_ignore_ascii_case(spelling) {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether the caller switched the upload off for this push,
/// with `-c hook.cli-git-lfs-pre-push.enabled=false` or the same setting named through the environment
///  (`--config-env`) among the global options,
/// or the same pair in `GIT_CONFIG_COUNT`/`GIT_CONFIG_KEY_n`/`GIT_CONFIG_VALUE_n`.
/// Why:
///   It is a documented way to push without the upload,
/// so a missing `git-lfs` must not stop such a push.
/// The last
///  setting decides,
/// as Git reads them,
/// and the caller's command-line options come after the environment's pairs.
/// Both spellings of a command-line
///  setting are read from one answer,
/// so `-c` and `--config-env` cannot push differently.
/// A repository or account
///  configuration file is not read:
/// that would cost a Git start for every push;
/// use `--no-verify` or
///  `GIT_LFS_SKIP_PUSH` for a standing choice in the environment.
/// Gotcha:
///  A `--config-env` the environment cannot answer is a command line Git itself refuses,
///  so the push never runs;
/// this answer then reports only what the numbered pairs set,
///  and the upload refuses to start for the same reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lastEnabledSetting([...envPairs, ...commandLinePairs(prefix, env)], `hook.${name}.enabled`) is false
/// ```
pub fn hook_switched_off(global_prefix: &[OsString], environment: &[(OsString, OsString)]) -> bool {
    let mut off: bool = false;
    let count: usize = environment_config_count(environment);
    for index in 0..count {
        let key_is_switch: bool = match numbered_config_variable(environment, "KEY", index) {
            Some(name) => is_switch_key(&name.to_string_lossy()),
            None => false,
        };
        if key_is_switch {
            off = match numbered_config_variable(environment, "VALUE", index) {
                Some(value) => reads_as_false(&value.to_string_lossy()),
                None => false,
            };
        }
    }
    let Some(pairs) = crate::lfs_command_line_configuration::command_line_pairs(global_prefix, environment) else {
        // Git refuses this command line, so the push never runs; a switch the numbered pairs set still stands.
        return off;
    };
    for (name, value) in pairs {
        // A name that is not valid text is no switch: Git compares the name's letters, and such a name holds none.
        let Some(text) = name.to_str() else {
            continue;
        };
        if is_switch_key(text) {
            // Git reads a name without `=` as the boolean true, which the resolver already wrote as that value.
            off = value.to_str().is_some_and(reads_as_false);
        }
    }
    return off;
}

/// What:
///  The first executable file named `git-lfs` in the directories of a `PATH` value,
/// in order.
/// Why:
///   A stat per directory answers it without starting any process;
/// an empty entry means the current directory,
/// as
///  `PATH` lookup treats it.
/// The upload starts exactly this file,
/// so the pre-check and the start agree about which
///  `git-lfs` runs even though the child's `PATH` leads with the real Git's directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.split(':').map((dir) => join(dir || '.', 'git-lfs')).find(isExecutableFile)
/// ```
pub fn find_program(path: &OsStr) -> Option<std::path::PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    for directory in std::env::split_paths(path) {
        let base: &Path = if directory.as_os_str().is_empty() { Path::new(".") } else { directory.as_path() };
        let candidate: std::path::PathBuf = base.join(LFS_PROGRAM);
        let is_executable_file: bool = match std::fs::metadata(&candidate) {
            Ok(found) => found.is_file() && found.permissions().mode() & 0o111 != 0,
            Err(_) => false,
        };
        if is_executable_file {
            // A relative `PATH` entry is relative to where the lookup happens; the child may start elsewhere (`-C`).
            return std::path::absolute(candidate).ok();
        }
    }
    return None;
}

/// Whether `git-lfs` is an executable file in one of the directories of a `PATH` value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// findProgram(path) !== undefined
/// ```
pub fn program_on_path(path: &OsStr) -> bool {
    return find_program(path).is_some();
}

/// What:
///  Whether the environment asks to skip the upload (`GIT_LFS_SKIP_PUSH` set to a true value).
/// Why:
///   `git lfs push` does not read that variable (only the hook form did),
/// so the parent honours it,
/// as the explanation
///  texts promise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['1', 't', 'true', 'y', 'yes', 'on'].includes(env.GIT_LFS_SKIP_PUSH?.toLowerCase())
/// ```
pub fn skip_requested(environment: &[(std::ffi::OsString, std::ffi::OsString)]) -> bool {
    return match crate::child_environment::environment_value(environment, "GIT_LFS_SKIP_PUSH") {
        Some(value) => {
            let text: String = value.to_string_lossy().to_ascii_lowercase();
            ["1", "t", "true", "y", "yes", "on"].contains(&text.as_str())
        }
        None => false,
    };
}

/// What:
///  The folders a push's children find their programs in,
///  and where that list came from.
/// Why:
///   A push typed in a shell searches that shell's `PATH`,
/// and a push the commit service runs searches the list it was given:
/// the shell's,
///  which the client records,
/// or its own,
/// which the user manager fixed at the service's start.
/// A refusal that names only `PATH` leaves the person
///  unable to tell which list was searched or how to change it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramSearch {
    /// The `PATH` the push's children receive.
    pub path: OsString,
    /// Where that list came from.
    pub source: ProgramSearchSource,
}

/// What:
///  Where a push's program search list came from.
/// Why:
///   The refusal says it,
/// so a person learns whether their own shell was searched or the service's list,
/// and which
///  one to change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgramSearchSource {
    /// The environment of the command that is pushing.
    ThisCommand,
    /// The record the client that made the commit left,
    /// at this path.
    ClientRecord(std::path::PathBuf),
    /// The service's own environment,
    /// because no client record was at this path.
    ServiceEnvironment {
        /// Where the client that made the commit would have left its list.
        missing_record: std::path::PathBuf,
    },
}

impl ProgramSearch {
    /// What:
    ///  The list the given environment holds,
    ///  and where it came from,
    /// read from the markers a service's push carries.
    /// Why:
    ///   Only the service composes a push environment out of a record it read or did not find,
    /// so it marks which one that was,
    /// and every other push searches the environment of its own command.
    /// The refusal then names the list it really searched,
    /// never one of another push.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const source = env[RECORD_VARIABLE] ? { kind: 'clientRecord', path: env[RECORD_VARIABLE] }
    ///   : env[MISSING_VARIABLE] ? { kind: 'serviceEnvironment', missing: env[MISSING_VARIABLE] } : { kind: 'thisCommand' };
    /// ```
    pub fn of(environment: &[(OsString, OsString)]) -> ProgramSearch {
        let record: Option<OsString> = crate::child_environment::environment_value(environment, crate::service_client_program_search::RECORD_VARIABLE);
        let missing: Option<OsString> = crate::child_environment::environment_value(environment, crate::service_client_program_search::MISSING_VARIABLE);
        let source: ProgramSearchSource = match (record, missing) {
            (Some(path), _) => ProgramSearchSource::ClientRecord(std::path::PathBuf::from(path)),
            (None, Some(path)) => ProgramSearchSource::ServiceEnvironment { missing_record: std::path::PathBuf::from(path) },
            (None, None) => ProgramSearchSource::ThisCommand,
        };
        return ProgramSearch { path: crate::child_environment::environment_value(environment, "PATH").unwrap_or_default(), source };
    }

    /// What:
    ///  The folders the list names,
    ///  as one line,
    ///  an empty entry standing for the current directory as `PATH` reads it.
    /// Why:
    ///   The refusal must name what was searched,
    /// and `PATH` itself is not a list of folders a person can read at a glance.
    fn folders(&self) -> String {
        let mut names: Vec<String> = Vec::new();
        for folder in std::env::split_paths(self.path.as_os_str()) {
            let shown: String = if folder.as_os_str().is_empty() { String::from(".") } else { folder.display().to_string() };
            names.push(shown);
        }
        return if names.is_empty() { String::from("none (the list is empty)") } else { names.join(", ") };
    }

    /// What:
    ///  Where the searched list came from,
    ///  in words that say what the person could change.
    /// Why:
    ///   The list is the whole question:
    /// a push that fails on the service's list would succeed on the shell's.
    fn origin(&self) -> String {
        return match &self.source {
            ProgramSearchSource::ThisCommand => String::from("The list is this command's own `PATH`"),
            ProgramSearchSource::ClientRecord(path) => format!(
                "The list is the `PATH` recorded at {} by the client that made this commit, so it is the list the shell that committed searched",
                path.display()
            ),
            ProgramSearchSource::ServiceEnvironment { missing_record } => format!(
                "No `PATH` this release can read was recorded at {} by the client that made the commit, so this push searched the service's own `PATH`, which the manager that started the service fixed; the shell that made the commit may hold `{LFS_PROGRAM}` anyway",
                missing_record.display()
            ),
        };
    }

    /// What:
    ///  Where the person can put the program so the same push finds it.
    /// Why:
    ///  A remedy that names a list the person cannot change sends them nowhere.
    fn remedy(&self) -> String {
        return match &self.source {
            ProgramSearchSource::ThisCommand => String::from("on this command's `PATH`"),
            ProgramSearchSource::ClientRecord(_) => String::from("on the `PATH` of the shell that commits"),
            ProgramSearchSource::ServiceEnvironment { .. } => {
                String::from("on the `PATH` of the shell that commits, then push by hand once (`git push`), which records that list")
            }
        };
    }
}

/// What:
///  The refusal printed when `git-lfs` is not in the folders the push searches.
/// Why:
///   Without the program the objects cannot be uploaded,
/// and pushing anyway would publish pointers the server cannot
///  serve;
/// the refusal names the program,
/// the folders that were searched and where that list came from,
/// and every valid way
///  forward,
/// including leaving LFS if the repository does not need it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: this push was stopped before it sent anything: ... not found in ${search.folders()}; ${search.origin()} ...`
/// ```
pub fn missing_program_message(search: &ProgramSearch) -> String {
    return format!(
        "monochromatic-git-policy-cli: this push was stopped before it sent anything. The Git configuration of this repository has Git LFS \
         keys (`filter.lfs.*` or `lfs.*`), so the wrapper runs `{LFS_PROGRAM} push` to upload the large \
         files the pushed commits point to before it publishes the pointer files, but `{LFS_PROGRAM}` was not found in \
         the folders this push searches: {folders}. {origin}. Pushing without that upload would publish pointers whose content the LFS server does \
         not have. Ways forward: put the folder that holds `{LFS_PROGRAM}` {remedy} and push again; or install Git LFS so that a folder the list \
         names holds it; or, if this \
         repository does not use Git LFS, remove its configuration (`git lfs uninstall`, or delete the `filter.lfs.*` \
         and `lfs.*` keys); or accept pointers without objects for this push with `git push --no-verify` and upload \
         the objects later with `git lfs push --all <remote>` once Git LFS is installed.\n",
        folders = search.folders(),
        origin = search.origin(),
        remedy = search.remedy(),
    );
}

/// What:
///  The refusal printed when the wrapper cannot work out which objects a push needs uploaded.
/// Why:
///   Running the push without an upload could publish pointers whose objects the server lacks,
/// so the push is not started;
///  every way forward is named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: this push was stopped before it sent anything: ${why}...`
/// ```
pub fn plan_failed_message(why: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: this push was stopped before it sent anything. The Git configuration of this repository has Git LFS \
         keys, so the wrapper uploads the large files the pushed commits point to before it publishes the pointer \
         files, but it could not work out which refs this push sends ({why}). Pushing without that upload could \
         publish pointers whose content the LFS server does not have. Ways forward: push again, because the cause may \
         be transient; or upload by hand with `git lfs push --all <remote>` and push with `git push --no-verify`.\n"
    );
}

/// What:
///  The explanation printed after `git-lfs push` failed.
/// Why:
///   `git-lfs` prints its own error first (a refused credential,
/// an unreachable server,
/// a missing local object);
///  this text says what that means for the push and every way forward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: git-lfs push exited with status ${code}; ...`
/// ```
pub fn failed_message(status: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: `{LFS_PROGRAM} push` {status}, so this push was not started and nothing was published. \
         The lines above come from `{LFS_PROGRAM}`; they usually name the cause: the LFS server \
         (`lfs.url`) is unreachable or refused the credentials, or a large file the pushed commits point to is not \
         in this clone's `.git/lfs`. Ways forward: fix that cause and push again, because objects the server already \
         has are not sent twice; upload by hand with `git lfs push --all <remote>` and push again; or publish the \
         pointer files without their objects, which nobody can download until the objects are uploaded, with \
         `git push --no-verify` (or `GIT_LFS_SKIP_PUSH=1`).\n"
    );
}

/// What:
///  The words that say a login was needed,
/// as Git 2.56.0 and git-lfs 3.8.1 print them when the wrapper has told them never to prompt.
/// Why:
///   The upload's own words name the cause,
/// and a person reading them may not know that "terminal prompts disabled" means
///  "a login is missing,
/// and nothing was allowed to ask for one";
/// these are the spellings the refusal is recognized by.
/// What a run printed on 2026-11-10,
/// with the pair set and a server answering `401` with `LFS-Authenticate`
///  (`binary_lfs_server_tests.rs`,
/// the control of the incident's own invocation):
/// `fatal: could not read Username for 'http://127.0.0.1:46731': terminal prompts disabled`
///  and `batch response: Git credentials for http://127.0.0.1:46731/lfs not found.`
/// The last entry is git-lfs 3.8.1's own answer when a login it was given is refused,
///  read in this session's control of the same incident with a prompt program that answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LOGIN_NEEDED = ['terminal prompts disabled', 'could not read Username', 'could not read Password', 'too many authentication attempts'];
/// ```
const LOGIN_NEEDED: [&str; 4] = [
    // Git's own words from `prompt.c`, with terminal prompts off.
    "terminal prompts disabled",
    // Git's credential prompt when no askpass program and no terminal answer.
    "could not read Username",
    "could not read Password",
    // git-lfs's words when every login it was given was refused.
    "too many authentication attempts",
];

/// What:
///  Whether text a failed upload printed says that it needed a login.
/// Why:
///   Only that failure has a way forward the person must be told about,
/// and a missing object or an unreachable server has others.
/// A server that refused a login it was given says "Authorization error" and is not one of these:
/// a login is there,
/// and the ways forward are the repository's access and not a credential helper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// LOGIN_NEEDED.some((words) => text.includes(words))
/// ```
pub fn reads_as_a_login_failure(text: &str) -> bool {
    for words in LOGIN_NEEDED {
        if text.contains(words) {
            return true;
        }
    }
    return false;
}

/// What:
///  The refusal that follows a failed upload which needed a login:
///  it names the server,
/// says what the wrapper did about prompting,
/// and names every way to give git-lfs a login it can use.
/// Why:
///   The incident of 2026-11-10 ran for seven minutes because nothing said what was missing,
/// and the push it belonged to could not be
///  published (`doc/troubleshooting/lfs-credential-in-url.md`).
/// The server is named without any credential,
/// and the setting that stores one is named without its value.
///  `prompting_disabled` is whether the overlay the program ran under carried the never-prompt pair:
/// a command at a terminal keeps Git's and `git-lfs`'s own prompting,
/// and telling its person that no terminal existed
///  names a cause that cannot be fixed because it was never there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// promptingDisabled ? `...this command has no controlling terminal, so ...` : `...this command has a controlling terminal, so ...`
/// ```
pub fn login_needed_message(server: &str, prompting_disabled: bool) -> String {
    if !prompting_disabled {
        return format!(
            "monochromatic-git-policy-cli: the Git LFS upload to {server} needed a login and stopped. This command has a \
             controlling terminal, so the wrapper left prompting to `git-lfs` and the Git it asks (`GIT_ASKPASS` and \
             `GIT_TERMINAL_PROMPT` unchanged): either a login they already had was refused, or a variable or Git setting of this \
             command stopped them from asking. Ways forward: store a login the server accepts in a Git credential helper for that \
             URL (`git config --global credential.helper libsecret` on a desktop, `store` for a file only you can read, or `cache` \
             for a while), and push again, because objects the server already has are not sent twice; or check a \
             `GIT_TERMINAL_PROMPT`, `GIT_ASKPASS`, `core.askPass` or `credential.helper` setting of this command that refuses the \
             login. A login written into the `lfs.url` of a repository's configuration instead of a helper is the defect \
             `doc/troubleshooting/lfs-credential-in-url.md` describes.\n"
        );
    }
    return format!(
        "monochromatic-git-policy-cli: the Git LFS upload to {server} needed a login, and this command has no controlling \
         terminal, so `git-lfs` and the Git it asks were told never to prompt (`GIT_ASKPASS` set to the empty string and \
         `GIT_TERMINAL_PROMPT=0`). That is why no password window opened for nobody to answer, and why the program stopped \
         instead. Ways forward: store the login where `git-lfs` finds it without a person, in a Git credential helper for \
         that URL (`git config --global credential.helper libsecret` on a desktop, `store` for a file only you can read, or \
         `cache` for a while), and push again, because objects the server already has are not sent twice; or run this push \
         from a terminal, where Git's and `git-lfs`'s own prompting is kept. A login written into the `lfs.url` of a \
         repository's configuration instead of a helper is the defect \
         `doc/troubleshooting/lfs-credential-in-url.md` describes.\n"
    );
}

/// Pure parts are controlled in the test module.
#[cfg(test)]
#[path = "lfs_pre_push_tests.rs"]
mod tests;
