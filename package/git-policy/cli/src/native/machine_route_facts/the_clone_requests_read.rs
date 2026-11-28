//! What:
//!  Every managed clone's durable mount request and the receipt that answers
//!  it.
//! Why:
//!  A clone asks the handler to mount or unmount it through one local property,
//!  and the handler answers in a receipt filed under the clone's own GUID,
//!  so a request with no matching receipt of this boot is the one work item a
//!  person can act on
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//!  results and ordering";
//!  plan step 18,
//!  "the latest readiness and unresolved requests").
//!  The request's own reading keeps three answers apart:
//!  a clone that asks for nothing,
//!  one whose property an ancestor set,
//!  and one whose property is words the grammar refuses.
//!  The last proves nothing about the clone's request,
//!  so it answers unknown rather than "no request",
//!  and the receipt's own refusal is the same.
//!  Nothing here writes a receipt or a property.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const requests = theRequestFacts({ datasets, receipts, unread }); // [{ clone, intentNonce, receiptNonce, receiptBoot }]
//! ```

/// The unknown a read that failed answers.
use super::the_unknown_a_failed_read_answers::the_unknown_of;
/// One check's stable name,
/// and the report line a failed read answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// One clone's request and the receipt that answers it.
use crate::doctor_zfs_route::handler_checks_unresolved_requests::RequestFacts;
/// One dataset's role and what its read answered.
use crate::doctor_zfs_route::zfs_checks_dataset_roles::{
    DatasetFacts,
    DatasetRole,
};
/// What a read of one dataset answered.
use crate::doctor_zfs_route::zfs_checks_reading::Reading;
/// The folder the handler files its receipts in,
/// and the GUID a receipt is filed under.
use crate::worktree_clone_mount_protocol::{
    DatasetGuid,
    ReceiptsFolder,
};
/// The property a clone asks through and the three answers its read gives.
use crate::worktree_clone_mount_protocol::{
    IntentReading,
    read_intent,
};

/// What:
///  The one check the requests feed.
/// Why:
///  A clone's request is judged only by the unresolved-requests check,
///  so a request that could not be read answers that check's unknown and no
///  other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_CHECK_OF_THE_REQUESTS = 'unresolved-requests';
/// ```
const THE_CHECK_OF_THE_REQUESTS: Check = Check::UnresolvedRequests;

/// What:
///  Every clone's request and the receipt that answers it,
///  where either is there.
/// Why:
///  The check compares the nonce a clone asks under against the nonce the
///  newest receipt answers,
///  and a receipt of another boot answers nothing here,
///  so the boot the receipt was filed in travels with it.
///  A property or a receipt this run could not read leaves the check unable to
///  say whether the clone was answered,
///  so it answers unknown naming the clone rather than reporting the clone as
///  unresolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const requests = theRequestFacts({ datasets, receipts, unread });
/// ```
pub(crate) fn the_request_facts(
    datasets: &[DatasetFacts],
    receipts: &ReceiptsFolder,
    unread: &mut Vec<CheckReport>,
) -> Vec<RequestFacts> {
    let mut requests: Vec<RequestFacts> = Vec::new();
    for dataset in datasets {
        if !matches!(dataset.role, DatasetRole::Clone) {
            continue;
        }
        let Reading::Read(facts) = &dataset.reading else {
            continue;
        };
        let intent_nonce: Option<String> = match read_intent(
            Some(&facts.properties),
            facts.name.as_str(),
        ) {
            IntentReading::Request(intent) => Some(String::from(intent.nonce.as_str())),
            IntentReading::Absent
            | IntentReading::Inherited {
                ..
            } => None,
            IntentReading::Refused(refusal) => {
                unread.push(the_unknown_of(
                        THE_CHECK_OF_THE_REQUESTS,
                        format!(
                            "`{}` asks for work through a property this route refuses ({}), so whether a receipt answers it is unproved",
                            facts.name,
                            refusal.message()
                        )
                        .as_str(),
                    ));
                None
            }
        };
        let answered: Option<(String, String)> =
            receipt_of(receipts, facts.guid, unread, facts.name.as_str());
        requests.push(RequestFacts {
            clone: String::from(facts.name.as_str()),
            intent_nonce,
            receipt_nonce: answered.as_ref().map(|pair| return pair.0.clone()),
            receipt_boot: answered.map(|pair| return pair.1),
        });
    }
    return requests;
}

/// What:
///  The newest receipt filed for one clone's GUID,
///  as the nonce it answers and the boot it was filed in.
/// Why:
///  The check compares exactly those two values against a clone's request,
///  so the receipt's other fields are read by no line of the report.
///  A receipt that could not be read makes the answer unproved,
///  and the clone is named so two clones' failures are told apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = receiptOf(receipts, guid, unread, name); // ['0123…', 'a0b1…'] | undefined
/// ```
fn receipt_of(
    receipts: &ReceiptsFolder,
    the_guid: u64,
    unread: &mut Vec<CheckReport>,
    clone: &str,
) -> Option<(String, String)> {
    // What:     `DatasetGuid::parse` reads the kernel's number back as the canonical GUID a receipt is filed
    //           under, and a number the grammar refuses is no name this folder holds.
    // Why:      The receipt's file name is that GUID's own text, so the two must be built the same way.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const held = DatasetGuid.parse(String(guid));
    // ```
    let held: DatasetGuid = match DatasetGuid::parse(the_guid.to_string().as_str()) {
        Ok(named) => named,
        Err(refusal) => {
            unread.push(the_unknown_of(
                THE_CHECK_OF_THE_REQUESTS,
                format!(
                    "`{clone}` reports a GUID this route cannot name ({refusal:?}), so its receipt cannot be found"
                )
                .as_str(),
            ));
            return None;
        }
    };
    return match receipts.read(held) {
        Ok(Some(receipt)) => Some((String::from(receipt.nonce.as_str()), receipt.boot)),
        Ok(None) => None,
        Err(refusal) => {
            unread.push(the_unknown_of(
                THE_CHECK_OF_THE_REQUESTS,
                format!(
                    "`{clone}`'s receipt could not be read ({}), so whether it was answered is unproved",
                    refusal.message()
                )
                .as_str(),
            ));
            None
        }
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_clone_requests_read_tests.rs"]
mod tests;
