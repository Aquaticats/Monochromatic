//! What:
//!  Which registration of the invoking repository a worktree directory belongs to,
//! and whether Git would pick that same
//!  worktree for the name the person typed.
//! Why:
//!  Git removes only a worktree that its own list for the repository it runs in contains,
//! found first by the last
//!  components of every registered path and only then by the resolved path (`find_worktree` in Git's `worktree.c`).
//! The
//!  `.git` file of a worktree and the `gitdir` file of an administrative directory that name each other prove only that
//!  the pair is consistent,
//! not that the repository the command runs in registered it,
//! and a directory that exists under
//!  the typed name proves nothing about the worktree Git would remove for that name.
//! Anything but a registration of the
//!  invoking repository that Git would also pick is left to Git,
//! which words its own answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const registration = registrationOf({ commonDir, target, operand }); // throws the reason Git must decide
//! ```

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

/// What:
///  The registration of one linked worktree in the repository the command runs in.
/// Why:
///   The administrative directory is where the registration lives;
/// its name is the worktree's id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Registration = { id: string; admin: string; worktrees: string; gitdirText: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Registration {
    /// The name of the administrative directory below `worktrees`.
    pub id: OsString,
    /// The canonical administrative directory,
    /// `<common>/worktrees/<id>`.
    pub admin: PathBuf,
    /// The canonical `<common>/worktrees` directory.
    pub worktrees: PathBuf,
    /// The bytes of the `gitdir` file as they were proven to name the worktree,
    /// to compare again before the removal.
    pub gitdir_text: Vec<u8>,
}

/// The line of a file without its final newline.
fn without_newline(text: &[u8]) -> &[u8] {
    return text.strip_suffix(b"\n").unwrap_or(text);
}

/// What:
///  Where `operand` starts inside `candidate` when the two meet at a directory boundary,
/// or nothing when the operand is empty or longer than the path.
/// Why:
///   Git's match by last components (`find_worktree_by_suffix` in `worktree.c`) asks every candidate those same two
/// questions,
/// and the two comparisons below differ only in the answer they take from the bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lastComponentsStart(candidate: Uint8Array, operand: Uint8Array): number | undefined;
/// ```
fn last_components_start(candidate: &[u8], operand: &[u8]) -> Option<usize> {
    if operand.is_empty() || operand.len() > candidate.len() {
        return None;
    }
    let start: usize = candidate.len() - operand.len();
    let at_boundary: bool = start == 0 || candidate[start - 1] == b'/';
    return at_boundary.then_some(start);
}

/// What:
///  Whether Git's match by last components finds `candidate` for `operand` by the operand's own bytes.
/// Why:
///   Git compares that suffix with `fspathcmp`,
/// which folds ASCII case only when `core.ignorecase` says so
/// (Git's `dir.c`),
/// so a byte match is one Git picks under either setting.
/// This is the comparison that picks the worktree to remove:
/// a name Git would refuse never removes another worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endsWithName(candidate: string, operand: string): boolean;
/// ```
fn ends_with_name(candidate: &[u8], operand: &[u8]) -> bool {
    let Some(start) = last_components_start(candidate, operand) else { return false };
    return candidate[start..].eq(operand);
}

/// What:
///  Whether Git's match by last components could find `candidate` for `operand` under any `core.ignorecase`
/// setting:
/// the same directory boundary,
/// with ASCII case ignored.
/// Why:
///   The guard that declines a removal must not miss a name Git's suffix match could take for another worktree,
/// and a looser match only sends more removals to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endsWithNameIgnoringCase(candidate: string, operand: string): boolean;
/// ```
fn ends_with_name_ignoring_case(candidate: &[u8], operand: &[u8]) -> bool {
    let Some(start) = last_components_start(candidate, operand) else { return false };
    return candidate[start..].eq_ignore_ascii_case(operand);
}

