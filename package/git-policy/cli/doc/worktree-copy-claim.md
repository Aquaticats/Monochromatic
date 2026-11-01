# Worktree copy without a settlement lock

The worktree-copy feature copies a source worktree's ignored files into each linked worktree that a forwarded
`git worktree add` registers.
Until this change a per-repository settlement lock (`worktree_copy_lock.rs`: a directory published by rename, an owner
record, liveness checks, a 20 ms poll while a live owner held it, and an inherited lease token for nested wrappers)
serialized every creating command and every recovery pass.
The owner's requirements for the lock-free replacement ("Use as few locks as possible", "Thoughts on locks: I believe
they're an anti-pattern", and the reading of "Didn't I say locks are bad?" as no flock, no lock files, no owner records,
no polling) are quoted in `doc/planning/cli-git-resident-service.md`.
This document records what the lock protected, the replacement, and why it holds.

## What the lock protected

The lock was held from before real Git started until the copy finished, for `git worktree add` and `git worktree move`
only, and briefly by every other forwarded command that found a pending journal or stage claim.
Reading `worktree_copy.rs`, `worktree_copy_recovery.rs` and `worktree_copy_stage_claim.rs`, it protected five things.

- Attribution of new worktrees.
  The commands registered by one Git run are found by comparing the administrative directory names under
  `<common>/worktrees` before and after Git.
  With the lock held across Git, no other creating command could register a worktree inside that window, so the
  difference was exactly this command's worktrees.
- One copy per destination.
  Without the window, two overlapping creating commands both see both new names and both would stage and install into
  both destinations.
- A destination that is complete.
  A peer's worktree is registered before its checkout finishes (Git writes `worktrees/<id>/locked` with the text
  `initializing` first and removes it last).
  Installing ignored files into a half-checked-out worktree can make the peer's checkout fail on a path that is tracked
  there and ignored in the source.
- Recovery against a live copy.
  `recover_worktree_copy_transactions` resumes a journal and `reap_abandoned_stages` removes every stage container that
  a claim names and no journal names.
  The latter's own comment says it is safe only because the caller holds the lock: staging takes minutes between the
  claim and the journal, and a concurrent pass would delete a live stage.
- Recovery against recovery.
  Two passes would resume the same journal into the same destination.

The lock never protected a copy against `git worktree remove`: removal takes no lock.
That race is guarded by registration checks (recovery before this change, the claim holder as well after it).

## Replacement

Nothing waits, polls or records an owner.
Exclusion has two parts, a one-time claim and the kernel's own record of who holds a file open.

### The claim

A claim is one file per destination under the journal root, `<journal root>/<admin id>.claim` (the administrative
directory name is unique among registered worktrees of a repository).
It names the destination and the stage container the copy will use, and the PID namespace it was made in.
It carries no process identifier.

- Publication.
  The creator writes the content to a temporary file under a unique name, keeps that file open, then hard-links it to
  the claim name.
  A link never replaces, so exactly one creator wins; the others get `AlreadyExists` and do not wait.
  The file is open before the claim name exists, so a claim is never visible without a holder.
- The claim lives until the copy ends, not until the journal exists, and it is removed after the journal and the stage.
  Every durable state of a copy (stage, journal, install log) therefore has a claim that names it.
- Another process that finds the claim asks the kernel whether any process still has that file open (`/proc/*/fd`,
  compared by device and inode, the evidence `process_use.rs` already uses).
  A holder means the copy is live and the asker yields at once.
  No holder means the creator died; the asker may adopt the claim.
- Adoption is open-then-scan.
  The asker opens the claim, then scans again for any other descriptor on it, and acts only when none exists.
  Two askers that both open before either scans each see the other and both yield; if one scans before the other opens,
  the second sees the first.
  So at most one asker acts, with no rename race on the shared name, and the next run settles whatever both skipped.
- Settlement of an adopted claim is what recovery always did for the state it names: resume or discard the journal, or
  remove a stage no journal names, with the same notices.
  Then the claim is removed.
- Unprovable cases fail closed.
  A claim made in another PID namespace is judged by the age rule `pid_namespace.rs` already has; an unreadable process
  table, or an unreadable descriptor directory of the same account, means the claim is left alone.

### Everything else follows from the claim

- Attribution.
  Overlapping creating commands may both see a new worktree.
  Whoever wins its claim copies it, from its own source worktree; the other yields.
  A worktree whose `locked` file still says `initializing` is left to its creator.
  A marker `worktrees/<id>/cli-git-copied`, written when the claim holder finishes, makes later arrivals skip a
  destination that is already done, and Git deletes it with the registration.
- Recovery against a live copy.
  A recovery pass only acts on a claim that has no holder, and on a journal that no claim names (journals an older
  release wrote), which it first adopts with a claim of its own.
- The nested-wrapper lease is gone.
  A wrapper run by a hook inside `git worktree add` no longer has a lock to deadlock on, so `CLI_GIT_WORKTREE_COPY_LEASE`
  is neither set nor read.
- Removal.
  The holder checks that the destination is still a registered worktree of the repository after staging and before
  installing; if it is not, the stage and journal are discarded with a notice and nothing is created.
  Installation never creates the destination root itself, so a removed destination is not brought back.

## Known limits

- Two overlapping `git worktree add` runs from different linked source worktrees can copy a destination from either
  source, because the administrative directory does not say which command created it.
  From one source (the common case, the main checkout) the result is identical.
- A destination created with `--lock` keeps a reason in `locked` instead of `initializing`, and a localized Git writes
  a translated `initializing`; an overlapping peer cannot tell such a worktree is still being checked out.
- Another PID namespace's claim is judged by age (`FOREIGN_OWNER_PATIENCE`), not by process.
- Release builds of a version that still uses the settlement lock must not run copies at the same time as this version;
  a leftover `settlement.lock` directory and `*.stage` files from that version are not read and may be deleted.
