//! What:
//!  The removal of the launcher records,
//!  and the launchers beside them,
//!  that this worktree's lock declares nowhere:
//!  taking each record aside under a name only this pass writes,
//!  reading it again before it goes,
//!  putting back what a concurrent command wrote,
//!  and putting back what a killed pass left aside.
//! Why:
//!  `package_repair_toolchain.rs` would otherwise pass the file-size limit,
//!  and these records are copies of a layout an earlier release had,
//!  which no install rewrites and nothing runs,
//!  so removing them is a proof of bytes rather than a step of the install,
//!  and it lives in a file of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const copy of copies) { const aside = takeAside(copy); if (!aside) continue; if (!stillACopy(aside, copy.name)) { putBack(aside, copy.record); continue; } unlink(aside); }
//! ```

/// Import the classification's record of a copy and the names the lock declares.
use super::launcher_records::{
    CopyOfAnEarlierLayout,
    declared_launcher_names,
};
/// Import the log tag.
use super::names_and_bounds::TAG;
/// Import the two slots a control stands a concurrent writer and a kill in,
/// which only a control build has.
#[cfg(test)]
use super::names_and_bounds::{
    KILLED_AFTER_THE_RECORD_IS_ASIDE,
    WHEN_THE_REMOVAL_BEGINS,
};
/// Import the launcher directory and the lock's name.
use super::names_and_bounds::{
    LAUNCHERS,
    LOCK_FILE,
};
/// Import the object comparisons the removal proves its bytes with.
use super::object_identity::{
    ThisWorktreesLock,
    identity_of,
    lock_named_by_record,
};
/// Import the one diagnostic log this crate writes.
use crate::diagnostic_log::debug;
/// Import the set of launcher names a lock declares.
use std::collections::HashSet;
/// Import native paths.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  Remove the launcher records, and the launchers beside them,
///  that this worktree's lock declares nowhere,
///  once the bytes being removed are proved to be that copy and not what another command wrote.
/// Why:
///   They are copies of a layout an earlier release had,
///  carried in by the copy that made the worktree,
///  and they name another checkout's lock file,
///  which is the state this repair exists to end.
///  No install rewrites them and nothing runs them,
///  so leaving them would keep a record that names another checkout in every worktree copied from this one.
///  A record that cannot be removed is said and left:
///  it is no install's work, so the relink neither fails nor waits on it.
///  The classification reads the lock and the directory once,
///  and a person's `toolchain:update` and install can write a correct record and launcher for a name the lock
///  declares after that read,
///  so each record is taken aside only when it is the object this pass judged,
///  read again under that name,
///  and put back when the install declared its name meanwhile or its bytes name this worktree's own lock;
///  the launcher goes only when it is the object this pass read beside the record.
///  Nothing taken aside is ever discarded unread:
///  a pass killed with a record aside leaves it for the next pass, which reads it and puts it back,
///  or keeps it and names it for the person when the record's own name holds another record by then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// restoreWhatAKillLeft(destination); if (!copies.length) return;
/// for (const copy of copies) { const aside = takeAside(copy); if (!aside) continue; if (!stillACopy(destination, aside, copy.name)) { putBack(aside, copy.record); continue; } unlink(aside); if (identityOf(copy.launcher) === copy.launcherIdentity) unlink(copy.launcher); }
/// ```
pub(super) fn remove_what_no_install_rewrites(
    destination: &Path,
    copies: &[CopyOfAnEarlierLayout],
) {
    // A control's own concurrent `toolchain:update` and install, run once as the removal begins: the window a
    //  classification cannot see lies between its read and the removal, and a control has to stand in it to show what
    //  the pass does when something is written there. Taken out, so it runs once; only a control build has it.
    #[cfg(test)]
    if let Some(write) = WHEN_THE_REMOVAL_BEGINS.with(|cell| return cell.borrow_mut().take()) {
        write();
    }
    // What a killed pass left aside is read and put back before this pass decides anything, whether or not it has a copy
    //  of its own to remove: a record an install wrote is not discarded with the pass that took it.
    restore_what_a_kill_left_aside(destination);
    if copies.is_empty() {
        return;
    }
    let own: ThisWorktreesLock = ThisWorktreesLock::of(destination);
    // The lock as it stands after the classification, read once for the whole removal: a name it declares now is one an
    //  install has just taken on, and a record the pass holds for it may not go. Reading it once per record would read
    //  the whole lock a thousand times in a worktree carrying a thousand copies.
    let declared_now: Option<HashSet<String>> = declared_launcher_names(destination);
    for copy in copies {
        let Some(aside) = take_aside(copy) else {
            continue;
        };
        // A control's own kill, at the one instruction a record is out of its place: nothing after this line runs, which
        //  is what a killed pass leaves. Only a control build has it.
        #[cfg(test)]
        if KILLED_AFTER_THE_RECORD_IS_ASIDE.with(std::cell::Cell::get) {
            return;
        }
        if !still_a_copy_of_an_earlier_layout(
            aside.as_path(),
            copy.name.as_deref(),
            &own,
            declared_now.as_ref(),
        ) {
            // Another command declared this name or wrote its own record there: the bytes are not a copy any more,
            //  and a name that holds a newer file keeps it.
            put_back(aside.as_path(), copy.record.as_path());
            continue;
        }
        match std::fs::remove_file(aside.as_path()) {
            Ok(()) => debug(
                TAG,
                format!(
                    "{}: removed {}, whose launcher name {} declares nowhere",
                    destination.display(),
                    copy.record.display(),
                    LOCK_FILE
                )
                .as_str(),
            ),
            Err(error) => {
                debug(TAG, format!("{}: {} is no record any install rewrites and it could not be removed ({error})", destination.display(), aside.display()).as_str());
                continue;
            }
        }
        // The launcher beside the record goes with it, and only the object this pass read: an install that wrote its own
        //  launcher meanwhile leaves a different object, which is not this pass's to remove.
        if copy.launcher_identity.is_none() {
            continue;
        }
        if identity_of(copy.launcher.as_path()) != copy.launcher_identity {
            debug(
                TAG,
                format!(
                    "{}: {} was written by another command and is left",
                    destination.display(),
                    copy.launcher.display()
                )
                .as_str(),
            );
            continue;
        }
        match std::fs::remove_file(copy.launcher.as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => debug(
                TAG,
                format!(
                    "{}: {} could not be removed ({error})",
                    destination.display(),
                    copy.launcher.display()
                )
                .as_str(),
            ),
        }
    }
}

