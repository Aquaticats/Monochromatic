//! What:
//!  Creates the shadow repository of one published transaction:
//!       an empty bare repository,
//!       its config and alternates,
//!       the linked shared entries,
//!       the conclusion state,
//!       and the ref snapshot.
//! Why:
//!  The shadow is a separate repository rather than a registered worktree,
//!  so `git worktree list` never shows it and preparation creates no ref in the real repository.
//!  Its object store receives the objects native `git commit` writes,
//!  and its alternates name the real object directory,
//!  so a real `git prune` cannot delete a pending commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await createShadowRepository({ gitPath, cwd, capture, transactionId, transactionDirectory });
//! ```

/// Shadow path,
/// redirect variables and the shadow request.
use crate::shadow_git::{REPOSITORY_REDIRECT_VARIABLES, shadow_repository_path};
/// Invocation capture.
use crate::capture_invocation::{InvocationCapture, InvocationLayout};
/// What a tree must be for this invocation,
/// which is the question a claim and a hand-out both ask.
use crate::shadow_pool_member::refusal;
/// The in-process entries of an empty bare repository.
use crate::shadow_skeleton::create_shadow_skeleton;
/// Format copy and the config bytes.
use crate::shadow_config::{RepositoryFormat, format_shadow_config, read_repository_format, repository_format_from_snapshot};
/// Entry links.
use crate::shadow_links::link_shadow_entries;
/// Conclusion copy and ref snapshot.
use crate::conclusion_copy::copy_conclusion_state;
use crate::shadow_refs::SnapshotSources;
use crate::shadow_ref_write::write_shadow_refs;
/// Private directory creation and file writes.
use crate::private_storage::create_private_directory;
/// Where Git's words go now that the `init` carries no `--quiet`.
use crate::inner_git_words::record_words;
/// Process context and one required Git start.
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, checked_from, refused_by_the_system, run_git};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// Strings for the redirect variable list.
use std::ffi::OsString;
/// The build phase's stretches,
/// which the creation marks as it goes.
use crate::commit_build_part_durations::BuildPartClock;
/// The bytes of a path,
/// for a file that names one.
use crate::path_bytes::path_bytes_of;

/// What:
///  Overwrites a file the shadow's `git init` already created,
/// with private mode.
/// Why:
///   The incumbent's `writeFile` replaces the template `config` and `alternates`,
///  while the exclusive writer used for new files would refuse them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(path, bytes, { mode: 0o600 });
/// ```
fn overwrite_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, bytes)?;
    return set_private_file(path);
}

