//! What:
//!  The handler's journal lines for one forced act:
//!  what each blocker's end was,
//!  what each propagation change was,
//!  what became of each destination directory,
//!  which unit was stopped,
//!  which holder was not ended and why,
//!  which namespace was refused,
//!  and the closing count.
//! Why:
//!  The records record decided that the handler's journal lines are the record
//!  of who a forced removal ended,
//!  so these lines are the only place that record exists
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The act's mechanism",
//!  and "The record of who a forced removal ended,
//!  and the questions left").
//!  One line carries one event with every field the design lists,
//!  and each value is quoted so a path or a command word with a space cannot
//!  split a line,
//!  because a reader that split one would lose the event.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! journal.blockerEnded({ dataset, nonce, namespace, mountId, holder, foundBy, signals, result });
//! journal.closed({ ended, unproven, scanClear, changed });
//! ```

use crate::worktree_clone_forced_removal::blocker_end::EndResult;
use crate::worktree_clone_forced_removal::forced_removal_words::Stage;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// The prefix every line of one forced act carries,
/// so one journal can be read for the act alone.
pub const LINE_PREFIX: &str = "forced-removal";

/// What:
///  One text as one quoted value.
/// Why:
///  A path,
///  a command word or a unit's name may hold a space,
///  and a value that split would read as two fields of one event.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = `'${text.replaceAll("'", "'\\''")}'`;
/// ```
pub fn quoted(text: &str) -> String {
    let mut value: String = String::with_capacity(text.len() + 2);
    value.push('\'');
    for character in text.chars() {
        if character == '\'' {
            value.push_str("'\\''");
            continue;
        }
        value.push(character);
    }
    value.push('\'');
    return value;
}

/// What:
///  The two fields every line of one act repeats:
///  the dataset and the request's nonce.
/// Why:
///  A journal holds every act of every clone,
///  and a line without its request could not be tied to what asked for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheActsJournal = { dataset; nonce };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct TheActsJournal<'a> {
    /// The clone's dataset name.
    pub dataset: &'a str,
    /// The request's nonce.
    pub nonce: &'a RequestNonce,
}

impl TheActsJournal<'_> {
    /// The two fields every line repeats.
    fn the_request(&self) -> String {
        return format!(
            "dataset={} nonce={}",
            quoted(self.dataset),
            quoted(self.nonce.as_str())
        );
    }
}

/// What:
///  One blocker's end,
///  with every field the design lists.
/// Why:
///  The record has to say which process was ended,
///  in which namespace and through which read it was found,
///  what was sent and what became of it,
///  or the line cannot answer "who did this removal end".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = journal.blockerEnded({ namespace, mountId, pid, started, session, foundBy, signals, result });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ABlockersEnd<'a> {
    /// The namespace the blocker was found in.
    pub namespace: &'a str,
    /// The mount the copy sat at,
    /// or nothing when the blocker held the mount without a copy.
    pub mount_id: Option<u64>,
    /// The task's number.
    pub pid: i32,
    /// The start time the kernel recorded for it.
    pub started: u64,
    /// The session the task belonged to.
    pub session: i32,
    /// The read that found it,
    /// such as `cwd` or `map_files`.
    pub found_by: &'a str,
    /// The signals sent,
    /// in the order they were sent.
    pub signals: &'a [&'a str],
    /// What became of it.
    pub result: EndResult,
}

impl TheActsJournal<'_> {
    /// One blocker and its end.
    pub fn a_blocker_ended(
        &self,
        ended: &ABlockersEnd<'_>,
    ) -> String {
        let mount: String = match ended.mount_id {
            Some(id) => id.to_string(),
            None => String::from("none"),
        };
        let mut sent: String = String::with_capacity(8);
        for signal in ended.signals {
            if !sent.is_empty() {
                sent.push(',');
            }
            sent.push_str(signal);
        }
        return format!(
            "{LINE_PREFIX} blocker-ended {} namespace={} mount={mount} pid={} started={} session={} found-by={} signals={} result={}",
            self.the_request(),
            quoted(ended.namespace),
            ended.pid,
            ended.started,
            ended.session,
            quoted(ended.found_by),
            quoted(sent.as_str()),
            ended.result.word()
        );
    }
}

