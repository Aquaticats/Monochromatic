//! What:
//!  The repository location that `git rev-parse --path-format=absolute --is-bare-repository --git-dir --git-common-dir
//!  --show-toplevel --show-prefix` prints, found from the file system without a Git process.
//! Why:
//!  Every wrapped command asks for it, and it is the only extra Git start of most forwarded commands.
//!  The reader walks up from the directory looking for `.git`, checks what Git checks for the plain layouts
//!  (a valid `HEAD`, the objects and refs directories, files ref storage, a known object format, the owner of every
//!  path, no `core.bare` that is true and no `core.worktree` in any scope), and returns `None` for everything else, which
//!  sends the caller to Git: a bare repository, `GIT_*` variables, a symbolic link, another file system, reftable, an
//!  unknown extension, a submodule that sets `core.worktree`.
//!  With `extensions.worktreeConfig` on, the worktree's own `config.worktree` is read too, as Git reads it
//!  (`Documentation/config/extensions.adoc`; `read_and_verify_repository_format` in `setup.c` of Git 2.56.0):
//!  its `core.bare` overrides the common file's, its entries are checked like the common file's, and its includes are
//!  scanned and never interpreted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const layout: RepoLayout | undefined = readLayout({ globalPrefix, overlay }); // undefined: ask Git
//! ```

