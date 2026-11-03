# Commits never fail: removing each environmental exit of `git commit` by construction

Scope: `package/git-policy/cli`, the commit path of the wrapper installed as `git` (`cli-git-native`)
and the resident commit service (`cli-git-service`).

The owner's rule, recorded verbatim in `package/git-policy/cli/SPEC.md`, "Merges and commits never fail":
"Commits should never fail."
The owner's correction (2026-11-03, verbatim): "never fail doesn't primarily mean retries!"
The owner's related rules, quoted: "Stop only on no progress", "You should (almost) never be waiting.",
"never discard user data", "commits vs native git should not take 5s longer.", "Use as few locks as possible.",
"Invoke real git as rarely as possible."

## How this record reads

The first version of this record, and the first code written from it, answered each exit with a longer wait, a bigger count
or a step started again.
The owner's correction says that is the wrong default: a failure that can happen is a design defect, and the design
removes it.
So each item below gives the root cause (why this can fail at all), the change that removes the failure mode by
construction, and, only where something outside the system remains, the residue: what happens then and why it cannot be
designed out.
Nothing past the red controls and the conflicted-path change is built; the coordinator and the owner review the design first.

What exists on the branch from before the correction, kept and not discarded:

- The planning record's first version (commit `6c8c686b2a`).
- A red control for item 1 (`53e40e9fc0`), a red control for item 3 (`7b7d9870f3`, written, not run).
- A retry-based first attempt at item 1 (`26db9e54af`): a Git step with no sign of work for its window (the push
  meter of `push_progress.rs`) is stopped and started again with a doubled window, and the object reader is restarted.
  Under this record it is at most the residue handling of item 1 (below) and is the owner's to keep or drop.

## Scope: what the owner settled

The owner settled which causes are outside the system's control (question tool, 2026-11-03).
This list is binding.

Outside our control: the wrapper may stop with a clear message, or retry where the cause can clear by itself;
work already handed over stays on disk and the next run finishes it.

- network down (no route, DNS, reset, refused);
- the remote refusing (login or auth, a server hook, GitHub's GH009, branch protection);
- disk full or quota;
- disk I/O errors or a read-only remount;
- a memory or process-count cap set by someone else;
- a kill (the OOM killer, a person), power loss or a reboot;
- a corrupt repository (missing objects, a broken index) caused by a disk fault or another tool;
- the person's signer failing (gpg or ssh-agent).

Not outside, so designed out: a slow remote; other programs' locks (an IDE or `/usr/bin/git` holding `index.lock` or a ref
lock); Git's own crashes or bugs.
Everything not listed as outside is designed out too, which includes the residues this record first called "a malformed
request" and "an object format this release cannot read" (see "Residues that are designed out", below).

Refusals that may still end a commit or merge with an error: policy errors at error severity; an empty message; a content
conflict no resolver settles (the merge stops with the conflict recorded).

Three more points were put to the owner and are decided (question tool, 2026-11-03):

- A live foreign program holding the branch's ref lock: "Save on a side ref (Recommended)". The commit is stored under a wrapper
  ref (`refs/cli-git/pending/<id>`), the command exits 0 and says the branch will show it once the other program lets go, the
  service then moves the branch onto it in order, and pending commits are pushed only after they land on the branch.
- The person's hooks and required filters: "Allowed refusals (Recommended)". A failing hook (`pre-commit`, `commit-msg`) or a failing
  required clean filter (`filter.<name>.required`) ends the commit with its own message and changes nothing, like a policy error.
  (Today the service route does not run configured hooks and warns that they never run; this decision covers the hooks and
  filters that do run, and does not change which hooks run.)
- A merge that would overwrite unsaved edits: "Carry edits forward (Recommended)". The merge lands, each edited file gets a
  three-way merge of the person's edit onto the merged result in the worktree, conflict markers hold both sides where they
  collide, nothing is lost, and a line names those files. `merge-isolation/paths-in-use` stops being a refusal.

The conflicted-file rule covers `commit -a` as well: `-a` stages the worktree copy of every tracked file, an unmerged file
included, so a `commit -a` over unmerged files commits the markers and prints the same warning for each of those files as the plain
commit does (owner: "Committing a conflicted path is fine. Warn and commit anyway"). Today `-a` commits them without a warning;
the fix lists the unmerged paths of the real index before the staging and warns for those it commits (built on the plain-commit
branch, `fix/git-policy-cli-plain-commit-finishes-conflicted-merge`, tip `a06af6102c`).

A commit that names a conflicted path is no longer a refusal.
Owner, verbatim: "Committing a conflicted path is fine. Warn and commit anyway, even if conflict markers will be committed."
That reverses `d8bcd95218`; it is built on `fix/git-policy-cli-commit-conflicted-path-warns` (tip `7ec22c34da`, for 0.5.26).

So item 7 of the starting list is now: policy errors, an empty message and an unresolvable conflict.
A named conflicted path commits with a warning, and a failing signer is outside.
`SPEC.md` says so.

Items 1 to 6 are designed out where the design below removes the failure, and listed as waits where it only moves it ("What is guaranteed to complete, and what may wait"); no wait is called designed out.
Item 4's moved-head questions were put to the owner and are decided (below).

## What the code shows about why these fail

Three facts of the present design explain most of the exits.

- A cap is a limit, not a mechanism.
  `START_WAIT`, `COMMIT_WAIT`, the class bounds, `LANDING_ATTEMPTS`, `REBUILD_ATTEMPTS`, the index wait of 1800 s and
  `RECONNECTS` are numbers somebody chose to end a wait; none of them detects a fault.
  A commit that fails on one of them fails although nothing is broken.
- The landing holds `index.lock` across slow Git work.
  `follow_up` in `service_land.rs` takes the lock, moves the branch, then builds the index with Git (`diff-tree`, then
  `reset`, in `service_land_follow.rs`) while holding the lock.
  The journal of 2026-11-03 shows that hold at 119 s (below).
  The commit is already durable when the branch moves; the index update is derived data (`build_index` copies the index
  beside the real one and installs it by the release of the lock) and is already finished later by recovery when it is
  not done.
- Work that Git does the wrapper could do in process.
  The wrapper has in-process readers for objects, trees, refs and the index (`repo_read_*.rs`, `repo_read_diff.rs`), and the
  commit-speed branch already compares the index with its base in process.
  Every Git child on the commit path is a chance to be slow for reasons that are not the commit's.

## Evidence for the 120 s commit

An observed commit on `main` (11:07:54Z on 2026-11-03) took exactly 120 s to prepare and 120 s to submit,
with `sync_ms` up to 119429.
The previous agent found no 120 s constant on the commit or landing path, and this record agrees:
the production 120 s constants are `QUERY_CEILING` (`bounded_query.rs`, auto-push queries only) and `CLAIM_LEASE`
(`service_work.rs`); `WAIT_CAP` in `binary_wrapper_store.rs` is test support.

What the evidence shows:

- The commit's timing record (`.git/cli-git-commit-timing.jsonl`, request `ac73f10b-a0a8-456a-91b7-efa8b8e75a25`)
  reads `totalMs` 299988, `prepareMs` 120013, `buildMs` 59790, `submitMs` 120009.
- The service journal (`journalctl --user -t cli-git-service`, instance process 1186934) answers the same request at
  06:07:54.042 local with `lock_hold_ms=119432` and `sync_ms=119429`: the service held `index.lock` for 119 s
  while it brought the index up to date (`follow_up` in `service_land.rs`).
- In that window the service printed wait notices for exactly two Git children, one after the other:
  `git diff-tree -r --name-only -z --no-renames <old> <new>` (process 1240607, a quick query step) started at
  06:05:54.6 and ended just before 06:06:54.0, which is under its 60 s class bound;
  then `git reset -q <new> --pathspec-from-file=- --pathspec-file-nul` (process 1246804, a bulk step) started at
  06:06:54.0 and ended at 06:07:54.0.
  These are the two Git calls of `build_index` for a commit that Git built (`IndexPaths::Derived`).
  So 120 s is two steps of about 60 s, not one 120 s limit.
