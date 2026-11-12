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
///   A command told to prepare again sends the same identity with a new attempt's tree,
///  and that answer ended the landing of the attempt it left,
///  which gave that tree back
///  (`remove_the_trees`,
/// `service_server.rs`):
///  keeping both would keep a tree no request lands on.
///  A request whose answer was lost is sent again with the identity and the tree it already had
///  (`submit_loop`,
/// `service_client.rs`),
///  so its note replaces itself and the tree its queued landing reads stays held.
///  A path with no file name in it names no tree of a scratch place and is not recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// note(id, tree); // void
/// ```
pub fn note(id: &str, tree: &Path) {
    let Some(name) = tree.file_name() else { return };
    let mut held = HELD.lock().expect("the trees the queued requests hold");
    if let Some(earlier) = position_of(held.as_slice(), id) {
        // The attempt this request left is its own to release, so only the tree it names now is kept.
        let _replaced = held.remove(earlier);
    }
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
    let mut held = HELD.lock().expect("the trees the queued requests hold");
    if let Some(entry) = position_of(held.as_slice(), id) {
        let _released = held.remove(entry);
    }
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
    let held = HELD.lock().expect("the trees the queued requests hold");
    for (_, named) in held.iter() {
        if named.as_os_str() == name {
            return true;
        }
    }
    return false;
}

/// The entry of the record that the request `id` holds,
/// as its place in the record.
/// Why:
///  The record is one entry per request,
///  and removing an entry by its place neither borrows the record for the removal nor hides the search in a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// positionOf(held, id); // the index of the request's entry, or nothing
/// ```
fn position_of(held: &[(String, OsString)], id: &str) -> Option<usize> {
    let mut place: usize = 0;
    while place < held.len() {
        if held[place].0.as_str() == id {
            return Some(place);
        }
        place += 1;
    }
    return None;
}

/// The record and its two ends are controlled in the test module.
#[cfg(test)]
#[path = "service_scratch_trees_of_queued_requests_tests.rs"]
mod tests;