use crate::child_environment::{COUNT_VARIABLE, environment_value, parse_config_count};
use crate::private_storage::read_regular_file;
use crate::repo_read_hook::enabled;
use crate::repo_read_config_scan::{ScopeEnvironment, discovery_key, entries_avoid_keys, outer_scopes_avoid_keys, read_following};
use crate::repo_read_config_text::{CONFIG_TEXT_LIMIT, ConfigEntry, last_entry, parse_bool, parse_config_text};
use crate::repo_read_config_typed::{entries_are_accepted, environment_is_accepted, outer_scopes_are_accepted};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The environment variables a plain read tolerates, by prefix; any other `GIT_` variable can move a path or a key.
///  The first group is the original list. The second group was added after reading what Git 2.56.0 does with each name
///  (`environment.h`, and the file named for each); none can change which configuration entry, attribute tree, repository,
///  index, object, ref or pathspec meaning an answer rests on, since each only traces, words output, reaches a network or a tool,
///  or sets a limit or a write-time choice:
///  Tracing and output: `GIT_TRACE` and every `GIT_TRACE2`, `GIT_TRACE_*` name (`trace.c` line 33, `trace2/tr2_sysenv.c`
///   lines 32 and 34), `GIT_FLUSH` (`write-or-die.c` line 25), `GIT_PROGRESS_DELAY` (`progress.c` line 286),
///   `GIT_PRINT_SHA1_ELLIPSIS` (`environment.c` line 242), `GIT_MERGE_VERBOSITY` (`merge-ort.c` line 5516), `GIT_ADVICE`
///   (`advice.c` line 139), `GIT_REFLOG_ACTION` (`sequencer.c` line 53), `GIT_CHERRY_PICK_HELP` (`sequencer.c` line 527);
///  Network and transport: `GIT_CURL_*` (`http.c` line 1494), `GIT_HTTP_*` (`http.c` lines 1475 to 1477), `GIT_SSL_*`
///   (`http.c` line 1465), `GIT_PROXY_*` (`connect.c` line 1046), `GIT_TCP_KEEP*` (`http.c` line 1507), `GIT_USER_AGENT`
///   (`version.c` line 33), `GIT_OVERRIDE_VIRTUAL_HOST` (`connect.c` line 1276), `GIT_PROTOCOL_FROM_USER` (`transport.c` line 1182);
///  Tools: `GIT_EXTERNAL_DIFF` (`diff.c` line 565), `GIT_DIFF_*` (`diff.c` line 4045, `builtin/difftool.c` lines 792 and 799),
///   `GIT_MERGETOOL_GUI` (`builtin/difftool.c` line 786), `GIT_MAN_VIEWER` (`builtin/help.c` line 521), `GIT_SHELL_PATH`
///   (`builtin/var.c` line 149);
///  Limits and write-time choices: `GIT_ALLOC_LIMIT` (`wrapper.c` line 25), `GIT_MMAP_LIMIT` (`wrapper.c` line 947),
///   `GIT_INDEX_VERSION` (`read-cache.c` line 1646, the version Git writes; the index reader takes versions 2 to 4),
///   `GIT_BASENAME_FACTOR` (`diffcore-rename.c` line 1497), `GIT_COMMIT_GRAPH_PARANOIA` (`commit-graph.c` line 1028),
///   `GIT_DISABLE_UNTRACKED_CACHE` and `GIT_FORCE_UNTRACKED_CACHE` (`dir.c` lines 2988 and 3174),
///   `GIT_AUTO_COMMENT_CHAR_CONFIG_WARNING_GIVEN` (`config.c` line 2260, only suppresses a warning).
///  Left out because they can move an answer: `GIT_DIR`, `GIT_WORK_TREE`, `GIT_COMMON_DIR`, `GIT_PREFIX`, `GIT_IMPLICIT_WORK_TREE`,
///  `GIT_INDEX_FILE`, `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, `GIT_CEILING_DIRECTORIES`,
///  `GIT_DISCOVERY_ACROSS_FILESYSTEM`, `GIT_NAMESPACE`, `GIT_CONFIG`, `GIT_CONFIG_PARAMETERS`, `GIT_ATTR_*`, `GIT_NO_REPLACE_OBJECTS`,
///  `GIT_REPLACE_REF_BASE`, `GIT_GRAFT_FILE`, `GIT_SHALLOW_FILE`, `GIT_QUARANTINE_PATH`, `GIT_REFERENCE_BACKEND`, `GIT_REF_PARANOIA`,
///  the four `GIT_*_PATHSPECS`, `GIT_TEMPLATE_DIR`, `GIT_TEXTDOMAINDIR` (the wording the synthesized failures assume),
///  `GIT_NOTES_*`, `GIT_NO_LAZY_FETCH`, `GIT_DEFAULT_*` and every `GIT_TEST_*`.
const PLAIN_ENVIRONMENT_PREFIXES: &[&str] = &[
    "GIT_AUTHOR_",
    "GIT_COMMITTER_",
    "GIT_TERMINAL_PROMPT",
    "GIT_ASKPASS",
    "GIT_PAGER",
    "GIT_EDITOR",
    "GIT_SEQUENCE_EDITOR",
    "GIT_EXEC_PATH",
    "GIT_SSH",
    "GIT_ALLOW_PROTOCOL",
    "GIT_LFS_",
    "GIT_CONFIG_NOSYSTEM",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_SYSTEM",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_KEY_",
    "GIT_CONFIG_VALUE_",
    "GIT_MERGE_AUTOEDIT",
    "GIT_OPTIONAL_LOCKS",
    "GIT_TRACE",
    "GIT_FLUSH",
    "GIT_PROGRESS_DELAY",
    "GIT_PRINT_SHA1_ELLIPSIS",
    "GIT_MERGE_VERBOSITY",
    "GIT_ADVICE",
    "GIT_REFLOG_ACTION",
    "GIT_CHERRY_PICK_HELP",
    "GIT_CURL_",
    "GIT_HTTP_",
    "GIT_SSL_",
    "GIT_PROXY_",
    "GIT_TCP_KEEP",
    "GIT_USER_AGENT",
    "GIT_OVERRIDE_VIRTUAL_HOST",
    "GIT_PROTOCOL_FROM_USER",
    "GIT_EXTERNAL_DIFF",
    "GIT_DIFF_",
    "GIT_DIFFTOOL_",
    "GIT_MERGETOOL_GUI",
    "GIT_MAN_VIEWER",
    "GIT_SHELL_PATH",
    "GIT_ALLOC_LIMIT",
    "GIT_MMAP_LIMIT",
    "GIT_INDEX_VERSION",
    "GIT_BASENAME_FACTOR",
    "GIT_COMMIT_GRAPH_PARANOIA",
    "GIT_DISABLE_UNTRACKED_CACHE",
    "GIT_FORCE_UNTRACKED_CACHE",
    "GIT_AUTO_COMMENT_CHAR_CONFIG_WARNING_GIVEN",
];

