//! What:
//!  Whether Git reaches an executable named `git-<word>` before it looks for an alias of that word.
//! Why:
//!  `run_argv` tries a built-in,
//!  then the dashed external,
//!  then the alias (`git.c:837` to
//!  `git.c:914`),
//!  so a repository that defines `alias.audit-external` cannot replace an installed
//!  `git-audit-external`:
//! Git runs the program,
//! and only a name with neither a built-in nor an executable reaches
//!  `handle_alias`.
//!  The wrapper hands real Git the alias expansion it resolved,
//!  so it has to answer this the way Git
//!  answers it,
//!  and the answer is the search Git makes:
//! its exec path prepended to `PATH`
//!  (`exec-cmd.c:333` to `:350`),
//!  every entry a directory whose `git-<word>` is a regular file with the owner execute bit
//!  (`run-command.c:191` to `:219`),
//!  and a word that holds a directory separator run from the path as it
//!  stands,
//! without any search.
//!  For the `git` a `!` alias starts,
//!  the search happens in the directory Git runs the alias in:
//!  the top of the working tree when Git found the repository by walking up to it,
//!  so a relative entry names there the directory the shell searches too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // dashedExternalSelected({ context, environment, globals, word: 'audit-external' })
//! //   => true when `git-audit-external` is executable on Git's search path
//! ```

/// Debug diagnostics.
use crate::diagnostic_log::debug;
/// The environment overlay reader the wrapper uses for every variable it consults.
use crate::child_environment::environment_value;
/// The directories a caller's `-C` options move Git through.
use crate::global_arguments::directory_changes;
/// The directory a `--exec-path=` global names,
///  read from the caller's own options.
use crate::global_arguments::exec_path_option;
/// Running real Git,
///  for the exec path only Git knows.
use crate::transaction_git::{GitContext, GitRequest, run_git};
/// `OsStr`/`OsString` hold raw native text.
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` hold raw native paths.
use std::path::{Path, PathBuf};

/// The mode bit Git's `is_executable` tests,
///  the owner's execute bit,
///  not the permission this process really has.
#[cfg(unix)]
const OWNER_EXECUTE_BIT: u32 = 0o100;

/// What:
///  Whether Git would run an executable named `git-<word>` for this command word,
///  so that no alias of
///       that word is selected.
///  `&[OsString]` borrows the caller's global options;
///  `&[(OsString, OsString)]` borrows the
///       environment the real Git will be started with;
///  `Option<&Path>` is the directory the wrapper was started in,
///  which the caller's
///       `-C` options move,
///  `Option<&Path>` is the directory Git's command word runs in,
///  those options already applied,
///  and nothing to apply them to the directory the wrapper was started in,
///  and nothing when it cannot be read.
/// Why:
///  The wrapper substitutes the expansion it resolved,
///  so it must not do that for a word Git resolves
///  to an external program instead.
///  An answer it cannot prove,
///  `true`,
///  leaves the alias to Git,
///  which then
///  makes the decision itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dashedExternalSelected({ context, environment, globals, currentDirectory, runDirectory, word }): boolean;
/// ```
pub fn dashed_external_selected(
    context: &GitContext,
    environment: &[(OsString, OsString)],
    globals: &[OsString],
    current_directory: Option<&Path>,
    run_directory: Option<&Path>,
    word: &OsStr,
) -> bool {
    // UNTESTED ON WINDOWS: the crate does not build for Windows (README, Requirements); this answer keeps the alias
    //  unchecked there rather than guessing at `PATHEXT` and the extension Git would run.
    #[cfg(not(unix))]
    {
        let _ = (context, environment, globals, current_directory, run_directory, word);
        return true;
    }
    #[cfg(unix)]
    {
        let mut candidate: OsString = OsString::from("git-");
        candidate.push(word);
        // A word with a directory separator is executed as it stands, so no PATH entry is searched for it.
        if candidate.as_encoded_bytes().contains(&b'/') {
            return is_executable_file(Path::new(candidate.as_os_str()));
        }
        let Some(directories) = search_directories(context, environment, globals, current_directory, run_directory) else {
            debug(
                "dashedExternalSelected",
                format!("cannot read the search directories, so {}", candidate.to_string_lossy()).as_str(),
            );
            return true;
        };
        for directory in directories {
            if is_executable_file(directory.join(candidate.as_os_str()).as_path()) {
                debug(
                    "dashedExternalSelected",
                    format!("{} is executable, so Git runs it", candidate.to_string_lossy()).as_str(),
                );
                return true;
            }
        }
        return false;
    }
}