/// What:
///  Gives a file the private file mode on Unix.
/// Why:
///   The shadow config names the worktree root,
/// includes the real common config and switches
///  every hook off;
/// it stays private to the account.
///  It names no hooks directory:
/// every shadow command passes its own `core.hooksPath`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await chmod(path, 0o600);
/// ```
#[cfg(unix)]
fn set_private_file(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    return std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

/// Windows has no mode bits for this file;
/// the incumbent's ACL work is not ported.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
fn set_private_file(_path: &Path) -> std::io::Result<()> {
    return Ok(());
}

/// What:
///  Creates the shadow's parent directories as private directories.
/// Why:
///   The incumbent creates `monochromatic-git-policy-cli/shadow` with mode 0700.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(dirname(shadowPath), { recursive: true, mode: 0o700 });
/// ```
fn create_parent(shadow: &Path) -> std::io::Result<()> {
    let parent: &Path = shadow.parent().unwrap_or(shadow);
    if let Some(grandparent) = parent.parent() {
        ensure_private_directory(grandparent)?;
    }
    return ensure_private_directory(parent);
}

/// What:
///  Create a private directory unless another process made it first.
/// Why:
///   Concurrent commits of one repository race to create the shared `monochromatic-git-policy-cli/shadow` parents;
/// whoever loses has
///   nothing left to do.
/// A path that exists as anything but a directory is still a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(directory, { mode: 0o700 }).catch((error) => { if (error.code !== 'EEXIST') throw error; });
/// ```
pub fn ensure_private_directory(directory: &Path) -> std::io::Result<()> {
    match create_private_directory(directory) {
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && directory.is_dir() => return Ok(()),
        other => return other,
    }
}

/// How many times the shadow's `git init` is started,
/// each in a fresh directory,
/// when a start is cut at its ceiling or refused by the
///  system.
/// Two:
/// the start and one more,
/// the same "redone once" the quiet-window rule reads (`doc/planning/cli-git-shipped-limits-audit.md`,
///  pick 4,
/// the coordinator's reading,
/// open to veto);
/// a shadow whose `init` is cut twice fails its transaction.
pub(crate) const SHADOW_CREATION_ATTEMPTS: u32 = 2;

/// What:
///  Creates an empty bare repository with `git init`,
/// for the ref backend whose tables this crate does not write.
/// Why:
///   Reftable tables carry their own format,
/// so only Git creates them.
///  The pathname handed to Git is the shadow's own bytes:
/// a spelling of it writes every byte that is not UTF-8 as U+FFFD,
/// and Git then initializes a different directory,
///  so the creation protects a path that does not exist and the commit cannot be prepared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGit({ args: ['init', '--bare', '--template=', `--object-format=${format}`, '--ref-format=reftable', shadowBytes] });
/// ```
fn init_with_git(
    context: &GitContext,
    template: &ShadowTemplate,
    shadow: &Path,
) -> Result<(), GitFailure> {
    let arguments: Vec<OsString> = vec![
        OsString::from("init"),
        OsString::from("--bare"),
        OsString::from("--template="),
        OsString::from(format!("--object-format={}", template.object_format)),
        OsString::from("--ref-format=reftable"),
        shadow.as_os_str().to_os_string(),
    ];
    let mut init: GitRequest = GitRequest::new(template.common_dir.as_path(), &arguments);
    init.without_prefix = true;
    init.unset = REPOSITORY_REDIRECT_VARIABLES.iter().copied().map(OsString::from).collect();
    let mut attempt: u32 = 1;
    loop {
        let outcome = run_git(context, &init);
        // A start ended at its ceiling, or refused a process or a thread, leaves a directory that a second `init` in place does not bring to
        //  the state of an uncut run (the cut experiment: 50 of 104 second runs differed in the reftable tables). It is discarded and the
        //  start is made again in a fresh directory, which the experiment found equal to an uncut run (104 of 104).
        let cut: bool = start_was_cut_or_refused(&outcome);
        if cut && attempt < SHADOW_CREATION_ATTEMPTS {
            let note: String = format!("the shadow's init was cut or refused (attempt {attempt} of {SHADOW_CREATION_ATTEMPTS}); its directory {} is discarded and the step starts again in a fresh one", shadow.display());
            crate::client_journal::record_entry(crate::journald_native::Priority::Warning, note.as_str(), &[]);
            discard_partial_shadow(shadow)?;
            attempt += 1;
            continue;
        }
        if cut {
            discard_partial_shadow(shadow)?;
        }
        let made = checked_from(outcome, init.arguments.as_slice())?;
        // The `init` carries no `--quiet`; what Git said of it is kept in the journal.
        record_words(&["init", "--bare", "--template=", "--ref-format=reftable"], &made);
        return Ok(());
    }
}

/// What:
///  Whether a start of `init` was ended at its ceiling or refused a process or a thread.
/// Why:
///   Only these leave a partial directory that a fresh start replaces.
/// An I/O error of another kind (no such program,
/// no permission)
///  is not retried:
/// a fresh start would fail the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outcome.error?.kind === 'timed-out' || outcome.output?.stderr.includes('Resource temporarily unavailable')
/// ```
fn start_was_cut_or_refused(outcome: &std::io::Result<GitOutput>) -> bool {
    return match outcome {
        // `WouldBlock` is `EAGAIN`: the system refused the process or the thread before the command ran.
        Err(error) => matches!(error.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock),
        Ok(finished) => refused_by_the_system(finished),
    };
}

/// What:
///  Removes what a cut `git init` left at the shadow's path.
/// Why:
///   A shadow is private to its transaction and holds nothing but what `init` made,
/// so a partial one is discarded whole and never
///  completed in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(shadow, { recursive: true, force: true });
/// ```
fn discard_partial_shadow(shadow: &Path) -> Result<(), GitFailure> {
    return match std::fs::remove_dir_all(shadow) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(GitFailure(format!("the partial shadow {} cannot be discarded: {error}", shadow.display()))),
    };
}

/// What:
///  Turn the result of a filesystem step on the shadow into a Git failure that names the verb and the shadow:
///  `<verb> <shadow> failed: <error>`.
///  `std::io::Result<T>` is "the value,
/// or an input/output error".
/// Why:
///   Several steps of the creation report their failure this way,
/// and a named function stands where a closure that built
///  that text stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stepFailure<T>(attempt: () => T, verb: string, shadow: string): T; // throws GitFailure(`${verb} ${shadow} failed: ${error}`)
/// ```
fn step_failure<T>(result: std::io::Result<T>, verb: &str, shadow: &Path) -> Result<T, GitFailure> {
    return match result {
        Ok(value) => Ok(value),
        Err(error) => Err(GitFailure(format!("{verb} {} failed: {error}", shadow.display()))),
    };
}

