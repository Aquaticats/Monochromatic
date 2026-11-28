//! What:
//! The name one install writes a link under while it is between its two steps,
//! `.<link name>.tmp.<process id>.<start time>.<namespace>.<boot>`,
//! and what that name says about the install that is still holding the link.
//! Why:
//! The link write makes the link under a temporary name and renames it over the final one,
//! so a kill between the two leaves a dotted name in the exposure farm;
//! the next rebuild removes such a name and says so,
//! and a second install rebuilding the farm at the same instant must not take a link an install that still runs is
//! about to rename.
//! A process id alone answers that wrongly
//! (the system hands those numbers out again, and a number means one thing only inside the process namespace that
//! gave it out),
//! which is why the name carries the same four facts a hidden sibling's name carries
//! (`archive_staging`,
//! `the_owner_of_a_hidden_sibling`),
//! and why the question is asked the same way.
//! The name is built by the one function the hidden names' own builder provides,
//! so the two shapes cannot drift apart.
//! A name written by `0.2.7-alpha.18` and earlier carries the process id alone:
//! a number no process holds is an install that certainly ended,
//! and a number still held may be that install or a program that took the number over,
//! so the link stays and a later rebuild asks again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-temporary-link-of-an-installer.ts
//! function theTemporaryLinkOf(linkPath: string): string;
//! function livenessOfTheInstaller(name: string): 'running' | 'gone' | 'unknown' | undefined;
//! ```

/// Imports `Path` from `std::path`,
/// the borrowed path type,
/// and `PathBuf`,
/// its owned sibling.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the builder of a hidden name,
/// which holds the one shape both a hidden sibling and a temporary link carry.
use super::super::archive_staging::hidden_sibling;
/// Imports whether the boot field of a name is the identifier the kernel writes.
use super::super::archive_staging::is_boot_id;
/// Imports what a process's record and the kernel's own facts say,
/// and the three-valued answer the owner judgement gives.
use super::super::archive_staging::{
    Liveness,
    Owner,
};

/// The purpose in a temporary link's name,
/// which says what the name holds and which no other reader of hidden names knows.
pub(crate) const TEMPORARY_PURPOSE: &str = "tmp";

/// What:
/// The name a link is written under while the install whose facts `owner` holds is between its two steps.
/// Why:
/// The name has to be unique to that install and say which install holds it,
/// so a later rebuild can tell a link it may remove from one an install still runs with;
/// the link write passes the record of the program running now,
/// and a control passes the record of a process of its own choosing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theTemporaryLinkOf(linkPath: string, owner: Owner): string
/// ```
pub(crate) fn the_temporary_link_of(
    link_path: &Path,
    owner: Owner,
) -> PathBuf {
    return hidden_sibling(link_path, TEMPORARY_PURPOSE, owner);
}

/// What:
/// What this process can say about the install that wrote the temporary link `name`,
/// or nothing when `name` is not a temporary link's name at all.
/// Why:
/// A rebuild asks this of every name no row declares:
/// `undefined` means the name is an ordinary orphan and may be removed,
/// a live or unjudgeable install means the link may still be renamed and must stay,
/// and an install that ended means the name is a leftover to remove and report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function livenessOfTheInstaller(name: string): 'running' | 'gone' | 'unknown' | undefined
/// ```
pub(crate) fn liveness_of_the_installer(name: &str) -> Option<Liveness> {
    // What: the name without its leading dot, split at every dot it holds.
    // Why: the link's own name may hold dots, so every field is read from the right.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parts = name.replace(/^\./, '').split('.');
    // ```
    let stem: &str = name.strip_prefix('.')?;
    let parts: Vec<&str> = stem.split('.').collect();
    // What: the shape this build writes,
    // which ends in the purpose, the process id, its start time, the process namespace and the boot.
    // Why: those four facts are the only ones that tell an install still holding this link from a program that took
    //  its number over, in the boot and the namespace that gave the number out.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [temporary, pid, started, namespace, boot] = parts.slice(-5);
    // if (temporary === 'tmp' && isPid(pid) && isMoment(started) && isNamespace(namespace) && isBootId(boot)) { ... }
    // ```
    if let [carried @ .., temporary, pid, started, namespace, boot] = parts.as_slice()
        && !carried.is_empty()
        && *temporary == TEMPORARY_PURPOSE
        && let Ok(number) = pid.parse::<u32>()
        && let Ok(moment) = started.parse::<u64>()
        && let Ok(space) = namespace.parse::<u64>()
        && is_boot_id(boot)
    {
        let owner: Owner = Owner {
            pid: number,
            started: moment,
            boot: Some((*boot).to_string()),
            namespace: Some(space),
        };
        return Some(owner.liveness());
    }
    // What: the shape this build writes when it cannot read its own boot or namespace,
    // which ends in the purpose, the process id and its start time.
    // Why: a name with no boot and no namespace cannot be judged against either, which the answer says.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [temporary, pid, started] = parts.slice(-3);
    // if (temporary === 'tmp' && isPid(pid) && isMoment(started)) return namedWithoutAPlace(pid);
    // ```
    if let [carried @ .., temporary, pid, started] = parts.as_slice()
        && !carried.is_empty()
        && *temporary == TEMPORARY_PURPOSE
        && let Ok(number) = pid.parse::<u32>()
        && let Ok(_moment) = started.parse::<u64>()
    {
        return Some(named_without_a_place(number));
    }
    // What: the shape `0.2.7-alpha.18` and earlier wrote,
    // which ends in the process id and the purpose.
    // Why: a farm written before this build still holds those names, and a kill still leaves one,
    //  so the rebuild has to read them rather than take them for ordinary orphans.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [pid, temporary] = parts.slice(-2);
    // if (temporary === 'tmp' && isPid(pid)) return namedWithoutAPlace(pid);
    // ```
    if let [carried @ .., pid, temporary] = parts.as_slice()
        && !carried.is_empty()
        && *temporary == TEMPORARY_PURPOSE
        && let Ok(number) = pid.parse::<u32>()
    {
        return Some(named_without_a_place(number));
    }
    return None;
}

/// What:
/// What this process can say about a temporary link whose name records only a process number.
/// Why:
/// The number is an answer only while it belongs to no process:
/// then the install that wrote the name has ended beyond doubt;
/// while some process holds the number it may be that install, or one that took the number over after it ended,
/// and nothing in the name tells those apart,
/// so the link stays and a later rebuild asks again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namedWithoutAPlace(pid: number): 'gone' | 'unknown'
/// ```
fn named_without_a_place(pid: u32) -> Liveness {
    if super::super::archive_staging::the_number_is_held(pid) {
        return Liveness::Unknown;
    }
    return Liveness::Gone;
}
