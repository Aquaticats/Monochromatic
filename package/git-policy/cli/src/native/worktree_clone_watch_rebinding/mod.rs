//! What:
//!  Whether a file watch opened before a clone's mount still sees the tree
//!  afterwards,
//!  and the line that tells the person which watches to reopen.
//! Why:
//!  An inotify watch belongs to the directory object it was set on,
//!  never to the path:
//!  a mount over the slot covers the directory Git registered there,
//!  so a program that opened the slot before the mount holds a watch on the
//!  covered directory and sees no write in the clone's own tree
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Dependency repair and watchers").
//!  The wrapper can act on that fact in three ways,
//!  and this folder owns the two the wrapper can prove:
//!  it decides whether the path names another directory object than the one the
//!  registration recorded,
//!  and it writes the sentence naming the watches a person must reopen.
//!  It never reopens another process's watch and never stops a daemon:
//!  an editor's watcher is the editor's,
//!  and the owner decided on 2026-11-03 that the wrapper neither starts nor
//!  stops Git's fsmonitor daemon (`package/git-policy/cli/README.md`,
//!  "The wrapper's own Git and fsmonitor").
//!  The wrapper's own waits are safe by construction:
//!  each sets its watch when it starts waiting
//!  (`landing/service_index_watch.rs`),
//!  so no watch of the wrapper's outlives a mount.
//!  Plan step 21 builds this ahead of its wiring,
//!  and the wiring step is the one that calls it after the fill.
//!
//! Files in this folder,
//! each using only the file listed before it:
//!
//! - `watch_rebinding.rs`: the comparison, the sentence and the one call the
//!   wiring step makes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { watches, line } = afterTheMount(slot, recorded); // line only when the watches are blind
//! ```

pub mod watch_rebinding;

pub use watch_rebinding::{
    AfterTheMount,
    Watches,
    after_the_mount,
    the_object_the_path_shows,
    the_persons_line,
    what_a_watch_set_before_still_sees,
};