/// The same for a step that is named by a phrase and not by a verb and the shadow:
/// `<phrase> failed: <error>`.
fn named_step_failure<T>(result: std::io::Result<T>, phrase: &str) -> Result<T, GitFailure> {
    return match result {
        Ok(value) => Ok(value),
        Err(error) => Err(GitFailure(format!("{phrase} failed: {error}"))),
    };
}

/// What:
///  The facts a shadow repository is made from,
///  resolved once so a member made ahead and a creation made here cannot disagree.
/// Why:
///   The creation writes the repository format,
/// the common directory,
/// the worktree root,
/// the object directory and the ref
/// backend into the shadow;
///  the pool's fill and the pool's claim compare those same facts,
/// so a member is only ever taken by a commit that would have written it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShadowTemplate = { commonDir; gitDir; repositoryRoot; objectDirectory; objectFormat; refFormat; format };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShadowTemplate {
    /// The repository's common Git directory,
    /// whose `config` the shadow includes.
    pub common_dir: PathBuf,
    /// The worktree's own Git directory,
    /// whose per-worktree state is copied.
    pub git_dir: PathBuf,
    /// The worktree root the shadow's `core.worktree` names.
    pub repository_root: PathBuf,
    /// The real object directory the shadow's alternates name.
    pub object_directory: PathBuf,
    /// The object hash algorithm the shadow is made with.
    pub object_format: String,
    /// The ref storage backend of the repository.
    pub ref_format: crate::transaction_journal::RefFormat,
    /// The format facts the shadow's `config` copies from the real common config.
    pub format: RepositoryFormat,
}

impl ShadowTemplate {
    /// What:
    ///  The facts of one invocation.
    /// Why:
    ///   The format facts come from the configuration snapshot the invocation already read when that read proves them,
    /// and from the common config file alone otherwise.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// ShadowTemplate.of(context, layout)
    /// ```
    pub fn of(context: &GitContext, layout: &InvocationLayout) -> Result<ShadowTemplate, GitFailure> {
        let format: RepositoryFormat = match layout.config_snapshot.as_ref().and_then(repository_format_from_snapshot) {
            Some(from_snapshot) => from_snapshot,
            None => read_repository_format(context, layout.common_dir.as_path())?,
        };
        return Ok(ShadowTemplate {
            common_dir: layout.common_dir.clone(),
            git_dir: layout.git_dir.clone(),
            repository_root: layout.repository_root.clone(),
            object_directory: layout.object_directory.clone(),
            object_format: layout.object_format.clone(),
            ref_format: layout.ref_format,
            format,
        });
    }

    /// What:
    ///  The bytes of the shadow's `config` for these facts.
    /// Why:
    ///   The creation writes it and a claim compares a member's own file with it,
    /// so one function says what the file holds;
    ///  the included path and the worktree it names are the bytes the layout holds,
    ///  because a spelling of either loses a name that is not UTF-8.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// template.configBytes()
    /// ```
    pub fn config_bytes(&self) -> Vec<u8> {
        return format_shadow_config(&self.format, self.common_dir.as_path(), self.repository_root.as_path());
    }

    /// What:
    ///  The bytes of the shadow's `objects/info/alternates` for these facts.
    /// Why:
    ///   The alternates are what lets the shadow's Git read the real object store,
    ///  and a claim checks that a member names this invocation's object directory;
    ///  the object directory's own bytes name it,
    ///  because a spelling of it loses a name that is not UTF-8 and would send the shadow to a store that does not exist.
    ///  A pathname a line would break is written C-quoted,
    /// which Git reads back exactly:
    ///  `parse_alternates` in `odb.c` hands a line that begins with a double quote to
    ///  `unquote_c_style` in `quote.c` of Git 2.56.0.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Buffer.concat([alternatesLine(pathBytes), Buffer.from('\n')])
    /// ```
    pub fn alternates_bytes(&self) -> Vec<u8> {
        let path: &[u8] = path_bytes_of(self.object_directory.as_path());
        let mut text: Vec<u8> = match alternates_line_needs_quoting(path) {
            true => c_quoted(path),
            false => path.to_vec(),
        };
        text.push(b'\n');
        return text;
    }
}