/// The prefix of the variables this package owns (`GIT_POLICY_NATIVE_IMAGE_TAG` keeps the images of parallel runs apart);
///  Git defines none of them, so they cannot move a path or a key.
pub const WRAPPER_ENVIRONMENT_PREFIX: &str = "GIT_POLICY_";

/// The per-worktree configuration file in a worktree's own Git directory.
const WORKTREE_CONFIG_NAME: &str = "config.worktree";

/// The longest `HEAD` or `commondir` or gitfile this reader looks at.
///  Exempt from a citation under `LQM` (class: reader fallback): a file, chain or count past this bound makes the reader
///  decline and real Git answer, so a Git start is added and no result changes. Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`, "Exempt limits"; the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
const SMALL_FILE_LIMIT: u64 = 4096;

/// How many environment-supplied configuration entries the reader inspects before it declines.
///  Exempt from a citation under `LQM` (class: reader fallback): a file, chain or count past this bound makes the reader
///  decline and real Git answer, so a Git start is added and no result changes. Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`, "Exempt limits"; the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
const ENV_CONFIG_ENTRY_LIMIT: u64 = 1024;

/// The hash algorithms of object names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectFormat {
    /// 20-byte names, 40 hexadecimal digits.
    Sha1,
    /// 32-byte names, 64 hexadecimal digits.
    Sha256,
}

impl ObjectFormat {
    /// The name `rev-parse --show-object-format` prints.
    pub fn name(self) -> &'static str {
        return match self {
            ObjectFormat::Sha1 => "sha1",
            ObjectFormat::Sha256 => "sha256",
        };
    }

    /// The number of hexadecimal digits of one object name.
    pub fn hex_length(self) -> usize {
        return match self {
            ObjectFormat::Sha1 => 40,
            ObjectFormat::Sha256 => 64,
        };
    }
}

/// What:
///  Where one worktree and its repository are, as Git reports them with absolute paths.
/// Why:
///   One value answers the location, the Git paths and the object format queries of a command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RepoLayout = { gitDir: string; commonDir: string; worktreeRoot: string; prefix: Uint8Array; objectFormat: 'sha1' | 'sha256' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepoLayout {
    /// The real path of the worktree's Git directory.
    pub git_dir: PathBuf,
    /// The real path of the directory that holds the objects and the shared refs.
    pub common_dir: PathBuf,
    /// The real path of the worktree's top level.
    pub worktree_root: PathBuf,
    /// The start directory below the top level, with a trailing `/`, or empty at the top level.
    pub prefix: Vec<u8>,
    /// The object hash of the repository.
    pub object_format: ObjectFormat,
}

/// The keys an environment-supplied configuration entry may not set without the reader declining.
fn env_key_moves_discovery(key: &str) -> bool {
    let lowered: String = key.to_ascii_lowercase();
    return lowered == "core.bare" || lowered == "core.worktree" || lowered.starts_with("include");
}

/// Whether the `GIT_CONFIG_COUNT` entries of `variables` (the wrapper injects some) all leave discovery alone.
fn env_config_is_plain(variables: &[(OsString, OsString)]) -> bool {
    return env_config_avoids(variables, &env_key_moves_discovery);
}

/// What:
///  Whether every environment-supplied configuration entry has a key `declines` does not name;
///  a malformed set of entries is not plain either.
/// Why:
///   The content queries use the same walk with their own key list, so an injected `diff.orderFile` is not missed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// envConfigKeys(variables).every((key) => !declines(key)); // false when malformed
/// ```
pub fn env_config_avoids(variables: &[(OsString, OsString)], declines: &dyn Fn(&str) -> bool) -> bool {
    let Some(count_text) = environment_value(variables, COUNT_VARIABLE) else {
        return true;
    };
    let Some(count) = parse_config_count(count_text.as_os_str()) else {
        return false;
    };
    if count > ENV_CONFIG_ENTRY_LIMIT {
        return false;
    }
    for index in 0..count {
        let Some(key) = environment_value(variables, format!("GIT_CONFIG_KEY_{index}").as_str()) else {
            return false;
        };
        match key.to_str() {
            Some(text) if !declines(text) => {}
            _ => return false,
        }
    }
    return true;
}