- The record of 11:10:06Z shows the same shape on the client: `prepareMs` 119474 and `buildMs` 59785,
  and the record of 11:18:08Z shows `submitMs` 119768.
  Phases of one and of two times about 60 s recur in the slow records of that hour (the slow records are phases of
  50 s to 250 s), while a commit of another worktree of the same repository landed in 85 ms
  of `index.lock` hold at 06:06:01 and 06:08:23 local, so the host was not stalled as a whole.
- A later `git diff-tree --no-commit-id --name-only -r -z <oid>` of the review of that commit also waited
  55 s and more (journal at 06:08:03 to 06:08:53).

What Git 2.56.0's own source says about those two children (tag `v2.56.0`, clone `~/temp/agent/git-2.56.0-20261103`):

- `diff-tree -r --name-only -z --no-renames <old> <new>` takes no lock.
  It reads the whole index first, whatever the two trees are: `builtin/diff-tree.c:133` calls `repo_read_index` before it
  looks at its arguments (this is why the package classed it as reading the index).
  Nothing in it waits on a timer; its cost is the index load plus the tree objects, which on a saturated disk with a
  large index and several packs is read time.
  I could not find in its source a place where it waits about 60 s.
- `reset -q <oid> --pathspec-from-file=- --pathspec-file-nul` takes `index.lock` once and dies at once if it is held
  (`builtin/reset.c:491`, `LOCK_DIE_ON_ERROR`, which is a single try with no timeout; `lockfile.c:215` shows that a timeout of
  zero means one try).
  It then does two things: `read_from_tree` (`builtin/reset.c:197`, a diff of the tree against the index limited to the named
  paths, applied by `update_index_from_diff` at `builtin/reset.c:149`), and, unless `--no-refresh` was given,
  `refresh_index` over the whole index (`builtin/reset.c:500-505`, quiet because of `-q`).
  The refresh makes one `lstat` per tracked file, so its cost grows with the number of tracked files and the state of the
  disk's caches; the package's earlier measurement (`doc/planning/cli-git-commit-timeouts.md`) puts it at seconds on a
  warm disk for twenty thousand files.
  The wrapper never needs that refresh: the index it resets is a private copy it installs itself.
  So the `reset` step has a cause in Git's source for being slow (a full-index refresh) and a Git-provided way to skip it.
- Neither command contains a 60 s timer.
  The only 60 s timeout in the source tree is the fsmonitor daemon's start timeout (`builtin/fsmonitor--daemon.c:40`),
  which the owner's rule keeps out of scope: it was not set, unset, pinned or chased.
- Was the 60.0 s `reset` stopped by the wrapper's 60 s quick-class bound?
  No.
  `reset` is not in `PLUMBING_SUBCOMMANDS` (`child_class.rs`), so it is a bulk step with a 300 s bound, and the notice the
  journal printed for it says "a bulk step ... stopping it after 300 seconds".
  The `diff-tree` is a quick step with a 60 s bound and ended 59.4 s after it started, before its bound.
  Neither was stopped; both ended by themselves, so the coincidence of two near-60 s durations is not the wrapper's clock.

What stays unexplained: why each of the two took about 60 s on the main worktree and not on its sibling worktree.
The source shows what each does and that the wrapper's clocks did not fire, but it does not show a 60 s wait, and the
timing was not reproduced (reproducing it would need synthetic load, which is banned).
What the evidence supports, and nothing more: both steps are Git work proportional to the size of the index and the state of
the disk, both ran inside the lock hold, and the design below removes both from the commit path.
A child that took a few seconds longer than its 60 s class bound would have been stopped, and the stop ends the commit
(before the branch moves) or leaves the index update undone (after).

## Transaction model