/// What:
///  Whether a pathname has to be written as a quoted line of the shadow's alternates.
/// Why:
///   The file holds one pathname per line and Git reads a line that begins with a double quote as a C-quoted pathname,
///  so a line feed would split the entry in two and a leading quote or hash would change how the line is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.includes('\n') || path.startsWith('"') || path.startsWith('#')
/// ```
fn alternates_line_needs_quoting(path: &[u8]) -> bool {
    return path.contains(&b'\n') || path.starts_with(b"\"") || path.starts_with(b"#");
}

/// What:
///  A pathname as one C-quoted line of the shadow's alternates,
/// which Git unquotes byte for byte.
/// Why:
///   A quote inside a quoted line would end the pathname early and a backslash would start an escape,
///  so the quote,
/// the backslash and the control bytes are written as the escapes Git reads back
///  (`unquote_c_style` in `quote.c`),
/// and every other byte is written as itself,
///  which keeps a name that is not text the name it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `"${path.replace(/[\\"\n\r\t]/g, escapeC)}"`
/// ```
fn c_quoted(path: &[u8]) -> Vec<u8> {
    let mut text: Vec<u8> = Vec::with_capacity(path.len() + 2);
    text.push(b'"');
    for byte in path {
        match byte {
            b'"' => text.extend_from_slice(b"\\\""),
            b'\\' => text.extend_from_slice(b"\\\\"),
            b'\n' => text.extend_from_slice(b"\\n"),
            b'\r' => text.extend_from_slice(b"\\r"),
            b'\t' => text.extend_from_slice(b"\\t"),
            _ => text.push(*byte),
        }
    }
    text.push(b'"');
    return text;
}

/// What:
///  Creates the shadow repository of one published transaction.
/// Why:
///   Every step runs before any native Git command reads the shadow,
///  so the first read already sees the real config,
/// objects,
/// links and refs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowRepository({ gitPath, cwd, capture, transactionId, transactionDirectory });
/// ```
pub fn create_shadow_repository(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    transaction_id: &str,
    transaction_directory: &Path,
) -> Result<PathBuf, GitFailure> {
    let shadow: PathBuf = shadow_repository_path(capture.layout.common_dir.as_path(), transaction_id);
    // No Git child runs here, so the stretch is named for the record a commit writes.
    let mut clock: BuildPartClock = BuildPartClock::begin("shadow parent");
    step_failure(create_parent(shadow.as_path()), "creating", shadow.as_path())?;
    create_shadow_repository_at(context, cwd, capture, shadow.as_path(), transaction_directory, &mut clock)?;
    return Ok(shadow);
}

/// What:
///  Creates a shadow repository at a given path,
/// whose parent directory exists.
/// Why:
///   The commit service builds the commits Git must build (an editor,
/// a signature) in a scratch repository inside its own
///  scratch directory,
/// away from the registry directory that recovery scans,
/// so no other process mistakes it for a dead
///  transaction's shadow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowRepositoryAt({ gitPath, cwd, capture, shadowPath, transactionDirectory });
/// ```
pub fn create_shadow_repository_at(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    shadow: &Path,
    transaction_directory: &Path,
    clock: &mut BuildPartClock,
) -> Result<(), GitFailure> {
    return create_shadow_repository_carrying(context, cwd, capture, shadow, transaction_directory, true, clock);
}

/// What:
///  `create_shadow_repository_at`,
/// with the merge,
/// cherry-pick and revert conclusion state of the worktree either copied into
///  the scratch repository or left out of it.
/// Why:
///   A commit of named paths while another session's merge is unfinished must build as an ordinary commit of those paths on
///  `HEAD` (real Git refuses it as a partial commit during a merge);
/// a scratch repository that holds no `MERGE_HEAD` builds
///  exactly that,
/// and the landing then leaves the worktree's own conclusion state alone because the scratch repository
///  never held it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowRepositoryCarrying({ gitPath, cwd, capture, shadowPath, transactionDirectory, carryConclusion });
/// ```
pub fn create_shadow_repository_carrying(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    shadow: &Path,
    transaction_directory: &Path,
    carry_conclusion: bool,
    clock: &mut BuildPartClock,
) -> Result<(), GitFailure> {
    let template: ShadowTemplate = ShadowTemplate::of(context, &capture.layout)?;
    create_shadow_template(context, &template, shadow, clock)?;
    return carry_the_commit_parts(context, cwd, capture, shadow, transaction_directory, carry_conclusion, clock);
}

