//! What:
//!  The commit scratch trees the requests this service holds still name.
//! Why:
//!  A request that a killed client left queued lands on the tree it made,
//!  and that landing reads the private index and the conclusion copies in it
//!  (`service_server.rs`,
//! `remove_the_trees`).
//!  A collection removes a commit tree nothing has written for `SCRATCH_TREE_ABANDONED_AFTER`,
//!  because the age of an entry is the only evidence a collection has
//!  (`service_scratch_abandonment.rs`),
//!  and a client that is gone renews nothing.
//!  So a request that waited longer than that age would lose the tree it lands on
//!  to the collection a landing starts
//!  (`service_repository_scratch_sweep.rs`),
//!  and the cleanup that follows its branch move would find neither the tree nor a copy of it
//!  (`remove_conclusion_state`,
//! `service_land_follow.rs`),
//!  leaving the real conclusion state in place for the next commit to conclude the same merge again.
//!  The service therefore writes down every tree a request it holds named,
//!  and the one decision that removes a commit tree spares them.
//!  A tree goes back to the age rule as soon as the service is done with the request that named it:
//!  the landing answered it,
//!  the service handed it back to its client,
//!  or its client withdrew it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = new Map<string, string>(); // request identity -> the name of the tree it named
//! note(id, tree); release(id); isHeld(tree);
//! ```

/// One name of a tree,
/// and the paths a request and a collection give it.
use std::ffi::OsString;
use std::path::Path;
/// What keeps the record of one process's requests to itself.
use std::sync::Mutex;

/// Lock kept:
///  the trees of this service's queued and in-flight requests,
///  each as the identity of the request that named it and the name of that tree:
/// the sections are one removal and one addition of an entry,
/// and nothing is read from or written to the device under the lock,
/// so a connection that queues a request and a landing that answers one never wait on storage,
/// and a collection reads the record while both keep serving.
static HELD: Mutex<Vec<(String, OsString)>> = Mutex::new(Vec::new());

/// What:
///  Records the tree the request `id` named,
///  in place of any tree an earlier send of the same request named.
/// Why:
///   A command whose answer was lost prepares again,
///  and the attempt it sends next names a tree of its own:
///  the attempt it left is its own to release,
///  and keeping both would keep a tree no request lands on.
///  A path with no file name in it names no tree of a scratch place and is not recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// note(id, tree); // void
/// ```
pub fn note(id: &str, tree: &Path) {
    let Some(name) = tree.file_name() else { return };
    let mut held = HELD.lock().expect("the trees the queued requests hold");
    held.retain(|(named, _)| { return named.as_str() != id; });
    held.push((String::from(id), name.to_os_string()));
}

/// What:
///  Forgets every tree the request `id` named.
/// Why:
///   A request the service is done with is one no landing reads,
///  so whatever is left of its tree is what the age rule collects,
///  as it collects the tree of a command that was killed before it submitted one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// release(id); // void
/// ```
pub fn release(id: &str) {
    HELD.lock().expect("the trees the queued requests hold").retain(|(named, _)| { return named.as_str() != id; });
}

/// What:
///  Whether a request this service holds named the tree at `path`.
/// Why:
///   A collection asks this of every entry of a scratch place,
///  so the answer is one comparison against the requests in flight,
///  never a question to the device.
///  The name of a tree carries the identity of the command that made it and the number of its attempt,
///  so no two trees of one host share it,
///  and a tree is spared by its name alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isHeld(path); // whether a request this service holds still lands on the tree
/// ```
pub fn is_held(path: &Path) -> bool {
    let Some(name) = path.file_name() else { return false };
    return HELD.lock().expect("the trees the queued requests hold").iter().any(|(_, held)| { return held.as_os_str() == name; });
}

/// The record and its two ends are controlled in the test module.
#[cfg(test)]
#[path = "service_scratch_trees_of_queued_requests_tests.rs"]
mod tests;