/// What:
///  One propagation change,
///  with the tags before and after.
/// Why:
///  Way 7 changes one mount's sharing,
///  and a person has to be able to read what it was and what it became,
///  with the two tag words the kernel prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = journal.aPropagationChange({ namespace, mountId, was, now, change, finished });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct APropagationChange<'a> {
    /// The namespace whose mount changed.
    pub namespace: &'a str,
    /// The mount that changed.
    pub mount_id: u64,
    /// The tags before the change,
    /// such as `shared:12`.
    pub was: &'a str,
    /// The tags after it,
    /// such as `master:12`.
    pub now: &'a str,
    /// Which change was made,
    /// `slave` or `private`.
    pub change: &'a str,
    /// Whether the removal finished after it.
    pub finished: bool,
}

impl TheActsJournal<'_> {
    /// One mount's propagation change.
    pub fn a_propagation_change(
        &self,
        changed: &APropagationChange<'_>,
    ) -> String {
        return format!(
            "{LINE_PREFIX} propagation-changed {} namespace={} mount={} was={} now={} change={} removal={}",
            self.the_request(),
            quoted(changed.namespace),
            changed.mount_id,
            quoted(changed.was),
            quoted(changed.now),
            quoted(changed.change),
            if changed.finished {
                "finished"
            } else {
                "failed"
            }
        );
    }
}

/// What:
///  One destination directory and what became of it.
/// Why:
///  A kill can leave the directory in the checkout,
///  and the next run's sweep and `doctor` read this line's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = journal.aDestinationDirectory({ namespace, name, removed });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ADestinationDirectory<'a> {
    /// The namespace the directory was made in.
    pub namespace: &'a str,
    /// The directory's name.
    pub name: &'a str,
    /// Whether the act removed it.
    pub removed: bool,
}

impl TheActsJournal<'_> {
    /// One destination directory.
    pub fn a_destination_directory(
        &self,
        directory: &ADestinationDirectory<'_>,
    ) -> String {
        return format!(
            "{LINE_PREFIX} destination-directory {} namespace={} name={} left={}",
            self.the_request(),
            quoted(directory.namespace),
            quoted(directory.name),
            if directory.removed { "no" } else { "yes" }
        );
    }
}

/// What:
///  One unit the act asked its manager to stop.
/// Why:
///  The person has to know which unit was stopped so they can start it again,
///  and with which namespace the copy went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = journal.aStoppedUnit({ unit, cgroup, namespace });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AStoppedUnit<'a> {
    /// The unit's name.
    pub unit: &'a str,
    /// The unit's cgroup.
    pub cgroup: &'a str,
    /// The mount namespace that went with it.
    pub namespace: &'a str,
}

impl TheActsJournal<'_> {
    /// One stopped unit,
    /// with the sentence that the removal may be run again.
    pub fn a_stopped_unit(
        &self,
        stopped: &AStoppedUnit<'_>,
    ) -> String {
        return format!(
            "{LINE_PREFIX} unit-stopped {} unit={} cgroup={} namespace={} again=run-the-removal-and-start-the-unit",
            self.the_request(),
            quoted(stopped.unit),
            quoted(stopped.cgroup),
            quoted(stopped.namespace)
        );
    }
}

/// What:
///  One holder the act did not end,
///  with the proof that failed.
/// Why:
///  A refusal that named only the word would hide which holder it was about,
///  and the next run meets the same one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = journal.aHolderNotEnded({ namespace, mountId, holder, failedProof });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AHolderNotEnded<'a> {
    /// The namespace the holder was found in.
    pub namespace: &'a str,
    /// The mount the copy sat at,
    /// where one did.
    pub mount_id: Option<u64>,
    /// The process or unit the report names.
    pub holder: &'a str,
    /// The read that refused.
    pub failed_proof: &'a str,
}

impl TheActsJournal<'_> {
    /// One holder not ended.
    pub fn a_holder_not_ended(
        &self,
        kept: &AHolderNotEnded<'_>,
    ) -> String {
        let mount: String = match kept.mount_id {
            Some(id) => id.to_string(),
            None => String::from("none"),
        };
        return format!(
            "{LINE_PREFIX} holder-not-ended {} namespace={} mount={mount} holder={} failed-proof={}",
            self.the_request(),
            quoted(kept.namespace),
            quoted(kept.holder),
            quoted(kept.failed_proof)
        );
    }
}

