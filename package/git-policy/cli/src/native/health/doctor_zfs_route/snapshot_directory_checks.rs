//! What:
//!  The `snapdir` of the checkout dataset and of the clone container,
//!  each reported where it is not `disabled`,
//!  with the command that sets it,
//!  the fact that every clone inherits the container's value,
//!  and the module floor `disabled` exists from.
//! Why:
//!  The project decided that hiding snapshot directories is what keeps an
//!  account from growing the machine's mount table until the route's look-up
//!  refuses for everyone:
//!  listing a snapshot's directory makes the kernel mount that snapshot as
//!  root and adds a row,
//!  and only `snapdir=disabled` refuses the name,
//!  while the default `hidden` merely leaves the directory out of a listing
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "Snapshot directories are hidden on the checkout",
//! and its
//!  "What hiding comes to,
//! answered 2026-11-23";
//!  plan step 18,
//!  "What it builds").
//!  Plan step 19's setup sets the value on the checkout dataset and again on
//!  the clone container so every clone inherits it,
//!  and this step is the reader:
//!  `doctor` changes nothing,
//!  so the command is printed and never run.
//!  The value arrives in the dataset-stats reply the dataset checks already
//!  read,
//! so no check here opens the device or starts `zfs`:
//!  the reply carries `snapdir` whenever it is set locally or inherited and
//!  omits it only at the default,
//! which is why an absent value is reported as
//!  the default `hidden` rather than as unreadable.
//!  Nothing compares the loaded module's release text with a rule
//!  (`doc/decision/cli-git-zfs-runtime-layout-checker.md`,
//!  "The release string,
//! Q2"),
//!  so the module floor is stated of a module older than OpenZFS 2.3.0 in the
//!  conditional and no finding claims this machine's module is one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkSnapdir(datasetFacts); // pass | finding('snapdir') | unknown
//! ```

use super::check_outcome::{
    Check,
    CheckOutcome,
    CheckReport,
    finding,
    pass,
    unknown,
};
use super::zfs_checks::{
    DatasetFacts,
    DatasetRole,
    Reading,
};
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    PropertyValue,
    value_of,
};

//region The property and its values

/// The property that decides whether a dataset's `.zfs/snapshot` directory is
/// listed and whether a snapshot may be entered by name.
pub const SNAPDIR_PROPERTY: &str = "snapdir";

/// What:
///  The `snapdir` value that leaves the directory out of a listing but still
///  lets a snapshot be entered by name,
///  which is OpenZFS's default.
/// Why:
///  The kernel's reply omits the property exactly when it is the default,
///  so the default has a name here and an absent property is never reported
///  as an unread value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DEFAULT_SNAPDIR = 0; // hidden
/// ```
pub const SNAPDIR_HIDDEN: u64 = 0;

/// The `snapdir` value that lists the directory,
/// from OpenZFS `include/sys/zfs_ioctl.h` at tag `zfs-2.4.4`
/// (`ZFS_SNAPDIR_VISIBLE`),
/// with the names in `module/zcommon/zfs_prop.c` in the same order.
pub const SNAPDIR_VISIBLE: u64 = 1;

/// The `snapdir` value that refuses a snapshot's name,
/// which is the only value that stops the kernel mounting a snapshot when an
/// account enters its directory.
pub const SNAPDIR_DISABLED: u64 = 2;

/// What:
///  The `snapdir` word a value stands for,
///  or nothing for a value this build does not know.
/// Why:
///  A finding must print the state a person would see in `zfs get`,
///  so each index value is one word and an unknown number is reported as a
///  value this build cannot name rather than guessed into a word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const word = snapdirWord(2); // 'disabled'
/// ```
pub fn snapdir_word(value: u64) -> Option<&'static str> {
    return match value {
        SNAPDIR_HIDDEN => Some("hidden"),
        SNAPDIR_VISIBLE => Some("visible"),
        SNAPDIR_DISABLED => Some("disabled"),
        _ => None,
    };
}

//endregion

//region What the reply said

