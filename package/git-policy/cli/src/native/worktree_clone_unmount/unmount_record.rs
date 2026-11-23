//! What:
//!  The record a removal writes before its first bind,
//!  and the shape checks the staging folder must pass first.
//! Why:
//!  The design writes the record before the bind and records a folder owned by
//!  the pinned account,
//! at the recorded mode,
//! and empty
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The staging target and who creates it").
//!  A record that describes a folder the account replaced refuses the bind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! recordTheBind(request, staging, name); // checks owner, mode and emptiness, then writes
//! ```

use super::staged_folder_identity::StagedFolderIdentity;
use super::staged_folder_record::{write_record, StagedFolderRecord};
use super::staged_record_reading::{read_staged_record, BindingDecision, RecordReading};
use super::unmount_turn::{call_refused, refused, UnmountOutcome, UnmountRequest};
use std::os::unix::fs::MetadataExt;

#[allow(clippy::result_large_err, reason = "the outcome carries the mount row and the step a person acts on")]
pub(super) fn record_the_bind(request: &UnmountRequest<'_>, staging: &std::fs::File, name: &str) -> Result<(), UnmountOutcome> {
    let folder_facts: std::fs::Metadata = match staging.metadata() {
        Ok(facts) => facts,
        Err(error) => return Err(refused("staging-folder", format!("the staging folder {name} could not be read: {error}"))),
    };
    let cache_facts: std::fs::Metadata = match request.cache.metadata() {
        Ok(facts) => facts,
        Err(error) => return Err(refused("staging-folder", format!("the cache folder could not be read: {error}"))),
    };
    let entry_count: usize = match crate::entries_of_one_open_folder::entry_names(staging) {
        Ok(names) => names.len(),
        Err(error) => return Err(refused("staging-folder", format!("the staging folder {name} could not be listed: {error}"))),
    };
    if folder_facts.uid() != cache_facts.uid() {
        return Err(refused("staging-folder", format!("the staging folder {name} is owned by {}, and the account's cache by {}", folder_facts.uid(), cache_facts.uid())));
    }
    if folder_facts.mode() & 0o7777 != 0o700 {
        return Err(refused("staging-folder", format!("the staging folder {name} has mode {:04o}, and the route records 0700", folder_facts.mode() & 0o7777)));
    }
    if entry_count != 0 {
        return Err(refused("staging-folder", format!("the staging folder {name} holds {entry_count} entries, and a staging folder is empty before its bind")));
    }
    let reading: RecordReading = read_staged_record(request.cache, request.expected.guid, request.boot, request.cache_path.join(name).as_path(), request.calls);
    let identity: StagedFolderIdentity = match StagedFolderIdentity::of(staging) {
        Ok(identity) => identity,
        Err(refusal) => return Err(refused("staged-record", refusal.message())),
    };
    if let RecordReading::Current { record, .. } = &reading {
        let moved = identity.differences(&record.identity);
        if !moved.is_empty() {
            return Err(refused(
                "staged-record",
                format!("the folder the record describes differs from the folder this invocation opened in the {}, so the record is kept and the bind refused", StagedFolderIdentity::field_names(moved.as_slice())),
            ));
        }
    }
    if BindingDecision::of(&reading) == BindingDecision::Refuse {
        let said: String = match reading {
            RecordReading::Refused { step, said } => format!("the staged record refused at its {step} step: {said}"),
            _ => String::from("the staged record refused"),
        };
        return Err(refused("staged-record", said));
    }
    let (handle_type, handle): (i32, Vec<u8>) = match request.calls.name_handle(staging) {
        Ok(handle) => handle,
        Err(refusal) => return Err(call_refused("name-handle", refusal, None)),
    };
    let (namespace_device, namespace_inode): (u64, u64) = match request.calls.namespace_pair() {
        Ok(pair) => pair,
        Err(refusal) => return Err(call_refused("namespace", refusal, None)),
    };
    let record: StagedFolderRecord = StagedFolderRecord {
        component: String::from(name),
        dataset: request.expected.source.clone(),
        guid: request.expected.guid,
        identity,
        handle_type,
        handle,
        nonce: String::from(request.nonce),
        namespace_device,
        namespace_inode,
        boot: String::from(request.boot),
    };
    return write_record(request.cache, &record).map_err(|refusal| return refused("staged-record", refusal.message()));
}
