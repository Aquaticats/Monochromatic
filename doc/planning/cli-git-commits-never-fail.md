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

Revision 11 and the owner's bound answers (2026-11-03T12:45, recorded as decisions 4 to 6 in "Owner decisions of
2026-11-03"): every blocked process has a bound in the register of rule B, and a command that returned never has its
commit land later by itself. The exceptions are stated, not hidden: a push ends at its bound with exit 0 and is queued
(the owner's own answer), the person's own programs in their foreground have no bound (owner decision 9), and a
`commit` instruction that the command itself already submitted when its bound passes cannot be recalled (owner
decision 7: the one exception to "nothing lands later", never a fresh attempt). Narrative text below that describes a
command returning exit 0 with a commit that "lands later" (a foreign ref lock, a held `index.lock` before publication,
a request set aside as pending) describes the earlier owner answers and applies now only to a request that has no
client; the normative state machine, its register of waits, W1 and W2 govern. Two exceptions to "nothing lands later"
exist, both owner-approved and both stated: decision 7 (a submitted `commit` instruction may complete after the error)
and decision 3 as it applies after a power loss (a lost receipt of a published exit can let recovery land the commit
again, if a deliberate reset and an erased reflog also hide the publication; decision 10 only chose when data is
forced). A different thing is not a commit landing later: derived index and worktree materialization after a commit or
merge that is already published (M1, MG3).

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

4. Bound of a wait, "Return with an error" (question tool, 2026-11-03T12:45; option text: "The command stops waiting and exits non-zero, naming what
   blocked it; the saved work stays saved."), after the owner's rule "nothing should ever wait forever or indefinitely". This touches "Commits
   should never fail." as the owner chose.
5. Whether a saved commit still lands by itself after the error, "No, keep it for a retry (Recommended)" (option text: "Nothing lands later by itself.
   Your staged changes stay staged, and the commit is also kept on a side ref. The error names the blocker and the command to run again.").
6. Pushes, "Bound it, keep retrying (Recommended)": a connected push without progress for a measured time is ended and kept in the service's retry plan; the
   command says the push is queued. This replaces "Wait while connected".
7. Branch write already issued when a bound passes, "Short extra wait (Recommended)" (question tool, 2026-11-03,
   answered before the commit named below records it): after the bound the command waits a short extra time (the named
   constant `WAIT_COMMITTING`, which must be measured before it ships) for a branch write it has already submitted,
   then exits non-zero saying the result is not known. This is the one exception to "nothing lands later by itself".
   Revision 13 states it exactly: the exception is a submitted `commit` instruction that has not been seen to
   complete, never a fresh attempt.
8. Merge order, "Branch first, files after (Recommended)" (same date and recording): a merge moves the branch first
   and writes the files after, so a withdrawal before the branch moves changes nothing; the merge's files and staging
   may lag the branch until the install runs. This replaces the delegated pick M-A.
9. The person's own programs before the commit exists, "No limit; Ctrl-C ends it (Recommended)" (same date and
   recording): the editor, `pre-commit`, `commit-msg`, a signer prompt, clean and smudge filters and a forwarded
   `git gc` have no time limit; the person ends the command.
10. How soon an accepted commit must survive a power cut, "Force at leaving, else background (Recommended)" (same date
    and recording): files are forced to disk before a command leaves without a published commit, and in the background
    otherwise (rule V). Revision 13 adds what the decision does not say and the review found necessary: the same force
    applies to every exit that promises saved work, and background forcing finishes before anything is reclaimed.
    Recorded by commit 4d8445dd52, whose time is 2026-11-03T20:16:29-05:00.
11. Staged version and file differ at a merge, "Carry the file, save staged (Recommended)" (question tool, 2026-11-03,
    answered before the commit named below records it, on the revision 13 question 5): the file you see is the edit
    that is carried and the staged-only version is saved aside. Given; asked again only because the thirteenth review
    found an option missing (revision 15, question 5), and the answer stands unless the added option changes it.
12. Slow forcing at leaving, "Short wait, then say unconfirmed (Recommended)" (same date and recording, revision 13
    question 6): provisional in revision 14, and superseded by decision 14: the owner answered that the final disk
    save is never cut short, so there is no timed departure and the question is not asked again.
13. Late `git switch` watch, "A measured span, then stop (Recommended)" (same date and recording, revision 13 question
    7): provisional: framing corrected. The question claimed the cleanup never undoes a later intentional change,
    which the thirteenth review disproved. It is asked again as three questions (revision 15, questions 7a, 7b and
    7c). Decisions 11 to 13 were recorded by commit 1734105e9a, whose time is
14. The final disk save at leaving, "Assume the final disk save would never be cut short." (owner, 2026-11-03,
    verbatim; answered before the commit named below records it): the departure barrier (the stop disposition and then
    the saved-content set) always runs to completion. `WAIT_STOP` and `WAIT_DEPART` are removed as limits, questions
    6a and 6b are not asked, and decision 12 ("Short wait, then say unconfirmed") is superseded by this decision. A
    sync that never completes is a disk I/O error, disk full or read-only remount, which the owner's list of causes
    outside our control names; the register says so and the rest of "nothing should ever wait forever or indefinitely"
    is unchanged. Decision 14 was recorded by commit d2682fb03f, whose time is 2026-11-03T22:13:12-05:00.
    2026-11-03T21:47:28-05:00.


Amendments by 4 to 6 (the newest owner answers govern): decision 1's set-aside-as-pending of a commit behind a person's hook, the earlier answer "Save on a
side ref (Recommended)" for a live foreign ref lock (the service then moves the branch onto the saved commit), the carried-forward `index.lock` single
try that "exits 0 and finishes later" for a merge, and the exit 0 notices of `pending` and `landing` all applied to a command that had already returned.
For a command with a client they are replaced: the command waits to its bound, then is withdrawn with an error, and the commit is kept on a side ref for a
retry. They still apply to a request that has no client (a service-originated reconcile merge, a request whose client was killed from outside), because
nothing waits on it. Decision 3 (land an unclear publication) is unchanged for the recovery of an earlier attempt.

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

### Normative state machine (revision 15, first committed 2026-11-03T22:13:12-05:00 in d2682fb03f)

Revision 12 of this machine was first committed at 2026-11-03T13:39:22-05:00 in 0cfe831dea, and revision 13 at
2026-11-03T20:22:09-05:00 in 4a7bea95a2.

This is the one operative statement of the protocol. The narrative subsections that follow explain it and add rationale and test
requirements; where one of them reads differently, this section governs. Every transition names its preconditions (Pre), its durable
effects in order (Do; where a step says "fsync" or "durable", rule V decides when that sync happens and who waits for
it: by default no transition waits for a sync, and the syncer or a departure barrier supplies it), and what recovery
does if the process dies at any point of it (Crash). "Journal" means a record the next owner can read. A transition never holds a lane
state generation, the lane lock, the gate or `index.lock` across the execution of a person's program, except where it says so.

Named entities. A worktree has one lane. The lane has an owner, an incarnation, a lock `lane/owner`, a gate `lane/gate`, a state file
`lane/state` with a generation, a journal, an order file `lane/order`, a queue, an inbox, and a spool per request. The common directory
`$GIT_COMMON_DIR/cli-git/` holds one ledger per lane (`ledgers/<instance>/ledger`, appended only by that lane's
owner), the release intents `releases/`, and the maintenance registry `maintenance/` (the identity bindings `ids/` of
revision 11 are removed by rule V: an identity is the content hash of the request plus the lane instance). The lane
also has the withdrawal marks (empty files `lane/decision/<id>.withdraw` and
`lane/decision/<id>.withdraw-unresolved`), the receipts `lane/receipt/<id>`, the client locks `lane/client/<id>`, the
attempt directories `lane/p2/`, and its state document is carried by the hash-chained state log `lane/state.log` (rule
S, rule V; the text says `lane/state` for the newest valid record). Nothing in the common directory has more than one
appender: a registry entry is a file created once by `link`, and a ledger is appended by exactly one lane.

Request states: `drafted` (client only) -> `bound` -> `accepted` -> `adopted` -> (`held` | `ready`) -> `publishing` -> (`published` |
`pending` | `landing` | `rewritten-away`) -> (`materialized` | `voided`) -> `retired`; a hook may end a request as `refused`; an obsolete
version of a held request ends as `superseded` (B3); a merge request has the extra state `merge-ready` and, after
publication, `installing` (MG1 to MG3, MG6c; the entries it leaves behind are of kind `attach-watch`) and may end
`conflicted`; a request whose client stopped waiting ends `withdrawn` (W1, W2) and never
lands. A request
that a client abandons before `bound` is removed by the lane after the client's process identity is gone; a `bound` request is never removed
as abandoned, because C3's recovery obligation is to complete it.

State document, the revision counter and the marker (rule S; revision 10 separates the two). All capture-visible lane state lives in one
document, `lane/state`: a revision counter `g`, a marker (`none`, `materializing`, `recovering`), the logical head, the logical tail, the
consumption map and the state of every non-terminal request (each with its `resume` point, see Q1). A consumption entry is (path, pre-entry
`E` with every field, request, version, kind); the kind is `install` (the request's published result is still to be applied to that path of
the index; outstanding installation authority) or `suppress` (the index entry `E` is a historical entry that a published request already
consumed; it is not a person's staging and no one is to install anything for it). The two end differently (see M2 and M3). A transition that
changes any of it does so in one S1 replacement: append the intent to the journal, then append to the state log a
record that holds the whole document with generation `g` + 1 (revision 13, rule V: V on write, D by the syncer or a
barrier; nothing is overwritten in place, and recovery takes the longest valid hash-chained prefix); `g` increases by
one with every replacement and means
nothing else. Consumption kinds added in revision 12: `merge-install` (a published merge whose files are still to be
installed, MG2) is a third kind of `install` entry with the pre-merge worktree digest; MG3 turns it into the kind
`attach-watch`, which has no effect on capture and only keeps the cleanup obligation of MG6c, and a `suppress` entry
stays what it was. The marker, not the parity of `g`, says that an effect outside the document is in progress: a
transition with such an effect (the index replacement of M1, the install of MG3, the
recovery of L4) first replaces the document with the marker set, performs the effect, then replaces it with the marker cleared. A reader that
reads the document once sees a state the lane was really in, including a multi-path release of the consumption map; C1 accepts a capture only
when two reads show the same `g` and the marker `none`.
The capture-visible mutations, each an S1 replacement: C5 adopt, B1 build/hold/ready, B2 held completion or refusal, B3 supersede, P1
publish-begin, P3 published, P4 swap-failed/pending, Q1 resume, M1 and M2 (with the marker), MG3 (with the marker), L4 (with the marker).
T1 retire changes no capture-visible state.
Bound on the cost of a replacement (revision 10): `lane/state` holds only requests that are not terminal, the entries of the consumption map
that still have an effect, and the logical tail; a request with a final client-visible outcome leaves the document only after T0 has published that outcome to the lane's ledger, and its
history then lives only in the ledger. The document therefore grows with the work in flight and the unresolved suppressions, never with the
lane's history, and a replacement costs the append of that bounded document to the page cache (V; the syncer forces it
later, rule V). `lane/order` is append-only history, but no transition
reads it past the adopt line of a request being resumed (the document names each request's position), and it is compacted at a ledger boundary by a
rotation (a new file started after the document records the old file as closed), so recovery reads one bounded file. The measurement gate: commit cost is
flat against ledger length (step 1, "state document size independent of history").

Rule V, durability levels and the syncs a person waits for (revision 12; repaired in revisions 13 to 15, R12-4, R12-5,
R12-7, R13-2, R13-4 to R13-6). Owner rules: "commits vs native git should not take 5s longer.", "Disk pressure being
high is not an excuse." and "sometimes the work itself can be optimized. And we control every part of the wrapper,
including everything it embeds." Input: the fourth review of the pure components counted `2n + 20` sync calls to
acceptance (22 for a one-path commit) and `2n + 27` (29) through retirement; the latency branch then measured, on the
0.5.28 base, 0 syncs waited by the wrapper on a publishing landing (Git's own pack sync on the unsigned `fast-import`
route remains), in `doc/planning/cli-git-commit-latency-under-disk-pressure.md`. Owner decision 10, "Force at leaving,
else background (Recommended)", chose when forcing happens. Two levels are kept apart. V, visible: the bytes are in
the page cache and named, so they survive the death of any process (a kill, the OOM killer, a crash of the owner or
the client), which is what the lane's crash recovery needs. D, durable: the bytes survive power loss or a kernel
crash, which only a sync provides. A record is written V by its transition with no sync. The rules for D (revision 14
split what a departure must make D into things of different urgency, so that the fact that publication is forbidden is
made D first; revision 15 adds the owner's decision that this final save is never cut short):
- Stop disposition (R13-2, R14-2; owner decision 14, "Assume the final disk save would never be cut short."). The
  departure barrier always runs to completion: a command that leaves does not return until its stop disposition and
  its saved-content set are D, and neither has a limit. The stop disposition comes first and alone. For a withdrawal
  it is the withdrawal mark, an empty file `lane/decision/<id>.withdraw` or `lane/decision/<id>.withdraw-unresolved`
  (the attempt in flight is the one the owner has recorded) created with `O_CREAT | O_EXCL`, forced by `fsync` of the
  file and then of `lane/decision/`; for any other terminal exit (a refusal, a conflict, `already`, `rewritten-away`,
  `unresolved`) it is that outcome's ledger line and its directory entry. Recovery reads the marks and the ledger
  before it decides anything, so a D stop disposition is what keeps a request from publishing after the error, plain
  or after attempt `a`. While the barrier runs the person's interrupt is held off (a second `SIGINT` does not cut it
  short either); a `SIGKILL` is an outside event, and then the request continues by the outside-cause rule. A sync
  that never completes is a disk I/O error, disk full or read-only remount, which the owner's list of causes outside
  our control names ("disk I/O errors", "disk full or quota"); it is not a wait of the wrapper, and a sync that
  returns an error ends the command non-zero saying the stop could not be made durable because of that disk error.
  What remains of the resurrection schedule: the power fails before the barrier completes (no error was reported yet,
  so no promise was broken: the person saw nothing, and recovery may finish the request by the outside-cause rule), or
  the person kills the command during the barrier (outside). A timed stop flush, which the earlier schedule needed, no
  longer exists. The earlier claim that no fail-closed recovery is implementable was wrong (the narrower fact is that
  recovery cannot tell a departed client from a killed one without more durable information); a fail-closed policy was
  designed in revision 15 and is not needed under decision 14, and its design is in the history record.
- Saved-content set (R12-5C, R13-6, R14-5). What the exit promises as saved, forced after the stop disposition, to
  completion (decision 14), in dependency order. The set by exit: W1 withdrawn and an interrupt: the request file,
  `inbox/`, the lane log files through the record that names the saved item; the unresolved COMMITTING exit: the same;
  a request left with the service (no client, or a `handed over` followed by exit): the request file, `inbox/` and the
  log through its `adopt`; `conflicted`, `rewritten-away` and `unresolved`: the saved inputs, the log record that
  names them and the ledger line. What is promised depends on what is verified, in two levels. Level 1, always: the
  captured intent (the immutable request, enough for another replay) is saved. Level 2, only when verified: the exact
  built commit `D`. Reachability and a side ref are not a durability barrier (a ref read back proves visibility) and
  do not stop a prune that took its view earlier, and the objects `D` inherits from a parent that another writer
  published may be V only. So level 2 requires all of: `syncfs` of the object store's file system (the lane cannot
  know which inherited objects are D, and a whole-file-system flush is the only barrier that covers them and their
  directory entries; it can take seconds under pressure, which is why it is the exact-commit promise and not the
  default one); the side ref `refs/cli-git/<instance>/withdrawn/<id>` created and read back; a connectivity check of
  `D` from that ref; and, around that check, the registry comparison of T1 (the probe of the next reservation number
  and the two listings), so that no wrapper-started prune can have taken its view before the side ref and deleted
  after it. When level 2 cannot be verified (the check fails, a prune is open), the exit promises the captured intent
  only and says the exact commit may have to be rebuilt on retry. A request held by a hook, or one whose build had not
  finished, has no commit and the message names the retained inputs. A command that waits for its request until it is
  published has no departure before publication.
- Terminal answer. A command that leaves with a final outcome forces that outcome's ledger line as its stop
  disposition; a successful return forces nothing of ours.
- Reclamation barrier (R12-5A). Nothing a request's recovery or its identity depends on is released until the syncer
  has made D the terminal ledger line and its directory entry and the log record that removes the request, up to a
  confirmed watermark (below). T1 has this in its Pre. It is background work.
- Preservation barrier (R12-5B). Before the recovery authority of a version that only the person had is replaced (the
  staged-only blob saved by MG3, a displaced inode of the carry-forward protocol, the pre-merge file `N`), the
  preserving file, its directory entry and the log record that names it are D. The carry-forward protocol's own
  explicit persistence ordering stands and is exempt from rule V's default. These barriers occur only when the person
  has an edit or a staged-only version to preserve, never in a plain commit.
- Background syncer. A thread or process separate from the owner's event loop (a `fsync` never blocks admission, the
  owner or another command) brings every V record that rule V names (requests, the log files, ledgers, saved items,
  marks, outcomes, pins; not the receipt or the client lock, which are authority only for process crashes) to D within
  `SYNC_LAG` (a shipped constant, unmeasured; its measurement is the longest healthy interval between a request's
  acceptance and its publication), syncing each distinct file once per pass and each directory once per pass. Durable
  prefix watermark (R13-5): before it issues a sync the syncer reads the owner's published append offset and chain
  hash of the file; a completed sync makes D that captured prefix and nothing appended during the sync; every claim
  "this is D" in reclamation, rotation and T1 rests on such a watermark and never on a per-file durable flag.
- Deadlines (R12-7, decision 14). The bound of a command is the deadline to start departing. Departure then spends
`WAIT_WITHDRAW` (the wait for W2) and then forces the stop disposition and the saved-content set to completion: those
two have no limit, because the owner decided that the final save is never cut short, and a sync that does not complete
is a disk fault on the outside list. A flush runs on the syncer's thread and the command waits for it; a foreground
thread inside `fsync` has no timeout to implement. What lets V stand in for D between a write and its sync, so that a
power loss after a partial write is recovered and not trusted (revisions 13 and 14):
- A checksum proves a record's integrity and nothing about its causes, because unsynced write-back can persist a later
  record while an earlier one is missing. So every append-only file of the lane (the journal, the order file, the
  state log, each ledger) is hash-chained: each record carries the hash of its predecessor, and recovery takes the
  longest prefix that validates and ignores every record after the first that does not.
- Dependencies are of two kinds and are discharged (R13-4, R14-4). A hash commitment is historical: it says what the
  record followed and is checked only while the predecessor is present in the chain being read. A live dependency
  names external content the record needs to exist: an `adopt` record names its request file; a record that names a
  saved item names the item; an install names its pins. The ledger is the single discharge authority and is a root: a
  request file's dependency is discharged by an `outcome` line for its identity only when that outcome promises
  nothing saved (`published`, `refused`, `already`, `superseded`); an outcome that promises saved inputs (`withdrawn`,
  `conflicted`, `unresolved`, `rewritten-away`) leaves the dependency live until a `disposed` line in the ledger
  discharges it. A discharge claimed by a record of the state log does not count; only the ledger's valid prefix does,
  so a suffix that purports to discharge a missing dependency cannot be trusted to do so.
- A checkpoint re-roots dependencies and does not erase them (R14-4). It discharges reliance on old log records. For
  live external content it carries the **live dependency list**: for each non-terminal request and each undisposed
  saved item, its identity, its checksum and the names of its pins. A checkpoint is written only when the owner can
  state that list, and the old root is removed only after the syncer has made D the new segment through the checkpoint
  and every entry of that list (the request files and their directory entries, the saved files and their directory
  entries, the pins). Recovery validates the list when it selects the checkpoint: an entry that fails drops that
  request (it was not accepted, or the filesystem lost D data, which is on the owner's outside list) and never the
  checkpoint.
- Recovery procedure and validation order (R13-4, R14-4). Pass 1 validates chains and checksums only: each ledger's
  longest valid prefix first (the roots), then the order file, then each state-log segment. Pass 2 selects the
  highest-numbered segment whose first record is a checkpoint with a valid live dependency list, then evaluates the
  records after the checkpoint in order: a live dependency is satisfied when the content exists and its checksum
  matches, or when the ledger's valid prefix discharges it; the first record with an unsatisfied dependency truncates
  the log there, with everything after it. A state record that names a ledger line the ledger does not hold is dropped
  with its suffix, and the request is still live in the recovered state; L4 completes T0 from what the earlier records
  say. The sealed predecessor is not required to exist: its back-reference is informational, and authority comes from
  the checkpoint alone.
- The state is carried by the state log `lane/state.log.<k>`; a log is never overwritten in place. S1 appends a record
  that holds the whole document (generation `g`, bounded). Checkpoint cutover (R13-5), a serialized owner-side
  protocol because the owner is the only appender: (1) the owner decides to rotate; (2) it appends a `seal` record to
  segment `k` naming segment `k + 1` and the final chain hash, and appends nothing else to `k` afterwards; (3) it
  creates segment `k + 1` whose first record is a checkpoint holding the full document after the seal, the ledger and
  order positions and hashes, and the lineage back-reference (immutable once written; no S1 can interleave between the
  seal and the checkpoint, because the same thread does both); (4) from then on S1 appends go to `k + 1`; (5) the
  syncer captures the watermark of `k + 1` through its checkpoint record, syncs the file and `lane/` and every entry
  of the checkpoint's live dependency list, and only then may segment `k` be removed. Recovery follows the recovery
  procedure above; if the newest segment has no valid checkpoint it falls back to `k`, which still exists and is
  sealed because it is removed only after the new root and its live dependency list are D, so no S1 is lost to a
  half-made cutover.
- A request is one file, `inbox/<id>`: its record, its manifest, every captured side and a fresh invocation nonce
  (R12-11), with one checksum, written under a temporary name in `inbox/tmp/` and published by a no-replace `link`.
  The identity `<id>` is the content hash of that request, nonce included, plus the lane instance. Two identical
  operations by different invocations carry different nonces and so different identities; a resend of one invocation
  carries its retained bytes and nonce and so the same identity. Directories (`inbox/`, `inbox/tmp/`, `lane/`,
  `queue/`, `lane/decision/`, `lane/receipt/`, `lane/client/`, `lane/p2/`, the ledger directory) are created once,
  when the lane is initialized, and synced then; no per-commit transition creates one or syncs an ancestor. The
  dependency chain for D, in the order the syncer and a barrier follow it: the stop disposition first, then the
  request file, `inbox/`, the `adopt` record, the state log through the record in question, and for a terminal outcome
  the ledger line. Object writes (`fast-import`, the embedded writer) and the ref write follow the person's
  `core.fsync` exactly as native Git does, so they add no wait beyond native; the index installed by M1 follows the
  same setting. The count of syncs a person waits for, per transition, is in "Cost of the durable steps".

Rule D, durable before reliance (revision 10; narrowed by rule V in revision 12). A name that a process created and
has not yet made durable is visible to other processes, so the creator's own sync is not a promise to an observer.
Under rule V this matters only at a departure barrier, at a reclamation or preservation barrier, and at the answer of
a resend that the asker will rely on across a power loss: whoever relies on a name it did not itself make D, at such a
point, makes it D first (the file, then the directory that contains it), following the dependency chain of rule V. The
cost is paid only by a resend or a departure, which is rare, and a sync of an already durable directory is cheap.
Rule N, names the lane may remove (revision 10; R9-3). Linux offers no unlink or rename that is conditional on the identity of the file at a
pathname: a check of the inode followed by `unlink`, a `rename` into a private directory and a `RENAME_EXCHANGE` each leave an instant in which a
replacement file that a foreign program created at that pathname is removed, displaced or substituted, and a journaled `(device, inode)` pair
identifies nothing once the inode is freed (an unnamed `O_TMPFILE` inode dies with its last descriptor and its number is reused). So the lane
removes a name only if no program but the lane ever creates that name: the unique names under `lane/`, `incoming/`, `saved/` and `lane/tmp/`
(including the carry-forward temporaries, which are created in `lane/tmp/` and moved by `RENAME_EXCHANGE` across directories of one file system,
or, where that fails, beside the path under a unique name that carries the incarnation id and a counter, which no other program picks). The lane
never removes or substitutes a lock that a foreign program may create (`index.lock`, a ref lock); a file of the person is displaced only by the
carry-forward protocol's `RENAME_EXCHANGE`, which removes nothing: the displaced inode is retained under a name the lane owns, so the instant it
leaves a different file at the pathname loses no content.
Where recovery would need such a removal, it leaves the file, reports it, and keeps the request deferred (M1 and MG2 are `pending`).
Normal abort is different from abandoned-lock recovery (revision 11; R10-2): a live creator releases the lock it created in this same incarnation, on a path
that ends without installing (a declined attachment guard, a plan that finds staged paths, a failed write), by `unlink` of `index.lock` after comparing the
identity of its own open descriptor with the pathname (`fstat` against `stat`), which is what Git's own lock rollback does. The premise, stated: no
program removes a lock it did not create while its creator is alive, which is the premise of every Git lock; the comparison narrows the window in
which a replacement could be removed and cannot close it, and a mismatch means the premise was broken, so nothing is unlinked and the event is
reported. This authority exists only for a lock held by the live creator; a lock found without a live creator is never removed. A journaled
inode identity is a claim only while a name the lane owns pins the inode (a hard link under `lane/`), and is then used for reports, never as
deletion authority at a shared name.

Rule B, bounded waits and withdrawal (revision 11; repaired in revision 12). Owner rule: "nothing should ever wait
forever or indefinitely". Owner answers of 2026-11-03T12:45 (decisions 4 to 6): at a wait's bound "Return with an
error"; "No, keep it for a retry (Recommended)"; pushes "Bound it, keep retrying (Recommended)". A process blocks on
another party's progress only in an entry of the register below, each with a bound and an action at the bound. A
durable request with no process blocked on it is not a wait; the register lists blocked processes only. The action at
the bound of a command's foreground wait is always W1: the command stops waiting and exits non-zero naming what
blocked it; the request is durably withdrawn and no attempt of it can write the branch afterwards, with the one corner
P2 states (the command's own `commit` instruction was already submitted; owner decision 7). The person's staging is as
it was
because no transition changes the index or the worktree before the request is published (revision 12: the merge
transitions publish first, so this holds for merges too; owner decision 8), and
because a withdrawn request has no consumption entry that could hide the staging from a later capture (C1 reads the
withdrawal marks). The error names the blocker and the command to run again, and what is kept: the built commit on
`refs/cli-git/<instance>/withdrawn/<id>` when the request has one; when it has none (a request held by a hook, or one
whose build had not finished), the retained request inputs, saying that no commit exists yet. Exit rule (revision 12;
R11-4). A client with a request that is not published leaves by exactly three ways: `published`, a terminal refusal,
or W1 (the unresolved COMMITTING exit is a W1). Every other exit path of such a client, an inline owner's ACQUIRE or
REAP expiry and a signal-driven exit included, performs W1 first, and W1 creates the withdrawal mark before anything
else the exit does, so no new attempt of the request can start after any normal exit (R12-1). Only an exit that no
process of ours takes (a kill from outside) leaves the request without a mark. Budget (revision 12; R11-9). Each
register entry that applies to a command has one
monotonic deadline, started when the command first needs the entry; a retry, a re-evaluation or a change of phase
never renews it, and a loop that re-reads state (C1 repeating, C0 re-evaluating, Q1 triggers) draws on the entry that
contains it. The upper bound of a command is the sum of the bounds of the entries it passes through. Bound values
follow the repository's rule for shipped limits: each is a measurement or a labelled unmeasured value whose owner
approval is pending; the placeholders below name the shipped constant they start from.
The register (waiting process; condition; bound and its basis; action at the bound):
- START: a client waiting for the service to start; `START_WAIT` (shipped, 10 s, unmeasured, owner approval pending);
  W1, or the plain error of CAPTURE when no request exists yet.
- CAPTURE (revision 12): a client in C0 and C1 before its request exists, including a set marker, a recovery in
  progress, a recovery helper that lost its acquisition, and C1 repeating because the lane's state changed;
  `WAIT_CAPTURE`, starting from `START_WAIT`, one deadline for all of C0 and C1; at the bound the command exits
  non-zero saying the lane kept changing or recovering, that nothing changed, and to run the command again (no request
  exists, so there is nothing to withdraw).
- MARKER: the part of CAPTURE spent waiting for a marker or for one recovery to finish, tied to that condition (the
  marker changing or clearing), not to a process's death; `WAIT_MARKER`, starting from `STOP_GRACE` (5 s, unmeasured
  for this use; the measurement is the duration of an M1 replacement and an L4 recovery on the owner's hosts) caps one
  such wait and CAPTURE caps their sum; same action as CAPTURE.
- FOREGROUND: a client waiting for its own request's outcome, from the moment the request is bound, which includes a
  held ref lock or `index.lock` it retries on triggers (no polling loop), a person's hook that the service runs and
  does not end, a held request, a quiet owner that holds the gate (owner question B, option B-hand: the error names
  the unit and the command that restarts it) and a signer failure; `WAIT_FOREGROUND`, starting from the shipped
  `COMMIT_WAIT` (600 s, unmeasured, owner approval pending; the measurement is the distribution of healthy commit
  durations in `.git/cli-git-commit-timing.jsonl`, with the margin the owner approves); W1.
- ACQUIRE: an inline-owner client waiting at L1; `WAIT_ACQUIRE`, starting from `START_WAIT`; the client stops being an
  owner candidate and waits as in FOREGROUND for whoever owns the lane. A recovery helper does not wait at L1 at all
  (C0).
- SERVICE-ACQUIRE (revision 12): the service process waiting at L1 behind an inline owner or a previous incarnation's
  child; `WAIT_ACQUIRE`; at the bound the service exits and the manager starts it again under its own restart
  back-off; requests stay durable. It is not a command and nothing is reported to a person.
- REAP: an acquirer waiting at L2 for the previous incarnation's scope to empty after the kill; `WAIT_REAP`, starting from `STOP_GRACE`; at the
  bound the acquirer releases the lane (by exiting; the lock is never unlocked while the process lives), requests stay durable, and the next trigger
  tries again; an inline client with a request that is not published performs W1 first (exit rule).
- KILL: an L7 restarter waiting for the owner to die after SIGKILL; `WAIT_REAP`; at the bound it releases the gate and
  exits non-zero naming an owner that did not die (uninterruptible sleep).
- COMMITTING (revision 12, replaces revision 11's entry): a client that has itself written `commit` to the publication
  pipe (P2b) and waits to see the branch write; `WAIT_COMMITTING`, starting from `STOP_GRACE` (5 s, unmeasured; the
  measurement is the latency from `commit` written to the tip equal to `new` in healthy runs). It is the only wait
  that can pass the FOREGROUND bound, and only by this amount. At the bound the command takes the unresolved exit of
  W1 (mark `withdraw-unresolved`, non-zero, saying the instruction was already submitted and its result is not known,
  and how to check). The owner approved this corner in decision 7, with the extra wait named `WAIT_COMMITTING`, which
  must be measured before it ships.
- WITHDRAW (revision 13): a command that withdrew, waiting for the lane to record the withdrawal (W2) so that a
  capture no longer sees withdrawn work; `WAIT_WITHDRAW`, starting from `STOP_GRACE` (5 s, unmeasured); at the bound
  the command goes on to its departure barrier, and C1 keeps refusing to capture while the lane has not applied the
  withdrawal.
- MATERIALIZE (revision 14): a command that published waiting for the single attempt at M1 or MG3; `WAIT_MATERIALIZE`,
  starting from `STOP_GRACE` (unmeasured); `WAIT_MATERIALIZE` bounds the start of the single attempt; a mutation
  already begun is finished, with interrupts held off. If the attempt has not started at the bound, or a foreign
  `index.lock` defers it, the command exits 0 `published` saying the index and files follow, with `resume = M1` or
  `MG3` recorded, and the service runs it (see the materialization executor rule).
- DEPARTURE (decision 14, "Assume the final disk save would never be cut short."): a departing command forcing its
  stop disposition and then its saved-content set to disk (rule V). It has no bound, and it is not a wait on another
  party: a sync that does not complete is a disk I/O error, disk full or read-only remount, which the owner's list of
  causes outside our control names, and a sync that returns an error ends the command non-zero naming that disk error.
  Interrupts are held off while it runs; a `SIGKILL` is an outside event. Everything else in "nothing should ever wait
  forever or indefinitely" is unchanged.
- STALL: the owner supervising a child; the measured stall interval ("Measured stall interval"), then `STOP_GRACE` (shipped, 5 s). The owner never
  blocks its event loop on a child; the blocked process is the client (FOREGROUND).
- PREPARED-HOLD (revision 12): the owner holding a prepared reference transaction while no closing line has been
  written; `PREPARED_HOLD`, starting from `STOP_GRACE` (5 s, unmeasured), counted from `prepare: ok`; at the bound the
  owner attempts to enqueue `abort` (P2b); enqueuing, execution and the release of the branch lock belong to Git and
  the scheduler and are not guaranteed. It bounds the owner's attempt to enqueue `abort` and nothing more: it does not
  bound how long the branch lock is held.
- PUSH: the service's or a command's push; the shipped no-progress deadline `DEFAULT_PUSH_DEADLINE` (30 s) and ceiling `DEFAULT_PUSH_CEILING`
  (1800 s), unmeasured, measured from the end of any `pre-push` hook (revision 12); at the bound the push child is
  ended, only when its exact invocation cannot run a protected program (X1; a push with a `pre-push` hook ends at
  FOREGROUND as queued instead, and the hook is not ended), and the push is kept in the service's retry plan, which
  starts no second execution until the first execution tree is empty. The command exits 0 saying the push is queued
  (owner answer 3). This is the one bound whose action is exit 0.
- No wait exists for: `index.lock` and ref locks (a single try each; Git's own 100 ms ref-lock timeout,
  `refs.c:996-997`, is its bound); the gate (a single try, see L6 for the failed try); maintenance (T1 defers; a
  deferred retirement is retention, not a blocked process); a person's hook that the service runs (a program of the
  person's, never ended, no process of ours blocked on it except the client in FOREGROUND); kernel I/O (an
  uninterruptible disk wait is on the owner's outside list); requests that have no client (a service-originated
  reconcile merge, a request whose client was killed from outside: the existing outside-cause rule finishes
  handed-over work on the next run); the derived follow-up of a published commit (M1, MG3: it is not a commit landing
  later; the command makes one attempt at it under `WAIT_MATERIALIZE`, takes a single try of `index.lock`, and waits
  for no foreign program; the register lists MATERIALIZE as a wait of the command).
- Forwarded maintenance (revision 13): a forwarded `git gc`, `prune` or `maintenance` that the wrapper runs in the
  foreground for the person has no bound, as covered by decision 9 as the owner saw it listed; it is its own line so a
  later decision can bound it without touching the rest.
- The person's own programs, not a wait on another party (revision 12; owner decision 9: no bound): the editor,
  `pre-commit`, `commit-msg`, `prepare-commit-msg`, a signer's prompt and clean or smudge filters that the command
  runs in its own foreground before any request exists, as native Git does, The command is that work, as the owner
  decided (decision 9); owner decision 1 forbids ending the person's hook; the person ends the command.

Lane transitions (the owner and its incarnation)
- L1 acquire. Pre: none. Do: open `lane/owner` (a file created once and never replaced) with close-on-exec, wait for the exclusive lock.
  Crash: nothing to recover. Note: the wait ends when the previous owner and any child still between fork and exec have gone; a child
  between fork and exec holds the description until it execs or dies, so this wait is bounded only by that child's own
  progress. Revision 12: the wait has a bound, ACQUIRE for an inline-owner client and SERVICE-ACQUIRE for the service;
  a bound reached ends the attempt to own the lane (the client then waits as in FOREGROUND; the service exits and is
  restarted).
- L2 reap. Pre: L1 done. Fresh lane: if `lane/incarnation` is absent (the lane was never owned) or names this process, L2 has nothing to
  reap and is done at once; the first owner of a lane goes from L1 to L3. Otherwise `lane/incarnation` names a previous incarnation `P`
  that is not this process. Do: terminate the cgroup whose
  name contains `P`'s identifier (each incarnation has its own cgroup: a transient scope for an inline client, a delegated sub-cgroup of
  the unit for a service; the identifier is unique, so the target can never be this owner's own group) and wait until it reports no
  processes. Crash: repeat L2. Until L2 completes the owner mutates nothing. Never targeted: a live owner, a group not named `P`.
- L3 incarnate. Pre: L2 done (reaped, or the fresh-lane case). Do: write `lane/incarnation` (a fresh id, process identity, pid-namespace identity, the cgroup name),
  create that cgroup, move this process into it. Crash: the next owner reaps this incarnation by L2.
- L4 resume. Pre: L3 done. Do: first replace the document with the marker `recovering` (fsync), so that no capture can mix the state of a half-completed mutation;
  then complete or roll back each journaled mutation; repair the torn tail of `lane/order` and of this lane's own ledger only (never
  another lane's ledger: another lane's tail may be an append in progress, and a reader of another lane's ledger treats a torn tail as not
  yet written); replace the document with the marker `none`; then, for an owner (not for a recovery helper, which exits here), continue each request from its recorded `resume` point. Crash: repeat L4, marker first.
- L5 spawn a child. Pre: L3 done. Do: start the child inside the incarnation's cgroup with the lock and gate descriptions closed at exec
  and the parent-death signal set. Crash: a child between fork and exec holds the lock description until it execs or dies (L1 waits for it).
- L6 gate-shared. Pre: none. Do: take `lane/gate` shared before spawning any person's program (hook, filter, signer, helper) or any
  ref-transaction child; then read `lane/restart-intent` and, if it names this incarnation, admit nothing and exit (the check comes after
  the shared gate is held and before the dispatch, so a restarter that already holds the gate exclusively cannot be
  raced). The take is one try; a failed try (revision 12) means a restarter holds the gate exclusively: the owner
  admits nothing, reads `lane/restart-intent`, and exits, and the request is retried by the next incarnation (the
  client's wait is FOREGROUND); keep the gate
  until the whole execution tree of that invocation has no processes (its own cgroup is empty), not until the immediate child exits. Crash: the kernel releases it; the program, if still running, was killed with the incarnation or
  is reaped by L2.
- L7 restart a quiet owner (client-triggered; shipped only with the measured interval; same pid namespace only). Pre: the owner's
  recorded pid-namespace identity equals this process's; no progress for the measured interval. Do: (1) open a pidfd on the recorded
  process and check its start time; (2) take `lane/gate` exclusively without waiting, abort if not taken; (3) re-read
  `lane/incarnation`, abort if it changed; (4) create `lane/restart-intent` naming the incarnation (fsync); (5) end the owner through the pidfd with an uncatchable signal (`pidfd_send_signal` of SIGKILL: a wedged owner can be ignoring or
  blocking the terminate signal, so a catchable one proves nothing); (6) hold the gate until the owner is dead. Crash: before (4) nothing happened; after (4) the owner, when it next reaches an
  admission point (L6) and finds the intent, admits nothing and exits, so the gate failing open cannot admit a program; a later
  restarter may complete (5). No manager stop request is used, so no request can outlive the restarter. There is no restart without the exclusive gate: a restarter
  that cannot take it ends nothing, because a process-tree sample cannot show that no program will be dispatched a moment later. The
  earlier pick that restarted anyway (owner question B, option B-restart) is withdrawn for option B-hand; a different admission interlock that could end an internal wedge without
  ending a hook is open design work, not part of this machine.

Execution ownership (revision 12; R11-4). The lifetime of a foreground command and the ownership of an execution are
kept apart.
- A protected program (a hook, a filter, a signer, a helper, and the person's `reference-transaction` hook through
  `update-ref`) runs only under the service owner, in the service's scope, whose life does not depend on any command.
  A foreground client, including an inline owner, never runs one. A request that needs one while no service owns the
  lane stays handed over: the client starts the service (START) and then waits as in FOREGROUND. Where no manager
  exists the service cannot be started and the command ends at START with an error; the work stays saved.
- An inline owner is allowed only for operations whose exact invocations can start no protected program (the table of
  X1), and every child it starts runs with `core.hooksPath` pointing at an empty lane-private directory and with no
  signer or helper configuration reachable, so a hook installed after the eligibility check is not run for that
  attempt (a native commit racing the installation of a hook has the same outcome). A planned exit of an inline owner
  is therefore never the end of a protected execution: there is none, and its children are idempotent or cancellable
  by X1, and an open P2 attempt is closed by `abort`. The next owner's L2 reap meets only scopes of incarnations that
  died or planned-exited with no protected execution.
- Materialization executor (R14-7). `WAIT_MATERIALIZE` is a start deadline for a mutation, not a completion deadline:
  once M1 or MG3 holds `index.lock` or an open marker, a planned exit never abandons it, so the command waits for it
  to finish (local operations, plus the preservation barrier of rule V when a unique version has to be preserved),
  with interrupts held off until the closing replacement. Before it starts, the command may leave with `resume = MG3`
  or `M1` recorded; the executor is then the service: the command asks the manager to start it, the service acquires
  the lane and L4 runs the install, which is idempotent by the rule of M1. With no service and no manager the inline
  owner runs the install itself and the command waits for it.
- Hooks that the client runs in its own build before handover (the person's `pre-commit`, `commit-msg`) are the
  person's foreground programs, not the lane's (see the register: the person's own programs).

Withdrawal (revision 14)
- W1 withdraw (client), in two forms with separate preconditions (R13-10). Plain withdrawal. Pre: the request is
  `bound` (C2) or later; the client has no `commit` that it submitted and that is still open; and one of: the
  command's foreground wait reached its bound; the person interrupted the command; an inline owner is about to exit
  (exit rule); START or setup failed after the request was exposed; the command's exit guard caught an internal error
  after the request was exposed. Unresolved withdrawal. Pre: the client submitted `commit` for attempt `a`, and when
  the COMMITTING bound passed neither landing evidence nor a positive close of `a` had been seen. Do, in this order:
  (1) look the identity up by the terminal route of C3; an `outcome` line there is the answer (published, refused,
  conflicted, already, withdrawn, rewritten-away, unresolved) and W1 stops after forcing that outcome's ledger line as
  its stop disposition (R14-2), because a final request is never withdrawn and nothing is promised about a side ref;
  (2) create the withdrawal mark, an empty file `lane/decision/<id>.withdraw` (plain) or
  `lane/decision/<id>.withdraw-unresolved`
  (unresolved) with `O_CREAT | O_EXCL`; `EEXIST` is the same outcome. The mark means: no new attempt of this request
  may start, ever; (3) force the stop disposition to completion (the mark's file, then `lane/decision/`), before
  anything else is forced (rule V, decision 14); (4) plain form only: if an attempt of this request is open, write
  `abort` on its pipe (the client never writes `abort` after it wrote `commit`); (5) plain form only: wait for W2 to
  record the withdrawal in the state, at most `WAIT_WITHDRAW` (starting from `STOP_GRACE`), so the capture-visible
  state no longer holds the request (the unresolved form waits for nothing here: its request stays pending until its
  attempt resolves); (6) force the saved-content set of this exit to completion (rule V); (7) look the identity up
  again: an `outcome` line that became durable meanwhile is reported, otherwise print the refusal naming the blocker,
  what is kept and the command to run again, and exit non-zero. Crash: the mark is the only fact; a client killed
  before (2) leaves the request live (the outside-cause rule: a request with no client lands later). Attempt scope
  (R12-1). "The client has submitted" is a fact about one attempt `a`. It ends only when the owner has recorded
  `closed a` after a positively established failure that did not publish (E1 class F); the client then returns to
  FOREGROUND and plain W1 is available again. An attempt whose result is unresolved (E1 class U) never ends that way:
  the client stays in COMMITTING and leaves through the unresolved form. The unresolved form (COMMITTING bound
  passed). The mark forbids every attempt after the one in flight, replay, retry or rebuild included. The owner keeps
  observing that attempt only: if it lands later, the outcome is `published` (the exception the owner approved in
  decision 7, stated in the error); if its child ends with no landing and no positive evidence, the outcome is
  `unresolved` (saved, not replayed; the message names the commit and the command to drop it); evidence of a rewrite
  gives `rewritten-away`; a late positive failure (class F) gives the outcome `withdrawn`, with the same W2 steps as a
  plain withdrawal including the re-basing of descendants. Why no attempt can land after W1 (R11-1). Only one party at
  a time decides a prepared transaction (P2b), and while the client lives that party is the client; W1 is the client's
  own step, so the client has either submitted `commit` (and does not withdraw before the attempt closes) or never
  will. The owner does not decide while the client lock is held, and checks the mark after the client lock is gone.
  After a power loss the stop disposition is what recovery honors, which is why it is made D first, separately, and
  never cut short.
- W2 withdrawn (lane). Pre: the mark exists; checked at C5, B1's end, P1's pre, P2a's pre and again once the pipe's
  reader exists, MG1, Q1, every held-hook completion and L4, so no step starts work for a withdrawn request. Do, in
  this order (T0): (1) append `outcome <id> withdrawn` (or the outcome of the unresolved form when its attempt ends)
  to the lane's ledger unless the ledger already holds an `outcome` line for the identity (the first line wins; a
  later one is ignored by every reader); (2) one S1 replacement making the request `withdrawn` (for the unresolved
  form this step and step (3) run only when E1 resolves the attempt: `published` goes the M1 way, and `unresolved`,
  `rewritten-away` or a late F remove the request with the re-basing below), dropping its consumption entries,
  removing it from the logical tail, and applying the removal rule of C1 (every request whose built commit has the
  withdrawn commit as an ancestor in the tail goes back to `resume = B1` with its deferred selections added) (its
  `ready` record is discarded; it is replayed on the new tail), so the capture-visible head and tail no longer contain
  withdrawn work (R12-6); (3) keep the built commit on `refs/cli-git/<instance>/withdrawn/<id>` best effort (the spool
  is the guarantee). A held hook is not ended (its completion is discarded as in B3). Crash: repeated; L4 reads the
  marks first.

Capture and acceptance (client)
- C0 ensure the lane. Pre: C1 finds the marker set, or an open recovery journal (defined next), or no owner and a state document that does not
  match the journal. An open recovery journal is an S1 intent without its replacement, or a journaled effect outside
  the document (M1, MG3, a carry-forward step, L4) without its completion record, found while no owner is alive; the
  journal of a publication attempt (`publishing`, P2 in flight) is never counted, because a reference-transaction hook
  may run for ever and C0 must not wait for it: a publication attempt is the owner's, and acceptance never waits for
  it. Do: if the lane lock is acquirable, the client starts a recovery helper: a separate process in its own transient
  scope, recorded in `lane/incarnation` as a recovery incarnation, that performs L1 to L4 (it makes one non-blocking
  try of the lock, never waits for it, and when the try fails it exits at once reporting `lost`; when the try succeeds
  it keeps the lock for its whole life, never unlocks, and exits when L4's repair is done without continuing any
  request: continuing requests would spawn hooks and make it an owner). The client is neither the owner nor in that
  scope; it waits for the helper's exit (a pidfd) and then re-evaluates C0 from its Pre, so a recovery that another
  acquirer already completed is seen as done and the client never waits behind that acquirer's later hook (R10-5).
  Every wait of C0 and every re-evaluation draws on CAPTURE (one deadline for C0 and C1 together; R11-9), and a wait
  on a live owner is tied to the marker changing or clearing and capped by MARKER. The next owner's L2 then finds a
  previous incarnation that is dead and a scope that is empty, which is exactly the proof of L1 to L3; no
  relinquishment exists, and no step ever unlocks `lane/owner`. Crash: the helper's death releases the lock, and the
  next acquirer (L1 to L4) repeats the repair.
- C1 capture. Pre: C0 passed, and the publication target is determined (target rule below). Do: sample the marks (a
  `stat` of `lane/decision/<id>.withdraw` for every request in the document's tail and consumption map), then read the
  newest valid record of `lane/state.log` once (taking `g`, the marker, the logical head, the logical tail, the
  consumption map); read the branch tip `B` and the base closure from the immutable objects `B` names; read the real
  index file once (it is replaced by rename, so one read is a consistent snapshot) and the worktree files into the
  spool; an index entry that equals, in every field, the `E` of a live consumption entry of either kind for its path
  is not staging, and any other index entry for that path is staging (a `merge-install` path has its own rule, MG2);
  then read the document again and sample the marks again. Accept only an unchanged `g` with the marker `none`, and
  only if no request that is still in the document has a plain `withdraw` mark in the second sample (R13-7). A plain
  mark on a request still in the document means the lane has not yet applied W2, so the logical head and tail still
  contain withdrawn work and a capture against them would omit its staging; the capture is repeated within CAPTURE and
  ends there with the busy error and no request. Why the second sample is enough: a mark exists before W1 returns, and
  the person's staging that follows W1's return is read after it, so a capture whose index read could include that
  staging is followed by a second sample that sees the mark; a mark created after the second sample is created before
  W1 returns, which is after every read of this capture, so the capture treats the request as pending (the accepted
  consumed-entry limitation while a request is pending). A request with a `withdraw-unresolved` mark is different: its
  attempt may still land, so it stays pending in the tail with its entries live, C1 does not refuse for it, and a
  wedged Git child therefore never freezes the lane's later commits. To keep the selection a capture made under a
  pending base (R14-6), C1 records, for every path it skips because a live `install` entry covers it, whoever owns the
  entry, the path and that entry (its blob is in the owner's spool): the deferred selection. The removal rule (one
  rule, cited by W2, E1 (b), the conflict outcome and `rewritten-away`): whenever a request that P1 put in the logical
  tail ends without publishing (a plain or late-F withdrawal, `refused` at P2, `conflicted` at replay, `unresolved`,
  `rewritten-away`), the lane removes it from the tail, re-enters each descendant at B1, and adds that descendant's
  deferred selections that name the removed request to its delta against the new base, so the staging the capture
  omitted is included; if the request publishes, the skip stays correct. The cost is two sets of `stat` calls over the
  in-flight requests and no sync. The tip is not re-read to validate: a tip that moved during capture does not
  invalidate the capture, because the delta is relative to the logical head and is replayed on the actual tip at
  adoption. Foreign changes to the index or the worktree after the single reads are later staging, not an
  inconsistency, and cause no retry. Crash: nothing durable. Starvation, stated: C1 repeats only when the lane's own
  state changed between its two reads or a mark appeared, and CAPTURE ends that delay with an error and no request; it
  never waits on a foreign program.
- C2 expose the request and bind (revision 14; R13-1). Pre: C1. Do, in this order: (1) draw a fresh invocation nonce
  for this invocation, build the whole request in memory (record, manifest, every captured side), compute its checksum
  and its identity (the content hash, nonce included, plus the lane instance; rule V); (2) create and `flock` the
  client lock `lane/client/<id>`, which the command holds for its whole life; (3) write the request to one file under
  a unique name in `inbox/tmp/` and publish it with a no-replace `link` to `inbox/<id>`; `EEXIST` means the same
  content already arrived (a resend), so check the existing file's checksum and use it. The request is `bound` at the
  link in (3), and not before: FOREGROUND starts there, and because (2) precedes (3) no owner can observe a request
  whose client lock is not held; a request seen with a free client lock and neither mark nor receipt is a request
  whose client was killed from outside. No sync, no directory creation, no binding: the request is V (rule V). Client
  resend rule: after a lost answer a client resends the retained bytes of the request it already built, nonce
  included, and never builds the request again from the worktree; a new invocation of the same operation draws a new
  nonce and is a new request, even when everything else is identical (R12-11). Crash: a temporary with no `inbox/`
  link belongs to a dead client and is removed by the lane after the client's identity is gone; an `inbox/` entry with
  a failing checksum is not accepted and is kept while its client could be alive. Exits before (3) have no request and
  need no withdrawal; an exit after (3) is W1.
- C3 look up an identity (revision 12). Pre: an identity (a resend, W1, or a query). Do: route by the most advanced
  state of the identity, in this order, whatever names the inbox and the queue still hold: terminal (an `outcome` line
  in the named lane's ledger, published by T0: answer the recorded outcome; a `superseded` line is followed to the
  request that replaced it, see B3), marked (a plain `withdraw` mark without an `outcome` line: answer `withdrawn`; a
  `withdraw-unresolved` mark: answer that the original attempt may still complete and to check `git log`, never that
  nothing can land), adopted (an `adopt` line in the order file, or the record in `queue/`), accepted (the file in
  `inbox/`), unknown. The terminal route reads the ledger only; it never opens the request file, because T1 may have
  released it. A lookup that chose a route which opens the request file and finds it gone restarts from the terminal
  route; it neither fails nor accepts again. Rule D applies to the line or name the answer relies on only when the
  asker departs on it (rule V). Crash: nothing durable is written by a lookup.
- C4 handed over. Pre: C2. Do: answer `handed over` as soon as the request is V; the answer promises survival of a
  process kill, and D follows by the syncer or by the departure barrier (rule V). Crash: a client that dies before the
  answer leaves a V request that the lane adopts and the outside-cause rule finishes.
- C5 adopt (lane). Pre: accepted, no `adopt <id>` yet, L4 done. Do: read the inbox file and verify its checksum
  (revision 12: a file that fails it is not adopted; no sync is issued, rule V); if the withdrawal mark exists, append
  `adopt` and go to W2; journal the intent; append `adopt <id> <crc>` to `lane/order` (this line is the authority);
  link the record into `queue/`; S1 replacement adding the request to the logical tail and the map with `resume = B1`
  (for a merge `resume = MG1`). The hold ref `refs/cli-git/<instance>/hold/<id>` (T1: extra protection, never a
  precondition of building or publishing) is no longer created here (revision 12, performance): it is created, in one
  batched `update-ref --stdin` per owner wake-up, only for a request that waits longer than `HOLD_AFTER` (unmeasured;
  a request published within it never needed one). Crash: L4 completes the link and the S1 replacement; a second
  `adopt` is never appended.

Publication target (applies to C1, P1 to P4 and M1). The target ref is the branch `HEAD` names; when `HEAD` is a symbolic reference to a branch
that does not exist yet (an unborn branch), the base `B` is the empty tree, the commit has no parent, and P2 is `update-ref <ref> <new>
0000...` (the zero value means "the ref must not exist"); when `HEAD` is detached the target is `HEAD` itself and P2 is
`update-ref` of `HEAD` without `--no-deref` (R10-12: the update follows what `HEAD` means when it runs, with the old-value check against the resolved value, as a native commit does; there is
no check-then-act on `HEAD`'s type), the reflog anchor and R3 read `logs/HEAD` and the branch's log when `HEAD` has become attached, and the attachment guard for M1 and the merge transitions is: `HEAD` detached in both reads and its value equals `new` or has `new` as an
ancestor (the wrapper's own successful `HEAD` update moves the detached value to `new`, so comparing with the captured value would decline it).
Publication authority is the request's recorded target, re-checked at the start of every attempt: for an attached target the compare-and-swap on the
recorded ref stands whatever `HEAD` names now and M1 declines when `HEAD` no longer names it; for a detached target `HEAD` must still be detached, and
if it is symbolic now the commit is saved on `refs/cli-git/<instance>/pending/<id>` and, with a client attached, the request is withdrawn by W1 with that error (a request with no client is `pending`) (it cannot be
published to a detached `HEAD` that no longer exists, and `--no-deref` is never used: a `HEAD` attached between the check and P2 makes the update land on the branch it names, which is what a native commit would do, and M1 then declines as the guard says). The logical head, the recovery classes and the ledger use the target ref in
place of the branch name. A repository state that has no such target (a bare worktree without `HEAD`) is not a lane.

Client result (revision 12; three facts and the owner's bound answers). `handed over` is the acknowledgement of C4
(the request is V; rule V), printed as soon as it is true; it is not an exit. The command's exit is the foreground
outcome: `published` (exit 0, the commit named, printed after the receipt of P2c and after the single attempt at
materialization within `WAIT_MATERIALIZE`; it does not wait for P3, T0 or any sync); `withdrawn` by W1 at a bound or
by the person's interrupt (non-zero, naming the blocker, what is kept and the command to run again; nothing lands
later by itself); `refused` by a person's hook, a failing required filter or a policy error; `conflicted`; a signer
failure (B1 (iii), then W1; non-zero; the work stays on disk and a signer failure is on the owner's outside list); the
unresolved COMMITTING exit (non-zero, result not known, owner decision 7). `queued` is the exit-0 answer of a push
only. `retired` is eventual, recorded in the ledger, and never awaited by the command.

Build and publication (lane)
- B1 build. Pre: adopted (or Q1 resumed with `resume = B1`). Do: build the commit by replaying the captured delta on the logical tail (an object
  write; no ref, index or worktree change). Outcomes of the build, each recorded in one S1 replacement and each with a defined client result:
  (i) `conflicted`: the replay on a moved tail meets a content conflict no resolver settles, the owner's "stops as any
      merge does": the request ends `conflicted` (the removal rule of C1 applies to its descendants), the conflict
      tree is saved on `refs/cli-git/<instance>/conflict/<id>` best effort and the spool is kept, its consumption
      entries are
  released (nothing was installed, so the staging stays the person's), the client result is nonzero with the conflicting paths named, and the request is
  saved until the person disposes of it with the command the message names; (ii) `refused`: a required clean filter or a hook failed, the owner's "Allowed
  refusals": the request ends `refused` with the program's own message, entries released, nonzero, nothing changed; (iii) `sign-failed`: the configured
  signer failed (an outside cause): with a client attached the request waits for the signer to recover within FOREGROUND (a trigger is the next try), then is
  withdrawn by W1 naming the signer, with the work on disk; with no client it is `pending` with `resume = B1`; (iv) `held`: a hook is configured for the build and the
  request has no `hook-ok` for its identity: spawn it under L6 through a runner (below); the request has no commit, is not in the logical tail,
  and keeps its consumption entries; (v) `ready`: record `ready <id> <attempt> <commit> <base>` where `commit` is the built object and `base` the tip
  it was replayed on (the object is in the spool closure and the record names it). Crash: before the S1 replacement the build is repeated (an object
  write, idempotent).
  The hook runner (hooks once per identity, default H-version): the hook is run by a small runner inside the incarnation's scope that, when the
  hook's execution tree is empty, writes the result (`hook-result <id> <status> <stderr>`) to a file in `lane/` and fsyncs it before the owner reads it.
  Success is recorded as `hook-ok <id>` in the S1 replacement that ends `held`; a later replay of the same identity on a moved tip (Q1, P4) builds
  with `hook-ok` present and does not spawn the hook again, so there is no held, success, replay, held loop. The crash exception to H-version, stated: if the owner
  dies before `hook-ok` is durable, the result file may not exist, and the next owner's L2 ends the previous scope, hook included; the hook is then run
  again, one new invocation for each recovery episode that lacks a durable result (revision 11; R10-11: not a total of one; repeated deaths before a durable result repeat the
  hook, see owner choice H), and the notice says that it ran again because its earlier run did not finish recording. A result file that is durable is
  honoured, so the hook is not re-run when its completion was recorded.
- B2 ordering exception. Pre: `R1` held. Do: a request `R2` whose changed-path set is disjoint from `R1`'s (adds, edits, deletions, modes,
  directory-versus-file conflicts) and that is not a conclusion or an amendment of anything held proceeds ahead of `R1`; an overlapping
  `R2` waits. When `R1` completes: success records `hook-ok` for that version and re-enters it at B1 at the tail (replayed on the then-tip, with no second hook run); refusal ends it as `refused` with the
  hook's message and changes nothing. A completion carries (request, version) and is accepted only for the active version; the
  completion of a superseded version is discarded. Crash: as B1.
- P1 publish-begin. Pre: `ready`, B2 clear. Do: journal `publishing <id> <attempt> <old> <new>` with the reflog anchor and, for a
  repository with a `reference-transaction` hook, `hook-dispatch`; the request enters the logical tail in an S1 replacement of this same step (so a capture sees it in the tail from the moment `publishing`
  is journaled), with no marker.
  Crash: the next owner treats the attempt by the recovery classes (R1 to R5 below).
- P2 swap (revision 12: the party that waits for the publication decides it; revision 13 states what the pipe does and
  does not decide; R11-1, R12-1, R12-2). Pre: P1, L6 held for the ref-transaction child, and no withdrawal mark. A
  durable `commit` decision is not used: revision 11's decision file selected permission to publish and not the
  publication. P2a prepare. Create the attempt directory `lane/p2/<id>.<attempt>/` (V only; recovery reads it as
  evidence of the attempts, E1, and removes it only after T0; a path is never reused, because a late writer must not
  be able to address a later attempt) with a named pipe `in`, a file `out` for standard output, a file `result` and a
  file for Git's trace events (E1). Pipe bootstrap, as a specification to check by experiment: open the read end
  `O_RDONLY | O_NONBLOCK` (it succeeds with no writer), open the write end `O_WRONLY | O_NONBLOCK` (it succeeds
  because a reader exists), which is the owner's keeper (below) and, like every writer's, is nonblocking, so no
  protocol write can block the owner's event loop (R14-3); every other writer also opens `O_WRONLY | O_NONBLOCK`, so a
  dead reader gives `ENXIO` and not a blocked open, clear `O_NONBLOCK` on the read end, pass it as the child's
  standard input, and close the owner's copy of the read end after the spawn; every other descriptor is close-on-exec.
  Start `update-ref --stdin` (never `--no-deref`; a detached target names `HEAD`) with `LC_ALL=C` and `out` as
  standard output; check for the mark once more now that the reader exists (a mark found closes the write end and ends
  the attempt as W2); write `start`, `update <ref> <new> <old>` and `prepare`. Pipe discipline (R13-3): the pipe must
  never read end of input between instructions, so the owner holds the keeper write end for the whole attempt, and
  closing it is a protocol action: the owner closes it only after the closing line has been executed (landing evidence
  or `abort: ok`) or the child has ended, and its death closes it (end of input aborts a prepared transaction when no
  other writer is open and the buffer is empty). Every other writer (the client) opens `O_WRONLY | O_NONBLOCK`, writes
  one line, and closes at once, so the client never keeps the pipe open idle. Every line is one `write` of at most
  `PIPE_BUF` bytes, so it is all or nothing; `EINTR` is repeated, `EAGAIN` (a full pipe) is retried as a whole line
  until the remainder of the applicable bound, whose clock starts at the first attempt and is not renewed when the
  event loop re-enters, `SIGPIPE` is ignored in the client and the owner and `EPIPE` is handled, and a line counts as
  submitted only when `write` returned its full length (so a failed `commit` write means the client has not submitted,
  and W1 is plain). The client records `submitted a` only after the full write returned and never before; a client
  that dies between the write and the record leaves the fate to E1's evidence, and E1 treats "never submitted" as true
  only for an attempt whose client recorded `not-submitted` or that never reached `prepare: ok`. Closing the keeper is
  not the same as reaching end of input: another writer, or a `commit` already buffered, still prevents the intended
  abort. If the owner cannot enqueue `abort` within `STOP_GRACE` it closes the keeper, which ends the transaction when
  Git reads end of input. `update-ref` reads one line at a time and finishes each command before it reads the next
  (`builtin/update-ref.c:712-770`), so a line written while a hook runs waits in the pipe; stdio may read ahead into
  its buffer, but dispatch is still in order. `prepare` takes the ref lock and runs the person's
  `reference-transaction` hook in `preparing` before the locks and in `prepared` after them (`refs.c:2713-2726`); it
  prints `prepare: ok` (`update-ref.c:621-631`) to `out`. While prepared, any line but `abort` or `commit` makes
  `update-ref` die (`update-ref.c:760-763`). P2b decide. The pipe orders instructions; it does not make an
  instruction, its execution and the ref write one event. What the order gives, for one live transaction process and
  writes as above: `update-ref` executes the first closing line it reads, and a second closing line only makes it die
  with "transaction is closed" after the first has run (`update-ref.c:765-768`) when the first handler returned
  successfully. So `commit` then `abort` does not undo the commit, and `abort` then `commit` does not commit that
  transaction; a successful write is not proof that Git has read, begun or finished the instruction. The decider is
  the attached client while its client lock (a `flock` on `lane/client/<id>`, held for the life of the command and
  released by the kernel at its death) is held, otherwise the owner. The client reads `prepare: ok` from `out` (a
  change notification, not a poll), and if at least `COMMIT_MARGIN` (starting from `WAIT_COMMITTING`) of its
  FOREGROUND budget remains, records `submitted a` in its own state and writes `commit`; otherwise it runs W1, which
  writes `abort`. The owner decides only for a request with no client: it writes `commit` on `prepare: ok`, or `abort`
  when a mark exists. When the client lock is released with no mark and no `outcome` line (the client was killed from
  outside, so a normal exit, which always leaves a mark, is told apart), the owner becomes the decider and writes
  `commit`: the request now has no client and may land later, which the outside-cause rule allows. If no closing line
  appears within `PREPARED_HOLD` of `prepare: ok`, the owner writes `abort`. PREPARED-HOLD bounds when an instruction
  is enqueued, not when the lock is released: `abort` is executed when Git reads it, and a Git child that never reads
  again is a wedge of Git's own (not on the owner's outside list; the C-kind pick exempts `update-ref` from X1), so
  the register states the residue and no process of ours waits on it: the client has already left, and other programs'
  wait on the branch lock is theirs. P2c commit. `update-ref` executes `commit` (the backend writes the ref, then runs
  the `committed` hook, `refs.c:2783-2785`), so the ref can be written before `commit: ok` is printed. The landing
  evidence is `commit: ok` on `out`, or the tip equal to `new` or having it as an ancestor (R2); the client creates
  the receipt `lane/receipt/<id>` (an empty file, `O_CREAT | O_EXCL`, V) on the first of them and only then prints
  `published`; recovery treats a receipt as published (R1), so a successful return cannot be followed by a fresh
  attempt after a process crash (R13-1). After a power loss a lost receipt is the weaker durability of decision 10 for
  a published exit, and the landing evidence of the tip decides. The owner records the end of the child in `result`
  and then `closed a` or `unresolved a` (E1). `commit` failing makes `update-ref` die on standard error with no line
  on standard output (`update-ref.c:645-658`). The exception the owner approved (decision 7): the client has submitted
  `commit` and neither the landing evidence nor a positive close has been seen when its COMMITTING bound passes. That
  is an instruction already submitted, whether Git has read it or not, and not only a disk write already begun. The
  client leaves by the unresolved exit of W1. The extra wait `WAIT_COMMITTING` improves the observed outcome of a slow
  write; it is not a correctness bound. Owner death: the child is ended by the parent-death signal that L5 sets and is
  reaped by L2; Git removes its own lock files when it ends on a catchable signal (`lockfile.c:83`, `tempfile.c:93`),
  a backend obligation to check for the signal L5 sets; a lock stranded by an uncatchable kill is the residue of
  choice R (the lane never removes a ref lock). End of input aborts a prepared transaction only when every write end
  is closed and the buffered input is consumed, and a `commit` already in the buffer is not cancelled by later end of
  input. After `commit` was written, `update-ref` finishes on its own; recovery classes R1 to R5 decide what happened,
  and the mark rule above forbids a new attempt after a normal exit. Nothing but L6 and the lifetime owner lock
  `lane/owner` is held across P2. Crash: the recovery classes apply; a request with an attached client waits as in
  FOREGROUND for the next attempt (Q1) only when its last attempt closed by class F, and W1 stays available.
- P3 published. Pre: P2 succeeded. Do: in one S1 replacement record `published` and update the logical state (no marker: nothing outside the document changes). Crash: recovery class R2.
- P4 swap-failed. Pre: E1 classified the attempt as class F (positive evidence only: owner death and an enqueued
  `abort` do not establish it), for example `store-failure`. Do: in one S1 replacement record
  `swap-failed <id> <attempt> <class>`, keep the request live with `resume = B1` (`MG1` for a merge) and write the
  pending side ref best effort; the foreground command keeps waiting (FOREGROUND) and a later attempt is made by Q1 on
  a trigger, one at a time, each a full B1, P1 to P3 cycle with a higher attempt number. When the command reaches the
  FOREGROUND bound it is W1 and the request never lands later. A request without a client (a service-originated merge,
  a request whose client was killed) stays `pending` and is retried by Q1. Crash: recovery classes below.
- Q1 resume a pending, landing or deferred request, by its `resume` point (revision 10; R9-10). Every request that waits records, in the S1 replacement
  that makes it wait, a `resume` value: `B1` (a commit or conclusion waiting for the branch or for a signer), `MG1` (a
  merge waiting for a moved tip or a lost swap; nothing was written, revision 12), `MG3` (a published merge whose
  install waits for a foreign `index.lock`), `MG6c` (a cleanup after a detected switch, waiting for `index.lock`),
  `M1` (a published request whose index follow-up is deferred). A request with a withdrawal mark is never dispatched
  for publication work (W2 is the only transition that acts for it; the cleanup transitions MG6c and L4's completion
  of a journaled effect are not publication work and run for it too). Pre: a trigger holds: the lane observed the
  branch tip differ from
  the request's `base`, or a branch lock or `index.lock` was observed free (a stat and a read of the tip, no more), or the index entry of a waiting path
  changed, or a client captured or the next command of the lane arrived and found such a request. Do: dispatch on `resume` only. `B1`: in one S1
  replacement set `base` to the then-tip, increment `attempt`, discard the old `ready` record, and re-enter B1 (with `hook-ok` kept, so no hook is
  re-run); P1 follows from `ready`. `MG1`, `MG3`, `MG6c`: re-enter that transition. `M1`: re-enter M1. The update is
  the replacement. Crash: recovery takes the longest valid hash-chained prefix of the state log, so a crash leaves the
  old state or the new one.
- E1 classify a P2 exit (revision 13; R10-10 replaced the stderr text with Git's own channels, R11-1 split the unknown
  class, R12-3 replaces the negative inference). Pre: P2's child ended, or `prepare` returned, while this incarnation
  was alive to observe it. Do: classify only from evidence that Git itself produces and that a hook or its descendants
  do not share: (i) the report lines on `out` (`start`, `prepare`, `commit`, `abort`: ok; a hook's own standard output
  is redirected to its standard error by Git, so it cannot reach this file, a backend obligation); (ii) Git's trace2
  event stream (`GIT_TRACE2_EVENT=<path>` in the attempt directory), which records `child_start` and `child_exit` for
  each hook with its exit code; only events that Git wrote about its own child count; (iii) the exit status; (iv)
  whether `commit` was submitted, which only the decider knows and records first. The standard-error text is shown to
  the person and is never parsed for a decision. The class is recorded in the next S1 replacement before anything else
  is done. There are three kinds of result and they are never collapsed: P, positively published: `commit: ok` seen,
  or the tip equals `new` or has it as an ancestor (R2); a non-zero exit after either is not a failure; or R3 (a
  reflog entry after the anchor whose own recorded old value is `new`): `rewritten-away`. F, positively failed before
  publication, which alone allows the attempt to close and a later attempt to start: `commit` was never submitted and
  the child ended (so nothing was executed); or `abort: ok` was seen; or a `child_exit` of the `reference-transaction`
  hook with a non-zero code during `preparing` or `prepared` (the hook refused: record `refused` with the hook's own
  standard error, nothing was changed, no retry of the hook; the removal rule of C1 applies to the request and its
  descendants); or `prepare` failed with no `prepare: ok` and a non-zero exit (the ref store refused the update: a
  held lock, an old value that no longer matches, or another store failure; class `store-failure`, P4, retried only on
  a trigger). U, unresolved execution after submission: `commit` was submitted, and neither P nor `abort: ok` nor a
  positive failure report was established (the child ended or is still running, the tip does not contain `new`, there
  is no reflog evidence). An ended child and an absent tip do not prove that nothing was published: another writer may
  have published `new` and reset the branch away before the child died, with no `commit: ok` printed. So U is never
  retried by a live owner, whatever N-silent concedes (N-silent covers a retry after a proven store failure, class F,
  and nothing else); with a client attached it is COMMITTING and then the unresolved exit; with no client it is a
  crash at P2 by the recovery classes (R5: owner decision 3). Evidence unavailable or inconsistent (no events, a
  signal, a result the owner could not record, the owner died) is F only if `commit` was never submitted, and U
  otherwise. X1 never cancels P2's child (update-ref is exempt). Crash: as P2. Residue stated: a refusal not yet
  recorded when the owner dies is later judged by the recovery classes, so the hook may run again at the next attempt
  of a request that still lands. Backend obligations (step 1): the trace2 events and the report lines exist in the
  shapes assumed on each supported Git version and backend; hook descendants that hold the error stream cannot alter
  them; a nested Git process started by a hook does not write to the same trace destination (isolate it explicitly);
  the lane-private path is not a security boundary against a same-user program; result handling does not wait for end
  of file on standard error from descendants after `update-ref` has exited; the pipe, the line order, the
  first-closing-line behavior (including a failing first handler), the explicit-`start` end-of-input abort versus the
  implicit transaction, the hook phases and the lock lifetime behave as stated on the files and reftable backends. The
  cited lines are from the 2.56.0 source read by the author; the review that follows has not inspected them, so they
  remain gates.
- Recovery classes for an attempt, in this order. R1 a receipt, a recorded `published`, or `commit: ok` in the
  attempt's `out` file: published. R2 the new commit is the tip or an ancestor of it:
  published. R3 a reflog entry after the anchor whose own recorded old value is the new commit: `rewritten-away` (saved, not republished); this
  is checked before any `swap-failed`, because a failed invocation does not say the request was never satisfied. R4 is withdrawn (revision 11; R10-8): a
  `swap-failed` recorded by an owner that has since died proves only that this invocation wrote no ref, not that no other writer satisfied the request and
  removed it, and the design has no oracle for an erased reflog suffix. R5 anything else, including every attempt found after an owner's death and every
  `swap-failed`: `landing`, owner decision 3: replay and land with a message naming the commit and the command to drop it. Only a request whose client is gone or
  that has no client lands this way; with a client attached the client's W1 withdraws it first (an attempt that lands
  needs the client's `commit`, P2b). The retry of a live owner after a proven `store-failure`
  is Q1 and not a recovery class; its concession is owner choice N.

Materialization and retirement
- M1 materialize (one path for every follow-up, whether the lock is free now or later). Pre: `published`, the attachment guard,
  and a single try of `index.lock` after the publication (never before it, never across P2). Do: (1) journal the intent to try (informational only: it
  proves that an incarnation meant to create a lock, nothing about whether one was created), then create `index.lock` with `O_CREAT|O_EXCL`; `EEXIST` is
  the single try failing: a foreign program (or a stranded lock, see Crash) holds it, the request keeps its entries and a later single try is made by Q1;
  (2) with the lock held read `HEAD` before and after reading the current index and decline unless the attachment guard of the Publication target rule holds
  in both reads (declined: release the lock by the normal abort of rule N, then M2); (3) S1 replacement setting the marker `materializing` and recording the
  plan: for each path, `E`, the target entry, and the outcome the entry rule computes; (4) apply the per-path entry rule to the index read under the lock, write the
  new index into the lock file, fsync it; (5) rename `index.lock` over the index; (6) apply M3 (c), record the per-path outcomes in the closing replacement
  and replace the document with the marker `none`. Per-path outcomes, one each and recorded with the request: `installed` (the entry was `E` and the worktree
  held the target; staged), `already` (the entry already equals the target; nothing written), `preserved` (the entry is neither `E` nor the target: the person's
  own staging; left alone; the entry is dropped because there is nothing to suppress), `declined` (the entry was `E` but the worktree is not the target, or the
  branch moved on: left alone and the entry becomes `suppress`, as in M2). The request is `materialized` when no path is `declined`, else it is `materialized`
  with the declined paths named as voided.
  Crash (revision 11; R10-1): recovery infers nothing from the lock. Neither the absence nor the presence of `index.lock`, nor a journaled intent, says whether the rename
  happened or who made a present lock. L4 re-reads the real index once (no lock) and re-evaluates every path of an open M1 against the recorded plan: an entry equal to
  the target is `already` (installed by the dead incarnation or never needed: the same result), an entry equal to `E` was not installed, so the entry
  stays (`install`) and M1 is deferred to a later single try (Q1, `resume = M1`), any other entry is `preserved`. An intent whose lock was never created leaves
  the install entry in force and so keeps suppressing the person's old entry. A lock that is present is reported without a claim: "an `index.lock` exists; this tool's
  dead incarnation may have made it; no process of that incarnation holds it; remove it if no Git process is running", and is never removed (rule N). M1 is idempotent because
  the entry rule is conditional on the entry read under the lock. This narrows the guarantee as owner question A does: a lane killed inside the window can leave
  a lock that only a person clears, as Git itself leaves one after its own kill.
- M2 void. Pre: the guard declined, the branch was rewritten (the commit is not reachable from the tip), or a later foreign successor made
  the entry rule decline. Do: in one S1 replacement record `voided` for the request and the paths, end the installation authority (kind
  `install` becomes kind `suppress`, keeping `E`), and tell the person which paths were left alone. Nothing is released by voiding: an
  index entry the person did not change must not return as staging in an unrelated commit. Crash: repeated; the conversion is idempotent.
- M3 release a suppression (by observable state, never by time or by the end of a request). Pre: a `suppress` or `install` entry for a path.
  Do: inside any S1 replacement the lane makes anyway, drop the entry when (a) the entry of the path in the current branch tip's tree equals
  `E` in mode and object (so index and head agree and there is nothing to suppress), or (b) a later published request consumes that path (its
  own entry replaces this one), or (c) an `install` entry was applied by M1 (its path is now at the target). The lane never reads the index
  for this; C1 alone tests an index entry against `E`, so a person's staging of the path (any entry unequal to `E`) is captured and the
  entry has no further effect. The one case this cannot tell apart is the owner-approved concession of decision 2: a deliberate restage to
  exactly `E`. Crash: repeated.
- B3 supersede a held version (revision 10: identities, path sets, retirement). Version identity: an amendment of a held request is a new request
  with a fresh identity `id2` whose record names the superseded `(id, version)`; `id` keeps its immutable content, so C3's rule (a different hash
  under an existing identity is rejected) stands and a resend of `id` with changed content is never a "new version". The consumption entries and
  `hook-ok` are keyed by request identity. Pre: `id` is `held` and `id2`, naming it, is adopted (C5). Do: in the S1 replacement that adopts
  `id2`: record `id` as `superseded by id2`; replace the consumption entries of `id` by the entries computed from `id2`'s own capture, by path:
  a path in `id`'s set and not in `id2`'s releases its entry (that staging was consumed by an abandoned version and returns to the person as
  ordinary staging, which C1 captures in a later commit); a path in `id2`'s set and not in `id`'s acquires an entry from `id2`'s capture; a path
  in both takes `id2`'s pre-entry (C1 of an amendment treats the entries of the request it supersedes as its own staging and not as suppressed,
  so `id2` can capture the paths it re-includes). Append `outcome <id> superseded by <id2>` to this lane's own ledger
  first (T0), and only then make the S1 replacement. The hook execution of `id`
  is the person's program and is not ended: its completion, success or refusal, is read and thrown away, and the gate is held until its
  execution tree is empty (L6). Release: T1-nocommit for `id` (ledger line exists, execution tree empty, no entry remains), spool released then.
  C3 lookup: the terminal route for `id` follows `superseded by` to `id2` and answers `id2`'s state, so "a terminal line exists for this
  identity" is not enough and a resend of `id` is answered as superseded by `id2`. Crash: repeated; the ledger line is idempotent per `id`.
- T0 publish a terminal outcome (revision 12; R10-7, R11-10). Pre: a request reaches a final client-visible outcome:
  `published` (P3), `refused`, `conflicted`, `withdrawn` (W2), `already` (a merge that a foreign successor made a
  no-op), `rewritten-away`, `unresolved` (terminal, non-zero, saved: its spool and commit are kept until the person
  disposes of them), `superseded by` (B3). Every transition that ends a request this way uses T0 and no other order.
  Do, in this order: (1) append `outcome <id> <kind> <detail>` to this lane's ledger unless the ledger already holds
  an `outcome` line for the identity; the first line for an identity is the outcome and any later line is ignored by
  every reader, so outcomes are absorbing and idempotent; (2) the S1 replacement that records the outcome and removes
  the request from `lane/state` (a published request stays in the document until M1 or M2 has finished with its
  consumption entries, and a published merge until its `attach-watch` entry settles, but its outcome line is already
  in the ledger). The outcome line is the lookup route of C3: reporting an outcome does not depend on being eligible
  to release the recovery data, which is T1's separate and later operation (`retired <id>`). Crash: the ledger line
  without the replacement is completed by L4; a replacement without the ledger line cannot occur because the line
  comes first. The ledger append needs no rescan: the owner keeps the end offset and the last valid line of its own
  ledger from L4's single read (revision 12, performance).
- T1 retire (revision 12; R11-3). Pre: the request has a terminal outcome already published to the ledger (T0), its
  terminal ledger line and the log record that removes it from the state are D (rule V, reclamation barrier), and the
  maintenance registry, listed once as `L0` (the names in `maintenance/`) together with the highest reservation number
  `n0` (valid only if the numbers are dense from `maintenance/floor` to `n0`; a gap means a reservation in progress or
  a missed name, and T1 withdraws), has every entry `terminal` by G3 (an entry that is not `terminal` blocks, whatever
  its scope: an entry with an empty or absent scope and a live launcher is not terminal, because that launcher may
  still launch). Two kinds. T1-commit, for a request whose commit is in the object store and published (materialized
  or voided). T1-nocommit, for a request that has no commit to publish (`superseded`, `refused`, `withdrawn`,
  `conflicted`, a held version): the hook execution tree of the version is empty and no consumption entry remains for
  it; it needs no connectivity check. A request that is saved (`pending`, `landing`, `rewritten-away`, a `withdrawn`
  or `conflicted` request whose inputs the person has not disposed of) never loses its spool. Do, in this order: (1)
  write `releases/<instance>/<id>` (the release intent) by the link method of C3, V; (2) for T1-commit, run the
  connectivity check from the tip; (3) after the check has ended, probe the name `maintenance/<n0 + 1>` (one `stat`;
  reservations are numbered consecutively, G1) and list the registry again as `L1`, and require that the probe finds
  nothing, that `L1` has the same names as `L0` and that every entry read in `L1` is `terminal`; any other result (a
  name in `L1` that is not in `L0`, whether or not it is terminal, or an entry not `terminal`) withdraws the intent
  and keeps everything, and a later T1 repeats from its Pre with a new `L0`; (4) append `retired <id>` to this lane's
  ledger; (5) release the spool; (6) remove the hold ref if there is one; (7) remove the intent. Crash: the intent
  names an unfinished release; L4 keeps the intent and repeats T1 from its Pre with a fresh `L0` (never from step (2)
  with a baseline lost in the crash), so the comparison always brackets the check it follows; the ledger line is D
  before any release, so retention is never released first. Retirement is retention, not a wait: nothing blocks on it,
  and it may be deferred for as long as a run keeps appearing (the cost is disk). Why a late maintenance admission
  cannot take the closure (revision 12; R11-3). Revision 11's proof read "terminal at the final registry read" as
  "finished before the connectivity check", and it is not: a run admitted after T1's Pre can take its view while the
  branch is elsewhere, delete after the check has passed, and be `terminal` at the reread. The argument is now a
  comparison of two listings around the check. Let `X` be any destructive execution of a wrapper-started run. (a) `X`
  is in `L0`: it is `terminal` there, so all its deletions precede its `terminal` record (G3), which precedes `L0`,
  which precedes the check; the check ran after every deletion of `X`. (b) `X` is not in `L0` and is in `L1`: the sets
  differ, T1 withdraws, whatever the state of `X`; this is the eleventh review's schedule, a run that was admitted
  after the Pre and became terminal before the reread. (c) `X` is in neither: its reservation (durable and visible
  before G2 and before its first view, G1) became visible after `L1` began listing, which is after the check ended, so
  every view of `X` comes after the check found the commit reachable from the tip; `X` can lose the closure only if a
  person rewrites the branch after the check (the stated residue; the reflog keeps the object for the person's own
  expiry). Implementable observation boundary: registry names are created once by `link` and are never removed while
  any release intent that predates the entry's `terminal` record exists (a compaction of old terminal entries is later
  work and must keep that condition); the argument does not rest on `readdir` alone (R12-10): the probe of the next
  consecutive name is an exact test for "no reservation was made after `L0`" because every new reservation claims the
  next number by a no-replace `link` (a claimer that loses retries with the next number), so a reservation made after
  `L0` is found by the probe or by `L1`, and one not yet linked takes its views after the check. Compaction removes
  only a prefix of old terminal entries, after writing the new lowest number to `maintenance/floor`, and only when no
  release intent that predates their `terminal` records exists. The hold ref is extra protection where the ref store
  allows it, advanced best effort; its absence changes nothing in this argument. Obligations for step 1 and for the
  backend: G1 is visible before G2 for every wrapper-started run; a run takes no reachability view before it is
  launched; a deletion by a run precedes its `terminal` record.
  deletion by a run precedes its `terminal` record.
Merges (revision 13: publish first, the owner's decision 8, "Branch first, files after"). A merge is a request of the
same lane, with the same adoption order, withdrawal, publication and
terminal transitions as a commit. What differs is its content `M` and its derived follow-up: the install of `M` into
the index and the worktree, which is derived data after a published commit, like M1 (it is not a commit landing later,
and nothing waits for it). The order was checkout first (owner choice M-A, delegated); the owner's bound answers make
it costly, because a command that is withdrawn at its bound after the files were written must restore them through an
`index.lock` that a foreign program may hold for as long as it likes (R11-2). Publishing first makes rule B's claim
true by construction: before the branch is written, nothing of the person's index or worktree has changed, so W1 at
any point before P2 changes nothing and needs no restoration. Two authorities are kept apart. Authority to do: with
`index.lock` held, `HEAD` is attached to the request's target ref in both reads around the index read, or, for a
detached target, is detached and `M`'s base is an ancestor of its value. Authority to undo: the lane may reverse its
own recorded installs on a path whose current file is still the installed file it recorded (same inode as the pinned
`R`, digest unchanged) and on index entries still equal to `M`'s, whatever `HEAD` names now; it needs only the single
try of `index.lock` and never needs authority to do. A merge that waits before publication (a held swap, a moved tip)
is a FOREGROUND wait with `resume = MG1`; at its bound it is W1, which changes nothing.
- MG1 compute. Pre: adopted, the merge request, no withdrawal mark. Do: compute the merge commit `M` from the logical
  tail and the other side with `merge-tree` (an object write; no ref, index or worktree change; X1 may cancel it).
  Record in one S1 replacement `merge-ready <id> <M> <base>` with the path set `P`: for each path the merge changes,
  `M`'s entry and the tip's entry `O`. If `merge-tree` reports a conflict that no resolver settles, the outcome is
  `conflicted` (non-zero, the merge stops with the conflict recorded; nothing in the index or the worktree is changed;
  the spool is kept). There is no pre-scan of the index and no wait for staging to change: the owner's decision P,
  "Merge anyway, unstage x", is applied by MG3. A waiting merge does not block later requests. Crash: repeated.
- MG2 publish. Pre: MG1 done. Do: P1 to P3 for `M` with `old = base` (`M` enters the logical tail in P1's replacement
  as for a commit). The P1 replacement also reads the real index once (a plain read, as C1 does) and records, for
  every path in `P`, a consumption entry of kind `merge-install` holding `E`, the attempt's installation pre-entry
  (the index entry read at P1; it is a value guard, not proof of what was staged when the ref was written), `O` (the
  pre-merge tip's blob) and `M`'s entry. A lost swap or a held ref lock: P4 with `resume = MG1` (the merge is computed
  again on the new tip; nothing was written, so no checkout state needs reconciling). Refused by a hook: `refused`,
  nothing changed. The window between the publication and the install (decision 8, "Branch first, files after") is
  stated: the branch holds `M` and the index and worktree still hold the old tip until MG3 runs. The wrapper's own
  captures use these rules for a `merge-install` path, by request kind (R12-13, R13-8), and in every case the capture
  is the person's edit replayed on `M` and never a complete replacement of the file:
  - plain commit: an index entry equal to `E` is not staging; an entry `S` that differs is the person's staged edit,
    captured as the three-way merge of the content of `E` (the base of what they staged), `S` and `M`, so staging
    `b = 1` in a lagging file where the merge changed `a` commits both and cannot revert `a`;
  - `commit -a` and a named path: the person's edit is the delta from `O` to the worktree file `W`, replayed on `M`
    (the three-way merge of `O`, `W`, `M`), and nothing is captured for the path when `W` equals `O`;
  - `W` equals `O` is decided from stat data only when `E`'s blob equals `O`'s blob and the entry is not racily clean
    by Git's rule and the path needs no conversion; otherwise by hashing `W` (after the repository's conversions,
    through the same eligibility gate as the in-process add) and comparing with `O`'s blob; a stat match against an
    `E` that differs from `O` proves only that `W` matches `E`. A later published request that consumes the path ends
    its `merge-install` entry (M3 rule (b)), so MG3 does not write that path again. A native `/usr/bin/git commit -a`
    run outside the wrapper in the window commits the reversal of the merge on top of it; the history keeps both.
    Foreground contract (R13-9): the command waits for materialization under `WAIT_MATERIALIZE` (register), because a
    script that runs `git merge && make` must see the files; at its bound, or when a foreign `index.lock` defers MG3,
    it exits 0 with `published, files and index will follow`, the install continues under the service (the
    `merge-install` entries and `resume = MG3` are its recovery authority), and the exit says so.
- MG3 install (derived data after a published merge, with a real obligation to preserve). Pre: `published` (P3, or the
  evidence R1 or R2) and an outstanding `merge-install` entry. Do: a single try of `index.lock` (`EEXIST`:
  `resume = MG3`, tried again by Q1 on a trigger). With the lock held read `HEAD`, the real index and `HEAD`; if
  authority to do fails, release the lock by the normal abort of rule N, write nothing, turn the entries into
  `suppress`, and tell the person the merge is on `<ref>` only and the worktree was not touched. Else plan every path
  of `P` from the versions read under the lock, the target being the entry of the path in the current tip's tree (the
  rule of M1): `O`, `S` the real index entry (all fields, stage included), `W` the worktree file, `M` the target.
  Three independent steps per path, so the cases cannot overlap (R13-8C); the composition applies decision P ("Merge
  anyway, unstage x") and decision 11 ("Carry the file, save staged"):
  1. Carry. The worktree takes the three-way merge of `O`, `W`, `M` by the carry-forward protocol (including a write
     through an open descriptor); when `W` equals `O` it takes `M`'s content. The person's edit is `O` to `W`,
     whatever the index holds.
  2. Index. If `S` differs from `E` (detectably changed since the attempt's pre-entry was read; `E` is a value guard,
     not proof of when anything was staged: a stat-only change can differ, and a later restage can reproduce `E` and
     be replaced, after being saved) the index entry stays `S`, the path is reported as `later-staging`, and step 3
     does not apply to it; otherwise the index entry becomes `M`'s ("unstage x").
  3. Preserve (mandatory whenever an index entry is replaced). If the replaced `S` carries content that no other
     retained version holds, that is its blob differs from `W`'s content, from `O`'s blob and from `M`'s, it is
     written to `saved/<path-id>.staged` with its mode, and the preservation barrier of rule V makes the file, its
     directory entry and the log record that names it D before the index entry is replaced. Not carried, the person's
     version stays at its path and the merged version is written beside it (the earlier decision "Carry edits
     forward", adopted here for these cases; veto open): a mode or type change, a staged deletion, a symlink, a
     non-text file, an unmerged entry (any stage) at the path, a directory in the way. Then one S1 replacement sets
     the marker `materializing` and records the plan (`S`, `W` digests, the pins `N` for the displaced pre-merge file
     and `R` for the installed file, `S` as read), the paths are written by the carry-forward protocol with its own
     persistence ordering, the new index (only the entries of `P` replaced, every other entry carried with its stat
     data) is written into the lock file and renamed over the index (its sync as `core.fsync` makes native Git do);
     the closing replacement clears the marker, records each path's outcome (`installed`, `carried`, `saved-staged`,
     `kept`, `later-staging`) and the identity of the index file just installed (its size and trailing checksum), and
     changes the entries to `attach-watch`. Post-check: read `HEAD`; if authority to do no longer holds, MG6c. Crash:
     L4 re-evaluates every path from the journal and the pins by the rule of M1 (observable file and index contents,
     not inference).
- MG6c cleanup after a late switch (R11-7, R12-9, R13-8D). Pre: an `attach-watch` entry inside its span (below),
  `HEAD` no longer naming the target. It never publishes: the merge is already on its ref. A path is a leftover only
  when all of these hold: the index file's recorded fingerprint (its size and trailing checksum, taken when MG3
  installed it) still matches, the index entry equals `M`'s entry, the file still matches the recorded installed `R`
  (which can include carried edits and need not equal the raw blob of the merge tree), and the entry of the path in
  the tree `HEAD` now names is not `M`'s entry. The fingerprint shrinks the set of states the cleanup can act on and
  is not proof that nothing wrote the index: a writer can write byte-identical contents, so a `git add` that changes
  nothing, or a refresh that reproduces the same bytes, does not defeat it, while a write that changes the fingerprint
  does. A `git reset --soft` followed by a switch that carries the staged content without rewriting the index leaves
  all four facts true while the state is the person's own. All index predicates are revalidated under the acquired
  `index.lock`; the worktree file can still change after its last check and before the exchange, and then the exchange
  can replace a later intentional file at its visible path while preserving the newer one under a saved name
  ("preserved somewhere" is not "left at its path"). These residues are an undo-authority concession and are put to
  the owner (questions 7a to 7c); a by-value predicate is not proof of provenance. Do: the single try of `index.lock`
  (`EEXIST`: `resume = MG6c`); with authority to undo, for each leftover restore from the saved pre-state, not by a
  reverse three-way merge: exchange `R` back with the pinned `N`, and set the index entry back to the entry `S` that
  MG3 read under its lock. A path written since keeps the person's file and is reported with the saved files. The
  closing replacement drops the entries and records `restored`. Triggers: every lane event, a watch on the worktree's
  `HEAD` file while any `attach-watch` entry exists (the service), and one re-check at the end of the span. Span:
  `ATTACH_WATCH_SPAN`, a named constant that must be measured (the longest healthy duration of a `git switch` on the
  owner's hosts with a margin the owner approves). A timer is not evidence that no switch is still in progress, so the
  permission to undo (`attach-watch`) is retained for the whole span and ends at the span's end even if a cleanup was
  deferred by `index.lock`; the reminder data of question 7c (the saved copies and the metadata to restore from) is a
  separate record that persists until the person disposes of it or dismisses it, and keeping it is not permission to
  undo. Conditional liveness: the restoration happens at a lane event or while the service runs, inside the span.
- MG-A, the checkout-first order, is retired: the owner chose "Branch first, files after" (decision 8), and its text
  is in the history record.


Exit audit (revision 15; the thirteenth and fourteenth reviews' lists, with the transition that governs each; a Rust
return-path inventory is a step-1 gate that this document cannot supply). Command exits: validation, empty message,
policy refusal, nothing to commit, or a failure before the request is exposed (C2 step (3)): safe, no request. CAPTURE
expiry and START expiry before exposure: no request, nothing changed. START or setup failure, an internal error, a
signer failure or a hook or filter refusal after exposure: the command's exit guard runs W1 (plain) or the terminal
route forces the absorbing outcome's ledger line first, so no exposed request is left clientless and publishable;
signer failure is B1 (iii) then W1. Plain W1 (deadline, Ctrl-C, inline-owner planned exit, ACQUIRE or REAP exit): the
stop disposition is D before the command returns (decision 14: the final save is never cut short), so the request
cannot publish after the error, and the index and worktree are unchanged because nothing was written before
publication; what remains is a power loss or a kill before the barrier completes, when no error has been reported yet.
The unresolved COMMITTING exit: only the attempt in flight may complete, and its message does not say that staging is
unchanged, because the attempt may publish and its follow-up may run. `published`: after a process crash the receipt
forbids a fresh attempt; after a power loss the tip decides, and the remaining residue is the conjunction of a lost
receipt, a deliberate reset that removed the publication and an erased reflog, in which recovery lands the commit with
a notice under owner decision 3 ("Land it": the publication cannot be told from a deliberate removal); that permission
is decision 3's, not decision 10's; the follow-up is materialized under `WAIT_MATERIALIZE` or explicitly continues
under the service. `conflicted`, `rewritten-away`, `unresolved`: the D-set of rule V; "staging as it was" is said only
for unpublished withdrawal and refusal, and a rewritten-away request is reported with what was materialized. `already`
and a superseded client: terminal, following the successor without renewing the deadline. Query and resend: C3 answers
`withdrawn` only for a plain mark and says for `withdraw-unresolved` that the original attempt may still complete. A
push that returns `queued`: the approved exception. The person's own foreground programs: decision 9. Outside kill,
OOM or power loss: the outside-cause rule, with the stop disposition and the client lock deciding whether a fresh
attempt is authorized. Owner and helper exits: a failed recovery-helper acquisition mutates nothing and the caller
re-evaluates within CAPTURE; a helper that finishes L4 dispatches nothing; SERVICE-ACQUIRE and REAP expiry mutate
nothing; an L6 gate failure admits nothing; an L7 restarter that cannot take the gate or finds a changed incarnation
aborts. Planned service shutdown (revision 15, R14-9): a draining shutdown, never a cancellation. The service stops
admitting new work, records `draining` in the log, lets every running protected execution and every open P2 attempt
finish by its own result, and exits when they are done; it writes `abort` to a prepared attempt only for a request
with no client and only when no closing line is buffered, and it never ends a person's hook. If the manager's own stop
timeout, or a person, kills the service before the drain ends, that is an outside kill (the manager's or the person's
act, not ours), and the owner-death rules apply. A client-held attempt keeps its client as decider throughout the
drain. When a submitted `commit` is followed by the death of the child with no positive evidence (a parent-death
signal, an L2 reap, a kill), the attempt is class U: the client stays in COMMITTING and leaves through the unresolved
form, and plain W1 is not available. An internal error in the owner after a request is exposed is an owner death, not
a normal exit, so recovery and the client lock decide.

Cancellation
- X1 cancel a stalled child. Pre: the child makes no progress for the measured interval, and its exact invocation (the subcommand, its options and the
  configuration that selects merge drivers, conversions, signing and helpers; not the subcommand label alone) is incapable of dispatching
  a protected program (a hook, a filter, a signer or a helper): the classification is by the kind of command, fixed in the retained-child
  table (`fast-import`, `merge-tree`, `commit-tree` without signing, `hash-object`, `mktree` and the other readers and object writers). It is
  never read from the hooks directory, the hooks path or the configuration at spawn time or later, because another program can install a
  hook between the classification and Git's dispatch point. `update-ref`, `commit`, and any command that can run a hook, filter, signer or
  helper by its kind are exempt from X1 whatever is configured now (owner question C-class, default C-kind). Do: send the terminate signal and a
  continue signal, wait the grace. Crash: the child dies with the incarnation. If a child that X1 may cancel ignores the signal, nothing further
  happens automatically except as owner question A decides. The wrapper never removes a ref lock file, for any reason.
  Input from the shipped lock rule: the branch `fix/git-policy-cli-never-removes-a-live-lock` is reworking the shipped code that deleted locks
  (the stopped-child sweep in `stopped_git_locks.rs` and the general judge in `stale_ref_lock_judge.rs`). This design depends on none of it: it
  works with zero removals. It needs that branch to leave no automatic deletion of a ref lock the design did not prove its own, and any later
  removal rule must supply a holder identity qualified by its pid namespace and a removal primitive that cannot unlink, displace or substitute a
  different file that later occupies the pathname before it is called a proof. Rename-to-private is not such a primitive (R9-S): between the
  rename and any restoration another program can create the pathname and enter a transaction, so the public pathname loses its single-holder
  property even when no wrong inode is deleted. The design therefore asks the shipped branch for non-removal of ref locks until a primitive is
  shown (rule N); it does not specify one.

Maintenance (object-store-wide)
- G1 reserve. Pre: a destructive run is to be started by the wrapper (a forwarded `gc`, `prune` or `maintenance` command, or the
  service's automatic maintenance) and a manager is available. Admission is never conditional on a lane: G1 writes the reservation before
  any destructive process exists and does not read `releases/`. Do: create a transient scope `S` named by the run id and place the
  launcher in it; write `maintenance/<n>` for the next free consecutive number `n` (state `reserved`, naming the run,
  `S` and the launcher's identity) by the link method of C3, trying the next number on `EEXIST`; the
  launcher inside `S` is the launch authority. Crash: see G3.
- G2 run. Pre: G1. Do: the launcher, inside `S`, starts the destructive process; entry state `running`. Crash: see G3.
- G3 terminal. Pre: `S` reports no processes and the launcher is dead or has recorded `end` after waiting. Do: record `terminal`.
  A missing end marker never means finished; an empty or absent scope without a dead launcher never means finished, because the live
  launcher inside `S` is still authorized to launch; the launcher's death with `S` empty proves launch can no longer happen. Crash:
  repeated from the registry.
- G4 no manager. Pre: no manager is available. Do: the wrapper treats the run as another tool's maintenance (the owner's outside list),
  prints that it cannot contain it, and retains the recovery copy of every request whose lifetime overlapped it until the person
  disposes of it explicitly, since an escaped descendant cannot be proved finished.

Narrowed guarantees stated by the machine, plainly (revision 12). Acceptance is not unconditional: C0 and C1 end at
CAPTURE with an error and no request when the lane keeps changing or recovering. Within CAPTURE it waits for the
lane's state updates (the marker is set only around the index replacement of M1, the install of MG3 and the recovery
of L4, never across a person's program or a foreign lock) and never for a program; for an ineligible
index, M1 runs Git on a copy under the marker, so there the capture waits on our own lane for a child that X1 can cancel, within MARKER. A quiet owner that holds the gate with
no person's program running (wedged in our own code between L6 and the end of a ref transaction) is not restarted automatically (owner question B, option
B-hand, picked); the command waits to FOREGROUND, then withdraws, naming the unit and how to restart it. An owner wedged inside a person's program is never
ended. L1 can wait for a child between fork and exec, within ACQUIRE and REAP. G4 extends retention at the cost of disk. The wait of a publication
behind a person's `reference-transaction` hook is the person's, and the command's share of it ends at FOREGROUND with a refusal, never with a landing
later. The owner's choice that a command may end with an error at a bound touches "Commits should never fail." and is recorded as the owner's.

### Cost of the durable steps (revision 14)

Owner rules: "commits vs native git should not take 5s longer."; "Disk pressure being high is not an excuse."; "Host
pressure might sometimes be the cause but it's never an excuse."; "sometimes the work itself can be optimized. And we
control every part of the wrapper, including everything it embeds." Owner decision 10: "Force at leaving, else
background (Recommended)". Input (the fourth review of the pure components): with `n` changed paths, bind, spool write
and publication cost `2n + 20` sync calls before acceptance (22 for a one-path commit) and `2n + 27` (29) through
retirement, before native Git's own work; the same review found every ledger append rescanning the whole ledger,
quadratic replay prefix checks, a resend writing a duplicate temporary before finding the identical file, and ancestor
chains synced twice. The counts below are counts of sync calls from the transitions as written, not measurements of
physical flushes; one sync costs what the disk charges, which is why the design counts the syncs a person waits for.

Three promises are kept apart (R12-7): when a command starts to leave (its bound), when it exits (the bound plus
`WAIT_WITHDRAW`, named and to be measured, plus the departure forcing, which has no limit by decision 14), and what is
durable at exit (the D-set of that exit, rule V). Zero fsync calls on one path is a count, not a latency proof; the
latency claim rests on the step-1 measurements named below.

Waited-for syncs of the wrapper's own durable steps, per plain commit with an attached client, a successful
publication, an initialized lane and no hook (the common case): **0**. Measured, in today's code on the 0.5.28 base,
by the latency branch (`doc/planning/cli-git-commit-latency-under-disk-pressure.md`): the person waits for 0 syncs of
the wrapper's own records in a landing that publishes (36 syncs over five signed landings, all made by the background
thread), plus 1 of Git's own pack sync on the unsigned `fast-import` route only; with `core.fsync=added` in the
repository, 1 (the index, as native Git flushes it). The design keeps that number. The person still waits for what
native Git waits for, because object writes, the ref write and the index install follow the person's `core.fsync`
exactly. By transition ("waited": the command is blocked on it; "background": the syncer issues it after the answer,
in a thread or process that never runs in the owner's event loop):
- C1 capture: waited 0 (reads, and one `stat` per in-flight request for the withdrawal marks).
- C2, C3, C4 (one request file, one `link`, no binding, no directory created): waited 0; background 1 for the request
  file and 1 for `inbox/`.
- C5 adopt, B1 build, P1 publish-begin (appends to the order file, the journal and the state log): waited 0;
  background 3, one per chained file, shared per pass. Folding the journal and the order file into the state log would
  save two more; a suggestion, not a requirement.
- P2 swap: waited 0 of ours (the attempt directory, the pipe and the files in it are V, never synced); the ref write
  is Git's.
- P3, T0: waited 0 (the client prints `published` on the landing evidence and does not wait for P3 or the ledger
  line); background 1 for the ledger.
- M1 and the MG3 install: the command waits for this one attempt, within `WAIT_MATERIALIZE` (R13-9) (a script that
  runs `git commit && git status` must see the index, and `git merge && make` the files), so the sync of the index
  follows `core.fsync` as native Git does; the preservation barrier of rule V applies only when a staged-only version
  or an edit has to be preserved (the saved file, its directory entry and its log record: 3 syncs, in the install,
  before the index entry is replaced).
- B1 with a person's hook: the runner's result file is forced before the owner reads it (1 sync, on a path that
  already waits for the hook).
- T1 retire: waited 0 (the command has returned); it needs the terminal ledger line and the removal record D first
  (reclamation barrier), so a power loss before that only delays reclamation.
- Whole lifecycle in the background for an isolated commit: 6 syncs (request file, `inbox/`, order file, journal,
  state log, ledger; the receipt and the client lock are authority for process crashes only and are not forced), and
  `N + 5` for `N` requests sharing a pass. Lane initialization and checkpoint rotation sync once each and are not per
  commit.
- Stop disposition (R13-2, decision 14; the command waits, to completion, but only when it leaves without a published
  commit or with saved work): 2 syncs for a withdrawal (the empty mark file and `lane/decision/`), 1 to 2 for a
  terminal outcome (its ledger line, and its directory entry the first time). It is forced first and alone. Syncing an
  empty file and a directory is not necessarily cheap: directory and journal dependencies can dominate, so the count
  is a count of calls and the latency is a measurement.
- Saved-content set (to completion, after the stop disposition): the request file, `inbox/`, the order file, the
  journal, the state log (5), plus, only for the exact-commit promise (level 2), a `syncfs` of the object store's file
  system, the side ref read back and the connectivity check with the registry comparison, plus the ledger for a
  terminal exit. A request left with the service, or a `handed over` followed by exit, forces the request file,
  `inbox/` and the log files through its `adopt` (5).
- Success: the receipt (an empty file, V) and the single materialization attempt; no sync of ours.
- Other Git work that is not native-equal: the pending and withdrawn side refs and the hold refs are ref transactions
  that Git performs under `core.fsync`; they are written off the waiting path, batched in one `update-ref --stdin` per
  owner wake-up (zero spawns once the ref writer is embedded), and a hold ref exists only for a request that waits
  longer than `HOLD_AFTER`. What this removes from the waiting path that no guarantee needs: a directory synced per
  name (directories are created once and kept); an ancestor chain synced per request; a binding file; one file per
  captured side (a request is one file: `n` paths cost one write and, in the background, one sync, not `2n + 1`); a
  rename-over of the state document; the decision link. What it does not remove, and rule V now keeps: the
  preservation barriers, the reclamation barrier, and the departure barrier of every saving exit. Component changes
  owed from the fourth component review (each a red control first): the ledger append reads the tail once at L4 and
  keeps the offset and last hash (no rescan); the replay prefix check is linear; a resend looks for `inbox/<id>` and
  for an `outcome` line before it writes anything; the request and spool writers take the initialized, durable root as
  given and never sync ancestors again. Costs that remain and need a measurement before a number is promised (the
  disk-pressure branch, `fix/git-policy-cli-commits-fast-under-disk-pressure`, measures why wrapped commits take 8 to
  36 s today; its result decides which matter):
- Large in-flight state: a replacement appends the whole document to the page cache and the syncer flushes the log
  once per pass; the document grows with requests in flight, which rule B bounds for requests with a client; requests
  with no client can accumulate behind a foreign lock, so the gate is the step-1 measurement of append and rotation
  cost at 100 and 1000 in-flight requests.
- Unresolved suppressions and `attach-watch` entries: they last until M3 releases them or their span ends, with no age
  rule for a suppression (a time limit would be a data hazard); their count is a measurement.
- Lookup: C3 on a resend reads the ledger backwards from its end until the identity is found; an old identity costs a
  read of the whole ledger (about a hundred and thirty bytes per outcome); an index is not proposed until a
  measurement shows the cost. The probe of the next registry number and the registry listing are two directory reads
  per retirement.
- Syncer pass time under pressure sets `SYNC_LAG`; if a pass cannot keep up, the barriers still hold the guarantee and
  reclamation is delayed (disk, not a wait).
- Namespace operations of an attempt (directory, pipe, files) can queue behind a journal commit on some file systems
  without a sync; if the measurement shows it, attempt directories move to the per-user runtime directory (a memory
  file system) that both processes of one user share.
- Spawns: `fast-import` and `update-ref` are two per commit, each seconds under pressure in the owner's observation;
  "embed instead of spawning" removes them (the embedded writer is the same journaled, gated code), with differential
  controls against Git first. Test that holds the count: a step-1 test counts the sync calls on the waiting path of a
  plain commit made by the wrapper's durable steps and fails above 0, a second counts the syncer's per pass, a third
  counts a departure barrier's against its D-set, and an instrument on the lane's descriptors confirms that no sync
  call is made on the owner's event loop.

### Revision 15 changelog (for the step-1 models agent; every changed transition, with the model and the schedule)

Naming. New names: the deferred selection of a capture, the fingerprint of the installed index, the permission to undo
(`attach-watch`) kept apart from the reminder record, the materialization executor rule, the draining shutdown, the
live dependency list of a checkpoint, the ledger as the single discharge authority, the level 1 and level 2 promises
of a saved exit, the register entry DEPARTURE. Removed: `WAIT_STOP`, `WAIT_DEPART`, the departure timeout and
`d-pending`, the claim that no fail-closed recovery exists, the planned-shutdown cancellation of protected executions.
Schedules below go red under revision 14 and green under revision 15 as far as the owner's answers and the mechanisms
allow; a model marked "new" is added to step 1; the models branch is not edited by this document.
- Decision 14 and the stop disposition (R14-2; models new "departure that always completes", "kill during the
  barrier", "disk error returned by the sync"). Schedule: W1 creates the mark, the sync is slow, power fails after the
  command returned: red under 14 (the timeout let the command return with the mark not D); green under 15 (the command
  does not return until the stop disposition is D; a power loss before that has reported nothing). A terminal refusal
  whose ledger line is forced as its stop disposition. What remains: a power loss or a kill before the barrier
  completes.
- Recovery after a power loss and R14-1 (model new "success, lost receipt, reset, erased reflog"). The permission is
  attributed to decision 3 and the residue is the conjunction named; the fail-closed alternative (quarantine of
  client-originated requests after a reboot, confirmation by the person) is designed in the history record and is not
  part of the machine.
- P2 nonblocking keeper, `submitted a` after the full write (R14-3; models new "full pipe while the owner writes",
  "failed commit write after bookkeeping"). Schedule: a full pipe blocks the owner's `abort` write through a blocking
  keeper: red; green (every protocol write is nonblocking and retried under the remaining bound).
- Checkpoint live dependency list, ledger as discharge authority, recovery procedure (R14-4; models new "checkpoint D
  while a request file is not D", "saved outcome crosses reclamation", "disposed record in an invalid suffix",
  "checkpoint authoritative with its predecessor deleted"). Schedules as in the review.
- Saved-content set level 1 and level 2 (R14-5; models new "inherited parent objects are V only", "prune view before
  the side ref"). Schedule: the exit names `D` built on a parent whose objects are V only: red (the barrier forced
  lane-written objects only); green (level 2 needs `syncfs`, the side ref, a connectivity check and the registry
  comparison; otherwise level 1).
- C1 deferred selection (R14-6; model new "unresolved base, later staging, late F"). Schedule: `x=1` staged and
  consumed by an unresolved R1, the person stages `y`, R2 captures, R1 fails: red (R2 replayed on `x=0` publishes
  `x=0,y=1`); green (the deferred selection adds `x` back).
- Materialization executor (R14-7; models new "inline owner holds index.lock at the deadline", "executor death or
  failed handoff"). Schedule: the deadline passes while MG3 holds `index.lock`: red (no specified transition both
  returns and continues safely); green (the deadline is a start deadline; a begun mutation is finished by the command,
  or by the service after `resume = MG3`).
- MG6c fingerprint wording, the revalidation under `index.lock`, the worktree race (R14-8; models new "byte-identical
  index rewrite", "worktree changes between the last check and the exchange").
- Draining shutdown (R14-9; models new "shutdown with a running hook", "buffered commit then parent-death before
  evidence"). Schedule: planned shutdown with a running hook: red (the hook was ended and called an outside death);
  green (the service drains; an end of the hook is only the manager's or a person's kill). A buffered `commit` with
  the child killed before evidence is class U.
- Wording (R14-10): unconditional "stop receipt is D" statements are replaced by decision 14's; MATERIALIZE is a wait
  of the command; `S != E` is a value guard; the background count excludes the receipt and the client lock; the cost
  wording for C1 is two sets of `stat`.
- Questions 5, 7a, 7b, 7c corrected as the review lists; 6a and 6b withdrawn.

### Revision 9 and revision 10 changelogs

Moved to `doc/planning/cli-git-commits-never-fail-review-history.md` (the step-1 models for them exist; the changelog
of revision 11 stays here).

### Revision 11 changelog

Moved to `doc/planning/cli-git-commits-never-fail-review-history.md` in revision 12 (the step-1 models for it exist on
`feat/git-policy-cli-commit-protocol-models`).

### Rationale for the lane transitions L1 to L7 (explains; the state machine governs)

Why one lock, one gate. The lane of a worktree is whoever holds the exclusive advisory lock on `lane/owner`. The repository's rule is lock-free
first with an unavoidable lock on the smallest unit and its reason beside it; the reason is that no lock-free claim can tell a new owner that
the previous owner is gone. The proof is stated as the review asks: acquisition proves that no conflicting lock remains on that lock inode;
the previous owner's death follows from the enforced invariants, which are that only the owner process holds the description (children
close it at exec, and a child between fork and exec is waited for by L1), that the lock file is created once and never replaced, that the
owner never unlocks explicitly, and that the lane never executes a person's program in the owner process itself. The gate exists because the
restart of a quiet owner and the admission of a person's program must be serialized by something both sides share; a kernel lock is the
smallest such thing. These two locks were the only locks added in revision 10; revision 12 adds the per-command client
lock `lane/client/<id>` (a kernel lock that exists to tell a killed client from one that left, and is released by the
kernel), which meets "Use as few locks as possible" as the smallest set that gives these
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
it, and the request is already durable. Revision 12 (R11-4): an inline owner never runs a protected program, and every
normal exit of one with an unpublished request is W1 (see "Execution ownership" in the state machine).

Why restart goes through an intent and a pidfd. A restarter that dies holding the gate releases it, so the gate alone fails open during an
incomplete cancellation; the durable `lane/restart-intent` makes it fail closed (an owner that finds the intent at its next admission admits
nothing and exits). A pidfd names the process it opened, so a delayed request cannot reach a replacement owner; a recorded pid is interpreted
in the right namespace only if the pid-namespace identities are equal, so restart is refused across namespaces. "The person's program" means
the whole execution tree of the invocation, so the owner keeps the shared gate until that tree's cgroup is empty; a daemon a hook leaves
behind keeps restart unavailable, which is the person's program being protected.

What restart cannot do. The shared holder case (the owner wedged inside our own code between L6 and the end of a ref transaction, with no
child and no person's program) cannot be restarted automatically, because the restarter cannot take the gate and ending the owner mid-transaction
could leave a lock. That is a remaining internal wait, stated; owner question B.

### Rationale for C1 to C5: acceptance needs no ref write and no sync (explains; the state machine governs)

Acceptance is the existence of one complete, checksummed request file in the worktree's own `inbox/`; no Git ref is
written, no foreign lock can stand in the way, and (rule V) no sync is waited for. The identity is the content hash of
the request plus the lane instance, so one identity names one content and the binding file of earlier revisions is not
needed: a resend of the same identity is a `link` that finds the name taken, and a resend with different content is a
different identity. Lookups by identity need the ledger of the named instance, which lives in the common directory, so
removing a worktree does not remove it. Capture reads the base closure from the immutable objects `B` names; if one is
missing because a rewrite and a prune raced the read, capture starts again, which is legitimate because nothing was
handed over. Capture waits for the lane's short state updates and never for a program, within CAPTURE.
Capture-invalidating generations are written only around lane state updates (C5, P3, M1) and never across P2 or a
person's program. The explicit red control is the reviewer's schedule: a reference-transaction hook waits for ever
during P2, a disjoint request must still be accepted.

### Adoption order, recovery and the logical head

`lane/order` is the authority on adoption. The lane appends `adopt <id> <crc>\n` (fsync) and then moves the record from
`inbox/<id>` to `queue/<id>`.
- After a durable `adopt <id>` the request is adopted even if its record still lies in `inbox/`; recovery completes the move and
  never appends a second `adopt`.
- A record in `inbox/` with no `adopt` line is adopted next, in the order of its capture time and id (a convention, not causal order).
- Each line is self-framing with a checksum; recovery truncates a torn last line to the last valid line before any append.
- The lane is the only writer, so there is no allocator to go obsolete.

Capture consistency: C1 and rule S of the state machine (revision 9 replaces the seqlock). All capture-visible lane state is one document
replaced atomically; the marker, not the parity of a generation, is set only around an effect outside the document (the index replacement of M1, the
recovery of L4); an interrupted writer is completed or rolled back by L4, and C0 routes a client out of a dead marker. Foreign writers of refs and the
index do not take part, so C1 reads the tip and the index once and does not re-read them to validate (C1); a foreign
change that leaves them as read is undetectable and is not claimed.

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

A request held by a person's hook (owner decision 1; transitions B1, B2 and B3 of the state machine, not the owner-question options B-hand and B-restart). A held request has no commit and is not part of the
logical tail. Its consumption entries are keyed by request and version and end by one rule set (revision 9): installed, already at the target or preserved by M1, or dropped by M3 when the head agrees or a later request
consumes the path; voiding turns them into suppressions and never releases them (M2); a refusal drops them; a superseded version re-keys them to the
active version (B3); never at publication (a published request whose follow-up is deferred keeps them, so a plain commit
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
nor evidence of a switch. The residue is the accepted unwanted staging of the old question 4 (picked A): only the
index is changed, content is
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
(`<id> <outcome> <commit> <time>`; V, then D by the syncer, rule V); (2) release the retention of a published request
(not of a saved one); (3) nothing else.
The ledger is append-only and has no horizon and no rotation. An entry is about a hundred and thirty bytes (a 32-digit identity,
an outcome word, a 64-digit object name, a time), so a worktree that commits a hundred times a day adds under five megabytes a
year; there is no size bound, so there is no limit to justify. A resend of an identity is answered from the ledger and
never published; C3 (revision 12) is the lookup that spans the terminal ledger line, the withdrawal mark, the order
file, the queue and the inbox, and an identity (a content hash plus the instance) never names a second content; since
the ledger is never forgotten, a resend after any delay is recognized. The sequence "lost answer, long stop, later
reset, resend" therefore cannot publish twice.
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
commit with `pre-commit` or `commit-msg` configured) is exempt from cancellation entirely. Revision 9 (R8-5) classifies by the kind of command, fixed in a table, and not at spawn from the hooks
directory, the hooks path or the `hook.*` configuration: any configuration read at spawn can change before Git reaches its dispatch point, so a
spawn-time classification races a program installed in between (owner question C-class lists the alternatives).
The restart of a quiet owner (L7) is unavailable while a ref-transaction child is in flight (the gate), so ending the owner can never leave a lock
that cancellation would not have left. After an outside kill (the OOM killer, a person) a lock may remain; it is treated as foreign, the request
becomes pending, and the notice names the file and says that if no Git process runs it can be removed by hand; that case is on the owner's
outside list.
X1 stays as written, and no finding forces otherwise (revision 9, model finding on F4). The step-1 model candidate that removed a stale lock through a
journaled `O_TMPFILE` plus `linkat` would make the wrapper delete a lock it did not create: a ref lock is created by Git, so no inode of it is
one the lane created and journaled, and the identity rule (delete only an inode the lane created and journaled) cannot hold for it. The same
technique is sound for `index.lock` that the lane itself creates by `linkat` (M1), where the inode is the lane's own, and that is the only place
it is used. A finding that would force the wrapper to remove a ref lock does not exist: R8-5 concerns classification, S-1 and S-2 concern the
shipped sweeper, and the accepted narrowing A-kill is the stated cost. The L7 restart ends the owner with an uncatchable signal through the
pidfd (SIGKILL, step 1 finding F3): a wedged owner can ignore the terminate signal, and L7 holds the gate exclusively, so no ref-transaction
child is in flight when it is ended.
Tests (red controls first): the retained-description deadlock in the fork-to-exec window; a closed-description descendant still writing when the
lock frees; the service cgroup reused by a replacement owner (the replacement must not kill itself); restart versus hook dispatch; restarter
death at every point of L7; a wedged shared gate holder; a pid read in another namespace.

### Retention lifecycle and the spool closure

The spool is the source of truth for the request until its ledger line is D and the request is retired (and
permanently for a saved request). It is a
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
  request's spool only if every entry overlapping the request's lifetime is `terminal` by G3 (T1; an entry with an empty scope and a live launcher is not
  terminal), after a release intent, and a connectivity check passes. Git's own
  `gc.pid` in the common directory is read as a second source and also holds release. Where no manager exists (G4) containment is not possible
  (a descendant can leave a process group), so the wrapper treats the run as another tool's maintenance, prints that it cannot contain it, and
  keeps the recovery copy of every overlapping request until the person disposes of it explicitly: disk is spent, nothing is claimed.
- Maintenance another tool runs is outside the wrapper's control: a loss after recovery data is released is a corrupt repository
  caused by another tool, which is on the owner's outside list. The wrapper reports the missing object when it next reads it.

Answer to R8-7 (revision 9). One predicate: T1 uses the terminal predicate of G3, not a scope test, so a reservation whose scope is empty
but whose launcher is alive blocks. The boundary between T1's observation and a new admission is covered by an explicit argument, and the
release is bracketed by a durable intent so L4 can finish or withdraw it. The schedule that fails under revision 8 and passes under
revision 9: T1 observes no open entry and checks connectivity; G1 publishes a reservation; G2 starts `prune`; T1 releases the copy.
Under revision 9 the released closure is reachable from the tip (the check of step (3)) so `prune` cannot remove it; a copy that is not
reachable from the tip (saved, pending, rewritten-away) is never released by T1, so the late run cannot take it either.

Revision 10 note (R9-3): the identity-based unlink described here is withdrawn; M1 creates `index.lock` with `O_EXCL`, journals the intent, and never
removes a stranded lock (rule N, choice R).
Answer to the M1 rows of R8-8 (revision 9). The earlier crash text promised that exit handling removes the lock; a process killed
by SIGKILL or power loss runs no exit handling, so that promise is withdrawn. The lock is now created by `linkat` of a complete unnamed
file whose inode was journaled first, so a stranded `index.lock` is either provably ours (same inode, unlinked by L4) or foreign (never
touched). The case that the rename happened and the consumption release was interrupted is recognized by the index file being the
journaled inode, and L4 completes it. Every path ends in exactly one named outcome (`installed`, `already`, `preserved`, `declined`), each
tied to what happens to its consumption entry.

### Installing into the current index (rationale for M1 and M2; the state machine governs)

Only a delta applied to the index read under the lock; never a whole-file install built earlier. There is one path, M1, for every follow-up:
the lane tries `index.lock` once after the publication, never before it and never across the swap (a lock taken before the swap could be held
across a hook that waits for ever), applies the attachment guard, applies the entry rule, and renames the new index over the old. The earlier
two-path design (an immediate critical section around the swap, and a deferred follow-up) is withdrawn: the immediate path skipped the
attachment guard, and its only benefit, no index writer between the publication and its materialization, is exactly the by-value concession
the owner accepted in decision 2. Publication never waits for `index.lock` and never fails for it. The marker is set only while the
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

The attachment guard is the one in "Adoption order": a guard against the usual cases, not a fence, with the residue
accepted as the old question 4 (picked A).

Extensions, per policy (not preserved wholesale): the cache-tree is dropped; the resolve-undo data and the untracked cache are
carried unchanged only after a differential control shows Git behaves the same afterwards (otherwise they are dropped or the index
is ineligible); the offset tables are not written; fsmonitor, split index, sparse index and unknown extensions make the index
ineligible, and the delta runs through Git on a copy made under the lock (`reset --no-refresh`). Index versions the writer does not
produce make it ineligible.

Voiding is not releasing, and a superseded version ends (R8-6, revision 9). The schedule: the physical head and the index both hold `x=A`;
`R1` captures worktree `x=B` and consumes the pre-entry `A`; `R1` publishes `B`; a foreign writer advances the branch to a successor whose
`x` is `C`; the worktree stays `B` and the index stays `A`. M1 declines (the worktree is not `C`), M2 voids. The person then stages only `y`
and commits plainly. Under revision 8 M2 released `R1`'s entry, so C1 compared the index `x=A` against the physical head `x=C` and captured
`x=A`, reverting `x`. Under revision 9 the entry survives as `suppress` with `E = A`, C1 sees the index `x=A` equal to `E` and does not
capture it, and the commit holds `y` only. The entry ends by M3 only when the head returns to `A` (rule a), a later request consumes `x`
(rule b), or the person stages `x` to anything but `A`, in which case C1 captures that staging and the entry no longer matches anything.
Two bookkeeping kinds are kept apart on purpose: voiding ends installation authority, which would otherwise let a lane overwrite an index
it cannot reconcile, and keeps the suppression of a known historical entry, which is what prevents the stale return.
The held case: B3 gives an obsolete version a durable terminal outcome (`superseded` in the lane's ledger), moves the consumption entries to
the active version without releasing them, and ties the release of its spool to the end of its discarded hook execution.

The wrapper's own index commands do not wait for the lane's intents behind a foreign lock.

### Merge order and restoration (revision 12; the state machine governs)

The narrative of the section "Merges, by the same lens" stays the explanation of the carry-forward protocol itself.
Two orders exist for a merge. Check out first and swap second (M-A, revisions 9 to 11) keeps capture clean (`M` enters
the logical tail in the same replacement as the checkout) and costs a restoration transition, which the owner's bound
answers make unsafe: a withdrawal after the files were written must restore them through an `index.lock` that a
foreign program may hold, so "staging as it was" can fail at the error. Swap first and install after (revision 12) has
no restoration before publication, so a withdrawal changes nothing; its cost is a window in which the branch holds `M`
while the index and worktree hold the old tip, closed for the wrapper's own captures by the `merge-install`
consumption entries and open to a native `commit -a` run outside the wrapper. The coordinator's ranking of M-A over
the swap-first order (revision 9) rested on that window; revision 12 put the choice to the owner, who chose swap-first
(decision 8, "Branch first, files after"); the checkout-first text is in the history record.

### The side ref for a held branch lock, or a branch a foreign program keeps moving

Revision 12 reading: for a command with a client, "the command exits 0 and says the branch will show it" below is
replaced by W1 at the FOREGROUND bound (the commit stays on the side ref and nothing lands by itself); the exit-0
answer applies to a request that has no client.

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

## What is guaranteed to complete, and what may wait (revision 15)

No wait below is called designed out, and every wait of a command ends at a bound of the register in rule B.

Completes without waiting on another program, with a bound where the lane's own state keeps changing:
- acceptance: local reads and unsynced writes (rule V) in the worktree's own directory, no ref write, no wait on a
  program or on a foreign lock; it waits for the lane's short state updates and for a recovery, within CAPTURE, and
  ends there with an error and no request when the lane kept changing or recovering (nothing changed; run the command
  again); failures are the owner's outside causes (a full disk, I/O errors);
- the answer after a crash or a long stop (a resend returns the request's state or the ledger's outcome);
- retention of everything the request needs until its ledger line is D (reclamation barrier), and permanently for
  anything promised as saved.

Waits of a command, each ending at its bound with an error (W1) except where the register says otherwise:
- the next owner's acquisition, while a child of the previous owner is between fork and exec (ACQUIRE, REAP);
- a quiet owner that holds the gate with no person's program running (owner question B; the error names the unit);
- the command's wait for its own request on its own lane, a held ref lock, a held `index.lock` before publication, a
  person's hook run by the service, a held request (FOREGROUND);
- a push (PUSH): the push is ended at its bound and queued, and the command exits 0 saying so;
- the `commit` instruction already submitted by the command itself (COMMITTING): the one corner that can pass the
  bound, by `WAIT_COMMITTING` (owner decision 7).

Not waits of a command, and not a commit landing later: the index and worktree catching up after a commit or merge
that is already published and that the command reported (M1, MG3, "derived index materialization"); a request that has
no client (a service-originated reconcile merge, a request whose client was killed from outside) landing when the
branch allows it. The remaining waits are the person's own programs in their own foreground (an editor, a signer's
prompt, a hook the command runs before any request exists, a forwarded `git gc`; owner decision 9).

Stuck Git children and the person's hooks (decision 1; X1 of the state machine).
A Git child that makes no progress (no processor time, disk activity or output) for the measured interval and whose command cannot launch a
protected program is sent the terminate signal and its step redone: the object writers and `merge-tree` are content-addressed, and
`update-ref` is a compare-and-swap that is repeated only after a positively established failure (E1 class F) and is
exempt from cancellation by its command kind. The person's hook is never
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
again only if the recovery classes say to publish (R5 unclear, which owner decision 3 lands; R4 is withdrawn, and
which includes every attempt found after an owner's death without positive evidence). Git's contract allows the
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

Revision 12 reading: "pending ... and the command answers success" below applies to a request that has no client. A
command with a client waits to FOREGROUND and then ends in W1; the request stays pending only in the sense that its
commit is kept on a side ref and nothing lands by itself.

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

Revision 12 reading: what follows is derived materialization after a published commit (M1, MG3). It is not a commit
landing later and not a wait, so rule B has no entry for it; the command has already reported `published`.

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

Revision 12 reading: the bullets below that say the command exits 0 and the checkout finishes later describe the
earlier owner answers. The state machine's MG1 to MG3 govern: a merge publishes first, so a foreign `index.lock`
delays only the install of an already published merge (derived data), and a held ref lock or a held hook before
publication is a FOREGROUND wait that ends in W1 with nothing changed.

The merge-fixes branch waits for any `index.lock` holder before the merge and computes the merge again on a moved head up to
twenty rounds (`merge_isolation_moved.rs`).
Revision 13 reading: the first review withdrew deferring the checkout past the publication, and the owner reversed
that for the bound rule (decision 8, "Branch first, files after"): the merge commit is computed, the ref is swapped,
and the worktree and index are installed after under the lock (MG1 to MG3).

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
     semantics, the repository's conflict style, labels naming the person's edit and the merge); create the temporary file in `lane/tmp/`
     (rule N; beside the path under a unique lane-owned name where the file system differs), write `R`, fsync it, and journal `R = (device, inode)` of that
     temporary file (fsync) while a link the lane owns pins it;
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
  Recovery after a crash follows rule N (revision 10): **the lane removes a name only if no foreign program creates that name** (the unique names under
  `lane/tmp/`, `saved/` and the lane's other private directories), and an inode identity is a claim only while a lane-owned link pins the inode; a
  journaled `(device, inode)` pair alone is never deletion authority, and the lane never removes `index.lock` or a ref lock. It never deletes on the
  strength of a missing journal step.
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

## Answers to the first eight reviews

Moved to `doc/planning/cli-git-commits-never-fail-review-history.md` (history only; the normative state machine governs,
and a mechanism that file names as withdrawn stays withdrawn).

## Answers to the ninth and tenth reviews

Moved to `doc/planning/cli-git-commits-never-fail-review-history.md` with the revision 9 and 10 changelogs; the
eleventh review's answers and the revision 11 and 12 changelogs are there too.

## Answers to the fourteenth review

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-14.txt`, with a closure line: closed (a transition
answers the schedule; a model is owed), or owner question. The verdict was: continued model work only. This revision
builds nothing. The review did not inspect the cited Git source lines.

- R14-1 (P2), decision 10 does not approve a fresh attempt after a successful return: accepted. The permission is
  attributed to decision 3 ("Land it": a publication that cannot be told from a deliberate removal), not to decision
  10, and the residue is the conjunction of a lost receipt, a deliberate reset and an erased reflog. The conservative
  alternative is designed in the history record. Closed as attribution.
- R14-2 (P1), a tiny timed stop flush still resurrects: accepted, and settled by the owner's decision 14: there is no
  timed flush, so the original schedule no longer exists; the claim that fail-closed recovery is not implementable is
  withdrawn as too broad; terminal exits force their ledger line as their stop disposition. What remains: a power loss
  or a kill before the barrier completes, when no error has been reported. Fail-closed recovery (quarantine after a
  reboot, at the cost that interrupted client work waits for confirmation) is designed in the history record and is
  not needed.
- R14-3 (P2), the keeper can block the owner's write: accepted. Every protocol write is nonblocking, retries run under
  the remaining bound, `submitted a` is recorded after the full write. Closed.
- R14-4 (P1), a checkpoint cannot discharge dependencies on content it lacks: accepted. A checkpoint carries a live
  dependency list that must be D before the old root is removed; the ledger is the single discharge authority and a
  saved outcome's inputs stay live until a `disposed` line; recovery is a two-pass procedure. Closed.
- R14-5 (P1), the exact saved commit lacks a durable, maintenance-safe closure: accepted. Level 1 (captured intent) is
  always promised; level 2 (the exact commit) only after `syncfs`, the side ref, a connectivity check and the registry
  comparison. Closed.
- R14-6 (P1), late F after an unresolved capture loses selection: accepted. C1 records the deferred selection and W2
  adds it back when the unresolved request resolves without publishing. Closed.
- R14-7 (P1), `WAIT_MATERIALIZE` needs an executor: accepted. It is a start deadline; a begun mutation is finished, by
  the command or by the service after `resume = MG3`. Closed.
- R14-8 (P2), the MG6c fingerprint is not proof of no write: accepted. The wording, the revalidation under
  `index.lock` and the worktree race are stated; the concession is in questions 7a to 7c.
- R14-9 (P1), planned shutdown invents a hook-cancellation exception: accepted. Shutdown is a drain; ending a hook is
  only an outside kill. A buffered `commit` followed by a child death with no evidence is class U. Closed.
- R14-10 (P3), qualifications: accepted and edited.
- Owner questions: 5, 7a, 7b, 7c corrected as the review lists and also written to a separate file; 6a and 6b
  withdrawn by decision 14.
- Not a defect: none of the findings was rejected.

## Exits that stay nonzero

- Policy errors at error severity, an empty message, and a content conflict no resolver settles (the merge stops with the
  conflict recorded, and a replay conflict keeps the captured inputs).
- `withdrawn` at a bound (owner, 2026-11-03T12:45, "Return with an error"): any wait of a command ends with an error naming the blocker, the commit is kept on a side
  ref for a retry and nothing lands by itself; the register in the state machine lists each wait and its bound.
- A failing hook or a failing required clean filter (owner): the commit ends with its own message and changes nothing.
- `sign-failed` (the configured signer failed, an outside cause; B1 (iii)): a command with a client waits for the
  signer to recover within FOREGROUND and then ends `withdrawn` by W1 naming the signer, nonzero, with the work on
  disk; a request with no client stays `pending` and the next trigger tries the signer again.
- The causes the owner listed as outside our control, with a clear message, and work already handed over finished by the next
  run.
- No longer here: a commit that names a conflicted path, a plain commit with unmerged files and `commit -a` over unmerged files
  (committed with a warning); a merge that would overwrite unsaved edits (the edits are carried forward). Two
  different things are told apart (R11-11): a commit landing later, and derived index materialization after an already
  published commit. A live foreign ref lock, or a foreign `index.lock` held before the publication of a merge, makes
  the command wait to its FOREGROUND bound and then end `withdrawn` with an error, the commit kept on a side ref;
  before the owner's bound answers it landed or was saved with exit 0. A foreign `index.lock` held after the
  publication does not fail or delay anything: the commit is on the branch, the command says the index will catch up
  when the lock is free, and M1 or MG3 finishes it.

## Questions for the owner (picks in force)

The full text of this section, as it stood in revision 13, is in
`doc/planning/cli-git-commits-never-fail-review-history.md` under "Questions for the owner, full text". What stands,
with each pick open to the owner's veto where it was delegated ("Pick options as if I had picked them and invite veto
when I say I'm back."; clarified: "\"As if I had chosen\" means what I would have picked most likely if I'm always
here."):
- Answered by the owner on 2026-11-03: stuck step, "End Git, hold hooks (Recommended)"; deferred staging, "Update the
  index (Recommended)"; unclear publication, "Land it (Recommended)"; a branch a foreign program keeps moving, "Save
  on the side ref (Recommended)"; a `--mirror` push, "Leave out wrapper refs (Recommended)"; an amend while commits
  are pending, "Amend the pending commit (Recommended)".
- Adopted from standing rules, not asked: the embedded path for a deterministic Git hang, background re-signing, saved
  copies kept until disposal, corruption by another tool after release on the outside list, whole-lifetime maintenance
  coordination.
- Question 1 (staging a path back to its consumed entry while commits are pending): picked A, the plain commit does
  not capture an entry equal to the consumed one; a notice says so; versioned restaging events are an optional later
  improvement with the sixth review's cases as requirements.
- Question 2 (a quiet owner of our own making): the pick of automatic restart was replaced by B-hand: nothing restarts
  automatically, the error names the unit and the restart command.
- Question 3 (a hook with an external effect on "prepared" after an owner death): picked A, `update-ref` runs again;
  Git's contract allows the repeat.
- Question 4 (the instant inside `checkout` where the index is written before `HEAD`): picked A, the residue is
  accepted; `git restore --staged` undoes it.
- A (a Git child that ignores the terminate signal and holds a ref lock): picked A-kill, an explicitly accepted
  narrowing: a lock may be left that only the person clears.
- C-class (which commands the stall rule may cancel): picked C-kind; `update-ref` and every command that can dispatch
  a protected program are exempt by kind.
- H (hook runs when a held request is replayed): picked H-version, once per version, with the stated crash exception;
  H2: picked H-episode, one new invocation per recovery episode that lacks a durable result.
- M (merge order): now owner decision 8, "Branch first, files after".
- R (a stranded `index.lock` of the lane's own dead incarnation): picked R-never; the lane never removes it and
  reports it.
- P (a merge touching a path the person has staged): owner decision, "Merge anyway, unstage x" (P-carry); composition
  in MG3 and decision 11.
- W (the attachment concession for merge file writes): owner decision, "Allow the rare leftover" (W-detect); its
  cleanup is MG6c, with questions 7a to 7c open.
- G (retention against late maintenance): picked G-order; the proof is the two registry listings and the probe of T1.
- N (retry of a live owner after a proven store failure): picked N-silent, a recorded concession that covers a proven
  failure (class F) only.
- B (a quiet owner that holds the gate with no person's program running): picked B-hand.

## Owner questions for revision 12 (answered)

The owner answered all four through the question tool; the answers are decisions 7 to 10 in "Owner decisions of
2026-11-03" and the full question text with its options is in
`doc/planning/cli-git-commits-never-fail-review-history.md`. Revision 13 states what each decision covers exactly,
because the twelfth review found the questions narrower than the mechanisms: decision 7 covers a submitted `commit`
instruction (not only a disk write) and never a fresh attempt; decision 8 covers a lag between the branch and the
files and staging, and the wrapper's own `commit -a` and edited-path behavior (MG2); decision 9 covers the editor, the
hooks the command runs before a request exists, a signer prompt, filters and a forwarded `git gc` as the owner saw
them listed, and the register lists forwarded maintenance as its own line so a later decision can bound it without
touching the rest; decision 10 covers every exit that promises saved work and a background force before reclamation,
and does not equate durable acceptance with revision 11's 22 syncs (the one-file request is the optimized form).

Adopted without a question (the coordinator may veto; each follows from a rule the owner already gave):
- Protected programs (hooks run by the lane, signers, helpers) run only under the service, never in a foreground
  client; where no service can start, the command ends at START with an error and the work stays saved ("The person's
  hook is never ended" and the bound answers).
- A merge that cannot carry a staged or edited path as text (a mode or type change, a staged deletion, a symlink, an
  unmerged entry) leaves the person's version in place and writes the merged version beside it ("Carry edits forward",
  "never discard user data").
- A staged-only version that a merge replaces in the index is saved to `saved/<path-id>.staged`, forced to disk,
  before the entry is replaced ("never discard user data").
- A hold ref is created only for a request that waits longer than `HOLD_AFTER` ("Invoke real git as rarely as
  possible"; the hold ref was never load-bearing).
- An invocation nonce is part of every request ("one operation, one identity"), so a repeated identical operation is a
  new request and a resend is the same one.

## Owner questions for revision 15

The owner decided "Assume the final disk save would never be cut short." (decision 14), so questions 6a and 6b are not
asked. The questions below are the only open ones; the same text is in a separate file in the review directory, named
`commits-never-fail-questions-` followed by the abbreviated hash of the commit that holds this text, so that they can
be checked on their own. Each question states its risk plainly; the recommended option is marked and the ranking gives
a reason for each adjacent pair. None is decided here.

### Question 5: when what you staged is not what is in the file

Plain context. You chose "Merge anyway, unstage x" and then "Carry the file, save staged". This asks again, with an
option added and the claims corrected. Example: `x.txt` is `A` at the branch tip; you stage a change so the index
holds `B`; you then edit the file to `D`; the merge makes `x.txt` `C`. Two limits apply to every option. The wrapper
reads your index entry once, when the merge asks to move the branch (the attempt's pre-entry), and later compares by
value: a path whose entry has detectably changed since then is left alone as later staging, but a change that only
touches stat data can look like staging, and a deliberate restage that reproduces the earlier entry looks like nothing
changed and is replaced. Whatever is replaced is saved aside first, so nothing is lost; which version is visible in
the file is what you are choosing.
- Option A, carry the file you see; save the staged-only version aside (Recommended; your earlier answer). Visible
  here: `x.txt` becomes the merge of your change `A` to `D` onto `C` (with the usual markers where they collide), the
  index holds `C`, and a line says your staged `B` is saved in `saved/x.txt.staged`. Risk: a staged change you meant
  to keep is not in the file and must be taken from the saved file by hand. Pros: the file stays readable; nothing is
  lost.
- Option B, carry the staged version onto the result and save your file aside. Visible here: `x.txt` becomes the merge
  of `B` onto `C` (markers where they collide), the index holds `C`, and your later file `D` is saved in
  `saved/x.txt.worktree`. Risk: the version you worked on last is the one that is not in the file. Pros: the most
  literal reading of "your staged x becomes an unstaged edit".
- Option C, show all three versions in the file (not yet implementable as described: it needs a defined text format
  for your staged `B`, your file `D` and the merge `C`, and it applies only to text files; modes, deletions, non-text
  files, symlinks and unmerged entries would still keep your version at its path with the merged one beside it, and
  the protection of concurrent writes would still need a saved backup). Visible here, if built: `x.txt` holds the
  three-version conflict, the index holds `C`. Risk: long files make it hard to read, and you are asked to resolve
  what you did not ask to resolve. Pros: nothing is archived or hidden for ordinary text.
- Option D, stop before the branch moves. Visible here: when the merge is computed, the wrapper scans the index once;
  if a path you changed has a staged version that differs from the file, the command exits non-zero naming the path
  and changes nothing. Risk: it can only refuse on what its scan saw; staging that appears between the scan and the
  move of the branch is not caught and is handled as later staging, and it is not "merge anyway". Pros: nothing is
  decided for you.
- Ranking: A > B > C > D. A beats B because it keeps the version you last worked on visible; B beats C because B
  leaves one readable file where C would leave a three-way conflict and is buildable now; C beats D because C
  completes the merge as you asked and D returns the work to you.

### Question 7a: may the late-switch cleanup act on a test of values?

Plain context. After a merge writes files, a `git switch` that was paused can finish and leave the merge's files in
your new branch. The wrapper can restore the old files, but it cannot prove the state is a leftover and not something
you did on purpose: it can only test values. It records a fingerprint of the index file it wrote (size and trailing
checksum) and checks that the fingerprint still matches, that the index entry still equals the merge's, that the file
still matches what it wrote (which can include edits it carried, so not necessarily the raw merge blob), and that the
new branch's version of the path differs. A fingerprint is not proof that nothing wrote the index: a write that
changes the fingerprint defeats the check, and a write that produces byte-identical contents does not. Two more
limits: the checks are repeated while the wrapper holds `index.lock`, but your file can change after the last check
and before the swap, in which case the newer version is saved aside under a name and the visible path gets the old
file back. Example: you merge; then you run `git reset --soft` to an earlier commit and switch to a branch whose
version of the file is the same as that commit's, keeping the merge's content staged on purpose without Git rewriting
the index. Every value test passes, and the cleanup would restore the old file and entry, undoing your deliberate
staged change (the merge's content stays in the merge commit). No measurement of how often such sequences happen
exists.
- Option A, yes: clean up on the value tests, inside the span of question 7b (Recommended). Visible here: in the
  example above your staged change is undone and a line says what was restored from which saved copy; in an ordinary
  late switch the leftover is cleaned; if a command wrote the index in a way that changed the fingerprint between the
  merge and the switch, the wrapper does not clean up and the line names the leftover instead. Risk: a deliberate
  sequence like the example, or a byte-identical rewrite, can be mistaken for a leftover inside the span; and the race
  above. Pros: the leftover you accepted is cleaned without a step from you.
- Option B, no automatic undo; the wrapper tells you and offers an explicit restore command. Visible here: nothing is
  undone by itself; the next wrapper command in that worktree prints the possible leftover and the exact restore
  command. Risk: the leftover stays until you act; and the restore command, when you run it, is also a value-based
  restore and says what it will replace and where your current versions are saved. Pros: the wrapper never undoes
  anything on its own.
- Option C, no cleanup and no restore command; the leftover is reported once at the merge. Visible here: one line
  naming the files and how to undo them by hand. Risk: the leftover is always yours to fix. Pros: simplest.
- Ranking: A > B > C. A beats B because it restores the leftover you accepted without a step from you, at the stated
  risks; B beats C because B keeps the restore one command away.

### Question 7b: how long may that automatic cleanup last?

Plain context. Only matters if you answered 7a with A. A timer is a limit on the permission to undo, not evidence that
a paused switch has finished. Example: a late switch finishes 5 s after the merge, or 5 minutes.
- Option A, a measured span after the merge, then the permission ends (Recommended). Visible here: a switch that
  finishes inside the span is cleaned only if the value tests still hold, a service or a wrapper command checks before
  the span ends, and `index.lock` can be taken; the permission ends at the end of the span even if a cleanup was
  waiting for `index.lock`. After it the files stay (question 7c). Risk: a slower switch leaves the files; a check
  that never happens inside the span cleans nothing. Pros: it bounds the chance of mistaking your own later action for
  a leftover.
- Option B, until the first wrapper command in that worktree after the merge. Visible here: that command checks once;
  if the branch is no longer the merge's and the value tests hold it cleans, and in every case the permission ends
  there, including when the branch is still the merge's. Risk: a switch that finishes after that command is not
  cleaned, and if you run no command for a long time the window stays open as long as you are away. Pros: no clock.
- Option C, until the files change. Visible here: cleaned whenever the leftover is next seen, however late. Risk: days
  later it can mistake your own state for a leftover, the risk of 7a made unbounded. Pros: no span to outlast.
- Ranking: A > B > C. A beats B because its window is bounded in time and not by when you happen to run a command; B
  beats C because C has no end at all.

### Question 7c: after the permission ends, what is kept and what is said?

Plain context. Both options below mean no automatic restoration after the permission ends; they differ in whether you
are told. A paused switch has no finite completion time, so a late switch can still leave the merge's files after the
permission ends. Example: the switch finishes an hour after the merge.
- Option A, one line at the merge naming how to undo by hand, and nothing afterwards. Visible here: nothing happens at
  the hour mark. Risk: you may never notice the files. The wrapper keeps the saved copies until you dispose of them,
  as it does for every saved copy. Pros: no standing record.
- Option B, keep a reminder (Recommended). Visible here: the wrapper keeps a small record of the merge's saved copies
  and metadata until you dismiss or dispose of it; a later wrapper command in that worktree prints a line only when
  its conditions match (the branch no longer names the merge's branch and a file matches what the wrapper wrote), and
  the line says "possible leftover", not "leftover", because the ambiguity of 7a remains. The restore command it names
  says what it will replace and where your current versions will be saved; the reminder is not renewed permission to
  undo automatically. Risk: no line appears if no command matches the conditions, and a standing record exists. Pros:
  you are told without anything being undone.
- Ranking: B > A. B beats A because it tells you about a possible leftover without undoing anything, at the price of a
  record to keep.

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
   - F7: two clients that send the same request bytes, and a client that builds the request again from the worktree
     after a lost answer (a new request, never a rejection: the resend rule of C2 sends the retained bytes), one
     before adoption and one after;
   - F8: a destructive prune launched from one worktree while another worktree accepts, imports, publishes and releases;
   - F1 as a control too: the intact anchor with the post-anchor entries deleted, and the identical commit written by another writer.
2. The hook-bypass ordering model (F9): held `R1` with independent `R2`, overlapping `R2`, `R1` refusing after `R2` landed, `R1` succeeding
   after `R2` landed.
3. The immutable request, spool and permanent-ledger components with no reclamation: serialization, validation, integrity, fsync ordering,
   torn tails, resend with changed content, upgrade recovery; retain everything.
4. Pure tree comparison and captured-delta replay with explicit base, result and current inputs, changing no live ref, index or worktree;
   including the mixed plain and named-path pending sequence and an amendment whose target is no longer the tip.
Models added from the seventh, eighth and ninth reviews: moved to
`doc/planning/cli-git-commits-never-fail-review-history.md` (the models for them exist on
`feat/git-policy-cli-commit-protocol-models`).

The models must not assume: an observable `reflog_intact` boolean with no implementable evidence; permanent uniqueness of a device/inode pair; an atomic
identity-check-and-pathname-removal (none exists on Linux); that a quiet owner must be unhealthy; that all protected-program capabilities are already
classified correctly (the eligibility table is verified per exact invocation, including merge drivers, conversions, signing and helpers).

Models added from the tenth, eleventh and twelfth reviews: moved to
`doc/planning/cli-git-commits-never-fail-review-history.md` (the thirteenth review's list below restates what is still
open).

Models added from the fourteenth review (revision 15; each starts as a failing control with the revision 15
changelog's schedule; observations, never the completed logical operation). The review's twenty controls, keyed to the
transition that answers each (controls 1 and 2 test the departure barrier of decision 14 and become
power-loss-before-completion and kill-during-barrier schedules):
1. R14-2: the departure barrier is cut short by power loss before it completes; no error was reported; request and
   adoption survive: the stop disposition and its residue.
2. R14-2: two identical durable states, one from an outside-killed client and one from a client that left with a lost
   mark; check what recovery may do in each (the owner chose continuation: it continues).
3. R14-2: a terminal refusal or conflict whose ledger line is the stop disposition; power loss before it is D;
   conditions later permit publication.
4. R14-3: a full FIFO while the owner writes through the keeper; the event loop does not block.
5. R14-3: a failed `commit` write after bookkeeping; submission follows the successful full write only.
6. R14-4: a checkpoint is D while a live request's file or name is not D; the old root is removed.
7. R14-4: a terminal saved request crosses a checkpoint and reclamation; its inputs stay live until disposal.
8. R14-4: a later `disposed` record in an invalid suffix with a missing earlier saved item; no circular discharge.
9. R14-4: a valid new checkpoint with the sealed predecessor deleted; why it stays authoritative.
10. R14-5: inherited parent objects are V only; the lane-written child objects and the side ref survive.
11. R14-5: a prune takes its view before the saved side ref exists and deletes after exact-commit confirmation.
12. R14-6: an unresolved exit, later staging, a descendant capture, then late F; check selection as well as ancestry.
13. R14-7: a materialization timeout while an inline owner holds `index.lock` with no service available.
14. R14-7: the foreground deadline during preservation or carry, with executor death or a failed handoff.
15. R14-8: a byte-identical index rewrite after MG3; the fingerprint is not proof of no write.
16. R14-8: the index changes between the first predicate check and the lock; the worktree changes after the last check
    and before the exchange.
17. Question 5: staging after P1 and before publication; staging after publication that reproduces `E`; stat-only
    changes.
18. R14-9: planned shutdown with a running protected hook; no cancellation is smuggled in as an outside death.
19. R14-9: a buffered `commit`, shutdown, a parent-death signal before evidence; the result is U unless positive
    evidence establishes P or F.
20. Questions 7b and 7c: cleanup deferred past the end of the permission, no service during the span, then a later
reminder; the retained recovery data and the permission to undo stay distinct. Do not make these pass using atomic
abstractions for "checkpoint", "durable file", "client attached", "withdrawn view" or "leftover". Models must not
label questions 5 and 7 green by adopting the recommended answers.

Properties to check at minimum (revision 11 restates the second and the third; revision 10 corrected the second: a perfect health classifier is not assumed, because the approved quiet-interval
heuristic can target healthy but unusually quiet work, so the checkable properties concern incarnation identity, gate admission and protected
executions): no foreign lock is removed; L2 ends only the scope of the recorded previous incarnation whose ownership has ended (the lock was acquired), L7 ends only the validated current incarnation after it holds the gate exclusively and has re-read the incarnation record, neither ever targets a replacement incarnation or an incarnation whose scope contains a protected execution it did not prove ended, and the protected-execution guarantee and its owner-death exceptions (the L2 reap, the hook rerun of B1) are stated and checked as a separate property; a protected running program is not cancelled;
every accepted identity stays recoverable to its immutable request or terminal outcome; no recovery copy is released while relevant destructive
execution can still occur; stale index entries are not silently reintroduced into unrelated commits; a superseded version cannot publish. Liveness is
stated with its fairness assumptions and is not made to pass by assuming that every gate holder, hook, child or owner eventually completes.
Explicit red controls named by the review: the intact-anchor deleted-suffix case, restart versus hook dispatch, the retained-descriptor scope
deadlock, same-identity concurrent acceptance, and cross-worktree maintenance overlap.

Later steps, unchanged in order (revision 10 adds: the exact-invocation dispatch-capability inventory for every retained child; actual `update-ref` and
reference-transaction diagnostic forms and classifier behavior including hook stderr; that a prune view includes `refs/cli-git/*`; files and reftable
behavior and the supported fallbacks for `O_TMPFILE`, `linkat` and `RENAME_EXCHANGE`; push exclusion and frozen-update execution; measured stall
intervals with explicit false-positive assumptions; and differentially tested alternatives for deterministic Git hangs, not an embedding direction alone): an offline narrowly eligible index writer with the extension policy (differential against Git, including
later Git behavior); disposable backend experiments (files and reftable, packed refs, `reference-transaction` hooks, linked worktrees,
interruption at real persistence boundaries, push forms against the installed Git); the carry-forward crash model before any filesystem
mutation code; the exit inventory from the 2.56.0 source; the stall-interval measurement of "Measured stall interval".
Each item then keeps its red control, then the change, then a guard shown red once.
