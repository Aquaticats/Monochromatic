//! What:
//!  The questions a repair asks Git and the readings that turn one answer into what it decides from:
//! the tree of an index and the paths that moved since the copy,
//! the listing an entry is set from,
//! the object a worktree path holds,
//! the deletions a landing could own,
//! and the copy of the index taken aside.
//! Why:
//!   A builder written once is the only way a synchronous face and a task of the runtime ask the
//! same question,
//! and a reader written once is the only way they answer a refused step the same way.
//! Both faces ask through this file;
//! nothing here starts a child,
//! waits for one or decides anything about a path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tree = await git(treeRequest(repository, liveIndex));
//! ```

use crate::owed_index_entries::{Entry, is_zero_byte, names_in, specs_of};
use crate::service_protocol::FailureCode;
use crate::transaction_git::{GitOutput, GitRequest};
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// What:
///  The question `moved_since` asks first:
///  the tree the live index holds.
/// Why:
///  Both faces ask the one question,
/// so neither compares another tree with the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['write-tree'], index: liveIndex };
/// ```
pub(super) fn tree_request(repository: &Path, live_index: &Path) -> GitRequest {
    let mut tree: GitRequest = GitRequest::new(repository, &["write-tree"]);
    tree.index = Some(live_index.to_path_buf());
    return tree;
}
/// What:
///  The question `moved_since` asks second:
///  which paths the commit's own index holds differently from `held`.
/// Why:
///  Both faces ask the one question,
/// so a path one of them would report is reported by the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['diff-index', '--cached', '--name-only', '-z', held], index: commitIndex };
/// ```
pub(super) fn moved_request(repository: &Path, commit_index: &Path, held: &str) -> GitRequest {
    let mut moved: GitRequest = GitRequest::new(repository, &["diff-index", "--cached", "--name-only", "-z", held]);
    moved.index = Some(commit_index.to_path_buf());
    return moved;
}
/// What:
///  The tree Git's answer to `tree_request` names.
/// Why:
///  Both faces read the name through this one function,
/// so a tree Git wrote and a step it refused cannot be told apart
///  differently by the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeOf(asked: GitOutput | Error): string; // throws FailureCode.Unavailable
/// ```
pub(super) fn tree_of(asked: std::io::Result<GitOutput>) -> Result<String, FailureCode> {
    return match asked {
        Ok(answer) if answer.succeeded() => Ok(String::from_utf8_lossy(answer.stdout.as_slice()).trim().to_string()),
        _ => Err(FailureCode::Unavailable),
    };
}
/// What:
///  The paths Git's answer to `moved_request` names.
/// Why:
///  Both faces read the listing through this one function,
/// so a listing and a refusal are the same answer for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function movedOf(asked: GitOutput | Error): path[]; // throws FailureCode.Unavailable
/// ```
pub(super) fn moved_of(asked: std::io::Result<GitOutput>) -> Result<Vec<Vec<u8>>, FailureCode> {
    return match asked {
        Ok(output) if output.succeeded() => Ok(names_in(output.stdout.as_slice())),
        _ => Err(FailureCode::Unavailable),
    };
}
/// What:
///  The two paths of one copy of an index aside,
/// owned so that a job of the blocking pool can take them.
/// Why:
///   Setting the commit's own index to the head keeps a copy of it first,
/// so a step that refuses leaves the file exactly as it was;
///  both copies wait in the kernel on the disk the repository is on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexCopy = { from: string; to: string };
/// ```
pub(super) struct IndexCopy {
    /// The file the copy is made of,
    /// or the copy that is put back.
    pub(super) from: PathBuf,
    /// The file the copy is written to,
    /// or the index the copy is put back into.
    pub(super) to: PathBuf,
}
/// The copy one job of the pool makes:
/// `std::fs::copy` over the two paths of the question.
pub(super) fn copy_of(work: IndexCopy) -> std::io::Result<()> {
    let _written: u64 = std::fs::copy(work.from.as_path(), work.to.as_path())?;
    return Ok(());
}
/// The removal one job of the pool makes:
/// the copy aside once the step it protected ended.
pub(super) fn removed(path: PathBuf) -> std::io::Result<()> {
    return std::fs::remove_file(path.as_path());
}
/// What:
///  The question `set_to_head_unchecked` asks first:
///  the removal of whatever the index held at `paths` and below them.
/// Why:
///  Both faces ask the one question,
/// so neither prunes another set of entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['rm', '--cached', '-r', '--force', '--ignore-unmatch', '--quiet', '--', ...specs], index };
/// ```
pub(super) fn prune_request(repository: &Path, index: &Path, paths: &[Vec<u8>]) -> GitRequest {
    let specs: Vec<OsString> = specs_of(paths);
    let mut prune_words: Vec<&OsStr> = vec![
        OsStr::new("rm"),
        OsStr::new("--cached"),
        OsStr::new("-r"),
        OsStr::new("--force"),
        OsStr::new("--ignore-unmatch"),
        OsStr::new("--quiet"),
        OsStr::new("--"),
    ];
    prune_words.extend(specs.iter().map(OsString::as_os_str));
    let mut prune: GitRequest = GitRequest::new(repository, prune_words.as_slice()).own_pathspecs();
    prune.index = Some(index.to_path_buf());
    return prune;
}
/// What:
///  The listing `set_to_head_unchecked` writes when Git's own `reset` refused:
///  the head's entry of each path,
///  and a removal for a path the head holds nothing at.
/// Why:
///  Both faces write the head's entries through this one function,
/// so a path the head does not hold and a path it does
///  are the same entry for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listing = head.has(path) ? `${mode} ${object} 0\t${path}\0` : `0 ${'0'.repeat(40)} 0\t${path}\0`;
/// ```
pub(super) fn index_info_listing(head: &BTreeMap<Vec<u8>, Entry>, paths: &[Vec<u8>]) -> Vec<u8> {
    let mut listing: Vec<u8> = Vec::new();
    for path in paths {
        let (mode, object) = match head.get(path) {
            Some((mode, object)) => (mode.clone(), object.clone()),
            // The head holds nothing there, so the entry goes.
            None => (String::from("0"), String::from("0".repeat(40))),
        };
        listing.extend_from_slice(format!("{mode} {object} 0\t").as_bytes());
        listing.extend_from_slice(path.as_slice());
        listing.push(0);
    }
    return listing;
}
/// What:
///  The question `set_to_head_unchecked` asks last:
///  the entries written where the head's tree has them.
/// Why:
///  Both faces write the one listing to the index they were given,
/// so neither writes another entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['update-index', '-z', '--index-info'], index, input: listing };
/// ```
pub(super) fn index_info_request(repository: &Path, index: &Path, listing: Vec<u8>) -> GitRequest {
    let mut write: GitRequest = GitRequest::new(repository, &["update-index", "-z", "--index-info"]);
    write.index = Some(index.to_path_buf());
    write.input = Some(listing);
    return write;
}
/// What:
///  Whether Git took one of the two writes this step makes.
/// Why:
///  Both faces read the answer through this one function,
/// so a write that could not be started and one Git refused are the
///  same answer for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tookIt(asked: GitOutput | Error): boolean;
/// ```
pub(super) fn took_it(asked: std::io::Result<GitOutput>) -> bool {
    return matches!(asked, Ok(output) if output.succeeded());
}
/// The path one worktree question names,
/// owned so that a job of the blocking pool can take it.
pub(super) struct WorktreePath {
    /// The path in the worktree the entry belongs to.
    pub(super) path: PathBuf,
}
/// What:
///  What one path of the worktree holds:
/// nothing,
/// or a file or a link with the mode Git records for it and the text of a link.
/// Why:
///  The repair compares the entry the index holds with what the worktree holds,
/// and both facts are read from the
///  same metadata,
/// so a file that is replaced between two looks cannot be read as two different entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeFile = { kind: 'absent' } | { kind: 'present'; mode: string; linkTarget?: Uint8Array };
/// ```
pub(super) enum WorktreeFile {
    /// Nothing is there.
    Absent,
    /// A file or a link is there,
    /// with the mode Git records and a link's target text.
    Present(WorktreeEntry),
}
/// One path of the worktree that is there:
/// its mode and a link's target.
pub(super) struct WorktreeEntry {
    /// The mode Git records:
    /// `100644`,
    /// `100755` or `120000`.
    pub(super) mode: String,
    /// The text of a link's target,
    /// which is what Git stores for a link.
    pub(super) link_target: Option<Vec<u8>>,
}
/// What:
///  The storage facts one path of the worktree answers,
/// or the error of the look.
/// Why:
///   A file that is not there is absent,
/// which is evidence;
/// one that is there and cannot be read is no evidence at all.
/// Git stores a link as the text of its target,
///  so that text is read here,
/// where the file's kind is known,
/// instead of through the link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeFile(path): WorktreeFile | Error;
/// ```
pub(super) fn worktree_file(work: WorktreePath) -> std::io::Result<WorktreeFile> {
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(work.path.as_path()) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(WorktreeFile::Absent),
        Err(error) => return Err(error),
    };
    let linking: bool = metadata.file_type().is_symlink();
    let link_target: Option<Vec<u8>> = if linking {
        use std::os::unix::ffi::OsStringExt as _;
        Some(std::fs::read_link(work.path.as_path())?.into_os_string().into_vec())
    } else {
        None
    };
    #[cfg(unix)]
    let mode: String = {
        use std::os::unix::fs::PermissionsExt as _;
        if linking {
            String::from("120000")
        } else if metadata.permissions().mode() & 0o111 != 0 {
            String::from("100755")
        } else {
            String::from("100644")
        }
    };
    #[cfg(not(unix))]
    let mode: String = String::from("100644");
    return Ok(WorktreeFile::Present(WorktreeEntry { mode, link_target }));
}
/// What:
///  The question `worktree_object` asks Git for one path:
///  the object of the file,
///  or of the text of a link's target.
/// Why:
///  Both faces ask the one question,
/// so a link's object is hashed from its target by both of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = linked ? { arguments: ['hash-object', '--stdin'], input: target } : { arguments: ['hash-object', '--', path] };
/// ```
pub(super) fn hash_request(repository: &Path, path: &OsStr, link_target: Option<&[u8]>) -> GitRequest {
    return match link_target {
        Some(target) => {
            let mut hashing: GitRequest = GitRequest::new(repository, &[OsStr::new("hash-object"), OsStr::new("--stdin")]);
            hashing.input = Some(target.to_vec());
            hashing
        }
        None => GitRequest::new(repository, &[OsStr::new("hash-object"), OsStr::new("--"), path]),
    };
}
/// What:
///  The entry Git's answer to `hash_request` names,
/// with the mode the worktree file carries.
/// Why:
///  Both faces read the object through this one function,
/// so a start Git refused and an object it wrote are the same
///  answer for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function objectAnswered(asked: GitOutput | Error, mode: string): [mode, object]; // throws FailureCode.Unavailable
/// ```
pub(super) fn object_answered(asked: std::io::Result<GitOutput>, mode: &str) -> Result<(String, String), FailureCode> {
    return match asked {
        Ok(answer) if answer.succeeded() => Ok((String::from(mode), String::from_utf8_lossy(answer.stdout.as_slice()).trim().to_string())),
        _ => Err(FailureCode::Unavailable),
    };
}
/// What:
///  The question `deletions_a_landing_could_own` asks Git:
///  the paths the commit's own index records as deleted against the head.
/// Why:
///  Both faces ask the one question,
/// so neither keeps another set of paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['diff-index', '--cached', '--name-only', '--diff-filter=D', '-z', 'HEAD'], index };
/// ```
pub(super) fn deletions_request(repository: &Path, index: &Path) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(repository, &["diff-index", "--cached", "--name-only", "--diff-filter=D", "-z", "HEAD"]);
    request.index = Some(index.to_path_buf());
    return request;
}
/// The names one listing of that question holds,
/// owned so that a job of the blocking pool can take them.
pub(super) struct ListedNames {
    /// The names Git listed,
    /// in its order.
    pub(super) names: Vec<Vec<u8>>,
    /// The worktree the names are looked for in.
    pub(super) repository: PathBuf,
}
/// What:
///  The names of `work` whose file the worktree holds,
/// in the order Git listed them.
/// Why:
///   A deletion the person staged is a file the worktree does not hold,
/// and one a landing left is a file that is still there;
///  the look is a `stat` of every listed name,
/// which waits in the kernel on a mount that stopped answering.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function presentInTheWorktree({ names, repository }) { return names.filter((name) => existsSync(join(repository, name))) }
/// ```
pub(super) fn present_in_the_worktree(work: ListedNames) -> Vec<Vec<u8>> {
    let mut at_risk: Vec<Vec<u8>> = Vec::new();
    for name in &work.names {
        let held: PathBuf = work.repository.join(<OsString as std::os::unix::ffi::OsStringExt>::from_vec(name.clone()).as_os_str());
        if std::fs::symlink_metadata(held.as_path()).is_ok() {
            at_risk.push(name.clone());
        }
    }
    return at_risk;
}
/// The names one listing of `deletions_request` holds,
/// and the failure the repair says when Git gave none.
pub(super) fn listed_deletions(asked: std::io::Result<GitOutput>) -> Result<Vec<Vec<u8>>, String> {
    let output: GitOutput = match asked {
        Ok(answer) if answer.succeeded() => answer,
        _ => return Err(String::from("the deletions this commit would record cannot be read")),
    };
    let mut names: Vec<Vec<u8>> = Vec::new();
    for name in output.stdout.split(is_zero_byte) {
        if name.is_empty() {
            continue;
        }
        names.push(name.to_vec());
    }
    return Ok(names);
}
