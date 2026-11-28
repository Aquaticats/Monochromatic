//! What:
//!  What one run of the step names:
//!  the checkout and dataset its caller resolved,
//!  the places it reads and writes,
//!  the answer it hands back and the refusal it reports.
//! Why:
//!  The step's acts read and write these values,
//!  and a caller that wires the step needs them without the acts,
//!  so they sit in one file the acts and the records both use.
//!  The places are one value because the release run takes the installed
//!  record and a control hands in a fixture,
//!  and the answer and the refusal are one value each because the mode's line
//!  and its exit status are built from them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type CheckoutResolution = { kind: 'at'; checkout; dataset } | { kind: 'refused'; why };
//! type PropagationPlaces = { record: PropagationFile; boot: string };
//! type PropagationDone = { outcome; line };
//! type PropagationRefusal = { step; errno?; message };
//! ```

use crate::checkout_propagation::propagation_record_file::PropagationFile;
use crate::checkout_propagation::propagation_words::{
    PropagationOutcome,
    PropagationStep,
};
use std::path::PathBuf;

/// The path the kernel names this boot by,
/// which the readiness record reads the same way.
const BOOT_ID_PATH: &str = "/proc/sys/kernel/random/boot_id";

/// What:
///  The checkout and the dataset its mount must serve,
///  as the caller resolved them,
///  or the refusal that resolution answered.
/// Why:
///  The step takes the answer its caller resolved and never reads the
///  configuration itself,
///  so a file that names neither dataset leaves the step working and the same
///  two refusals reach it
///  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
///  "With no configuration file";
///  the design's own choice V5 puts the step after that resolution).
///  A resolution that refused is still this step's refusal to file,
///  with the word `resolve`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckoutResolution = { kind: 'at'; checkout: string; dataset: string } | { kind: 'refused'; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckoutResolution {
    /// The checkout's path and the dataset its mount must serve.
    At {
        /// The checkout's path.
        checkout: PathBuf,
        /// The dataset's name.
        dataset: String,
    },
    /// The caller's own resolution refused,
    /// and the step files that refusal.
    Refused {
        /// The sentence the caller's refusal said.
        why: String,
    },
}

/// What:
///  Where one run of the step reads and writes.
/// Why:
///  The release run takes the installed record place and this boot's
///  identifier,
///  while a control hands in a fixture directory and a stated boot,
///  so the places are one value and no step hardcodes them twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropagationPlaces = { record: PropagationFile; boot: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropagationPlaces {
    /// Where the record is published.
    pub record: PropagationFile,
    /// The boot identifier the record names.
    pub boot: String,
}

impl PropagationPlaces {
    /// What:
    ///  The installed places and this boot's own identifier.
    /// Why:
    ///  The handler runs from the installed unit,
    ///  and the record it writes must name the boot a reader compares it with.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const places = () => ({ record: PropagationFile.live(), boot: readFile('/proc/sys/kernel/random/boot_id').trim() });
    /// ```
    ///
    /// # Errors
    /// This boot's identifier could not be read.
    pub fn live() -> Result<Self, String> {
        let boot: String = crate::plain_file::read_plain_text(std::path::Path::new(BOOT_ID_PATH)).map_err(|error| {
            return format!(
                "this boot's identifier could not be read from {BOOT_ID_PATH} ({error}), so a propagation record would name no boot"
            );
        })?;
        return Ok(PropagationPlaces {
            record: PropagationFile::live(),
            boot: boot.trim().to_string(),
        });
    }
}

/// What:
///  What the step did,
///  and the line the mode says about it.
/// Why:
///  The mode's summary line is what an operator and a guest arm read,
///  so the step hands it back rather than printing it,
///  and it carries the same facts the record does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropagationDone = { outcome: PropagationOutcome; line: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropagationDone {
    /// What the step did.
    pub outcome: PropagationOutcome,
    /// The line the mode says.
    pub line: String,
}

/// What:
///  Why the step refused.
/// Why:
///  A refusal is the act that refused,
///  the kernel's number where one answered,
///  and the sentence a person reads,
///  and the mode turns it into the handler's exit status of 2.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PropagationRefusal = { step: PropagationStep; errno?: number; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropagationRefusal {
    /// The act that refused.
    pub step: PropagationStep,
    /// The kernel's error number,
    /// where a call answered one.
    pub errno: Option<i32>,
    /// The sentence a person reads.
    pub message: String,
}