/// Whether every `GIT_` variable among `variables` is one that cannot move discovery:
///  a tolerated prefix, or one of this package's own.
pub(crate) fn environment_is_plain(variables: &[(OsString, OsString)]) -> bool {
    for (name, _) in variables {
        let Some(text) = name.to_str() else {
            continue;
        };
        let tolerated: bool = text.starts_with(WRAPPER_ENVIRONMENT_PREFIX) || PLAIN_ENVIRONMENT_PREFIXES.iter().any(|prefix| { return text.starts_with(prefix); });
        if text.starts_with("GIT_") && !tolerated {
            return false;
        }
    }
    return env_config_is_plain(variables);
}

/// The environment Git would see: the process environment with the child overlay applied over it.
pub(crate) fn effective_environment(overlay: &[(OsString, OsString)]) -> Vec<(OsString, OsString)> {
    let mut variables: Vec<(OsString, OsString)> = std::env::vars_os()
        .filter(|(name, _)| { return !overlay.iter().any(|(added, _)| { return added == name; }); })
        .collect();
    variables.extend(overlay.iter().cloned());
    return variables;
}

/// The directory a command starts in: the process directory moved by leading `-C <path>` options.
///  Any other global option returns `None`.
pub(crate) fn start_directory(global_prefix: &[OsString]) -> Option<PathBuf> {
    let mut directory: PathBuf = std::env::current_dir().ok()?;
    let mut options = global_prefix.iter();
    while let Some(option) = options.next() {
        if option != "-C" {
            return None;
        }
        let path: &OsString = options.next()?;
        if !path.is_empty() {
            directory = directory.join(path);
        }
        // Git changes directory at each `-C` in turn and fails at the first one that is not a directory.
        if !std::fs::metadata(&directory).is_ok_and(|found| { return found.is_dir(); }) {
            return None;
        }
    }
    return std::fs::canonicalize(directory).ok();
}

/// A small regular file's text, refusing links.
fn small_file(path: &Path) -> Option<String> {
    let bytes: Vec<u8> = read_regular_file(path, SMALL_FILE_LIMIT).ok()?;
    return String::from_utf8(bytes).ok();
}

/// Whether `directory` holds the three entries Git takes for a Git directory.
fn has_git_directory_entries(directory: &Path) -> bool {
    return directory.join("HEAD").exists() && directory.join("objects").is_dir() && directory.join("refs").is_dir();
}

/// Whether the `HEAD` text is one of the two spellings Git accepts that this reader knows.
fn head_is_valid(text: &str, format: ObjectFormat) -> bool {
    let Some(line) = text.strip_suffix('\n') else {
        return false;
    };
    if let Some(name) = line.strip_prefix("ref: ") {
        return name.starts_with("refs/") && !name.contains('\n');
    }
    return line.len() == format.hex_length() && line.bytes().all(|byte| { return byte.is_ascii_hexdigit(); });
}

/// Whether `path` is owned by the effective user, as Git's ownership check requires.
#[cfg(unix)]
fn owned_by_current_user(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    // SAFETY: `geteuid` takes no argument, touches no memory and cannot fail.
    return metadata.uid() == unsafe { libc::geteuid() };
}

/// Without user ids, every repository is left to Git.
#[cfg(not(unix))]
fn owned_by_current_user(_path: &Path) -> bool {
    return false;
}

/// The device a directory lives on, so a walk can stop where Git stops.
#[cfg(unix)]
fn device_of(path: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    return std::fs::metadata(path).ok().map(|metadata| { return metadata.dev(); });
}

/// Without device numbers, every walk is left to Git.
#[cfg(not(unix))]
fn device_of(_path: &Path) -> Option<u64> {
    return None;
}