/// What:
///  The directories Git searches for `git-<word>`,
///  its exec path first and then every `PATH` entry,
/// or
///       nothing when the directory Git runs in or its exec path cannot be read.
///  `Option<&Path>` is the directory Git's command word runs in,
///  the caller's directory options already applied,
///  and nothing to apply those options to the directory the wrapper was started in.
/// Why:
///  `setup_path` puts the exec path in front of `PATH` once,
///  before any command runs
///  (`exec-cmd.c:333`),
///  and `locate_in_PATH` then walks the entries in order,
///  an empty entry being the
///  current directory (`run-command.c:203`).
///  The order only matters to Git;
///  this caller asks whether any
///  entry holds the program.
///  Every entry without a leading separator is resolved against the directory Git runs in,
///  as the
///  kernel resolves it for `run-command.c:203` to `:218`,
///  so that directory is read first;
///  a relative exec path is made absolute against it as well
///  (`exec-cmd.c:325` to `exec-cmd.c:330`).
///  The directory Git runs a word in is not always the one its options reach:
///  it runs a `!` alias from the top of the working tree,
///  which `aliasRunDirectory` answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function searchDirectories({ context, environment, globals, currentDirectory, runDirectory }): string[] | undefined;
/// ```
fn search_directories(
    context: &GitContext,
    environment: &[(OsString, OsString)],
    globals: &[OsString],
    current_directory: Option<&Path>,
    run_directory: Option<&Path>,
) -> Option<Vec<PathBuf>> {
    let start: &Path = current_directory?;
    let directory: PathBuf = match run_directory {
        Some(directory) => directory.to_path_buf(),
        None => effective_directory(start, globals),
    };
    let mut directories: Vec<PathBuf> = Vec::new();
    let path: OsString = environment_value(environment, "PATH").unwrap_or_default();
    for entry in std::env::split_paths(path.as_os_str()) {
        // POSIX specifies an empty entry as the current directory, and Git reads it that way too; any other entry
        //  without a leading separator is that same relative directory, read from where Git runs.
        if entry.as_os_str().is_empty() || !entry.is_absolute() {
            directories.push(directory.join(entry));
        } else {
            directories.push(entry);
        }
    }
    let exec_path: PathBuf = exec_path(context, environment, globals, directory.as_path(), start)?;
    // An empty answer adds no executables directory at all (`exec-cmd.c:325`), so the search keeps to `PATH`.
    if !exec_path.as_os_str().is_empty() {
        directories.push(exec_path);
    }
    return Some(directories);
}

/// What:
///  The directory Git runs in after the caller's `-C` options,
///  read from the directory the wrapper was started in.
/// Why:
///  Each `-C` moves Git once,
///  in the order written,
///  and a relative value is resolved against the directory the
///       move starts from
///  (`git.c:316` to `git.c:328`);
///  `Path::join` keeps a value as written,
///  so a `..` or a link in it resolves
///  exactly as the kernel resolves it when Git changes into it.
///  A frame that already starts in a directory Git moved to,
///  the shell of a `!` alias,
///  reads its own options from there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const effectiveDirectory = (start: string, globals: string[]) =>
///   directoryChanges(globals).reduce((at, change) => join(at, change), start);
/// ```
pub fn effective_directory(start: &Path, globals: &[OsString]) -> PathBuf {
    let mut directory: PathBuf = start.to_path_buf();
    for change in directory_changes(globals) {
        directory = directory.join(change);
    }
    return directory;
}

