//! What:
//!  Whether one result is the answer to the request the caller made:
//!  the same dataset GUID and the same nonce,
//!  and the refusal that names a foreign dataset or an old nonce.
//! Why:
//!  A result for another clone,
//!  or one written for the previous scan of this clone,
//!  must never be read as this request's answer,
//!  so the reader compares both fields with its own request before acting on
//!  anything the result holds
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces",
//!  which asks for a fresh,
//!  nonce- and GUID-bound result).
//!  The GUID says which dataset the answer is about and the nonce says which
//!  request of it,
//!  and both are the writer's own values rather than anything a caller
//!  composed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! scan.accept({ dataset, nonce }); // throws StaleResult
//! ```

use crate::clone_holder_scan::holder_scan::HolderScan;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// What:
///  The dataset and the request a caller is holding.
/// Why:
///   The two fields are what a result must match,
///  and keeping them together means a caller cannot compare one request's
///  nonce with another clone's result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScanExpectation = { dataset: string; nonce: string };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ScanExpectation<'a> {
    /// The dataset GUID the caller asked about.
    pub dataset: DatasetGuid,
    /// The nonce the caller's request carries.
    pub nonce: &'a RequestNonce,
}

/// What:
///  Why one result is not the answer to the request at hand.
/// Why:
///  A result for another dataset and a result for an earlier request of this
///  dataset are two different mistakes for a person to look at,
///  and each names both values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StaleResult = { kind: 'foreign-dataset' | 'another-nonce'; expected; found };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaleResult {
    /// The result names another dataset.
    ForeignDataset {
        /// The GUID the caller asked about.
        expected: DatasetGuid,
        /// The GUID the result carries.
        found: DatasetGuid,
    },
    /// The result answers another request of this dataset.
    AnotherNonce {
        /// The nonce the caller holds.
        expected: String,
        /// The nonce the result carries.
        found: String,
    },
}

impl StaleResult {
    /// The refusal in words,
    /// naming both values and the way forward.
    pub fn message(&self) -> String {
        return match self {
            StaleResult::ForeignDataset {
                expected,
                found,
            } => format!(
                "this holder scan's result is about dataset GUID {} and the request at hand is about GUID {}. Nothing was read from it. Ways forward: scan the dataset the request names, and read only the result that carries that dataset's GUID.",
                found.file_name(),
                expected.file_name()
            ),
            StaleResult::AnotherNonce {
                expected,
                found,
            } => format!(
                "this holder scan's result answers the request with nonce {found}, and the request at hand carries {expected}. Nothing was read from it. Ways forward: scan again under the request at hand; a result is one request's answer and no earlier scan answers a later request."
            ),
        };
    }
}

impl HolderScan {
    /// Whether this result is the answer to one request,
    /// and the refusal naming the field that differs when it is not.
    ///
    /// # Errors
    /// The result names another dataset or answers another request.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scan.accept({ dataset, nonce });
    /// ```
    pub fn accept(
        &self,
        expectation: &ScanExpectation<'_>,
    ) -> Result<(), StaleResult> {
        if self.dataset != expectation.dataset {
            return Err(StaleResult::ForeignDataset {
                expected: expectation.dataset,
                found: self.dataset,
            });
        }
        if self.nonce.as_str() != expectation.nonce.as_str() {
            return Err(StaleResult::AnotherNonce {
                expected: String::from(expectation.nonce.as_str()),
                found: String::from(self.nonce.as_str()),
            });
        }
        return Ok(());
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "holder_scan_acceptance_tests.rs"]
mod tests;
