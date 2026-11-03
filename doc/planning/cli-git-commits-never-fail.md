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

### Normative state machine (revision 9)

This is the one operative statement of the protocol. The narrative subsections that follow explain it and add rationale and test
requirements; where one of them reads differently, this section governs. Every transition names its preconditions (Pre), its durable
effects in order (Do, each followed by an fsync of the file and then its directory unless it says otherwise), and what recovery
does if the process dies at any point of it (Crash). "Journal" means a record the next owner can read. A transition never holds a lane
state generation, the lane lock, the gate or `index.lock` across the execution of a person's program, except where it says so.

Named entities. A worktree has one lane. The lane has an owner, an incarnation, a lock `lane/owner`, a gate `lane/gate`, a state file
`lane/state` with a generation, a journal, an order file `lane/order`, a queue, an inbox, and a spool per request. The common directory
`$GIT_COMMON_DIR/cli-git/` holds the identity bindings `ids/`, the ledger, and the maintenance registry `maintenance/`.

Request states: `drafted` (client only) -> `bound` -> `accepted` -> `adopted` -> (`held` | `ready`) -> `publishing` -> (`published` |
`pending` | `landing` | `rewritten-away`) -> (`materialized` | `voided`) -> `retired`; a hook may end a request as `refused`. A request
that a client abandons before `accepted` is removed by the lane after the client's process identity is gone.

Lane transitions (the owner and its incarnation)
- L1 acquire. Pre: none. Do: open `lane/owner` (a file created once and never replaced) with close-on-exec, wait for the exclusive lock.
  Crash: nothing to recover. Note: the wait ends when the previous owner and any child still between fork and exec have gone; a child
  between fork and exec holds the description until it execs or dies, so this wait is bounded only by that child's own progress.
- L2 reap. Pre: L1 done; `lane/incarnation` names a previous incarnation `P` that is not this process. Do: terminate the cgroup whose
  name contains `P`'s identifier (each incarnation has its own cgroup: a transient scope for an inline client, a delegated sub-cgroup of
  the unit for a service; the identifier is unique, so the target can never be this owner's own group) and wait until it reports no
  processes. Crash: repeat L2. Until L2 completes the owner mutates nothing. Never targeted: a live owner, a group not named `P`.
- L3 incarnate. Pre: L2 done. Do: write `lane/incarnation` (a fresh id, process identity, pid-namespace identity, the cgroup name),
  create that cgroup, move this process into it. Crash: the next owner reaps this incarnation by L2.
- L4 resume. Pre: L3 done. Do: complete or roll back any journaled mutation, repair torn tails of `lane/order` and the ledger, write an even
  state generation, then continue each request from its recorded state. Crash: repeat L4.
- L5 spawn a child. Pre: L3 done. Do: start the child inside the incarnation's cgroup with the lock and gate descriptions closed at exec
  and the parent-death signal set. Crash: a child between fork and exec holds the lock description until it execs or dies (L1 waits for it).
- L6 gate-shared. Pre: none. Do: take `lane/gate` shared before spawning any person's program (hook, filter, signer, helper) or any
  ref-transaction child; then read `lane/restart-intent` and, if it names this incarnation, admit nothing and exit (the check comes after
  the shared gate is held and before the dispatch, so a restarter that already holds the gate exclusively cannot be raced); keep the gate
  until the whole execution tree of that invocation has no processes (its own cgroup is empty), not until the immediate child exits. Crash: the kernel releases it; the program, if still running, was killed with the incarnation or
  is reaped by L2.
- L7 restart a quiet owner (client-triggered; shipped only with the measured interval; same pid namespace only). Pre: the owner's
  recorded pid-namespace identity equals this process's; no progress for the measured interval. Do: (1) open a pidfd on the recorded
  process and check its start time; (2) take `lane/gate` exclusively without waiting, abort if not taken; (3) re-read
  `lane/incarnation`, abort if it changed; (4) create `lane/restart-intent` naming the incarnation (fsync); (5) end the owner through
  the pidfd; (6) hold the gate until the owner is dead. Crash: before (4) nothing happened; after (4) the owner, when it next reaches an
  admission point (L6) and finds the intent, admits nothing and exits, so the gate failing open cannot admit a program; a later
  restarter may complete (5). No manager stop request is used, so no request can outlive the restarter. There is no restart without the exclusive gate: a restarter
  that cannot take it ends nothing, because a process-tree sample cannot show that no program will be dispatched a moment later. The
  earlier pick that restarted anyway (owner question B, option B-restart) is withdrawn for option B-hand; a different admission interlock that could end an internal wedge without
  ending a hook is open design work, not part of this machine.

Capture and acceptance (client)
- C1 capture. Pre: none. Do: read the branch tip `B` and the base closure from the immutable objects `B` names; read the lane generation
  `g` (wait for the lane if it is odd); read the logical head, the consumption map, the worktree and the real index; re-read `g` and the
  foreign inputs (the branch tip, and the index entries used); accept only an unchanged even `g` and unchanged foreign inputs, else
  repeat. Crash: nothing durable.
- C2 spool. Pre: C1. Do: write the spool files and the request record into `incoming/<id>.<hash>/`, fsync each, fsync the directory.
  Crash: an `incoming` directory with no binding belongs to a dead client and is removed by the lane after the client's identity is gone.
- C3 bind. Pre: C2. Do: write the binding content (hash of the immutable request and the path of `incoming/<id>.<hash>`) to a temporary
  file, fsync, then `link` it to `ids/<xx>/<id>` (atomic: complete or absent); if the name exists, read it (always complete): the same
  hash means resend, go to C4 if the request is not yet in the inbox, else answer from its state; a different hash is rejected with a
  message. Crash: a binding with a durable `incoming` request and no inbox entry is a reserved request: any same-content resend, or the
  lane at L4, completes C4.
- C4 accept. Pre: C3. Do: publish the record into `inbox/<id>` with a no-replace rename or hard link, fsync `inbox/` and its parent,
  answer `handed over`. Crash: repeated by C4 from the binding.
- C5 adopt (lane). Pre: accepted, no `adopt <id>` yet, L4 done. Do: journal the state change; write generation odd; append
  `adopt <id> <crc>` to `lane/order`; move the record to `queue/`; update the logical tail and consumption map; write generation even.
  Crash: L4 completes the move and the state update; a second `adopt` is never appended.

