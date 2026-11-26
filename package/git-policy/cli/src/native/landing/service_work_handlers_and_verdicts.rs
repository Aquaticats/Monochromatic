//! What:
//!  What a handler of the service's work came to (`Verdict`),
//!  and the handlers themselves (`WorkHandlers`):
//!  one method per kind,
//!  each with the awaiting twin that a task of the runtime calls.
//! Why:
//!  Every kind the drain runs is answered in these words,
//!  and a handler that does not know a kind skips it in words of its own;
//!  the awaiting twin exists because a handler that starts Git cannot be
//!  called from a task of the runtime,
//!  so a caller reads one trait rather than a cast per kind
//!  (`service_work_drain_waves_of_one_kind.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Verdict = { kind: 'done' } | { kind: 'delivered', commit: string, said: string } | { kind: 'failed', text: string } | { kind: 'skipped', text: string } | { kind: 'retry', text: string };
//! interface WorkHandlers { push(items: WorkItem[]): Verdict; pushAwaiting(items: WorkItem[]): Promise<Verdict>; /* ...one pair per kind... */ }
//! ```

use crate::work_item::WorkItem;

/// What a handler came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// The work was done,
    /// or there was nothing to do.
    Done,
    /// What:
    ///  The push delivered the commit,
    ///  and the words are what the server and the reconcile said about it,
    /// which the person is told through a record.
    /// Why:
    ///   Only the push answers this,
    /// and the record is written where every other record of a push is written,
    ///  by the step that settles it,
    /// so that a storage that refused the words leaves the item not done and says why
    /// (`service_work_settling_a_verdict.rs`),
    /// and so that both the round and the retry plan
    ///  record what a landed push said.
    ///  The commit is the one the push sent,
    /// never the newest record the round claimed:
    /// the record is named for it,
    ///  and a commit that landed after the claim is what the push sends.
    Delivered {
        /// The commit the push sent,
        /// which its record is named for.
        commit: String,
        /// What the push chose to show,
        /// which is nothing when it said nothing.
        said: String,
    },
    /// The work failed;
    /// the text is what the person should be told.
    Failed(String),
    /// Nothing was done and nothing is left to retry,
    /// but the person should be told why;
    /// the text is what they are told.
    Skipped(String),
    /// The push failed for a reason that can pass (a stall,
    /// the network,
    /// an LFS server error,
    /// a held lock,
    /// a remote that advanced again,
    /// the server's protected-branch refusal);
    /// the text is what the person should be told,
    /// and the service tries the push again on its own,
    /// with growing pauses,
    /// for a fixed number of tries when the cause is the
    ///  server's protected-branch refusal.
    Retry(String),
}

/// What runs each kind of work.
pub trait WorkHandlers {
    /// Push the branch of these items,
    /// which share a repository and a branch;
    /// the last is the newest.
    fn push(
        &self,
        items: &[WorkItem],
    ) -> Verdict;
    /// The same push,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The round's items are tasks,
    /// and a handler that starts Git cannot be called from one;
    /// a handler that knows nothing of the runtime keeps the synchronous one as its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pushAwaiting(items: WorkItem[]): Promise<Verdict>
    /// ```
    fn push_awaiting<'a>(
        &'a self,
        items: &'a [WorkItem],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.push(items)));
    }
    /// Run the repository's automatic upkeep.
    fn maintenance(
        &self,
        item: &WorkItem,
    ) -> Verdict;
    /// The same upkeep,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The upkeep runs Git,
    /// and a task may not start one from a synchronous face;
    /// a handler that does no upkeep keeps the synchronous one as its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// maintenanceAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn maintenance_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.maintenance(item)));
    }
    /// Review one landed commit into the repository's shadow log.
    fn review(
        &self,
        item: &WorkItem,
    ) -> Verdict;
    /// The same review,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The round's items are tasks,
    /// and a review's wait for its contained job is the runtime's there;
    /// a handler that knows nothing of the runtime keeps the synchronous one as its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reviewAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn review_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.review(item)));
    }
    /// Empty the directory of left-out files a snapshot-made worktree set aside;
    /// a handler that does not know it skips.
    fn trash(
        &self,
        _item: &WorkItem,
    ) -> Verdict {
        return Verdict::Skipped(String::from(
            "this handler does not empty trash directories",
        ));
    }
    /// The same emptying,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The drain of a round is a task,
    /// and the synchronous disposal enters the runtime for its entry
    /// removals,
    /// which is refused from inside it;
    /// a handler that knows nothing of the runtime keeps the synchronous one
    /// as its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trashAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn trash_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.trash(item)));
    }
    /// Fill a new worktree's Cargo build directory from the build state of the checkout it was copied from;
    /// a handler that does not know it skips.
    fn prewarm(
        &self,
        _item: &WorkItem,
    ) -> Verdict {
        return Verdict::Skipped(String::from(
            "this handler does not fill Cargo build directories",
        ));
    }
    /// The same filling,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The drain of a round is a task,
    /// and the filling starts Cargo children and copies a tree,
    /// which the synchronous face would refuse from inside the runtime;
    /// a handler that knows nothing of the runtime keeps the synchronous one as its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// prewarmAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn prewarm_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.prewarm(item)));
    }
    /// Relink the copied package state of a new worktree with each tool's own install;
    /// a handler that does not know it skips.
    fn relink(
        &self,
        _item: &WorkItem,
    ) -> Verdict {
        return Verdict::Skipped(String::from(
            "this handler does not relink the package state of a worktree",
        ));
    }
    /// The same relinking,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The drain of a round is a task,
    /// and each tool's install is a child process the synchronous face would refuse to start from inside the runtime;
    /// a handler that knows nothing of the runtime keeps the synchronous one as its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// relinkAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn relink_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.relink(item)));
    }
    /// Delete the subvolume of a removed worktree that the removal handed over;
    /// a handler that does not know it skips.
    fn subvolume_delete(
        &self,
        _item: &WorkItem,
    ) -> Verdict {
        return Verdict::Skipped(String::from("this handler does not delete subvolumes"));
    }
    /// The same delete,
    /// awaited by a caller that is itself a task of the runtime,
    /// which is how the round asks every kind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// subvolumeDeleteAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn subvolume_delete_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.subvolume_delete(item)));
    }
    /// Make a snapshot of a worktree ahead of the next add;
    /// a handler that does not know it skips.
    fn snapshot_ahead(
        &self,
        _item: &WorkItem,
    ) -> Verdict {
        return Verdict::Skipped(String::from("this handler makes no snapshots ahead"));
    }
    /// The same snapshot,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The ioctl waits for a btrfs transaction commit,
    /// so it belongs on the blocking pool and never on a worker the
    ///  runtime needs for the commits queued behind it;
    /// a handler that knows nothing of the runtime keeps the synchronous one as its
    ///  answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshotAheadAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn snapshot_ahead_awaiting<'a>(
        &'a self,
        item: &'a WorkItem,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.snapshot_ahead(item)));
    }
}