Four cross-vendor reviews shaped this section.
The fourth found a data-loss window in the carry-forward crash rule, ambiguity in what publication evidence proves, retirement
dedup that expired, a lane-ownership rule that could kill a healthy owner, a watchdog that was a time bound and could repeat a
hook, lock sweeping by contents, a push exclusion that depended on an earlier expansion, and index intents that outlived their
authority.
This version removes the mechanisms those came from (the kill before acquire, the watchdog, lock sweeping after a crash, the
conditional push exclusion, the ledger horizon) instead of adding protocol.
It also drops every reliance on a resource limit of the service unit: the owner decided the unit sets none ("Remove all three
(Recommended)", recorded in `SPEC.md`), and a shipped limit would need a measurement or an owner decision beside it.
It is a design to be checked, not a proof; claims marked "to test" need models or fault-injection controls before code ships.

### Owner decisions of 2026-11-03 (question tool), with the scenarios shown

1. Stuck step, "End Git, hold hooks (Recommended)". Scenario: a Git child of a commit makes no progress; or a hook the person wrote
   is running. A Git child that makes no progress (no processor time, disk activity or output for a measured interval, the
   measurement recorded beside the number) is ended and its step redone. The person's hook is never ended: its commit is set aside
   as pending with a notice naming the hook and its process, and the other commits behind it keep landing on top.
2. Deferred staging, "Update the index (Recommended)". Scenario: a commit of `x` is published while another program holds
   `index.lock`; later the lock frees. The follow-up stages the committed content as Git's `git commit -- path` would. In the rare case
   of a deliberate restage to exactly the old entry inside a foreign index lock's window, that restage is replaced; this is
   accepted, and it cannot be detected. This is an owner-approved concession: the stat-equality comparison below is a conflict guard
   and not a provenance proof, because one index entry can be made again with the same fields by another staging operation.
3. Unclear publication, "Land it (Recommended)". Scenario: the owner died around the swap, the branch now shows neither the commit
   nor proof that it was published and removed (no reflog, or a reflog that lacks the entry). When the final traces cannot tell
   never-published from published-then-deliberately-removed, the service lands the commit with a message that names it and gives the
   command to drop it.

Adoptions from standing rules (the coordinator adopted them; the owner may veto; they are not owner answers):
- A deterministic Git hang proceeds toward an embedded implementation ("embed instead of spawning", "Invoke real git as rarely as
  possible", Git bugs are on the designed-out list); pending with a message is not an accepted end state.
- A signed commit whose parent moved is re-signed as the replayed commit in the background with the configured signer, as the service
  already signs its auto-push reconcile merges. A failing signer is on the owner's outside list: the request stays pending with a
  clear message and the work stays on disk.
- Saved copies of carried edits are retained until explicit disposal ("never discard user data"); the saved-copies age rule never
  deletes a sole recovery copy.
- Corruption by another tool after recovery data is released is on the owner's outside list ("Corrupt repository"). Maintenance the
  wrapper itself starts is coordinated over its whole lifetime, not admitted only at an idle lane.

### One owner per worktree, enforced by the kernel

The lane of a worktree (the process that publishes its requests and materializes them) is whoever holds an exclusive advisory
lock on `lane/owner` in the worktree's private service directory (an open-file-description lock).
There is no epoch, no fence ref, no takeover of a live owner, no signal sent to anyone before the lock is held, and no watchdog.

- Every child the lane starts inherits the lock's open file description (not close-on-exec) and is started with the parent-death
  signal set from a thread that lives as long as the owner. When the owner dies its children die with it, the kernel releases the
  lock, and the next owner simply acquires it. Descendants of a hook are children of a child and inherit the description too.
  Inheritance is not itself a lifetime proof; the implementation obligations are controls for: exec boundaries, hooks and their
  descendants, cancellation of the owner, and an inline owner (below).
- The lock covers the smallest unit (one worktree) and exists for exactly the owner's life. The reason it is needed (the
  repository's rule is lock-free first): no lock-free claim can tell a new owner that the previous owner and all its children are gone.
- Acquisition blocks with a notice that names the current owner; it never terminates the owner. A process-start-time match
  identifies a process, it does not give authority to end it.
- Inline ownership. A client that finds no owner takes the lock and acts as the owner only for as long as its own command lives:
  it drains the requests adopted before its own, publishes its own, retires it, and exits, leaving later arrivals in `inbox/` for the
  service. It is the person's foreground command, so the person can interrupt it (the request is already durable and the next owner
  finishes it).
- Descendant containment. Parent-death handling covers only the processes the lane starts. A hook can create descendants that close
  inherited descriptors or outlive their parent, so containment is a cgroup, not a descriptor:
  - a service owner runs in its unit's cgroup, which the manager ends on stop or restart;
  - an inline client owner moves itself into a transient scope created through the manager's bus interface the wrapper already speaks
    (`systemd_dbus.rs`) and starts every child inside it;
  - a descendant that keeps the lock description keeps the lane unavailable until it exits, but it is safe, because no new owner
    can mutate while it lives; a descendant that closed the description is contained by the cgroup;
  - a scope left by a dead inline owner holds only orphans (the owner process, recorded with its start time, is dead); the next owner
    stops that scope after checking that the recorded owner is dead, which is authority over a dead owner's leftovers and never
    over a live one;
  - where the manager is unavailable, an inline owner is allowed only while no hook is configured for the operation (no
    `reference-transaction` hook), so no unknown descendant can exist; otherwise the request stays handed over for the service.
  Controls: a hook that leaves a descendant holding the description, and one that closes it, with the owner killed at each phase.
- A wedged owner. No process takes over a live owner by identity, and the unit has no watchdog or limit (owner decision, recorded
  in `SPEC.md`). What remains is the owner's own rule for stuck steps (decision 1): a Git child with no progress for the measured
  interval is ended and redone, and a hook is never ended. A quiet owner of our own making is a separate case, put to the owner
  below.

One lock per worktree, and no other lock added, meets "Use as few locks as possible".
A service-only lane would remove even that lock by relying on the manager for single-instance and descendant lifetime; the reviewer
judged this an implementation choice, so it is not put to the owner.

### Durable acceptance needs no ref write

A command is accepted when its immutable request is durable in the worktree's service directory:
1. read the branch tip `B` (reading refs takes no lock) and read the base content the replay needs from `B` into the spool; if an
   object is missing because a foreign rewrite and a prune raced the read, read the tip again (nothing was handed over yet);
2. read the worktree and the real index and write the captured bytes into the spool;
3. fsync each spool file, fsync the spool directory;
4. write the request record to a temporary name, fsync it, rename it into `inbox/<id>`, fsync `inbox/` and its parent;
5. answer that the request is handed over.

### Adoption order, recovery and the logical head

`lane/order` is the authority on adoption. The lane appends `adopt <id> <crc>\n` (fsync) and then moves the record from
`inbox/<id>` to `queue/<id>`.
- After a durable `adopt <id>` the request is adopted even if its record still lies in `inbox/`; recovery completes the move and
  never appends a second `adopt`.
- A record in `inbox/` with no `adopt` line is adopted next, in the order of its capture time and id.
- Each line is self-framing (length-free fixed text with a checksum); recovery ignores a torn last line.
- The lane is the only writer, so there is no allocator to go obsolete.

Capture against the logical head.
A request records the logical head it was captured against (the newest pending commit as published in the lane's state, or the
branch tip when none is pending) and the delta (paths, new bytes, deletions, modes) relative to that base. At adoption the lane
replays the delta onto the actual logical tail by the three-way replay below. A base is never substituted under an unchanged tree.

The logical index (staged changes while requests are pending).
Pending requests are not materialized, so the real index keeps whatever it held, including entries the person staged for an earlier
pending request. The lane therefore keeps a consumption map, per path, of the real-index entry that an adopted request has already
consumed (for a named-path request, its pre-commit entry). A plain commit's staged change is the set of paths whose real-index
entry differs from the consumption map where one exists and from the physical head's tree otherwise.
Reviewer's sequence: the person stages `x=1` and plain request R1 becomes pending (consumed: `x=1`); named-path request R2 captures
worktree `x=2` (its pre-entry is `x=1`, consumed: `x=1`); the person stages only `y`; plain request R3 sees `x=1` equal to the
consumed entry, so `x` is not in R3's change, and R3 is `y` alone. If the person later stages `x=3` it differs from the consumed
entry and joins the change. The map entry of a path is cleared when the request that set it is materialized or voided.
The limit, stated: while requests are pending, staging a path back to exactly its consumed entry cannot express a revert of a
pending commit.

Pending requests are immutable versions.
An amendment is a new immutable request that names its target request and that request's commit `D1`. The one sequential lane
adopts it between steps:
- target unpublished: the amendment replaces it (the old version is dropped, the new one keeps its place);
- target published and still the tip: an amend of the tip;
- target published but no longer the tip (successors were published before the amendment was adopted): the amendment is stacked
  as a new commit on the tip, its change being the amend's tree relative to `D1` (the owner's moved-head rule), and it never
  amends an unrelated successor.

A branch switch while a request is pending.
The request records the worktree's attachment at capture (the branch, or detached). Publication is to the branch the commit was
made on, and is a compare-and-swap whatever the worktree has checked out now. Before any materialization the lane reads the
worktree's current `HEAD`: if it is no longer attached to that branch (switched, detached, branch deleted, worktree removed), the
materialization is voided, nothing is written to the current index, and a line says where the commit went and that the index of
the current branch was not touched. The materialization generation of the worktree (a counter in `lane/state` that changes on a
checkout of another branch observed by the lane) is recorded with the intent, and an intent from an older generation is voided.

Signed commits.
A signature cannot survive a changed parent, so the replayed commit is signed again. Adopted (above): the service re-signs it in
the background with the configured signer, as it does for the auto-push's reconcile merges. If the signer fails, which is on the owner's
outside list, the request stays pending with a clear message and the work stays on disk.

Capture consistency.
A request records, atomically with its capture, the lane's state version, the logical head, the consumption map it was computed
against, the physical head and the index it read. The lane's state carries a version number that changes on every adoption and
every materialization; capture reads it before and after reading the index and starts again if it changed. The delta is a set of
paths and bytes computed at capture; adoption never reinterprets it against a different map, it replays it. Capture time orders
requests by convention only and is not causal order between clients; tests assume no more than the protocol says.

### Publication and recovery

The publication is `update-ref refs/heads/<branch> <new> <old>`, a compare-and-swap, nothing else. Before the swap the lane writes
(fsync) `publishing <id> <attempt> <old> <new>` with an anchor of the branch's reflog: the byte length of the reflog file and a
hash of its last line (`anchor-absent` if there is none). After the swap it writes `published` (fsync).
Before its first publication on a branch the lane creates `logs/refs/heads/<branch>` if it does not exist (Git appends to a reflog
file that exists), so later updates are logged with their old values.

Recovery decides, for the latest attempt of a request, in this order:
1. The record says `published` (durable): published; a later deliberate reset is not a reason to publish again.
2. The new commit is the tip or an ancestor of the tip: published.
3. The anchor is intact (the reflog still has, at the recorded length, the line whose hash was recorded) and the bytes after it hold
   a line whose old value is the new commit: the branch held it during this attempt and a later update moved it away. The outcome is
   `rewritten-away`: not published again, kept under a hold and saved, and the person is told where.
4. The anchor is intact and the bytes after it hold no line whose old value is the new commit: the swap did not complete. The lane
   publishes again by replaying the captured delta on the tip.
5. The anchor is absent from the file (the reflog was deleted, expired or rewritten), or the reflog did not exist: the traces cannot
   tell a commit that was never published from one published and then deliberately removed. Owner decision 3 applies: the service
   lands the commit and prints a message that names it and the command to drop it (for example `git reset --keep <old tip>` while it is
   the tip, or `git revert <commit>`).

Correlation to the attempt, because commit identifiers can repeat: only reflog lines after the anchor count, so an older publication
and removal of an identical commit object (same captured fields) is not evidence about this attempt. A reflog line is never read
as proof of a swap that began; only an old-value line, which an update that moved the branch away writes, proves the branch held the
commit, and only a line after the anchor belongs to this attempt.
Nothing here claims once-only publication across repeated crashes and rewrites; each recovery decision uses only the evidence above.
Anything promised as saved (`rewritten-away`, a pending commit) is retained permanently until the person disposes of it
explicitly; the wrapper names the command.

