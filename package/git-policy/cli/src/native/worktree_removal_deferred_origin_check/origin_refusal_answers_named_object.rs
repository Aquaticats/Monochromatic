//! What:
//!  The clone and the slot a refusal names:
//!  the clone's full dataset name where the cause knows it,
//!  and the slot the removal was asked for.
//! Why:
//!  A refusal is printed for one attempt at one slot,
//!  so the sentence and the record name the object the refusal is about,
//!  and a person reading it can tell which clone and which slot stopped the
//!  run
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "Reading the mark").
//!  The fields each cause carries are matched in one place here,
//!  apart from the words and the message that use them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! cause.cloneName(); // 'pool/clone'
//! cause.slot(); // 'slot-name'
//! ```

use super::origin_refusal::OriginRefusal;
use super::planned_origin::PlannedOrigin;

impl OriginRefusal {
    /// The clone's full dataset name,
    /// where the cause knows it.
    pub fn clone_name(&self) -> Option<&str> {
        return match self {
            OriginRefusal::PlanIncomplete {
                ..
            } => None,
            OriginRefusal::OriginMarkedBeforeTheUnmount {
                clone_name,
                ..
            }
            | OriginRefusal::MarkUnreadable {
                clone_name,
                ..
            }
            | OriginRefusal::CloneGone {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
            }
            | OriginRefusal::CloneIsNotAFilesystem {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
                ..
            }
            | OriginRefusal::CloneIsNotThePlannedOne {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
                ..
            }
            | OriginRefusal::CloneHasNoOrigin {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
            }
            | OriginRefusal::CloneOriginDiffers {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
                ..
            }
            | OriginRefusal::OriginDatasetGone {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
            }
            | OriginRefusal::OriginDatasetIsNotAFilesystem {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
                ..
            }
            | OriginRefusal::OriginDatasetIsNotThePlannedOne {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
                ..
            }
            | OriginRefusal::OriginSnapshotGone {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
            }
            | OriginRefusal::OriginIsNotASnapshot {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
            }
            | OriginRefusal::OriginSnapshotIsNotThePlannedOne {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
                ..
            }
            | OriginRefusal::OriginSnapshotCreatetxgDiffers {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
                ..
            }
            | OriginRefusal::OriginMarkedForDeferredDestroy {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
                ..
            }
            | OriginRefusal::ReadFailed {
                planned:
                    PlannedOrigin {
                        clone_name,
                        ..
                    },
                ..
            } => Some(clone_name.as_str()),
        };
    }

    /// The slot the removal was asked for.
    pub fn slot(&self) -> &str {
        return match self {
            OriginRefusal::PlanIncomplete {
                slot,
                ..
            }
            | OriginRefusal::OriginMarkedBeforeTheUnmount {
                slot,
                ..
            }
            | OriginRefusal::MarkUnreadable {
                slot,
                ..
            } => slot.as_str(),
            OriginRefusal::OriginMarkedForDeferredDestroy {
                planned,
                ..
            }
            | OriginRefusal::CloneGone {
                planned,
            }
            | OriginRefusal::CloneIsNotAFilesystem {
                planned,
                ..
            }
            | OriginRefusal::CloneIsNotThePlannedOne {
                planned,
                ..
            }
            | OriginRefusal::CloneHasNoOrigin {
                planned,
            }
            | OriginRefusal::CloneOriginDiffers {
                planned,
                ..
            }
            | OriginRefusal::OriginDatasetGone {
                planned,
            }
            | OriginRefusal::OriginDatasetIsNotAFilesystem {
                planned,
                ..
            }
            | OriginRefusal::OriginDatasetIsNotThePlannedOne {
                planned,
                ..
            }
            | OriginRefusal::OriginSnapshotGone {
                planned,
            }
            | OriginRefusal::OriginIsNotASnapshot {
                planned,
            }
            | OriginRefusal::OriginSnapshotIsNotThePlannedOne {
                planned,
                ..
            }
            | OriginRefusal::OriginSnapshotCreatetxgDiffers {
                planned,
                ..
            }
            | OriginRefusal::ReadFailed {
                planned,
                ..
            } => planned.slot.as_str(),
        };
    }
}
