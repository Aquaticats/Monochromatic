//! What:
//!  Whether the live snapshot is pinned or marked,
//!  which is a block before any request:
//!  its clone count,
//!  its hold count,
//!  and the mark another writer left on it.
//! Why:
//!  The counts report the block before a request that would be refused anyway,
//!  and a mark is another writer's,
//!  which the wrapper never clears
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "What ends it").
//!  The counts and the block they write sit together,
//!  apart from the attempt's order,
//!  so a reader sees every reason a snapshot is not destroyed here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const block = theCounts(record, attemptedAt, identity); // undefined when free
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_record::{
    CleanupBlock,
    OriginCleanupRecord,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_words::BlockReason;
use crate::worktree_removal_deferred_origin_check::origin_mark::{
    OriginMark,
    mark_of_reply,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity;

/// What:
///  Whether the live snapshot is pinned or marked,
///  which is a block before any request.
/// Why:
///  The counts report the block before a request that would be refused anyway,
///  and a mark is another writer's,
///  which the wrapper never clears.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const block = theCounts(record, attemptedAt, identity); // undefined when free
/// ```
pub(super) fn the_counts(
    record: &OriginCleanupRecord,
    attempted_at: &str,
    identity: &DatasetIdentity,
) -> Option<(OriginCleanupRecord, String)> {
    let clones: u64 = identity.stats.dds_num_clones;
    let mark: OriginMark = mark_of_reply(identity);
    let holds: u64 = match &mark {
        OriginMark::Marked {
            holds,
            ..
        } => match holds {
            crate::worktree_removal_deferred_origin_check::origin_mark::UserHolds::Counted(
                count,
            ) => *count,
            crate::worktree_removal_deferred_origin_check::origin_mark::UserHolds::Unreadable {
                said,
            } => {
                let block: CleanupBlock = CleanupBlock {
                    reason: BlockReason::Unproved,
                    errno: 0,
                    clones,
                    holds: 0,
                };
                return Some((
                    record.blocked(block, attempted_at),
                    format!("the snapshot's hold count could not be read ({said})"),
                ));
            }
        },
        _ => 0,
    };
    if clones != 0 || holds != 0 {
        let block: CleanupBlock = CleanupBlock {
            reason: BlockReason::Dependency,
            errno: 0,
            clones,
            holds,
        };
        return Some((
            record.blocked(block, attempted_at),
            format!("{clones} clone(s) and {holds} hold(s) pin the snapshot"),
        ));
    }
    if let OriginMark::Marked {
        ..
    } = mark
    {
        let block: CleanupBlock = CleanupBlock {
            reason: BlockReason::Deferred,
            errno: 0,
            clones,
            holds,
        };
        return Some((
            record.blocked(block, attempted_at),
            String::from("the snapshot is marked for deferred destroy"),
        ));
    }
    if let OriginMark::Unreadable {
        said,
    } = mark
    {
        let block: CleanupBlock = CleanupBlock {
            reason: BlockReason::Unproved,
            errno: 0,
            clones,
            holds,
        };
        return Some((
            record.blocked(block, attempted_at),
            format!("the mark could not be read ({said})"),
        ));
    }
    return None;
}
