//! What:
//!  The reversing request the cancellation files,
//!  opened from the slot's own record when the command first asks for one.
//! Why:
//!  The release answer needs the clone's dataset and its recorded GUID
//!  (`worktree_clone_removal_cancel::the_reversing_request`),
//!  and the command learns them from the record it proves,
//!  which is a read the grammar cannot make before the command runs.
//!  Reading the record here is sound because the command holds the slot's claim
//!  from before the act to after it,
//!  and the add takes the same claim
//!  (`worktree_clone_removal_cancel::the_removal_claim`),
//!  so no add or removal of that slot can rewrite the record in between.
//!  A read that refused or a clone the record does not name is the command's own
//!  refusal for a request it could not write,
//!  and it is taken before the property is written,
//!  so a refusal leaves the clone as it stood.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const writer = new TheMountRequestFiledWhenAsked(commonDir, 'one');
//! writer.requestTheMount(intent); // reads `one.binding`, then LiveTheMountRequest.open(...)
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::the_reversing_request::{
    LiveTheMountRequest,
    MountAnswer,
    TheMountRequest,
};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::{
    RecordedBinding,
    read_the_recorded_binding,
};
use std::cell::OnceCell;
use std::path::Path;

/// What:
///  One clone's request writer,
///  made from the slot's record at the first request.
/// Why:
///  The writer names the clone and its GUID,
///  and the record is where both are proved,
///  so the grammar hands in the slot and the writer reads what the command
///  already proved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheWriter = { commonDir: string; slot: string; writer: OnceCell<Result<LiveTheMountRequest, string>> };
/// ```
pub(crate) struct TheMountRequestFiledWhenAsked<'a> {
    /// The common Git directory whose record home holds the slot's binding.
    common_dir: &'a Path,
    /// The slot the command proved.
    slot: &'a str,
    /// The release writer,
    /// opened once.
    writer: OnceCell<Result<LiveTheMountRequest, String>>,
}

impl<'a> TheMountRequestFiledWhenAsked<'a> {
    /// A writer for one slot whose record has not been read yet.
    pub(crate) fn for_the_slot(
        common_dir: &'a Path,
        slot: &'a str,
    ) -> TheMountRequestFiledWhenAsked<'a> {
        return TheMountRequestFiledWhenAsked {
            common_dir,
            slot,
            writer: OnceCell::new(),
        };
    }

    /// The release writer for the clone this slot's record names.
    fn the_writer(&self) -> Result<&LiveTheMountRequest, String> {
        return self
            .writer
            .get_or_init(|| {
                let recorded: RecordedBinding =
                    read_the_recorded_binding(self.common_dir, self.slot)
                        .map_err(|stop| return stop.message())?;
                return LiveTheMountRequest::open(
                    recorded.clone_dataset.as_str(),
                    recorded.clone_guid,
                )
                .map_err(|refusal| return refusal.message());
            })
            .as_ref()
            .map_err(|said| return said.clone());
    }
}

impl TheMountRequest for TheMountRequestFiledWhenAsked<'_> {
    fn request_the_mount(
        &self,
        intent: &MountIntent,
    ) -> Result<MountAnswer, CancelRefusal> {
        let writer: &LiveTheMountRequest = self.the_writer().map_err(|said| {
            return CancelRefusal::TheMountRequestCouldNotBeWritten {
                said,
            };
        })?;
        return writer.request_the_mount(intent);
    }
}