Build and publication (lane)
- B1 build. Pre: adopted. Do: build the commit by replaying the captured delta on the logical tail; where a person's hook is configured
  for the build, spawn it under L6 and mark the request `held` (it has no commit, it is not in the logical tail, its consumption entries
  are kept and are keyed by request and version). Crash: the build is repeated; a held request's hook is the person's program and is
  re-run by the repeated build.
- B2 ordering exception. Pre: `R1` held. Do: a request `R2` whose changed-path set is disjoint from `R1`'s (adds, edits, deletions, modes,
  directory-versus-file conflicts) and that is not a conclusion or an amendment of anything held proceeds ahead of `R1`; an overlapping
  `R2` waits. When `R1` completes: success re-enters it at the tail (replayed on the then-tip); refusal ends it as `refused` with the
  hook's message and changes nothing. A completion carries (request, version) and is accepted only for the active version; the
  completion of a superseded version is discarded. Crash: as B1.
- P1 publish-begin. Pre: `ready`, B2 clear. Do: journal `publishing <id> <attempt> <old> <new>` with the reflog anchor and, for a
  repository with a `reference-transaction` hook, `hook-dispatch`; the request is in the logical tail from here. No generation change.
  Crash: the next owner treats the attempt by the recovery classes (R1 to R5 below).
- P2 swap. Pre: P1 and L6 held for the ref-transaction child. Do: run `update-ref refs/heads/<branch> <new> <old>`. A person's
  `reference-transaction` hook may run inside it; nothing but L6 is held across it. Crash: recovery classes below.
- P3 published. Pre: P2 succeeded. Do: write generation odd, record `published`, update the state, write even. Crash: recovery class R2.
- P4 swap-failed. Pre: P2 failed with a held lock or an old-value mismatch. Do: record `swap-failed`, then the request becomes `pending`
  (side ref written best effort, the command already answered, the lane waits for the branch) or is replayed once if the old-value
  mismatch shows the branch moved (one operative rule: one try, then pending). Crash: recovery classes below.
- Recovery classes for an attempt, in this order. R1 a durable `published`: published. R2 the new commit is the tip or an ancestor of
  it: published. R3 a reflog entry after the anchor whose own recorded old value is the new commit: `rewritten-away` (saved, not
  republished); this is checked before any `swap-failed`, because a failed invocation does not say the request was never satisfied.
  R4 a recorded `swap-failed` and no R1 to R3 evidence: replay on the tip. R5 anything else: `landing`, owner decision 3: replay and
  land with a message naming the commit and the command to drop it.

Materialization and retirement
- M1 materialize (one path for every follow-up, whether the lock is free now or later). Pre: `published`, the attachment guard,
  and a single try of `index.lock` after the publication (never before it, never across P2). Do: with the lock held, read `HEAD` before
  and after reading the current index and decline unless it is attached to the request's branch in both reads (declined: M2); write
  generation odd; apply the per-path entry rule to the index read under the lock; write the lock file and rename it over the index;
  release the consumption entries of the paths installed; write even. Crash: the lock file of this process is removed by its own exit
  handling; L4 repeats M1 from the journal.
- M2 void. Pre: the guard declined, the branch was rewritten (the commit is not reachable from the tip), or a later foreign successor made
  the entry rule decline. Do: record `voided` for the request and the paths, release their consumption entries, tell the person.
  Crash: repeated.
- T1 retire. Pre: published (materialized or voided) or terminal otherwise, and no overlapping open maintenance entry whose scope is
  non-empty, and a connectivity check passed. Do: append the terminal line to the ledger, then release the retention of a request that
  is not promised as saved. Crash: the ledger line exists, the release is completed; retention is never released first.

Cancellation
- X1 cancel a stalled child. Pre: the child makes no progress for the measured interval and its command cannot launch a protected
  program (no hook configured for it; otherwise the child is exempt and waits). Do: send the terminate signal and a continue signal,
  wait the grace. Crash: the child dies with the incarnation. If the child ignores the signal, nothing further happens automatically:
  owner question A decides between ending it by force (leaving whatever lock it holds for the person) and never ending it (the
  request stays pending). The wrapper never removes a ref lock file, for any reason.

Maintenance (object-store-wide)
- G1 reserve. Pre: a destructive run is to be started by the wrapper (a forwarded `gc`, `prune` or `maintenance` command, or the
  service's automatic maintenance) and a manager is available. Do: create a transient scope `S` named by the run id and place the
  launcher in it; write `maintenance/<run>` (state `reserved`, naming `S` and the launcher's identity) by the link method of C3; the
  launcher inside `S` is the launch authority. Crash: see G3.
- G2 run. Pre: G1. Do: the launcher, inside `S`, starts the destructive process; entry state `running`. Crash: see G3.
- G3 terminal. Pre: `S` reports no processes and the launcher is dead or has recorded `end` after waiting. Do: record `terminal`.
  A missing end marker never means finished; an empty or absent scope without a dead launcher never means finished, because the live
  launcher inside `S` is still authorized to launch; the launcher's death with `S` empty proves launch can no longer happen. Crash:
  repeated from the registry.
- G4 no manager. Pre: no manager is available. Do: the wrapper treats the run as another tool's maintenance (the owner's outside list),
  prints that it cannot contain it, and retains the recovery copy of every request whose lifetime overlapped it until the person
  disposes of it explicitly, since an escaped descendant cannot be proved finished.

Narrowed guarantees stated by the machine, plainly. Acceptance waits for the lane's short state updates (odd generations never span a
person's program or a foreign lock) and never for a program. A quiet owner that holds the gate with no person's program running (wedged in
our own code between L6 and the end of a ref transaction) is not restarted automatically (owner question B, option B-hand, picked); the command prints how to
restart the unit and waits, and the request is durable. An owner wedged inside a person's program is never ended. L1 can wait for a child between fork and exec. G4 extends
retention at the cost of disk. The wait of a publication behind a person's `reference-transaction` hook is the person's.

### Rationale for the lane transitions L1 to L7 (explains; the state machine governs)

Why one lock, one gate. The lane of a worktree is whoever holds the exclusive advisory lock on `lane/owner`. The repository's rule is lock-free
first with an unavoidable lock on the smallest unit and its reason beside it; the reason is that no lock-free claim can tell a new owner that
the previous owner is gone. The proof is stated as the review asks: acquisition proves that no conflicting lock remains on that lock inode;
the previous owner's death follows from the enforced invariants, which are that only the owner process holds the description (children
close it at exec, and a child between fork and exec is waited for by L1), that the lock file is created once and never replaced, that the
owner never unlocks explicitly, and that the lane never executes a person's program in the owner process itself. The gate exists because the
restart of a quiet owner and the admission of a person's program must be serialized by something both sides share; a kernel lock is the
smallest such thing. These two locks are the only locks added, which meets "Use as few locks as possible" as the smallest set that gives these
two kernel-atomic facts.