/// What:
///  One worktree path of Git's list,
/// with the name of the `worktrees/<id>` directory that records it.
/// Why:
///   Git's suffix match counts registrations,
/// so two registrations whose paths name one directory through a symbolic link are still two answers to it;
/// the registration a removal deletes is therefore known by its id,
/// never by the file its path names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RegisteredPath = { id: string | undefined; path: Uint8Array };
/// ```
struct RegisteredPath {
    /// The name of the administrative directory below `worktrees`,
    /// or nothing for the main checkout.
    id: Option<OsString>,
    /// The worktree path as Git's list holds it,
    /// bytes and all.
    path: Vec<u8>,
}

/// What:
///  Every worktree path the repository lists:
/// the main one,
/// and each `gitdir` of `worktrees/*`,
/// as Git reads them,
/// each with the registration that records it.
/// Why:
///   The list Git matches the typed name against.
/// A `gitdir` file Git skips (unreadable or empty) is skipped;
/// a
///  `worktrees` directory that cannot be listed leaves the list unknown,
/// which is an error.
/// A record is read the way the target's own records are read:
/// without following a link,
/// up to the bytes Git writes,
/// and without waiting for a writer,
/// so a `gitdir` that is a named pipe is skipped as one Git would skip
/// instead of holding the removal inside `open(2)` for ever
/// (`SPEC.md`,
/// "Every wait has a bound").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function registeredPaths(commonDir: string): RegisteredPath[];
/// ```
fn registered_paths(common_dir: &Path, worktrees: &Path) -> Result<Vec<RegisteredPath>, String> {
    let mut paths: Vec<RegisteredPath> = Vec::new();
    let main: &[u8] = common_dir.as_os_str().as_bytes();
    paths.push(RegisteredPath { id: None, path: main.strip_suffix(b"/.git").unwrap_or(main).to_vec() });
    let listing = match std::fs::read_dir(worktrees) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(paths),
        Err(error) => return Err(format!("the registrations in {} cannot be listed ({error})", worktrees.display())),
    };
    for found in listing {
        let entry: std::fs::DirEntry = match found {
            Ok(listed) => listed,
            Err(error) => return Err(format!("the registrations in {} cannot be listed ({error})", worktrees.display())),
        };
        // A registration Git would skip is skipped here too: one whose record is missing, unreadable, no regular file Git writes,
        //  or larger than a record. A named pipe at the name would wait inside `open(2)` for a writer that may never come.
        let Ok(text) = crate::worktree_registration_proof::read_record_here(entry.path().join("gitdir").as_path(), "a registration's back-link") else {
            continue;
        };
        let trimmed: &[u8] = text.trim_ascii_end();
        if trimmed.is_empty() {
            continue;
        }
        let path: &[u8] = trimmed.strip_suffix(b"/.git").unwrap_or(trimmed);
        let spelled: Vec<u8> = if path.starts_with(b"/") {
            path.to_vec()
        } else {
            // A relative record is relative to the administrative directory.
            let joined: PathBuf = entry.path().join(OsStr::from_bytes(path));
            std::fs::canonicalize(joined.as_path()).unwrap_or(joined).into_os_string().into_vec()
        };
        paths.push(RegisteredPath { id: Some(entry.file_name()), path: spelled });
    }
    return Ok(paths);
}

/// What:
///  The worktree path Git picks for `operand` by the last components of the registered paths,
///  or nothing when it picks none or more than one.
/// Why:
///   Git matches the typed name against the end of every registered worktree path at a directory boundary before it
///  resolves a real path (`find_worktree_by_suffix` in `worktree.c`),
///  so a person may name a worktree by its last component and the wrapper serves the same removal.
/// That comparison is byte for byte unless Git's own `core.ignorecase` setting folds the case,
/// so a name that differs from the registered path only by case picks nothing here:
/// it is Git's answer to give,
/// and a worktree Git would refuse must never be the one this removes.
/// A name two registered paths end in is Git's ambiguous case,
/// which falls back to the resolved path,
///  and no path here resolves to a directory for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathGitPicksForOperand(commonDir: string, operand: string): string | undefined;
/// ```
pub fn path_git_picks_for_operand(common_dir: &Path, operand: &OsStr) -> Option<PathBuf> {
    let worktrees: PathBuf = common_dir.join("worktrees");
    let registered: Vec<RegisteredPath> = registered_paths(common_dir, worktrees.as_path()).ok()?;
    let mut picked: Option<&[u8]> = None;
    for listed in &registered {
        if !ends_with_name(listed.path.as_slice(), operand.as_bytes()) {
            continue;
        }
        // Git's suffix match answers a unique worktree or none; two matches send it to the resolved path instead.
        if picked.is_some() {
            return None;
        }
        picked = Some(listed.path.as_slice());
    }
    let only: &[u8] = picked?;
    let resolved: PathBuf = std::fs::canonicalize(Path::new(OsStr::from_bytes(only))).ok()?;
    return resolved.is_dir().then_some(resolved);
}

