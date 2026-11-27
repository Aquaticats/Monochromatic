//! What:
//!  Every step an add enters,
//!  and the words a person reads for it.
//! Why:
//!  The step names are the timing record's,
//!  written for someone reading that record,
//!  so each is said here as the work it is.
//!  The sentences live in one table beside the names so a step cannot be entered without words,
//!  and the control `every_step_an_add_enters_has_its_own_sentence` fails when one is named there and
//!  not said here.
//!  This file is part of the module `add_standing_notice.rs`,
//!  split out because that module was over the line limit:
//!  the table of a step's words is the largest thing in it,
//!  and the notice's state, its thread and its line are the rest.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sentenceOf = (step) => STEP_SENTENCES.get(step) ?? 'the add is working in a step its timing record names';
//! ```

/// What:
///  Every step an add enters,
/// and the words a person reads for it.
/// Why:
///   The step names are the timing record's,
/// written for someone reading that record,
/// so each is said here as the work it is.
/// The sentences live in one table beside the names so a step cannot be entered without
///  words,
/// and `every_step_an_add_enters_has_its_own_sentence` fails when one is added and not said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STEP_SENTENCES: [step: string, sentence: string][] = [['recover', 'it is settling ...'], ...]
/// ```
pub(crate) const STEP_SENTENCES: &[(&str, &str)] = &[
    (
        "recover",
        "it is settling what interrupted commands left in this repository",
    ),
    ("chooseRung", "it is deciding how to make the worktree"),
    (
        "startBesideTheCheck",
        "it is starting the size read the snapshot rung decides with",
    ),
    (
        "abandonRead",
        "it is ending the size read this rung will not use",
    ),
    (
        "abandonTakeTheAnswer",
        "it is ending the size read this rung will not use",
    ),
    (
        "abandonEndTheChild",
        "it is ending the size read this rung will not use",
    ),
    ("refusals", "it is deciding how to make the worktree"),
    ("checkSource", "it is deciding how to make the worktree"),
    ("claimStage", "it is deciding how to make the worktree"),
    ("trackedEntries", "it is deciding how to make the worktree"),
    (
        "walkSource",
        "it is reading the checkout the worktree is made from",
    ),
    (
        "readIndexAndTree",
        "it is comparing the source's index with the files it recorded",
    ),
    (
        "indexAgainstTree",
        "it is comparing the source's index with the files it recorded",
    ),
    (
        "gitAdd",
        "it is in the `git worktree add` this command started",
    ),
    (
        "bindRegistration",
        "it is recording which directories Git made for the worktree",
    ),
    (
        "recordBinding",
        "it is recording which directories Git made for the worktree",
    ),
    (
        "recordDestination",
        "it is recording where the new worktree goes",
    ),
    (
        "reserveDestination",
        "it is reserving the directory the new worktree is made in",
    ),
    (
        "prepareTarget",
        "it is asking for a snapshot of the main checkout",
    ),
    (
        "snapshotAheadProof",
        "it is asking for a snapshot of the main checkout",
    ),
    (
        "snapshotAheadWait",
        "it is waiting for the commit service to finish a snapshot it already started",
    ),
    (
        "takeSnapshot",
        "it is taking the snapshot of the main checkout",
    ),
    (
        "snapshotIoctl",
        "it is taking the snapshot of the main checkout",
    ),
    (
        "walkSnapshot",
        "it is taking the snapshot of the main checkout",
    ),
    (
        "readOwnIndex",
        "it is taking the snapshot of the main checkout",
    ),
    ("prune", "it is taking the snapshot of the main checkout"),
    (
        "removeStubs",
        "it is taking the snapshot of the main checkout",
    ),
    (
        "replacePointer",
        "it is taking the snapshot of the main checkout",
    ),
    (
        "installIndexFile",
        "it is taking the snapshot of the main checkout",
    ),
    ("installIndex", "it is installing the snapshot's index"),
    (
        "applyDifference",
        "it is bringing the snapshot to the commit the worktree is at",
    ),
    (
        "restoreDirty",
        "it is bringing the snapshot to the commit the worktree is at",
    ),
    (
        "exchange",
        "it is swapping the finished snapshot into place",
    ),
    (
        "settleDisplaced",
        "it is clearing the directory Git made once the snapshot is in place",
    ),
    (
        "finishRung",
        "it is finishing the step that made the new worktree",
    ),
    (
        "markCompleteFile",
        "it is recording that the worktree is complete",
    ),
    (
        "markCompleteDirectory",
        "it is recording that the worktree is complete",
    ),
    (
        "releaseClaim",
        "it is recording that the worktree is complete",
    ),
    (
        "partRead",
        "it is reading the source's index and the commit the worktree is at",
    ),
    (
        "partClassify",
        "it is proving which tracked files the source's changes did not touch",
    ),
    (
        "partStaged",
        "it is checking that no path the source staged is among the files it clones",
    ),
    (
        "cloneEntries",
        "it is cloning the tracked files of the source into the new worktree",
    ),
    (
        "cloneIndex",
        "it is writing the index the clones' worktree is read through",
    ),
    (
        "partIndex",
        "it is writing the index the new worktree is read through",
    ),
    (
        "conversionFence",
        "it is reading which files Git's checkout would write other bytes for",
    ),
    (
        "fenceSizesWaitingForTheRead",
        "it is waiting for the file sizes the fences read",
    ),
    (
        "targetTreeDifference",
        "it is checking whether the commit the worktree is added at changes a file Git must write",
    ),
    (
        "conversionBesideTheClone",
        "it is letting the fences read the source while the tracked files are copied",
    ),
    (
        "writeConversions",
        "it is letting Git write the files a conversion changes",
    ),
    (
        "copyIgnored",
        "it is copying the ignored files into the new worktree",
    ),
    (
        "handOverTrash",
        "it is handing the files the snapshot set aside to the commit service",
    ),
];

/// What:
///  The step of the add a person reads,
/// in the words of what it does.
/// Why:
///   The step names are the timing record's,
/// so each is said from the one table that holds the names and their words;
/// a step of a later release that this table does not name is said as the add's own work
///  rather than left unnamed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sentenceOf = (step) => step === undefined ? 'it is between two of its steps' : (STEP_SENTENCES.get(step) ?? 'the add is working in a step its timing record names');
/// ```
pub(crate) fn step_sentence(step: Option<&str>) -> String {
    let Some(name) = step else {
        return String::from("it is between two of its steps");
    };
    // A Git run is named by its own subcommand (`git read-tree`), so the name is the words: a name that
    // starts with `git ` says which run it is without a sentence of its own in the table.
    if let Some(run) = name.strip_prefix("git ")
        && !run.is_empty()
    {
        return format!("it is in Git's own `{name}`");
    }
    for (stepped, sentence) in STEP_SENTENCES {
        if *stepped == name {
            return (*sentence).to_string();
        }
    }
    return String::from("the add is working in a step its timing record names");
}