/// What:
///  Take one record out of the way of anything else writing it,
///  under a name only this pass writes,
///  or nothing when the record at that name is not the one this pass judged.
/// Why:
///  The name is ours alone (this process's number and a count),
///  so no other command can write what is read there,
///  which is what makes the deletion provable:
///  whatever is at the record's own name afterwards was written by that other command and is left.
///  A record another command wrote at that name since the classification is another object,
///  and taking it would put an install's record out of its place for a reason that was not about it;
///  the object is looked at before the take and again after it, because a name can be written between the two.
///  A record put back after that second look is restored by a name taken only when it is free,
///  and one that cannot be restored stays kept for the next pass, which reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const takeAside = (copy) => { if (identityOf(copy.record) !== copy.recordIdentity) return undefined; const aside = renameAside(copy.record); if (identityOf(aside) !== copy.recordIdentity) { restore(aside, copy.record); return undefined; } return aside; };
/// ```
fn take_aside(copy: &CopyOfAnEarlierLayout) -> Option<PathBuf> {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let record: &Path = copy.record.as_path();
    let Some(judged) = copy.record_identity else {
        debug(
            TAG,
            format!(
                "{} was not the same object even when this pass judged it, so it is left",
                record.display()
            )
            .as_str(),
        );
        return None;
    };
    if identity_of(record) != Some(judged) {
        debug(
            TAG,
            format!(
                "{} is another record than the one this pass judged, so it is left",
                record.display()
            )
            .as_str(),
        );
        return None;
    }
    let token: u32 = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let name: String = format!(
        "{}.removing-{}-{token}",
        record.file_name()?.to_string_lossy(),
        std::process::id()
    );
    let aside: PathBuf = record.with_file_name(name);
    match std::fs::rename(record, aside.as_path()) {
        Ok(()) => {}
        Err(error) => {
            debug(
                TAG,
                format!(
                    "{} could not be taken aside ({error}), so it is left",
                    record.display()
                )
                .as_str(),
            );
            return None;
        }
    }
    if identity_of(aside.as_path()) == Some(judged) {
        return Some(aside);
    }
    // A name written between the look and the rename: what is in this pass's hand is that other command's record, and it
    //  goes back where it was. A name that is taken meanwhile keeps it, and the next pass reads it and puts it back.
    match std::fs::hard_link(aside.as_path(), record) {
        Ok(()) => {
            let _removed = std::fs::remove_file(aside.as_path());
        }
        Err(error) => debug(TAG, format!("{} was written as this pass took the name, and could not be put back yet ({error}); the next pass reads it", record.display()).as_str()),
    }
    return None;
}