/// The Git directory a `.git` entry names: the entry itself when it is a directory,
///  the `gitdir:` line of a plain file, `None` for a link or anything else.
pub fn dot_git_target(worktree: &Path) -> Option<Option<(PathBuf, bool)>> {
    let dot_git: PathBuf = worktree.join(".git");
    let metadata = match std::fs::symlink_metadata(&dot_git) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Some(None),
        Err(_) => return None,
    };
    if metadata.is_dir() {
        return Some(Some((dot_git, false)));
    }
    if !metadata.is_file() {
        return None;
    }
    let text: String = small_file(dot_git.as_path())?;
    // Git strips every trailing carriage return and line feed (`read_gitfile_gently` in `setup.c`).
    let target: &str = text.strip_prefix("gitdir: ")?.trim_end_matches(['\r', '\n']);
    if target.is_empty() || target.contains('\n') {
        return None;
    }
    return Some(Some((worktree.join(target), true)));
}

/// What:
///  The repository format keys the layout needs from the repository's own configuration file:
///  the object format, and whether `extensions.worktreeConfig` turns the per-worktree file on.
/// Why:
///   The object format decides the length of a name; any other extension may change how the repository is read.
///  Git respects `extensions.worktreeConfig` at every `core.repositoryFormatVersion`
///  (`Documentation/config/extensions.adoc`, `handle_extension_v0` in `setup.c`); the last entry decides, as a boolean.
///  A repository whose configuration names no `core.repositoryformatversion` has no format at all to Git, so the
///  extension is off there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const format: { objectFormat: ObjectFormat; worktreeConfig: boolean } | undefined = readFormat(entries); // undefined: ask Git
/// ```
fn format_of(entries: &[ConfigEntry]) -> Option<(ObjectFormat, bool)> {
    // A bare `core.repositoryformatversion` is an error in Git, not an absent key.
    let version: Option<&[u8]> = match last_entry(entries, "core.repositoryformatversion") {
        Some(entry) => Some(entry.value.as_deref()?),
        None => None,
    };
    let version_one: bool = match version {
        None | Some(b"0") => false,
        Some(b"1") => true,
        Some(_) => return None,
    };
    let mut format: ObjectFormat = ObjectFormat::Sha1;
    let mut worktree_config: bool = false;
    for entry in entries.iter().filter(|entry| { return entry.key.starts_with("extensions."); }) {
        if entry.key == "extensions.worktreeconfig" {
            worktree_config = parse_bool(entry.value.as_deref())?;
            continue;
        }
        if !version_one {
            return None;
        }
        let value: &[u8] = entry.value.as_deref().unwrap_or_default();
        match (entry.key.as_str(), value) {
            ("extensions.noop", _) => {}
            ("extensions.preciousobjects", _) if parse_bool(entry.value.as_deref()).is_some() => {}
            ("extensions.objectformat", b"sha1") => format = ObjectFormat::Sha1,
            ("extensions.objectformat", b"sha256") => format = ObjectFormat::Sha256,
            ("extensions.refstorage", b"files") => {}
            _ => return None,
        }
    }
    // Git clears the whole format when no version was found (`read_repository_format` in `setup.c`), the extension with
    // it, so an absent version never turns the per-worktree file on.
    return Some((format, worktree_config && version.is_some()));
}

/// What:
///  The entries of the worktree's own `config.worktree`, or none when the extension is off or the file is absent;
///  `None` when the file cannot be read or parsed.
/// Why:
///   Git reads `<git dir>/config.worktree` (the worktree's own Git directory, not the common one) after the common
///  file, and only when `extensions.worktreeConfig` is on; a missing file is an empty file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entries = enabled ? parse(await readIfExists(join(gitDir, 'config.worktree'))) : []; // undefined: ask Git
/// ```
fn worktree_entries(git_dir: &Path, enabled: bool) -> Option<Vec<ConfigEntry>> {
    if !enabled {
        return Some(Vec::new());
    }
    return match read_following(git_dir.join(WORKTREE_CONFIG_NAME).as_path())? {
        Some(bytes) => parse_config_text(bytes.as_slice()),
        None => Some(Vec::new()),
    };
}

