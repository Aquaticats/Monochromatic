//! What:
//!  The commits of one attempt's batch,
//!  the commit its branch is about to name,
//!  the reflog row of that move,
//!  and the record the journal holds of it:
//!  everything a landing writes between its reads and the index lock.
//! Why:
//!  These four are made in one stretch and read together by every step after it,
//!  so they stand in one record that the steps below take
//!  (`service_land_attempt_owed_and_witness.rs` and the three beside it),
//!  and the seam a control plants after the objects and after the record
//!  stays in the order it was made in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const built: WhatTheBatchBuilt = buildTheBatch(the);
//! ```

use crate::service_fault::reach;
use crate::service_journal::Intent;
use crate::service_land_attempt_values::TheAttempt;
use crate::service_land_objects_of_batch::objects_of_batch;
use crate::service_land_record_the_batch::{
    TheBatch,
    of,
};
use crate::service_protocol::FailureCode;
use std::time::Instant;

/// What one attempt's batch came to,
/// once its objects exist and its record is in the journal.
pub(crate) struct WhatTheBatchBuilt {
    /// The commits the batch holds,
    /// in the order their requests are made.
    pub(crate) commits: Vec<String>,
    /// The commit the branch is about to name.
    pub(crate) new: String,
    /// The row the branch move writes in the reflog.
    pub(crate) reflog: std::ffi::OsString,
    /// The record the journal holds,
    /// which a restart reads to decide the batch from the branch.
    pub(crate) intent: Intent,
}

/// What:
///  Makes the batch's commits,
///  writes its record,
///  and answers with what the steps after it read.
/// Why:
///  Nothing before this point is written to the journal,
///  and nothing after it is made without the record that a restart needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function buildTheBatch(the: TheAttempt): WhatTheBatchBuilt
/// ```
///
/// # Errors
///
/// `Unavailable` when an object the batch names is gone,
/// and the failure of writing the journal's record.
pub(crate) fn build_the_batch(the: &mut TheAttempt<'_>) -> Result<WhatTheBatchBuilt, FailureCode> {
    let objects_began: Instant = Instant::now();
    let (commits, new): (Vec<String>, String) = objects_of_batch(
        &the.git,
        the.context.journal.as_path(),
        the.seq,
        the.first,
        the.scratch.as_str(),
        the.old.as_str(),
        the.batch.as_slice(),
        the.amended_parents.as_slice(),
        the.settings,
    )?;
    the.timing.objects += objects_began.elapsed();
    reach("after-objects");
    if let Some(seam) = &the.context.on_objects_written {
        seam();
    }
    let (intent, reflog): (Intent, std::ffi::OsString) = of(TheBatch {
        context: the.context,
        timing: &mut *the.timing,
        seq: the.seq,
        first: the.first,
        batch: the.batch.as_slice(),
        commits: commits.as_slice(),
        old: the.old.as_str(),
        new: new.as_str(),
        unborn: the.unborn,
    })?;
    reach("after-intent");
    return Ok(WhatTheBatchBuilt {
        commits,
        new,
        reflog,
        intent,
    });
}
