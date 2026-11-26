//! What:
//!  Why one invocation could not be read as a mode:
//!  one variant per cause,
//!  naming the argument a person typed and every way forward.
//! Why:
//!  `handler_mode_arguments.rs` reads the words and would go over the
//!  file-size limit carrying the answers too,
//!  and each answer is what a person meeting a refused mode reads,
//!  so a reader holds the refusals together apart from the grammar that raises
//!  them.
//!  Every refusal opens with the words every root mode is reached through and
//!  says what was not done.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ArgumentRefusal = { kind: 'unknown-mode'; word: string } | { kind: 'missing-option'; option: string };
//! ```

use super::handler_mode_words::{
    CHECKOUT_OPTION,
    DATASET_OPTION,
    DEVICE_OPTION,
    INODE_OPTION,
    MODE_WORDS,
    NONCE_OPTION,
    REPLACE_OPTION,
};
use crate::worktree_clone_mount_handler::handler_install_paths::root_modes_name;
use crate::worktree_clone_mount_handler::root_mode_names::ROOT_GROUP_WORD;

/// What:
///  Why one invocation could not be read as a mode.
/// Why:
///   Each cause names the argument a person typed,
///  so the answer says what to change rather than only that the input was wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ArgumentRefusal = { kind: 'not-text' | 'unknown-mode' | 'missing-value' | ...; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgumentRefusal {
    /// An argument is not UTF-8 text.
    NotText {
        /// Its one-based place among the arguments,
        /// which is where a person sees it.
        place: usize,
    },
    /// The leading word is no mode of this build.
    UnknownMode {
        /// The word as typed.
        word: String,
    },
    /// An option came last and has no value.
    MissingValue {
        /// The option.
        option: &'static str,
    },
    /// An option was given more than once,
    /// so two identifications would compete.
    RepeatedOption {
        /// The option.
        option: &'static str,
    },
    /// The adoption's words lack an option it needs,
    /// or a holder scan's do.
    MissingOption {
        /// The option.
        option: &'static str,
    },
    /// The nonce is not one this route carries.
    Nonce {
        /// What the parse answered.
        reason: String,
    },
    /// The dataset a mode names is no dataset name this route reads.
    DatasetName {
        /// The mode that named it.
        mode: &'static str,
        /// What that mode's dataset is,
        /// as the phrase after the mode's name.
        what: &'static str,
        /// The name as typed.
        name: String,
        /// Why it is no dataset name.
        reason: String,
    },
    /// An option's value is not the decimal count it must be.
    OptionNotACount {
        /// The option.
        option: &'static str,
        /// The value as typed.
        text: String,
    },
    /// A word this invocation does not take,
    /// after the mode or after an option's value.
    UnexpectedArgument {
        /// The word as typed.
        word: String,
    },
}

impl ArgumentRefusal {
    /// The refusal in words,
    /// naming the argument and every way forward,
    /// and opening with the words every root mode is reached through.
    pub fn message(&self) -> String {
        let named: String = root_modes_name();
        let prefix: String = format!("{named}: ");
        return match self {
            ArgumentRefusal::NotText {
                place,
            } => format!(
                "{prefix}argument {place} is not text, and every argument the root modes read is UTF-8 text. Nothing was done. Ways forward: give the mode word and the adoption's numbers as text; a checkout path that is not valid UTF-8 cannot be named here, so name a checkout whose path is text."
            ),
            ArgumentRefusal::UnknownMode {
                word,
            } => format!(
                "{prefix}`{word}` is not a mode. Nothing was done. The modes are {}, and running the command with no mode word after `{ROOT_GROUP_WORD}` is refused rather than run. Ways forward: run `{named} --help` for the modes this build accepts.",
                MODE_WORDS.join(", ")
            ),
            ArgumentRefusal::MissingValue {
                option,
            } => format!(
                "{prefix}`{option}` is the last argument and has no value. Nothing was done. Ways forward: give `{option}` its value, as `{option} <value>`, and run the command again."
            ),
            ArgumentRefusal::RepeatedOption {
                option,
            } => format!(
                "{prefix}`{option}` was given more than once, and two values would compete. Nothing was done. Ways forward: give `{option}` once, then run the command again."
            ),
            ArgumentRefusal::MissingOption {
                option,
            } => format!(
                "{prefix}this invocation needs `{option}`, and it was not given. Nothing was done. Ways forward: run `{named} --help` for the options the mode takes and give every one of them; `adopt-worktree-parent` takes `{CHECKOUT_OPTION} <path> {DEVICE_OPTION} <count> {INODE_OPTION} <count>`, `holder-scan` takes `{DATASET_OPTION} <name>` and, only when the caller names one, `{NONCE_OPTION} <32 hex digits>`, `record-checkout` takes `{CHECKOUT_OPTION} <path>` and, only when it replaces a record a restore moved, `{REPLACE_OPTION} <the kernel's guid>`, and `mount-path-descendant` takes `{DATASET_OPTION} <name>`."
            ),
            ArgumentRefusal::Nonce {
                reason,
            } => format!(
                "{prefix}the nonce is not one this route carries: {reason}. Nothing was done. Ways forward: give `{NONCE_OPTION}` exactly {} lowercase hexadecimal digits, as the wrapper's own request carries them, or leave the option out and the scan makes its own.",
                crate::worktree_clone_mount_protocol::clone_mount_property_names::NONCE_HEX_DIGITS
            ),
            ArgumentRefusal::DatasetName {
                mode,
                what,
                name,
                reason,
            } => format!(
                "{prefix}`{name}` is not a dataset name this route reads: {reason}. Nothing was done. Ways forward: give `{mode}`'s `{DATASET_OPTION}` {what}."
            ),
            ArgumentRefusal::OptionNotACount {
                option,
                text,
            } => format!(
                "{prefix}`{option} {text}` is not a decimal count. Nothing was done. Ways forward: give the number `stat --format=%d` (device) or `stat --format=%i` (inode) prints for the intended replacement, as decimal digits without a sign or a prefix."
            ),
            ArgumentRefusal::UnexpectedArgument {
                word,
            } => format!(
                "{prefix}`{word}` is an argument this invocation does not take. Nothing was done. Ways forward: run `{named} --help` for the modes and their arguments; a mode takes only the words the help lists beside it."
            ),
        };
    }
}
