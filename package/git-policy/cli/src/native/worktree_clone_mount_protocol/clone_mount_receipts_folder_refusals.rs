//! What:
//!  Why the results directory or one receipt could not be used:
//!  the directory's ownership and mode,
//!  the receipt's ownership,
//!  mode and shape,
//!  and the staging sibling's own failures.
//! Why:
//!  A caller acts only on a result it can prove root filed,
//!  so each deviation names the path and says nothing was matched on it
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering",
//!  "Decided defaults").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class ReceiptsRefusal extends Error {}
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_receipt::MAXIMUM_RECEIPT_BYTES;
use crate::worktree_clone_mount_protocol::clone_mount_receipt_refusals::ReceiptRefusal;
use std::path::PathBuf;

/// The directory the route files its results in.
pub const RESULTS_DIRECTORY: &str = "/run/git-worktree-clone-mount-results";

/// The directory's designed mode.
pub const DIRECTORY_MODE: u32 = 0o755;

/// The receipt's designed mode.
pub const RECEIPT_MODE: u32 = 0o644;

/// The words a refusal prints about the `doctor` lines and the handler unit this
/// release does not have.
/// Why:
///  Plan steps 18 and 19 are built ahead:
///  `doctor` reaches none of the route's checks and nothing installs the boot
///  unit,
///  so a sentence that sends a person to `doctor` for a record or to that unit's
///  journal sends them to a line no command prints and a journal no unit writes
///  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).
///  The round that wires either one deletes this claim,
///  and every sentence that prints it follows from one place.
pub const NO_DOCTOR_LINE_AND_NO_INSTALLED_UNIT: &str = "This release prints no `doctor` line for a receipt or a readiness record and installs no handler unit, so `git monochromatic-git-policy-cli doctor` reports neither and `journalctl -u reconcile-git-worktree-clone-mounts.service` has rows only where an administrator installed that unit";

/// What:
///  Why the results directory or one receipt could not be used.
/// Why:
///   The caller must not act on a file it could not prove root wrote,
///  so each deviation from the designed ownership,
///  mode or shape has its own cause and names the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReceiptsRefusal = { kind: 'directory' | 'owner' | 'mode' | 'file' | 'shape' | 'staging'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReceiptsRefusal {
    /// The directory could not be opened,
    /// or is not a directory.
    Directory {
        /// The path.
        path: PathBuf,
        /// What the call or the facts answered.
        reason: String,
    },
    /// The directory is not there at all,
    /// which on a fresh boot is the answer before the handler's first publish.
    DirectoryAbsent {
        /// The path.
        path: PathBuf,
    },
    /// The directory is not owned by the expected account.
    DirectoryOwner {
        /// The path.
        path: PathBuf,
        /// The owner found.
        found: u32,
        /// The owner the route requires.
        wanted: u32,
    },
    /// The directory does not carry its designed mode.
    DirectoryMode {
        /// The path.
        path: PathBuf,
        /// The mode found,
        /// its permission bits only.
        found: u32,
    },
    /// One receipt could not be opened or written.
    File {
        /// The file's name.
        name: String,
        /// What the call answered.
        reason: String,
    },
    /// One receipt is not owned by the expected account.
    FileOwner {
        /// The file's name.
        name: String,
        /// The owner found.
        found: u32,
        /// The owner the route requires.
        wanted: u32,
    },
    /// One receipt does not carry its designed mode.
    FileMode {
        /// The file's name.
        name: String,
        /// The mode found,
        /// its permission bits only.
        found: u32,
    },
    /// The name does not hold a regular file.
    NotRegular {
        /// The file's name.
        name: String,
    },
    /// The receipt is longer than a receipt may be.
    TooLarge {
        /// The file's name.
        name: String,
        /// How many bytes the file holds.
        bytes: u64,
    },
    /// The receipt's bytes are not text.
    NotText {
        /// The file's name.
        name: String,
    },
    /// The receipt's own dataset GUID is not the name it is filed under.
    WrongDataset {
        /// The file's name.
        name: String,
        /// The GUID the file carries.
        found: String,
    },
    /// The writer built a text its own reader does not read back as the same
    /// receipt,
    /// so nothing could ever read it.
    NotWhatWasWritten {
        /// The file's name.
        name: String,
    },
    /// The text is not a receipt this route wrote.
    NotAReceipt {
        /// The file's name.
        name: String,
        /// What the parse refused.
        refusal: ReceiptRefusal,
    },
    /// The staging sibling could not be made or renamed.
    Staging {
        /// The staging name.
        name: String,
        /// What the call answered.
        reason: String,
    },
}