/// What:
///  The entries with every direct `core.bare` removed when the last one leaves the repository non-bare;
///  `None` when it makes the repository bare or any of them is not a boolean.
/// Why:
///   Git takes the last `core.bare` of the common file then the per-worktree file (`read_worktree_config` in
///  `setup.c`); a bare repository, an unreadable boolean and a `core.worktree` are left to Git by the key scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = lastBare(files) === true ? undefined : files.map((entries) => entries.filter((e) => e.key !== 'core.bare'));
/// ```
fn without_false_bare(files: [&[ConfigEntry]; 2]) -> Option<[Vec<ConfigEntry>; 2]> {
    let mut last: Option<bool> = None;
    for entry in files.iter().flat_map(|entries| { return entries.iter(); }).filter(|entry| { return entry.key == "core.bare"; }) {
        last = Some(parse_bool(entry.value.as_deref())?);
    }
    if last == Some(true) {
        return None;
    }
    return Some(files.map(|entries| {
        return entries.iter().filter(|entry| { return entry.key != "core.bare"; }).cloned().collect::<Vec<ConfigEntry>>();
    }));
}

/// What:
///  Check the discovered Git directory and build the layout.
/// Why:
///   Everything beyond the plain layout returns `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finish(start: string, worktree: string, gitDir: string, fromFile: boolean): RepoLayout | undefined;
/// ```
fn finish(start: &Path, worktree: &Path, entry: (PathBuf, bool), scope: &ScopeEnvironment, typed: bool) -> Option<RepoLayout> {
    let (target, from_file) = entry;
    let git_dir: PathBuf = std::fs::canonicalize(target).ok()?;
    let dot_git: PathBuf = worktree.join(".git");
    let owners_ok: bool = owned_by_current_user(worktree)
        && owned_by_current_user(git_dir.as_path())
        && (!from_file || owned_by_current_user(dot_git.as_path()));
    if !owners_ok {
        return None;
    }
    let common_dir: PathBuf = match small_file(git_dir.join("commondir").as_path()) {
        // Git strips every trailing carriage return and line feed here too (`get_common_dir_noenv` in `path.c`).
        Some(text) => std::fs::canonicalize(git_dir.join(text.trim_end_matches(['\r', '\n']))).ok()?,
        None if git_dir.join("commondir").exists() => return None,
        None => git_dir.clone(),
    };
    if git_dir.join("reftable").exists() || common_dir.join("reftable").exists() || !has_git_directory_entries_split(&git_dir, &common_dir) {
        return None;
    }
    let config_bytes: Vec<u8> = read_regular_file(common_dir.join("config").as_path(), CONFIG_TEXT_LIMIT as u64).ok()?;
    let entries: Vec<ConfigEntry> = parse_config_text(config_bytes.as_slice())?;
    let (object_format, worktree_config): (ObjectFormat, bool) = format_of(entries.as_slice())?;
    let worktree_path: PathBuf = git_dir.join(WORKTREE_CONFIG_NAME);
    let per_worktree: Vec<ConfigEntry> = worktree_entries(git_dir.as_path(), worktree_config)?;

    // The repository's own file carries `bare = false` after `init`; that spelling is the plain layout. With the
    // per-worktree file on, the last `core.bare` of the two files decides, and a `true` there is a bare repository.
    let [discoverable, worktree_discoverable] = without_false_bare([entries.as_slice(), per_worktree.as_slice()])?;
    let local_ok: bool = entries_avoid_keys(discoverable.as_slice(), common_dir.join("config").as_path(), scope.home.as_deref(), 0, &discovery_key)?
        && entries_avoid_keys(worktree_discoverable.as_slice(), worktree_path.as_path(), scope.home.as_deref(), 0, &discovery_key)?;
    if !local_ok || !outer_scopes_avoid_keys(scope, &discovery_key)? {
        return None;
    }
    // Git types many keys for every command and fails, or warns, on a value that does not fit; the readers leave
    // such a repository to Git, which words its own failure.
    // The callers that ask for no typed proof need the structure of the layout and no diagnostic about values:
    //  they do not rely on an earlier command having validated every value (see `read_layout_after_success`).
    let typed_ok: bool = !typed
        || (entries_are_accepted(entries.as_slice(), common_dir.join("config").as_path(), scope.home.as_deref(), 0)?
            && entries_are_accepted(per_worktree.as_slice(), worktree_path.as_path(), scope.home.as_deref(), 0)?
            && outer_scopes_are_accepted(scope)?);
    if !typed_ok {
        return None;
    }
    let head: String = small_file(git_dir.join("HEAD").as_path())?;
    if !head_is_valid(head.as_str(), object_format) {
        return None;
    }
    let relative: &Path = start.strip_prefix(worktree).ok()?;
    let mut prefix: Vec<u8> = Vec::new();
    for component in relative.components() {
        prefix.extend_from_slice(component.as_os_str().as_encoded_bytes());
        prefix.push(b'/');
    }
    return Some(RepoLayout { git_dir, common_dir, worktree_root: worktree.to_path_buf(), prefix, object_format });
}