### Retirement and permanent deduplication

When all obligations are done the lane retires the request: (1) append a terminal line to `lane/ledger`
(`<id> <outcome> <commit> <time>`, fsync); (2) release the retention of a published request (not of a saved one); (3) nothing else.
The ledger is append-only and has no horizon and no rotation. An entry is about a hundred and thirty bytes (a 32-digit identity,
an outcome word, a 64-digit object name, a time), so a worktree that commits a hundred times a day adds under five megabytes a
year; there is no size bound, so there is no limit to justify. A resend of an identity found in the ledger is answered from it and
never published. An identity found in neither the inbox, the queue nor the ledger is a new request; since the ledger never forgets,
a resend after any delay is recognized. The sequence "lost answer, long stop, later reset, resend" therefore cannot publish twice.
If a crash falls between (1) and (2) the ledger line exists and recovery completes (2); retention is never released before its
line is durable.
Obligations of the ledger and the order file: each line carries a checksum; recovery repairs a torn tail by truncating to the last
valid line before any append (ignoring it and appending behind it is not enough); the files are preserved by service upgrades and by
ordinary worktree operations (a worktree removal retires its lane by a final line and keeps the ledger with the Git directory); a
resend that reuses an identity with different content (the ledger and the inbox record both carry a hash of the immutable request) is
rejected with a message and published as nothing.

### Obsolete workers cannot exist

Every process that acts for the lane holds the lock's description and dies with the owner, so a new owner starts only when none is
left. Cleanup of a spool, a hold or a saved file is done only by the owner, and only for requests whose ledger line is durable.
To test: kill the owner at every phase, start the next owner at once, check that no child of the old owner is alive and that no
request is published twice; include a healthy inline owner met by a starting service (the service waits, nobody is signalled).

### Interrupted wrapper-owned transactions

The wrapper never removes a ref lock file after an owner crash, whatever its contents: a file with the expected bytes may be a
native writer's live lock created later, and a path check followed by a delete is a race. A lock left by a crashed wrapper child
is treated as a foreign lock: the request becomes pending, the notice names the file and says that if no Git process is running
it can be removed by hand. To make that rare, `update-ref` is never stopped by the wrapper's stall rule (a stopped writer is the
only way the wrapper itself leaves a lock), so a lock is left only by an owner killed in the milliseconds between creating and
renaming one.
For a child the live owner stops itself (the idempotent object writers, which take no ref lock), nothing is left to sweep.

### Retention lifecycle and the spool closure

The spool is the source of truth for the request until its ledger line exists (and permanently for a saved request). It is a
self-contained closure for replay: the captured bytes, the paths and modes, the expected pre-commit index entry of each path, the
content at the captured base of each path and the trees along each path, and for a conclusion, an amend or a revert the other
parent's commit and what the replay needs from it, the captured resolved tree, the message and the identities.
The lane imports the spool with one `fast-import` stream and, when the ref store allows, a hold ref
`refs/cli-git/<instance>/hold/<id>`; the hold is an extra protection and the spool is the guarantee.

The prune boundary, stated accurately. A request's objects can be lost to a destructive maintenance operation that took its
reachability view before the objects were referenced and deletes afterwards; in Git's source `git prune` removes unreachable
objects whose modification time is not later than its expiry, the default expiry being the largest time (`builtin/prune.c:36`,
`:98`, `:172`), so such a prune can delete objects written after it began. The boundary is whether a destructive operation can
outlive the retained recovery copy.
- Maintenance the wrapper starts (the service's automatic maintenance after commits, and a `gc`, `prune` or `maintenance` command
  forwarded through the wrapper) is coordinated over its whole lifetime: the lane records each run's start and end in
  `lane/maintenance`, a forwarded run starts only after the lane has recorded it, and no request accepted while a run is active, nor
  any request whose lifetime overlaps a run, releases its spool until that run has ended and a connectivity check after its end
  passes. Retention is extended, not the maintenance delayed.
- Maintenance another tool runs is outside the wrapper's control: a loss after recovery data is released is a corrupt repository
  caused by another tool, which is on the owner's outside list. The wrapper reports the missing object when it next reads it.

### Installing into the current index

Only a delta applied to the index read under the lock; never a whole-file install built earlier.
When the lock is free at publication time the lane takes it first (a single try), swaps the ref, applies the delta and releases it
in one critical section, so no index writer interleaves between the publication and its materialization.
When a foreign program holds it, the commit is published without the index and the delta is applied later, by these steps:
1. take `index.lock` with a single try; 2. read the current index in process; 3. check the worktree attachment and generation
(above), and that the intent is current (below); 4. apply the per-path rule; 5. write the lock file and rename it over the index.

Per-path supersession.
An outstanding intent is never applied on its own authority. For each path the lane composes the chain of unapplied intents of
published requests that touch it: the expected pre-state is the oldest intent's pre-entry, the target is the newest intent's
committed entry, and older intents' entries for that path are dropped. Reviewer's sequence (`R1` publishes `B`, `R2` publishes `C`,
pre-entry `A`, the person edits the worktree back to `B` without staging): the target for the path is `C`; the worktree holds `B`,
not `C`; so the entry stays `A`; no reversal is staged. A reachable ancestor is not authority to apply its transition.

The rule for a path: the target is the entry of the path in the **current branch tip's tree**, which is the effective content
whether the tip came from the wrapper or from a foreign successor; a wrapper intent supplies only the oldest pre-entry. With
`E_now` read under the lock, the oldest pre-entry `E_pre`, the target `E_target` and the worktree file's content: if `E_now` equals
`E_target`, nothing; if `E_now` equals `E_pre` in every field of the entry and the worktree file holds exactly the target's content,
stage the target (as `git commit -- path` would leave the index); otherwise leave `E_now` and report the path.
Reviewer's foreign successor: the index holds `A`, `R1` published `B`, a native writer advanced the branch to `C` whose version of
the path is `C`, the worktree still holds `B`: the target is the tip's `C`, the worktree is not `C`, so the entry stays `A`.

Provenance, stated as the owner decided it (decision 2): comparing every field of the entry, cached stat data included, is a conflict
guard and not proof of who staged it, because one entry can be made again with identical fields. The accepted concession: a deliberate
restage to exactly the old entry inside a foreign index lock's window is replaced; it cannot be detected; no content is lost.

The worktree attachment is checked and bracketed. While holding `index.lock`, the lane reads the worktree's `HEAD` and its
`logs/HEAD` length before it reads the index and again before it renames the new index into place; the materialization is declined
(the intent is kept for the report, nothing is written) if `HEAD` is not attached to the request's branch in both reads, if the
two reads differ, or if the `HEAD` reflog grew since the request was captured. Git's own `checkout` and `switch` write the index
first and update `HEAD` afterwards without holding the index lock, so a bracket cannot see the instant between them; that window
exists for native `git commit` too. In it the consequence is a change staged on the new branch's index that equals the carried
worktree content: nothing is lost and `git restore --staged` undoes it. This residue is a question for the owner below.

Extensions, per policy (not preserved wholesale): the cache-tree is dropped; the resolve-undo data and the untracked cache are
carried unchanged only after a differential control shows Git behaves the same afterwards (otherwise they are dropped or the index
is ineligible); the offset tables are not written; fsmonitor, split index, sparse index and unknown extensions make the index
ineligible, and the delta runs through Git on a copy made under the lock (`reset --no-refresh`). Index versions the writer does not
produce make it ineligible.

The wrapper's own index commands do not wait for the lane's intents behind a foreign lock.

### The side ref for a held branch lock, or a branch a foreign program keeps moving