/// What:
///  The setting one dataset reported,
///  in the four states a report keeps apart:
///  one of the three words,
///  the default the reply leaves out,
///  a value this build does not name,
///  and a reply it could not read.
/// Why:
///  `disabled` passes and the other two words are findings that name the
///  word,
///  while an absent property is the default `hidden` rather than an unread
///  value,
///  and only an unreadable reply leaves the setting unproved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SnapdirSetting = { kind: 'word'; word } | { kind: 'default' } | { kind: 'unknown-value'; value } | { kind: 'unreadable'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapdirSetting {
    /// The reply carried one of the three words.
    Word(&'static str),
    /// The reply left the property out,
    /// which is the default `hidden`.
    Default,
    /// The reply carried a value this build does not name.
    UnknownValue(String),
    /// The reply's entry could not be read.
    Unreadable {
        /// What about the entry could not be read.
        why: String,
    },
}

/// What:
///  What one dataset's `snapdir` is,
///  from the property reply the dataset checks already hold.
/// Why:
///  The kernel writes a property as an index number and omits the default,
///  and a value written as text is the same word `zfs get` prints,
///  so both spellings are read as the word they carry instead of one of them
///  being called unread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const setting = snapdirSetting(properties); // { kind: 'word', word: 'disabled' }
/// ```
pub fn snapdir_setting(
    properties: &monochromatic_zfs_request_to_openzfs_kernel_device::NvList
) -> SnapdirSetting {
    return match value_of(properties, SNAPDIR_PROPERTY) {
        PropertyValue::Absent => SnapdirSetting::Default,
        PropertyValue::Number(value) => match snapdir_word(value) {
            Some(word) => SnapdirSetting::Word(word),
            None => SnapdirSetting::UnknownValue(value.to_string()),
        },
        PropertyValue::Text(text) => match text.as_str() {
            "hidden" => SnapdirSetting::Word("hidden"),
            "visible" => SnapdirSetting::Word("visible"),
            "disabled" => SnapdirSetting::Word("disabled"),
            other => SnapdirSetting::UnknownValue(String::from(other)),
        },
        PropertyValue::Unreadable {
            why,
        } => SnapdirSetting::Unreadable {
            why,
        },
    };
}

//endregion

//region The sentences

/// What the route reads the setting for on the checkout dataset.
const CHECKOUT_SUBJECT: &str =
    "the checkout dataset, whose `.zfs/snapshot` an account browsing the checkout meets";

/// What the route reads the setting for on the clone container.
const CONTAINER_SUBJECT: &str = "the clone container, whose value every clone below it inherits, so this one setting is what hides each clone's `.zfs/snapshot` as well";

/// What the route reads the setting for on one clone,
/// which the step reports only through its container.
const CLONE_SUBJECT: &str = "a clone below the container, which inherits the container's value unless a setting stands on the clone itself";

/// What:
///  The subject a report line names for one dataset's role.
/// Why:
///  The container's report has to say that its value is what the clones
///  inherit,
///  which is the half of the setting a person would otherwise miss while
///  every clone keeps its own directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const subject = role === 'container' ? CONTAINER_SUBJECT : CHECKOUT_SUBJECT;
/// ```
fn subject_of(role: &DatasetRole) -> &'static str {
    return match role {
        DatasetRole::Checkout {
            ..
        } => CHECKOUT_SUBJECT,
        DatasetRole::Container => CONTAINER_SUBJECT,
        DatasetRole::Clone => CLONE_SUBJECT,
    };
}