/// What:
///  The directory Git runs an alias in:
///  the top of the working tree when Git found the repository by walking up to it,
///  and the directory the caller's `-C` options reach otherwise,
///  or nothing when the directory Git was started in cannot be read or Git cannot be asked.
///  `&[OsString]` borrows the globals of the Git process that reached the alias;
///  `Option<&Path>` is the directory that process was started in,
///  the caller's `-C` options not yet applied.
/// Why:
///  A `!` alias runs its shell after `setup_git_directory_gently`
///  (`git.c:388` to `git.c:407`),
///  which changes to the top of the working tree when discovery walked up from below that top
///  (`setup.c:1959` to `setup.c:1962`),
///  and changes to nothing when there was no repository to walk to,
///  a repository named by `--git-dir` or `GIT_DIR` included;
///  so the shell searches a relative `PATH` entry from that top and not from the
///  directory the caller's `-C` named.
///  Git writes the prefix of the directory it was started in below that top only in the
///  case that changes directory,
///  so an empty prefix answers the directory the caller's options reach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function aliasRunDirectory({ globals, start }): string | undefined {
///   const [top, prefix] = await git(start, [...globals, 'rev-parse', '--path-format=absolute', '--show-toplevel', '--show-prefix']).split('\n');
///   return prefix === '' ? effectiveDirectory(start, globals) : top;
/// }
/// ```
pub fn alias_run_directory(
    context: &GitContext,
    globals: &[OsString],
    current_directory: Option<&Path>,
) -> Option<PathBuf> {
    let start: &Path = current_directory?;
    let mut arguments: Vec<OsString> = globals.to_vec();
    arguments.push(OsString::from("rev-parse"));
    arguments.push(OsString::from("--path-format=absolute"));
    arguments.push(OsString::from("--show-toplevel"));
    arguments.push(OsString::from("--show-prefix"));
    // Git's own `-C` options in the globals apply from the directory this process was started in and nowhere else, so
    //  the question is asked there.
    let mut request: GitRequest = GitRequest::new(start, arguments.as_slice());
    request.without_prefix = true;
    let Ok(output) = run_git(context, &request) else {
        debug("aliasRunDirectory", "Git could not be asked where an alias runs");
        return None;
    };
    // A bare repository, and a repository Git could not read at all, have no top to change to, so Git runs the alias
    //  where its options left it; so does a directory outside every repository.
    if !output.succeeded() {
        debug(
            "aliasRunDirectory",
            "Git names no top of a working tree, so an alias runs where Git was started",
        );
        return Some(effective_directory(start, globals));
    }
    let mut lines = output.stdout.split(is_record_break);
    let top: Option<&[u8]> = lines.next();
    // Git writes the prefix of the directory it was started in below that top, an empty line when it was started at
    //  the top, which is the only case that changes directory.
    let prefix: Option<&[u8]> = lines.next();
    let changes_directory: bool = matches!(prefix, Some(line) if !line.is_empty());
    if !changes_directory {
        return Some(effective_directory(start, globals));
    }
    return top.map(printed_path);
}

