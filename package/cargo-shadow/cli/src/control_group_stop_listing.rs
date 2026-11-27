//! What:
//! The listing a stop makes when its bound has passed:
//! every member of the scope's own group and of every group below it,
//! with the group it stands in and the state of its threads,
//! inside the listing's own bound.
//! Why:
//! The kernel's `populated` word counts the groups below the scope,
//! so a scope whose own `cgroup.procs` is empty can still hold a live process
//! (`doc/troubleshooting/a-cgroup-does-not-count-a-member-that-has-exited.md`),
//! and a refusal that listed only the scope's own group would name nothing while the kernel said something was left.
//! Split out of `control_group_stop.rs`,
//! which keeps the stop itself,
//! to keep that file under the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control-group-stop-listing.ts
//! ControlGroup.prototype.listings = (): Listing
//! ```

/// What:
/// `use super::LISTING_BOUND;` imports how long reading the members may take.
/// Why:
/// The listing runs after the stop's own bound,
/// so it is the one wait a refusal adds and it must be bounded itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LISTING_BOUND } from './control_group_stop.ts';
/// ```
use super::LISTING_BOUND;
/// What:
/// `use super::report::{Listing, MemberListing, standing_of_within};` imports what a listing answers,
/// the member it names and the read of one member's threads.
/// Why:
/// The listing is built from those three,
/// and the words a refusal carries come from the listing's own type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Listing, MemberListing, standingOfWithin } from './control_group_stop_report.ts';
/// ```
use super::report::{
    Listing,
    MemberListing,
    standing_of_within,
};
/// What:
/// `use crate::control_group::ControlGroup;` imports the scope whose members are listed.
/// Why:
/// The listing is a method of that one type,
/// and it reads the scope's own directory and its `cgroup.procs` files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ControlGroup } from 'crate/control_group';
/// ```
use crate::control_group::ControlGroup;
/// What:
/// `use rustix::io::Errno;` imports the error code of a removed group.
/// Why:
/// A read of a removed group's file says `ENODEV`,
/// which is one of the two answers that mean the group is gone rather than unreadable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Errno } from 'rustix/io';
/// ```
use rustix::io::Errno;
/// What:
/// `use std::fs;` imports the file system calls.
/// Why:
/// Each group's `cgroup.procs` and each group's directories are read from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readdirSync, readFileSync, statSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::io;` imports the input/output error type (its sibling `String` is the reason this module carries).
/// Why:
/// A removed group is told from a group that cannot be read by the error's kind and code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // error.code === 'ENOENT' or 'ENODEV'
/// ```
use std::io;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed view of a path and the owned one (`PathBuf` owns where `&Path` borrows).
/// Why:
/// A group's directory is lent to the reads,
/// and the walk keeps the owned paths of the groups it has still to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // paths are plain strings in TS
/// ```
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `use std::time::Instant;` imports a point on the clock (its sibling `Duration` is a span of time).
/// Why:
/// The listing stops at a deadline of its own,
/// so a scope with many threads cannot hold a refusal for as long as the reads take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now()
/// ```
use std::time::Instant;

