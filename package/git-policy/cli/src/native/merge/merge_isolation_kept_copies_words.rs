//! What:
//!  The lines a kept copy earns:
//!  one saying it stays where it is and what ends it,
//!  with the reason it stays when the file
//!  at its path holds its bytes but not everything else it holds.
//! Why:
//!  A person reading a reason that is not theirs cannot judge whether the copy is theirs to drop,
//! and the file that
//!  decides a copy is at its code-line limit,
//! so the decision's words live here
//! (`merge_isolation_kept_copies_decisions.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! sayTheCopyStays({ out, copy, named }); sayTheCopyStaysBecause({ out, copy, why });
//! ```

/// Writing the lines a person reads.
use std::io::Write;
/// One copy a merge kept,
/// with the worktree path it belongs to.
use crate::merge_isolation_kept_copies::KeptCopy;

/// What:
///  Say that a copy stays,
///  where it is,
///  what the path holds,
///  and what ends it.
/// Why:
///   The person is the only one who can judge bytes nothing else holds,
/// so the line names the copy,
///  the path it belongs to,
///  the two commands that end it,
/// and the end a later command performs by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// say(`${aside} holds a copy of ${location} ...; mv puts it back, rm drops it, and a later command removes it once ${location} holds its bytes`);
/// ```
pub(crate) fn say_the_copy_stays<W: Write + ?Sized>(out: &mut W, copy: &KeptCopy, named: bool) {
    let whose: &str = if named {
        "A merge in this worktree that has not finished set it aside for"
    } else {
        "A merge set it aside for"
    };
    let _said = out.write_all(
        format!(
            "monochromatic-git-policy-cli: {} holds a copy of a file, and {} holds something else, so the copy stays where it is and \
             nothing of yours was removed. {whose} that path. `mv -- {} {}` puts the copy back in the checkout, `rm -- {}` drops the \
             copy, and a later command removes the copy by itself once {} holds its bytes again.\n",
            copy.aside.display(),
            copy.location.display(),
            copy.aside.display(),
            copy.location.display(),
            copy.aside.display(),
            copy.location.display()
        )
        .as_bytes(),
    );
}

/// What:
///  Say that a copy stays because the path holds its bytes but not everything else the copy holds,
///  with that reason,
///  the two commands that end it,
/// and the end a later command performs by itself.
/// Why:
///   The line about a copy that stays said the path held something else,
/// which is not true when the bytes are
///  the same and the mode or an open writer is what keeps the copy;
/// a person reading a reason that is not theirs cannot
///  judge whether the copy is theirs to drop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// say(`${aside} holds a copy of ${location}, and ${location} holds those same bytes, but ${why}, so the copy stays ...`);
/// ```
pub(crate) fn say_the_copy_stays_because<W: Write + ?Sized>(out: &mut W, copy: &KeptCopy, why: &str) {
    let _said = out.write_all(
        format!(
            "monochromatic-git-policy-cli: {} holds a copy of a file, and {} holds those same bytes, but {why}, so the copy stays \
             where it is and nothing of yours was removed. `mv -- {} {}` puts the copy back over what is there, `rm -- {}` drops \
             the copy, and a later command decides it again once {why} is no longer true.\n",
            copy.aside.display(),
            copy.location.display(),
            copy.aside.display(),
            copy.location.display(),
            copy.aside.display()
        )
        .as_bytes(),
    );
}