Why an incarnation-specific cgroup. A unit's own cgroup path is reused by the replacement owner, so reaping "the recorded cgroup" can kill the
replacement. Each incarnation therefore has a cgroup whose name contains its unique id: a transient scope created through the manager's bus
interface the wrapper already speaks (`systemd_dbus.rs`) for an inline client, and a delegated sub-cgroup of the unit for a service (the
template unit enables delegation, which is not a resource limit). L2 targets only the group named by the recorded previous id. Containment
limit, stated: a hook can ask the manager to start another unit, and cgroup membership does not confine that work; that is outside the wrapper's
control. Where the manager is unavailable an inline owner is allowed only when the operation can start no arbitrary child program at all: no
configured hook, clean or smudge filter, signer or signing helper, credential or other helper; otherwise the request stays handed over for the
service.

Why inline ownership is narrow. A client that finds no owner takes the lock and acts as owner only to drain the requests adopted before its own,
publish and retire its own, and exit; later arrivals stay for the service. It is the person's foreground command, so the person can interrupt
it, and the request is already durable.

Why restart goes through an intent and a pidfd. A restarter that dies holding the gate releases it, so the gate alone fails open during an
incomplete cancellation; the durable `lane/restart-intent` makes it fail closed (an owner that finds the intent at its next admission admits
nothing and exits). A pidfd names the process it opened, so a delayed request cannot reach a replacement owner; a recorded pid is interpreted
in the right namespace only if the pid-namespace identities are equal, so restart is refused across namespaces. "The person's program" means
the whole execution tree of the invocation, so the owner keeps the shared gate until that tree's cgroup is empty; a daemon a hook leaves
behind keeps restart unavailable, which is the person's program being protected.

What restart cannot do. The shared holder case (the owner wedged inside our own code between L6 and the end of a ref transaction, with no
child and no person's program) cannot be restarted automatically, because the restarter cannot take the gate and ending the owner mid-transaction
could leave a lock. That is a remaining internal wait, stated; owner question B.

### Rationale for C1 to C5: durable acceptance needs no ref write (explains; the state machine governs)

Acceptance is the durable existence of a complete immutable request in the worktree's own service directory; no Git ref is written and no
foreign lock can stand in the way. The identity binding is published as a fully written file by `link`, so it is never empty or partial,
and it names an already-durable request, so a client that dies between C3 and C4 leaves a reserved request that any same-content resend or
the lane completes. A resend with different content is rejected because the binding is never overwritten or removed; the binding, the ledger and
the maintenance registry live in the common directory, so removing a worktree does not remove them, and lookups use the identity alone (a
resend of the same identity from another worktree is rejected as a mismatch). Capture reads the base closure from the immutable objects `B`
names; if one is missing because a rewrite and a prune raced the read, capture starts again, which is legitimate because nothing was
handed over. Capture waits for the lane's short state updates and never for a program; this is not wait-free, and it never waits on a foreign
program. Capture-invalidating generations are written only around lane state updates (C5, P3, M1) and never across P2 or a person's program.
The explicit red control is the reviewer's schedule: a reference-transaction hook waits for ever during P2, a disjoint request must still be
accepted.

### Adoption order, recovery and the logical head

`lane/order` is the authority on adoption. The lane appends `adopt <id> <crc>\n` (fsync) and then moves the record from
`inbox/<id>` to `queue/<id>`.
- After a durable `adopt <id>` the request is adopted even if its record still lies in `inbox/`; recovery completes the move and
  never appends a second `adopt`.
- A record in `inbox/` with no `adopt` line is adopted next, in the order of its capture time and id (a convention, not causal order).
- Each line is self-framing with a checksum; recovery truncates a torn last line to the last valid line before any append.
- The lane is the only writer, so there is no allocator to go obsolete.

Capture consistency: C1 and the seqlock rule of the state machine. The writer journals the intent, writes the generation odd, mutates, and
writes it even, and only for lane state updates; an interrupted writer is completed or rolled back by L4. Foreign writers of refs and the
index do not take part, so C1 re-reads the tip and the index entries it used; a foreign change that leaves them as read is undetectable and
is not claimed.

Capture against the logical head.
A request records the logical head it was captured against (the newest published-or-pending, non-held commit, or the branch tip when
none is pending) and the delta (paths, new bytes, deletions, modes) relative to that base. The delta is computed once, at capture; at
adoption the lane replays it onto the actual logical tail by the three-way replay below and never reinterprets it against another
map. A base is never substituted under an unchanged tree.

The logical index (staged changes while requests are pending).
Pending requests are not materialized, so the real index keeps whatever it held. The lane keeps a consumption map, per path, of the
real-index entry that an adopted request has already consumed (for a named-path request, its pre-commit entry). A plain commit's staged
change is the set of paths whose real-index entry differs from the consumption map where one exists and from the physical head's tree
otherwise. The entry for a path is cleared when the request that set it is materialized or voided.
Reviewer's sequence: the person stages `x=1` and plain request R1 becomes pending (consumed: `x=1`); named-path request R2 captures
worktree `x=2` (its pre-entry is `x=1`, consumed: `x=1`); the person stages only `y`; R3 sees `x=1` equal to the consumed entry, so R3
is `y` alone.
The limit, stated before it can surprise (question 1, picked A): while requests are pending, staging a path back to exactly its
consumed entry is not captured by a plain commit; a notice on a plain commit with pending requests says so, a plain commit whose
captured delta is empty ends as nothing to commit and never as a new failure, `commit -a` obeys its own rule (it captures tracked
worktree content explicitly, the settled conflicted-file behavior included), and a named-path commit undoes the pending change.
Optional later improvement, not part of this protocol: versioned restaging events. While requests are pending the wrapper, which forwards
`git add`, `git rm`, `git restore --staged` and `git update-index`, could record the resulting staged entry or deletion of each explicit
restage as a versioned, consumable event, written with the staging in a way a crash cannot invent or lose it, so that a plain commit
captures an explicit restage to the consumed entry. The sixth review's cases are the requirements, and the bare record of paths is
withdrawn as a candidate because each case breaks it:
- `git add x` with unchanged index content must still count (an index before/after diff misses it);
- `git add .`, `-A`, `-p`, `-N`, `--refresh` and dry runs: pathspecs are not the entries actually restaged, rejected hunks and stat-only
  operations are not content intent;
- `git restore --staged` records the resulting entry or deletion and its source, not worktree bytes;
- native Git, libgit2 and IDE staging produce no event and fall back to the limit above (no claim to detect all deliberate staging);
- editors that change only worktree bytes are not staging events;
- `git stash` and `stash pop --index` change the staged state without the four commands, and treating every internal index change as
  intent invents intent;
- `git commit -a` captures tracked worktree content as an explicit staging operation;
- ordinary aliases are normalized before classification; shell aliases and absolute native invocations are bypasses;
- an unconsumed event (an explicit restage of `x=1` used by `R3`, then a named request capturing `x=3`, then a plain commit for `y`) must
  not return `x` to `1`: events are consumed by the request that uses them;
- crash ordering: record first and crash before staging invents an operation, stage first and crash before recording loses it; an
  intent-then-effect-then-confirm journal with a recovery rule is required, especially for unchanged-content staging.

Pending requests are immutable versions.
An amendment is a new immutable request that names its target request and that request's commit `D1`. The one sequential lane adopts it
between steps: target unpublished, the amendment replaces it (the old version is dropped, the new one keeps its place); target
published and still the tip, an amend of the tip; target published but no longer the tip, the amendment is stacked as a new commit on
the tip, its change being the amend's tree relative to `D1`, and it never amends an unrelated successor.

A request held by a person's hook (owner decision 1; B1 and B2 of the state machine). A held request has no commit and is not part of the
logical tail. Its consumption entries are keyed by request and version and are released by one rule: when the request that set them is
materialized, voided or refused, never at publication (a published request whose follow-up is deferred keeps them, so a plain commit
staging only `y` does not capture a reversal of it). An amendment of a held request is a new version that names the held version, not a
commit: its immutable base is the held version's captured logical base and its delta replaces the held delta. The new version's build is
queued behind the old version's hook, since a running hook is never ended; when the old hook completes, its completion carries
(request, version) and is discarded because the version is no longer active, so a superseded version can neither publish nor change the
active consumption entries. The reviewer's cases are the ordering model: `R1` held with disjoint `R2`, overlapping `R2`, `R1` refusing
after `R2` landed, `R1` succeeding after `R2` landed, and an amendment of `R1` while its hook runs.

The attachment guard (the operative rule; there is no attachment generation).
A request records the worktree's attachment at capture (the branch, or detached). Publication is to the branch the commit was made on,
by compare-and-swap, whatever the worktree has checked out now. Before every index follow-up (M1 is the only path, so the guard applies to the immediate follow-up as well; a switch that completed before
the follow-up is not the accepted instant) the lane, holding `index.lock`, reads the worktree's `HEAD` before and after reading the index; it declines the follow-up (nothing written, a line says where the commit
went) unless `HEAD` is attached to that branch in both reads. This is a guard against the usual cases and not a fence: `HEAD`
can change after the last read without the index lock, and a change away and back is invisible. The reflog length is no condition: an
ordinary commit, including the publication being materialized, can grow `logs/HEAD`, so growth is neither a complete switch detector
nor evidence of a switch. The residue is the accepted unwanted staging of question 4 (picked A): only the index is changed, content is
preserved, the staging is reported and `git restore --staged` undoes it, and the concession is not reused to justify any worktree
mutation.