/// What:
///  Put a record this pass read aside back under its name,
///  unless another command has written one there meanwhile.
/// Why:
///   A rename would replace whatever that command wrote;
///   a name taken only when it is free leaves the newer record and drops the copy this pass holds,
///   whose bytes this pass has read and judged a copy of another checkout's layout.
///   A record that cannot be put back stays where it is, and the next pass reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const putBack = (aside, record) => { if (link(aside, record) fails with EEXIST) unlink(aside); else if (it succeeds) unlink(aside); };
/// ```
fn put_back(
    aside: &Path,
    record: &Path,
) {
    match std::fs::hard_link(aside, record) {
        Ok(()) => {
            let _removed = std::fs::remove_file(aside);
            debug(TAG, format!("{} was written by another command and is left", record.display()).as_str());
        }
        // A record another command wrote is newer than the copy in this pass's hand, which is dropped here.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let _removed = std::fs::remove_file(aside);
        }
        Err(error) => debug(TAG, format!("{} could not be put back from {} ({error}); the next pass reads it and puts it back", record.display(), aside.display()).as_str()),
    }
}

/// What:
///  Whether the record this pass took aside still says it is a copy no install rewrites:
///  it names a lock other than this worktree's,
///  and the worktree's lock, as the removal read it, declares no launcher of its name.
/// Why:
///   The record is read again here because it may have changed since the classification,
///   and a record an install has just written for a name the lock declares is not this pass's to remove.
///   A lock that cannot be read keeps the rule the classification used: every record counts as one an install answers,
///  so a copy is still a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stillACopy = (aside, name, own) => own.namesAnotherLock(aside) === true && (name === undefined || !(declaredLauncherNames(destination)?.has(name) ?? false));
/// ```
fn still_a_copy_of_an_earlier_layout(
    aside: &Path,
    name: Option<&str>,
    own: &ThisWorktreesLock,
    declared_now: Option<&HashSet<String>>,
) -> bool {
    match own.names_another_lock(aside) {
        Some(true) => {}
        Some(false) | None => return false,
    }
    let Some(launcher) = name else { return true };
    return match declared_now {
        Some(declared) => !declared.contains(launcher),
        None => true,
    };
}

