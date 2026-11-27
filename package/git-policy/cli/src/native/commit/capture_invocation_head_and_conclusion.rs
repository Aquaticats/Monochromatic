//! What:
//!  What the owning worktree says about the commit being made:
//!  the symbolic `HEAD` target,
//!  and the conclusion kind an amend or a sequencer marker decides.
//! Why:
//!  The landing reads `HEAD` afresh at landing time,
//!  so the ref file answers when it is one of the two plain spellings of the files backend,
//!  and anything else asks Git.
//!  A sequencer marker is resolved rather than detected:
//!  a marker naming a missing object still decides a conclusion,
//!  so a reftable repository answers as a file-backed one does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! readHeadFile(refFiles) ?? (await resolveSymbolicHeadWithGit({ gitPath, cwd }));
//! ```

/// The questions of this capture,
///  one of whose failed start is an answer.
use crate::capture_invocation_git_questions::probe;
/// The files-backend `HEAD` reader and where one worktree's ref files live.
use crate::ref_files::{
    RefFilesLayout,
    read_symbolic_head,
};
/// Proof that a sequencer marker names nothing,
/// without a Git process.
use crate::sequencer_marker::markers_maybe_present;
/// The transaction Git wrapper:
///  the context a question is asked in,
///  the failure its answer is reported as,
///  and the output it prints.
use crate::transaction_git::{
    GitContext,
    GitFailure,
    GitOutput,
};
/// Journal enums shared with the transaction record:
///  the symbolic `HEAD` and the conclusion.
use crate::transaction_journal::{
    Conclusion,
    SymbolicHead,
};
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// What:
///  Reports whether a pseudoref names an object in the owning worktree.
/// Why:
///   `MERGE_HEAD`,
/// `CHERRY_PICK_HEAD` and `REVERT_HEAD` decide the conclusion kind
///  by resolution,
/// not by file presence,
/// so a reftable repository answers the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await runTransactionGit({ args: ['rev-parse', '--verify', '--quiet', name], allowFailure: true })).exitCode === 0
/// ```
fn pseudoref_exists(
    context: &GitContext,
    cwd: &Path,
    name: &str,
) -> Result<bool, GitFailure> {
    return Ok(probe(context, cwd, &["rev-parse", "--verify", "--quiet", name])?.code == Some(0));
}

/// What:
///  Resolves the commit kind from the amend flag and the owning worktree's conclusion state.
/// Why:
///   An amend always wins;
///  otherwise the first present marker in the incumbent's order decides.
///  A marker whose every lookup place is empty on the files backend is absent without asking Git;
///  any entry in any place,
/// of any content,
/// is resolved by `rev-parse --verify`,
///  so a marker naming a missing object stays a conclusion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await resolveConclusionKind({ gitPath, cwd, amend });
/// ```
pub fn resolve_conclusion_kind(
    context: &GitContext,
    cwd: &Path,
    amend: bool,
    ref_files: &RefFilesLayout<'_>,
) -> Result<Conclusion, GitFailure> {
    if amend {
        return Ok(Conclusion::Amend);
    }
    let names: [&str; 3] = ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD"];
    let kinds: [Conclusion; 3] = [
        Conclusion::Merge,
        Conclusion::CherryPick,
        Conclusion::Revert,
    ];
    let maybe_present: Vec<bool> = markers_maybe_present(ref_files, &names);
    for ((name, kind), maybe) in names.into_iter().zip(kinds).zip(maybe_present) {
        if maybe && pseudoref_exists(context, cwd, name)? {
            return Ok(kind);
        }
    }
    return Ok(Conclusion::None);
}

/// What:
///  Resolves the symbolic `HEAD` target in the owning worktree.
/// Why:
///   The `HEAD` file answers when it is one of the two plain spellings on the files backend,
///  read afresh on every call so the landing's re-read stays a read at landing time;
///  anything else asks Git,
///  where exit 1 from `symbolic-ref --quiet` means detached and any other failure is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readHeadFile(refFiles) ?? (await resolveSymbolicHeadWithGit({ gitPath, cwd }));
/// ```
pub fn resolve_symbolic_head(
    context: &GitContext,
    cwd: &Path,
    ref_files: &RefFilesLayout<'_>,
) -> Result<SymbolicHead, GitFailure> {
    if let Some(read) = read_symbolic_head(ref_files) {
        return Ok(read);
    }
    let output: GitOutput = probe(context, cwd, &["symbolic-ref", "--quiet", "HEAD"])?;
    if output.code == Some(1) {
        return Ok(SymbolicHead::Detached);
    }
    if output.code != Some(0) {
        return Err(GitFailure(format!(
            "git symbolic-ref HEAD failed: {}",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        )));
    }
    // The name is the bytes Git printed: a branch name is a byte string, and a reading that replaced a byte
    //  would name another ref (`ref_name.rs`).
    return Ok(SymbolicHead::Branch(
        crate::ref_name::RefName::from_git_output(output.stdout.as_slice()),
    ));
}