/// What:
/// What:
///  One worktree of Git's own list for the repository,
///  as Git prints it.
/// Why:
///   Git answers `find_worktree` from its own list (`worktree.c`),
///  so the paths a guard compares a typed name against are read from Git itself:
///  a reader of the same records can spell a path another way,
///  and a guard that compares against its own spelling names no worktree where Git names one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ListedWorktree = { path: Uint8Array; main: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListedWorktree {
    /// The path Git printed for the worktree,
    /// bytes and all,
    /// as its own list holds it.
    pub path: Vec<u8>,
    /// Whether Git calls it the repository's main working tree,
    /// which no removal of a worktree takes.
    pub main: bool,
}

/// What:
///  The worktree one `git worktree remove` operand names,
///  by Git's own answer.
/// Why:
///   A removal goes on to real Git only when the worktree it names is one this program can judge or
///  hand Git as the worktree it judged,
///  so the answer is one worktree or the reason there is none:
///  the main working tree is the one Git refuses,
///  and it is answered as itself rather than as no answer,
///  because the person's word then gets the sentence about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OperandWorktree = { kind: 'linked', path: string } | { kind: 'main', path: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperandWorktree {
    /// A linked worktree of the repository,
    /// by its own resolved path.
    Linked(PathBuf),
    /// The repository's main working tree,
    /// by its own resolved path.
    Main(PathBuf),
}

/// What:
///  Why Git's own resolution names no worktree for an operand,
///  and which of those answers leaves the removal to Git.
/// Why:
///   The two answers are not the same thing to a guard:
///  a name Git's own resolution names no worktree for is a name Git refuses,
///  and it removes nothing,
///  so the command is Git's to answer as the person wrote it;
///  a name two worktrees resolve to is a removal Git makes,
///  of whichever its list puts first,
///  and nothing here can judge that worktree.
///  One sentence per answer keeps the guard from reading the second as the first
///  (`worktree_removal_typed_git_work.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OperandUnresolved = { kind: 'no-worktree', reason: string } | { kind: 'ambiguous', reason: string } | { kind: 'unreadable', reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperandUnresolved {
    /// Git's own resolution of the name names no worktree of this repository,
    ///  so Git refuses the removal and removes nothing.
    NoWorktree(String),
    /// More than one worktree of this repository resolves to the name,
    ///  so which one Git removes is its list's order.
    Ambiguous(String),
    /// Nothing here can read Git's list,
    ///  or a listed path cannot be resolved,
    ///  so what Git would do is unknown.
    Unreadable(String),
}