/// What:
///  Put back every record a killed pass left aside,
///  reading it first,
///  and keep and name the ones whose own name holds another record by now.
/// Why:
///   A pass killed between taking a record aside and removing or putting it back leaves that name behind,
///   and nothing else would read it:
///   it may be a copy of an earlier layout, which a later pass removes under its own proof,
///   or the record a concurrent install had just written, which nothing else would recreate.
///   So it is read and put back by a name taken only when it is free,
///   and one whose name is taken by then is kept rather than replaced or discarded, and named for the person.
///   A name whose process still runs is a pass in flight and is left to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of readdir(launchers)) if (isAnAsideOfADeadPass(entry)) { if (!readableRecord(entry)) continue; if (link(entry, nameOf(entry))) unlink(entry); else nameForThePerson(entry, nameOf(entry)); }
/// ```
fn restore_what_a_kill_left_aside(destination: &Path) {
    let launchers: PathBuf = destination.join(LAUNCHERS);
    let Ok(listing) = std::fs::read_dir(launchers.as_path()) else {
        return;
    };
    for item in listing.flatten() {
        let name: std::ffi::OsString = item.file_name();
        let Some(text) = name.to_str() else { continue };
        let Some((record_name, owner)) = text.rsplit_once(".removing-") else {
            continue;
        };
        let Some((written_pid, _token)) = owner.split_once('-') else {
            continue;
        };
        let Ok(pid) = written_pid.parse::<u32>() else {
            continue;
        };
        if std::path::Path::new(format!("/proc/{pid}").as_str()).exists() {
            continue;
        }
        let aside: PathBuf = item.path();
        // Nothing is put back from a file this release cannot read as a record, and nothing is discarded unseen either;
        //  the read says which file it refused and why.
        if lock_named_by_record(aside.as_path()).is_none() {
            continue;
        }
        let record: PathBuf = launchers.join(record_name);
        match std::fs::hard_link(aside.as_path(), record.as_path()) {
            Ok(()) => {
                let _removed = std::fs::remove_file(aside.as_path());
                debug(
                    TAG,
                    format!(
                        "{}: put back {} from what a killed pass left",
                        destination.display(),
                        record.display()
                    )
                    .as_str(),
                );
            }
            Err(error) => {
                debug(TAG, format!("{}: {} could not be put back from what a killed pass left ({error}), so it is named for the person", destination.display(), record.display()).as_str());
                name_what_a_killed_pass_kept(
                    destination,
                    aside.as_path(),
                    record.as_path(),
                    &error,
                );
            }
        }
    }
}

/// What:
///  Leave one line for the person about a record a killed pass kept aside,
///  naming the file and the command that puts it back.
/// Why:
///   The wrapper could not put it back because the record's own name holds another record by then,
///   and it may not replace that one, nor discard what it holds:
///   the person is the one who can say which record their worktree needs.
///   The line waits in the worktree's own Git directory, where the next command there shows it once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = (worktree, aside, record, why) => recordNotice(gitDirOf(worktree), `toolchain-kept-${basename(aside)}`, `...kept ${aside}... mv ${aside} ${record}`);
/// ```
fn name_what_a_killed_pass_kept(
    destination: &Path,
    aside: &Path,
    record: &Path,
    why: &std::io::Error,
) {
    let words: String = format!(
        "monochromatic-git-policy-cli: a relink of {} was stopped with a launcher record set aside, and the wrapper could not put it back at {} ({why}), so it kept it at {} instead of discarding it. If it is the record this worktree needs, run `mv -- {} {}`; otherwise remove it.",
        destination.display(),
        record.display(),
        aside.display(),
        crate::shell_quote::shell_quote(aside.as_os_str().as_encoded_bytes()),
        crate::shell_quote::shell_quote(record.as_os_str().as_encoded_bytes())
    );
    let Some(git_dir) = crate::package_repair_relink_notice::git_dir_of_a_worktree(destination)
    else {
        crate::service_error!(
            "the record kept at {} could not be named for the person, because {} has no Git directory this release can read: {words}",
            aside.display(),
            destination.display()
        );
        return;
    };
    let id: String = format!(
        "toolchain-kept-{}",
        aside
            .file_name()
            .map_or(String::from("a-record"), |name| return name
                .to_string_lossy()
                .into_owned())
    );
    if let Err(problem) = crate::package_repair_relink_notice::record_notice(
        git_dir.as_path(),
        id.as_str(),
        words.as_str(),
    ) {
        crate::service_error!(
            "the record kept at {} could not be named for the person: {problem}",
            aside.display()
        );
    }
}
