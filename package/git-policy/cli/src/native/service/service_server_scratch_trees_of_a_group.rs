//! What:
//!  The scratch trees one group's requests name,
//!  and the removal of those trees once the landing that read them is over.
//! Why:
//!   Each request names the commit's own scratch directory,
//!   which nothing may remove before the landing that reads it,
//!   and a request the service handed a finished shadow names that shadow too:
//!   a tree of the wrapper's own in this worktree's drawn folder,
//!   whose end is decided by the landing that read it and nowhere else.
//!  A tree is removed beside the loop once its landing is over,
//!  except the one a request the service did not land still keeps:
//!  a client that reconnects sends the same request again and reads that tree,
//!  so removing it here would take the read from under every rebuild
//!  (2026-11-09,
//!  `service_prepare.rs`,
//!  `retry_of`).
//!  The service is done with what it queued of every request of the group here,
//!  so no tree a send of this group named is held against the collection any more
//!  (`service_scratch_trees_of_queued_requests.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! removeTheTrees(theTreesAGroupNames(group), handedBack)
//! ```
use crate::service_server_queue_types_and_send::Queued;

/// What:
///  The trees the requests of one group name:
///  the commit's own scratch directory,
///  and the finished shadow the service handed that request.
/// Why:
///   The landing that reads a tree is the only place that says when the commit is done with it,
///   so the group's trees are read before the landing and removed after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// queued.map((q) => [q.id, scratchTreeOf(q) ?? drawnTreeOf(q)]).filter(Boolean)
/// ```
pub(super) fn the_trees_a_group_names(
    queued_group: &[Queued]
) -> Vec<(String, std::path::PathBuf)> {
    // The trees this group's requests name: each is the commit's own scratch directory, which nothing may remove before the
    //  landing that reads it, and which the service removes once that landing is over.
    //  A request the service did not land keeps its tree in its client's hands instead (see `remove_the_trees`).
    let mut trees: Vec<(String, std::path::PathBuf)> = Vec::new();
    for queued in queued_group {
        if let Some(tree) = crate::service_scratch_release_removal::the_tree_of(
            queued.0.scratch.as_path(),
            queued.0.git_dir.as_path(),
        ) {
            trees.push((queued.0.id.clone(), tree));
        }
        // The finished shadow the service drew ahead of this commit, when it was handed one: it is a tree of the
        //  wrapper's own in this worktree's drawn folder, and the landing that read it is the only place that says
        //  when the commit is done with it.
        if let Some(handed) = queued.0.shadow.as_deref()
            && crate::shadow_drawn::is_a_drawn_tree(
                handed,
                queued.0.common_dir.as_path(),
                queued.0.git_dir.as_path(),
            )
        {
            trees.push((queued.0.id.clone(), handed.to_path_buf()));
        }
    }
    // What: the trees of the group, each with the identity of the request that named it.
    // Why:  the removal answers the collection by identity, and leaves the handed-back trees to their clients.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return trees;
    // ```
    return trees;
}

/// What:
///  Starts the removal of the scratch trees of the requests a group has just finished with,
///  beside the loop,
///  except those the service handed back to their clients.
/// Why:
///   The tree holds the private index and the conclusion copies a landing reads,
///   so only the landing that named it may decide that it is done with it.
///  A tree whose client is gone is left where it is rather than removed with its answer:
///  a client that reconnects sends the same request again and reads it there,
///  and a request with no client left to make its commit gives its tree to the sweep by age.
///  A request the service did not land (a commit sent back to be prepared again,
/// a branch that moved) has no landing
///  that read its tree:
/// its client reads the change it prepared from that tree to make the commit again,
/// so the client
///  releases it beside its own command.
///  Removing it here took that read from under every rebuild,
/// which failed with a missing private index
///  (2026-11-09,
///  `service_prepare.rs`,
///  `retry_of`).
///  The service is done with what it queued of every request of the group here,
///  so no tree a send of this group named is held against the collection any more
///  (`service_scratch_trees_of_queued_requests.rs`):
/// a tree the client did not take back goes to the age rule once its removal is over.
///  A release gives up one send of the request,
///  never the identity:
///  `land_group` answered this group inside its own call,
///  so the same request can already be queued again with the next attempt's tree,
///  and that send keeps its tree held beyond this release.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// trees.filter(([id]) => !handedBack.has(id)).forEach(startTheRemovalBesideTheLoop);
/// ```
pub(super) fn remove_the_trees(
    trees: &[(String, std::path::PathBuf)],
    handed_back: &std::collections::HashSet<String>,
) {
    for (id, _) in trees {
        crate::service_scratch_trees_of_queued_requests::release(id.as_str());
    }
    for tree in released_trees(trees, handed_back) {
        crate::service_scratch_release_removal::start(tree.clone());
    }
}

/// What:
///  The trees of a group's requests that no client still reads.
/// Why:
///   A request the service handed back still reads the tree it named,
///  so that one is the client's to remove;
///  every other answer ends the landing that read the tree,
///  and the service removes it beside the loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// releasedTrees(trees, handedBack) // the trees no client still reads
/// ```
pub(super) fn released_trees<'a>(
    trees: &'a [(String, std::path::PathBuf)],
    handed_back: &std::collections::HashSet<String>,
) -> Vec<&'a std::path::PathBuf> {
    let mut released: Vec<&std::path::PathBuf> = Vec::new();
    for (id, tree) in trees {
        if handed_back.contains(id) {
            continue;
        }
        released.push(tree);
    }
    return released;
}
