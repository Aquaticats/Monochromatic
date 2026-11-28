//! What:
//!  The words one invocation of the root modes is read as:
//!  no argument at all for the clone-mount sweep ZED's own start runs,
//!  `boot-reconcile`,
//!  `start-probe`,
//!  `shutdown-unmount`,
//!  `holder-scan` with the dataset the scan reports on,
//!  `adopt-worktree-parent` with the replacement's device and inode,
//!  `record-checkout` with the checkout it records,
//!  `mount-path-descendant` with the nested dataset it mounts,
//!  and the two words that print what this build is.
//! Why:
//!  The design names the modes and no argument
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit",
//!  "The administrator's adoption"),
//!  and the plan's suggestion is that the default invocation,
//!  which is how ZED's own start calls it,
//!  runs the sweep
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Where the design is silent:
//! this plan's suggestions").
//!  This grammar is reached through the wrapper's own executable now,
//!  so every mode is named after the words `monochromatic-git-policy-cli zfs-root`
//!  (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`,
//!  choice 2),
//!  and every refusal opens with those words:
//!  the family name is printed nowhere.
//!  A mode is a word,
//!  never a command line the caller composes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mode = parse(words); // { kind: 'attaching-sweep' } | { kind: 'boot-reconcile' } | ...
//! ```

use super::handler_mode_arguments_option_reads::{
    parse_adoption,
    parse_holder_scan,
    parse_mount_path_descendant,
    parse_record_checkout,
};
use std::ffi::OsString;

/// The words one invocation is read as:
/// the mode words,
/// the options each mode takes and the usage text.
pub use super::handler_mode_words::{
    CHECKOUT_OPTION,
    DATASET_OPTION,
    DEVICE_OPTION,
    HELP_WORD,
    INODE_OPTION,
    MODE_WORDS,
    NONCE_OPTION,
    REPLACE_OPTION,
    ROOT_MODE_FILE_PLACEHOLDER,
    VERSION_WORD,
    usage,
};

/// Why one invocation could not be read as a mode.
pub use super::handler_mode_argument_refusals::ArgumentRefusal;

/// What:
///  Read the arguments as one mode.
/// Why:
///   The grammar is one leading word and,
///  for an adoption only,
///  three options with one value each;
///  every other word,
///  repetition and unreadable argument is a refusal rather than a guess,
///  because guessing a mode at root would start a sweep nobody asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mode = parse(argv); // throws ArgumentRefusal
/// ```
///
/// # Errors
/// A word that is no mode,
/// a missing,
/// repeated or non-text option,
/// a value that is no decimal count,
/// or another word where the invocation takes none.
pub fn parse(arguments: &[OsString]) -> Result<HandlerMode, ArgumentRefusal> {
    if arguments.is_empty() {
        return Ok(HandlerMode::AttachingSweep);
    }
    let mut words: Vec<&str> = Vec::with_capacity(arguments.len());
    for (place, argument) in arguments.iter().enumerate() {
        let Some(word) = argument.to_str() else {
            return Err(ArgumentRefusal::NotText {
                place: place + 1,
            });
        };
        words.push(word);
    }
    let Some(first) = words.first().copied() else {
        return Ok(HandlerMode::AttachingSweep);
    };
    return match first {
        HELP_WORD => match words.len() {
            1 => Ok(HandlerMode::Help),
            _ => Err(ArgumentRefusal::UnexpectedArgument {
                word: words[1].to_string(),
            }),
        },
        VERSION_WORD => match words.len() {
            1 => Ok(HandlerMode::Version),
            _ => Err(ArgumentRefusal::UnexpectedArgument {
                word: words[1].to_string(),
            }),
        },
        "boot-reconcile" => match words.len() {
            1 => Ok(HandlerMode::BootReconcile),
            _ => Err(ArgumentRefusal::UnexpectedArgument {
                word: words[1].to_string(),
            }),
        },
        "start-probe" => match words.len() {
            1 => Ok(HandlerMode::StartProbe),
            _ => Err(ArgumentRefusal::UnexpectedArgument {
                word: words[1].to_string(),
            }),
        },
        "shutdown-unmount" => match words.len() {
            1 => Ok(HandlerMode::ShutdownUnmount),
            _ => Err(ArgumentRefusal::UnexpectedArgument {
                word: words[1].to_string(),
            }),
        },
        "holder-scan" => parse_holder_scan(&words[1..]).map(HandlerMode::HolderScan),
        "adopt-worktree-parent" => {
            parse_adoption(&words[1..]).map(HandlerMode::AdoptWorktreeParent)
        }
        "record-checkout" => parse_record_checkout(&words[1..]).map(HandlerMode::RecordCheckout),
        "mount-path-descendant" => {
            parse_mount_path_descendant(&words[1..]).map(HandlerMode::MountPathDescendant)
        }
        other => Err(ArgumentRefusal::UnknownMode {
            word: other.to_string(),
        }),
    };
}

/// What one invocation asks this build to do,
/// and the identities each mode that takes an argument is handed,
/// so a caller reaches them through this module as it did before the split.
pub use super::handler_mode_arguments_requests::{
    AdoptRequest,
    HandlerMode,
    HolderScanInvocation,
    MountPathDescendantRequest,
    RecordCheckoutRequest,
};

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "handler_mode_arguments_tests.rs"]
mod tests;
