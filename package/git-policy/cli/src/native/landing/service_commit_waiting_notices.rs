//! What:
//!  One submission of a prepared commit whose waiting notices are written away from this command's thread.
//! Why:
//!  The thread that submits is the only one that reads the service's answer,
//! withdraws a request that reached its ceiling and
//!  keeps the commit's scratch directory fresh,
//! so a terminal whose reader stopped draining must never hold it:
//! the lines go to
//!  `foundation/notice_stream_that_does_not_hold_its_writer.rs`,
//! which drops a line the writer still holds and gives the last one
//!  `WARNING_WRITE_CEILING` to reach the reader.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await submitSayingWhereItStands(config, request, trace) // notices on a thread of their own
//! ```

use crate::commit_timing::SubmitTrace;
use crate::notice_stream_that_does_not_hold_its_writer::NoticeStreamThatDoesNotHoldItsWriter;
use crate::scratch_location::SCRATCH_REFRESH_EVERY;
use crate::service_client::{
    ClientConfig,
    FreshScratch,
    Submitted,
    submit_traced,
};
use crate::service_messages::ClientFailure;
use crate::service_protocol::CommitRequest;
use std::path::Path;

/// What:
///  Submits `request` as `submit_traced` does,
/// keeping `scratch` fresh while it waits and having its notices written by a thread of their own.
/// Why:
///   The two bounds a long wait needs are chosen in one place,
/// so every command that submits a prepared commit has
///  both and none writes a waiting line in the thread that waits for the answer:
/// the scratch directory outlives a wait that
///  outlasts its age,
/// and a terminal that stopped draining never holds the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = new NoticeStreamThatDoesNotHoldItsWriter(stderr); try { return submitTraced({...config, keepFresh: scratch}, request, notices, trace) } finally { notices.finish() }
/// ```
pub(crate) fn submit_saying_where_it_stands(
    config: &ClientConfig,
    scratch: &Path,
    request: &CommitRequest,
    trace: &mut SubmitTrace,
) -> Result<Submitted, ClientFailure> {
    let waiting_config: ClientConfig = ClientConfig {
        keep_fresh: Some(FreshScratch {
            path: scratch.to_path_buf(),
            every: SCRATCH_REFRESH_EVERY,
        }),
        ..config.clone()
    };
    // What: The waiting lines go to this process's standard error descriptor itself.
    // Why:  A reader that stopped draining holds the writer thread inside its write, and the shared
    //       handle's lock is one for the whole process: a line waiting there would make every later
    //       write of this command wait behind it with no end (`standard_error_descriptor.rs`).
    let mut notices: NoticeStreamThatDoesNotHoldItsWriter =
        NoticeStreamThatDoesNotHoldItsWriter::start(Box::new(
            crate::standard_error_descriptor::StandardErrorDescriptor,
        ));
    let answered: Result<Submitted, ClientFailure> =
        submit_traced(&waiting_config, request, &mut notices, trace);
    notices.finish();
    return answered;
}