/// The three-entry check, with `HEAD` in the worktree's directory and the rest in the common directory.
fn has_git_directory_entries_split(git_dir: &Path, common_dir: &Path) -> bool {
    return git_dir.join("HEAD").exists() && common_dir.join("objects").is_dir() && common_dir.join("refs").is_dir();
}

/// What:
///  The layout of the repository a command run with `global_prefix` and `overlay` starts in,
///  or `None` when Git must be asked.
/// Why:
///   This is the one entry point the callers use; it never errors, because "cannot tell" and "not a repository" both
///  mean "ask Git", which words the failure itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = readLayout(globalPrefix, overlay); // undefined: ask Git
/// ```
pub fn read_layout(global_prefix: &[OsString], overlay: &[(OsString, OsString)]) -> Option<RepoLayout> {
    if !enabled(overlay) {
        return None;
    }
    let variables: Vec<(OsString, OsString)> = effective_environment(overlay);
    if !environment_is_plain(variables.as_slice()) || !environment_is_accepted(variables.as_slice()) {
        return None;
    }
    let start: PathBuf = start_directory(global_prefix)?;
    return walk_up(start.as_path(), &ScopeEnvironment::from_variables(variables.as_slice()), true);
}

/// What:
///  The read with its environment injected: the variables Git would see and the configuration scope facts.
/// Why:
///   The controls run in a process whose own environment and home they must not depend on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = readLayoutWith(start, variables, scope); // undefined: ask Git
/// ```
pub fn read_layout_with(start: &Path, variables: &[(OsString, OsString)], scope: &ScopeEnvironment) -> Option<RepoLayout> {
    if !environment_is_plain(variables) || !environment_is_accepted(variables) {
        return None;
    }
    return walk_up(std::fs::canonicalize(start).ok()?.as_path(), scope, true);
}

/// What:
///  The same read for a command run in `directory` without global options.
/// Why:
///   The transaction Git requests name their directory and leave the invocation's global options out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = readLayoutIn(directory, overlay); // undefined: ask Git
/// ```
pub fn read_layout_in(directory: &Path, overlay: &[(OsString, OsString)]) -> Option<RepoLayout> {
    if !enabled(overlay) {
        return None;
    }
    let variables: Vec<(OsString, OsString)> = effective_environment(overlay);
    return read_layout_with(directory, variables.as_slice(), &ScopeEnvironment::from_variables(variables.as_slice()));
}

/// What:
///  The layout for a caller that follows a Git command that exited with status 0 and needs the structure of the repository
///  but not a diagnostic about its configuration values.
/// Why:
///   `read_layout_in` leaves a repository to Git when any configuration value is one Git types for every command and the
///  reader cannot prove it accepts (a key such as `diff.dirstat` is one).
/// That proof guards the wording of a failure Git would give for a malformed value,
/// and a caller that follows a command that succeeded,
/// or that needs no valid configuration,
/// has none to give.
/// This is not a certificate that every value is valid:
/// the files are read again,
/// not the snapshot the earlier command saw,
/// and a successful command types only the keys it consults.
/// The structure checks (owner,
/// object format,
///  `core.bare`,
/// `core.worktree`,
/// reftable,
/// `HEAD`) still apply,
/// because they decide whether the files mean what the reader takes them to mean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = readLayoutAfterSuccess(directory, overlay); // undefined: the files are not a plain layout
/// ```
pub fn read_layout_after_success(directory: &Path, overlay: &[(OsString, OsString)]) -> Option<RepoLayout> {
    if !enabled(overlay) {
        return None;
    }
    let variables: Vec<(OsString, OsString)> = effective_environment(overlay);
    if !environment_is_plain(variables.as_slice()) {
        return None;
    }
    return walk_up(std::fs::canonicalize(directory).ok()?.as_path(), &ScopeEnvironment::from_variables(variables.as_slice()), false);
}