/// What:
///  The sentence that says what the value does,
///  which command sets it,
///  and that a module older than OpenZFS 2.3.0 has no such value at all.
/// Why:
///  A person reading the finding must learn three things from the line:
///  that only `disabled` stops the kernel mounting a snapshot entered by name,
///  the exact command to run,
///  and that the value does not exist on an older module,
///  where the mount table's bound is all that stands in the directories'
///  place.
///  The command is printed and not run,
/// because `doctor` changes nothing,
///  and the module floor is stated of such a module rather than judged from
///  this machine:
/// no release string is compared with a rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remedy = remedySentence('zpcachyos/…/checkout'); // '`zfs set snapdir=disabled …` …'
/// ```
fn remedy_sentence(dataset: &str) -> String {
    return format!(
        "`zfs set snapdir=disabled {dataset}` sets it, printed here and not run, because `doctor` changes nothing; `disabled` exists from OpenZFS 2.3.0, and a module older than that has no such value to set, so on one of those this setting cannot be made at all and the mount table's bound is what stands in the snapshot directories' place"
    );
}

//endregion

//region The check

/// What:
///  One dataset's `snapdir` against the value the project decided.
/// Why:
///  The decided state is `disabled` on the checkout dataset and on the clone
///  container,
///  so a dataset carrying another value or the default is a finding naming the
///  value and the command,
///  and a dataset whose reply this run could not read is unknown rather than a
///  finding about a value nobody saw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkSnapdir(facts); // pass | finding('snapdir') | unknown
/// ```
pub fn check_snapdir(facts: &DatasetFacts) -> CheckOutcome {
    let name: &str = facts.name.as_str();
    let subject: &'static str = subject_of(&facts.role);
    let read = match &facts.reading {
        Reading::Read(read) => read,
        Reading::Missing {
            why,
        } => {
            return finding(
                "dataset",
                format!(
                    "`{name}` is not in its pool ({why}), so its `{SNAPDIR_PROPERTY}` cannot hold anything"
                ),
            );
        }
        Reading::Unreadable {
            why,
        } => {
            return unknown(format!(
                "`{name}` could not be read ({why}), so its `{SNAPDIR_PROPERTY}` is unproved"
            ));
        }
    };
    return match snapdir_setting(&read.properties) {
        SnapdirSetting::Word("disabled") => pass(format!(
            "`{name}`'s `{SNAPDIR_PROPERTY}` is `disabled`, so the kernel refuses a snapshot directory's name on {subject}"
        )),
        SnapdirSetting::Word(word) => finding(
            "snapdir",
            format!(
                "`{name}`'s `{SNAPDIR_PROPERTY}` is `{word}`, and only `disabled` keeps the kernel from mounting a snapshot when an account enters its directory by name, which is how a listing grows the machine's mount table; on {subject}: {}",
                remedy_sentence(name)
            ),
        ),
        SnapdirSetting::Default => finding(
            "snapdir",
            format!(
                "`{name}`'s `{SNAPDIR_PROPERTY}` is unset, which is the default `hidden` and leaves the snapshot directory out of a listing while still letting an account enter a snapshot by name; on {subject}: {}",
                remedy_sentence(name)
            ),
        ),
        SnapdirSetting::UnknownValue(value) => unknown(format!(
            "`{name}`'s `{SNAPDIR_PROPERTY}` is `{value}`, a value this build does not name, so whether it hides the snapshot directories is unproved"
        )),
        SnapdirSetting::Unreadable {
            why,
        } => unknown(format!(
            "`{name}`'s `{SNAPDIR_PROPERTY}` could not be read ({why}), so the setting is unproved"
        )),
    };
}

/// What:
///  The report line one dataset's `snapdir` produces,
///  or nothing for a dataset whose role the step does not report it for.
/// Why:
///  The step reports the setting of the checkout dataset and of the clone
///  container only,
///  and a clone's own value is the container's by inheritance,
///  so the role decides both the check's name and whether there is a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = snapdir_report(facts); // { check: 'container-snapdir', outcome } | undefined
/// ```
pub fn snapdir_report(facts: &DatasetFacts) -> Option<CheckReport> {
    let check: Check = match facts.role {
        DatasetRole::Checkout {
            ..
        } => Check::CheckoutSnapdir,
        DatasetRole::Container => Check::ContainerSnapdir,
        DatasetRole::Clone => return None,
    };
    return Some(CheckReport::new(check, check_snapdir(facts)));
}

//endregion

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "snapshot_directory_checks_tests.rs"]
mod tests;