Signed commits (the operative rule).
A signature cannot survive a changed parent, so the replayed commit is signed again in the background with the configured signer, as the
service does for the auto-push's reconcile merges. A failing signer is on the owner's outside list: the request stays pending with a
clear message and the work stays on disk. The older rule of waiting for the person to sign again is withdrawn.

### Rationale for P1 to P4 and the recovery classes (explains; the state machine governs)

The publication is one compare-and-swap, `update-ref refs/heads/<branch> <new> <old>`, run with the gate shared. Before the swap the lane
journals `publishing` with an anchor of the branch's reflog read through Git (`git reflog show`, which covers reftable); the entry count and
the identity of the last entry, or `anchor-absent`. The lane creates `logs/refs/heads/<branch>` on the files backend if it is missing, since Git
appends to a reflog file that exists. The old value of an entry is read from the entry's own recorded old value; it is never inferred from the
adjacent displayed entry, because reflog editing can invalidate that inference. How the chosen Git interface exposes it on each backend is a
backend inspection.
The recovery classes R1 to R5 are ordered so that positive evidence comes before a failed invocation: a `swap-failed` record proves that one
invocation did not complete, not that the request was never satisfied, because another writer can install exactly the requested commit and
remove it after the failure. An intact anchor with no old-value line proves nothing about the suffix, since a legitimate reflog operation can
delete post-anchor entries; it falls in R5, unclear, which owner decision 3 lands with a message. Object-level observation is enough for R3
(the branch held what was requested). A reflog line with the commit as its new value is never read as proof of a swap that began or finished.
Nothing claims once-only publication across repeated crashes and rewrites. Anything promised as saved (`rewritten-away`, a pending commit)
is retained until the person disposes of it explicitly; the wrapper names the command.

### Retirement and permanent deduplication

