//! What:
//!  The notices a commit's preparation prints on standard error,
//! given to the standard error descriptor under a ceiling.
//! Why:
//!  These lines are printed on the commit's own path,
//! before its request reaches the service,
//! and a standard error whose
//!  reader has stopped draining waits in the kernel with no end.
//!  The owner's rule is "nothing should ever wait forever or indefinitely",
//! and no notice is worth a
//!  commit that never returns,
//! so a line is given to a thread of its own and awaited no longer than
//!  `WARNING_WRITE_CEILING`;
//! what cannot be said in that time is left unsaid and logged.
//!  The write goes to the descriptor itself and not through the standard library's shared handle,
//! whose lock every
//!  synchronous write takes,
//! so a worker left at the ceiling holds nothing another writer would wait for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! deliverNotice(`${note}\n`); // written to descriptor 2 within the ceiling, or dropped
//! ```

/// Debug logging of a line that could not be said.
use crate::diagnostic_log::debug;
/// The process's standard error descriptor,
/// written without the standard library's shared lock.
use crate::standard_error_descriptor::StandardErrorDescriptor;
/// The longest such a line may take to reach the person.
use crate::wait_ceilings::WARNING_WRITE_CEILING;
/// The line,
/// given to the descriptor on a thread of its own.
use crate::work_awaited_within_a_ceiling::write_within;

/// What:
///  Give one notice to standard error within the warning ceiling,
/// without taking the standard library's shared lock.
///  `bool` is whether it reached the person.
/// Why:
///  These notices are printed on the commit's own path,
/// and a standard error whose reader has stopped draining waits in the
///  kernel with no end;
/// the owner's rule is "nothing should ever wait forever or indefinitely",
/// and the commit's
///  result is never one of these lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deliverNotice(`${note}\n`);
/// ```
pub(crate) fn deliver_notice(text: String) -> bool {
    let delivered: Option<std::io::Result<()>> = write_within(WARNING_WRITE_CEILING, StandardErrorDescriptor, text);
    return match delivered {
        Some(Ok(())) => true,
        Some(Err(error)) => {
            debug("prepare", format!("the notice could not be written: {error}").as_str());
            false
        }
        None => {
            debug("prepare", format!("the notice was not written within {WARNING_WRITE_CEILING:?}").as_str());
            false
        }
    };
}

/// `deliver_notice` for a control that asks it directly.
#[cfg(test)]
pub(crate) fn deliver_notice_for_test(text: &str) -> bool {
    return deliver_notice(text.to_string());
}