Owner decisions (question tool, 2026-11-03): "Save on a side ref (Recommended)" for a held branch ref lock, and "Save on the side
ref (Recommended)" for a branch a foreign program keeps moving.
The publication is tried once and its failure classified from Git's message: the lock on the branch exists
(`unable_to_lock_message`, `refs/files-backend.c:859-866`; Git waits 100 ms by default, `refs.c:996-997`); the old value did not
match; no space, I/O error or read-only (the owner's outside causes); or other (kept `handed-over`, reported, retried when the
repository changes). An unchanged tip never counts as proof of a lock.
For a held lock or a lost swap the request becomes `pending`: the commit is built and in the spool and record, the command exits 0
and says the branch will show it when the other program lets go, and the lane publishes pending requests in adoption order when the
branch allows it. The side ref `refs/cli-git/<instance>/pending/<id>` is written when the ref store lets the lane; the record and
spool are the guarantee. Pending requests are not materialized.

Pushes.
Refs under `refs/cli-git/` are never pushed. The exclusion is unconditional, not derived from an earlier expansion: every push the
wrapper runs (the auto-push and a forwarded `git push`) carries the negative refspec `^refs/cli-git/*`, so a ref created between an
earlier dry run and the real push is excluded too. A control must show, on the installed Git, that the negative refspec is honored
for each invocation form: an explicit refspec, `refs/*:refs/*`, a configured `remote.<name>.push`, `push.default`, `--all`,
`--tags` and `--mirror`. For a form where it is not honored, the wrapper runs that push with the lane's best-effort ref creation paused by a marker the lane
acknowledges (the lane finishes or aborts a ref creation in flight, writes its acknowledgement, and creates none until the marker is
removed) and sends a frozen, filtered explicit update set: the invocation names every ref update itself, so it cannot expand a
wildcard, mirror, all-branch or tag selection again (mirror deletions are listed explicitly after one listing of the remote); the wrapper prints a line saying wrapper refs were left out and why (owner, "Leave out wrapper refs
(Recommended)").

### Replay of the captured change (owner's choice: stack on the new head)

Let `B` be the captured base (the logical head at capture, or for an amendment of a published commit, that commit `D1`), `T` the
captured resulting tree (or resolved tree for a conclusion) and `C` the tree to publish on. The new tree is the three-way merge
with base `B`, ours `C`, theirs `T` (`git merge-tree --write-tree --merge-base=B C T`, or an in-process equivalent that matches it).
A conflict is an allowed outcome: the replay stops with the conflict recorded and the captured inputs kept. The commit's parents
are `C` (and `T`'s other parent for a conclusion).

## What is guaranteed to complete, and what may wait

No wait below is called designed out.

Guaranteed to complete, with no wait on another program and no cap or count:
- acceptance: local reads and an fsynced write in the worktree's own directory, no ref write; failures are the owner's outside
  causes (a full disk, I/O errors);
- the answer after a crash or a long stop (a resend returns the record's state or the permanent ledger's outcome);
- retention of everything the request needs until its ledger line is durable, and permanently for anything promised as saved.

May wait, with a notice that names what is waited for:
- the command, on its own lane (a slow or wedged owner of our own making; the person can interrupt at no cost; the request is
  durable);
- off the command (it has exited 0): publishing a pending request while a live foreign program holds the branch's ref lock or keeps
  moving the branch; the index catching up while a live foreign program holds `index.lock` (owner: "Same as the ref lock
  (Recommended)"; a merge saves its result on a side ref, exits 0 and finishes the checkout when the lock frees).
The remaining waits are the person's own: an editor, a signer's prompt, a hook.

Stuck Git children and the person's hooks (decision 1).
A Git child that makes no progress (no processor time, disk activity or output) for the measured interval is ended and its step
redone: the object writers and `merge-tree` are content-addressed, and `update-ref` is a compare-and-swap that is safe to run twice
after the evidence rules of "Publication and recovery". The person's hook is never ended. A hook running as a descendant of a Git
child (a `reference-transaction` hook of the publication's `update-ref`, or `pre-commit` and `commit-msg` in a commit Git builds)
makes that child exempt from the stall rule while it runs. A commit whose build is held by such a hook is set aside as pending
with a notice naming the hook and its process, and the other commits behind it keep landing on top; a publication whose
`update-ref` is held by a `reference-transaction` hook holds the branch's ref lock, so the commits behind it find it held and become
pending in adoption order until the hook returns. Internal transactions on `refs/cli-git/` run without the person's hooks.
A deterministic Git bug that hangs the same input every time cannot be restarted away. The direction is adopted (above): embed the
operation instead of spawning Git for it, starting with the files-backend ref transaction and the object writes the lane uses, with
differential controls against Git; reftable and any operation not yet embedded keep using Git until embedded. Pending with a message
is not an end state. Embedding moves a hang into the owner, so the embedded code carries the same step journaling and redo.

Hooks and owner death.
When a `reference-transaction` hook is configured the lane writes `hook-dispatch <id> <attempt>` (fsync) before it invokes
`update-ref`. After an owner death with no durable `published`, recovery runs the evidence rules first and invokes `update-ref` again
only if they say the swap did not complete. Git's contract allows the repetition: the hook "executes whenever a reference
transaction is preparing, prepared, committed or aborted and may thus get called multiple times"
(`Documentation/githooks.adoc:486-488`), and a transaction that never reached "committed" had no effect to repeat. What cannot be
recovered is a "committed" notification lost with the owner after the swap, which a native Git crash loses too. The one case that is
not Git's contract is a hook that performs an external effect on "prepared"; whether recovery repeats it is a question for the owner
below. Hooks that ran in the client's own build (before handover) are not recovered: the person's command died with them and
nothing was handed over.

Measured stall interval (the shipped value needs a measurement beside it).
The classes of item 1 (60 s and 300 s) came from elapsed-time measurements (`cli-git-commit-timeouts.md`), which measure a different
thing from the quiet interval of the stall rule; they are not carried over as evidence. Plan, before the number ships:
1. add to the commit timing record, per step, the longest interval with no sign of work by the push meter (`push_progress.rs`) and
   whether a hook was running;
2. collect it across the hosts and loads the owner uses, healthy runs only (steps that ended by themselves);
3. record the distribution beside the number, with the host load average of each sample;
4. choose the interval above the longest quiet gap observed among healthy steps by a margin the owner approves, and record the
   measurement file and the margin beside the constant.
Until then no new number ships: the existing class numbers stay as the stall windows (the behavior of the retry-based attempt on this
branch) and are labelled unmeasured as stall intervals.

## Item 1: a Git step of the commit crosses its class bound

Root cause: the commit path starts many Git children (the quick class, 60 s, includes `diff-tree`; the bulk class, 300 s)
and a clock ends any child that outlasts its class, whatever it is doing.
A child that is slow on a loaded or saturated host is killed and the commit fails.
Two causes are stacked: the clock kills, and the steps are slow.

Change that removes it:

- Remove the kill.
  The watcher keeps its notices (it names the process and what it waits for) and stops ending children.
  A commit then has no time-based exit at all.
- Remove the steps.
  The slow ones in the evidence are the follow-up's `diff-tree` (old to new commit, to learn which paths changed)
  and `reset` (to put those paths in the index), and the review's `diff-tree`.
  The first is a tree comparison the wrapper already does in process (`repo_read_diff.rs`); the service has the
  request's change list for every commit it builds itself, and for a commit Git built it can compare the two trees
  with the in-process reader.
  The second is an index entry update for known paths, which is a write of the index file the wrapper can do
  itself (the index is copied beside the real one already), so the landing starts no Git for it.
  The goal is a landing that starts Git only for `fast-import`, `update-ref` and, for commits Git must build
  (signed, hooked, editor), the build itself.
- `diff-index --cached --quiet HEAD` over 50 s on main (reported by the coordinator, not measured here): the
  commit-speed branch replaces it by an in-process comparison of the index with its base; this record takes that over.
- Two more client steps measured on main by the coordinator at about 07:35 local on 2026-11-03 (installed 0.5.25, load
  average about 23): `git add --all -- toolchain.lock.json` (bulk class, process 3490505) waited over 50 s, and the commit
  before it waited over 20 s on `git read-tree --reset <head>`.
  From Git's source: `add` takes `index.lock` with a single try (`builtin/add.c:536`, `LOCK_DIE_ON_ERROR`), refreshes the
  index for the pathspec (`builtin/add.c:134`), runs the diff of the worktree against the index for the pathspec
  (`add_files_to_cache`, `read-cache.c:4027`) and writes the index; `read-tree --reset` takes the lock the same way
  (`builtin/read-tree.c:191`), reads the index and rewrites every entry from the tree.
  In both the work that Git does for one named file is proportional to the whole index.
  The change: hash the named path in process (with the repository's clean filters and line-ending rules, ported from Git's
  `convert.c`, and falling back to Git for a path with a filter the port does not cover) and write its entry into the
  private index in process; and skip `read-tree --reset` when the private index already equals its base tree, which the
  commit-speed branch already decides.
  Expected saving per step: `add`, the whole observed wait except the read and write of the index file the in-process
  writer still does once (no process start, no lock try, no full refresh, no untracked walk); `read-tree --reset`, all of
  the observed wait when the index equals its base, which is the common case for a path commit.
  These are expectations, not measurements: the observed waits came from a loaded host and I did not reproduce them.
  Before building, time each step in the fixture with `strace -f -ttt` as `cli-git-commit-timeouts.md` did, so the saving is
  measured and not asserted.
- The follow-up `reset` of the landing: put the named paths' entries into the index copy in process by porting
  `update_index_from_diff` (`builtin/reset.c:149`): for a path missing from the tree remove the entry, otherwise make a
  cache entry from the tree's mode and object (keeping a skip-worktree bit as that function does) and replace the entry,
  which also clears unmerged stages of the path.
  Where the port cannot decide (sparse index, sparse checkout), run Git with `--no-refresh`, which is Git's own switch
  (`builtin/reset.c:352`) and removes the whole-index refresh.
  The follow-up `diff-tree` becomes an in-process tree comparison (`repo_read_diff.rs`), which also never reads the index.

Residue: a Git child that never ends (a dead network file system under the repository, a process stopped by someone else).
No time bound can tell it from a slow one without risking the commit, and killing a writer in the middle of its write
can cost data, so the commit waits with a notice every interval that names the process, and the person (or the process's
owner) ends it.
The retry-based attempt on the branch (stop a read-only step that shows no sign of work, start it again with a doubled
window) can serve this residue for steps that only read; the owner decides whether it stays.
It also serves as the containment of a Git bug in a retained child that is safe to run twice (a hung `fast-import` or `commit-tree`);
`update-ref` is stopped and started again only when no `reference-transaction` hook is configured, because the compare-and-swap
is safe to run twice; with such a hook it waits, since the hook is the person's program (third review, "Transaction model, hung Git
children"). A deterministic Git bug on one input is narrowed there to a pending request and a message.

## Item 2: the service does not start within `START_WAIT`

Root cause: the commit depends on a second process that the manager must start, and the client gives it ten seconds.
The commit's own work is complete before the service is asked.

Change: the service stops being a correctness dependency.
The client hands the request over by writing the record into `inbox/` (durable, no ref write) and then asks the manager to start
the instance.
If no owner holds the lane lock, the client takes the lock and runs the lane inline for its own queue, until empty; a service
that starts later waits for that lock and takes the rest over. One lock per worktree, held by the kernel, replaces the ref fence,
the epoch and the takeover of earlier drafts (third review, "Transaction model").
If the owner is the service and it is alive, the client waits for its answer under item 3.

Residue: none that ends a commit.
Failures about the machine's setup (another account's service, a program that is missing, a unit file in the way) become
notices that the service is not running.

## Item 3: the service does not answer within `COMMIT_WAIT`

Root cause: silence is read as a fault, and a read timeout decides.

Change: the commit waits on the record, not on a connection or a clock.
The client reads the request's state (the service pushes it on the open connection and the client re-reads the record if
the connection closes).
The lane writes a phase counter into its state file after each phase, for notices only: a heartbeat shows that the sender runs, not
that the request moves, and nothing acts on the counter (no watchdog, no takeover; the unit sets no limits).
A closed connection is a resend with the same id, answered from the record or the permanent ledger.
A command whose request sits with an owner that makes no progress waits with a notice that names the phase and says how to
restart the unit; the person can interrupt it at no cost, because the request is durable and the next owner finishes it.
`WaitedTooLong` and `RECONNECTS` leave the commit path.

May wait: a service that is alive and slow makes the commit slow, and the notice names the phase and the process.

## Item 4: the branch keeps changing; a commit cannot replay onto a moved head

Root cause: the landing reads the tip, builds on it, then moves the ref only if it is still the tip
(`update-ref` with the old value), and counts failed rounds (`LANDING_ATTEMPTS` 3 in the service, 5 builds and 3 prepares
in the client).
A failed compare-and-swap means another writer moved the ref, which means the system as a whole made progress: the
compare-and-swap is lock-free, so some writer always succeeds.
A count turns that guarantee into a failure for the writer that happened to lose three times.
For a commit Git builds elsewhere (signed, hooked) the moved head also invalidates the work, because a signature covers
the parents.

Change that removes it:

- Remove the counts, and say honestly what remains.
  A commit built by the lane from a change list is rebuilt on the new tip with no cost to the person, and each lost
  compare-and-swap is another writer's commit.
  That is system-wide progress, not progress for this request: a compare-and-swap is not starvation-free (review finding 7).
- Make the moves rare.
  The lane of a worktree is the one writer of that worktree's publications: its commits, the push's reconcile merge and the
  merges it completes are items of the same lane, served in adoption order. Another worktree of the same repository publishing to
  the same branch is just another writer of the compare-and-swap (a branch checked out in one worktree is not a mutual-exclusion
  primitive: forced checkouts and native ref writes exist), and a published result is materialized only into the worktree of the lane
  that published it.
- For a commit Git must build (signed, hooked, editor) the build starts from the logical head and its publication is the compare-and-swap;
  a build whose base moved is replayed, or, for a signed commit that cannot be replayed, stays pending until the person signs again.
- A conclusion, an amend or a revert whose head moved while the person worked is built by replaying the captured change on
  the new head (owner's choice, below).

May wait: a foreign program that writes the branch with real Git directly, faster than a commit can be built.
Real Git offers only the compare-and-swap for a ref, so that program cannot be excluded; each lost round is that program's
commit, and the request is built again on it with a notice that names the foreign tip. Not starvation-free for one request.

### Decided: stack the result on the new head

The owner chose (question tool, 2026-11-03): "Stack on the new head (Recommended)".
Options that rewrote the other session's commits (rebuild on the new head), held the commit until the head was quiet, or
kept failing are dropped.

Setting: `main` is at commit `B`; another session lands `C` on `main` while this commit is built or while the person works.

An amend.
The person typed `git commit --amend -m "fix the parser"` on top of `B`, whose message was "parse the file".
Before: `A <- B (parse the file)`; the other session lands `C (add the lexer)`: `A <- B <- C`.
After: `A <- B <- C <- D (fix the parser)`, where `D` is the amend's captured change (`B` to the amend's tree) replayed on `C`,
so the lexer that `C` added is still in `D`'s tree.
`B` and `C` stay as they are, so no commit another session landed is rewritten.
Standard error says why: the branch moved, so the amend was made as a new commit on the new head.

A conclusion.
The person finished a merge of `topic` (tip `T`) into `main` on `B` with `git commit`; the other session lands `C`.
After: `A <- B <- C <- M`, where `M` has parents `C` and `T` and its tree is the person's captured resolution (`B`'s merge with
`T`, with their manual choices) replayed on `C`; the resolution is not recomputed from scratch, so the person's choices are kept.
A content conflict that the replay on `C` cannot settle stops it as any merge does, with the conflict recorded.

A revert.
The person ran `git revert B` and then `git commit`; the other session lands `C`.
After: `A <- B <- C <- R`, where `R` is the captured revert (`B` to the revert's tree, with any edits the person made while
preparing it) replayed on `C`, and stderr names the new head.

What the owner's choice leaves: an amend on a moved head is no longer an amend (`B` stays and `D` is added), so the person sees
two commits where they asked for one, and stderr says so.
This is the one place where the commit does not do what the words of the command mean, chosen by the owner.
The commit is built inside the worktree's lane from the captured inputs, not from the worktree.

## Item 5: `index.lock` is refused after the service's wait

Root cause: the landing needs the real index lock to install the updated index and holds it across Git work; a foreign program can
hold it for as long as it likes, and after 1800 s, or at once when no inotify watch can be set, the commit fails although the
commit itself did not need the index.

Change (owner, 2026-11-03, "Same as the ref lock"): the commit does not wait for `index.lock` at all.
The publication does not take it. The index catches up afterwards by the delta applied to the index read under the lock
("Transaction model, installing into the current index"), with a single try for the lock and a watch on its path set before the
try (re-checked on every wake, with a periodic re-check when the watch fails, so a missed event or a replaced file cannot stall
it).
The command exits 0 with a line saying the index will catch up when the lock is free.
No foreign lock is ever removed: a PID or an open descriptor does not prove the pathname still names an abandoned lock, and a dead
writer's lock can hold a complete index not yet installed.
The earlier plan to build the index off the lock and install the whole file is withdrawn, because it can erase another program's
staging.

May wait, off the command: the index catching up, for as long as a foreign program holds the lock. The commit is on the branch.

## Item 6: `ContentPruned`

Root cause: the objects a commit captured are unreferenced until the branch moves, and `git gc --prune=now` deletes every
unreferenced object at once.

Change: the retention lifecycle of "Transaction model": the spool is the source of truth until the ledger line exists; it is a
self-contained closure (new bytes, the base content the replay needs, the conclusion's dependencies); the lane imports it with
one stream and an extra hold ref when the store allows; a prune after handover is repaired by importing again; before handover the
client reads again, which is legitimate because nothing was handed over. No base hold ref is needed at capture (no ref write).
The earlier plan to prepare again from the worktree after handover is withdrawn: it would commit a later edit.

Residue: a prune that is still running across the release of the spool. Git's source shows it can delete objects written after it took
its view (`builtin/prune.c:36`, `:98`, `:172`); the lane checks connectivity before it releases the spool and imports again if
anything is missing, and a loss after the release is a corrupt repository caused by another tool (the owner's outside list). A
question for the owner.
The spool is on the same disk, so a full disk or an I/O error while writing it is one of the owner's outside causes.

## Merges, by the same lens (design only)

The merge-fixes branch waits for any `index.lock` holder before the merge and computes the merge again on a moved head up to
twenty rounds (`merge_isolation_moved.rs`).
Deferring the checkout past the publication is withdrawn (first review): a checkout after another commit descends from the merge
cannot be rolled back.
The order stays: compute the merge commit (no index), materialize the worktree and index under the lock, then swap the ref.

- Held `index.lock` (owner, "Same as the ref lock"): the merge takes the lock with a single try. If a live foreign program holds
  it, the merge commit is saved on `refs/cli-git/pending/<id>` (and in the spool), the command exits 0 and says the checkout will
  finish when the lock is free, and the lane does the checkout and the swap then. Nothing was published or checked out, so the
  person's worktree is untouched meanwhile; edits made in the meantime are carried forward at the time of the checkout (below).
  If the branch moved meanwhile, the merge is replayed on it as for any moved head.
- Held branch ref lock at the final swap: the checkout already happened; the merge commit is saved on the side ref, the command
  exits 0, and the lane swaps when the lock is free. Until then `git status` shows the merge's changes as staged against the old
  head.
- The twenty rounds: a lost swap is another writer's commit, so the count is artificial; the merge is an item of the worktree's
  lane (first in, first out with commits and the push) and a branch a foreign program keeps moving makes it pending.
- `merge-isolation/paths-in-use` stops being a refusal (owner, "Carry edits forward"). Per file that the person edited and the
  merge changes, under the lane lock, the index lock and before the swap (no obsolete worker can run it, because the lane lock is the
  only authority). Every displaced inode is retained, and every step is journaled in `lane/carry/<path-id>` (fsync before the next):
  1. open the path with `O_NOFOLLOW`, read its bytes `E`, keep the descriptor (inode `N`); a symlink, a directory or a special file is
     not merged as text (below);
  2. compute `R`, Git's three-way text merge of the old head's content `O`, `E` and the merged result `M` (`git merge-file`
     semantics, the repository's conflict style, labels naming the person's edit and the merge); create the temporary file beside
     the path, write `R`, fsync it, and journal `R = (device, inode)` of that temporary file (fsync);
  3. pin `N` with a hard link to the inode itself under `saved/<path-id>.1` (`linkat` via `/proc/self/fd/<n>`, same file system)
     and journal it;
  4. journal the intent `exchange temp <-> path` first (fsync), then do the `renameat2(RENAME_EXCHANGE)`: the path holds `R` and was
     never absent; the temporary name holds whatever inode was at the path at that instant;
  5. rename the temporary name to the next `saved/<path-id>.<n>` at once, so the displaced inode is a named, retained saved file
     (if it is `N` it is the same inode as `saved/<path-id>.1`); journal it; read it: if its bytes are `E` the merge is complete; if
     they differ, or it is another inode (an editor saved a new file between the read and the exchange), merge once more with those
     bytes as the person's file, write `R2` with the same journaling, pin and exchange again; the inode that `R` was is renamed to
     the next saved name. Two exchanges at most. Every inode involved is named under `saved/`; none is a disposable temporary;
  6. the index receives `M` for the path, so the person's edits are ordinary unstaged changes on the merged result; a conflict
     leaves markers with both sides, and a line names the file and every saved file.
  A bounded protocol cannot incorporate every concurrent write; by design it leaves additional versions in named saved files rather
  than lose one, and the line says so. Saved artifacts belong to the person's record and are retained until explicit disposal, never removed by retirement and never by
  the saved-copies age rule (`discarded_store.rs`), because a saved file can hold the only copy of an edit written through a
  descriptor that was still open; the line names the command that disposes of them.
  Recovery after a crash follows one rule: **a file is deleted only if its identity is an inode the lane created and journaled**
  (`R` or `R2`, by device and inode number), and never on the strength of a missing journal step.
  For an `exchange` intent without a recorded result, recovery looks at both names:
  - the temporary name holds the journaled `R` inode: the exchange did not happen; remove that temporary file (our own);
  - the temporary name holds any other inode: the exchange happened (an editor's newer inode `Q` is there); rename it to the next
    saved name, never delete it, and treat the path as holding `R` for the rest of the protocol;
  - the temporary name is absent and the path holds `R`: record it and report; remove nothing.
  Persistence ordering against power loss: create the temporary file, fsync it and its directory; journal `R`; create the hard link and
  fsync the saved directory; journal the exchange intent and fsync the journal; exchange, then fsync the directory of the path and the
  directory of the temporary name; rename the displaced inode into `saved/` and fsync both directories; only then journal the result.
  Recovery is identity-based, so it tolerates either order of the last two persisted effects. Directory durability on each file system
  (btrfs, ext4, xfs) is a verification obligation.
  This closes the reviewer's execution (the editor replaces the path with `Q`, the exchange happens, the process crashes before the
  result is recorded): recovery finds `Q` at the temporary name, which is not `R`, and keeps it.
  Where a hard link is not possible (another file system) or the exchange is not supported, the person's file stays at its path and
  the merged version is written beside it under a name the line gives; nothing is overwritten.
  A file that is not text, a mode or type change, a symlink, a directory in the way, or an untracked file colliding with a file the
  merge adds keeps the person's version at its path the same way.
  Supported semantics, stated: an edit through a descriptor that is still open is preserved in `R`, in `R2`, or in a saved file; an
  atomic-save editor that renames a new file over the path is treated as a new version of the person's file.
  To test: writes through an open descriptor before the first exchange, between the exchanges and after the second; a truncation; an
  atomic-save editor; a symlink at the path; a crash after each journaled step.

## Residues, by class

- A malformed request.
  Structural errors are detected before the record is written (the client builds it from typed values and validates it), and a
  record carries a format version: an owner that does not know a record's version does not process it and leaves it in `inbox/` for an
  owner of a release that does (a service of the newer release is started by the manager). A well-formed request that means something the lane cannot do is a policy decision, classified below.
- An object format or layout this release cannot read.
  Every in-process port is gated by eligibility (files or reftable ref backend, SHA-1 or SHA-256, no sparse or split index or
  unknown extension, attributes and filters covered), and every operation has a Git-command implementation complete for all
  layouts: the landing, the reconciliation and the recovery all run through Git when the gate says no. The ports (hashing with the
  repository's conversions, index delta, `update_index_from_diff`) are correctness boundaries, each with differential controls
  against Git on attribute precedence, required filters, encodings, line endings, index flags and stages.
  Failure independence is not claimed for the Git path: it meets the same locks and crashes, and those are classified as above.
- Local hooks and required clean filters: allowed refusals (owner); the commit ends with the hook's or filter's own message and
  changes nothing. The service route does not run configured hooks today and warns; this does not change which hooks run.
- The exit inventory is a gate, not a runtime default: before any code, every reachable `die` and `error` of each retained Git
  child (`fast-import`, `commit-tree`, `merge-tree`, `update-ref`, `reset`, `add`, `read-tree`, the commit Git builds) is listed
  from the 2.56.0 source with its class: outside the owner's list (clear message), designed out, or an allowed refusal.
  An exit that cannot be assigned is a design defect to report to the owner, not a nonzero exit left in place.

## Answers to the first review

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

## Exits that stay nonzero

- Policy errors at error severity, an empty message, and a content conflict no resolver settles (the merge stops with the
  conflict recorded, and a replay conflict keeps the captured inputs).
- A failing hook or a failing required clean filter (owner): the commit ends with its own message and changes nothing.
- The causes the owner listed as outside our control, with a clear message, and work already handed over finished by the next
  run.
- No longer here: a commit that names a conflicted path, a plain commit with unmerged files and `commit -a` over unmerged files
  (committed with a warning); a live foreign ref lock or `index.lock` (the commit lands or is saved, exit 0); a merge that would
  overwrite unsaved edits (the edits are carried forward).

## Questions for the owner

Answered on 2026-11-03: stuck step, "End Git, hold hooks (Recommended)"; deferred staging, "Update the index (Recommended)"; unclear
publication, "Land it (Recommended)"; earlier, a branch a foreign program keeps moving, "Save on the side ref (Recommended)"; a
`--mirror` push, "Leave out wrapper refs (Recommended)"; an amend while commits are pending, "Amend the pending commit (Recommended)".
Adopted from standing rules, not asked: the embedded path for a deterministic Git hang, background re-signing, saved copies kept until
disposal, corruption by another tool after release on the outside list, whole-lifetime maintenance coordination.

Four decisions remain that no standing rule settles. None is decided here.

1. Staging a path back to its consumed entry while commits are pending.
   Case: request `R1` (a plain commit) consumed `x=1` and is pending; named-path request `R2` set `x=2` and is pending; the real index
   still holds `x=1`. The person now wants to undo `R2` and runs `git add x` after putting `x=1` back in the file, then `git commit`.
   - Option A, as designed: the plain commit sees `x=1` equal to the entry already consumed and does not include `x`; `R2` stays.
     Visible: after both land, `x` is `2`; the person undoes `R2` with a named-path commit of `x`, and a notice on a plain commit with
     pending requests says staged paths equal to a pending request's old entry are not captured.
   - Option B, any plain commit with pending requests captures every staged entry that differs from the logical tail's content, stale
     ones included. Visible: the reviewer's earlier case (`R1` staged `x=1`, `R2` made `x=2`, the person stages only `y`) commits
     `y` and also reverts `R2` to `x=1`, which the person did not ask for.
   - Option C, a plain commit refuses to guess while requests are pending whose paths are staged: it is set aside as pending with a
     notice that asks the person to name the paths. Visible: no wrong commit, and an extra step for the person.
2. A quiet owner of our own making.
   Case: the service owner holds adopted request `R`, and for the measured quiet interval shows no processor time, disk activity or
   output, and no person's hook is running in its tree. A waiting command (the person's own `git commit`) sees it.
   - Option A, the waiting command asks the manager to restart the unit, using the same measured interval as owner decision 1; the
     steps are journaled and redone. Visible: after the interval the person's command prints that it restarted the service, and the
     commit lands. Risk: a healthy owner that is only slow is ended, and its step is redone.
   - Option B, never restart automatically; the command prints how to restart the unit and waits. Visible: the command waits until the
     person acts. This conflicts with "never stuck" if a silent wedge is possible.
3. A hook with an external effect on "prepared", after an owner death.
   Case: the person's `reference-transaction` hook posts a chat message when it sees "prepared"; the owner is killed after the post
   and before the swap; recovery finds the swap did not complete.
   - Option A, run `update-ref` again: the hook runs again and posts again; Git's contract allows this ("may thus get called
     multiple times"). Visible: two chat messages, one commit.
   - Option B, do not run it again: the request is set aside as pending with a notice naming the hook, until the person confirms.
     Visible: no second message, and the commit waits for the person.
4. The instant inside Git's own `checkout` where the index is written before `HEAD` changes.
   Case: the person ran `git switch topic` with edits carried across, and in the instant after Git wrote `topic`'s index and before it
   updated `HEAD`, the lane applied a deferred follow-up for a commit made on `main`.
   - Option A, accept the residue: the carried change appears staged on `topic`; nothing is lost; `git restore --staged` undoes it.
     Visible: a rare staged change on the new branch, which is also what a native `git commit` racing the switch would do.
   - Option B, decline every deferred follow-up whenever the index file was written by anyone since the commit was captured. Visible: the
     index lags more often (any foreign `git add` makes the follow-up wait for the person), and a notice says so.

## Verification plan and build order

Nothing past the red controls and the conflicted-file changes is built.
The order is the reviewer's, adopted:
1. Executable models and crash fixtures first: a model of the lane lock and the publication (kill at every phase, next owner at once),
   receipt-free recovery cases including a native writer and disabled or expired reflogs, delayed retries after retirement, and the
   adoption order after a crash.
2. Immutable request and spool components: serialization, validation, the complete replay closure, integrity checks, the fsync
   ordering, the ledger; during development retain everything and implement no reclamation.
3. Pure tree comparison and captured-delta replay, differentially tested against Git with explicit base, result and current inputs,
   changing no live ref, index or worktree.
4. A narrowly eligible index writer with the extension policy above, under-lock delta only, no checksum provenance.
5. Backend-specific publication experiments on disposable repositories: files and reftable, packed refs and `pack-refs`,
   `reference-transaction` hooks, linked worktrees, interruption at real persistence boundaries, the branch and `HEAD` reflogs.
Models to add from the fifth review: an inline hook descendant that keeps and one that closes the lock description, with the owner killed
at each phase; a partial or expired reflog and the anchor rule; a reused commit identifier; HEAD changing between the bracketed reads; a
foreign successor commit under a deferred intent; the ledger torn tail and resend with changed content; maintenance overlapping a
request's whole lifetime; power-loss ordering of the carry-forward effects on btrfs, ext4 and xfs; the stall-interval measurement of
"Measured stall interval".
Models to add from the fourth review: a healthy inline owner met by a starting service; a long phase and a hook-bearing owner restart
(no watchdog exists); forgotten ledger identities (there are none); adoption-before-move crashes; the mixed plain and named-path
pending sequence; an amendment whose target is no longer the tip; ref creation between a push's expansion and its execution; a worktree
attachment change with a request pending; composed deferred index intents; the carry-forward crash between the exchange and its journal
completion with an editor-created displaced inode, before any filesystem mutation code exists.
Not built from this revision: live carry-forward from the current journal, automatic retention deletion, the production pending and
materialization coordinator, until their models pass.
Also before building: the prune controls (starting before, during and after each phase, with and without `--expire=now`); the push
controls (`refs/*:refs/*`, `--mirror`, configured refspecs, the auto-push); the exit inventory from the 2.56.0 source; measured times
of each step with `strace`.
Each item then keeps its red control, then the change, then a guard shown red once.