/// What:
///  Every worktree Git lists for the repository,
///  in Git's own order,
///  which puts the main one first.
/// Why:
///   The list is Git's own answer to which worktrees its `worktree remove` may act on,
///  and its own order is where the main working tree stands
///  (`get_worktrees_internal` puts it first and `list` sorts only the rest,
///  in `worktree.c` and `builtin-worktree.c`).
///  A listing that cannot be read is the reason rather than an empty list,
///  because an empty list would name no worktree for a removal that does have one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listedByGit(context, commonDir): ListedWorktree[]; // throws the reason
/// ```
fn listed_by_git(context: &crate::transaction_git::GitContext, common_dir: &Path, answers_with: &[OsString]) -> Result<Vec<ListedWorktree>, String> {
    // `-z` keeps a path with a newline or a quote in it one record, and porcelain keeps the format Git documents.
    // It starts in the common Git directory, which the caller's `-C` chain already reached, and carries the caller's
    // configuration options and no option that moves it: a forwarded relative `-C` failed here
    // (a review of `8858f843ee`, finding 4).
    let mut arguments: Vec<OsString> = answers_with.to_vec();
    arguments.extend(["worktree", "list", "--porcelain", "-z"].map(OsString::from));
    let request: crate::transaction_git::GitRequest = crate::transaction_git::GitRequest { without_prefix: true, ..crate::transaction_git::GitRequest::new(common_dir, arguments.as_slice()) };
    let output = match crate::transaction_git::run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("Git cannot list the worktrees of this repository ({error})")),
    };
    if output.code != Some(0) {
        return Err(format!(
            "Git cannot list the worktrees of this repository ({})",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    return Ok(worktrees_of_a_git_listing(output.stdout.as_slice()));
}

/// What:
///  The worktrees of one `git worktree list --porcelain -z` answer.
/// Why:
///   Each worktree is one record of NUL-terminated lines,
///  the first line of which is `worktree <path>`,
///  and the first record of the answer is the main working tree
///  (`show_worktree_porcelain` in `builtin-worktree.c`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreesOfAGitListing(bytes: Uint8Array): ListedWorktree[];
/// ```
pub fn worktrees_of_a_git_listing(bytes: &[u8]) -> Vec<ListedWorktree> {
    let mut listed: Vec<ListedWorktree> = Vec::new();
    for field in bytes.split(|byte| return *byte == 0) {
        let Some(spelled) = field.strip_prefix(b"worktree ") else { continue };
        listed.push(ListedWorktree { path: spelled.to_vec(), main: listed.is_empty() });
    }
    return listed;
}

/// What:
///  The worktree Git's own `git worktree remove <operand>` picks for `operand`,
///  or why Git's own resolution names none.
/// Why:
///  Git resolves the operand in two steps (`find_worktree` in `worktree.c`, Git 2.56.0):
///  first the whole operand as the last components of a listed path,
///  meeting at a directory boundary,
///  answered only when exactly one listed path ends in it;
///  then the operand joined to the command's prefix,
///  resolved as Git resolves a path,
///  and compared as Git compares paths with each listed path resolved the same way.
///  Both comparisons fold ASCII case exactly when the repository sets `core.ignorecase`
///  (`fspathcmp` in `dir.c`),
///  and the paths are Git's own, from `git worktree list`,
///  so the answer is the worktree the command itself names.
///  A name whose own resolution names no path,
///  and a name no listed worktree answers,
///  are what Git answers by dying on `'<name>' is not a working tree`,
///  so `OperandUnresolved::NoWorktree` says so and the removal stays Git's to answer:
///  nothing is removed there and no guard has a worktree to judge.
///  A name two listed worktrees answer is not that:
///  Git removes whichever its list puts first,
///  so nothing here can prove which worktree a guard would judge,
///  and `OperandUnresolved::Ambiguous` refuses instead.
///  A listing that cannot be read,
///  and a listed path that cannot be resolved,
///  are `OperandUnresolved::Unreadable`: what Git would do is unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitWorktreeForOperand({ context, commonDir, operand, start, foldCase }): OperandWorktree | OperandUnresolved;
/// ```
///
/// # Errors
///
/// The three answers above, each with the sentence naming why.
pub fn git_worktree_for_operand(
    context: &crate::transaction_git::GitContext,
    common_dir: &Path,
    operand: &OsStr,
    start: &Path,
    fold_case: bool,
    answers_with: &[OsString],
) -> Result<OperandWorktree, OperandUnresolved> {
    let listed: Vec<ListedWorktree> = match listed_by_git(context, common_dir, answers_with) {
        Ok(found) => found,
        Err(reason) => return Err(OperandUnresolved::Unreadable(reason)),
    };
    // Git's first step: the whole operand as the last components of a listed path, at a directory boundary.
    let mut by_suffix: Vec<&ListedWorktree> = Vec::new();
    for one in &listed {
        let Some(at) = last_components_start(one.path.as_slice(), operand.as_bytes()) else { continue };
        let tail: &[u8] = &one.path[at..];
        let same: bool = match fold_case {
            true => tail.eq_ignore_ascii_case(operand.as_bytes()),
            false => tail.eq(operand.as_bytes()),
        };
        if same {
            by_suffix.push(one);
        }
    }
    if by_suffix.len() == 1 {
        return match resolved_operand(by_suffix[0]) {
            Ok(found) => Ok(found),
            Err(reason) => Err(OperandUnresolved::Unreadable(reason)),
        };
    }
    // Two listed paths ending in the name is no answer here, as it is none to Git: `find_worktree` falls to the
    // resolved path for both no match and more than one.
    let joined: PathBuf = match Path::new(operand).is_absolute() {
        true => PathBuf::from(operand),
        false => start.join(operand),
    };
    let resolved: PathBuf = match crate::worktree_path_as_git_resolves_it::git_real_path(joined.as_path()) {
        Some(found) => found,
        None => {
            return Err(OperandUnresolved::NoWorktree(format!(
                "Git's own resolution of the name {} names no path: a component before the last one is not there, a separator follows a last component that is not there, or links lead back to themselves",
                joined.display()
            )));
        }
    };
    let mut by_path: Option<&ListedWorktree> = None;
    for one in &listed {
        let spelled: &Path = Path::new(OsStr::from_bytes(one.path.as_slice()));
        let Some(theirs) = crate::worktree_path_as_git_resolves_it::git_real_path(spelled) else { continue };
        if !crate::worktree_path_as_git_resolves_it::the_same_path(&theirs, &resolved, fold_case) {
            continue;
        }
        if by_path.is_some() {
            return Err(OperandUnresolved::Ambiguous(format!(
                "more than one worktree of this repository resolves to {}, so which one Git picks is its list's order and nothing this program can read",
                resolved.display()
            )));
        }
        by_path = Some(one);
    }
    return match by_path {
        Some(only) => match resolved_operand(only) {
            Ok(found) => Ok(found),
            Err(reason) => Err(OperandUnresolved::Unreadable(reason)),
        },
        None => Err(OperandUnresolved::NoWorktree(format!("no worktree of this repository resolves from the name {}", joined.display()))),
    };
}

/// What:
///  One listed worktree as the answer for an operand,
///  by the path Git itself resolves it to.
/// Why:
///   The path handed to real Git in place of the operand is this one,
///  so the worktree this program judged and the worktree Git removes are one thing by construction
///  rather than by two resolutions agreeing.
///  A listed path Git cannot resolve is the reason:
///  its parent is not there, or a link on the way loops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolvedOperand(listed: ListedWorktree): OperandWorktree; // throws the reason
/// ```
fn resolved_operand(listed: &ListedWorktree) -> Result<OperandWorktree, String> {
    let spelled: &Path = Path::new(OsStr::from_bytes(listed.path.as_slice()));
    let Some(resolved) = crate::worktree_path_as_git_resolves_it::git_real_path(spelled) else {
        return Err(format!("the worktree Git lists as {} cannot be resolved to a path", spelled.display()));
    };
    return Ok(match listed.main {
        true => OperandWorktree::Main(resolved),
        false => OperandWorktree::Linked(resolved),
    });
}
/// What:
///  The registration of `target` in the repository whose common Git directory is `common_dir`,
/// when it is a linked worktree of that repository.
/// Why:
///   The removal deletes this registration,
/// so its id names the worktree's commit service units,
/// and its `gitdir` text is what the removal proves again before the destroy.
/// Whether Git would also pick this worktree for the name the person typed is the separate question
/// `registration_of` answers.
/// The error is the reason,
/// for the log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function registeredWorktree(args: { commonDir: string; target: string }): Registration; // throws the reason
/// ```
pub fn registered_worktree(common_dir: &Path, target: &Path) -> Result<Registration, String> {
    let pointer_path: PathBuf = target.join(".git");
    // The pointer is read without waiting for a writer: a named pipe at its name would hold a removal for ever
    //  (`SPEC.md`, "Every wait has a bound").
    let pointer: Vec<u8> = match crate::worktree_registration_proof::read_record_here(pointer_path.as_path(), "the worktree's pointer") {
        Ok(bytes) => bytes,
        Err(refusal) => return Err(format!("{}, so this is no linked worktree", refusal.into_words())),
    };
    let line: &[u8] = without_newline(pointer.as_slice());
    let Some(named) = line.strip_prefix(b"gitdir: ") else {
        return Err(format!("{} does not start with `gitdir: `", pointer_path.display()));
    };
    let admin: PathBuf = match std::fs::canonicalize(target.join(OsStr::from_bytes(named))) {
        Ok(resolved) => resolved,
        Err(error) => return Err(format!("the administrative directory {} that {} names cannot be resolved ({error})", String::from_utf8_lossy(named), pointer_path.display())),
    };
    let worktrees: PathBuf = match std::fs::canonicalize(common_dir.join("worktrees")) {
        Ok(resolved) => resolved,
        Err(error) => return Err(format!("{} cannot be resolved ({error}), so this repository has no linked worktree", common_dir.join("worktrees").display())),
    };
    if admin.parent() != Some(worktrees.as_path()) {
        return Err(format!(
            "the administrative directory {} of {} is not one of the registrations in {}, the repository this command runs in",
            admin.display(),
            target.display(),
            worktrees.display()
        ));
    }
    let Some(admin_name) = admin.file_name() else {
        return Err(format!("{} has no name", admin.display()));
    };
    let id: OsString = admin_name.to_os_string();
    let gitdir_text: Vec<u8> = match crate::worktree_registration_proof::read_record_here(admin.join("gitdir").as_path(), "the registration's back-link") {
        Ok(bytes) => bytes,
        Err(refusal) => return Err(refusal.into_words()),
    };
    let recorded: PathBuf = PathBuf::from(OsStr::from_bytes(without_newline(gitdir_text.as_slice())));
    // The registration may spell this worktree through a symbolic link or relative to itself; the question is which file it names.
    let recorded_pointer: PathBuf = if recorded.is_absolute() { recorded.clone() } else { admin.join(recorded.as_path()) };
    if !crate::real_git_candidate::same_file(recorded_pointer.as_path(), pointer_path.as_path()) {
        return Err(format!("{} records {}, not {}", admin.join("gitdir").display(), recorded_pointer.display(), pointer_path.display()));
    }
    return Ok(Registration { id, admin, worktrees, gitdir_text });
}

/// What:
///  The registration of `target` in the repository whose common Git directory is `common_dir`,
/// when it is a linked
///  worktree of that repository that Git would also pick for `operand`.
/// Why:
///   See the module description.
/// The error is the reason,
/// for the log;
/// the removal is then left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function registrationOf(args: { commonDir: string; target: string; operand: string }): Registration; // throws the reason
/// ```
pub fn registration_of(common_dir: &Path, target: &Path, operand: &OsStr) -> Result<Registration, String> {
    let registration: Registration = registered_worktree(common_dir, target)?;
    let registered: Vec<RegisteredPath> = registered_paths(common_dir, registration.worktrees.as_path())?;
    for other in registered {
        // The registration this removal deletes is the worktree's own and the answer Git gives for it; every other one
        // that Git's suffix match could take the name for is a second worktree to Git, even when its path names the same
        // directory through a symbolic link, and then the name is Git's to answer.
        if other.id.as_deref() == Some(registration.id.as_os_str()) {
            continue;
        }
        if ends_with_name_ignoring_case(other.path.as_slice(), operand.as_bytes()) {
            return Err(format!(
                "another registration, {}, ends in the name {} at a directory boundary, ignoring ASCII case, so Git could pick it instead of {}",
                String::from_utf8_lossy(other.path.as_slice()),
                operand.to_string_lossy(),
                target.display()
            ));
        }
    }
    return Ok(registration);
}

/// Registry controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_registry_tests.rs"]
mod tests;
