//! What:
//!  Whether Git's own record of a branch's remote says a held push's commit has reached it.
//! Why:
//!  A push record is held until the post-commit gate approves it,
//! and nothing settles it once a later push of the branch has carried its
//!  commit.
//! Every command then says the commit is held while `git status` shows it on the remote,
//! so the line and
//!  Git disagree for as long as the record exists.
//! The record is settled here,
//! on the proof that Git already holds the commit in the branch's upstream ref.
//!
//! Proof:
//! a command asks no remote.
//! The question is put to the local repository alone:
//! `<branch>@{upstream}` names the remote-tracking ref the branch's pushes update,
//! and `git merge-base --is-ancestor <commit> <that ref>` says whether the commit is in it.
//! A remote-tracking ref can lag the remote (a push whose reference update was refused still delivered the commits,
//! and a ref that was never fetched says nothing),
//! so a lagging record leaves the held line standing:
//! that costs a line that is still true of the record and loses nothing,
//! while settling on anything weaker would drop the only word about a commit that is not backed up.
//! No upstream at all,
//! a branch outside `refs/heads`,
//! a Git that cannot be started and a question that fails all answer "nothing is known",
//! and the line stands.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const there = onTheRemote(git, cwd, 'refs/heads/main', commit); // 'there' | 'not-there' | 'unknown'
//! ```

use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::path::Path;

/// What a command can say about a held commit and the branch's remote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OnTheRemote = 'there' | 'not-there' | 'unknown';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnTheRemote {
    /// Git's record of the branch's upstream ref contains the commit.
    There,
    /// Git's record of the branch's upstream ref does not contain the commit.
    NotThere,
    /// There is no upstream to ask about,
    /// or the question could not be put.
    Unknown,
}

/// What:
///  The question a command puts to the repository about a held commit.
/// Why:
///   A record's settlement must not depend on Git being there or on a remote being reachable,
/// and a control must be able to
///  say what Git would have said without a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RemoteRecord { onTheRemote(reference: string, commit: string): OnTheRemote }
/// ```
pub trait RemoteRecord {
    /// Whether the commit is in Git's record of `reference`'s upstream ref.
    fn on_the_remote(&self, reference: &str, commit: &str) -> OnTheRemote;
}

/// What:
///  The answer of a repository that is not asked:
/// nothing is known,
/// so nothing is settled.
/// Why:
///   A caller with no Git context,
/// and a control of the line itself,
/// need the held records read
///  exactly as they were before this question existed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nothingAsked = { onTheRemote: () => 'unknown' };
/// ```
pub struct NothingAsked;

impl RemoteRecord for NothingAsked {
    fn on_the_remote(&self, _reference: &str, _commit: &str) -> OnTheRemote {
        return OnTheRemote::Unknown;
    }
}

/// What:
///  The question put to the repository with real Git.
/// Why:
///   The proof is Git's own answer about its own refs,
/// never the wrapper's bookkeeping of what it pushed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class GitRemoteRecord { onTheRemote(reference, commit) { ... } }
/// ```
pub struct GitRemoteRecord<'a> {
    /// The Git program and the environment every child receives.
    pub context: &'a GitContext,
    /// The directory the questions are asked in.
    pub cwd: &'a Path,
}

/// What:
///  The remote-tracking ref a branch's pushes update,
/// as `<remote>/<branch>`,
/// or nothing when the branch has no upstream or Git cannot name it.
/// Why:
///   `@{upstream}` is Git's own answer,
/// so a branch whose upstream was never set is not guessed at from the
///  settings,
/// and the answer is the ref the wrapper's own push plan asks about (`auto_push::plan_named_push`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const upstreamOf = (git, cwd, short) => git(['rev-parse', '--abbrev-ref', '--symbolic-full-name', `${short}@{upstream}`]).trim() || undefined;
/// ```
fn upstream_of(record: &GitRemoteRecord<'_>, short: &str) -> Option<String> {
    let request: GitRequest = GitRequest::new(record.cwd, &["rev-parse", "--abbrev-ref", "--symbolic-full-name", format!("{short}@{{upstream}}").as_str()]);
    let output = run_git(record.context, &request).ok()?;
    if output.code != Some(0) {
        return None;
    }
    let named: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
    if named.is_empty() { return None; }
    return Some(named);
}

/// What:
///  Whether `commit` is an ancestor of the ref `of`,
/// asked of Git.
/// Why:
///   `merge-base --is-ancestor` answers with its exit status alone:
/// `0` is yes,
/// `1` is no,
/// and anything else is a question Git could not
///  answer,
/// which must not read as "no".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const contains = (git, cwd, of, commit) => { const code = git(['merge-base', '--is-ancestor', commit, of]).status; return code === 0 ? 'there' : code === 1 ? 'not-there' : 'unknown'; };
/// ```
fn contains(record: &GitRemoteRecord<'_>, of: &str, commit: &str) -> OnTheRemote {
    let request: GitRequest = GitRequest::new(record.cwd, &["merge-base", "--is-ancestor", commit, of]);
    let Ok(output) = run_git(record.context, &request) else {
        return OnTheRemote::Unknown;
    };
    return match output.code {
        Some(0) => OnTheRemote::There,
        Some(1) => OnTheRemote::NotThere,
        _ => OnTheRemote::Unknown,
    };
}

impl RemoteRecord for GitRemoteRecord<'_> {
    fn on_the_remote(&self, reference: &str, commit: &str) -> OnTheRemote {
        let Some(short) = reference.strip_prefix("refs/heads/") else {
            return OnTheRemote::Unknown;
        };
        let Some(upstream) = upstream_of(self, short) else {
            return OnTheRemote::Unknown;
        };
        return contains(self, upstream.as_str(), commit);
    }
}

/// The proof's controls stay out of the release executable.
#[cfg(test)]
#[path = "held_push_commit_on_the_remote_tests.rs"]
mod tests;
