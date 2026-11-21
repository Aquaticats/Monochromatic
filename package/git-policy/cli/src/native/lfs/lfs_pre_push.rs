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

/// The variable the commit service marks the environment of a push it runs with,
/// so the refusal says whose list was searched.
/// Nothing is written to disk and no value travels in it.
pub(crate) const PUSH_FROM_SERVICE: &str = "MONOCHROMATIC_PUSH_RUNS_IN_THE_COMMIT_SERVICE";

/// What:
///  The folders a push's children find their programs in,
///  and where that list came from.
/// Why:
///   A push typed in a shell searches that shell's `PATH`,
/// and a push the commit service runs searches the service's own,
/// which the
///  user manager fixed at the service's start and which no command of the person reaches.
/// A refusal that names
///  only `PATH` leaves the person unable to tell which list was searched or how to change it.
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
    /// The commit service's own environment,
    /// because it runs the push of a commit.
    ServiceEnvironment,
}

impl ProgramSearch {
    /// What:
    ///  The list the given environment holds,
    ///  and where it came from,
    /// read from the marker the commit service puts on a push it runs.
    /// Why:
    ///   Only the service composes a push environment,
    /// so it marks it,
    /// and every other push searches the environment of
    ///  its own command.
    /// The refusal then names the list it really searched,
    /// never one of another push.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const source = env[PUSH_FROM_SERVICE] ? { kind: 'serviceEnvironment' } : { kind: 'thisCommand' };
    /// ```
    pub fn of(environment: &[(OsString, OsString)]) -> ProgramSearch {
        let from_the_service: Option<OsString> = crate::child_environment::environment_value(environment, PUSH_FROM_SERVICE);
        let source: ProgramSearchSource = if from_the_service.is_some() { ProgramSearchSource::ServiceEnvironment } else { ProgramSearchSource::ThisCommand };
        let path: OsString = crate::child_environment::environment_value(environment, "PATH").unwrap_or_default();
        return ProgramSearch { path, source };
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
            ProgramSearchSource::ServiceEnvironment => format!(
                "No list of the shell that made the commit came with this push, so it searched the commit service's own `PATH`, which the manager that started the service fixed, and that `PATH` is not the one the shell that made the commit searches; that shell may hold `{LFS_PROGRAM}` anyway"
            ),
        };
    }

    /// What:
    ///  Where the person can put the program so the same push finds it,
    ///  with the push that follows it.
    /// Why:
    ///  A remedy that names a list the person cannot change sends them nowhere,
    /// and a remedy that promises an effect the named command does not have sends them round a circle.
    /// A push typed in that shell runs the upload in that shell's environment,
    /// and the commit service runs no part of that shell.
    fn remedy(&self) -> String {
        return match &self.source {
            ProgramSearchSource::ThisCommand => String::from("on this command's `PATH` and push again"),
            ProgramSearchSource::ServiceEnvironment => String::from(
                "on the `PATH` of a shell and push by hand once from that shell (`git push`), which searches that shell's own `PATH`",
            ),
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
         not have. Ways forward: put the folder that holds `{LFS_PROGRAM}` {remedy}; or install Git LFS so that a folder the list \
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
///  The refusal printed when `git-lfs` is found where the lookup looked and cannot be started.
/// Why:
///   The lookup found an executable file,
///  so it is not missing;
///  the operating system refused the start,
///  and its own error says why (a missing interpreter for a script,
///  a wrong machine type,
///  no permission to read it).
///  Reporting a program that is there as absent sends its reader to a `PATH` that is right.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: this push was stopped before it sent anything. ... but \`{LFS_PROGRAM}\` was found at ${program} and could not be started (${why}). ...`
/// ```
pub fn start_failed_message(program: &str, why: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: this push was stopped before it sent anything. The Git configuration of this repository has Git LFS \
         keys (`filter.lfs.*` or `lfs.*`), so the wrapper runs `{LFS_PROGRAM} push` to upload the large files the pushed commits point \
         to before it publishes the pointer files, and `{LFS_PROGRAM}` was found at {program} but could not be started ({why}). \
         Pushing without that upload would publish pointers whose content the LFS server does not have. Ways forward: make that file \
         runnable (the interpreter or loader the parentheses name must exist on this machine) and push again; or accept pointers \
         without objects for this push with `git push --no-verify` and upload the objects later with `git lfs push --all <remote>` \
         once it runs; or, if this repository does not use Git LFS, remove its configuration (`git lfs uninstall`, or delete the \
         `filter.lfs.*` and `lfs.*` keys).\n"
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
const LOGIN_NEEDED: [&str; 5] = [
    // Git's own words from `prompt.c`, with terminal prompts off.
    "terminal prompts disabled",
    // Git's credential prompt when no askpass program and no terminal answer.
    "could not read Username",
    "could not read Password",
    // git-lfs's words when every login it was given was refused.
    "too many authentication attempts",
    // OpenSSH's words when it would not or could not use a key: a key whose passphrase nobody could ask for at a
    //  terminal, and a key the server does not accept, print the same line (`SSH_ASKPASS_REQUIRE=force` with a
    //  program that answers nothing, the owner's answer of 2026-11-19: "Passphrases count too").
    "Permission denied (",
];

/// What:
///  The word that names the key an SSH key's passphrase is missing for:
///  it is the agent,
/// and the command that fills it is `ssh-add`.
/// Why:
///   A passphrase counts as a credential under the owner's answer of 2026-11-19T16:58:00-05:00,
///  so a key that no agent holds fails at once instead of asking,
///  and OpenSSH's own `Permission denied (publickey)` says nothing about what to do;
///  the explanation must name the command that loads the key.
///  A key the server simply does not accept prints the same line,
/// so the sentence says "refused or could not be unlocked" and names `ssh-add` as the way to rule the second out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SSH_ADD = 'ssh-add'
/// ```
pub const SSH_ADD_COMMAND: &str = "ssh-add";

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
///  Whether text a failed upload printed says that an SSH key could not be used,
///  which is the one login failure whose way forward can be `ssh-add`.
/// Why:
///   The words are OpenSSH's own,
/// and they arrive for either of two causes:
///  a passphrase nobody could ask for,
/// or a key the server refuses.
///  Telling an HTTP server's refused password to run `ssh-add` would send its reader to a command that does nothing for it,
///  so the sentence is added from these words and not from every login failure;
///  the sentence itself names both causes,
/// because these words do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.includes('Permission denied (')
/// ```
pub fn reads_as_a_key_failure(text: &str) -> bool {
    return text.contains("Permission denied (");
}

/// What:
///  Whether text a failed push printed says this machine does not know the SSH host key.
/// Why:
///   The no-prompt rule sends OpenSSH's yes/no question for an unknown host key to a program that answers nothing,
///  and an empty answer is not "yes",
///  so the connection is refused before any login and OpenSSH prints `Host key verification failed.`
///  (`confirm`,
///  `sshconnect.c` at `V_10_6_P2` lines 603 to 627).
///  That refusal is neither a credential the push lacked nor a fault it can retry away,
///  so it gets its own sentence.
///  OpenSSH prints that same last line for every refusal it makes over a host key,
///  and the other refusals are different causes that must keep OpenSSH's own words alone,
///  as `main` prints them:
/// a changed key says someone may be attacking the connection,
///  and a revoked key says a stolen key may be in use,
///  so the sentence that says to add the key would answer either warning with advice to trust the very key it doubts.
///  The reading is therefore a positive one about the words an unknown key leaves:
///  every refusal prints a line that names a host key or a certificate,
///  so a transcript is this refusal only when its host-key words are an unknown key's own:
///  the last line by itself (the default `ask` path,
///  where OpenSSH prints nothing of its own and the question is the askpass program's,
///  `sshconnect.c` lines 1211 to 1246),
///  the sentence for a key this machine has none of under strict checking (line 1204),
///  or the wrapper's own sentence,
/// which a pre-push hook may echo into the push's words.
///  Anything else that names a host key or a certificate is another refusal:
///  a changed key (`warn_changed_key`,
///  lines 1716 to 1726,
///  and `Host key for ... has changed and you have requested strict checking.`,
///  lines 1358 to 1360),
///  a key a `@revoked` line marks (the `HOST_REVOKED` case,
///  lines 1287 to 1303)
///  or a `RevokedHostKeys` file lists (lines 1552 to 1556),
///  and a certificate that does not fit the host (`Certificate invalid: ...`,
///  `sshkey.c`).
///  So a refusal this release has never seen keeps OpenSSH's words rather than gaining a sentence that may mislead.
///  Two kinds of line are passed over,
///  because neither can hold a refusal's own words:
///  the lines OpenSSH prints only at a debug log level (`debug1: `,
///  `debug2: `,
///  `debug3: `,
///  which `do_log` writes in front of those three levels alone,
///  `log.c` lines 371 to 382),
///  and the words inside single quotes,
///  which OpenSSH writes only around a host name,
///  an address or a path,
///  and Git only around the remote's URL in the line that ends a push its hook refused.
///  A certificate that does not fit is no refusal of its own:
///  `fail:` drops the certificate and retries with the plain key (`sshconnect.c` lines 1473 to 1486),
///  so the words that end the connection are that plain key's.
///  Two certificate cases still answer as other refusals:
///  a key a `@revoked` line marks,
///  whose words OpenSSH prints before `fail:` returns (lines 1287 to 1303),
///  and `Certificate invalid: ...` (`sshkey.c`),
///  which a certificate that does not fit the host prints before that last line.
///  Each arm's transcript was measured against a throwaway `sshd` with OpenSSH 10.6p2
///  and the wrapper's own two variables (`SSH_ASKPASS=false`,
///  `SSH_ASKPASS_REQUIRE=force`),
///  certificates included.
///  One refusal is still read as an unknown key though it is not:
///  with `CheckHostIP=yes` and a key this machine knows for the host beside a different one for its address,
///  OpenSSH writes that warning into `confirm`'s question (`sshconnect.c` lines 1429 to 1447),
///  the askpass program gets the question and standard error gets nothing of it,
///  so the transcript is the last line alone,
///  exactly as an unknown key's,
///  and nothing in a transcript can tell the two apart.
///  The sentence's way to add the key is then wrong for it and its way to connect once is right,
///  because the question plain `ssh` asks is OpenSSH's own about the address.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = text.split('\n').filter((line) => !isADebugLine(line))
/// text.includes('Host key verification failed') && lines.every((line) => !namesAHostKeyOutsideQuotes(line) || isAnUnknownKeysOwnLine(line))
/// ```
pub fn reads_as_unknown_host_key(text: &str) -> bool {
    if !text.contains(HOST_KEY_NOT_KNOWN_PHRASE) {
        return false;
    }
    // One fold of the constant, and one of each line: the comparison is the whole reason to read the lines.
    let last_line: String = HOST_KEY_NOT_KNOWN_PHRASE.to_ascii_lowercase();
    for line in text.lines() {
        let folded: String = line.to_ascii_lowercase();
        if is_a_debug_line(folded.as_str()) || !names_a_host_key_outside_quotes(folded.as_str()) {
            continue;
        }
        let an_unknown_keys_own: bool = folded.contains(last_line.as_str())
            || (folded.contains(NO_HOST_KEY_KNOWN_UNDER_STRICT_CHECKING_PHRASE) && folded.contains(STRICT_CHECKING_REQUESTED_PHRASE))
            || folded.contains(HOST_KEY_NOT_KNOWN_SENTENCE_OPENING);
        if an_unknown_keys_own {
            continue;
        }
        return false;
    }
    return true;
}

/// Whether one line,
/// in lower case,
/// names a host key or a certificate outside the single quotes in which OpenSSH and Git write data,
/// which every refusal OpenSSH makes over a host key does in at least one line.
/// OpenSSH writes a host name,
/// an address or a file path inside them,
/// and Git writes the remote's URL,
/// so the word in a remote's name is no refusal's own.
fn names_a_host_key_outside_quotes(folded: &str) -> bool {
    for words in folded.split('\'').step_by(2) {
        if words.contains(HOST_KEY_WORDS) || words.contains(CERTIFICATE_WORD) {
            return true;
        }
    }
    return false;
}

/// Whether one line,
/// in lower case,
/// is one OpenSSH prints only while the account asks it for debug output,
/// which no refusal it makes over a host key is:
/// `error`,
/// `logit` and `fatal` print their message alone at any log level,
/// and a debug line that names a host key or a certificate is either news about the key
/// (`Server host key: ...`,
/// `sshconnect.c` at `V_10_6_P2` lines 1523 to 1539)
/// or the certificate retry's own note (lines 1308 to 1317),
/// which `fail:` follows by dropping the certificate for the plain key (lines 1473 to 1486).
fn is_a_debug_line(folded: &str) -> bool {
    for prefix in OPENSSH_DEBUG_LINE_PREFIXES {
        if folded.starts_with(prefix) {
            return true;
        }
    }
    return false;
}

/// The last line OpenSSH prints when it will not connect because of the host key,
/// whichever refusal it found (`sshconnect2.c` at `V_10_6_P2` lines 102 to 104,
/// after `verify_host_key` answered non-zero).
const HOST_KEY_NOT_KNOWN_PHRASE: &str = "Host key verification failed";

/// The words,
/// in lower case,
/// with which a line names the thing a host-key refusal is about.
const HOST_KEY_WORDS: &str = "host key";

/// The word,
/// in lower case,
/// with which a line names the other thing a host-key refusal is about.
const CERTIFICATE_WORD: &str = "certificate";

/// The words `do_log` puts in front of every line of a debug log level,
/// and of those three levels alone (`log.c` at `V_10_6_P2` lines 371 to 382);
/// a line an error,
/// a log or a fatal refusal printed on standard error carries no prefix at all.
const OPENSSH_DEBUG_LINE_PREFIXES: [&str; 3] = ["debug1: ", "debug2: ", "debug3: "];

/// The words of OpenSSH's sentence for a host key this machine has none of while the account asks for strict checking
/// (`check_host_key`'s `HOST_NEW` case,
/// `sshconnect.c` lines 1204 and 1205).
const NO_HOST_KEY_KNOWN_UNDER_STRICT_CHECKING_PHRASE: &str = "host key is known for";

/// The words that end that sentence,
/// which no changed or revoked key's line carries beside `host key is known for`.
const STRICT_CHECKING_REQUESTED_PHRASE: &str = "and you have requested strict checking";

/// The opening words,
/// in lower case,
/// of the sentence this module prints for an unknown host key,
/// which a pre-push hook may echo into the words of the push it refused,
/// and which the service's record reads so that one refusal is told once.
pub(crate) const HOST_KEY_NOT_KNOWN_SENTENCE_OPENING: &str = "this machine does not know the ssh host key";

/// What:
///  The one sentence a refusal prints when the cause is a host key this machine does not know:
///  it says the connection was refused before any login,
///  and names the two ways to add the key that the wrapper does not touch,
///  then to retry.
/// Why:
///   The project's answer of 2026-11-20 keeps the refusal the no-prompt rule causes at a terminal
///  and puts the way out where a person meets it:
///  connect once with plain `ssh <host>`,
///  which still asks its own question,
///  or add the host key to `known_hosts`,
///  then push again.
///  No retry can add a host key,
///  so saying only that the push failed would leave the person with no move.
///  The sentence is shared by the push's failure record and the upload's explanation,
///  so the two faces cannot say different things about one refusal.
///  `host` is the host the wrapper can name for this refusal,
///  and the remote is written as `ssh <host>` only when it names none:
///  an upload's destination may be another host than the Git remote,
///  and a person told to run `ssh` must be told which machine to name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: this machine does not know the SSH host key ... Connect once with plain ${host ? `ssh ${host}` : 'ssh <host>'} ...`
/// ```
pub fn host_key_not_known_sentence(host: Option<&str>) -> String {
    let connect: String = match host {
        Some(name) => format!("Connect once with plain `ssh {name}`"),
        None => String::from("Connect once with plain `ssh <host>`"),
    };
    return format!(
        "monochromatic-git-policy-cli: this machine does not know the SSH host key of the remote, and no command the wrapper \
         runs asks whether to trust a new one (`SSH_ASKPASS_REQUIRE=force` sends that yes/no question to a program that \
         answers nothing), so the connection was refused before any login. {connect}, which the wrapper does not touch and \
         which still asks, or add the host key to `known_hosts`, then push again."
    );
}

/// What:
///  The refusal that follows a failed upload which needed a login:
///  it names the server,
/// says that no command the wrapper runs asks anyone for a credential,
/// and names every way to give git-lfs a login it can use,
/// including `ssh-add` for a key whose passphrase was never unlocked.
/// Why:
///   The incident of 2026-11-10 ran for seven minutes because nothing said what was missing,
/// and the push it belonged to could not be
///  published (`doc/troubleshooting/lfs-credential-in-url.md`).
/// The server is named without any credential,
/// and the setting that stores one is named without its value.
///  The rule is the same for every command since the owner's answer of 2026-11-19T16:58:00-05:00:
/// "Avoid ever asking for someone to type a username or password.",
/// with an SSH key's passphrase counted as a credential,
/// so there is no branch left for a command at a terminal
///  and the words never name a terminal that did not decide anything.
///  `key_not_unlocked` is whether the program's own words say a key could not be used,
/// which is when an agent and `ssh-add` are the way forward;
///  an HTTP server's refused password gets the credential-helper advice alone.
///  The sentence starts the agent rather than assuming one runs,
///  because the project's answer of 2026-11-20 is "We must support situations in which ssh-agent isn't there":
///  it names `eval "$(ssh-agent)"` and `ssh-add`,
///  or a key without a passphrase,
///  which needs no agent.
///  The first form is Bourne shell syntax,
/// which fish rejects,
///  so the sentence names fish's own form beside it:
///  `ssh-agent -c` prints `setenv` commands,
/// which `eval (ssh-agent -c)` reads
///  (`ssh-agent`'s manual,
///  "The agent prints the commands to set the environment variables ... `-c` Generate C-shell commands on stdout").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `...needed a login, and this wrapper never lets a command ask for one... ${keyNotUnlocked ? '...`ssh-add`...' : ''}`
/// ```
pub fn login_needed_message(server: &str, key_not_unlocked: bool) -> String {
    let key_words: String = if key_not_unlocked {
        format!(
            " The key that server asked for was refused or could not be unlocked: with no program to ask a passphrase and \
             no terminal read, a key that no agent holds cannot be used, and a key the server does not accept answers \
             the same way. Start an SSH agent with `eval \"$(ssh-agent)\"` (`eval (ssh-agent -c)` in fish), then load the key \
             with `{SSH_ADD_COMMAND}` \
             (`{SSH_ADD_COMMAND} <key file>` for a key outside the agent's default list; `{SSH_ADD_COMMAND} -l` lists \
             what the agent holds), or use a key without a passphrase, and push again; a key the server still refuses \
             needs the server's account to accept it."
        )
    } else {
        String::new()
    };
    return format!(
        "monochromatic-git-policy-cli: the Git LFS upload to {server} needed a login, and this wrapper never lets a command \
         ask for one (`GIT_ASKPASS` set to the empty string, `GIT_TERMINAL_PROMPT=0`, `SSH_ASKPASS=false` and \
         `SSH_ASKPASS_REQUIRE=force`), whether or not a terminal is there: the owner decided that no command it runs or \
         forwards asks a person for a username or a password. That is why nothing prompted, and why the program stopped \
         instead. Ways forward: store the login where `git-lfs` finds it without a person, in a Git credential helper for \
         that URL (`git config --global credential.helper libsecret` on a desktop, `store` for a file only you can read, or \
         `cache` for a while), and push again, because objects the server already has are not sent twice.{key_words} A login \
         written into the `lfs.url` of a repository's configuration instead of a helper is the defect \
         `doc/troubleshooting/lfs-credential-in-url.md` describes.\n"
    );
}

/// Pure parts are controlled in the test module.
#[cfg(test)]
#[path = "lfs_pre_push_tests.rs"]
mod tests;