/// What:
///  One refused namespace and why.
/// Why:
///  The preflight's refusals are read one by one by a person,
///  and each names a namespace and the read that refused it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = journal.aRefusedNamespace({ namespace, word, said });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ARefusedNamespace<'a> {
    /// The namespace refused.
    pub namespace: &'a str,
    /// The refusal's own word.
    pub word: &'a str,
    /// What the read answered.
    pub said: &'a str,
}

impl TheActsJournal<'_> {
    /// One refused namespace.
    pub fn a_refused_namespace(
        &self,
        refused: &ARefusedNamespace<'_>,
    ) -> String {
        return format!(
            "{LINE_PREFIX} namespace-refused {} namespace={} word={} said={}",
            self.the_request(),
            quoted(refused.namespace),
            quoted(refused.word),
            quoted(refused.said)
        );
    }
}

/// What:
///  A refusal in the middle of the structural stage:
///  what was changed,
///  what the directory's state is and that no copy moved.
/// Why:
///  That state is the design's third refusal state,
///  and a person must be able to read what the act left behind,
///  because nothing re-shares the parent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = journal.aStructuralRefusal({ namespace, changed, directory, said });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AStructuralRefusal<'a> {
    /// The namespace the refused change was about.
    pub namespace: &'a str,
    /// Which namespaces' mounts were already changed,
    /// in the order they were changed.
    pub changed: &'a [&'a str],
    /// The directory's state,
    /// such as `removed` or `left`.
    pub directory: &'a str,
    /// What the kernel refused.
    pub said: &'a str,
}

impl TheActsJournal<'_> {
    /// One refusal in the middle of the structural stage.
    pub fn a_structural_refusal(
        &self,
        refused: &AStructuralRefusal<'_>,
    ) -> String {
        let mut changed: String = String::with_capacity(16);
        for namespace in refused.changed {
            if !changed.is_empty() {
                changed.push(',');
            }
            changed.push_str(namespace);
        }
        return format!(
            "{LINE_PREFIX} structural-refusal {} namespace={} changed={} directory={} copy-moved=no said={}",
            self.the_request(),
            quoted(refused.namespace),
            quoted(changed.as_str()),
            quoted(refused.directory),
            quoted(refused.said)
        );
    }
}

/// What:
///  The one closing line of an act.
/// Why:
///  A reader who reads one line of an act reads this one:
///  how many blockers were ended,
///  how many were unproven,
///  whether the following scan was clear,
///  and how many namespaces' propagation changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = journal.closed({ ended, unproven, scanClear, changed, stage });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AClosingLine {
    /// How many blockers were ended.
    pub ended: usize,
    /// How many could not be proven,
    /// which every reader counts as still there.
    pub unproven: usize,
    /// Whether the scan after the act was clear.
    pub scan_clear: bool,
    /// How many namespaces' propagation changed.
    pub propagation_changes: usize,
    /// The stage the act ended in.
    pub stage: Stage,
}

impl TheActsJournal<'_> {
    /// The act's closing line.
    pub fn closed(
        &self,
        line: &AClosingLine,
    ) -> String {
        return format!(
            "{LINE_PREFIX} act-closed {} ended={} unproven={} scan={} changed={} stage={}",
            self.the_request(),
            line.ended,
            line.unproven,
            if line.scan_clear {
                "clear"
            } else {
                "not-clear"
            },
            line.propagation_changes,
            quoted(line.stage.word())
        );
    }
}

/// What:
///  One act's lines as one text,
///  each on its own line.
/// Why:
///  A journal is written a line at a time,
///  and a caller that built its own newlines could join two events into one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = lines.map((line) => `${line}\n`).join('');
/// ```
pub fn as_a_block(lines: &[String]) -> String {
    let mut text: String = String::new();
    for line in lines {
        text.push_str(line.as_str());
        text.push('\n');
    }
    return text;
}

/// What:
///  The dataset's canonical name for a journal,
///  which is the GUID's own file name.
/// Why:
///  A line must name the clone the same way the receipt's file does,
///  or a reader matching the two cannot.
pub fn dataset_word(dataset: DatasetGuid) -> String {
    return dataset.file_name();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_acts_journal_lines_tests.rs"]
mod tests;
