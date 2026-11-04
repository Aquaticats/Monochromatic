# Commits never fail: review history archive (first ten reviews, revision 9 and 10 changelogs)

Scope: `package/git-policy/cli`. The oldest part of `doc/planning/cli-git-commits-never-fail-review-history.md`, split off in revision 18 so that each record stays within the size a reviewer can
read whole. Every section is superseded by the normative state machine of `doc/planning/cli-git-commits-never-fail.md`.

## Answers to the first review

History. Revision 8's "Normative state machine" is the operative statement; where this section conflicts with it, it is superseded.

Each finding of `~/temp/agent/reviews/commits-never-fail-sol.txt`, with a decision.

- Critical 1, deferred merge checkout overwrites later edits: accepted; the merge keeps Git's order.
- Critical 2, index reconcile erases a foreign program's staged content: accepted; superseded by the second review's finding on
  whole-index installs (below).
- Critical 3, client fallback is a second coordinator: accepted; superseded by the fence in the publication transaction (below).
- Critical 4, reclaiming a "proven dead" lock: accepted; no foreign lock is removed.
- High, stacking replays the captured change: accepted (Transaction model).
- High, hold refs leave a window and end with the connection: accepted; superseded by the retention lifecycle (below).
- High, unbounded waits: accepted in part. Rejected in part: the claim that waiting off the commit's path moves the wait onto the
  next batch; recovery uses a single try for the lock and never blocks a batch (`service_land.rs`, `recover_selected` calls
  `try_acquire` and leaves the intent open).
- High, native fallback is not end to end: accepted; gates and a complete Git implementation, and failure independence is not claimed.
- High, hooks, required filters and `paths-in-use` unclassified: accepted; decided by the owner.
- Git source qualifications: accepted ("one `lstat` per tracked file" is too categorical; a failed `update-ref` is classified by
  its message; the lock watcher is files-backend only; `core.lockfilePid` proves only what it records; the port of
  `update_index_from_diff` needs differential tests).

## Answers to the second review

History. Revision 8's "Normative state machine" is the operative statement; where this section conflicts with it, it is superseded.

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-2.txt`.
This section is kept as history. Where it names the fence ref, the receipt ref, `queue/<n>` or the index checksum, those features
are withdrawn and "Answers to the third review" gives the current answer.

- Critical, epoch fencing does not fence an obsolete worker: accepted. The reviewer's execution is right: a check that precedes the
  swap proves nothing. The fence is now a ref verified inside the publication transaction, whose refs are all locked in
  `prepare` (Transaction model, "Fencing"). Atomic renames of state files do not fence either; state files are exclusive,
  monotone caches of facts, and only the ref transaction is fenced.
- Critical, carry-forward loses writes through an open descriptor: accepted. A rename does not freeze an inode. The protocol now
  pins the inode by a hard link, exchanges with `RENAME_EXCHANGE`, re-reads the exchanged-out inode, and states the supported
  semantics; a late write goes to the saved file (Merges).
- Critical, installing an index built off the lock erases foreign staging: accepted. The delta is applied to the index read under
  the lock. On provenance: equality is a conflict guard, not provenance; the index checksum recorded at the publication detects
  that a program wrote the index after it; an entry equal to the old one in a changed index is left (the worktree keeps the
  content, so nothing is lost). I do not claim more than that (Transaction model, "Installing into the current index").
- Critical, recovery infers "never published" from reachability: accepted. A receipt ref is created in the publication
  transaction; recovery decides from the receipt; a commit is never resurrected after a deliberate reset. A crash in the
  microsecond window of the files backend on a repository without a branch reflog leaves the commit saved under its receipt ref
  and says so (case 4).
- High, acknowledgement conflated with `closed`: accepted. Acknowledgement is a flag; `closed` means all obligations are
  discharged; recovery has an explicit case per state; the persistence ordering for spool files, records and directories is
  listed.
- High, pending needs a logical head and a durable order: accepted, with the reviewer's `value` and delete examples.
  Capture is against the newest pending commit; order is `queue/<n>` created exclusively under the fence.
- High, the spool must hold the complete replay closure: accepted; the closure is enumerated, the base is held before any read,
  and a prune is tested at every interval.
- High, the side ref can meet a second foreign lock; an unchanged tip does not prove a lock: accepted. The failure is classified
  from the actual message; pending does not depend on a ref write; reftable is stated.
- High, remaining Git hangs: accepted in part. `update-ref` is made safe to run twice by the receipt and is restarted like the
  other children. A deterministic Git bug cannot be restarted away; an in-process transaction writer for the files backend is
  named as the alternative and not chosen. The exit inventory is a gate, as the reviewer asks.
- Medium, pending refs kept from pushes by assertion: accepted. Enforcement is at the wrapper's push boundary for the auto-push and
  explicit refspecs; a `--mirror` push leaves wrapper refs out with a line saying so (owner, below).
- Foreign `index.lock` wait: the owner decided "Same as the ref lock"; the commit and the merge no longer wait on it. The
  carry-forward loop is bounded at two rounds. The reviewer's point that a wait is not a completion guarantee stands: the
  pending request and the lagging index complete only when the foreign program lets go, and the record says so.

## Answers to the third review

History. Revision 8's "Normative state machine" is the operative statement; where this section conflicts with it, it is superseded.

This section is kept as history. Where it names a watchdog, the kill of a recorded process group, the `ledger-open` lock sweep, the
ledger horizon, the conditional push exclusion, checksum or entry-equality provenance or the accepted ABA residue, the fourth
review's answers supersede it.

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-3.txt`, with a decision and a closure line. Closure is one of
closed (the design answers it, to be tested), narrowed (the feature was reduced and the owner decision it still meets is named), or
open (with the reason).

1. The files-backend receipt does not prove publication happened (critical): accepted; the receipt ref is withdrawn.
   Publication is a plain compare-and-swap; the record writes `publishing` before and `published` after; recovery treats
   reachability from the tip as the only proof of publication, uses a branch-reflog entry only to avoid resurrecting a commit that
   was on the branch, and otherwise publishes again by replay. The reviewer's execution (receipt, crash, foreign commit `C`) now
   ends in a second publication by replay, not in a closed request. Closure: narrowed. Meets "never discard user data" (a commit is
   published again or saved, never classified published without proof) and "never fail". The accepted residue is one commit
   reappearing after a non-descendant rewrite with no reflog, and a question for the owner.
2. Deleting the receipt removes duplicate protection (critical): accepted. Retirement appends a terminal line to `lane/ledger`
   (fsynced) before any retention is released, a resend is answered from the ledger, and the ledger is kept for a horizon far longer
   than a client lives. Closure: closed (to test: crash between the ledger line and the release; delayed retry after retirement).
3. Ref fencing does not make obsolete worktree, index and cleanup operations harmless (critical): accepted. The ref fence is
   withdrawn. One kernel lock per worktree, inherited by every child, means an obsolete worker or child cannot exist when a new
   owner holds it; cleanup is done only by the owner and only after the ledger line. Closure: closed in the model (to test: kill at
   every phase, next owner at once, no old child alive). Meets "Use as few locks as possible" by one lock per worktree and an
   NLK reason; the lock-free alternative is a service-only lane, which is a question for the owner.
4. An index checksum is not staging provenance (critical): accepted. Checksum provenance is withdrawn. When the lock is free the
   lane takes it before the swap and installs in one critical section, so nothing interleaves; when it is held the deferred install
   verifies the commit is still reachable from the tip (a rewritten branch voids the intent), and uses entry equality plus the
   worktree file's content as a conflict guard, not as provenance. The ABA by value is stated as an accepted residue (the file, the
   history and the index agree; no content is lost). Closure: narrowed. Meets "never discard user data".
5. Two exchanges do not retain every displaced inode (high): accepted. Every inode involved (the person's file, the file displaced
   by each exchange, the first merge result) is renamed to a named saved file, journaled per step with a crash recovery rule, and
   saved files are independent of request cleanup. The protocol is bounded and says it may leave more versions in saved files
   rather than incorporate every concurrent write; the copy fallback is replaced by "the person's file stays, the merged version is
   written beside it". Closure: narrowed. Meets "Carry edits forward" and "never discard user data".
6. Capture and takeover need ref writes a foreign lock blocks (high): accepted. Acceptance writes no ref: the request is durable in
   `inbox/`; the base hold ref is replaced by reading the base content into the spool (a race with a prune is detected and read
   again before handover); there is no fence ref and no takeover; an interrupted wrapper-owned ref transaction leaves lock files
   that the next owner sweeps only when its own ledger lists them with matching content. Closure: closed.