When all obligations are done the lane retires the request: (1) append a terminal line to the ledger in `$GIT_COMMON_DIR/cli-git/` (stable, repository-owned; see "Durable acceptance")
(`<id> <outcome> <commit> <time>`, fsync); (2) release the retention of a published request (not of a saved one); (3) nothing else.
The ledger is append-only and has no horizon and no rotation. An entry is about a hundred and thirty bytes (a 32-digit identity,
an outcome word, a 64-digit object name, a time), so a worktree that commits a hundred times a day adds under five megabytes a
year; there is no size bound, so there is no limit to justify. A resend of an identity is answered from the ledger and never published; the permanent binding file `ids/<xx>/<id>` (created with
`O_EXCL` at acceptance) is the lookup that spans the inbox, the queue and the ledger, so an identity is never new once bound; since
neither the ledger nor the binding is ever forgotten, a resend after any delay is recognized. The sequence "lost answer, long stop, later reset, resend" therefore cannot publish twice.
If a crash falls between (1) and (2) the ledger line exists and recovery completes (2); retention is never released before its
line is durable.
Obligations of the ledger and the order file: each line carries a checksum; recovery repairs a torn tail by truncating to the last
valid line before any append (ignoring it and appending behind it is not enough); the files are preserved by service upgrades and by
ordinary worktree operations (a worktree removal retires its lane by a final line and keeps the ledger with the Git directory); a
resend that reuses an identity with different content (the ledger and the inbox record both carry a hash of the immutable request) is
rejected with a message and published as nothing.

### Rationale for X1 and the owner questions on cancellation (explains; the state machine governs)

The wrapper never removes a ref lock file: not after a crash, not after cancelling its own child, whatever its birth time or contents.
Creation inside a child's lifetime does not prove creation by that child (a foreign program can take the lock while the child is stalled before
acquiring it, or after the child released it and stalled before exiting), and a check of inode and birth time followed by an unlink is not an
atomic conditional unlink. The earlier rule that deleted a lock born during the child's life is withdrawn and is not to be implemented.
Git removes its own lock files when it ends on a signal it catches (`lockfile.c:83`, `tempfile.c:93`, installed at `tempfile.c:117`), so X1 sends
the terminate signal and a continue signal and waits the existing grace (`STOP_GRACE`, `child_watch.rs`). That is all it does automatically.
A child that ignores termination is the one case where the wrapper would have to kill what it started; killing a Git child that holds a ref
lock manufactures a lock the wrapper may not remove, and not killing it leaves the child stuck. That is a real choice (owner question A); until
it is decided the default is the conservative one: do not kill, leave the request pending with a notice that names the child and says it
ignored the signal. A command that can launch a protected program (for instance `update-ref` while a `reference-transaction` hook exists, or a
commit with `pre-commit` or `commit-msg` configured) is exempt from cancellation entirely, decided at spawn from the hooks directory, the hooks
path and the `hook.*` configuration, not from a sample of the process tree, because Git can dispatch a hook between the sample and the signal.
The restart of a quiet owner (L7) is unavailable while a ref-transaction child is in flight (the gate), so ending the owner can never leave a lock
that cancellation would not have left. After an outside kill (the OOM killer, a person) a lock may remain; it is treated as foreign, the request
becomes pending, and the notice names the file and says that if no Git process runs it can be removed by hand; that case is on the owner's
outside list.
Tests (red controls first): the retained-description deadlock in the fork-to-exec window; a closed-description descendant still writing when the
lock frees; the service cgroup reused by a replacement owner (the replacement must not kill itself); restart versus hook dispatch; restarter
death at every point of L7; a wedged shared gate holder; a pid read in another namespace.

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
  forwarded through the wrapper) is coordinated over its whole lifetime by one registry per object store, G1 to G4 of the state machine,
  not per lane. The reservation, the launch authority and the terminal state are separate: the launcher is placed in a per-run scope
  before the entry is observable and starts the destructive process from inside it, so while the launcher lives the scope is non-empty and
  an empty or absent scope is never read as finished; a run is terminal only when its scope reports no processes and the launcher is dead
  or has recorded its end after waiting; the launcher's death with an empty scope proves launch can no longer happen. A lane releases a
  request's spool only if no entry overlapping the request's lifetime is open and a connectivity check after the run passes. Git's own
  `gc.pid` in the common directory is read as a second source and also holds release. Where no manager exists (G4) containment is not possible
  (a descendant can leave a process group), so the wrapper treats the run as another tool's maintenance, prints that it cannot contain it, and
  keeps the recovery copy of every overlapping request until the person disposes of it explicitly: disk is spent, nothing is claimed.
- Maintenance another tool runs is outside the wrapper's control: a loss after recovery data is released is a corrupt repository
  caused by another tool, which is on the owner's outside list. The wrapper reports the missing object when it next reads it.

### Installing into the current index (rationale for M1 and M2; the state machine governs)

Only a delta applied to the index read under the lock; never a whole-file install built earlier. There is one path, M1, for every follow-up:
the lane tries `index.lock` once after the publication, never before it and never across the swap (a lock taken before the swap could be held
across a hook that waits for ever), applies the attachment guard, applies the entry rule, and renames the new index over the old. The earlier
two-path design (an immediate critical section around the swap, and a deferred follow-up) is withdrawn: the immediate path skipped the
attachment guard, and its only benefit, no index writer between the publication and its materialization, is exactly the by-value concession
the owner accepted in decision 2. Publication never waits for `index.lock` and never fails for it. The generation is odd only while the
index file is being replaced, which is local and short.