/// What:
///  The failure of a test-only phase marker reached between a commit's question of the handed tree and its first write.
/// Why:
///   The marker variable decides a command's whole life when it is malformed (`transaction_phase.rs`),
///  and a command that cannot reach its marker says so in the words a commit reports failures with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function phaseMarkerFailure(error: Error): GitFailure; // new GitFailure(`...`)
/// ```
fn phase_marker_failure(error: crate::transaction_phase::PhaseMarkerError) -> GitFailure {
    return GitFailure(format!("the handed shadow's test-only phase marker could not be reached: {error}"));
}

/// What:
///  The shadow one commit was given,
///  and the finished one the service drew ahead of it when that is where it came from.
/// Why:
///   A drawn tree is used where it stands and is not the commit's own,
///  so the value that keeps it renewed,
///  releases it,
///  and tells the request which tree to name travels with the path:
///  a caller that took the path alone would leave a tree nothing renews.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShadowTree = { path: string; handed?: HandedShadow };
/// ```
pub struct ShadowTree {
    /// Where the shadow is.
    pub path: PathBuf,
    /// The drawn tree the service handed this commit,
    /// which is renewed while the commit lives and released when the service does not take it;
    /// nothing when the shadow is inside the commit's own scratch tree.
    pub handed: Option<crate::shadow_drawn::HandedShadow>,
}

