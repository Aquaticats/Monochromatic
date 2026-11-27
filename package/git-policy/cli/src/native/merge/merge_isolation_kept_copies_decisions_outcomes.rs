//! What:
//!  What one decision about a kept copy does:
//!  put the copy back where it belongs,
//!  remove it because the person's own file holds its bytes again,
//!  or remove it because Git holds them,
//!  and the line each of the three prints.
//! Why:
//!   The decision is one,
//! and what comes of it is three,
//!  each with its own words:
//! a copy put back is a file the merge goes on from,
//! a copy removed is a file whose bytes the person can reach,
//!  so the line says where they are and how to get them back.
//!  A removal that fails says so and names the copy instead of losing it.
//!  The three are a file of their own so the decision that chooses between them
//! stays inside the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await putTheCopyBack({ gitDir, copy, named, out }); await removeTheCopyBecauseTheFileHoldsIt({ gitDir, copy, proof, out });
//! ```

/// The one job of the blocking pool that makes the step's file calls,
/// so a disk that makes them wait holds no worker of the runtime.
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// The copies a merge kept,
/// and the record of each.
use crate::merge_isolation_kept_copies::KeptCopy;
use crate::merge_isolation_kept_copies::forget_the_copy;
/// The lines a kept copy earns.
use crate::merge_isolation_kept_copies_words::say_the_copy_stays;
/// The proof that no program can write into a file whose last name is about to go.
use crate::merge_isolation_take_asides_proof::ProofNothingCanWrite;
/// The step that puts one name back where it was found,
/// as a job of the blocking pool.
use crate::rename_that_never_replaces::{
    TheAnswerOfARename,
    rename_noreplace_on_the_blocking_pool,
};
/// Writing the lines a person reads.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Put one copy back where it belongs,
///  and say so.
/// Why:
///   The copy is the file the merge judged at that path,
/// so the path holds it again and the merge goes on
///  from the checkout it expected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const putBack = async (aside, location) => { const moved = await renameNoReplaceOnThePool(aside, location); moved.kind === 'renamed' ? forget(gitDir, aside) : sayTheCopyStays(...); };
/// ```
pub(crate) async fn put_the_copy_back_awaiting<W: Write + ?Sized>(
    git_dir: &Path,
    copy: &KeptCopy,
    named: bool,
    out: &mut W,
) -> bool {
    match rename_noreplace_on_the_blocking_pool(copy.aside.as_path(), copy.location.as_path()).await
    {
        TheAnswerOfARename::Renamed => {
            forget_the_copy(git_dir, copy.aside.as_path());
            let _said = out.write_all(
                format!(
                    "monochromatic-git-policy-cli: put back {}, which a command that stopped while finishing a merge had set aside; its \
                     bytes are the ones it had.\n",
                    copy.location.display()
                )
                .as_bytes(),
            );
            return true;
        }
        // The path was taken between the reading and the rename, so both stay and the copy is named; a rename the pool did not
        //  make leaves both as they are, and the copy is named the same way.
        TheAnswerOfARename::NameTaken(_)
        | TheAnswerOfARename::Failed(_)
        | TheAnswerOfARename::NoThreadToRunIt(_) => {
            say_the_copy_stays(out, copy, named);
            return false;
        }
    }
}

/// What:
///  Remove one copy whose bytes the person's file holds again,
///  and say so.
/// Why:
///   Two files holding one run of bytes are one too many,
/// and the copy beside the path is the one to go:
///  the person's file is where they look for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm(aside); forget(gitDir, aside); say(`${aside} was removed: ${location} holds those same bytes`);
/// ```
pub(crate) async fn remove_the_copy_because_the_file_holds_it_awaiting<W: Write + ?Sized>(
    git_dir: &Path,
    copy: &KeptCopy,
    proof: ProofNothingCanWrite,
    out: &mut W,
) {
    match proof
        .remove_on_the_blocking_pool(copy.aside.as_path())
        .await
    {
        Ok(()) => {
            forget_the_copy(git_dir, copy.aside.as_path());
            let _said = out.write_all(
                format!(
                    "monochromatic-git-policy-cli: removed {}, a copy of {} that a merge set aside: {} holds those same bytes again, so \
                     the copy was the second one and nothing of yours was lost.\n",
                    copy.aside.display(),
                    copy.location.display(),
                    copy.location.display()
                )
                .as_bytes(),
            );
        }
        Err(error) => {
            let _said = out.write_all(
                format!("monochromatic-git-policy-cli: {} could not be removed ({error}); it still holds a copy of {}.\n", copy.aside.display(), copy.location.display())
                    .as_bytes(),
            );
        }
    }
}

/// What:
///  Remove one copy whose bytes one of the merge's commits holds,
///  and say where they are.
/// Why:
///   The bytes are in the repository,
/// so the copy beside the path is the second one,
/// and the line names the
///  commit and the command that puts those bytes back in the checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm(aside); forget(gitDir, aside); say(`${aside} was removed: ${oid} holds its bytes`);
/// ```
pub(crate) async fn remove_the_copy_because_git_holds_it_awaiting<W: Write + ?Sized>(
    git_dir: &Path,
    copy: &KeptCopy,
    object: &str,
    out: &mut W,
) {
    // The removal is a file call that can wait on a disk, so a job of the blocking pool makes it.
    let copy_at: PathBuf = copy.aside.clone();
    let removed: std::io::Result<()> = match run_blocking_job_within_the_budget(move || {
        return std::fs::remove_file(copy_at.as_path());
    })
    .await
    {
        Ok(outcome) => outcome,
        // The system gave the pool no thread, so the name may still be there and the copy is said to stay as it is.
        Err(refused) => Err(std::io::Error::other(refused.to_string())),
    };
    match removed {
        Ok(()) => {
            forget_the_copy(git_dir, copy.aside.as_path());
            let _said = out.write_all(
                format!(
                    "monochromatic-git-policy-cli: removed {}, a copy of {} that a merge set aside: {} holds other bytes, and {} holds \
                     the bytes of the copy, so nothing of yours was lost; `git restore --no-enforce-worktree --source={} -- {}` puts \
                     them in the checkout if you want them.\n",
                    copy.aside.display(),
                    copy.location.display(),
                    copy.location.display(),
                    object,
                    object,
                    copy.location.display()
                )
                .as_bytes(),
            );
        }
        Err(error) => {
            let _said = out.write_all(
                format!(
                    "monochromatic-git-policy-cli: {} could not be removed ({error}); it still holds a copy of {}, whose bytes {} \
                     holds.\n",
                    copy.aside.display(),
                    copy.location.display(),
                    object
                )
                .as_bytes(),
            );
        }
    }
}