Per-path supersession.
An outstanding intent is never applied on its own authority. For each path the lane composes the chain of unapplied intents of
published requests that touch it: the expected pre-state is the oldest intent's pre-entry, the target is the entry of the path in the current branch tip's tree (the
rule below; the newest intent only says that the path is outstanding), and older intents' entries for that path are dropped. Reviewer's sequence (`R1` publishes `B`, `R2` publishes `C`,
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

The attachment guard is the one in "Adoption order": a guard against the usual cases, not a fence, with the residue accepted as question 4 (picked A).

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
Execution contract of the frozen set (sixth review): the update set fixes the source object identifiers as well as the names; wrapper refs
are excluded from updates and from mirror deletions; every option or configuration that would expand the selection again is removed or
neutralized for that invocation; force, leases, atomicity, deletion and tag semantics are preserved as the person's command meant them;
the pause marker is acknowledged by the lane, never by the lane waiting for itself; concurrent pushes share a reference-counted pause,
not a single flag. Whether the installed Git honors the negative refspec for each form, and how it interacts with default and configured
selection, are controls, not established facts.

### Replay of the captured change (owner's choice: stack on the new head)

Let `B` be the captured base (the logical head at capture, or for an amendment of a published commit, that commit `D1`), `T` the
captured resulting tree (or resolved tree for a conclusion) and `C` the tree to publish on. The new tree is the three-way merge
with base `B`, ours `C`, theirs `T` (`git merge-tree --write-tree --merge-base=B C T`, or an in-process equivalent that matches it).
A conflict is an allowed outcome: the replay stops with the conflict recorded and the captured inputs kept. The commit's parents
are `C` (and `T`'s other parent for a conclusion).

## What is guaranteed to complete, and what may wait

No wait below is called designed out.

Guaranteed to complete, with no wait on another program and no cap or count:
- acceptance: local reads and fsynced writes in the worktree's own directory and the common directory, no ref write, no wait on a
  program or on a foreign lock; it may wait for the lane's short state updates (an odd generation never spans a person's program or a
  foreign lock); failures are the owner's outside causes (a full disk, I/O errors);
- the answer after a crash or a long stop (a resend returns the record's state or the permanent ledger's outcome);
- retention of everything the request needs until its ledger line is durable, and permanently for anything promised as saved.

May wait, with a notice that names what is waited for:
- the next owner's acquisition, while a child of the previous owner is between fork and exec;
- a quiet owner that holds the gate with no person's program running (owner question B);
- the command, on its own lane (a slow or wedged owner of our own making; the person can interrupt at no cost; the request is
  durable);
- off the command (it has exited 0): publishing a pending request while a live foreign program holds the branch's ref lock or keeps
  moving the branch; the index catching up while a live foreign program holds `index.lock` (owner: "Same as the ref lock
  (Recommended)"; a merge saves its result on a side ref, exits 0 and finishes the checkout when the lock frees).
The remaining waits are the person's own: an editor, a signer's prompt, a hook.

Stuck Git children and the person's hooks (decision 1; X1 of the state machine).
A Git child that makes no progress (no processor time, disk activity or output) for the measured interval and whose command cannot launch a
protected program is sent the terminate signal and its step redone: the object writers and `merge-tree` are content-addressed, and
`update-ref` without a hook is a compare-and-swap that is safe to run twice after the evidence classes of P1 to P4. The person's hook is never
ended, and a command that can launch a person's program is exempt from cancellation altogether (decided at spawn, not by sampling), so it
waits. A commit whose build is held by a hook is `held` (B1, B2); independent requests behind it keep landing. A publication whose
`update-ref` is held by a `reference-transaction` hook holds the branch's ref lock, so the requests behind it find it held and become
pending in adoption order until the hook returns. Internal transactions on `refs/cli-git/` run without the person's hooks. A child that
ignores the terminate signal is owner question A. A deterministic Git bug that hangs the same input every time cannot be restarted away; the
direction is adopted: embed the operation instead of spawning Git for it, starting with the files-backend ref transaction and the object
writes the lane uses, with differential controls against Git; reftable and any operation not yet embedded keep using Git until embedded.
Pending with a message is not an end state. Embedding moves a hang into the owner, so the embedded code carries the same journaling and
redo, runs inside the shared gate, and is not itself a termination proof.

Hooks and owner death.
When a `reference-transaction` hook is configured the lane writes `hook-dispatch <id> <attempt>` (fsync) before it invokes
`update-ref`. It is conservative evidence that a hook may have run; it is not an admission interlock (the gate is) and it does not give
exactly-once effects. After an owner death with no durable `published`, recovery runs the evidence rules first and invokes `update-ref`
again only if the recovery classes say to publish (R4 a recorded `swap-failed` with no positive evidence, or R5 unclear, which owner
decision 3 lands). Git's contract allows the
repetition: the hook "executes whenever a reference transaction is preparing, prepared, committed or aborted and may thus get called
multiple times" (`Documentation/githooks.adoc:486-488`). The concession is stated directly (question 3, picked A): an external effect of
a hook on "prepared" may happen again after an interrupted attempt, and neither the journal nor the transaction state proves
otherwise. A "committed" notification lost with the owner after the swap is lost as it is for a native Git crash. This does not
authorize restarting a hook that is running now. Hooks that ran in the client's own build (before handover) are not recovered: the
person's command died with them and nothing was handed over.

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

- Replace the clock with a no-progress rule (owner decision 1).
  The elapsed-time kill is removed. The watcher keeps its notices and cancels a child only for the stall rule of X1: no progress for the
  measured interval, a command that cannot launch a protected program, the terminate signal first. A child that ignores it is owner
  question A. A command that can launch a person's program is exempt and waits.
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

Residue: a Git child that never ends. X1 cancels it with the terminate signal only when its command cannot launch a protected program;
one that ignores the signal is owner question A; one that can launch a person's program waits, and the wait is the person's. A deterministic
Git bug is handled by the embedding direction adopted in the transaction model, not by pending as an end state. The retry-based attempt on
this branch (a doubled window and a restart of the step) remains the redo half of X1 for the idempotent children.

## Item 2: the service does not start within `START_WAIT`

Root cause: the commit depends on a second process that the manager must start, and the client gives it ten seconds.
The commit's own work is complete before the service is asked.

Change: the service stops being a correctness dependency.
The client hands the request over by writing the record into `inbox/` (durable, no ref write) and then asks the manager to start
the instance.
If no owner holds the lane lock, the client takes the lock and acts as owner only to drain the requests adopted before its own and to
publish and retire its own, then exits (L1 to L4 and the inline-owner rule); a service that starts later waits for that lock and takes
everything else over. One lock per worktree, held by the kernel, replaces the ref fence,
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
that the request moves, and nothing acts on the counter (no unit watchdog, no takeover; the unit sets no limits; the one external supervisor is the gated client restart of "One owner per worktree").
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

- Remove the counts. One operative transition (P4): the swap is tried once; a lost swap or a held branch lock makes the request
  pending with a side ref and the command answers success, and the lane replays the captured delta on the tip when the branch allows it.
  Repeating the build in the command on every lost swap is withdrawn. A compare-and-swap is not starvation-free (review finding 7), so a
  request can stay pending while a foreign writer keeps winning; it is never lost.
- Make the moves rare.
  The lane of a worktree is the one writer of that worktree's publications: its commits, the push's reconcile merge and the
  merges it completes are items of the same lane, served in adoption order. Another worktree of the same repository publishing to
  the same branch is just another writer of the compare-and-swap (a branch checked out in one worktree is not a mutual-exclusion
  primitive: forced checkouts and native ref writes exist), and a published result is materialized only into the worktree of the lane
  that published it.
- For a commit Git must build (signed, hooked, editor) the build starts from the logical head and its publication is the compare-and-swap;
  a build whose base moved is replayed on the tip (B1), and a signed commit is signed again in the background with the configured signer (the
  operative rule; the older rule of waiting for the person to sign again is withdrawn).
- A conclusion, an amend or a revert whose head moved while the person worked is built by replaying the captured change on
  the new head (owner's choice, below).

May wait, off the command: a request pending behind a foreign program that holds or keeps moving the branch. The notice names the foreign tip.

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
The publication tries the lock once and never waits for it (optional index locking, "Installing into the current index"). If the
try fails the index catches up afterwards by the delta applied to the index read under the lock, with a single try for the lock and a
watch on its path set before the try (re-checked on every wake, with a periodic re-check when the watch fails, so a missed event or a
replaced file cannot stall it).
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

Residue: a prune another tool runs that overlaps the request. Git's source shows it can delete objects written after it took its view
(`builtin/prune.c:36`, `:98`, `:172`); the lane checks connectivity before it releases the spool and imports again if anything is missing, and a
loss after the release by a tool the wrapper did not start is a corrupt repository caused by another tool, on the owner's outside list. Maintenance
the wrapper starts is coordinated over its whole lifetime (G1 to G4), including its retention extension where no manager exists. The question the
review left about overlapping prune is therefore settled by the outside list plus coordination; it is not an open owner question.
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

History. Revision 8's normative state machine governs; where this section describes the seqlock across publication, the birth-time lock
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

Four decisions no standing rule settles were picked at 2026-11-03T09:45:54-05:00 by the coordinator, under the owner's delegation:
"Pick options as if I had picked them and invite veto when I say I'm back."
Clarified by the owner: "\"As if I had chosen\" means what I would have picked most likely if I'm always here."
Each pick stays open to the owner's veto; its reason follows the question's options.

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
   - Picked: A. The commit lands and nothing is lost: `x=1` stays in `R1`'s commit and in the file.
     B commits a revert the person did not ask for; C makes a commit wait on the person, against "You should (almost) never be
     waiting". A differs from native Git in this case, which would commit `x=1`.
     The sixth review judged the coordinator's candidate (record the paths the person restaged through the wrapper) not adoptable as a
     bare path set; it is withdrawn. A is kept as the documented fallback, and versioned restaging events are recorded in "Adoption
     order" as an optional later improvement with the review's cases as requirements.
2. A quiet owner of our own making.
   Case: the service owner holds adopted request `R`, and for the measured quiet interval shows no processor time, disk activity or
   output, and no person's hook is running in its tree. A waiting command (the person's own `git commit`) sees it.
   - Option A, the waiting command asks the manager to restart the unit, using the same measured interval as owner decision 1; the
     steps are journaled and redone. Visible: after the interval the person's command prints that it restarted the service, and the
     commit lands. Risk: a healthy owner that is only slow is ended, and its step is redone.
   - Option B, never restart automatically; the command prints how to restart the unit and waits. Visible: the command waits until the
     person acts. This conflicts with "never stuck" if a silent wedge is possible.
   - Picked: A, with the gate, the pidfd incarnation check and the in-flight ref-transaction exclusion of "One owner per worktree", and
     shipped only once the interval is measured as "Measured stall interval" plans, since a shipped limit cites a
     workload measurement or an owner decision beside its value. Owner decision 1 already ends our own stalled Git work and redoes
     the step, and the case excludes a person's hook, which is never ended; B leaves a command waiting on the person.
3. A hook with an external effect on "prepared", after an owner death.
   Case: the person's `reference-transaction` hook posts a chat message when it sees "prepared"; the owner is killed after the post
   and before the swap; recovery finds the swap did not complete.
   - Option A, run `update-ref` again: the hook runs again and posts again; Git's contract allows this ("may thus get called
     multiple times"). Visible: two chat messages, one commit.
   - Option B, do not run it again: the request is set aside as pending with a notice naming the hook, until the person confirms.
     Visible: no second message, and the commit waits for the person.
   - Picked: A. Git's contract allows the repeat, and a person who reruns a native `git commit` after the same kill starts a new
     transaction whose hook sees "prepared" again; B makes the commit wait on the person.
4. The instant inside Git's own `checkout` where the index is written before `HEAD` changes.
   Case: the person ran `git switch topic` with edits carried across, and in the instant after Git wrote `topic`'s index and before it
   updated `HEAD`, the lane applied a deferred follow-up for a commit made on `main`.
   - Option A, accept the residue: the carried change appears staged on `topic`; nothing is lost; `git restore --staged` undoes it.
     Visible: a rare staged change on the new branch, which is also what a native `git commit` racing the switch would do.
   - Option B, decline every deferred follow-up whenever the index file was written by anyone since the commit was captured. Visible: the
     index lags more often (any foreign `git add` makes the follow-up wait for the person), and a notice says so.
   - Picked: A. Nothing is lost and `git restore --staged` undoes the residue; the owner's "Update the index (Recommended)" chose a
     current index over a lagging one, and B makes the index lag after any foreign write.

Two decisions the seventh review leaves to the owner, picked by the coordinator under the owner's delegation (reasons beside each).

A. A Git child that ignores the terminate signal and holds a ref lock.
   Case: the service runs `update-ref` for `main`; it creates `.git/refs/heads/main.lock`, then stalls and ignores the terminate signal; the person
   has no reference-transaction hook.
   - Option A-keep, never kill it: the request stays pending with a notice naming the child and saying it ignored the signal; nothing is manufactured.
     Visible: the commit is saved and the command exits 0, `main` does not show it, and later requests behind it are pending too until the child
     ends or the person ends it. The default of revision 8 (revision 9 follows the pick: A-kill).
   - Option A-kill, kill it and leave its lock for the person to clear: the child ends; `main.lock` stays (the wrapper never removes a ref lock); the request
     and the requests behind it are pending behind a lock with no live holder, and the notice names the file and says to remove it if no Git runs.
     Visible: the same pending state, now with a lock file that needs a person's action, and the stuck child gone.
   - Picked (coordinator, 2026-11-03, under the owner's delegation, veto open): A-kill. The owner's "End Git, hold hooks (Recommended)" ends our
     own stalled Git and redoes the step; A-keep keeps a child the owner chose to end, with every later request behind it.
     The lock it leaves is then judged by the rule already shipped since 0.5.26 (`package/git-policy/cli/SPEC.md`, "Reference locks left by a Git
     that was killed"; `src/native/stale_ref_lock_judge.rs` and its siblings), which removes a lock only when its holder is proven gone and
     otherwise prints what it found and the command that clears it. That rule contradicts this revision's "the wrapper never removes a ref lock";
     the eighth review is asked whether the shipped proof survives the seventh review's foreign-lock schedule. If it does not, A-kill stands as
     written (the notice names the file for the person), and the shipped rule is reopened too.
     Outcome of the eighth review (2026-11-03T11:33:50-05:00): the shipped rule does not survive (S-1, S-2; fixed separately on
     `fix/git-policy-cli-never-removes-a-live-lock`), so A-kill stands as an explicitly accepted narrowing: ending a stalled Git child that
     ignores the terminate signal can leave a lock that only the person clears, and the request behind it stays pending, with the notice
     naming the file. This does not remove that exit by construction; it trades a stuck child for a named lock.
B. A quiet owner that holds the gate with no person's program running.
   Case: the service owner took the shared gate for a ref transaction, wedged in our own code before launching the child (or after the child exited,
   before releasing the gate); no hook runs; a waiting `git commit` sees the quiet interval pass.
   - Option B-hand, do nothing automatic: the command prints how to restart the unit and waits; the request is durable. Visible: the command waits until
     the person restarts the service; this conflicts with "never stuck" if such a wedge occurs.
   - Option B-restart, restart anyway: the waiting command ends the owner without the gate and the manager restarts the service. Visible: the commit lands after
     the restart, but if the owner was inside a ref transaction with a child, ending it can leave `main.lock` that the wrapper will not remove, with
     the pending state of A-kill.
   - Picked first (coordinator, 2026-11-03, under the owner's delegation): B-restart, withdrawn after the eighth review.
     The eighth review (R8-1) showed B-restart can end a person's hook: the restarter's correct "no program running" observation is followed by the
     owner dispatching a ref transaction whose hook starts, and the restart then reaps it. That breaks the owner's explicit
     "End Git, hold hooks (Recommended)" (a hook is never ended), which outranks the general "never leak internal things".
   - Picked now (coordinator, under the delegation, veto open): B-hand. L7 keeps its gate requirement; a wedge of our own code inside the shared
     gate is reported with how to restart the unit, and the request stays durable. A different admission and cancellation interlock that
     can end such a wedge without ending a hook is open design work for revision 9.

## Verification plan and build order

Nothing past the red controls and the conflicted-file changes is built. The sixth review's verdict governs: step 1 is executable models and
disposable crash fixtures only; the live protocol, automatic restart, production pending publication, live carry-forward and retention
deletion are not enabled until those models establish a coherent protocol.

Step 1, in this order (every model starts as a failing control that preserves the review's schedule):
1. Failing models for the findings, each a red control first:
   - F2: the retained-descriptor scope deadlock; the closed-descriptor descendant still writing when the lock frees;
   - F3: a restart observation, a hook dispatch after it, and a restart that targets a replacement incarnation;
   - F4: a quiet `update-ref` cancelled, the owner restarted during it, the lock left behind, then every future publication pending;
   - F5: `HEAD` changed after the last guard read, and changed away and back;
   - F6: the reviewer's schedule with `x=0`, `x=1`, `x=2` and a materializer paused before it advances the generation;
   - F7: the same identity with different content from two clients, one before adoption and one after;
   - F8: a destructive prune launched from one worktree while another worktree accepts, imports, publishes and releases;
   - F1 as a control too: the intact anchor with the post-anchor entries deleted, and the identical commit written by another writer.
2. The hook-bypass ordering model (F9): held `R1` with independent `R2`, overlapping `R2`, `R1` refusing after `R2` landed, `R1` succeeding
   after `R2` landed.
3. The immutable request, spool and permanent-ledger components with no reclamation: serialization, validation, integrity, fsync ordering,
   torn tails, resend with changed content, upgrade recovery; retain everything.
4. Pure tree comparison and captured-delta replay with explicit base, result and current inputs, changing no live ref, index or worktree;
   including the mixed plain and named-path pending sequence and an amendment whose target is no longer the tip.
Models added from the seventh review (each starts as a failing control with the review's schedule): the fork-to-exec window with the owner dying
and the next owner waiting; the service cgroup reused by a replacement owner (it must not kill itself); the gate with the restarter dying at every
point of L7, a hook admitted after the check, and a wedged shared holder; a pid read in another pid namespace; the foreign lock taken while a child
is stalled before acquiring it, and after it released and stalled before exit (no deletion may occur); the reserved binding (C3 done, C4 not) with a
same-content resend, a different-content resend, and a partial binding that cannot exist; maintenance with a launcher alive and an empty scope, with
process-group escape, and release racing the run; the seqlock with a hook that waits for ever during P2; the immediate follow-up after a completed
switch; a held request completed after its amendment, with the consumption entries across deferred materialization; a failed swap followed by the
identical commit installed and removed by another writer.
Properties to check at minimum: no foreign lock is removed; no replacement or healthy owner is targeted; a protected running program is not cancelled;
every accepted identity stays recoverable to its immutable request or terminal outcome; no recovery copy is released while relevant destructive
execution can still occur; stale index entries are not silently reintroduced into unrelated commits; a superseded version cannot publish. Liveness is
stated with its fairness assumptions and is not made to pass by assuming that every gate holder, hook, child or owner eventually completes.
Explicit red controls named by the review: the intact-anchor deleted-suffix case, restart versus hook dispatch, the retained-descriptor scope
deadlock, same-identity concurrent acceptance, and cross-worktree maintenance overlap.

Later steps, unchanged in order: an offline narrowly eligible index writer with the extension policy (differential against Git, including
later Git behavior); disposable backend experiments (files and reftable, packed refs, `reference-transaction` hooks, linked worktrees,
interruption at real persistence boundaries, push forms against the installed Git); the carry-forward crash model before any filesystem
mutation code; the exit inventory from the 2.56.0 source; the stall-interval measurement of "Measured stall interval".
Each item then keeps its red control, then the change, then a guard shown red once.
