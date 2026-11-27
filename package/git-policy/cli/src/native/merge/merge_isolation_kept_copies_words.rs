//! What:
//!  The lines a kept copy earns:
//!  one saying it stays where it is and what ends it,
//!  with the reason it stays when the file
//!  at its path holds its bytes but not everything else it holds,
//!  naming the programs that hold it open when they are what keeps it.
//! Why:
//!  A person reading a reason that is not theirs cannot judge whether the copy is theirs to drop,
//! and the file that
//!  decides a copy is at its code-line limit,
//! so the decision's words live here
//! (`merge_isolation_kept_copies_decisions.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! sayTheCopyStays({ out, copy, named }); sayTheCopyStaysBecause({ out, copy, why }); sayTheCopyStaysHeldOpen({ out, copy, whatHoldsTheCopy });
//! ```

/// `PID <n> (<command>)`,
/// the one shape a process is named in.
use crate::index_lock_evidence_words::holder_name;
/// One process a scan found holding a file open.
use crate::index_lock_holders::Holder;
/// One copy a merge kept,
/// with the worktree path it belongs to.
use crate::merge_isolation_kept_copies::KeptCopy;
/// Writing the lines a person reads.
use std::io::Write;

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
pub(crate) fn say_the_copy_stays<W: Write + ?Sized>(
    out: &mut W,
    copy: &KeptCopy,
    named: bool,
) {
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
pub(crate) fn say_the_copy_stays_because<W: Write + ?Sized>(
    out: &mut W,
    copy: &KeptCopy,
    why: &str,
) {
    let _said = out.write_all(
        the_copy_stays_sentence(copy, why, format!("{why} is no longer true").as_str()).as_bytes(),
    );
}

/// What:
///  Say that a copy stays because a program this command can read holds it open,
///  naming each of them by its process number and command.
/// Why:
///   A refusal that says only that some program holds the copy leaves the person with no way
///   to find it:
/// the copy goes once that program ends,
/// and only the program's own name says which one that is
/// (issue 6542,
/// whose failure named no process at all).
///  The line keeps what ends a copy that is held:
/// a later command removes it by itself once no process this command can read holds it open,
/// so the person may also end the program and run the command again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// say(`${aside} holds a copy of ${location}, and ${location} holds those same bytes, but a program this command can read still holds the copy open: ${holders}, so the copy stays ...`);
/// ```
pub(crate) fn say_the_copy_stays_held_open<W: Write + ?Sized>(
    out: &mut W,
    copy: &KeptCopy,
    holders: &[Holder],
) {
    let _said = out.write_all(
        the_copy_stays_sentence(
            copy,
            format!(
                "a program this command can read still holds the copy open: {}",
                the_holders(holders)
            )
            .as_str(),
            "no process this command can read holds it open",
        )
        .as_bytes(),
    );
}

/// What:
///  The sentence a copy that stays is named with,
///  given what keeps it and what ends it.
/// Why:
///   The two lines that say a copy stays where the file at its path holds its bytes differ in the
///  reason alone,
/// so the sentences a person reads that name the copy,
/// its path and the two commands that end it
/// are written once here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${aside} holds a copy of a file, and ${location} holds those same bytes, but ${but}, so the copy stays ...`
/// ```
fn the_copy_stays_sentence(
    copy: &KeptCopy,
    but: &str,
    ends_when: &str,
) -> String {
    return format!(
        "monochromatic-git-policy-cli: {} holds a copy of a file, and {} holds those same bytes, but {but}, so the copy stays where it is \
         and nothing of yours was removed. `mv -- {} {}` puts the copy back over what is there, `rm -- {}` drops the copy, \
         and a later command decides it again once {ends_when}.\n",
        copy.aside.display(),
        copy.location.display(),
        copy.aside.display(),
        copy.location.display(),
        copy.aside.display()
    );
}

/// What:
///  The processes a scan named,
/// each as `PID <n> (<command>)`
///  and separated by a comma.
/// Why:
///   A person reading the line ends the programs it names,
/// and a line naming one of three leaves the copy where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holders.map(({ pid, command }) => `PID ${pid}${command ? ` (${command})` : ''}`).join(', ')
/// ```
fn the_holders(holders: &[Holder]) -> String {
    let named: Vec<String> = holders
        .iter()
        .map(|holder| return holder_name(holder.pid, holder.command.as_deref()))
        .collect();
    return named.join(", ");
}

/// Controls of these sentences stay out of the release executable.
#[cfg(test)]
#[path = "merge_isolation_kept_copies_words_tests.rs"]
mod tests;