7. The fence namespace is repository-wide (high): accepted. No fence ref remains. The lane lock, the inbox, the order file, the
   ledger and the hold and pending refs are per worktree (`refs/cli-git/<instance>/`), branch publication is only a compare-and-swap,
   materialization is only into the publishing worktree, and "a branch is checked out in one worktree" is not used as an
   exclusion. Closure: closed.
8. `queue/<n>` is not a fenced durable acceptance, and recapture by base substitution is not a rebase of intent (high): accepted.
   Acceptance is the durable existence of the complete record in `inbox/`; the single lane orders by an appended `adopt` line;
   there is no numbered slot to stall; requests carry a delta against a logical base and the lane replays it, and the base is never
   substituted under an unchanged tree. Closure: closed.
9. Logical head needs a logical-index model; pending amendments need versions; signed replay needs a rule (high): accepted. A plain
   commit captures the difference between the real index and the physical head's tree and applies it to the logical tail; the stated
   limit is that staging a path back to the physical head's content cannot express a revert of a pending commit. Amendments are new
   immutable requests that supersede, adopted by the one sequential lane. A signed commit publishes only on its own parent and
   otherwise stays pending until the person signs again. Closure: narrowed for signed commits (a question for the owner), closed
   for the rest.
10. Index extensions cannot be kept byte for byte (high): accepted. A per-extension policy replaces the claim: the cache-tree is
    dropped; the resolve-undo data and the untracked cache are carried unchanged; the offset tables are not written; fsmonitor,
    split index, sparse index and unknown extensions make the index ineligible and the delta runs through Git on a copy under the
    lock. Closure: closed (to test: differential against Git on each extension).
11. Spool reclamation needs an overlapping-prune proof (high): accepted and checked in Git's source (`builtin/prune.c:36`, `:98`,
    `:172`): a prune that took its view earlier can delete objects written later. A proof that no schedule loses data does not
    exist and the design does not claim one. Closure: narrowed. The lane checks connectivity after publication and before releasing
    the spool, imports again on a loss, and treats a loss after the release as a corrupt repository caused by another tool, on the
    owner's outside list. A question for the owner.
12. Push enforcement, reference-transaction hooks and deterministic hangs (high): accepted.
    - Push: enforcement is on the effective updates found by a `--dry-run --porcelain` expansion, with a negative refspec for the
      namespace, covering `refs/*:refs/*`, configured push refspecs and `--mirror`; whether the negative refspec combines with
      `--mirror` is a control on the installed Git. Closure: closed pending that control.
    - Hooks: internal transactions on `refs/cli-git/` run without the person's hooks; `update-ref` for the branch is not restarted
      while a `reference-transaction` hook is configured. Closure: closed.
    - Deterministic Git hangs: a child that stalls twice on identical input makes the request pending with a message and exit 0.
      Closure: narrowed (the commit is saved, not landed); a question for the owner, because Git bugs were listed as designed out.
    - Exit inventory: still a gate and not yet performed. Closure: open until the inventory is made from the 2.56.0 source.
- Ref storage and reflog gates: accepted. They join the verification plan (packed refs, `pack-refs`, deletion, interrupted
  transactions, reftable, the branch and worktree `HEAD` reflogs through the real service invocation).
- Review's verdict: not ready to build as the live protocol. This revision removes live takeover, receipt refs, the ref fence, the
  numbered queue and checksum provenance, and does not build; "What can safely be built first" is adopted as the build order below.

## Answers to the fourth review

History. Revision 8's "Normative state machine" is the operative statement; where this section conflicts with it, it is superseded.

This section is kept as history. Where it names `saved-ambiguous` as an outcome, a reflog line as publication evidence without an
anchor, entry equality as provenance, `update-ref` never being stopped, the saved-copies age rule, or Questions 1 to 4 below its
end, the fifth review's answers and the owner's decisions of 2026-11-03 supersede it.

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-4.txt`, with a closure line: closed (answered by the design, to be
tested), narrowed (reduced, with the owner decision it still meets), or open (with the reason).

Critical.
- Carry-forward crash recovery deletes the newest user version: accepted. The recovery rule is replaced by "a file is deleted only if
  its identity is an inode the lane created and journaled", the `R` inode is journaled before the exchange, the exchange intent is
  journaled before the exchange, and a temporary name holding any other inode is renamed to a saved name (Merges). Closure: closed
  in the model; to test: a crash between the exchange and its journal completion with an atomic-save editor inserting `Q`, at every
  journaled step.

Third-round findings that were open.
1. Publication evidence: accepted. A durable `published` record wins; only a reflog line whose old value is the commit proves it was
   rewritten away (that line is written by the update that moved the branch from it); a line whose new value is the commit does
   not; the lane creates the branch's reflog file before its first publication so the evidence exists; with no reflog the outcome is
   `saved-ambiguous`; anything promised as saved is retained permanently; no once-only promise is made. Closure: narrowed; the
   residue (no reflog) is an owner decision (below).
2. Retirement deduplication beyond the horizon: accepted. The ledger is append-only with no horizon and no rotation (about a hundred
   and thirty bytes an entry, under five megabytes a year at a hundred commits a day, so there is no bound to justify); an
   identity is never treated as new once recorded. Closure: closed. The assumption "clients live seconds" is withdrawn.
3. Obsolete workers: accepted that inheritance is not a lifetime proof; the descriptor retention across exec, hooks, descendants,
   cancellation and inline ownership are named obligations with controls, and children carry the parent-death signal from a thread
   that lives as long as the owner. Closure: narrowed to a single owner, controls pending.
4. Staging provenance: accepted. The accepted residue is replaced by conservative preservation: an entry is replaced only if it equals
   the oldest pre-entry in every field, cached stat data included (entries made afterwards by `git add`, `reset`,
   `update-index --cacheinfo` or a path restore carry other stat data and are left); the only case undetectable is a byte-exact
   restore of an earlier index file, which is the same state as no change. Closure: closed to the extent the format allows; no owner
   decision needed.
5. Displaced inodes and the journal: closed by the critical item above.
6. Acceptance blocked by ref writes, interrupted-transaction cleanup: accepted. Acceptance stays ref-free. Lock sweeping by
   contents is withdrawn: the wrapper never removes a ref lock file after an owner crash; a left lock is treated as a foreign lock
   (see new 3). Closure: closed.
7. Worktree branch switch: accepted. The request records the worktree's attachment and a materialization generation; publication is
   to the branch the commit was made on; materialization is voided when the worktree is no longer attached to it (switched,
   detached, branch deleted, worktree removed) and a line says so. Closure: closed.
8. Adopt-log recovery: accepted. `lane/order` is authoritative; after a durable `adopt` the request is adopted wherever its record
   lies; recovery completes the move and never appends twice; lines are framed with a checksum and a torn last line is ignored.
   Closure: closed.
9. Logical index, amendments, signed: accepted. The claim is replaced by the consumption map (the real-index entry already consumed
   by an adopted request, per path), which fixes the reviewer's `x=1`, `x=2`, `y` sequence; an amendment whose target is no longer
   the tip is stacked as a new commit relative to the target commit and never amends a successor; signed commits are an owner
   decision (below). Closure: closed except signed.
10. Extensions: accepted; carrying the resolve-undo data and the untracked cache needs a differential control on Git's later
    behavior, not only that Git accepts the file; otherwise they are dropped or the index is ineligible. Closure: narrowed to a
    verification obligation.
11. Prune boundary: accepted that "prunes starting after handover" is inaccurate; the boundary is whether a destructive operation can
    outlive the retained recovery copy (above). Closure: narrowed; an owner decision (below).
12. Push, hooks, hangs, inventory: accepted. The negative refspec is unconditional and the lane pauses best-effort ref creation
    under a marker for any form where the control shows the refspec is not honored; avoiding restart of `update-ref` also holds
    because the owner is no longer killed by a watchdog; deterministic hangs are an owner decision (below); the exit inventory is
    still an unperformed gate. Closure: closed for push and hooks, open for the inventory.

New findings.
1. Kill before acquire and unsupervised inline owners: accepted. Nothing is signalled before the lock is held; acquisition blocks
   with a notice; children die with the owner by the parent-death signal; an inline owner lives only as long as its own foreground
   command, drains requests adopted before its own, and leaves later ones to the service. Closure: closed.
2. The watchdog is a time bound and can repeat a hook: accepted. The watchdog is removed, with the progress-counter takeover; the
   unit sets no limits (owner decision, recorded in `SPEC.md`), and no value is shipped without a measurement or owner decision
   beside it. `update-ref` is never stopped by the wrapper. A wedged owner of our own making is handled by the person restarting
   the unit, with the request durable meanwhile. Closure: narrowed; the cost is that a wedged owner has no automatic recovery, a
   consequence of the owner's decision that the unit has no limits.
3. Lock contents do not prove ownership: accepted; the sweep is withdrawn (see 6 above). Closure: closed.
4. Dry-run expansion is not the real push's: accepted; unconditional exclusion plus the marker (above). Closure: closed pending the
   per-form control.
5. Reachability does not make an old index intent current: accepted. Per-path supersession composes unapplied intents: the oldest
   pre-entry is the expected state and the newest committed entry is the target, so the reviewer's `R1`, `R2` sequence stages nothing.
   Closure: closed.

Judgment on the five questions put to the owner before: kernel lock versus service-only lane is an implementation choice, so it
is not asked; re-signing is necessary only under "single replayed commit, changed parent, no delegated signing"; the deterministic
Git bug narrowing is optional if a path avoiding the child is built; the prune boundary as worded was inaccurate; the no-reflog
ambiguity is inherent. The list below keeps only what the review says is the owner's.

## Answers to the fifth review

History. Revision 8's "Normative state machine" is the operative statement; where this section conflicts with it, it is superseded.

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-5.txt`, with a closure line (closed: answered by the design, to be
tested; narrowed: reduced, with the decision it meets; open: with the reason).