impl ReceiptsRefusal {
    /// The refusal in words,
    /// naming the path and the way forward.
    pub fn message(&self) -> String {
        let reason: String = match self {
            ReceiptsRefusal::Directory {
                path,
                reason,
            } => format!(
                "the results directory {} could not be opened ({reason})",
                path.display()
            ),
            ReceiptsRefusal::DirectoryAbsent {
                path,
            } => format!(
                "the results directory {} does not exist yet",
                path.display()
            ),
            ReceiptsRefusal::NotWhatWasWritten {
                name,
            } => format!(
                "the receipt `{name}` this route built is not one its own reader reads back, so it was not written; a field of it holds a character or a length the receipt's grammar refuses"
            ),
            ReceiptsRefusal::DirectoryOwner {
                path,
                found,
                wanted,
            } => format!(
                "the results directory {} is owned by user {found}, and this route requires user {wanted}",
                path.display()
            ),
            ReceiptsRefusal::DirectoryMode {
                path,
                found,
            } => format!(
                "the results directory {} has mode {found:04o}, and this route requires {DIRECTORY_MODE:04o}",
                path.display()
            ),
            ReceiptsRefusal::File {
                name,
                reason,
            } => format!("the receipt `{name}` could not be opened or written ({reason})"),
            ReceiptsRefusal::FileOwner {
                name,
                found,
                wanted,
            } => {
                format!(
                    "the receipt `{name}` is owned by user {found}, and this route requires user {wanted}"
                )
            }
            ReceiptsRefusal::FileMode {
                name,
                found,
            } => format!(
                "the receipt `{name}` has mode {found:04o}, and this route requires {RECEIPT_MODE:04o}"
            ),
            ReceiptsRefusal::NotRegular {
                name,
            } => format!("`{name}` does not name a regular file, so it is not a receipt"),
            ReceiptsRefusal::TooLarge {
                name,
                bytes,
            } => {
                format!(
                    "the receipt `{name}` is {bytes} bytes and a receipt is at most {MAXIMUM_RECEIPT_BYTES}"
                )
            }
            ReceiptsRefusal::NotText {
                name,
            } => format!("the receipt `{name}` is not UTF-8 text"),
            ReceiptsRefusal::WrongDataset {
                name,
                found,
            } => format!(
                "the receipt `{name}` carries dataset GUID {found} and is filed under another GUID"
            ),
            ReceiptsRefusal::NotAReceipt {
                name,
                refusal,
            } => {
                return format!(
                    "{}\nThe file is `{name}` in {RESULTS_DIRECTORY}.",
                    refusal.message()
                );
            }
            ReceiptsRefusal::Staging {
                name,
                reason,
            } => format!(
                "the sibling `{name}` a receipt is staged under could not be made or renamed ({reason}); a rename that completed before a failed directory sync may still be visible after a crash, so read the receipt before retrying"
            ),
        };
        return format!(
            "{reason}. Nothing was matched, mounted or unmounted on this file, because a caller acts only on a result it can prove root filed. Ways forward: as the account, read the receipt files under {RESULTS_DIRECTORY}, one per clone dataset and named for that dataset's GUID. {NO_DOCTOR_LINE_AND_NO_INSTALLED_UNIT}; a wrong owner or mode needs an administrator, since this route never changes either."
        );
    }
}
