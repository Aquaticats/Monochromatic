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
  The worktrees registered by one Git run are found by comparing the administrative directory names under
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
It names the destination and the stage container the copy will use, the PID namespace it was made in, and whether its
creator can be seen in the process table.
It carries no process identifier.

- Publication.
  The creator writes the content to a temporary file under a unique name, keeps that file open, then hard-links it to
  the claim name.
  A link never replaces, so exactly one creator wins; the others get `AlreadyExists` and do not wait.
  The file is open before the claim name exists, so a claim is never visible without a holder.
- The claim lives until the copy ends, not until the journal exists, and it is removed after the journal and the stage.
  Every durable state of a copy (stage, journal, install log) therefore has a claim that names it.
- Another process that finds the claim asks the kernel whether any process still has that file open (`/proc/*/fd`,
  compared by device and inode, the evidence `process_use.rs` already uses, plus this process's own descriptors).
  A holder means the copy is live and the asker yields at once.
  No holder means the creator died; the asker may adopt the claim.
- Adoption is open-then-scan.
  The asker opens the claim, then scans again for any other descriptor on it, and acts only when none exists.
  Two askers that both open before either scans each see the other and both yield; if one scans before the other opens,
  the second sees the first.
  So at most one asker acts, with no rename race on the shared name, and the next run settles whatever both skipped.
- Settlement of an adopted claim is what recovery always did for the state it names: resume or discard the journal, or
  remove a stage no journal names, with the same notices.
  Then the claim is removed, last, so a crash inside the settlement leaves the claim for the next pass.
- What the process table cannot show fails safe.
  A creator in another PID namespace, or one that hides its descriptors from its own account (a sandbox that entered a
  user namespace, a program that cleared its dump flag), cannot be found holding anything, so its claim is judged by the
  age rule `pid_namespace.rs` already has (`FOREIGN_OWNER_PATIENCE`).
  Such a creator says so in the claim (`observable: false`), and such a process never adopts a claim itself, so every
  holder the scan has to find was observable when it took the claim; processes of the account whose descriptors cannot
  be read (a desktop session has several) are therefore no evidence against a claim.
  A process table that cannot be listed leaves the claim alone.

### Everything else follows from the claim

- Attribution.
  Overlapping creating commands may both see a new worktree.
  Whoever wins its claim copies it, from its own source worktree; the other yields with one notice line.
  A worktree whose `locked` file still says `initializing` is left to its creator.
  A marker `worktrees/<id>/cli-git-copied`, written when the claim holder finishes (installed or failed) or when a
  recovery pass ends a copy that reached its journal, makes later arrivals skip a destination that is already done; Git
  deletes it with the registration.
  A copy that died before its journal leaves no marker, so a surviving command fills that worktree.
- Recovery against a live copy.
  A recovery pass only acts on a claim that has no holder, and on a journal that no claim names (journals an older
  release wrote), which it first adopts with a claim of its own.
  Journals are listed before claims, and a journal is only ever written after its claim and removed before it, so a
  journal seen with no claim has no running copy.
- The nested-wrapper lease is gone.
  A wrapper run by a hook inside `git worktree add` no longer has a lock to deadlock on, so `CLI_GIT_WORKTREE_COPY_LEASE`
  is neither set nor read.
  A creating command whose own source is a linked worktree that Git still marks `initializing` (a filter or hook of an
  enclosing `git worktree add` that creates another worktree) only forwards, because that source holds nothing yet; the
  enclosing command, whose list of new worktrees includes the nested one, copies it from the real source.
- Removal.
  The holder checks that the destination is still a registered worktree of the repository after the claim and again after
  staging; if it is not, the stage and journal are discarded with a notice and nothing is created.
  Installation never creates the destination root itself, so a removed destination is not brought back.
- No second query for the repository's identity.
  The wait it guarded is gone.
  The before-set of administrative names is read after recovery, as close to real Git as possible.
- Journal listing tolerates a journal removed by its finished copy while another process lists, because copies run
  at the same time now.

## Why it is correct

Each claim below is a case of the one rule: a process acts on a destination's state only while it is the only process
that has the destination's claim open.

- Two copies into one destination.
  The link is the single decision point; the loser holds no claim and does nothing to the destination.
  After the winner finishes, the marker is written before the claim is removed, and a latecomer checks the marker both
  before and after winning a claim, so it cannot start a second copy.
- Copies into different destinations touch disjoint files: their claims, journals and stages have distinct names, and the
  only shared reads (listing the journal root) tolerate concurrent change.
- A copy killed at any instruction leaves a claim whose file nobody holds open, because the kernel closes a dead
  process's descriptors however it dies, plus whatever durable state it had reached; the claim names that state, and
  recovery settles it without anyone's help.
  A kill between the temporary file and the link leaves a temporary nobody holds, which a pass removes.
- A pass never touches a live copy: it finds the live copy's descriptor in the process table, or it does not open the
  claim at all.
- What a wrong guess costs, were the process table ever to hide a holder (a security module that denies the read):
  the intruder settles a live copy's state, the live copy's next step fails on a missing stage or journal, and the copy
  ends with a notice and exit status 2 with its rollback of paths it can prove; the destination is never overwritten
  because installation only creates absent paths and rollback only removes paths it can prove.

## Known limits

- Two overlapping `git worktree add` runs from different linked source worktrees can copy a destination from either
  source, because the administrative directory does not say which command created it.
  From one source (the common case, the main checkout) the result is identical.
- A destination created with `--lock` keeps a reason in `locked` instead of `initializing`, and a localized Git writes
  a translated `initializing`; an overlapping peer cannot tell such a worktree is still being checked out.
- A claim whose creator the process table cannot show is judged by age (one hour).
- Without `/proc` (macOS) no process is observable, so claims are never adopted before they are old.
- A worktree created by a hook through a wrapper whose enclosing source is already complete (a `post-checkout` hook)
  is copied from that source, not from the enclosing command's.
- Release builds of a version that still uses the settlement lock must not run copies at the same time as this version;
  a leftover `settlement.lock` directory and `*.stage` files from that version are not read and may be deleted.
- `CopyFacts.global_prefix` and `CopyFacts.environment` are no longer read; removing them touches callers outside the
  copy.

## Verification

The tests that pin the design are in `worktree_copy_claim_tests.rs` (publication, holder evidence from this process and
from another process that is killed, the second scan, the age rule), `worktree_copy_race_tests.rs` (two copies into one
destination, a copy against a removal, a copy killed at each durable state) and `binary_worktree_copy_tests.rs` (two real
wrappers overlapping inside their checkouts).
Each guard is a planted control in `bin/native-planted-controls-landing.ts`, and each control fails its test when the
guard is removed.