1. Publication evidence (open in review): accepted. Rule 4 is replaced: an existing reflog is evidence only if the anchor recorded
   before the swap (reflog length and the hash of its last line) is still in the file, and only lines after the anchor belong to the
   attempt, which also answers the reused-commit-identifier execution (an older publication and removal of an identical commit
   cannot be read as this attempt's). When the anchor is gone the traces are unclear and owner decision 3 applies: land it, with a
   message that names the commit and the command to drop it. Closure: closed; the residue is the owner's decision 3.
2. Retirement and duplicate protection: closed at the protocol level, as the reviewer says; the four implementation obligations are
   adopted: torn-tail repair before append, per-line integrity, preservation across upgrades and worktree lifecycle, and rejection of
   an identity reused with different content (hash of the immutable request).
3. Obsolete workers: narrowed. "Obsolete workers cannot exist" is restated as a safety argument that holds if every mutator keeps the
   lock description, with a cgroup for descendants that close it (inline owners in a transient scope, services in their unit's cgroup),
   and controls for retained-descriptor and closed-descriptor descendants. Closure: narrowed to controls.
4. Foreign staging provenance: accepted that stat fields do not identify a staging operation (the reviewer's tree-created zero-stat
   entry). Owner decision 2 settles it: the deferred follow-up stages the committed content as Git's `git commit -- path` would, a
   deliberate restage to exactly the old entry inside a foreign lock's window is replaced, and this is an accepted concession.
   Closure: closed by the owner's decision; stated as a concession, not a provenance claim.
5. Carry-forward crash recovery: process-crash counterexample closed in the model. Power-loss ordering is now written down (each
   namespace effect with the fsync of the affected directory, recovery identity-based) and is a per-file-system verification
   obligation. Saved copies are retained until explicit disposal (adoption), which closes the sole-copy deletion by the age rule.
   Closure: closed in the model; filesystem durability open as a verification obligation.
6. Ref-free acceptance and interrupted lock cleanup: closed. An abandoned lock keeps the request pending until someone removes it; the
   notice names the file.
7. Worktree attachment and generation: accepted. The generation counter is withdrawn as a fence. The lane brackets reads of `HEAD`
   and `logs/HEAD` around the index read while holding `index.lock` and declines ambiguous materialization. The window between Git's
   own index write and its `HEAD` update in `checkout` and `switch` cannot be seen by any bracket, so the residue is a question for the
   owner (below). Closure: narrowed; the residue is the owner's.
8. Adoption order: closed at the protocol level, with the torn-suffix repair before appends and capture time stated as a convention.
9. Logical index, amendments, signed requests: capture consistency is now explicit (state version before and after reading, a delta
   replayed and never reinterpreted); signed replay follows the adopted background re-signing; the consumption map's limitation (staging a
   path back to exactly its consumed entry cannot express a revert of a pending commit) is a question for the owner (below). Closure:
   closed except the limitation, which is the owner's.
10. Index extensions: narrowed to a verification gate (differential controls on Git's later behavior); no new counterexample.
11. Overlapping prune: narrowed. The temporal boundary is stated accurately; corruption by another tool after release is on the owner's
    outside list (adoption); maintenance the wrapper starts is coordinated over its whole lifetime by extending retention, which closes
    the reviewer's idle-lane hole. Closure: narrowed to the owner's outside list.
12. Push, hooks, hangs, inventory:
    - Push: the unconditional exclusion stands; the fallback is a frozen filtered explicit update set under an acknowledged marker, so a
      ref created after the marker cannot enter the set (mirror deletions listed explicitly). Closure: closed pending the per-form control.
    - Hooks after an owner death: `hook-dispatch` is journaled before `update-ref`; recovery runs the evidence rules and invokes
      `update-ref` again only if the swap did not complete; Git's own contract allows the repetition (`Documentation/githooks.adoc:486-488`).
      The exception, a hook with an external effect on "prepared", is a question for the owner (below). Closure: narrowed.
    - Deterministic hangs: the end state "pending with a message" is withdrawn; the direction is the embedded path (adoption).
      Closure: open as work (the embedding), with differential controls.
    - Exit inventory: still an unperformed gate. Closure: open.

Fourth-round new findings.
- Kill before acquire: closed. Inline lifecycle: closed with the cgroup scope (finding 3 above) and the stated fallback (no inline
  ownership where a hook is configured and the manager is unavailable).
- Watchdog cancellation and repeated hooks: narrowed. The reviewer's separate point stands: removing the watchdog does not satisfy
  "never stuck". The stuck-step rule of owner decision 1 covers Git children and holds hooks; a quiet owner of our own making is a
  question for the owner (below). Closure: open as a question.
- Lock ownership by contents: closed.
- Dry-run versus actual push expansion: narrowed; see push above.
- Stale deferred intents: accepted. The target is the current branch tip's entry for the path, not the newest wrapper intent, so a
  foreign successor is respected (the reviewer's `R1`, native `C` sequence leaves the entry at `A`). Closure: closed.

New high findings.
- Descendant lifetime for an inline owner: closed by the cgroup design and controls for both descriptor cases.
- Age cleanup deleting the only carried edit: closed (adoption: retained until explicit disposal).
- Maintenance admission at an idle lane: closed (coordination over the whole lifetime by extending retention).

Judgments requested of the reviewer, as adopted: a wedged owner has no general construction that detects a silent wedge, tells it from a
slow correct operation, and never repeats a hook's effect; the design therefore uses the measured no-progress rule of decision 1 and
asks the owner what remains. Full entry equality does not preserve every expressible foreign staging state (decision 2 accepts it). The
signed-commit and hang questions are settled by the adoptions above unless the owner vetoes them; the prune question is partly settled by
the outside list.

## Answers to the sixth review

History. The normative state machine of revision 9 governs; where this section describes the seqlock (any form), across publication, the birth-time lock
removal, scope reaping by a recorded path, an `O_EXCL` binding before the request exists or an empty scope as finished, it is superseded.

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-6.txt` (summarized on main in `doc/handover/cli-git-rust-implementation.md`
under "Commits, merges and pushes that never fail"), with a closure line (closed in the model: answered by the design, a red control
exists to be built; narrowed: reduced, with the decision it meets; open: with the reason). The verdict: ready to build step 1, executable
models and disposable crash fixtures only; not ready to build the live protocol. This revision does not build.

- F1, an intact anchor does not prove an intact suffix (P2): accepted. Recovery now has evidence classes: a durable `published`, a
  recorded `swap-failed` (the only proof a swap did not complete), the tip or ancestry, a positive old-value line after the anchor
  (`rewritten-away`; object-level observation is enough because the branch held what was requested), and everything else is unclear
  and goes through owner decision 3 ("Land it"), including an intact anchor with no old-value line. The anchor is read through Git so
  reftable is covered; its backend behavior is a control. Closure: closed in the model.
- F2, dead inline-owner scope cleanup can be unreachable (P1): accepted. Children never hold the lock, the owner records its
  incarnation and scope before launching anything, and the new owner, whose acquisition proves the previous owner dead, reaps the
  recorded scope and waits for it to report empty before any mutation. The manager-unavailable fallback covers all arbitrary child
  programs (hooks, filters, signers, helpers); a hook that asks the manager to start another unit is outside, stated. Closure: closed
  in the model; red controls: retained-descriptor scope deadlock, closed-descriptor descendant writing after the lock frees.
- F3, quiet-owner restart races hook admission (P1): accepted. A gate lock, taken shared by the owner around every person's program and
  every ref transaction and taken exclusively without waiting by the restarter, serializes the two; the restart acts through a pidfd
  on the observed incarnation after re-reading it under the gate. `hook-dispatch` stays as conservative evidence, not as the interlock.
  Closure: closed in the model; red control: restart versus hook dispatch.
- F4, stopping `update-ref` can leave an orphan lock (P1): accepted. One cancellation protocol (terminate signal so Git removes its
  lock files, killing only a child that ignores it, and then only the lock whose birth time lies inside the child's life and that
  was absent before launch), no restart during a ref transaction, no lock removal after a crash, and "never stopped" withdrawn.
  Closure: closed in the model for wrapper-made locks; a crash or outside kill leaves a foreign-treated lock, which is the owner's
  outside list; red control: quiet `update-ref`, cancel and owner restart.
- F5, the HEAD bracket is a guard, not a fence (P2): accepted. Relabelled as a guard; the reflog-growth condition is dropped because an
  ordinary commit can grow `logs/HEAD`; the residue is question 4 (picked A), index-only and recoverable, and is not reused for worktree
  mutation. Which operations update the worktree HEAD reflog is a backend inspection. Closure: narrowed.
- F6, reading the same version twice is not a consistent capture (P1): accepted. A seqlock: intent journaled, generation odd before any
  mutation, even after; readers accept only an unchanged even generation and re-validate foreign refs and index entries; interrupted writers
  are completed or rolled back from the journal; starvation is stated and capture waits only on our own lane. Closure: closed in the
  model; red control: the reviewer's `x=0`, `x=1`, `x=2` schedule.
- F7, immutable acceptance does not enforce identity binding (P1): accepted. A permanent binding file created with `O_EXCL`
  (`ids/<xx>/<id>` holding the request hash) before the record is published with a no-replace rename, and a repository-owned location
  in the common directory for the binding and the ledger. Closure: closed in the model; red control: same identity, different content,
  concurrent acceptance.
- F8, maintenance coordination must be object-store-wide and recoverable (P1): accepted. One registry in the common directory,
  entries created before launch, each run in its own scope as the identity of the destructive execution, release only when no
  overlapping entry's scope is non-empty, and the four crash cases written out. Closure: closed in the model; red control: cross-worktree
  maintenance overlap.
- F9, hook bypass needs an ordering exception (P2): accepted. A held request is not part of the logical tail; a disjoint independent
  request replays its own delta and publishes ahead; an overlapping request waits; the held request later succeeds (replayed on the new
  tip, order reversed and stated) or refuses (nothing changes, the other request unaffected). Closure: closed in the model; hook-bypass
  ordering model.
- Question 1: A kept; versioned restaging events recorded as an optional later improvement with the review's cases; the bare path-marker
  candidate withdrawn. Questions 2 to 4: the picks stand; the stated conditions of the review are now written (the gate, the incarnation,
  the in-flight exclusion, the concession stated directly, the guard).
- P3 cleanup: one operative rule each for cancellation, inline-owner lifetime, optional index locking, background re-signing and the
  attachment guard, each named in "One owner per worktree", "Cancelling a stalled Git child", "Installing into the current index" and
  "Adoption order"; earlier answer sections are marked history; "no watchdog" now says no unit watchdog and names the gated client restart
  as the one external supervisor.
- Push fallback: the execution contract of the frozen set is written (fixed object identifiers, mirror deletions, neutralized options,
  preserved semantics, reference-counted pause); it stays a P2 verification gate. Ledger: kept, no expiry.
- Still open and unchanged: the retained-child exit inventory and the deterministic-hang embedding (work to do), and the extension and
  carry-forward durability gates.

## Answers to the seventh review

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-7.txt`, with a closure line (closed in the model: answered by a named transition
and a red control to build; narrowed: reduced, with the guarantee stated plainly; open: with the reason). Verdict: step 1 remains the right
first build; the live protocol is not ready. This revision builds nothing.

- F1 precedence (P2): accepted. R3 (positive old-value evidence after the anchor) is checked before R4 (`swap-failed`), so a failed invocation
  followed by the identical commit installed and removed by another writer is `rewritten-away`, not replayed. Old values are read from the
  entry's own record, never from an adjacent entry; the Git interface for this on each backend is a backend inspection. Closure: closed in the model.
- F2 (P1): accepted in both parts. (A) The fork-to-exec window is modelled: close-on-exec acts at exec, a child between fork and exec holds the
  description, and L1 waits for it (bounded by that child's progress; stated as a narrowed guarantee), instead of claiming children never hold it.
  The proof is restated: acquisition proves no conflicting lock remains on that inode, and the previous owner's death follows from the enforced
  invariants (children close at exec, the lock file is never replaced, no explicit unlock, no person's program runs in the owner process).
  (B) Each incarnation has its own cgroup named by its unique id (transient scope for an inline client, delegated sub-cgroup for a service), and L2
  targets only the recorded previous id, so a replacement owner can never kill its own group. Closure: closed in the model; the older-implementation
  case is not covered by the proof and needs no migration while nothing is built.
- F3 (P1): accepted. "The person's program" is the execution tree: the owner holds the shared gate until the invocation's cgroup is empty. The
  restarter is fail-closed through the durable `lane/restart-intent` (an owner that finds it admits nothing and exits), uses a pidfd and no manager stop
  request, and runs only within one pid namespace. The wedged shared holder is a remaining internal wait, stated plainly as owner question B.
  Closure: closed in the model except the wedged shared holder, which is narrowed.
- F4 (P1): accepted. The birth-time lock deletion rule is removed and is not to be implemented; the wrapper never removes a ref lock file. X1 sends
  the terminate signal only and exempts any command capable of launching a protected program, decided at spawn. A child that ignores the signal is
  owner question A (default: do not kill, request pending). Closure: narrowed; the choice is the owner's.
- F5 (P2): accepted. M1 is the only materialization path and carries the attachment guard, so an immediate follow-up cannot bypass it. Closure:
  closed in the model; the accepted residue stays question 4 (picked A).
- F6 (P1): accepted. The generation is odd only around lane state updates (C5, P3, M1's index replacement), never across P2, a hook or a foreign
  lock; publication is journaled in P1 without a generation change and the in-flight commit is already in the logical tail. The optional
  index lock is taken after the publication, so it is not held across a hook. The base closure read is part of C1's validated capture. Acceptance
  waits for short local updates only; stated. Closure: closed in the model; red control: a hook that waits for ever during P2 with a disjoint
  request being accepted.
- F7 (P1): accepted. The binding is published by `link` from a fully written file (never partial), names an already-durable `incoming` request, and
  a same-content resend or L4 completes the no-replace inbox publication; the reserved state is defined. Closure: closed in the model.
- F8 (P1): accepted. Reservation, launch authority and terminal state are separate: the launcher is placed in the run's scope before the entry is
  observable and launches from inside it; empty or absent scope is never read as finished; the launcher's death with an empty scope proves launch can
  no longer happen. Where no manager exists, containment is impossible and the recovery copy is kept until explicit disposal (G4). Closure: closed in
  the model with a manager; narrowed without one.
- F9 (P2): accepted. One consumption release rule (materialized, voided or refused, never at publication); amendment of a held request is a versioned
  successor with an immutable base, its build queued behind the running hook, and completions carry (request, version) and are discarded for a
  superseded version. Closure: closed in the model.
- Section 3, seven operative contradictions: (1) Item 1 now describes X1 and the embedding direction and no longer says the kill is removed or that
  pending is an end state; (2) Item 4 uses background re-signing; (3) Item 4 uses the one transition P4 (one try, then pending); (4) Item 2 limits an
  inline owner to draining earlier requests and its own; (5) the acceptance guarantee states what it waits for; (6) the consumption release rule is
  one; (7) Item 6's stale owner-question language is replaced by the outside list plus coordination. Closure: closed.
- Step 1 expressive power and properties: adopted in "Verification plan and build order".

## Answers to the eighth review

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-8.txt`, with a closure line (closed in the model means: closed in the revision 9
state machine and owed an executable model in step 1; nothing is built). The revision 9 changelog names every changed transition.

- R8-1 (P1), restart without the gate: accepted. L7 requires the exclusive gate and ends nothing without it; L6 reads the restart intent after
  the shared gate is held; the pick of B-restart is withdrawn for B-hand. An internal wedge that holds the gate is therefore not restarted
  automatically; the command says how to restart the unit. Closure: closed in the model; red control: restart versus hook dispatch.
- R8-2 (P1), shared ledger and tail repair: accepted. One ledger per lane with a single appender; L4 repairs only the own ledger; a reader of
  another lane's ledger treats a torn tail as not yet written. Closure: closed in the model; red control: two lanes, one repairer.
- R8-3 (P1), shared candidate directory, same-identity writers: accepted. Private candidate directories with a random nonce; C3 routes by
  the identity's state; the record's fsyncs are redone before any answer; a bound request is never removed as abandoned (the F7 model finding).
  Closure: closed in the model; red controls: same content from two writers, resend after adoption and retirement, resend of a visible record.
- R8-4 (P1), capture mixes state: accepted. One atomic `lane/state` document (rule S); the marker only around effects outside it; C0 routes a
  client out of a dead odd generation; a moving tip does not invalidate capture. Closure: closed in the model; red control: `x=0`, `x=1`,
  `x=2` with a logical-head mutation between two reads.
- R8-5 (P1), protection classification races dispatch: accepted. Classification by command kind, not by configuration read at spawn; the
  alternatives are the C-class owner question. Closure: closed in the model; red control: a hook installed after classification.
  S-1 and S-2 (the shipped sweeper and judge): accepted. The design depends on neither; it works with zero removals and states what it needs
  from `fix/git-policy-cli-never-removes-a-live-lock`; A-keep and A-kill are recorded as an accepted narrowing, not as a removal by construction.
- R8-6 (P2), consumption release after voiding: accepted. Voiding ends installation authority and keeps a suppression; release is by
  observable state (M3); a superseded held version gets a durable outcome (B3). Closure: closed in the model; red control: the foreign
  successor followed by a plain commit of `y`, and a superseded held version.
- R8-7 (P2), T1 and the terminal predicate: accepted. T1 uses G3's predicate and a release intent; later admissions are covered by the
  reachability argument, with the residue stated (a person rewriting the branch after the check and then pruning). Closure: closed in the
  model for the stated boundary; red control: admission racing the final release.
- R8-8 (P2), completeness: accepted in full. The lifetime owner lock is exempt from the "nothing but L6" wording; the fresh lane skips L2;
  B1 records `ready`; `hook-ok` makes a hook run once per version (the H question); Q1 resumes pending and landing; the foreground makes
  one try and background attempts are Q1; E1 classifies every P2 exit; M1 recovers by journaled lock identity and names every per-path
  outcome; detached and unborn targets are defined; the client result has three facts; MG1 to MG6 carry the merge. Closure: closed in
  the model; red controls: every M1 crash boundary, merge refusal with a concurrent editor, lost swap after checkout.
- Owner picks A2 and B2 (now A-keep, A-kill, B-hand, B-restart): A-kill stays as an explicitly accepted narrowing and the record does not claim a proof
  the shipped rule lacks; B-restart is withdrawn for B-hand and the two questions are no longer coupled through lock cleanup.
- Not a defect: none of the eight findings was rejected. One reading was refined: the review's phrase "the lane-lock lifetime" is answered by an
  exemption in P2's wording, not by releasing `lane/owner`, because releasing it would let a second owner reap a live incarnation.

## Revision 9 and revision 10 changelogs, moved from the design in revision 12

### Revision 9 changelog (for the step-1 models agent; every changed transition, with the model and the schedule)

Naming. Transition names are stable where the meaning holds. New names: B3, M3, Q1, E1, MG1 to MG6 and the unnamed rule S and the
"Publication target" rule. Owner-question options are now A-keep, A-kill, B-hand and B-restart (they were A1, A2, B1 and B2) so that no
option can be read as the transitions B1 and B2; C-class and H and M are new questions with their own letters. The models below use F1
to F9 of the verification plan; "new" is a model to add. Each schedule goes red under revision 8 and green under revision 9.
- L7 and L6 (R8-1; model F3). Schedule: the restarter samples a quiet owner with no person's program, the owner then dispatches a
  `reference-transaction` hook, the restarter ends the owner. Red under 8 (restart without the gate ends a hook); green under 9 (L7
  needs the exclusive gate, so it aborts while the hook holds the shared gate). Also L7 step (5): an owner that ignores the terminate
  signal; red under 8 (survives), green under 9 (pidfd SIGKILL). Option B-restart is withdrawn in favour of B-hand.
- Named entities, L4 (R8-2 and the L4 model finding; model new "two lanes, one repairer" and F6). Schedule: lane X appends to a shared
  ledger while lane Y repairs the torn tail. Red under 8 (Y truncates X's append in progress); green under 9 (one ledger per lane,
  repair of the own ledger only). Schedule for F6: kill L4 after it released one mutation and before the generation was even; red under
  8 (even generation shows half a mutation); green under 9 (odd first).
- C2, C3, C4 (R8-3 and the F7 model finding; model F7 and new "visible but not durable"). Schedules: two writers, same identity, same
  content; resend after adoption and after retirement; a resend that reads a record visible in `inbox/` that power loss then removes.
  Red under 8 (a shared candidate directory is written by two writers; an answer from a non-durable record); green under 9 (private
  candidate directories, routing by identity state, the C4 fsyncs redone before the answer).
- Rule S, C0, C1, C5, P1, M1 marker (R8-4; model F6 extended). Schedule: the reviewer's `x=0`, `x=1`, `x=2` with a logical-head and
  consumption-map mutation between two reads; a dead odd generation with no owner; a branch that keeps moving during capture. Red under
  8 (mixed state; capture invalid for ever; starvation); green under 9 (one atomic document, C0 recovery route, tip not re-read).
- X1 (R8-5; models F4 and F3). Schedule: a stalled `update-ref`, a `reference-transaction` hook installed after classification and
  before Git's dispatch. Red under 8 (a spawn-time classification calls it unprotected and cancels it, ending the hook); green under 9
  (classified by command kind, `update-ref` exempt). The shipped-lock input (S-1, S-2) is a stated dependency; no transition removes a lock.
- M2, M3 (new), B3 (new), C1 (R8-6; models F9, F6 and new "superseded version"). Schedules: the foreign-successor schedule followed by a
  plain commit of `y` (red under 8: `x=A` returns as staging; green under 9: suppression holds); a held version superseded by an
  amendment whose hook completes later (red under 8: no terminal outcome, entries and spool undefined; green under 9: `superseded` in the
  ledger, entries re-keyed).
- T1 and G1 (R8-7; model F8 extended). Schedule: T1 observes no open run, G1 publishes a reservation, G2 starts a prune, T1 releases.
  Red under 8 (scope-based predicate; no boundary); green under 9 (G3 terminal predicate, release intent, reachability: a closure released by
  T1 is reachable from the tip, so the late prune cannot take it).
- L2, L3 (fresh lane), B1 (`ready`, `hook-ok`), B2 (success without a second hook), P2 (wording), P4 (one foreground try), Q1 (new), E1
  (new), M1 (crash recovery by journaled identity, per-path outcomes), Publication target (new), Client result (new), MG1 to MG6 (new)
  (R8-8; models new "M1 crash boundaries" and "merge refusal"). Schedules: kill M1 at each of the six boundaries including after the rename and
  before the release (red under 8: a stranded `index.lock` or an unreleased entry; green under 9: identity-based L4); a merge whose
  hook refuses after the checkout while an editor saves a merged file (red under 8: no restoration; green under 9: MG6); a lost swap after a
  checkout (green under 9: MG5); a held hook that succeeds and the tip moves (red under 8: held, success, replay, held; green: `hook-ok`).
- R4 (model finding on F1). Schedule: the post-anchor reflog lines erased after a `swap-failed`. Red under 8 (silent replay); green under 9
  (R4 requires an intact reflog, otherwise class R5, which lands with a notice). Revision 10 note (R9-7): this closure is withdrawn; there is no
  observable intact-reflog predicate, and absence of evidence is class R5.

### Revision 10 changelog (for the step-1 models agent; every changed transition, with the model and the schedule)

Naming. Transition names are stable where the meaning holds: C0, C1 to C5, L4, M1, M2, B1, B2, P1 to P4, E1, Q1, T1, X1, MG1 to MG6 keep their
names with changed bodies. New names: rule D, rule N, M3 stays from revision 9, B3 is rewritten under its name, and the outcomes `conflicted`,
`sign-pending`, `unrooted`, `superseded by`. Owner-question letters added: R, P, W, G (and M restated); the options are named so that none reads as a
transition. Each schedule goes red under revision 9 and green under revision 10; a model marked "new" is added to step 1.
- Rule S (R9-11; model F6 and new "revision counter versus marker"). Schedule: an even, marker-free state, one ordinary C5 replacement, a capture
  that reads before and after. Red under 9 (`g` becomes odd with no marker, so the arithmetic contradicts itself); green under 10 (`g` counts
  replacements, the marker alone says an effect is in progress). The performance bound: the document holds non-terminal work only (model new
  "state document size independent of history").
- C0 and L1 to L4 (R9-1; models F2 and new "recovery helper"). Schedule: client A finds a dead marker, recovers, releases the lock and stays alive,
  service B acquires the lock and runs L2. Red under 9 (B ends A's scope); green under 10 (A starts a helper whose death ends its ownership; A is
  never an owner and is never in the scope L2 ends). Also "journal open" is defined; a publication attempt with a hook that never returns does not make
  C0 wait (model: disjoint acceptance during a hanging hook).
- Rule D, C3, C4 (R9-2; models F7 and new "visible but not durable at every boundary", `NineFsyncOnExists` is the inbox instance). Schedule: A links the
  binding and pauses before fsyncing `ids/<xx>/`; same-content resend B routes by it, completes C4 and answers; power fails. Red under 9 (the inbox
  entry survives and the binding does not, so another lane can bind the identity again); green under 10 (B fsyncs the binding's directory, the
  candidate and `incoming/` before answering).
- Rule N, M1 (R9-3; models new "inode reuse after an unnamed file dies" and "recovery racing pathname replacement"). Schedule: M1 creates an unnamed
  inode, the owner dies before linking, a foreign program creates `index.lock` and the filesystem reuses the inode number; or L4 compares, a cleaner
  removes our lock, a foreign program creates a new one, L4 unlinks. Red under 9 (the foreign lock is removed as ours); green under 10 (no removal of
  a shared name; the lock is reported). Rename-to-private (R9-S) is a control for the shipped branch (model new "namespace gap").
- MG1 to MG6 (R9-4, R9-5; models new "merge with unrelated staging", "switch before merge resumes", "capture between checked-out and swap",
  "restore without an editor", F5). Schedules: staged `y=1` while the merge changes `x` (red under 9: the whole-index install erases `y`; green: only
  `P`'s entries are replaced); a completed `git switch` then the lock frees (red: the main merge lands in the topic worktree; green: the attachment
  authority sends the merge to MG4 without a checkout); a capture after `checked-out` then a hook refusal (red: the capture holds the merge as
  staging and republishes it; green: `M` is in the logical tail with the checkout in one replacement); a refusal with no editor (red: reverse three-way
  can leave conflict markers; green: the pinned `N` is exchanged back and the bytes equal the pre-merge file).
- T1 and C5 (R9-6; model F8 extended). Schedule: T1 steps 1 to 3 pass, a run is admitted and views while the branch is elsewhere, the branch returns,
  T1 releases, the run deletes. Red under 9 (no root protects R); green under 10 (the hold root exists until step 6, so the earlier view includes
  it). A control with the root absent (`unrooted`) must keep the spool.
- Recovery classes R3 to R5 (R9-7; model F1 with new "erased suffix"). Schedule: the post-anchor entries removed so the log equals its anchor after a
  `swap-failed`. Red under 9 (the model's `reflog_intact` boolean decides); green under 10 (no oracle: R5 with its notice, except a live-observed
  `lock-held`).
- E1 (R9-8; model new "refusal then foreign move"). Schedule: the hook refuses, a foreign writer advances the branch before E1, E1 runs. Red under 9
  (`tip == old` fails, E1(d) calls crash recovery and the hook runs again); green under 10 (the class comes from the invocation's exit status and last
  line before any state read). R1 and R2 give `published`; R3 gives `rewritten-away`.
- B3 and T1-nocommit (R9-9; models new "held amendment adding and dropping paths", F9). Schedule: held `x=1`, amendment drops `x` and changes `y`.
  Red under 9 (the entry for `x` stays and suppresses staged `x=1` for ever); green under 10 (paths in the old set and not the new release; paths in
  the new set and not the old acquire). Identity routing: the amendment is a fresh identity; a resend of the superseded identity answers
  "superseded by".
- B1, Q1, P4, M1, publication target (R9-10; models new "every resume point", "hook-completion crash boundary", "conflict after acceptance",
  "signer and filter outcomes", "detached materialization"). Schedules: kill after each recorded `resume` value (red: Q1 re-enters the wrong
  transition for a checked-out merge; green: dispatch on `resume`); the hook finishes before `hook-ok` is durable and the owner dies (red: undefined;
  green: result file honoured, otherwise re-run once with a notice); a detached `HEAD` after the wrapper's own successful update (red: declined by
  comparing with the captured value; green: `new` or a descendant).


## Answers to the ninth review

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-9.txt`, with a closure line (closed in the model means closed in the revision 10 state
machine and owed an executable model in step 1; nothing is built). The verdict is accepted: continued model work and disposable backend experiments only.

- R9-1 (P1), C0 unlocks: accepted. Recovery is a helper process whose own death ends its ownership; the client is never an owner; nothing unlocks
  `lane/owner`. Closure: closed in the model; red control: C0 relinquishing while the client lives.
- R9-2 (P1), unflushed binding: accepted. Rule D for every link-before-directory-fsync boundary, with the dependency chain of a request. Closure:
  closed in the model; red control: the observation at each boundary, `NineFsyncOnExists` as the inbox instance.
- R9-3 (P1), inode identity is not deletion authority: accepted. Linux has no conditional unlink by identity, so rule N removes only names no foreign
  program creates, and M1 never removes `index.lock` (choice R). Closure: closed in the model for the lane; the shipped sweeper is separate.
- R9-4 (P1), whole-index install and attachment: accepted. MG2 replaces only `P`'s entries in the index read under the lock and every merge mutation
  has attachment authority (choices P and W). Closure: closed in the model; W leaves a stated micro-window.
- R9-5 (P2), capture window and reverse carry: accepted. `M` enters the logical tail in the closing replacement of MG3; MG6 restores from pinned
  pre-merge files and states the concurrent-edit residue. Revision 9's "never apart" is withdrawn. Closure: closed in the model.
- R9-6 (P1), late maintenance: accepted. An object-store root until T1's last step (choice G). Closure: closed in the model for the stated residue (a
  person rewriting the branch after the check); backend obligation: a prune view includes `refs/cli-git/*`.
- R9-7 (P2), R4: accepted. R4 is only a live-observed held lock; the F1 closure is withdrawn in place. Closure: not claimed closed at the observation
  level; the residue is stated as R5 with its notice.
- R9-8 (P2), E1: accepted. E1 classifies from the invocation's own result before reading any state; R1 and R2 give `published`, R3 gives
  `rewritten-away`. Closure: closed in the model; the diagnostic forms are a backend obligation.
- R9-9 (P2), B3: accepted. Fresh identity per amendment, path-set replacement, `superseded by` lookup, T1-nocommit. Closure: closed in the model.
- R9-10 (P2), execution gaps: accepted in full. `resume` points with Q1 dispatch, the hook result file and its stated crash exception to H-version,
  `conflicted`, `refused`, `sign-pending`, detached materialization, and the definition of an open journal. Closure: closed in the model.
- R9-11 (P3), generation arithmetic: accepted. The revision counter is separate from the marker.
- Section 5: all fourteen controls and the "must not assume" list are in step 1, with two added (state size against history; counter versus marker).
- Performance: the state document holds only non-terminal work and the unresolved suppressions, so a replacement is bounded by work in flight and not
  by history; the gate is the measurement "commit cost flat against ledger length".
- R9-S (production, handled separately): the design keeps X1 and states what it needs from the shipped rule (non-removal of ref locks until a primitive
  that cannot revoke a replacement lock is shown; rename-to-private is not one).
- Coordinator choices: C-kind accepted subject to the exact-invocation inventory (X1 text now says so); H-version accepted with the stated crash
  exception and without claiming it judges every published tree; M-A kept and repaired; B-restart withdrawal accepted and the internal-wait limit of
  B-hand stays stated.
- Not a defect: none. One refinement: the review's "unnamed inode can die" is answered by removing the reliance, not by pinning it, because the pin
  fixes inode reuse and leaves the pathname race.

## Answers to the tenth review

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-10.txt`, with a closure line (closed in the model means closed in the revision 11 state machine and
owed an executable model; nothing is built). The verdict is accepted: continued model work, pure components without reclamation, and disposable backend experiments only.

- Owner rule and answers (not a review finding): "nothing should ever wait forever or indefinitely"; at a bound "Return with an error"; "No, keep it for a retry (Recommended)"; pushes "Bound it, keep retrying (Recommended)".
  Applied as rule B, the register, W1 and W2, P2 in three steps, and the removal of every command-attached design that lands after the command returned.
- R10-1 (P1), M1 inference: accepted. Recovery re-reads the index and re-evaluates the recorded plan; lock presence or absence and a journaled intent prove nothing. Closure: closed in the model.
- R10-2 (P2), normal abort: accepted. A live creator releases its own lock by identity-checked unlink, with the premise stated; an abandoned lock is never removed. Closure: closed in the model; the premise is a modelled residue.
- R10-3 (P1), W-detect and MG6: accepted. Authority to undo is separate from authority to do; the check is repeated at MG4 and at every lane event; MG6 has reasons and resume points. Closure: closed in the model
  for the detected switch; the missed-check residue is stated and needs the owner's approval (choice W).
- R10-4 (P1), the hold root: accepted. The root is demoted to extra protection and the proof is the order connectivity check then registry re-read; `unrooted` is removed; publication never depends on the root. Closure: closed in
  the model for the stated residue; backend obligations: G1 durable before G2, no view before launch, deletion before `terminal`. Choice G is restated.
- R10-5 (P2), helper wait: accepted. One non-blocking try; the client re-evaluates; the wait on a live owner is tied to the marker and ends at MARKER. Closure: closed in the model.
- R10-6 (P2), candidate after retirement: accepted. The terminal route reads the ledger only; C5 applies rule D to the inbox record. Closure: closed in the model.
- R10-7 (P2), terminal outcome lookup: accepted. T0 appends the `outcome` line before the request leaves `lane/state`; T1 appends `retired` later and separately. Closure: closed in the model.
- R10-8 (P2), R4: accepted. R4 is withdrawn as a recovery class; after an owner's death a `swap-failed` is R5; the live retry is choice N (a concession to confirm). Closure: closed in the model for recovery; N is open.
- R10-9 (P2), MG5 and resume: accepted. MG5 plans over the union of old and new paths with its own starting-state rule; the table has `MG4n`, `MG5`, `MG6`; the pending merge does not block the commit that resolves it. Closure: closed in the model.
- R10-10 (P2), stderr provenance: accepted. Classification uses the report lines, Git's trace2 events and the exit status; standard-error text is shown and never parsed; an unclassifiable result withdraws a request that has a client.
  Closure: closed in the model; backend obligation: those channels exist in the assumed shapes and descendants cannot alter them.
- R10-11 (P2), hook reruns: accepted. The text says one new invocation per recovery episode; choice H2 offers a total bound. Closure: closed in the model; the runner lifecycle is a backend experiment.
- R10-12 (P2), detached type check: accepted. `--no-deref` is never used; the update follows `HEAD` as a native commit does. Closure: closed in the model; backend obligation: the old-value check through `HEAD`.
- R10-13 (P3), property and heading: accepted. L2 and L7 properties are separate; the heading says revision 11; "odd generation" narrative is updated or marked historical.
- Assessment of the five picks: R-never stands (normal abort is now defined separately); P-wait and W-detect are re-opened as choices under the owner's rule; G-root is demoted to extra protection with G-order carried; M-A stands as an
  ordering choice whose correctness now rests on MG5 and MG6 as repaired.
- Not a defect: none of the thirteen findings was rejected. One refinement: the review's "a person can delete the root" is answered by removing the reliance on the root, not by contract or coordination.

## Revision 11 changelog, moved from the design in revision 12

### Revision 11 changelog (for the step-1 models agent; every changed transition, with the model and the schedule)

Naming. Names are stable where the meaning holds. New names: rule B (the register of waits), W1, W2, T0, MG4n, `withdrawn`, `sign-failed` (was `sign-pending`),
the resume values `MG4n`, `MG5`, `MG6`. Bodies changed: P2 (now three steps P2a prepare, P2b decide, P2c commit), P4, Q1, E1, R4 (withdrawn), T1, M1, MG1 to MG6, C0, C3,
C5, B1, rule N, the Publication target rule and the Client result rule. Owner-question letters added: N and H2; P, W and G are restated. Each schedule goes red
under revision 10 and green under revision 11; a model marked "new" is added to step 1.
- Rule B, the register, W1, W2, P2, P4, Client result (the owner's bound rule and answers; models new "bounded wait" and "withdrawal versus landing", F2). Schedules:
  (1) a person's `reference-transaction` hook that never returns inside P2a and a client that reaches FOREGROUND: red under 10 (the command waits without end, or
  returns exit 0 and the commit lands when the hook finally accepts); green under 11 (W1 creates the decision `withdraw`, prepare returns later, P2b finds it,
  `abort` is written, the request never lands, the staging is as it was). (2) W1 and P2b race on `lane/decision/<id>`: both orders are explored; exactly one
  of `commit` and `withdraw` exists; a client that lost reports `published`. (3) a crash after W1 and before W2, and a crash between P2a and P2c: still
  withdrawn after restart; `update-ref` aborts a prepared transaction when its input ends (`builtin/update-ref.c:786-793`), so no prepared transaction survives. (4)
  every entry of the register reaches its bound in the model (START, FOREGROUND, MARKER, ACQUIRE, REAP, KILL, COMMITTING, STALL, PUSH).
- M1 and rule N (R10-1, R10-2; models new "intent without lock", "lock after an unfulfilled intent", "normal abort after lock creation"). Schedules: the owner dies
  after the intent and before `open(O_EXCL)`: red under 10 (absence of the lock is read as "renamed", M3(c) drops the entry, a plain commit reverses `x`); green under
  11 (the index is re-read, `E` found, the entry stays). A foreign lock present after an earlier intent: red (attributed to the dead incarnation); green (reported
  without a claim). A declined guard after a successful `O_EXCL`: red (undefined: "released" versus "never removed"); green (normal abort by the live creator, with
  the replacement race as a stated premise of every Git lock).
- MG1 to MG6, MG4n, resume table (R10-3, R10-9; models new "restore after a detected switch", "switch after the post-check", "retarget with own checkout entries",
  "retarget to a no-op", F5). Schedules: MG3 installs, the switch finishes, the post-check sees `topic`: red under 10 (MG6 needs authority to do, which has just failed, and
  ends `refused`, then proceeds to MG4); green under 11 (authority to undo, reason `switched`, `resume = MG4n`). A switch that finishes after the last check: red (files stay in the
  topic worktree with nothing scheduled); green (MG4 repeats the check and every lane event checks; the residue is stated and needs approval). Retarget after a lost
  swap: red (MG1's pre-scan reads the lane's own `x=1` as staged); green (MG5's own plan). A foreign successor makes the merge a no-op while `x=1` is checked out:
  red (`already` leaves the obsolete checkout); green (the union of old and new paths is reconciled). A waiting merge and a later commit of exactly the blocking paths:
  red (blocked behind it); green (proceeds ahead and resolves it).
- T1 (R10-4; models F8 extended, new "mixed-time view", "root deleted", "root not covering the replay"). Schedule: admission between the registry read and the check,
  the branch away at the run's view, the branch back, T1 releases: red under 10 (the root named only the captured closure; the check and the root removal were not
  ordered against admission); green under 11 (the registry is re-read after the check; the run is non-terminal in that read and T1 withdraws). Hold ref absent,
  deleted, or unwritable: no change in behaviour (it was never load-bearing), and publication is never blocked by it.
- C0 (R10-5; model new "helper loses acquisition"). Schedule: the helper's try loses to a service that then runs a hook that never returns: red under 10 (the client waits behind the helper, which waits in L1);
  green under 11 (one non-blocking try, the client re-evaluates and captures).
- C3, T0, C5 (R10-6, R10-7; models new "resend after candidate release", "terminal lookup before retirement", F7). Schedules: B1 records `refused`, the request leaves the document, a
  maintenance run keeps T1 from running, a resend arrives: red under 10 (answered "adopted"); green (the `outcome` line is in the ledger first). Resend after T1 released the candidate:
  red (rule D requires a deleted directory); green (the terminal route reads only the ledger). C5 meets a visible but unflushed inbox entry: red (adopts on it); green (rule D).
- R4, E1 (R10-8, R10-10; models F1 with new "live lock-held then satisfied and removed", new "hook descendant output"). Schedules: lock-held recorded, another writer installs and removes the
  commit, the reflog suffix is erased, the owner dies: red under 10 (R4 lands silently); green (R5 with the notice; the live retry is choice N). A hook descendant prints a held-lock line after Git's
  diagnostic and the hook refused: red (classified lock-held, the hook runs again); green (a non-zero `child_exit` of the hook in Git's event stream is `refused`; stderr text is never parsed).
- B1 hook runner (R10-11; model new "repeated deaths before a durable hook result"). Schedule: the hook performs its effect and the owner dies twice before a result file: red under 10 (the text promised a total bound of one rerun);
  green (the text and the model say one invocation per recovery episode, choice H2). Runner lifecycle: the writer of the result must not keep the scope non-empty (backend experiment).
- Publication target (R10-12; model new "HEAD attaches between check and P2"). Schedule: detached at `B`, another program attaches `HEAD` to a branch whose tip is `B`, P2 runs: red under 10 (`--no-deref` replaces the
  symbolic reference); green (no `--no-deref`; the update lands on the branch `HEAD` now names with the old-value check, M1 declines).
- Properties and heading (R10-13). The L2 and L7 properties are separated and the heading says revision 11; remaining "odd generation" narrative is updated.

## MG-A, the checkout-first merge transitions (retired in revision 13)

The owner chose "Branch first, files after" for the merge order, so these transitions and their revision 12 amendments are no longer part of the protocol.
They are kept as history; they are incomplete (they still wait for staged paths) and are not an implementable fallback.

- MG-A, the other order (if owner question 2 keeps checkout first). The revision 11 transitions MG1 to MG6 and MG4n
  apply with these amendments, each closing one gap of the eleventh review: (i) W1 and MG3's writes are made exclusive
  by an `flock` on `lane/decision/<id>.lock` that MG3 holds shared from its first write to its closing replacement and
  W1 takes exclusively with one try; a failed try means W1 waits for MG3 to finish, within FOREGROUND; (ii) a
  withdrawal whose MG6 meets a foreign `index.lock` cannot restore at the bound, so the command exits non-zero saying
  the merge's files and index remain until the lock frees, and `resume = MG6` finishes the restoration later: this
  weakens "staging as it was" and needs the owner's answer to question 2; (iii) the no-checkout continuation is its
  own transition (MG4n reached from MG2 with no pins), and Q1 resumes it without MG5; (iv) MG5 accepts its own
  checkout when the index equals `M`'s entry whatever the worktree holds (an edit is carried, not rejected), and every
  retarget keeps the first install's pins `N` and `R` and the pre-merge index entry `S` as the one undo root; (v) the
  staged-only version of MG3 above is saved before the first index replacement, and MG6 restores `S`, not `O`, when
  the merge is refused after a P-carry install. The revision 11 text of the transitions that the amendments modify,
  kept verbatim for option M-A (it names `MG1` to `MG6` and `MG4n` as in revision 11):
- MG1 compute. Pre: adopted, the merge request. Do: compute the merge commit `M` from the logical tail and the other side with `merge-tree` (an object
  write; no ref, index or worktree change; X1 may cancel it). Record in one S1 replacement `merge-ready <id> <M> <base>` with the path set `P`: for each path
  the merge changes, the tip's index entry `E_tip` and `M`'s entry. If `merge-tree` reports a conflict that no resolver settles, the outcome is `conflicted`
  (non-zero, the merge stops with the conflict recorded; nothing in the index or the worktree is changed by this record, and the spool is kept). Then the
  pre-scan, for a first computation only (a retarget uses MG5's own plan): one snapshot read of the real index (no lock); if the index entry of any path
  in `P` differs from `E_tip` (the person staged something at a path the merge changes), nothing is changed, the request waits with `resume = MG1` and the
  message names the paths (choice P). Ordering: a waiting merge of this kind does not block later requests, and a request that commits or unstages
  exactly the blocking paths proceeds ahead of it and is what resolves the wait (Q1 repeats MG1 when the index entry or the tip changes). Crash: repeated.
- MG2 take the index lock and plan. Pre: MG1 done with the pre-scan clear. Do: create `index.lock` with `O_CREAT|O_EXCL` (a single try); `EEXIST` makes the
  request wait with `resume = MG2`. With the lock held, read `HEAD`, the real index and `HEAD` again. If authority to do fails, release the lock by the normal
  abort of rule N; nothing is written; the merge is not checked out and goes to MG4 (it lands on the target ref only) with the message that the worktree was not
  touched because `HEAD` no longer names that ref. Else compute the plan: for each path in `P`, if its entry is not `E_tip` the person staged after the
  pre-scan: release the lock by the normal abort and wait with `resume = MG1` (nothing written). The new index is the index read under the lock with only
  the entries of `P` replaced; every other entry, with its stat data, is carried unchanged, so unrelated staging survives. Crash: `index.lock` may be left by a
  kill; it is reported and never removed.
- MG3 check out. Pre: MG2 holds `index.lock` with a plan. Do: S1 replacement setting the marker `materializing` and recording the plan; for each path in `P`, by
  the carry-forward protocol (journaled in `lane/carry/<path-id>`, temporaries in `lane/tmp/`, a displaced inode pinned under `saved/` by a link the lane owns),
  install `M`'s content in the worktree, merging the person's unsaved edit when the worktree file differs from the tip's content; write the new index into the lock
  file, fsync, rename it over the index. The closing S1 replacement clears the marker, sets `checked-out`, adds `M` to the logical tail and records for each path the
  consumption entry `install` with `E_tip` and the pins (`N` the displaced pre-merge file, `R` the installed file and its digest), so a capture between `checked-out`
  and the swap sees `M` in the logical tail and takes nothing of the merge. Post-check: read `HEAD`; if authority to do no longer holds, go to MG6 with reason
  `switched`. Crash: L4 re-evaluates every path from the journal and the pins by the rule of the previous M1 (observable file and index contents, not inference).
- MG4 swap. Pre: `checked-out` (or the no-checkout disposition of MG2/MG6) and the same check of authority to do repeated, because a switch can complete
  after the post-check; this check is also made at every lane event while the merge is `checked-out`. Failed: MG6 with reason `switched`. Do: P2 with `old = base`
  and `new = M` (P1 finds `M` already in the logical tail), P3, E1. Published: M3 (c) drops the entries, the request is `materialized` (`published` for the
  no-checkout case). Refused by a hook: MG6 with reason `refused`. Held lock or old-value mismatch: P4 with `resume = MG4`.
- MG5 retarget. Pre: waiting with `resume = MG4` and the tip moved. Do: the affected set is the union of the old set `P` and the new set `P'`, so a path
  that stopped being a merge change is still reconciled. Compute `M'` on the new tip (MG1 without its pre-scan). For each path in the union the target is `M'`'s
  tree entry for it. The lock and authority to do are taken as in MG2. The planned starting state of a path is the lane's own checkout (the index entry equals
  `M`'s entry and the file is the pinned `R` unchanged), or, for a path only in `P'`, the new tip's entry; any other entry is the person's staging after the
  checkout, left, reported, and the request waits with `resume = MG5`. A path is moved to its target by the carry-forward protocol with `M`'s content as the
  old side, so edits made since are merged. If `M'` equals the new tip, the paths of `P` are still returned to the new tip's content (the lane's checkout is
  reversed) and the request ends `already` only afterwards. The closing replacement replaces `M` by `M'` in the logical tail, replaces the entries by those
  keyed to `M'`, and sets `resume = MG4`.
- MG6 restore. Pre: reason `refused`, `switched`, `withdrawn` (W2) or `abandoned`. Do: the single try of `index.lock` (`EEXIST`: the request waits with `resume = MG6`
  and the reason recorded; the FOREGROUND bound applies); with authority to undo, for each path restore from the saved pre-state, not by a reverse three-way
  merge, which is not an inverse: if the path still holds the installed file `R` (the same inode as the pinned `R`, its digest unchanged), exchange it back
  with the pinned `N` (the person's own pre-merge file, bytes exactly as before) and set an index entry that equals `M`'s back to `E_tip`; if the path was written
  since, leave the person's current file and report it with the saved files (the concurrent-edit residue: the operation then does not claim to change nothing for
  that path, and no content is lost). The closing replacement removes `M` from the logical tail and drops the entries. Then by reason: `refused` ends the request
  `refused` with the hook's message; `withdrawn` ends it `withdrawn`; `switched` leaves the request live with `resume = MG4n` (publish the merge to the target ref
  without a checkout); `abandoned` ends it. A request captured while `M` was in the tail is relative to `M`; it is replayed on the new logical head (a three-way
  merge), and a conflict on a path the merge changed is `conflicted`, never a silent publication of the merge's content. Crash: as MG3.
- MG4n publish without a checkout. Pre: `resume = MG4n`. Do: P1 to P3 for `M` on the target ref only; the worktree is not touched; the notice says so.
  Residue of W-detect, stated: a switch that finishes after the last check leaves the merge's files in the new branch's worktree until the next check at a lane
  event or MG4, and then MG6 restores them (reason `switched`); until then they are visible there as local modifications. Nothing is lost.