/// What:
///  Whether the worktree holds the state a conclusion leaves,
///  which is the state a commit may conclude.
/// Why:
///   The landing that concludes compares what Git removed in the shadow with the copy of those files,
///  and it reads the shadow at the path the commit's own scratch tree holds,
///  so the hand-out is bounded by the presence of the state itself and not by the caller's wish to carry it:
///  every explicit-path commit asks to carry it, and almost none of them has any
///  (`doc/planning/git-policy-cli-shadow-pool-claim-rename-options.md`,
///  "The design the owner's decision made").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const conclusionStatePresent = (gitDir) => REMOVED_CONCLUSION_FILES.some((name) => exists(join(gitDir, name)));
/// ```
fn conclusion_state_present(git_dir: &Path) -> bool {
    for name in crate::conclusion_cleanup::REMOVED_CONCLUSION_FILES {
        if git_dir.join(name).exists() {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether this attempt may ask the service for a drawn shadow.
/// Why:
///   Three cases the hand-out cannot serve, each of which keeps the commit on the path it knows:
///  a reftable repository's shadow is Git's own tables,
///  a linked worktree that keeps its own `info` builds its own directory,
///  and both are never drawn for, so asking would be one round trip for nothing.
///  A commit that concludes a merge,
///  a cherry-pick or a revert copies the state that conclusion leaves and compares it with what Git removed in the
///  shadow,
///  reading the shadow at the path its own scratch tree holds,
///  so it takes a member or makes its creation until that comparison names the tree the request built in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mayBeHandedATree = (template, carryConclusion) => template.refFormat !== 'reftable' && !keepsItsOwnInfo(template) && !carryConclusion;
/// ```
pub(crate) fn may_be_handed_a_tree(template: &ShadowTemplate, carry_conclusion: bool) -> bool {
    if carry_conclusion {
        return false;
    }
    if template.ref_format == crate::transaction_journal::RefFormat::Reftable {
        return false;
    }
    return !crate::shadow_links::keeps_its_own_info(template.common_dir.as_path(), template.git_dir.as_path()).unwrap_or(false);
}

/// What:
///  Whether the shadow still holds the entries this commit's build was given,
///  which is what Git needs to work in it.
/// Why:
///  A removal is not one step:
///  a process killed between its steps leaves the root directory with some of its entries gone,
///  and a root that exists answers nothing about whether Git can work there
///  (`fatal: not a git repository`).
///  What the build was given is the creation's own entries and the ref store this commit's attempt wrote:
///  `HEAD` and the ref it names when the base is a commit,
///  `config`,
///  the `objects` tree down to the `alternates` file that reaches the real object store,
///  `refs`,
///  and the packed refs of the files backend.
///  A reftable shadow keeps its refs in Git's own tables and so has no packed file to look for.
///  The answer is a few metadata calls and one read of `HEAD`;
///  a tree that holds them all and still fails Git failed for the repository's or the volume's reason and is the
///  commit's to report,
///  and a tree that has lost any of them is one the commit builds in another tree rather than fails in
///  (`service_native.rs`,
///  `build_with_git`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whole = (tree, template, base) => hasFile('HEAD') && hasFile('config') && hasDir('objects') && hasDir('objects/info')
///   && hasFile('objects/info/alternates') && hasDir('refs') && (reftable || hasFile('packed-refs'))
///   && (reftable || base === 'unborn' || hasFile(headTargetOf(readFile('HEAD'))));
/// ```
pub fn the_shadow_holds_what_a_build_needs(tree: &Path, template: &ShadowTemplate, base: &crate::transaction_journal::Base) -> bool {
    use crate::transaction_journal::RefFormat;
    if !the_shadow_holds_what_a_creation_makes(tree, template) {
        return false;
    }
    // A reftable repository's shadow is Git's own tables, which no loose file and no packed file of ours stands for.
    if template.ref_format == RefFormat::Reftable {
        return true;
    }
    if !std::fs::symlink_metadata(tree.join("packed-refs")).is_ok_and(|metadata| { return metadata.is_file(); }) {
        return false;
    }
    if matches!(base, crate::transaction_journal::Base::Unborn) {
        return true;
    }
    // The ref `HEAD` names is a loose file this attempt wrote, unless `HEAD` is detached and names the commit itself.
    let Ok(head) = std::fs::read(tree.join("HEAD")) else { return false };
    let text: String = String::from_utf8_lossy(head.as_slice()).trim().to_string();
    let Some(name) = text.strip_prefix("ref: ") else { return true };
    return std::fs::symlink_metadata(tree.join(name.trim())).is_ok_and(|metadata| { return metadata.is_file(); });
}

/// What:
///  Whether the shadow still holds the entries a creation makes,
///  which are the entries every later write of the commit is written on top of.
/// Why:
///  It is the question a write that failed is asked,
///  where the ref store of the attempt may be exactly what could not be written (`a_shadow_from_the_handed_tree`):
///  `HEAD` and `config` as files,
///  the `objects` tree down to the `alternates` file that reaches the real object store,
///  and `refs` as a directory.
///  A tree that has lost any of them is one a removal reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = (tree) => hasFile('HEAD') && hasFile('config') && hasDir('objects') && hasDir('objects/info')
///   && hasFile('objects/info/alternates') && hasDir('refs');
/// ```
pub fn the_shadow_holds_what_a_creation_makes(tree: &Path, template: &ShadowTemplate) -> bool {
    let is_a_file: &dyn Fn(&Path) -> bool = &|path: &Path| { return std::fs::symlink_metadata(path).is_ok_and(|metadata| { return metadata.is_file(); }); };
    let is_a_directory: &dyn Fn(&Path) -> bool = &|path: &Path| { return std::fs::symlink_metadata(path).is_ok_and(|metadata| { return metadata.is_dir(); }); };
    if !is_a_directory(tree) || !is_a_file(&tree.join("HEAD")) || !is_a_file(&tree.join("config")) {
        return false;
    }
    let objects: PathBuf = tree.join("objects");
    if !is_a_directory(&objects) || !is_a_directory(&objects.join("info")) || !is_a_file(&objects.join("info").join("alternates")) {
        return false;
    }
    if !is_a_directory(&tree.join("refs")) {
        return false;
    }
    // A reftable repository's shadow is Git's own tables, which no entry of a creation stands for beyond these.
    let _ = template;
    return true;
}

/// What:
///  The shadow of one commit:
/// a finished one the service drew ahead and handed over,
/// a finished one this invocation can use from the pool,
/// or one made here when neither is there.
/// Why:
///   The creation is about 40 calls that create a directory entry on the volume the commit already waits on:
/// ten `mkdir`,
///  two file writes and three mode changes,
/// one link per shared entry,
/// one directory for the conclusion copies,
///  and the ref store.
/// A commit that takes a finished member makes one `rename` instead,
/// and a commit the pool cannot serve
///  makes the creation exactly as it did before
/// (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
/// "What each part of a commit's private repository is for,
/// and how many commits need it").
///   The ref store and the conclusion state are always this commit's,
/// after a claim as well as after a creation:
/// they name the branch and base of this invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await takeOrCreateShadow({ context, cwd, capture, shadow, transactionDirectory, carryConclusion, clock });
/// ```
pub fn take_or_create_shadow(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    shadow: &Path,
    transaction_directory: &Path,
    carry_conclusion: bool,
    clock: &mut BuildPartClock,
    service: &crate::service_server_socket_binding::Bind,
) -> Result<ShadowTree, GitFailure> {
    let template: ShadowTemplate = ShadowTemplate::of(context, &capture.layout)?;
    // What: The commit asks whether the service drew a finished shadow ahead of it, and where it is.
    // Why:  The one write a claim makes is the call that waited seconds on this volume, and the service made it before
    //       the commit asked. Every check a claim runs is still run here, on the tree the commit was handed, and a tree
    //       this invocation refuses is handed back and the commit takes the pool's member or makes its own: nothing about
    //       the question can fail a commit.
    clock.next("shadow hand-out");
    let concludes: bool = carry_conclusion && conclusion_state_present(capture.layout.git_dir.as_path());
    let asked: Option<(PathBuf, std::os::unix::net::UnixStream)> = match may_be_handed_a_tree(&template, concludes) {
        true => crate::service_client_shadow_hand_out::ask_for_a_drawn_shadow(service, &template, transaction_directory),
        false => None,
    };
    if let Some((drawn, answer)) = asked {
        // What: The tree this commit was handed, when it could take it and its question of it named nothing to refuse.
        // Why:  Everything this arm leaves behind is released by the value that held the tree, so a tree this invocation
        //       cannot use is handed back once and not twice; the connection the answer came on is kept until the tree is the
        //       commit's, so the service's claim covers every instant of the hand-over (`handle_connection`).
        if let Some(taken) = a_shadow_from_the_handed_tree(context, cwd, capture, &template, drawn.as_path(), answer, transaction_directory, carry_conclusion, clock)? {
            return Ok(taken);
        }
    }
    clock.next("shadow pool");
    if crate::shadow_pool::take(&template, shadow) {
        carry_the_commit_parts(context, cwd, capture, shadow, transaction_directory, carry_conclusion, clock)?;
        return Ok(ShadowTree { path: shadow.to_path_buf(), handed: None });
    }
    clock.next("shadow skeleton");
    create_shadow_template(context, &template, shadow, clock)?;
    carry_the_commit_parts(context, cwd, capture, shadow, transaction_directory, carry_conclusion, clock)?;
    return Ok(ShadowTree { path: shadow.to_path_buf(), handed: None });
}

/// What:
///  The commit's shadow, built in the tree the service handed it,
///  or nothing when this invocation cannot use that tree,
///  or the failure of a write whose tree is still there.
/// Why:
///   The commit claims the tree before it asks anything of it,
///  so the tree a sweep finds claimed is one it leaves,
///  and a tree a sweep has taken to remove is one this commit gives up on instead of building in
///  (`tree_held_by_a_live_process.rs`).
///  Everything after the claim can lose the tree to a cause no sweep explains, a person or a full drive,
///  and none of those may end the commit:
///  a failure whose tree is gone is answered with nothing, so the commit takes the pool's member or makes its own tree,
///  and a failure whose tree is still there is the repository's or the volume's and is the commit's to report
///  (`SPEC.md`, "Merges and commits never fail").
///  Nothing has been sent to the service when any of this happens, and what was written went into a tree this command
///  abandons.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = await aShadowFromTheHandedTree({ ... }); // ShadowTree | undefined, or a failure to report
/// ```
fn a_shadow_from_the_handed_tree(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    template: &ShadowTemplate,
    drawn: &Path,
    answer: std::os::unix::net::UnixStream,
    transaction_directory: &Path,
    carry_conclusion: bool,
    clock: &mut BuildPartClock,
) -> Result<Option<ShadowTree>, GitFailure> {
    let Some(held) = crate::shadow_drawn::HandedShadow::hold(drawn.to_path_buf()) else {
        crate::diagnostic_log::debug(
            "shadowDrawn",
            format!("the drawn shadow {} is being removed, so this commit takes another tree", drawn.display()).as_str(),
        );
        return Ok(None);
    };
    // What: The connection the answer came on closes here, and no later: the tree is this commit's from the claim above.
    // Why:  The service keeps its own claim on the tree until this end closes, so a client killed before its claim leaves a
    //       tree nobody holds, which the sweep takes a bound later; a client whose claim is taken hands the tree's keeping to
    //       itself and lets the service's go.
    drop(answer);
    // What: A control may hold the command at this instant, between its claim on the handed tree and its first write into it
    //       (`transaction_phase.rs`, `Phase::ShadowHanded`).
    // Why:  A tree that goes here decides whether the commit lands in it or in a tree of its own, and no other instant of a
    //       commit can be held at that point. Nothing arms this marker in a run.
    crate::service_pruned::reach_service_phase(crate::transaction_phase::Phase::ShadowHanded).map_err(phase_marker_failure)?;
    match refusal(template, drawn) {
        Ok(None) => {}
        Ok(Some(reason)) => {
            crate::diagnostic_log::debug("shadowDrawn", format!("refused the drawn shadow {}: {reason}", drawn.display()).as_str());
            return Ok(None);
        }
        Err(error) => {
            crate::diagnostic_log::debug("shadowDrawn", format!("refused the drawn shadow {}: {error}", drawn.display()).as_str());
            return Ok(None);
        }
    }
    match carry_the_commit_parts(context, cwd, capture, drawn, transaction_directory, carry_conclusion, clock) {
        Ok(()) => return Ok(Some(ShadowTree { path: drawn.to_path_buf(), handed: Some(held) })),
        Err(error) => {
            // A failure whose tree still holds what this build was given is the repository's or the volume's, and it is the
            //  commit's to report; a tree that has lost an entry is one a removal reached, and the commit takes another.
            if the_shadow_holds_what_a_creation_makes(drawn, template) {
                return Err(error);
            }
            crate::diagnostic_log::debug(
                "shadowDrawn",
                format!("the drawn shadow {} went before this commit wrote into it: {error}", drawn.display()).as_str(),
            );
            // The tree stops being this commit's here, and what is left of it goes beside the command.
            drop(held);
            return Ok(None);
        }
    }
}

/// What:
///  Creates everything of a shadow that does not depend on the commit or its branch:
/// the skeleton,
/// the config,
/// the alternates and the links.
/// Why:
///   These are the stretches a pool member holds,
/// and the two callers of them (a commit's own creation and the pool's fill) must make
///  the same entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowTemplate({ context, template, shadowPath, clock });
/// ```
pub fn create_shadow_template(
    context: &GitContext,
    template: &ShadowTemplate,
    shadow: &Path,
    clock: &mut BuildPartClock,
) -> Result<(), GitFailure> {
    if template.ref_format == crate::transaction_journal::RefFormat::Reftable {
        // The reftable backend's tables are Git's to create, so this one case still starts `git init`.
        init_with_git(context, template, shadow)?;
    } else {
        step_failure(create_shadow_skeleton(shadow), "creating", shadow)?;
    }
    step_failure(set_private_directory_mode(shadow), "protecting", shadow)?;
    clock.next("shadow config");
    named_step_failure(
        overwrite_private(shadow.join("config").as_path(), template.config_bytes().as_slice()),
        "writing the shadow config",
    )?;
    named_step_failure(
        overwrite_private(
            shadow.join("objects").join("info").join("alternates").as_path(),
            template.alternates_bytes().as_slice(),
        ),
        "writing the shadow alternates",
    )?;
    clock.next("shadow links");
    named_step_failure(
        link_shadow_entries(shadow, template.common_dir.as_path(), template.git_dir.as_path()),
        "linking the shadow entries",
    )?;
    return Ok(());
}

/// What:
///  Adds what belongs to this commit alone:
/// the conclusion state it concludes and the ref store it points at.
/// Why:
///   A member of the pool is made before any commit names a branch,
/// so these two run after the claim as they run after a creation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await carryTheCommitParts({ context, cwd, capture, shadow, transactionDirectory, carryConclusion, clock });
/// ```
fn carry_the_commit_parts(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    shadow: &Path,
    transaction_directory: &Path,
    carry_conclusion: bool,
    clock: &mut BuildPartClock,
) -> Result<(), GitFailure> {
    let layout = &capture.layout;
    clock.next("shadow conclusion");
    if carry_conclusion {
        copy_conclusion_state(
            context,
            cwd,
            layout.git_dir.as_path(),
            shadow,
            transaction_directory,
            layout.ref_format,
        )?;
    }
    clock.next("shadow refs");
    write_shadow_refs(
        context,
        cwd,
        shadow,
        &SnapshotSources::of(layout.git_dir.clone(), layout.common_dir.clone(), layout.ref_format, layout.object_format.as_str()),
        &layout.target_ref,
        &layout.symbolic_head,
        &capture.base,
    )?;
    return Ok(());
}

/// What:
///  Gives a shadow directory the private mode.
/// Why:
///   A shadow holds the transaction's objects and refs,
/// and a pool member is a shadow whose owner is not known until a commit claims it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await chmod(shadowPath, 0o700);
/// ```
#[cfg(unix)]
fn set_private_directory_mode(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    return std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700));
}

/// Windows has no mode bits for the shadow directory.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
fn set_private_directory_mode(_path: &Path) -> std::io::Result<()> {
    return Ok(());
}

/// Shadow creation controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_create_tests.rs"]
mod tests;