/// What:
///  The layout for a caller that follows a successful command and holds the environment and the configuration scope itself.
/// Why:
///   The merges from objects read `attr.tree` with the variables their Git children get, which the process environment alone does
///  not say (the service adds an overlay); the controls give a hermetic scope the same way `read_layout_with` takes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = readLayoutAfterSuccessWith(start, variables, scope); // undefined: ask Git
/// ```
pub fn read_layout_after_success_with(start: &Path, variables: &[(OsString, OsString)], scope: &ScopeEnvironment) -> Option<RepoLayout> {
    if !environment_is_plain(variables) {
        return None;
    }
    return walk_up(std::fs::canonicalize(start).ok()?.as_path(), scope, false);
}

/// What:
///  The layout for a caller that runs after a Git command succeeded that was given the same global options and environment,
///  for a command that starts where `global_prefix` says.
/// Why:
///   The two query-driven LFS pushes plan their upload through Git queries first,
/// and read the layout only once those queries exited with status 0
/// (an interactive push may hold settings read earlier),
/// so the diagnostic the typed proof guards has been Git's to give;
/// the files are read again,
/// and a plan that cannot be narrowed is still pushed by `git-lfs` and Git.
/// They take `-C` options like `read_layout`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = readLayoutAfterSuccessFor(globalPrefix, overlay); // undefined: the files are not a plain layout
/// ```
pub fn read_layout_after_success_for(global_prefix: &[OsString], overlay: &[(OsString, OsString)]) -> Option<RepoLayout> {
    if !enabled(overlay) {
        return None;
    }
    let variables: Vec<(OsString, OsString)> = effective_environment(overlay);
    if !environment_is_plain(variables.as_slice()) {
        return None;
    }
    let start: PathBuf = start_directory(global_prefix)?;
    return walk_up(start.as_path(), &ScopeEnvironment::from_variables(variables.as_slice()), false);
}

/// Walk from `start` to the root looking for the first `.git`, as Git does, and build the layout there.
pub(crate) fn walk_up(start: &Path, scope: &ScopeEnvironment, typed: bool) -> Option<RepoLayout> {
    let mut directory: PathBuf = start.to_path_buf();
    loop {
        if let Some(entry) = dot_git_target(directory.as_path())? {
            return finish(start, directory.as_path(), entry, scope, typed);
        }
        if has_git_directory_entries(directory.as_path()) {
            return None;
        }
        let parent: PathBuf = directory.parent()?.to_path_buf();
        if device_of(parent.as_path())? != device_of(directory.as_path())? {
            return None;
        }
        directory = parent;
    }
}

/// What:
///  The absolute path `git rev-parse --path-format=absolute --git-path <name>` prints for the names this reader knows
///  in a repository whose worktree owns its Git directory.
/// Why:
///   A linked worktree splits names between its own directory and the common one by a table in Git;
///  those answers stay with Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitPath = (layout, name) => layout.gitDir === layout.commonDir && KNOWN.has(name) ? join(layout.gitDir, name) : undefined;
/// ```
pub fn git_path(layout: &RepoLayout, name: &str) -> Option<PathBuf> {
    const KNOWN: [&str; 9] = [
        "index",
        "objects",
        "HEAD",
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "cli-git-transactions",
        "packed-refs",
        "config",
    ];
    if layout.git_dir != layout.common_dir || !KNOWN.contains(&name) {
        return None;
    }
    let path: PathBuf = layout.git_dir.join(name);
    // With `--path-format=absolute` Git prints the target of a link, so a linked name is left to Git.
    if std::fs::symlink_metadata(path.as_path()).is_ok_and(|metadata| { return metadata.file_type().is_symlink(); }) {
        return None;
    }
    return Some(path);
}

#[cfg(test)]
#[path = "repo_read_layout_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "repo_read_layout_worktree_tests.rs"]
mod worktree_tests;