/// The methods that read the members a stop left.
impl ControlGroup {
    /// What:
    /// `fn listings(&self) -> Listing` is every member the scope lists,
    /// in its own group and in every group below it,
    /// with the state of its threads:
    /// or the reason the list could not be read,
    /// or the fact that the scope was gone,
    /// or the members it read before its own bound passed.
    /// Why:
    /// A refusal that could not list the members must say so instead of naming none,
    /// because "no member is listed" and "the listing failed" mean different things,
    /// and `populated` counts the groups below the scope,
    /// so a scope whose own `cgroup.procs` is empty can still hold a live member
    /// (`doc/troubleshooting/a-cgroup-does-not-count-a-member-that-has-exited.md`).
    /// Each member is read through a pidfd,
    /// so a number the kernel listed cannot name another process by the time its threads are read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// listings(): Listing
    /// ```
    pub(crate) fn listings(&self) -> Listing {
        // A test makes the listing find the scope gone: the window between the last `populated` read and the listing is too short
        // to produce from outside, because a removal before the read makes the stop report the scope empty at once.
        #[cfg(test)]
        if crate::failpoints::fires(crate::failpoints::Point::ContainmentScopeGoneAtListing) {
            return Listing::ScopeGone;
        }
        let deadline: Instant = Instant::now() + LISTING_BOUND;
        let own: Vec<i32> = match self.members() {
            Ok(found) => found,
            Err(error) => return Listing::Unreadable(error.to_string()),
        };
        // `members()` reads a removed group as empty; the directory itself says whether it was removed.
        if own.is_empty() && fs::metadata(&self.directory).is_err() {
            return Listing::ScopeGone;
        }
        let mut members: Vec<MemberListing> = Vec::new();
        // The scope's own group first, then every group below it, depth first.
        let mut pending: Vec<(PathBuf, Option<String>)> = vec![(self.directory.clone(), None)];
        while let Some((group, below)) = pending.pop() {
            if Instant::now() >= deadline {
                return Listing::Cut(members);
            }
            let listed: Vec<i32> = if below.is_none() {
                own.clone()
            } else {
                match read_procs(&group) {
                    Ok(Some(found)) => found,
                    // A group below the scope that is gone holds nothing.
                    Ok(None) => continue,
                    Err(reason) => return Listing::Unreadable(reason),
                }
            };
            // Sorted, so two readings of the same members name them in the same order whatever the file system answers.
            let mut ordered: Vec<i32> = listed;
            ordered.sort_unstable();
            for pid in ordered {
                if Instant::now() >= deadline {
                    return Listing::Cut(members);
                }
                members.push(MemberListing {
                    pid,
                    below: below.clone(),
                    standing: standing_of_within(pid, deadline),
                });
            }
            match sub_groups(&group) {
                Ok(children) => {
                    for (path, name) in children {
                        let path_below: String = match &below {
                            Some(parent) => format!("{parent}/{name}"),
                            None => name,
                        };
                        pending.push((path, Some(path_below)));
                    }
                }
                Err(reason) => return Listing::Unreadable(reason),
            }
        }
        return Listing::Read(members);
    }
}

/// What:
/// `fn read_procs(group: &Path) -> Result<Option<Vec<i32>>, String>` lists one group's `cgroup.procs`,
/// or `None` when the group is gone,
/// or the reason it could not be read.
/// Why:
/// A member of a group below the scope is left out of the scope's own list,
/// and a group that was removed while the listing ran holds nothing rather than failing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readProcs(group: string): number[] | undefined  // throws on other errors
/// ```
fn read_procs(group: &Path) -> Result<Option<Vec<i32>>, String> {
    let path: PathBuf = group.join("cgroup.procs");
    let text: String = match fs::read_to_string(&path) {
        Ok(found) => found,
        // `ENODEV` is what a read of a removed group's file says, and `NotFound` is what an absent directory says.
        Err(error)
            if error.kind() == io::ErrorKind::NotFound
                || error.raw_os_error() == Some(Errno::NODEV.raw_os_error()) =>
        {
            return Ok(None);
        }
        Err(error) => return Err(format!("{} cannot be read ({error})", path.display())),
    };
    let mut found: Vec<i32> = Vec::new();
    for line in text.lines() {
        match line.trim().parse::<i32>() {
            Ok(number) => found.push(number),
            Err(error) => return Err(format!("{} holds {line:?} ({error})", path.display())),
        }
    }
    return Ok(Some(found));
}

/// What:
/// `fn sub_groups(group: &Path) -> Result<Vec<(PathBuf, String)>, String>` lists the groups directly below one group,
/// each with its name,
/// in name order.
/// Why:
/// The kernel's `populated` word counts a descendant group's threads,
/// so the listing has to walk them;
/// sorting the names makes the members named in the same order from one reading to the next
/// (`doc/directory-listing-order.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function subGroups(group: string): Array<[string, string]>
/// ```
fn sub_groups(group: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let entries = match fs::read_dir(group) {
        Ok(found) => found,
        // A group that was removed while the listing ran has no groups below it any more.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("{} cannot be listed ({error})", group.display())),
    };
    let mut groups: Vec<(PathBuf, String)> = Vec::new();
    for listed in entries {
        let entry: fs::DirEntry = match listed {
            Ok(found) => found,
            Err(error) => return Err(format!("{} cannot be listed ({error})", group.display())),
        };
        let is_dir: bool = match entry.file_type() {
            Ok(kind) => kind.is_dir(),
            Err(error) => {
                return Err(format!(
                    "{} cannot be read ({error})",
                    entry.path().display()
                ));
            }
        };
        if !is_dir {
            continue;
        }
        groups.push((
            entry.path(),
            entry.file_name().to_string_lossy().into_owned(),
        ));
    }
    groups.sort();
    return Ok(groups);
}
