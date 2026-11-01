//! What:
//!  The reflog search that proves which commit an interrupted transaction landed.
//! Why:
//!  Before `ref-updated.json` exists,
//!  a landing counts as landed only when the target's
//!      reflog holds the transaction's nonce entry;
//!  the search covers the whole reflog because
//!      later landings push the entry down (`src/policy-engine/commit-transaction-recovery-reflog.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const oids = await listNonceReflogOids({ gitPath, cwd, ref, subjectPrefix });
//! ```

/// The fail-closed recovery failure.
use super::recovery_error::RecoveryError;
/// Running real Git.
use super::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The distinct commits of `%H%x00%gs` reflog lines whose subject starts with
///       `subject_prefix`,
///  in first-seen order.
/// Why:
///   Git's `--grep-reflog` matches anywhere in the subject;
///  only an entry that starts with
///       the nonce-bearing action is the transaction's own movement.
///  A line without a NUL is
///       malformed output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nonceOids(output: string, subjectPrefix: string): string[];
/// ```
pub fn nonce_oids(output: &str, subject_prefix: &str) -> Result<Vec<String>, RecoveryError> {
    // `mut` allows collecting distinct commits in order.
    let mut oids: Vec<String> = Vec::new();
    for line in output.split('\n') {
        if line.is_empty() {
            continue;
        }
        let Some((oid, subject)) = line.split_once('\0') else {
            return Err(RecoveryError(String::from(
                "Git returned malformed transaction reflog output.",
            )));
        };
        let owned: String = String::from(oid);
        if subject.starts_with(subject_prefix) && !oids.contains(&owned) {
            oids.push(owned);
        }
    }
    return Ok(oids);
}

/// What:
///  The distinct commits whose reflog entries on `reference` carry the nonce-bearing
///       subject prefix,
///  at any depth.
/// Why:
///   `listNonceReflogOids`:
///  a reflog Git cannot read fails closed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function listNonceReflogOids({ gitPath, cwd, ref, subjectPrefix }): Promise<readonly string[]>;
/// ```
pub fn list_nonce_reflog_oids(
    context: &GitContext,
    cwd: &Path,
    reference: &str,
    subject_prefix: &str,
) -> Result<Vec<String>, RecoveryError> {
    let grep: String = format!("--grep-reflog={subject_prefix}");
    let request: GitRequest = GitRequest::new(
        cwd,
        &[
            "reflog",
            "show",
            "--fixed-strings",
            grep.as_str(),
            "--format=%H%x00%gs",
            reference,
            "--",
        ],
    );
    let output: GitOutput = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(RecoveryError(format!(
                "{reference} reflog is unreadable: {error}"
            )));
        }
    };
    if !output.succeeded() {
        return Err(RecoveryError(format!(
            "{reference} reflog is unreadable: {}",
            output.error_text()
        )));
    }
    let Ok(text) = std::str::from_utf8(output.stdout.as_slice()) else {
        return Err(RecoveryError(format!(
            "{reference} reflog output is not UTF-8."
        )));
    };
    return nonce_oids(text, subject_prefix);
}

/// Parser and real-reflog controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_reflog_tests.rs"]
mod tests;
