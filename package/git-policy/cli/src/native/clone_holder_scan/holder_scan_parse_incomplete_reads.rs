//! What:
//!  One read the scan could not complete,
//!  read back from the line that names its cause:
//!  the namespace or the thread it was about,
//!  how many tasks or namespaces were tried,
//!  the bound a table passed,
//!  and the reason the reader gives.
//! Why:
//!  Each cause writes its own line with its own words,
//!  so each is read by the word it starts with,
//!  and a line that does not fit its cause refuses the whole result;
//!  it lives apart from the whole result so each file says one thing
//!  (`crate::clone_holder_scan::holder_scan_parse`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const read = incompleteOf('task-ended some-namespace 4242 the task ended');
//! ```

/// The canonical decimal count a bound is written as.
use crate::clone_holder_scan::holder_scan_parse_cursor::decimal_count;
/// Why a line that is not the field the reader expected refuses the whole result.
use crate::clone_holder_scan::holder_scan_refusals::HolderScanRefusal;
/// The writer's own escaping for a path and for a sentence,
/// and the key an incomplete read's line begins with.
use crate::clone_holder_scan::holder_scan_text::{
    INCOMPLETE_KEY,
    unescape_path,
    unescape_text,
};
/// Every cause a read can name,
/// with the word each writes first.
use crate::clone_holder_scan::incomplete_reads::{
    DETAILS_PAST_THE_BOUND_WORD,
    IncompleteRead,
    LISTING_WORD,
    MOUNTS_PAST_THE_BOUND_WORD,
    NAMESPACE_LINK_UNREADABLE_WORD,
    NAMESPACES_PAST_THE_BOUND_WORD,
    NO_LIVE_MEMBER_WORD,
    NO_TASK_AT_THE_NAMESPACE_ROOT_WORD,
    TABLE_UNREADABLE_WORD,
    TASK_ENDED_WORD,
};
/// The refusal one incomplete read's line makes when it does not fit its
/// cause.
fn incomplete_refusal(value: &str) -> HolderScanRefusal {
    return HolderScanRefusal::Value {
        key: INCOMPLETE_KEY,
        shown: String::from(value),
    };
}

/// One incomplete read,
/// read by the cause its own word names.
pub(crate) fn incomplete_of(value: &str) -> Result<IncompleteRead, HolderScanRefusal> {
    let Some((kind, rest)) = value.split_once(' ') else {
        return Err(incomplete_refusal(value));
    };
    return match kind {
        LISTING_WORD => {
            let Some((proc_root, reason)) = rest.split_once(' ') else {
                return Err(incomplete_refusal(value));
            };
            Ok(IncompleteRead::Listing {
                proc_root: unescape_path(proc_root),
                reason: String::from(reason),
            })
        }
        NAMESPACE_LINK_UNREADABLE_WORD => {
            let Some((thread, reason)) = rest.split_once(' ') else {
                return Err(incomplete_refusal(value));
            };
            Ok(IncompleteRead::NamespaceLinkUnreadable {
                thread: unescape_path(thread),
                reason: String::from(reason),
            })
        }
        TASK_ENDED_WORD | TABLE_UNREADABLE_WORD => {
            let parts: Vec<&str> = rest.splitn(3, ' ').collect();
            let [namespace, task, reason] = parts.as_slice() else {
                return Err(incomplete_refusal(value));
            };
            let (namespace_text, task_path, reason_text) = (
                unescape_text(namespace),
                unescape_path(task),
                String::from(*reason),
            );
            if kind == TASK_ENDED_WORD {
                return Ok(IncompleteRead::TaskEnded {
                    namespace: namespace_text,
                    task: task_path,
                    reason: reason_text,
                });
            }
            return Ok(IncompleteRead::TableUnreadable {
                namespace: namespace_text,
                task: task_path,
                reason: reason_text,
            });
        }
        NO_LIVE_MEMBER_WORD => {
            let parts: Vec<&str> = rest.splitn(3, ' ').collect();
            let [namespace, tasks_tried, reason] = parts.as_slice() else {
                return Err(incomplete_refusal(value));
            };
            let Ok(tried) = tasks_tried.parse::<usize>() else {
                return Err(incomplete_refusal(value));
            };
            Ok(IncompleteRead::NoLiveMember {
                namespace: unescape_text(namespace),
                tasks_tried: tried,
                reason: String::from(*reason),
            })
        }
        NO_TASK_AT_THE_NAMESPACE_ROOT_WORD => Ok(IncompleteRead::NoTaskAtTheNamespaceRoot {
            namespace: unescape_text(rest),
        }),
        NAMESPACES_PAST_THE_BOUND_WORD => Ok(IncompleteRead::NamespacesPastTheBound {
            bound: decimal_count(INCOMPLETE_KEY, rest)?,
        }),
        DETAILS_PAST_THE_BOUND_WORD => Ok(IncompleteRead::DetailsPastTheBound {
            bound: decimal_count(INCOMPLETE_KEY, rest)?,
        }),
        MOUNTS_PAST_THE_BOUND_WORD => {
            let Some((namespace, bound)) = rest.split_once(' ') else {
                return Err(incomplete_refusal(value));
            };
            Ok(IncompleteRead::MountsPastTheBound {
                namespace: unescape_text(namespace),
                bound: decimal_count(INCOMPLETE_KEY, bound)?,
            })
        }
        _ => Err(incomplete_refusal(value)),
    };
}