/// What:
///  Whether a byte ends one line of what Git printed.
/// Why:
///  `rev-parse` writes one answer per line,
///  and a line is read up to its newline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isRecordBreak = (byte: number) => byte === 0x0a;
/// ```
fn is_record_break(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  A path Git printed,
///  from the bytes it printed,
///  decoded only where the platform has no other way.
/// Why:
///  The top of a working tree is arbitrary bytes on Unix,
///  and a path that round-trips exactly is the one Git changes to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(printed).toString('latin1') // exact bytes, no decoding
/// ```
fn printed_path(printed: &[u8]) -> PathBuf {
    #[cfg(unix)]
    {
        /// The Unix byte-to-native-string conversion for a printed path.
        use std::os::unix::ffi::OsStringExt;
        return PathBuf::from(OsString::from_vec(printed.to_vec()));
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return PathBuf::from(String::from_utf8_lossy(printed).into_owned());
    }
}

/// What:
///  The directory Git keeps its own commands in,
///  read from the caller's `--exec-path` option,
///  then from a nonempty `GIT_EXEC_PATH`,
///  then from Git itself,
///  or nothing when Git cannot be asked.
/// Why:
///  `--exec-path=<dir>` sets the exec path
///  (`git.c:178` to `git.c:181`),
///  and `git_exec_path` answers with that setting before it reads the environment
///  (`exec-cmd.c:313` to `exec-cmd.c:323`);
///  only when neither is set does Git answer with the path it was built with,
///  which only it can name,
///  so the answer comes from `git --exec-path` with the
///  caller's globals.
///  A relative answer
///  is made absolute against the directory Git runs in,
///  which the caller's `-C` chain moves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function execPath({ context, environment, globals, directory, start }): string | undefined;
/// ```
fn exec_path(
    context: &GitContext,
    environment: &[(OsString, OsString)],
    globals: &[OsString],
    directory: &Path,
    start: &Path,
) -> Option<PathBuf> {
    if let Some(configured) = exec_path_option(globals) {
        return Some(usable_exec_path(configured, directory));
    }
    // A caller that exported `GIT_EXEC_PATH` as an empty string names no setting, so Git answers with its own path.
    if let Some(configured) = environment_value(environment, "GIT_EXEC_PATH")
        && !configured.is_empty()
    {
        return Some(usable_exec_path(configured, directory));
    }
    return Some(usable_exec_path(printed_exec_path(context, globals, start)?, directory));
}

/// What:
///  A configured exec path as an absolute directory this reading can search.
/// Why:
///  An empty setting adds no executables directory at all
///  (`exec-cmd.c:325`),
///  and a relative one is made absolute against the directory Git runs in
///  before Git adds it to `PATH` (`exec-cmd.c:325` to `exec-cmd.c:330`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const usableExecPath = (configured: string, directory: string) =>
///   configured === '' ? '' : isAbsolute(configured) ? configured : join(directory, configured);
/// ```
fn usable_exec_path(configured: OsString, directory: &Path) -> PathBuf {
    let path: PathBuf = PathBuf::from(configured);
    if path.as_os_str().is_empty() || path.is_absolute() {
        return path;
    }
    return directory.join(path);
}

/// What:
///  The exec path Git itself reports,
///  or nothing when Git cannot be asked or answers nothing.
/// Why:
///  `--exec-path` prints the path Git uses and exits without opening a repository
///  (`git.c:181` to `:188`),
///  so asking it is the only way to read the path Git was built with.
///  The question runs in the directory the wrapper was started in,
///  because Git's own `-C` options in the
///  caller's globals apply there and nowhere else.
///  The answer ends with one newline,
///  which is the record terminator and not part of the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function printedExecPath({ context, globals, start }): string | undefined;
/// ```
fn printed_exec_path(context: &GitContext, globals: &[OsString], start: &Path) -> Option<OsString> {
    let mut arguments: Vec<OsString> = globals.to_vec();
    arguments.push(OsString::from("--exec-path"));
    // Git answers from the directory it was started in, where its own `-C` options apply; asking from any other
    //  directory would run a chain the caller never wrote.
    let mut request: GitRequest = GitRequest::new(start, arguments.as_slice());
    request.without_prefix = true;
    match run_git(context, &request) {
        Ok(output) if output.succeeded() => return Some(without_record_terminator(output.stdout.as_slice())),
        Ok(_) | Err(_) => {
            debug("printedExecPath", "Git did not report its exec path");
            return None;
        }
    }
}

/// What:
///  Printed bytes without the newline a command ends its answer with,
///  and without the byte a Windows answer puts
///       before it.
/// Why:
///  `--exec-path` writes the path with `puts`,
///  so the newline comes from the printing and not from the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.replace(/\r?\n$/, '')
/// ```
fn without_record_terminator(printed: &[u8]) -> OsString {
    let mut end: usize = printed.len();
    if printed.get(end.wrapping_sub(1)) == Some(&b'\n') {
        end -= 1;
        // A Windows answer writes the newline as a carriage return and a newline, the way `puts` does.
        if printed.get(end.wrapping_sub(1)) == Some(&b'\r') {
            end -= 1;
        }
    }
    #[cfg(unix)]
    {
        /// The Unix byte-to-native-string conversion for a printed path.
        use std::os::unix::ffi::OsStringExt;
        return OsString::from_vec(printed[..end].to_vec());
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return OsString::from(String::from_utf8_lossy(&printed[..end]).into_owned());
    }
}

/// What:
///  Whether the path names a regular file that carries the owner execute bit,
///  as Git asks before it
///       runs a command (`run-command.c:136` to `:168`).
/// Why:
///   Git follows links here and tests the bit itself,
///  not this process's right to execute the file,
///  so a program
///       only its group may run is one Git skips.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await stat(path)).isFile() && ((await stat(path)).mode & 0o100) !== 0
/// ```
#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    /// The Unix permission bits of a file.
    use std::os::unix::fs::PermissionsExt;
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    return metadata.permissions().mode() & OWNER_EXECUTE_BIT != 0;
}

/// The search and the exec path stay out of the release executable.
#[cfg(test)]
#[path = "external_git_command_tests.rs"]
mod tests;
