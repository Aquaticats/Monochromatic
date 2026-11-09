//! What:
//!  The sweep of the journals an interrupted fix left:
//!  what it removes,
//!  what it keeps and names,
//! and the
//!       sentences it uses.
//! Why:
//!  `direct_fix_install_journal.rs` holds the journal's shape and record and stays under the
//!  file-size limit;
//!  the sweep is the one place that reads journals,
//!  so it is also the one place that has to read
//!  every name's type before anything else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // reapInterruptedInstalls(gitDir, notices);
//! ```

/// The permission bits,
///  owner and extended attributes a journal's record names.
use crate::direct_fix_install_writes::Kept;
/// The journal's own shape,
/// record and removal.
use crate::direct_fix_install_journal::{JOURNAL_PREFIX, content_of, read_record, remove};
/// The proof that the process which wrote a record still runs.
use crate::worktree_copy_owner::token_owner_is_dead;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// The directories of a Git directory that start with the journal prefix,
///  sorted by name.
fn journals_below(git_dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let Ok(listed) = std::fs::read_dir(git_dir) else {
        return found;
    };
    for entry in listed.flatten() {
        let path: PathBuf = entry.path();
        if !entry.file_name().to_string_lossy().starts_with(JOURNAL_PREFIX) {
            continue;
        }
        if path.is_dir() {
            found.push(path);
        }
    }
    found.sort();
    return found;
}

/// What:
///  Look at every journal a killed fix left in `git_dir`,
///  remove only the ones that provably hold none of
///       the person's bytes,
///  and name every other one.
///  `&mut Vec<String>` collects one sentence per journal that was
///       removed or kept.
/// Why:
///  A journal is the only record of what an interrupted fix did,
///  and its `content` may be the
///  person's only copy of a file.
///  Nothing here swaps two files:
///  a recovery that exchanged would need a
///  proof that the exchange it is undoing happened,
///  and the destination's bytes are not that proof
///  (a person can save the correction's own bytes),
///  nor is a reading of the destination a licence to swap -  a save
///  that lands between the reading and the swap moves into the journal and is then removed with it,
///  and two recoveries can exchange one journal twice.
///  So the only removal is the one the record proves:
/// a `content`
///  that hashes to the correction itself holds none of the person's bytes,
/// because a fix writes exactly
///  those bytes and nothing else there.
///  Everything else stays and is named on every later command.
///  An owner the
///  platform will not report,
///  and an owner of another PID namespace whose record has outlived the
///  patience,
///  prove nothing,
///  so their journals stay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const journal of journalsBelow(gitDir)) { if (!provenGone(journal)) continue; if (digestOf(journal.content) === journal.record.replacement) removeOrName(journal); else notices.push(reportedLeftover(journal)); }
/// ```
pub(crate) fn reap_interrupted_installs(git_dir: &Path, notices: &mut Vec<String>) {
    for journal in journals_below(git_dir) {
        let Some(record) = read_record(journal.as_path()) else {
            notices.push(format!(
                "monochromatic-git-policy-cli found {} from a fix whose process ended, and its record cannot be read, so nothing was done with it. Look at it and remove it with `rm --recursive` once you have what you need.",
                journal.display()
            ));
            continue;
        };
        // What: Only a proof that the writer is gone allows even looking at the bytes; a refusal of another
        //       namespace's expired record, and a platform that will not report, name nothing.
        // Why:  A live command's journal is that command's business.
        //
        // In TS you'd write (pseudocode):
        // const dead = ownerIsDead(record.owner, journal); // false when unprovable
        let Some(owner) = record.owner.as_deref() else {
            notices.push(format!(
                "monochromatic-git-policy-cli found {} from a fix whose process ended, and its record names no writer, so nothing was done with it. Look at it and remove it with `rm --recursive` once you have what you need.",
                journal.display()
            ));
            continue;
        };
        let proven_gone: bool =
            token_owner_is_dead(Some(owner), journal.as_path()).unwrap_or(false);
        if !proven_gone {
            continue;
        }
        let content: PathBuf = content_of(journal.as_path());
        // What: The correction first, judged as the rollback judges a file it took out of a swap: a regular file
        //       whose bytes, permission bits and owner are the ones the record names. The type is read before any byte.
        // Why:  A digest match alone does not say whose file it is: the person can save the correction's own bytes and
        //       narrow them to `0600`, and the swap then moves their file into the journal. A name that is not a regular
        //       file is never opened, so a FIFO cannot hold the sweep.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const recorded = { mode: record.mode, uid: record.uid, gid: record.gid };
        // if (!holdsDigest(content, record.replacement, recorded, 'reading what a killed fix left')) { notices.push(reportedLeftover(journal)); continue; }
        // ```
        let recorded: Kept = Kept {
            mode: record.mode,
            uid: record.uid,
            gid: record.gid,
            attributes: record.attributes.clone(),
        };
        match crate::direct_fix_install_judging::holds_digest(
            content.as_path(),
            record.replacement.as_str(),
            &recorded,
            "reading what a killed fix left",
        ) {
            Ok(true) => {}
            Ok(false) => {
                // What it holds is not exactly the correction, so it may be the person's: it stays, and the file it
                // belongs to is not touched.
                notices.push(reported_leftover(journal.as_path()));
                continue;
            }
            Err(detail) => {
                notices.push(format!("{detail}; {}", reported_leftover(journal.as_path())));
                continue;
            }
        }
        // The correction itself: recomputable, and none of the person's bytes. Removing it changes no file.
        if remove(journal.as_path()) {
            notices.push(format!(
                "monochromatic-git-policy-cli removed {}: a fix whose process ended had not reached its file, so nothing had changed.",
                journal.display()
            ));
            continue;
        }
        notices.push(format!(
            "monochromatic-git-policy-cli could not be removed {}: a fix whose process ended had not reached its file, so it holds only a correction you can recompute; remove it with `rm --recursive` when you like.",
            journal.display()
        ));
    }
    return;
}

/// What:
///  The sentence a journal that stays is named with,
///  for a control that compares the whole message.
/// Why:
///   The sentences are the contract the person reads,
///  and one function keeps the words in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reportedLeftover = (journal) => `... ${journal} ...`;
/// ```
pub fn reported_leftover(journal: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli left {} as it is: it holds bytes from a fix whose process ended, and nothing here proves the file it belongs to is the fix's own, so nothing was changed and the bytes stay for you to look at.",
        journal.display()
    );
}
