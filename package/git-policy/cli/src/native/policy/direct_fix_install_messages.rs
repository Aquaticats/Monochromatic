//! What:
//!  The sentences a direct fix's install reports:
//!  every failure and every journal it keeps.
//! Why:
//!  One file holds them so the protocol file stays readable and under its size limit,
//!  and so the
//!       words the person reads are written in one place:
//!  no message names a candidate's pathname,
//!       because a pathname can hold the text a scan exists to keep out of output.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // installFailure(detail); keptNotice(site); unchangedFailure(why); restoreFailed(error);
//! ```

/// The journal a prepared site keeps its bytes in.
use crate::direct_fix_install_journal::{JournalFailure, content_of};
/// The site a message is about.
use crate::direct_fix_install::Prepared;

/// The start of every installation failure message.
pub(crate) const INSTALL_FAILED: &str =
    "monochromatic-git-policy-cli fix could not install its corrections";

/// What:
///  An installation failure message:
///  the shared start,
///  then what happened.
/// Why:
///   Every failure reads the same way and names no candidate file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const installFailure = (detail: string) => `monochromatic-git-policy-cli fix could not install its corrections: ${detail}`;
/// ```
pub(crate) fn install_failure(detail: &str) -> String {
    return format!("{INSTALL_FAILED}: {detail}");
}

/// What:
///  The failure sentence of a file that is not as the fix read it.
///  `String` is the reason the file could
///       not be read,
///  or what it holds now.
/// Why:
///   The search for the file and its refusal read the same way to the person:
///  nothing was changed,
///  and
///       the fix can be run again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unchanged = (why: string) => installFailure(`${why}, so no file was changed.`);
/// ```
pub(crate) fn unchanged_failure(why: String) -> String {
    return install_failure(format!("{why}, so no file was changed.").as_str());
}

/// What:
///  The sentence a journal that stays is reported with,
///  when the bytes in it are another writer's.
/// Why:
///   The person must be able to find the bytes,
///  and the message must not name the candidate's pathname,
///       which can hold the text a scan exists to keep out of output.
///  What the swap displaced is what the other writer saved,
///  and the path itself holds the correction or the bytes the fix put back,
///  so the message names the journal as the place those bytes are
///  (finding of the fifth review).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `kept ${contentOf(journal)} ... another writer saved ... in that journal, not at the file's own path`;
/// ```
pub(crate) fn kept_notice(site: &Prepared) -> String {
    return format!(
        "monochromatic-git-policy-cli kept {}, which holds the bytes another writer saved while the fix ran; nothing there was removed, and those bytes are in that journal, not at the file's own path.",
        content_of(site.journal.as_path()).display()
    );
}

/// What:
///  The sentence a journal that stays is reported with,
///  when nothing proves who wrote the bytes in it.
/// Why:
///   A comparison that could not be made says nothing about its writer,
///  and a rollback that could not move the bytes back leaves them where the fix put them,
///  so the message names the journal and claims no writer at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `kept ${contentOf(journal)} ... the bytes that file held ...`;
/// ```
pub(crate) fn kept_bytes_notice(site: &Prepared) -> String {
    return format!(
        "monochromatic-git-policy-cli kept {}, which holds bytes that file held while the fix was replacing it; nothing there was removed, and they are in that journal, not at the file's own path.",
        content_of(site.journal.as_path()).display()
    );
}

/// What:
///  The sentence a journal a fix could not remove is named with.
/// Why:
///  A removal that fails leaves the journal behind;
///  a fix that reports success without naming it
///       leaves a later command to act on the evidence of a fix that already finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const couldNotRemove = (site) => `... could not be removed ${contentOf(site.journal)} ...`;
/// ```
pub(crate) fn could_not_remove(site: &Prepared) -> String {
    return format!(
        "monochromatic-git-policy-cli corrected the file and then could not be removed {}: the corrected file is in place and the journal that stays holds only the bytes it replaced, so remove it with `rm --recursive` when you like.",
        site.journal.display()
    );
}

/// What:
///  The failure sentence of a journal that could not be made.
/// Why:
///   Both refusals are complete answers for the person,
///  and one of them is not an accident of this
///       machine but a property of the repository's layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = (failure) => installFailure(`${failure}, so no file was changed.`);
/// ```
pub(crate) fn journal_refusal(failure: JournalFailure) -> String {
    return install_failure(format!("{failure}, so no file was changed.").as_str());
}
