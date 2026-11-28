//! What:
//!  The controls of each clone's request and the receipt that answers it:
//!  a clone that asks under a nonce and a receipt of this boot that answers it,
//!  a clone that asks for nothing,
//!  a property the grammar refuses,
//!  and a receipt the folder's owner term refuses.
//! Why:
//!  A request with no receipt of this boot is the one work item a person can
//!  act on,
//!  while a property or a receipt nobody could read proves nothing,
//!  so the two arms must be shown apart from a fixture folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theRequestFacts({ datasets, receipts, unread })[0]).toEqual({ clone, intentNonce, receiptNonce, receiptBoot });
//! ```

/// The read a control judges.
use super::the_request_facts;
/// The check a failed read answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// One clone's request and the receipt that answers it.
use crate::doctor_zfs_route::handler_checks_unresolved_requests::RequestFacts;
/// The names the design's layout fixes.
use crate::doctor_zfs_route::route_test_facts::CLONE_DATASET;
/// One dataset's facts,
/// with a control's own properties.
use crate::doctor_zfs_route::route_test_facts::DatasetBuilder;
/// One dataset's role and reading.
use crate::doctor_zfs_route::zfs_checks_dataset_roles::{
    DatasetFacts,
    DatasetRole,
};
/// What a dataset's read answered.
use crate::doctor_zfs_route::zfs_checks_reading::Reading;
/// One fixture folder of this call's own.
use crate::testing::test_support::fixture;
/// The account this process runs as.
use crate::worktree_clone_add_route::add_grants::this_account;
/// The property a clone asks through,
/// the nonce it asks under,
/// and the boot this report judges records against.
use crate::worktree_clone_mount_protocol::{
    DESIRED_STATE_PROPERTY,
    ReceiptsFolder,
};
/// The binding's name-value list,
/// which a dataset's properties are.
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;
/// The owned paths these controls use.
use std::path::PathBuf;

/// The nonce a control's clone asks under.
const A_NONCE: &str = "0123456789abcdef0123456789abcdef";

/// What:
///  A clone whose own property names a nonce answers that nonce,
///  and a clone whose property names none answers no request.
/// Why:
///  The unresolved-requests check compares a nonce with a receipt's,
///  so a request read from another clone's property would report work nobody
///  asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(requests[0].intentNonce).toBe(A_NONCE);
/// expect(requests[1].intentNonce).toBeUndefined();
/// ```
#[test]
fn a_clone_that_asks_answers_its_nonce_and_one_that_asks_nothing_answers_none() {
    let receipts: ReceiptsFolder = the_receipts("requests-asking");
    let mut unread: Vec<CheckReport> = Vec::new();
    let asking: Vec<DatasetFacts> = vec![a_clone(&format!("v1:{A_NONCE}:mount"))];
    let requests: Vec<RequestFacts> = the_request_facts(asking.as_slice(), &receipts, &mut unread);
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].clone, CLONE_DATASET);
    assert_eq!(requests[0].intent_nonce.as_deref(), Some(A_NONCE));
    assert_eq!(requests[0].receipt_nonce, None);
    assert_eq!(requests[0].receipt_boot, None);
    assert!(unread.is_empty(), "{unread:?}");
    let quiet: Vec<DatasetFacts> = vec![a_clone_without_a_property()];
    let quiet_requests: Vec<RequestFacts> =
        the_request_facts(quiet.as_slice(), &receipts, &mut unread);
    assert_eq!(quiet_requests[0].intent_nonce, None);
    assert!(unread.is_empty(), "{unread:?}");
}

/// What:
///  A property the grammar refuses answers the requests' check an unknown
///  naming the clone,
///  and the clone keeps no nonce.
/// Why:
///  Words this route refuses prove nothing about the request,
///  so reporting the clone as unresolved would accuse it of asking for work
///  nobody can read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(unread.map((r) => r.check)).toEqual(['unresolved-requests']);
/// ```
#[test]
fn a_property_the_grammar_refuses_answers_an_unknown() {
    let receipts: ReceiptsFolder = the_receipts("requests-refused");
    let mut unread: Vec<CheckReport> = Vec::new();
    let refused: Vec<DatasetFacts> = vec![a_clone("words this grammar refuses")];
    let requests: Vec<RequestFacts> = the_request_facts(refused.as_slice(), &receipts, &mut unread);
    assert_eq!(requests[0].intent_nonce, None);
    assert_eq!(
        unread
            .iter()
            .map(|report| return report.check)
            .collect::<Vec<Check>>(),
        vec![Check::UnresolvedRequests]
    );
    assert!(
        unread[0].line().contains(CLONE_DATASET),
        "{}",
        unread[0].line()
    );
}

/// What:
///  A receipt filed by another account answers the requests' check an unknown,
///  because a receipt another account could write decides nothing.
/// Why:
///  The folder's owner term is what makes a receipt an answer the handler filed,
///  and a control's folder belongs to the account running it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(unread.map((r) => r.check)).toEqual(['unresolved-requests']);
/// ```
#[test]
fn a_receipt_another_account_could_have_written_answers_an_unknown() {
    let root: PathBuf = fixture("requests-receipt-owner");
    let results: PathBuf = root.join("run/git-worktree-clone-mount-results");
    std::fs::create_dir_all(results.as_path()).expect("the results folder");
    let receipts: ReceiptsFolder = ReceiptsFolder::at(results, 0);
    let mut unread: Vec<CheckReport> = Vec::new();
    let asking: Vec<DatasetFacts> = vec![a_clone(&format!("v1:{A_NONCE}:mount"))];
    let requests: Vec<RequestFacts> = the_request_facts(asking.as_slice(), &receipts, &mut unread);
    assert_eq!(requests.len(), 1);
    assert_eq!(
        unread
            .iter()
            .map(|report| return report.check)
            .collect::<Vec<Check>>(),
        vec![Check::UnresolvedRequests]
    );
}

/// One receipts folder at a fixture of this call's own,
/// owned by the account running the control.
fn the_receipts(name: &str) -> ReceiptsFolder {
    let root: PathBuf = fixture(name);
    let results: PathBuf = root.join("run/git-worktree-clone-mount-results");
    std::fs::create_dir_all(results.as_path()).expect("the results folder");
    return ReceiptsFolder::at(results, this_account());
}

/// One clone whose property names the intent the control chose.
fn a_clone(intent: &str) -> DatasetFacts {
    let facts = DatasetBuilder::clone().build();
    let mut properties: NvList = facts.properties.clone();
    let mut pair: NvList = NvList::new();
    pair.add_str("value", intent);
    pair.add_str("source", facts.name.as_str());
    properties.add_nvlist(DESIRED_STATE_PROPERTY, pair);
    return a_dataset_with(
        crate::worktree_clone_mount_handler::sweep_dataset_facts::CloneObjectFacts {
            properties,
            ..facts
        },
    );
}

/// One clone that asks for nothing.
fn a_clone_without_a_property() -> DatasetFacts {
    return a_dataset_with(DatasetBuilder::clone().build());
}

/// One dataset whose reading holds the facts the control built.
fn a_dataset_with(
    facts: crate::worktree_clone_mount_handler::sweep_dataset_facts::CloneObjectFacts
) -> DatasetFacts {
    return DatasetFacts {
        role: DatasetRole::Clone,
        name: facts.name.clone(),
        reading: Reading::Read(facts),
    };
}
