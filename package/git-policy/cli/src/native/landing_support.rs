//! What:
//!  Target comparison, the nonce-bearing reflog message and the compare-and-swap a landing uses
//!  to advance its target.
//! Why:
//!  The reflog subject carries the transaction nonce, so recovery can tell whether the target
//!  advanced to this landing even after a crash (`src/policy-engine/commit-landing-support.ts`).
//!  The swap runs in the owning worktree's context, which also writes the real `HEAD` reflog.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await compareAndSwapTarget({ gitPath, cwd, capture, transactionId, newOid, expectedOld });
//! ```

/// Journal target value.
use super::transaction_journal::Base;
/// Strict-enough JavaScript whitespace trimming for Git's subject text.
use super::js_text::trim_javascript;
/// Running real Git through the transaction runner.
use super::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Reports whether two target values are equal.
/// Why:
///   An unborn target equals only another unborn target.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sameTarget({ left, right });
/// ```
pub fn same_target(left: &Base, right: &Base) -> bool {
    return left == right;
}

/// What:
///  The reflog subject prefix carrying the transaction nonce.
/// Why:
///   Recovery searches the reflog for this prefix to find an advanced target.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landingReflogPrefix(nonce); // 'commit (cli-git <id>):'
/// ```
pub fn landing_reflog_prefix(nonce: &str) -> String {
    return format!("commit (cli-git {nonce}):");
}

/// What:
///  Builds the nonce-bearing reflog message of one landed commit.
/// Why:
///   The message is the prefix followed by the commit's subject, as Git's own commit reflog writes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await landingReflogMessage({ gitPath, cwd, nonce, oid });
/// ```
pub fn landing_reflog_message(
    context: &GitContext,
    cwd: &Path,
    nonce: &str,
    oid: &str,
) -> Result<String, GitFailure> {
    let request = GitRequest::new(cwd, &["show", "--no-patch", "--format=%s", oid]);
    let output: GitOutput = run_git(context, &request)
        .map_err(|error| { return GitFailure(format!("git show {oid} could not start: {error}")); })?;
    if !output.succeeded() {
        return Err(GitFailure(format!(
            "git show {oid} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim_end()
        )));
    }
    let subject: String = String::from_utf8(output.stdout)
        .map_err(|error| { return GitFailure(format!("git show {oid} printed non-UTF-8 text: {error}")); })?;
    return Ok(format!("{} {}", landing_reflog_prefix(nonce), trim_javascript(&subject)));
}

/// What:
///  One compare-and-swap of a landing's target.
/// Why:
///   Grouping the facts keeps the call site readable and names what the swap depends on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { targetRef, symbolicHead, transactionId, newOid, expectedOld, emptyTreeOid }
/// ```
pub struct CompareAndSwap<'swap> {
    /// Ref to move: the branch, or `HEAD` when detached.
    pub target_ref: &'swap str,
    /// Whether `HEAD` is detached, so `--no-deref` moves `HEAD` itself.
    pub detached: bool,
    /// Transaction ID written into the reflog message.
    pub nonce: &'swap str,
    /// Commit to land.
    pub new_oid: &'swap str,
    /// Value the target must still hold.
    pub expected_old: &'swap Base,
    /// Hash length in hex digits, which sets the width of the all-zero unborn value.
    pub hash_length: usize,
}

/// What:
///  Advances the target from the expected value to the new commit, in the owning worktree.
/// Why:
///   A nonzero exit is a lost race, not an error: the caller reads the winning value next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, args: ['update-ref', '-m', message, ...], allowFailure: true });
/// ```
pub fn compare_and_swap_target(
    context: &GitContext,
    cwd: &Path,
    swap: &CompareAndSwap<'_>,
) -> Result<GitOutput, GitFailure> {
    let message: String = landing_reflog_message(context, cwd, swap.nonce, swap.new_oid)?;
    let expected: String = match swap.expected_old {
        Base::Commit(oid) => oid.clone(),
        Base::Unborn => "0".repeat(swap.hash_length),
    };
    let mut arguments: Vec<String> = vec!["update-ref".to_string(), "-m".to_string(), message];
    if swap.detached {
        arguments.push("--no-deref".to_string());
    }
    arguments.push(swap.target_ref.to_string());
    arguments.push(swap.new_oid.to_string());
    arguments.push(expected);
    let request = GitRequest::new(cwd, &arguments);
    return run_git(context, &request)
        .map_err(|error| { return GitFailure(format!("git update-ref could not start: {error}")); });
}

/// Landing support controls stay out of the production code path.
#[cfg(test)]
#[path = "landing_support_tests.rs"]
mod tests;
