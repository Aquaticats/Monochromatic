# Commits never fail: review history (revision 11 onward)

Scope: `package/git-policy/cli`. This is the history of `doc/planning/cli-git-commits-never-fail.md`, moved out of it so the design
stays within the size a reviewer can read whole.
Every section below is superseded by the normative state machine of that design (revision 12 at the time of the move);
each section keeps its own note saying which mechanisms it names were withdrawn.
The revision 9, 10 and 11 changelogs and the answers to the ninth and tenth reviews were moved here in revision 12.

The first ten reviews, the revision 9 and 10 changelogs and the oldest moved sections are in `doc/planning/cli-git-commits-never-fail-review-history-archive.md`.

## Revision 12 changelog, moved from the design in revision 13

### Revision 12 changelog (for the step-1 models agent; every changed transition, with the model and the schedule)

Naming. Names are stable where the meaning holds. New names: rule V, `SYNC_LAG`, the syncer, the departure barrier,
the withdrawal mark (the file `lane/decision/<id>`, now with the sole content `withdraw`), the client lock
`lane/client/<id>`, the attempt directory `lane/p2/`, the decider, the register entries CAPTURE, SERVICE-ACQUIRE,
COMMITTING (redefined) and PREPARED-HOLD, `HOLD_AFTER`, `ATTACH_RECHECK`, the consumption kind `merge-install`, MG2
(publish) and MG3 (install) and MG6c (cleanup) with MG-A for the other order. Removed: the `commit` content of the
decision file and P2b as a link race, MG4, MG4n, MG5 and MG6 in the revision 12 order, the identity binding `ids/`,
and the per-side spool files. Each schedule goes red under revision 11 and green under revision 12; a model marked
"new" is added to step 1; the models branch is not edited by this document.
- P2, W1, W2, E1, rule B's COMMITTING (R11-1; models new "decider writes the line", "owner wedged after prepare",
  "owner death at each point of P2", "commit failed", the register model's COMMITTING entry and the W1-against-P2
  model's `commit` decision step become stale and are replaced). Schedules: (1) `prepare: ok`, the owner wedges before
  it writes anything, the client reaches its bound: red under 11 (the `commit` decision is durable, W1 waits in
  COMMITTING and exits 0, and the commit lands when the owner resumes); green under 12 (the client is the decider, so
  it either wrote `commit` itself, and the commit lands before the command returns, or it writes `abort` and nothing
  can land; the wedged owner has no deciding step). (2) the owner dies after `prepare: ok` with a client attached: red
  (the decision is `commit`, W1 cannot withdraw, recovery lands the request after the client returned); green (the
  client is the decider and writes `abort` or never writes `commit`; the child is ended by the parent-death signal, so
  nothing landed, W1 withdraws). (3) `commit` written, `update-ref` exits non-zero with no `commit: ok`: red (E1(d)
  says withdraw, but the decision stands); green (`store-failure`, P4, the attempt is closed). (4) `commit` written,
  `update-ref` still running at the COMMITTING bound: red (exit 0); green (the corner, non-zero, result not known;
  owner question 1). (5) the client stopped after `prepare: ok`: red (the branch lock is held without end); green
  (PREPARED-HOLD writes `abort`, a late `commit` is the second line and is not executed, or fails with `EPIPE`). (6)
  the client is killed from outside after `prepare: ok`: green (the owner becomes the decider, the request lands later
  as a request with no client).
- C1, C3 (R11-6; models new "capture between W1 and W2"). Schedule: R1 consumed `x=1`, W1 returns, the person stages
  `y`, a plain R2 captures: red (R2 omits `x`); green (the mark makes R1's entry void, R2 captures `x` and `y`).
- Execution ownership, exit rule, REAP (R11-4; models new "planned exit with a protected execution", "REAP expiry of
  an inline owner", "hook installed after the eligibility check"). Schedules: an inline owner starts a hook, the bound
  passes: red (the exit ends the hook); green (an inline owner starts no protected program; the service owns it).
  Inline REAP expiry with a durable request: red (a later trigger publishes it); green (W1 before exit).
- T1, G (R11-3; models F8 extended, new "admitted after the Pre, terminal before the reread"). Schedule: the eleventh
  review's: red (T1 releases); green (`L1` has a name that `L0` lacks, T1 withdraws).
- T0, W2, C3, rule D, T1-nocommit (R11-10; models new "terminal outcome racing W1, lookup and reclamation").
  Schedules: W2 orders the ledger line first; C3 answers terminal before any inbox or queue name; a lookup whose
  candidate vanished restarts through the terminal route; W1 after a refusal already final reports the refusal;
  `already` is a terminal kind; a held request's error names retained inputs, not a commit.
- Rule B, CAPTURE, SERVICE-ACQUIRE, L6, PUSH, the person's own programs (R11-9; models new "continuous invalidation
  against one deadline", "repeated recovery re-evaluation against one deadline", "failed shared gate try", "push with
  a pre-push hook"). Schedules: C1 invalidated for ever and C0 re-evaluating: red (no end); green (CAPTURE ends the
  command with no request).
- MG1 to MG3, MG6c, MG-A (R11-2, R11-5, R11-7, R11-8; models new "W1 before publication changes nothing", "P-carry
  with O, S, W, M distinct", "staged-only version saved before the index entry is replaced", "switch completing after
  publication and active-state removal", "install deferred behind a foreign index.lock", "capture between publication
  and install"). Schedules: the staged-only version `O = A`, `S = B`, `W = A`, `M = C`: red (neither the pre-merge pin
  nor the tip entry keeps `B`); green (`saved/<path-id>.staged` holds `B`). A switch finishing after the merge became
  `materialized`: red (nothing schedules MG6); green (`attach-watch`). Withdrawal with a foreign `index.lock` held:
  red under MG-A (restoration cannot happen at the bound); green under publish-first (nothing was written).
- Rule V, C2 to C5, S1, T0, the cost section (coordinator's input; models new "sync calls on the waiting path", "power
  loss at each write", "state pair torn write", "request file with a bad checksum"). Schedules: count the sync calls
  of a plain commit's waiting path: red under 11 (22 and more); green under 12 (0). A power loss that lands the newer
  state file in part: green (the older generation is valid). A power loss that lands the `inbox/` entry without the
  file's content: green (the checksum fails, the request is not accepted).
- Heading and wording (R11-11). The disclaimers, the guarantees, the exits and the owner questions P, W and M are
  aligned with the machine above.



## Answers to the eleventh review, moved from the design in revision 13

## Answers to the eleventh review

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-11.txt`, with a closure line: closed (a transition
answers the schedule; a model is owed), or owner question (closing it needs a contract the owner has not approved; the
mechanism is stated and the question is in "Owner questions for revision 12"). The verdict was: continued model work,
pure components without reclamation, and disposable backend experiments only. This revision builds nothing.

- R11-1 (P1), a durable `commit` decision is not publication: accepted. The decision file no longer carries `commit`.
  The client decides the publication by writing `commit` or `abort` to the pipe of the prepared transaction (P2b), and
  the owner never decides while the client lock is held; a wedged owner and a dead owner both lose their deciding
  role. Closed for the owner-wedged, owner-death and post-decision failure schedules (E1(d) is split by whether
  `commit` was written). Owner question 1 for the remaining corner, a branch write already issued when the bound
  passes.
- R11-2 (P1), checkout-first withdrawal cannot meet the bound and unchanged staging: accepted. Publish-first (MG1 to
  MG3) writes nothing before publication, so W1 changes nothing and the interlock between W1 and a multi-file checkout
  is not needed. Owner question 2, because it changes a delegated pick (M-A) and adds a window the owner has not seen.
- R11-3 (P1), G-order proof: accepted, the proof was invalid. T1 compares two listings of the registry around the
  check; any new name withdraws, whatever its state. Closed in the model; the listing boundary is stated.
- R11-4 (P1), inline owner exit: accepted. Protected programs run only under the service; an inline owner starts none
  and freezes its children's hook path; every exit of a client with an unpublished request is W1 (REAP and ACQUIRE
  included). Closed; no owner question.
- R11-5 (P1), P-carry data model: accepted. MG3 plans from `O`, `S`, `W` and `M`; the worktree takes the three-way
  merge of `O`, `W`, `M`; a staged-only version is saved before its index entry is replaced; modes, deletions, type
  changes and unmerged entries keep the person's version with the merged one beside it (the earlier owner decision
  "Carry edits forward", adopted, veto open). Closed under publish-first; under MG-A restoring `S` is amendment (v).
- R11-6 (P2), withdrawal not capture-visible until W2: accepted. C1 reads the withdrawal marks. Closed.
- R11-7 (P2), W-detect loses the cleanup after publication: accepted. The `attach-watch` entry outlives the request's
  active state; triggers are every lane event, a watch on `HEAD` by the service and one re-check; the liveness is
  conditional and says so. Closed; the owner's approved residue is unchanged.
- R11-8 (P2), merge continuations: accepted. Under publish-first there is no checkout state before publication, so the
  no-checkout and retarget gaps do not arise and the worktree-edit rule is MG3's; under MG-A amendments (iii) and
  (iv). Closed for the order the machine uses; owner question 2 decides.
- R11-9 (P2), the register is not exhaustive: accepted. CAPTURE, SERVICE-ACQUIRE, the L6 failed try, the budget rule,
  PUSH's hook phase, and the person's own programs are classified; owner question 3 asks whether the last have a
  bound.
- R11-10 (P2), terminal ordering: accepted. T0 is the only order, outcomes are absorbing, C3 prefers the terminal
  route, rule D is amended, `already` is a terminal kind, W1 reports a final outcome, a held request's error names
  retained inputs. Closed.
- R11-11 (P3), contradicting text: accepted and edited (the opening reading, rule B, the guarantees, the exits, the
  owner questions P, W and M, and a reading note on the earlier narrative sections).
- Section 4, backend-dependent obligations: kept as obligations in E1 and P2 (the pipe and the line order, the trace
  channel, the end-of-input abort, the signal that L5 sets). Section 5, controls: listed in "Models added from the
  eleventh review".
- Coordinator input after the review, the fourth component review's sync counts: answered by rule V and "Cost of the
  durable steps (revision 12)".
- Not a defect: none of the eleven findings was rejected.



## Owner questions for revision 12, full text as asked, moved from the design in revision 13

## Owner questions for revision 12

These are the contracts the eleventh review says the owner has not approved, plus one the coordinator added. Each is
written so it can be answered without the earlier messages. The design states a mechanism first wherever one meets the
owner's stronger contract; a question remains only where no mechanism can. The recommended option is marked and the
ranking gives a reason for each adjacent pair. None of the four is decided here.

### Question 1: a branch write that has already started when the time runs out

Plain context. A commit with a person's hook goes through three steps: the wrapper asks Git to get ready to move
`main` (the hook runs now), then the wrapper's command says "go", then Git writes the new commit into `main`. Your
rule is that when the time limit passes the command returns an error and nothing lands later by itself. The wrapper
now makes that true at every point before "go": at the limit the command says "stop" instead, and `main` cannot move.
The one point it cannot undo is a write that Git has already begun when the limit passes, because a started disk write
cannot be recalled. Example: `git commit -m "fix parser"`; the limit passes 2 s after "go"; the disk is stalled and
the write finishes 4 s after "go".
- Option A, a short extra wait, then an error that says the result is not known (Recommended). The command waits up to
  a few more seconds (a shipped constant, unmeasured) for the write it started. Visible here: the write finishes in
  time, the command prints `published` and exits 0. If the disk is still stalled it exits non-zero: "the write to main
  had started and had not finished; run `git log -1` to see whether the commit landed; if it did not, the commit is
  kept on refs/cli-git/<instance>/withdrawn/<id>". Pros: an ordinary slow disk gets the right answer. Cons: the
  command can run a few seconds past the limit, and in the stalled case the commit may land after the error, which is
  the one exception to "nothing lands later".
- Option B, stop at the limit and say the result is not known. Visible here: the command exits non-zero at the limit
  with the same message, and the commit lands 2 s later anyway. Pros: the command never runs past the limit. Cons: it
  reports "not known" for writes that were about to finish, and the exception to "nothing lands later" is hit by every
  slow write, not only stalled ones.
- Option C, wait for the write with no limit. Visible here: `published` after 4 s; with a disk that never answers, the
  command never returns. Pros: the answer is always exact. Cons: it breaks "nothing should ever wait forever or
  indefinitely" for a hung disk.
- Ranking: A > B > C. A beats B because it turns the slow-but-working case into a correct `published` and leaves the
  same exception only for a stalled write; B beats C because B always returns, and C can wait for ever on a hung disk.

### Question 2: merge order, and what a stopped merge leaves behind

Plain context. A merge writes new file contents into your folder and moves the branch. Today the wrapper writes the
files first and moves the branch last (the order the coordinator picked, "M-A"). Your bound answers make that order
costly: if the time limit passes after the files were written, the command must put your files back, and putting them
back needs Git's `index.lock`, which another program (an IDE, a `git status`) may hold for as long as it likes.
Example: `git merge topic` into `main` changes `x.txt`; a slow `reference-transaction` hook keeps `main` from moving
for the whole limit; at the limit your IDE holds `.git/index.lock`.
- Option A, move the branch first, then write the files (Recommended; the machine's MG1 to MG3). Visible here: for the
  whole limit your folder is exactly as before; at the limit the command prints "withdrawn", your files and staging
  are unchanged, and the merge is kept for a retry. If the hook accepts in time, the branch moves to the merge and the
  files follow: if the IDE holds the lock at that moment, `git status` shows the old files as changes until the lock
  frees, and the command says so. In that window a plain wrapper `git commit` is safe, but a `/usr/bin/git commit -a`
  run directly (not through the wrapper) commits the reverse of the merge on top of it (the history keeps both, and
  `git revert` undoes it). Pros: the promise "staging as it was" holds by construction; no lock is held while the hook
  runs; a merge needs no restoring. Cons: a lag window in which the branch and the files disagree, with the
  `commit -a` hazard above (a case of the same kind as the switch window you already accepted).
- Option B, write the files first and keep `index.lock` until the branch has moved or the limit passes. Visible here:
  the IDE cannot take the lock while the merge waits (its own command fails with "index.lock exists" for up to the
  limit); at the limit the wrapper restores your files and releases the lock, and your folder is exactly as before.
  Pros: restoration is always possible, and files and branch never disagree after the merge. Cons: every other program
  that needs the lock fails for as long as the hook runs (native Git does the same during an editor), and it adds a
  lock held across a person's program.
- Option C, write the files first and restore later (the order the machine had in revision 11). Visible here: at the
  limit the command prints "withdrawn" and exits, but the merge's files and its index entries are still in your
  folder, because the IDE holds the lock; they are put back when the lock frees. `git status` right after the error
  shows the merge's changes, not your earlier staging. Pros: the files and the branch agree when the merge succeeds.
  Cons: it breaks "your staging stays as it was" at the error, and the restore depends on a lock the wrapper does not
  control.
- Ranking: A > B > C. A beats B because it keeps the promise without holding a lock that other programs need, at the
  price of the lag window; B beats C because B keeps staging unchanged when the error returns and C does not.

### Question 3: do the person's own programs before the commit exists have a time limit?

Plain context. Before a commit exists the command runs your own programs in its foreground: your editor, `pre-commit`,
`commit-msg`, a signer prompt, and a `git gc` the wrapper forwards for you. The command is those programs, as in
native Git. Your decision "End Git, hold hooks" says the wrapper never ends your hook; your rule says nothing should
wait for ever. They collide here. Example: `pre-commit` runs your test suite for 40 minutes.
- Option A, no limit; Ctrl-C ends the command (Recommended). Visible here: after 40 minutes the hook ends and the
  commit goes on; if the hook hangs for ever the command stays until you press Ctrl-C. Pros: it matches native Git and
  your "hold hooks" decision, and leaves no program running unwatched. Cons: a hung hook of yours waits for ever,
  which "nothing should ever wait forever" does not allow in general.
- Option B, a limit; at the limit the command exits with an error and leaves the hook running. Visible here: after the
  limit "stopped waiting for pre-commit, which is still running; nothing was committed"; the test suite finishes later
  with nobody reading its result. Pros: the command always returns. Cons: a program of yours keeps running after its
  command is gone, possibly changing files.
- Option C, a limit, and the hook is ended at it. Visible here: "pre-commit ended after the limit". Pros: nothing is
  left running. Cons: it ends a hook of yours, which "End Git, hold hooks" forbids.
- Ranking: A > B > C. A beats B because B leaves your program running with no command watching it, while A keeps the
  command and the hook together; B beats C because C breaks your explicit decision that a hook is never ended.

### Question 4: how soon must an accepted commit survive a power cut?

Plain context. The wrapper writes a request file, an order line, a state file and a ledger line for each commit, and
today it forces each to disk (about 22 forced writes before the request is accepted, 29 through the end, for a
one-path commit). Each forced write can take seconds when the disk is busy, which cannot meet "commits vs native git
should not take 5s longer." and "Disk pressure being high is not an excuse." Surviving a process kill needs no forced
write; only a power cut or kernel crash needs them. Example: you run `git commit`; the wrapper accepts it; 1 s later
the power fails while a slow hook still holds the branch.
- Option A, force everything to disk before the command is told "handed over" (revision 11). Visible here: after the
  reboot the wrapper finishes your commit. Pros: nothing accepted is lost on a power cut. Cons: 22 or more forced
  writes on the waiting path; seconds each under pressure.
- Option B, force to disk before the command leaves without a published commit, and in the background soon after
  (Recommended; rule V). Visible here: the commit is lost on the power cut, as it would be with native Git; your
  staged changes and files are intact, so you run the commit again. If the command had left earlier (the error at the
  limit, or handed to the service), its files were forced out first and survive. No forced write on the waiting path.
  Pros: meets the 5 s rule for the wrapper's own writes; every promise made when the command leaves is kept. Cons: a
  power cut while the command is still waiting loses that attempt, the same loss native Git has.
- Option C, background only, never force at leaving. Visible here: after "withdrawn, the commit is kept on a side ref"
  a power cut in the next seconds can lose it. Pros: nothing is ever waited for. Cons: it can lose the commit the
  command just promised to keep, which "never discard user data" forbids.
- Ranking: B > A > C. B beats A because it keeps every promise made when a command leaves and removes the forced
  writes from the waiting path; A beats C because A never loses accepted work and C can lose a commit the command said
  was saved.

### Adopted without a question (the coordinator may veto; each follows from a rule the owner already gave)

- Protected programs (hooks run by the lane, signers, helpers) run only under the service, never in a foreground
  client; where no service can start, the command ends at START with an error and the work stays saved ("The person's
  hook is never ended" and the bound answers).
- A merge that cannot carry a staged or edited path as text (a mode or type change, a staged deletion, a symlink, an
  unmerged entry) leaves the person's version in place and writes the merged version beside it ("Carry edits forward",
  "never discard user data").
- A staged-only version that a merge replaces in the index is saved to `saved/<path-id>.staged` before the entry is
  replaced ("never discard user data").
- A hold ref is created only for a request that waits longer than `HOLD_AFTER` ("Invoke real git as rarely as
  possible"; the hold ref was never load-bearing).


## Models added from the seventh, eighth and ninth reviews, moved from the design in revision 13

Models added from the seventh review (each starts as a failing control with the review's schedule): the fork-to-exec window with the owner dying
and the next owner waiting; the service cgroup reused by a replacement owner (it must not kill itself); the gate with the restarter dying at every
point of L7, a hook admitted after the check, and a wedged shared holder; a pid read in another pid namespace; the foreign lock taken while a child
is stalled before acquiring it, and after it released and stalled before exit (no deletion may occur); the reserved binding (C3 done, C4 not) with a
same-content resend, a different-content resend, and a partial binding that cannot exist; maintenance with a launcher alive and an empty scope, with
process-group escape, and release racing the run; the seqlock with a hook that waits for ever during P2; the immediate follow-up after a completed
switch; a held request completed after its amendment, with the consumption entries across deferred materialization; a failed swap followed by the
identical commit installed and removed by another writer.
Models added from the eighth review (each starts as a failing control with the revision 9 changelog's schedule): a restarter racing hook
dispatch with no gate, and an owner that ignores the terminate signal; two lanes with a ledger tail repair; a same-identity resend that reads a
non-durable record; a dead odd generation, a moving tip and a mutation of the logical head between the two reads of capture; a classification
race for `update-ref` against a hook installed after it; the foreign-successor schedule followed by a plain commit of an unrelated path; a
held version superseded by an amendment; maintenance admission racing the final release of T1; each crash boundary of M1 and each per-path
outcome; a merge refused by a hook after its checkout with a concurrent editor, a lost swap after a checkout; the post-anchor reflog erased.

Models added from the ninth review (each starts as a failing control with the revision 10 changelog's schedule, and the model represents observations,
not desired conclusions). Every one of these is a control: (1) C0 relinquishing ownership while its client lives, then an owner's L2 reaping; (2) a
visible but unflushed binding followed by a durable inbox acknowledgement, with observation allowed at every link-before-directory-fsync boundary
(`NineFsyncOnExists` is the inbox instance); (3) an unnamed journaled inode that disappears before linking, followed by inode-number reuse; (4) recovery
identity inspection racing pathname replacement; (5) rename-to-private moving a replacement live lock; (6) unrelated staging across the merge checkout;
(7) a completed `git switch` before a pending merge resumes, and a switch finishing between the guard and the end of the install; (8) a capture between
`checked-out` and the swap, followed by a refusal; (9) restoration of a carry conflict with no concurrent editor, and with one; (10) late maintenance
taking its view during a temporary loss of reachability, with the hold root present and with it absent; (11) reflog suffix deletion restoring exactly the
recorded anchor; (12) a hook refusal followed by a foreign ref update or lock acquisition before E1 reads anything; (13) held amendments that add and that
drop paths; (14) every merge resume point and every hook-completion crash boundary; (15) the state document size against ledger length (cost flat); (16)
the revision counter and the marker with an ordinary replacement between two marked ones.

## Revision 13 changelog, moved from the design in revision 14

### Revision 13 changelog (for the step-1 models agent; every changed transition, with the model and the schedule)

Naming. New names: the unresolved exit and the mark content `withdraw-unresolved <attempt>`, `submitted a` and
`closed a` (attempt scope), E1 classes P, F and U, the state log `lane/state.log` with hash-chained records and
checkpoints, the departure, reclamation and preservation barriers, `WAIT_WITHDRAW`, `WAIT_DEPART`,
`ATTACH_WATCH_SPAN`, the invocation nonce, numbered registry names with the probe of the next number, the register
entries WITHDRAW and DEPART. Removed: the state pair, the `declined` request-side effect on a `merge-install` path,
MG-A (to the history record), `ATTACH_RECHECK`. Each schedule goes red under revision 12 and green under revision 13;
a model marked "new" is added to step 1; the models branch is not edited by this document.
- W1, mark, P4, Q1, E1 (R12-1, R12-3; models new "normal exit then retry", "unresolved attempt never replayed", "ended
  child, reset branch, no report"). Schedule: the client submits `commit`, the child stalls, COMMITTING expires, the
  client exits with no mark, the child ends without landing, P4 retries and lands: red under 12 (a fresh attempt after
  a normal exit); green under 13 (the exit creates `withdraw-unresolved a`; no attempt after `a` can start; the
  outcome is `unresolved` or `published` for `a` only). Schedule: Git writes `new`, another writer resets the branch
  away, Git dies before `commit: ok`: red (E1 says the attempt did not land and retries); green (class U, never
  retried; R3 gives `rewritten-away` when the reflog shows it).
- P2 (R12-2; models new "pipe bootstrap and descriptors", "buffered commit before end of input", "abort enqueued to a
  child that never reads"). Schedules: a retained write end prevents end of input; a short write; `EPIPE`; the read
  end retained by the parent hides the failure: red under 12 (unspecified); green (the bootstrap and the discipline
  are specified and each failure is a failure of that writer). PREPARED-HOLD against a child that never reads again:
  red (the text claimed lock release); green (the claim is the enqueue only, and the residue is stated).
- Rule V, state log, ledger, order file, journal (R12-4; models new "two unsynced overwrites tear both slots", "later
  record persists without its dependency", "checkpoint rotation racing the syncer"). Schedules: the alternating pair
  `g0` durable, `g1` and `g2` unsynced, power loss: red (both torn, `g0` gone); green (an append-only chain keeps the
  longest valid prefix, and the old file is removed only after the new one is D). A valid ledger line with its
  predecessor missing: red (believed); green (the chain breaks and the reference to the removal record fails).
- T0, T1, rule V reclamation barrier (R12-5A; model new "terminal V then reclaimed then power loss"). Schedule:
  `published`, outcome V, `retired` V, spool released, the ledger tail lost, a resend: red (no request and no
  outcome); green (T1's Pre requires the outcome line and the removal record D).
- MG3, rule V preservation barrier (R12-5B; models new "staged-only version, index replaced, saved file lost",
  "carry-forward power loss under rule V"). Schedule: `S` read at install, the index entry replaced, power loss loses
  the saved file and its record: red; green (the barrier makes them D first).
- Departure barrier by exit kind (R12-5C; model new "every saving exit at every durability boundary"). Schedules:
  `conflicted`, `rewritten-away`, W1 finding a terminal outcome, and the unresolved exit, each at power loss right
  after the answer: red (no D-set); green (the D-set of that exit).
- W1 step (4), W2 step (2), C1 (R12-6; model new "capture against a withdrawn logical base"). Schedule: R1 pending
  with the index at `x=1`, W1, the person stages `y`, R2 captures: red under 12 (the base still contains R1, so `x` is
  omitted); green (W1 waits for W2, W2 re-bases the descendants, and C1 refuses to capture while a marked request is
  still in the document).
- Rule V, W1, the register (R12-7; models new "departure time after the bound", "syncer blocking the owner").
  Schedule: the bound passes, six syncs run, the command returns late: red; green (`WAIT_WITHDRAW` and `WAIT_DEPART`
  are named bounds, and a departure that cannot confirm exits with the not-yet-confirmed message).
- MG3 composition and authority (R12-8; models new "later staging during the lag", "S differs from W"). Schedule: the
  merge returns with the install deferred, the person stages `z`, MG3 runs: red (`z` replaced by `M`); green (`E`
  recorded at publication; an entry that differs is left, reported as `later-staging`).
- MG6c, `attach-watch` (R12-9; models new "switch paused longer than the span", "switch begins after the install",
  "leftover test"). Schedules: the switch finishes after the span: red under 12 (a timer dropped the obligation and
  the text promised a next check); green (the residue is the one in owner question 7, stated). A switch that begins
  after the install: red (MG6c restores `N` over Git's own result); green (a path is a leftover only when the index
  equals `M`, the file is `R`, and the new `HEAD`'s tree entry is not `M`'s).
- MG2 capture rules (R12-13; models new "edit to another line during the lag", "commit -a and named path during the
  lag"). Schedule: `M` changes line a, the lagging file still has `O`, the person edits line b, `commit -a`: red (the
  full-file capture reverts a); green (the three-way merge of `O`, `W`, `M` is captured).
- T1, G1 (R12-10; models new "T1 recovery without its original L0", "registry probe", "compaction with a predating
  release intent"). Schedules: crash after step (2), restart at step (2) with a lost baseline: red; green (restart
  from the Pre with a fresh `L0`). A reservation linked after `L0` and before the probe: green (the probe or `L1`
  finds it).
- C2, C3, nonce (R12-11; model new "identical operation after a reset"). Schedule: publish, reset to the base, repeat
  the same operation: red (answered with the old outcome); green (a new nonce makes a new identity; a resend keeps its
  nonce).
- B3, MG-A, the rationale and summary texts (R12-12). B3 appends the ledger line first and uses the
  `outcome ... superseded by` form; MG-A is retired to the history record; the remaining contradictions are edited.



## Answers to the twelfth review, moved from the design in revision 14

## Answers to the twelfth review

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-12.txt`, with a closure line: closed (a transition
answers the schedule; a model is owed), or owner question (closing it needs a contract the owner has not approved).
The verdict was: continued model work only. This revision builds nothing. The review did not inspect the cited Git
source lines; they stay gates in E1 and P2.

- R12-1 (P1), a normal COMMITTING exit authorizes fresh attempts: accepted. Every normal exit creates the withdrawal
  mark first; the unresolved exit writes `withdraw-unresolved a`, which forbids any attempt after `a`; the owner
  observes `a` only. "Submitted" is attempt-scoped and ends only on a positively established close (class F). Closed.
- R12-2 (P2), the pipe is not a publication decision with the claimed bounds: accepted. The text says the pipe orders
  instructions; PREPARED-HOLD bounds the enqueue and not the lock release (the residue is stated); the bootstrap, the
  single-write discipline, descriptor handling and the buffered-input facts are written; the source facts are labelled
  gates. Closed as specification; backend gates remain.
- R12-3 (P1), an ended child and an absent tip do not prove nothing was published: accepted. E1 has three kinds, P, F
  and U; only F allows a retry; U is never retried and N-silent covers F only. Closed.
- R12-4 (P1), the unsynced pair can lose both generations: accepted. The state is a hash-chained append-only log with
  checkpoint files removed only after the new one is D; recovery takes the longest valid prefix; cross-file references
  are validated. Closed.
- R12-5 (P1), rule V removed barriers needed for reclamation and preservation: accepted. A reclamation barrier (T1's
  Pre), a preservation barrier (before a unique version's recovery authority is replaced; the carry-forward ordering
  stands) and a D-set for every saving exit. Closed; the cost is only on the paths that need them.
- R12-6 (P1), marks do not remove withdrawn work from the capture base: accepted. W1 waits for W2, W2 re-bases the
  descendants, and C1 refuses to capture while a marked request is still in the document. Closed.
- R12-7 (P2), zero waited-for syncs is a narrow target: accepted. The three promises (start, exit, durable at exit)
  are separated with named bounds `WAIT_WITHDRAW` and `WAIT_DEPART`; the counts are corrected (`N + 5`, the journal,
  the hook runner); the syncer is never in the owner's event loop. Owner question 6 for the slow-departure message.
- R12-8 (P2), P-carry composition and later staging: accepted. MG3 states the composition by case, uses the
  publication-time entry `E` to leave later staging alone, and saves a staged-only version. Owner question 5, because
  the owner saw only the common case.
- R12-9 (P2), `attach-watch` can settle early: accepted in part. No timer is evidence that a switch has finished, so
  the obligation is retained for a named, measured span and MG6c's leftover test and undo target are defined; the
  weaker residue is stated and put to the owner as question 7.
- R12-10 (P2), the listing contract and the T1 recovery baseline: accepted. Numbered reservations and a probe of the
  next number give an exact test for "no reservation after `L0`"; compaction keeps the retention condition; T1
  recovery restarts from its Pre. Closed; the probe is a backend gate.
- R12-11 (P2), identical operation versus resend: accepted. A fresh invocation nonce is part of the hashed request.
  Closed.
- R12-12 (P2/P3), remaining contradictions: accepted and edited (B3 order and line form, MG-A retired, locks,
  cancellation, merge-order narrative, retention, N-silent).
- Publish-first capture algorithm: accepted. MG2 gives the rules by request kind; a `merge-install` path is captured
  as the three-way merge of `O`, `W`, `M`.
- The four owner questions: answered as decisions 7 to 10; the review's framing corrections are stated in "Owner
  questions for revision 12 (answered)". Forwarded `gc` is covered by decision 9 as the owner saw it; the register
  lists it on its own line. The chosen merge order is option A of the revision 12 question 2.
- Not a defect: none of the findings was rejected.



## Owner questions for revision 13, full text as asked, moved from the design in revision 14

## Owner questions for revision 13

Three contracts that the twelfth review shows no mechanism can settle for the owner. The mechanisms for everything
else are in the state machine. The recommended option is marked and the ranking gives a reason for each adjacent pair.
None is decided here.

### Question 5: when what you staged is not what is in the file

Plain context. You chose "Merge anyway, unstage x": when a merge changes a file you have staged, your staged content
becomes an unstaged edit merged onto the result. That is clear when the staged content and the file are the same. This
question is about the other case, where they differ, for example because you staged a change and then put the file
back. Example: `x.txt` is `A` at the branch tip; you stage a change so the index holds `B`; you then edit the file
back to `A`; the merge makes `x.txt` `C`.
- Option A, the file you see is the edit that is carried; the staged-only version is saved aside (Recommended).
  Visible here: `x.txt` becomes `C` (your file had no change to carry), the index holds `C`, `git status` shows
  `x.txt` clean, and a line says your staged `B` is saved in `saved/x.txt.staged`. Pros: no confusing markers from
  content you cannot see in the file; nothing is lost. Cons: a staged change you meant to keep must be taken from the
  saved file by hand.
- Option B, merge both: first the staged `B` onto the result, then the later change from `B` to your file. Visible
  here: `x.txt` holds conflict markers between `B` and `C` (both changed `A`), and your later revert is applied on top
  of those markers, so the markers can nest. Pros: your staged intent is carried into the file and nothing needs
  saving. Cons: nested markers are hard to read, and the revert you made on purpose is mixed into them.
- Option C, stop with an error and change nothing. Visible here: the command exits non-zero naming `x.txt`, and the
  merge is kept for a retry. Pros: nothing decided for you. Cons: it is not "merge anyway", which you chose.
- Ranking: A > B > C. A beats B because it keeps the file readable and loses nothing, where B mixes content you can no
  longer see into markers; B beats C because B still completes the merge as you asked and C returns the work to you.

### Question 6: a command that leaves while saving to disk is slow

Plain context. You chose "Force at leaving, else background": a command that leaves with saved work first forces that
work to disk. Forcing can take seconds when the disk is busy, and your rule is that the command returns at its time
limit. Example: a commit waits 600 s behind a slow hook and is withdrawn; forcing its saved copy takes 40 s because
the disk is saturated.
- Option A, wait for the force up to a short named limit, then leave and say it is not yet confirmed (Recommended).
  Visible here: after the limit the command exits non-zero: "withdrawn; your commit is saved on
  refs/cli-git/<instance>/withdrawn/<id>; it is not yet confirmed on disk and the wrapper keeps forcing it". Pros: the
  command always returns; the saved copy is on disk within moments in every ordinary case. Cons: a power cut inside
  that gap can lose the saved copy.
- Option B, wait for the force without a limit. Visible here: the command stays 40 s past its limit, exits with the
  exact promise, and stays longer on a worse disk. Pros: the promise made when it leaves is always exact. Cons: it
  breaks "nothing should ever wait forever or indefinitely" for a saturated disk.
- Option C, do not force at leaving; background only. Visible here: the command leaves at its limit and the saved copy
  reaches the disk later. Pros: nothing is waited for. Cons: it undoes decision 10 and can lose a copy the command
  just said was saved.
- Ranking: A > B > C. A beats B because it always returns and gives the true state of the copy; B beats C because B
  keeps your decision to force at leaving and C drops it.

### Question 7: how long the wrapper watches for a late `git switch` after a merge

Plain context. You accepted a rare leftover: if a `git switch` finishes just after the wrapper wrote a merge's files,
those files can show up as changes in your new branch until the wrapper's next check restores them. The wrapper cannot
tell when a paused switch will finish, so "until the next check" cannot be kept for ever without risk. Example: in one
terminal `git merge topic`; in another a `git switch other` is paused for 30 s after writing its index and before
updating `HEAD`; the merge's files are written in that gap.
- Option A, watch for a measured span after the merge and then stop (Recommended). Visible here: if the switch
  finishes inside the span (a constant that must be measured from real switch durations, with a margin you approve),
  the wrapper restores the files from its saved copies; if it finishes after the span, the merge's files stay as
  changes in the new branch, nothing is lost, and the line printed by the merge names how to undo them. Pros: it never
  undoes anything you did later on purpose. Cons: a switch slower than the span leaves the files in place.
- Option B, watch until the files change. Visible here: the wrapper restores the files whenever it next sees the
  leftover, however long after. Pros: no span to outlast. Cons: days later it may undo a state you made on purpose
  (for example you staged the same file deliberately), which is worse than the rare leftover.
- Option C, stop the late switch by holding `HEAD.lock` while the files are written. Visible here: the other
  terminal's `git switch` fails at its last step with a lock error, after it has already changed its index and files.
  Pros: the leftover cannot happen. Cons: it makes a native Git command fail halfway and it is a lock held against a
  program the wrapper does not own.
- Ranking: A > B > C. A beats B because it bounds the chance of undoing a deliberate change, which B does not; B beats
  C because C breaks a native command midway, and B at worst restores a leftover late.


## Questions for the owner, full text, moved from the design in revision 14

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
C-class. Which commands may the stall rule cancel (new in revision 9; for the coordinator to pick).
   Case: the service runs `update-ref` for `main`; no `reference-transaction` hook is configured when it starts; it stalls; before the wrapper
   signals it, another program installs a `reference-transaction` hook; Git has not yet reached its dispatch point.
   - Option C-kind, classify by the kind of command and exempt `update-ref`, `commit` and every command that can dispatch a protected program,
     whatever is configured. Pros: no race can end a hook; consistent with "End Git, hold hooks". Cons: a stalled `update-ref` (a Git bug) with
     no hook cannot be ended, the request behind it stays pending, and A-kill has no object until a writer is embedded.
   - Option C-freeze, run `update-ref` with its dispatch inputs frozen (a hooks-path snapshot and configuration overrides) so the set of
     programs it can launch cannot change after classification, and allow X1. Pros: a stalled `update-ref` can be ended, A-kill applies.
     Cons: a hook installed during the attempt is not run for it; the freeze must be verified for each retained Git version and for filters,
     signers and helpers as well; more machinery.
   - Option C-sample, re-read the configuration at cancellation time. Pros: simple. Cons: the race remains and can end a person's hook,
     which the owner's decision forbids.
   - Ranking: C-kind > C-freeze > C-sample. C-kind beats C-freeze because it keeps the hook guarantee without new machinery, at the cost
     of leaving an internal wedge to the person; C-freeze beats C-sample because it removes the race by construction instead of narrowing
     its window.
   - Picked (coordinator, 2026-11-03, under the owner's delegation, veto open): C-kind, the machine's default. The owner's
     "End Git, hold hooks (Recommended)" puts the person's hook first; the wedge it leaves needs a Git bug and stays named and durable.
H. Hook runs when a held request is replayed (new in revision 9; for the coordinator to pick).
   Case: a configured pre-commit hook takes minutes; it succeeds for the captured content; meanwhile the branch moves, so the commit must be
   built again on the new tip.
   - Option H-version, the hook runs once per version of the request; a success is recorded and a replay on a moved tip does not run it again.
     Pros: no held, success, replay, held loop; the person's slow program is not re-run by a foreign writer's activity. Cons: the hook saw
     the captured content, not the new tip's combination; a hook that checks the combination does not see it.
   - Option H-build, the hook runs once per build, so it sees every combination. Pros: the hook judges what is published. Cons: on a branch that
     keeps moving the hook may never finish before the tip moves, so the request may stay held for ever; each foreign commit re-runs the
     person's program.
   - Ranking: H-version > H-build. H-version beats H-build because the owner's rules are that nothing is waited for without end and a
     person's program is not repeated, and the loss (a combination check by the hook) is the same loss every ordinary Git commit has when
     a pre-commit hook ran before a later merge.
   - Picked (coordinator, 2026-11-03, under the owner's delegation, veto open): H-version, the machine's default, for the ranking's reason
     and the owner's "You should (almost) never be waiting."
M. Merge order (new in revision 9, restated in revision 10; for the coordinator to pick).
   Case: a merge whose hook refuses after the files were already written, while an editor keeps saving one of the merged files.
   - Option M-A, check out first, swap second, with restoration from pinned pre-merge files (MG6) and retargeting (MG5). Pros: `M` enters the
     logical tail in the same atomic replacement as the checkout, so a capture in the interval takes nothing of the merge; a refusal restores the
     unedited paths exactly. Cons: the worktree and the branch are apart in `checked-out` and while the swap is pending (revision 9 wrongly
     claimed otherwise); more machinery; a refusal with a concurrent editor leaves the edited path as the person has it, with saved files.
   - Option M-B, swap first, then install by the per-path rule (the ninth review's reading of the first review's objection answered by a
     worktree-side suppression: a consumption entry would carry the digest of the pre-merge file, and C1 would treat a worktree file equal to it
     as no edit, and a request that names a path whose worktree differs would wait for the install). Pros: a refusal changes nothing because nothing was
     written; there is no reverse path. Cons: until the install, the branch is the merge and the worktree is the old head, so a
     native `/usr/bin/git commit -a` (outside the wrapper) commits a reversal of the merge, and a wrapper request that names an edited merge
     path must wait for the install, which can wait on a foreign `index.lock`; C1 gains a rule and the models gain a class of schedules.
   - Ranking: M-A > M-B. M-A beats M-B because M-A's open residues are bounded and recoverable (saved files, a restoration report), while
     M-B's native-git window can silently lose the merge's content, which "never discard user data" excludes, and it adds a waiting state.
   - Revision 12: the bound answers change the cost of this order (a withdrawal after the files were written must
     restore them through a lock a foreign program may hold), so the order was put to the owner again, who chose
     publish first (decision 8).
   - Picked (coordinator, 2026-11-03, under the owner's delegation, veto open): M-A, the machine's default. Revision 10 keeps it, repaired for
     the ninth review (delta install, attachment authority, atomic capture representation, restore from pins). Confirmed after revision 10
     (coordinator, same delegation): M-A stays, because M-B's window lets a native commit record the merge's reversal.
R. A stranded `index.lock` made by the lane's own dead incarnation (new in revision 10; for the coordinator to pick).
   Case: the service is killed by an outside kill between creating `index.lock` and renaming it; the next commit's follow-up finds the lock.
   - Option R-never, the lane never removes `index.lock` (rule N); it reports which incarnation made it and defers M1 and merges.
     Pros: no schedule can remove a foreign program's live lock; one rule for every shared pathname. Cons: the person clears the lock by
     hand, so a lane crash inside this short window leaves a manual step (the same step Git itself needs after its own kill).
   - Option R-pin-and-unlink, keep a hard-link pin under `lane/` so the inode cannot be reused, compare the identity, then unlink the
     pathname. Pros: self-heals in the common case. Cons: the pin fixes inode reuse only; the compare-then-unlink race remains (a cleaner
     removes our lock, a foreign program creates a new one, the unlink removes it), and this is the flaw of the shipped sweeper again.
   - Ranking: R-never > R-pin-and-unlink. R-never beats R-pin-and-unlink because the pin leaves the pathname race standing, and that race
     can break a mutual exclusion that the owner's "never discard user data" and Git's own protocol depend on, while R-never costs one manual
     step in a rare window.
   - Picked (coordinator, 2026-11-03, under the owner's delegation, veto open): R-never, for the ranking's reason; it matches the shipped
     wait on an unproven `index.lock` owner, which names the file and waits.
P. A merge that touches a path the person has staged (restated in revision 11 under the owner's bound rule; for the coordinator to pick).
   Case: `x` and `y` are staged, the merge changes `x`; Git itself would refuse (local changes would be overwritten). Under the owner's answers a merge
   that waits ends with an error at its bound and nothing lands later by itself.
   - Option P-carry, treat the staged content of `x` like an unsaved edit: three-way merge it onto the merged result and leave it unstaged. Pros: the
     merge never waits and never fails ("Manual merge should never fail too."), nothing is lost. Cons: the person's staging of `x` becomes an unstaged
     edit they did not ask for, and the three-way merge on an index entry needs its own differential controls.
   - Option P-refuse, withdraw at once: the merge changes nothing, exits non-zero naming `x` and the command to run again after `x` is committed or
     unstaged. Pros: nothing reshaped. Cons: the merge fails on the person's staging although only they can resolve it, which is against the
     never-fail rules the owner has not waived for this case.
   - Option P-wait, wait up to FOREGROUND, then withdraw as in P-refuse. Pros: a commit of `x` made in another terminal resolves it. Cons: the same
     failure as P-refuse after a pointless wait, because the person is at the blocked terminal.
   - Ranking: P-carry > P-refuse > P-wait. P-carry beats P-refuse because it is the only option that neither fails nor waits, at the price of reshaping
     staging without loss; P-refuse beats P-wait because it reaches the same end without the wait the owner wants
     gone. The coordinator's earlier pick, P-wait, was replaced by the owner's decision below; revision 12's MG1 and
     MG3 implement P-carry.
   - Earlier pick (coordinator, 2026-11-03, before the bound rule, veto open): P-wait, because of "never discard user data"; the wait was on a decision only the
     person can make, and Git itself refuses this merge. Re-opened by the bound rule.
   - Owner's decision (question tool, 2026-11-03, answered before the 12:56:51-05:00 commit that records it): "Merge anyway, unstage x", that is P-carry. The option read: "Your staged x becomes an
     unstaged edit merged onto the result, so nothing is lost and the merge completes." It needs its own differential controls against Git; MG1 changes
     from P-wait to P-carry.
W. The attachment concession for merge file writes (restated in revision 11; for the coordinator to pick).
   Case: `git switch topic` finishes its index write and has not yet updated `HEAD` (`builtin/checkout.c:927` commits the index, `:1261` updates the refs
   after); the lane's merge for `main` takes `index.lock` in that instant, reads `HEAD` as `main` and installs files; or the switch resumes after
   every check the lane made.
   - Option W-detect, authority to do before the writes, a check again before the swap and at every lane event, and restoration by authority to undo
     (MG6) from pinned files. Pros: the merge never waits for the person's switch; nothing is lost. Cons: it is not a fence; a switch finishing after the
     last check leaves the merge's files visible as local modifications in the new branch's worktree until the next check, which is a persistent wrong-attachment
     effect and not a momentary write; the owner approved that residue (the owner's decision below).
   - Option W-stop, never write worktree files for a merge unless the person asks (the merge is computed and kept, the error gives the finishing command).
     Pros: no wrong-attachment residue at all. Cons: "Auto merging should never fail." is met by saving, but the merge does not complete by itself, and it is an
     error exit under the bound answers.
   - Ranking: W-detect > W-stop. W-detect beats W-stop because it completes the merge as the owner asked and loses nothing, while W-stop returns the work to the
     person; the residue of W-detect is why this is an approval question and not a technical one.
   - Earlier pick (coordinator, 2026-11-03, before the tenth review, veto open): W-detect, because of "Auto merging should never fail." The tenth review does not
     approve W-detect as specified in revision 10; revision 11 repaired it and the owner then approved the residue
     (the owner's decision below); revision 12 keeps it in MG6c and moves its trigger to a retained `attach-watch`
     entry.
   - Owner's decision (question tool, 2026-11-03, answered before the 12:56:51-05:00 commit that records it): "Allow the rare leftover", that is W-detect with the residue approved. The option read:
     "Merges keep completing by themselves; in the rare instant above, the merge's files can show up as changes in your new branch until the wrapper's next
     check restores them from saved copies." The option also named the cost: "a `git commit -a` in that window could commit them to the wrong branch."
G. Retention against late maintenance (restated in revision 11; for the coordinator to pick).
   Case: T1 sees no open maintenance run; a run is admitted and takes its reachability view while the branch is temporarily elsewhere; the branch moves
   back; T1 releases the recovery copy; the run deletes by its earlier view. The tenth review showed that a hold root created at adoption does not cover
   later builds, that a root can be deleted by a person, and that a mixed-time view is possible, so a root cannot be the proof.
   - Option G-order, T1 re-reads the maintenance registry after the connectivity check and before any release; the hold ref stays as extra protection. Pros: no
     wrapper-owned ref is load-bearing, no deletion contract, no reliance on how a prune reads refs; the proof is an order of two durable facts. Cons: a
     retirement can be deferred for as long as a run is open (disk, not a wait); the residue (a person rewriting the branch after the check) remains.
   - Option G-root, keep the root as the invariant and maintain it through every build, replay and retarget, with a contract that deleting a recovery ref
     revokes the guarantee. Pros: protects a view taken at any time while the root exists. Cons: the invariant must be maintained at every transition and
     a person's deletion is undetectable after the fact; a root that cannot be written constrains retirement; it relies on a verified prune behaviour.
   - Option G-handshake, G1 reads release intents after its reservation and defers on an intent whose lane has a live owner. Pros: no reliance on refs.
     Cons: a maintenance run waits on a live lane, which the owner's bound rule makes a registered wait with an error at the bound, and a wedged owner blocks a
     person's `git gc`.
   - Ranking: G-order > G-root > G-handshake. G-order beats G-root because it needs no invariant over wrapper-owned refs and no contract with the
     person; G-root beats G-handshake because it adds no wait to maintenance.
   - Earlier pick (coordinator, 2026-11-03, before the tenth review, veto open): G-root, because of "You should (almost) never be waiting."; the machine now
     carries G-order with the root as extra protection, and the pick is for the coordinator to re-make.
   - Picked (coordinator, 2026-11-03, after the tenth review, under the owner's delegation, veto open): G-order, for the ranking's reason; it adds no wait.
N. The retry of a live owner after a proven store failure (new in revision 11; for the coordinator to pick).
   Case: P2a failed because a branch lock is held; the live owner records it and retries on the trigger; meanwhile another writer could in principle have
   installed the abandoned commit by its id and removed it again, and the erased reflog would show nothing. The tenth review (R10-8) asks that this
   ambiguity be handled as unclear publication or explicitly conceded.
   - Option N-silent, the live owner retries with a newly built commit and says nothing. Pros: the common benign case (a held lock) has no extra notice;
     the abandoned object is usually unreferenced; a best-effort hold or pending ref may name it. Cons: in the
     far-fetched history above the deliberately removed change returns without the
     unclear-publication notice; this is a concession.
   - Option N-notice, every retry after a store failure carries the unclear-publication notice of decision 3. Pros: no silent concession. Cons: the notice
     appears on every ordinary held-lock retry, which teaches the person to ignore it.
   - Ranking: N-silent > N-notice. N-silent beats N-notice because the far-fetched history needs another program to
     find an unreferenced or hold-ref-named object and install it by id,
     while the notice would be noise in the common case.
   - Picked (coordinator, 2026-11-03, under the owner's delegation, veto open): N-silent, for the ranking's reason; it is recorded as a concession.
H2. Reruns of a hook after owner deaths (new in revision 11, refines H; for the coordinator to pick).
   Case: a hook performs an external effect, the owner dies before a durable result exists, the next owner runs the hook again, and dies again before a durable result.
   - Option H-episode, one new invocation for each recovery episode that lacks a durable result, with a notice each time. Pros: the request still completes
     after repeated outside kills. Cons: the hook's external effect may repeat once per such death; there is no total bound of one.
   - Option H-total, after one rerun the request stops running the hook: it ends with an error (W1, naming the hook and that its result is unknown) and the
     commit is kept for a retry. Pros: at most one repeat of an external effect. Cons: a second outside kill makes the commit fail with an error.
   - Ranking: H-episode > H-total. H-episode beats H-total because "Commits should never fail." and the outside-cause rule treat a kill as the outside's act,
     and a hook that is not idempotent is already covered by Git's own contract that a hook "may thus get called multiple times".
   - Picked (coordinator, 2026-11-03, under the owner's delegation, veto open): H-episode, for the ranking's reason.
B. A quiet owner that holds the gate with no person's program running.
   Case: the service owner took the shared gate for a ref transaction, wedged in our own code before launching the child (or after the child exited,
   before releasing the gate); no hook runs; a waiting `git commit` sees the quiet interval pass.
   - Option B-hand, do nothing automatic: the command prints how to restart the unit and waits; the request is durable. Visible: the command waits until
     the person restarts the service; this conflicts with "never stuck" if such a wedge occurs. Revision 11: the wait ends at the FOREGROUND bound with an error that
     names the unit and the restart command, the request is withdrawn and never lands later, and the commit is kept on a side ref (the owner's bound answers).
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


## Models added from the tenth, eleventh and twelfth reviews, moved from the design in revision 14

Models added from the tenth review (each starts as a failing control with the revision 11 changelog's schedule; observations, not completed logical operations): (1) M1 death after the intent and before lock creation; (2)
a present foreign lock after an unfulfilled intent; (3) every normal abort after a successful `index.lock` creation; (4) a detected switch followed by MG6; (5) a switch completing after MG3's post-check; (6) retargeting when the
lane's own checkout entries meet a pre-scan; (7) retargeting to a no-op with the old checkout still in place; (8) a hold ref that covers the captured inputs and not the replay, its write failure, its deletion, and
retirement racing a reachability scan with a mixed-time view; (9) a recovery helper that loses acquisition to an owner running a hook that never returns; (10) resend after candidate release and terminal lookup before retirement
is eligible; (11) a live lock-held followed by external satisfaction, removal and suffix erasure; (12) hook-descendant output after Git's final diagnostic; (13) repeated owner deaths before a durable hook result; (14) a
detached-to-symbolic change between the check and P2; and, from the owner's rule, (15) every register entry at its bound, W1 against the commit decision in both orders, a crash at each point of W1 and W2, and a prepared
transaction whose owner dies. "Root exists", "rename happened", "hook result is durable" and "terminal outcome is discoverable" are modelled by implementable evidence only (a file, an index content, a ledger line), never
by a boolean the protocol cannot read. Performance gates added: ordinary acceptance and publication fsync costs, large in-flight state, unresolved suppressions, and hold-ref management overhead, each against the owner's native-Git latency
requirement ("commits vs native git should not take 5s longer.").


Models added from the eleventh review (revision 12; each starts as a failing control with the revision 12 changelog's
schedule; each model represents observations, never the completed logical operation). The review's fifteen controls,
keyed to the transition that answers each:
1. A `commit` decision durable, the owner wedged: P2b (the client is the decider; no durable `commit` decision
   exists). Control: the owner stops after `prepare: ok` at every step; the client either wrote `commit` (and the
   commit lands before the command returns) or `abort`; the client is never an idle holder of the pipe's write end.
2. Owner death after the decision with an attached client at its deadline: P2 owner death, W1 step (3). Control: the
   owner dies before and after `commit` is written.
3. P2c failure and uncertainty after the decision: E1(c) and E1(d), P4, COMMITTING. Control: `update-ref` exits
   non-zero with no `commit: ok`; `update-ref` still running at the bound; a surplus line making it die after
   `commit: ok`.
4. Merge withdrawal while MG6 is blocked by a foreign `index.lock`: MG2 (nothing written before publication). Control:
   the MG-A variant is retired (decision 8); its red schedule is kept in the history record only.
5. W1 between MG2's check and MG3's effects: W1 against P1 and P2a (the mark checked after the reader exists).
   Control: W1 at every step before P2b.
6. Capture after W1 returns and before W2: C1 reads the marks.
7. Inline timeout exit with a protected execution alive: execution ownership. Control: a hook installed after the
   eligibility check; the frozen hook path.
8. Inline REAP expiry without withdrawal: exit rule, REAP.
9. Maintenance deleting after the connectivity check and terminal before the reread: T1 with `L0` and `L1`; the
   observation boundary of a listing.
10. P-carry with `O`, `S`, `W`, `M` distinct, a staged-only version absent from the worktree: MG3. Control: modes,
    deletions, type changes, unmerged entries.
11. Refusal and withdrawal restoring the pre-merge index after P-carry: publish-first has no restoration before
    publication (control: assert it); (MG-A is retired;) MG6c restores the worktree side after a switch.
12. A switch completing after publication and removal from the active state: MG6c with `attach-watch`, its triggers
    and its settle rule.
13. Retarget with unchanged index and edited worktree, repeated retargets, no-checkout publication failure: MG1 on a
    lost swap (nothing to reconcile); (the MG-A amendments are retired with MG-A.)
14. A terminal outcome racing W1, the C3 lookup and candidate reclamation: T0, W1 steps (1) and (4), C3 precedence.
15. Continuous capture invalidation and repeated recovery re-evaluation against one deadline: CAPTURE and the budget
rule. New controls beyond the review's list: two closing lines written to the pipe, the first wins and the late writer
sees "transaction is closed" (`commit` then `abort` lands and exits non-zero; `abort` then `commit` lands nothing); a
client stopped while it holds no write end; the client lock released by a kill versus by W1 (the owner checks the mark
after the lock); `PREPARED_HOLD` against a late `commit`; power loss at each write of the request file, the state pair
and the order file; a request file whose checksum fails; the sync count on the waiting path (0) and the syncer's count
per pass; two registry listings with a run admitted between them; `merge-install` suppression for a plain commit, a
`commit -a` and a named-path commit between publication and install. Do not substitute these model conveniences (the
review's list, kept): `decision = commit` means the ref was published; `terminal at the final registry read` means
deletion preceded the connectivity check; `withdraw recorded` means restoration finished; `protected child exists`
means an inline owner may exit harmlessly; `terminal outcome exists` means every lookup can already discover it. Add:
`V` means durable; `request file linked` means its content reached the disk.

Models added from the twelfth review (revision 13; each starts as a failing control with the revision 13 changelog's
schedule; observations, never the completed logical operation). The review's twenty controls, keyed to the transition
that answers each:
1. Normal COMMITTING exit, original attempt fails, Q1 tries again; assert no fresh publication: W1 unresolved exit,
   the mark, Q1 and P1's pre.
2. `commit` written, client killed before it recorded `submitted`, duplicate owner closing line: P2b (the second
   closing line is not executed).
3. Git dies before reading the closing line, after reading it, and after writing the ref but before reporting: E1
   classes F, P and U.
4. Publication followed by a deliberate reset before the child exits, with and without reflog evidence: E1 class U and
   R3.
5. PREPARED-HOLD writes `abort` to a transaction process that never reads again: P2b (the enqueue is bounded, the lock
   release is not).
6. FIFO bootstrap, single-write atomicity, inherited descriptors, `EPIPE`, buffered commands before end of input: P2a
   pipe discipline.
7. Two alternating state writes without a durable checkpoint, both torn: the state log and its checkpoint rotation.
8. A valid later state or ledger record with its earlier dependency missing, and the syncer racing a checkpoint: hash
   chains and cross-file references.
9. A V terminal outcome, persistent reclamation and a lost ledger tail, then a same-identity resend: T1's Pre
   (reclamation barrier).
10. `conflicted`, `rewritten-away` and the unresolved exit at every durability boundary: the departure barrier's D-sets.
11. A staged-only `S` created after capture, the index replacement persisting without the saved file: MG3 and the
    preservation barrier.
12. Carry-forward power loss with the real filesystem ordering, not atomic "saved" booleans: the carry-forward
    protocol under rule V's exemption.
13. W1 returns before W2, R2 captures against the withdrawn logical base, replay after the removal: W1 step (4), W2
    step (2), C1's refusal.
14. A lagging merge install followed by new staging; `commit -a` and a plain commit with an edit to another line of an
    affected file: MG2 and MG3.
15. A native switch paused longer than the span, then completing: `attach-watch`, owner question 7 (control the stated
    residue).
16. A switch beginning after the install, testing what MG6c restores: the leftover test.
17. T1 recovery without its original `L0`; registry enumeration with a reservation between the probe and the listing;
    compaction with a predating release intent.
18. A new identical invocation versus a resend that keeps its nonce: C2 and C3.
19. Departure barriers consuming time after the bound; foreground progress while the syncer flushes: `WAIT_DEPART` and
    the syncer's separation from the owner's loop.
20. A terminal lookup and reclamation race inside W1's departure barrier, not only inside C3: W1 steps (1), (5) and
(6). Do not make these models pass by assuming: a successful pipe write means Git began the ref write; an ended child
and an absent tip prove no earlier publication; one of two unsynced state slots must survive; checksums enforce
dependency order; "background" means the foreground cannot wait indirectly; a finite recheck proves that a native
switch has finished; ignoring a withdrawn consumption entry also removes its effects from the capture base.

## Revision 14 changelog, moved from the design in revision 15

### Revision 14 changelog (for the step-1 models agent; every changed transition, with the model and the schedule)

Naming. New names: the receipt `lane/receipt/<id>`, the stop receipt and the saved-content set, empty mark files named
by kind, `WAIT_STOP`, `WAIT_MATERIALIZE`, the keeper write end, the durable prefix watermark, discharge of a live
dependency, the seal record and checkpoint cutover, the unresolved and plain forms of W1, C2's `bound` step with the
client lock before the link, the register entries STOP and MATERIALIZE, the exit audit. Removed: attempt number in the
mark name, the idle-writer rule, the per-file durable flag. Each schedule goes red under revision 13 and green under
revision 14; a model marked "new" is added to step 1; the models branch is not edited by this document.
- P2c, Client result, recovery R1 (R13-1; models new "success then owner death then reset", "attempt directory read by
  recovery"). Schedule: the client sees `commit: ok`, returns success, the owner dies before P3 and T0, another writer
  resets the branch away and the reflog is erased: red under 13 (R5 authorizes a fresh publication); green under 14
  (the receipt, or `commit: ok` in the attempt's `out`, is read by recovery as published).
- C2 (R13-1; models new "exposure before the client lock", "exit on each side of bound"). Schedule: the request is
  linked in `inbox/` before the client lock is held: red (the owner sees no client); green (the lock is taken first,
  `bound` is the link, and an exit before the link has no request).
- Rule V stop receipt, W1 (R13-2; models new "departure timeout loses only the mark", "same for the unresolved form").
  Schedule: the request and its adoption are D, W1 creates the mark, `WAIT_DEPART` expires before the mark is D, power
  loss: red under 13 (recovery sees a clientless live request and publishes, or makes an attempt after `a`); green
  (the stop receipt is two syncs, forced first within `WAIT_STOP`; the residue of a timeout is stated in question 6a).
- P2a pipe (R13-3; models new "no writer between instructions", "EAGAIN, SIGPIPE, failed closing write", "abort cannot
  be enqueued"). Schedule: the owner writes `prepare` and closes, Git prepares, the client has not yet opened a
  writer: red (end of input aborts the transaction); green (the owner's keeper stays open until the closing line has
  run or the child has ended). PREPARED-HOLD: red (the register said the lock is released); green (the register says
  the enqueue is attempted).
- Rule V dependency discharge (R13-4; models new "reclaimed request then restart", "old segment deleted then
  checkpoint predecessor", "valid checkpoint with an absent transitive dependency"). Schedule: R1 and R2 adopted in
  one chain, R1 retired and its request deleted, restart: red (R1's adopt cannot validate and R2 is dropped); green
  (the terminal outcome discharges R1's dependency).
- Rule V checkpoint cutover and watermark (R13-5; models new "append racing snapshot", "sync completes while another
  append happens"). Schedule: snapshot at g10, S1 appends g11 to the old segment, the old segment is removed: red;
  green (the owner seals the old segment, writes the checkpoint after the seal on the same thread, and the syncer
  claims only a captured prefix).
- Rule V saved-content set (R13-6; model new "named built commit with a parent absent from the request"). Schedule:
  another writer publishes `C`, B1 builds `D` on `C`, the exit names `D`, power loss or maintenance removes content
  only `C` held: red (the barrier forced the request and lane-written files only); green (the closure is forced and
  the side ref is read back, or the exit promises the captured intent only).
- C1 two-sample marks (R13-7; model new "mark appears between the samples"). Schedule: R2 reads g with no mark, R1's
  W1 creates the mark, the person stages `y`, R2 reads the index and accepts: red under 13; green (the second sample
  sees the mark and the capture is repeated). A late class F after the unresolved form: the outcome `withdrawn` and
  the same descendant re-basing.
- MG2 capture and MG3 steps (R13-8A to R13-8C; models new "plain staged commit of an old-based edit during the lag",
  "E names B, O names A, stat matches E", "S equals E, differs from O, W differs from both"). Schedule: `O: a=0 b=0`,
  `M: a=1`, the person edits and stages `b=1`, a plain commit: red (it reverses `a`); green (the staged edit is
  replayed on `M`). MG3's three steps replace the overlapping cases.
- MG6c identity (R13-8D; model new "reset --soft then a switch that keeps the staged content"). Schedule: red under 13
  (all three tests pass and the deliberate state is undone); green for every schedule that rewrites the index; the
  schedule that does not is the stated concession of question 7a.
- Client result, MG2, register MATERIALIZE (R13-9; model new "slow preservation after publication"). Schedule:
  publication, then MG3 with a preservation barrier: red (the exit waited with no bound); green (`WAIT_MATERIALIZE`,
  then exit 0 `published` with the install continuing under the service).
- W1 preconditions, C3, T0, P4, the register, the rationale texts (R13-10). Plain and unresolved forms have separate
  preconditions; C3 answers a `withdraw-unresolved` mark with "the original attempt may still complete"; T0 lists
  `unresolved`; P4's only retry precondition is class F; PREPARED-HOLD no longer guarantees release; the capture
  rationale and the R4 mention are corrected.
- Exit audit (the review's section 6; models new "planned service shutdown with a protected execution and a prepared
  transaction", "internal error after exposure"). New text.



## Answers to the thirteenth review, moved from the design in revision 15

## Answers to the thirteenth review

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-13.txt`, with a closure line: closed (a transition
answers the schedule; a model is owed), or owner question (closing it needs a contract the owner has not approved).
The verdict was: continued model work only. This revision builds nothing. The review did not inspect the cited Git
source lines; they stay gates in E1 and P2.

- R13-1 (P1), normal success has no no-replay boundary and the admission boundary is undefined: accepted. The client
  creates the receipt before it prints `published`, recovery reads the receipt and the attempt directory as evidence
  (R1), C2 takes the client lock before the link and defines `bound` as the link. Closed for process crashes; a lost
  receipt after a power loss is the weaker durability of decision 10 for a published exit, stated in P2c.
- R13-2 (P1), timed departure can resurrect a withdrawn request: accepted. The stop receipt is separated from the
  saved content, made D first and alone, within `WAIT_STOP` (2 syncs of a few bytes); the larger set is charged to
  `WAIT_DEPART`. A power loss inside the stop receipt's own window is the residue of question 6a, because no
  fail-closed rule is implementable (after a power loss a departed client and a killed one cannot be told apart).
- R13-3 (P2), the no-idle-writer rule permits end of input: accepted. The owner's keeper write end stays open for the
  whole attempt and its closing is a protocol action; `EAGAIN`, `SIGPIPE`, short writes and a failed enqueue are
  specified; the register no longer guarantees lock release. Closed as specification; backend gates remain.
- R13-4 (P1), dependency validation versus reclamation: accepted. Hash commitments are told apart from live
  dependencies, which a terminal outcome, a `disposed` record or a checkpoint discharges; validation order is fixed
  (ledgers first). Closed.
- R13-5 (P1), checkpoint rotation lacks a handoff: accepted. The owner seals, checkpoints on the same thread, switches
  appends, and the syncer claims only a captured prefix watermark before the old segment is removed. Closed.
- R13-6 (P1), D-sets do not preserve a named built commit: accepted. Captured intent, exact commit and closure are
  told apart; naming a commit requires its forced objects and a side ref read back, otherwise the exit promises the
  captured intent only. Closed.
- R13-7 (P1), C1's marks are not one coherent view: accepted. C1 samples the marks before the state read and again
  after the index read and refuses a plain mark in the second sample; the late class F of an unresolved form has its
  outcome and re-basing. Closed.
- R13-8 (P2), merge capture and cleanup: accepted. Plain staged capture replays the staged edit on `M`; the stat
  shortcut needs `E` equal to `O`; MG3 is three independent steps with mandatory preservation; the cleanup requires
  the index file unchanged since the install, which makes it fail safe but not provenance-exact. Questions 7a to 7c
  carry what remains, question 5 the composition.
- R13-9 (P2), successful return has incompatible rules: accepted. One contract: the command waits for the single
  materialization attempt under `WAIT_MATERIALIZE`, then exits 0 `published` with the install continuing under the
  service. Closed.
- R13-10 (P2/P3), remaining contradictions: accepted and edited (W1 forms, C3, T0, P4, PREPARED-HOLD, rationale texts).
- Exit audit and planned shutdown: added as the "Exit audit" paragraph of the state machine.
- Questions 5 to 7: the owner's answers are recorded (decisions 11 to 13); 6 and 7 are provisional and are asked again
  with corrected framing, 5 with the added options.
- Not a defect: none of the findings was rejected.



## Owner questions for revision 14, full text as drafted (6a and 6b were withdrawn), moved from the design in revision 15

## Owner questions for revision 14

Why these are asked again. The owner answered revision 13's questions 5 to 7 (decisions 11 to 13). The thirteenth
review showed that questions 6 and 7 were framed inaccurately, so decisions 12 and 13 are provisional, and that
question 5 lacked an option. This section asks them again, split where the review found separable decisions. Where a
mechanism meets the owner's contract it is in the state machine and not asked here: the stop disposition is now a tiny
separate receipt made durable first, so "nothing lands later" no longer waits for the larger flush; the cleanup now
also requires that the index file is unchanged since the install. Each question states its risk plainly. The
recommended option is marked and the ranking gives a reason for each adjacent pair. None is decided here.

### Question 5, asked again: when what you staged is not what is in the file

Plain context. You chose "Merge anyway, unstage x" and then "Carry the file, save staged". The review found an option
missing and an unsupported claim in the old text. Example: `x.txt` is `A` at the branch tip; you stage a change so the
index holds `B`; you then edit the file to `D`; the merge makes `x.txt` `C`. "Stop" can only apply before the branch
moves; a path that you stage after the branch has moved is first seen too late to stop and is left alone as later
staging.
- Option A, carry the file you see; save the staged-only version aside (Recommended; your earlier answer). Visible
  here: `x.txt` becomes the merge of your change `A` to `D` onto `C` (with the usual markers if they collide), the
  index holds `C`, and a line says your staged `B` is saved in `saved/x.txt.staged`. Pros: the file stays readable;
  nothing is lost. Cons: a staged change you meant to keep is not in the file, so it must be taken from the saved file
  by hand.
- Option B, carry the staged version onto the result and save your file aside. Visible here: `x.txt` becomes the merge
  of `B` onto `C` (markers where they collide), the index holds `C`, and your later file `D` is saved in
  `saved/x.txt.worktree`. Pros: it is the most literal reading of "your staged x becomes an unstaged edit". Cons: the
  version you last worked on is the one that is not in the file.
- Option C, show all of it as an explicit conflict. Visible here: `x.txt` holds a conflict in the three-version form
  (your staged `B`, your file `D`, the merge `C`), the index holds `C`, and nothing is saved aside. Pros: nothing is
  hidden or archived. Cons: it needs a defined format, it can be long, and it asks you to resolve what you did not ask
  to resolve.
- Option D, stop before the branch moves. Visible here: when the merge is computed, if a path you changed has a staged
  version that differs from the file, the command exits non-zero naming the path and changes nothing; you commit or
  unstage and run it again. Pros: nothing is decided for you. Cons: it is not "merge anyway", and it cannot apply to
  staging made after the branch moved.
- Ranking: A > B > C > D. A beats B because it keeps the version you last worked on visible; B beats C because B
  leaves one readable file where C leaves a three-way conflict; C beats D because C completes the merge as you asked
  and D returns the work to you.

### Question 6a: forcing the stop mark to disk when the disk is slow

Plain context. When a command stops waiting, the wrapper records a tiny mark that says this commit must never be
published later. Revision 14 makes that mark the first thing forced to disk (two forced writes, a few bytes). Your
decision "Force at leaving" and your rule that commands return at their limit meet here. Example: you press Ctrl-C
while the disk is saturated and forcing the mark would take 30 s.
- Option A, wait up to a short named limit, then leave and say the stop is not confirmed on disk (Recommended).
  Visible here: after the limit the command exits non-zero: "stopped; the stop mark is not yet confirmed on disk".
  Risk, stated plainly: if the power fails inside that window, the mark can be lost, and after the reboot the wrapper
  can finish the commit (or, if the stop came after a submitted `commit`, make a further attempt) although the command
  reported an error. Pros: the command always returns, and in every ordinary case the mark is on disk in moments.
  Cons: that power-cut window.
- Option B, wait for the mark without a limit. Visible here: the command stays until the disk answers, 30 s here and
  longer on a worse disk. Pros: "nothing lands later" is exact even across a power cut. Cons: it breaks "nothing
  should ever wait forever or indefinitely" for a saturated disk.
- Option C, do not force the mark at leaving; the background forces it. Visible here: the command leaves at once.
  Pros: nothing is waited for. Cons: it gives up the power-cut protection of the stop, which your decision "Force at
  leaving" asked for.
- Ranking: A > B > C. A beats B because it always returns and states the one window honestly; B beats C because B
  keeps the promise exact, while C drops the forcing that you chose.

### Question 6b: forcing the saved copy to disk when the disk is slow

Plain context. After the stop mark, the command forces the saved copy: the request, the log, and a built commit's
files with its side ref. Two examples. First, a commit waits 600 s behind a held branch lock and is withdrawn; its
built commit is saved on `refs/cli-git/<instance>/withdrawn/<id>`, and forcing it takes 40 s. Second, a commit is held
by your `pre-commit` hook run by the service and is withdrawn; no commit exists yet, so only the request inputs are
saved and the message names them, not a side ref.
- Option A, wait up to a named limit, then leave and say the saved copy is not yet confirmed on disk (Recommended).
  Visible here: the stop mark is confirmed; the message says "the commit is kept; it is not yet confirmed on disk and
  the wrapper keeps forcing it". Risk: a power cut in the window can lose the saved copy. Your staged changes and
  files are untouched, because nothing was written before the branch moved. Pros: always returns; the stop promise is
  already exact. Cons: that window for the copy.
- Option B, wait without a limit. Visible here: the command stays 40 s or more past its limit. Pros: the promise made
  at leaving is exact. Cons: breaks "nothing should ever wait forever or indefinitely".
- Option C, background only. Visible here: the command leaves at its limit. Pros: no wait. Cons: it undoes "Force at
  leaving, else background" for saved work.
- Ranking: A > B > C. A beats B because it returns and gives the true state of the copy; B beats C because B keeps
  your decision and C drops it.

### Question 7a: may the late-switch cleanup act on a by-value test?

Plain context. After a merge writes files, a `git switch` that was paused can finish and leave the merge's files in
your new branch. The wrapper can restore the old files, but it cannot prove the state is a leftover and not something
you did on purpose; it can only test values (the index file is unchanged since the wrapper wrote it, the entry and
file still equal the merge's, and the new branch's version differs). Example: you merge; then you run
`git reset --soft` to an earlier commit and switch to a branch whose version of the file is the same as that commit's,
keeping the merge's content staged on purpose without Git rewriting the index. Every value test passes, and the
cleanup would restore the old file and entry, undoing your deliberate staged change (the merge's content stays in the
merge commit).
- Option A, yes, with the index-unchanged test and the span of question 7b (Recommended). Visible here: in the rare
  example above your staged change is undone and a line says what was restored from which saved copy; in the ordinary
  late switch the leftover is cleaned. If any command wrote the index between the merge and the late switch (a
  `git add`, a refresh), the wrapper does not clean up, and the line names the leftover instead. Pros: the leftover
  you accepted is cleaned automatically. Cons: a deliberate sequence like the example can be mistaken for a leftover
  inside the span.
- Option B, yes, but ask first: the next wrapper command in that worktree prints the leftover and the exact command
  that restores it, and restores nothing by itself. Visible here: nothing is undone; the line stays until you act or
  dismiss it. Pros: no deliberate change is ever undone. Cons: the leftover stays until you act.
- Option C, no cleanup at all; the leftover is reported once. Visible here: one line naming the files and how to undo
  them by hand. Pros: simplest. Cons: the rare leftover is always yours to fix.
- Ranking: A > B > C. A beats B because it restores the leftover you accepted without a step from you, at a small
  stated risk; B beats C because B keeps the restore one command away.

### Question 7b: how long may that cleanup authority last?

Plain context. Only matters if you answered 7a with A. Example: a late switch finishes 5 s after the merge, or 5
minutes.
- Option A, a measured span after the merge, then stop (Recommended). Visible here: a switch that finishes inside the
  span is cleaned; after it the files stay. Pros: bounds the chance of mistaking your own later action for a leftover.
  Cons: a slower switch leaves the files.
- Option B, until the next wrapper command in that worktree finds the branch still the merge's, then stop. Visible
  here: the first command after the merge decides. Pros: no clock. Cons: a switch that finishes after that command is
  not cleaned, and a long idle gap lets the window last as long as you are away.
- Option C, until the files change. Visible here: cleaned whenever the leftover is next seen, however late. Pros: no
  span to outlast. Cons: days later it can mistake your own state for a leftover, which is the risk of 7a made
  unbounded.
- Ranking: A > B > C. A beats B because its window is bounded in time and not by when you happen to run a command; B
  beats C because C has no end at all.

### Question 7c: after the watch ends, is a permanent leftover accepted?

Plain context. A paused switch has no finite completion time, so after the watch ends a late switch can still leave
the merge's files in your branch. Example: the switch finishes an hour after the merge.
- Option A, accept it and print one line at the merge naming how to undo it by hand. Visible here: nothing happens at
  the hour mark. Pros: simple; no standing obligation. Cons: you may never notice the files.
- Option B, keep a reminder (Recommended): the wrapper keeps a small record of the merge's saved copies and its next
  command in that worktree prints a line when the branch no longer names the merge's branch and the files match, until
  you dismiss it. Visible here: the next command after the hour prints the line and the restore command. Pros: you are
  told; nothing is undone. Cons: a standing record and a line you must dismiss.
- Ranking: B > A. B beats A because it tells you about the leftover without undoing anything, at the price of a record
  to keep.



## Models added from the thirteenth review, moved from the design in revision 15

Models added from the thirteenth review (revision 14; each starts as a failing control with the revision 14
changelog's schedule; observations, never the completed logical operation). The review's twenty-two controls, keyed to
the transition that answers each:
1. Successful normal exit before P3 and T0, owner death, deliberate reset and erased reflog: assert no fresh attempt:
   P2c receipt, R1.
2. C2 exposure before client-lock acquisition: C2 step (2) before (3).
3. Define `bound`; inject an exit on both sides of the link: C2 step (3).
4. Departure timeout loses only the mark or the outcome while the request stays D: the stop receipt and `WAIT_STOP`.
5. The same schedule for the unresolved form; assert no attempt after the one in flight: the stop receipt and the
   unresolved mark.
6. No FIFO writer between the initialization lines and between `prepare: ok` and the client's open: the keeper.
7. `EAGAIN`, `SIGPIPE`, failed closing writes and an `abort` that cannot be enqueued: the pipe discipline.
8. Request reclamation followed by validation of its historical adoption dependency: discharge by the terminal outcome.
9. Old-segment deletion followed by validation of a checkpoint's predecessor reference: the checkpoint is
   self-contained.
10. S1 append racing checkpoint snapshot, seal, publication and old-segment deletion: the cutover.
11. A sync completing while another append occurs; only the captured prefix becomes D: the watermark.
12. A syntactically valid checkpoint whose transitive dependency is absent: validation order and discharge.
13. C1 observes no mark, W1 returns without W2, later staging appears, C1 accepts an unchanged `g`: the two-sample
    marks.
14. Late positive F after an unresolved exit, with every descendant transition: W1's unresolved form and W2.
15. Plain staged commit of an old-based edit during the merge-install lag: MG2's plain-commit rule.
16. `E` names `B`, `O` names `A`, and `W`'s stat data matches `E`: MG2's stat condition.
17. MG3's overlapping `S = E != O`, `W != S`, `W != O` cases: the three independent steps.
18. `reset --soft` followed by a switch that carries staged `M` while `R` keeps its inode: MG6c (the stated
    concession, question 7a).
19. Successful publication followed by slow MG3 preservation; check the actual command-exit deadline:
    `WAIT_MATERIALIZE`.
20. Planned service shutdown with a protected execution and a prepared transaction: the exit audit.
21. A saved built commit whose replay parent holds content absent from the immutable request: the saved-content set.
22. Queries and resends during `withdraw-unresolved`; never report unconditional "nothing can land": C3. Do not make
these pass using atomic abstractions for "checkpoint", "durable file", "client attached", "withdrawn view" or
"leftover". Each needs the observations and ordered effects the implementation will have. Models must not label
questions 5 to 7 green by adopting the recommended answers.


## Fail-closed recovery design after a reboot (revision 15 draft; not needed under decision 14)

Boot gate and recovery policy (revision 15; R14-1, R14-2; owner question 6a). Within one boot every V record survives the death of any process, so the stop marks,
receipts, outcomes and attempt evidence that recovery reads are the true state, and the outside-cause rule (a request with no client continues) is safe. After a reboot,
a power loss or a kernel crash that is no longer so: any V record may be missing, and no recovery can tell a client that left normally with its stop mark lost from a client
that was killed. The boot gate acts on that difference. The lane records the kernel's boot identity (`/proc/sys/kernel/random/boot_id`) in every checkpoint and in the first
record of every incarnation. At L4, when the recorded identity differs from the current one:
- Policy confirm after reboot (fail closed). Every request with `origin = client` that is not terminal and has no receipt moves, in one S1 replacement, to the state
  `quarantined`: its request file, spool and saved items are kept, and nothing is attempted, published or dispatched for it, however its other records survived. A request
  with `origin = service` (a reconcile merge, a push) is unaffected, because no client departure can have lost its stop. The next wrapper command in that worktree prints one
  line per quarantined request (its message's first line and time, that it was not published, and the two commands the line names: one authorizes it, one drops it). The
  authorization is an empty file `lane/decision/<id>.resume`, forced, after which the request continues from its resume point as a request with no client; the recorded tip
  decides first whether it already landed (R2), so a commit that did land is not published again. A kill within the same boot (the OOM killer, a person's kill) still
  continues automatically. Cost, stated plainly: client-originated work that a power loss interrupted is not finished by itself; the person's staged changes and files are
  untouched (publish-first), so they run the commit again or confirm; some work of a client that was killed by the power loss also waits for confirmation. The gate adds no
  sync to any waiting path, and under this policy the stop disposition needs no foreground forcing.
- Policy continue automatically (the behavior of revisions 12 to 14). Nothing is quarantined. The stop disposition is forced within `WAIT_STOP` (rule V), and the residue is
  stated in owner question 6a. Owner decision 3 ("Land it") then applies after a power loss to a request whose client already received success, when its receipt is lost and a
  deliberate reset plus an erased reflog hide the publication: recovery lands the commit with a notice. That permission is decision 3's, which covers a publication that
  cannot be told from a deliberate removal; it is not read from decision 10, which only chose when to force data.
The machine implements both settings, and the default until the owner answers question 6a is the first.

## Revision 15 changelog, moved from the design in revision 16

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



## Answers to the fourteenth review, moved from the design in revision 16

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



## Owner questions for revision 15, full text as drafted (answered as decisions 15 to 18), moved from the design in revision 16

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



## Models added from the fourteenth review, moved from the design in revision 16

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

## Revision 16 changelog, moved from the design in revision 17

### Revision 16 changelog (for the step-1 models agent; every changed transition, with the model and the schedule)

Naming. New names: `WAIT_VERIFY`, `INLINE_PATHS`, `INLINE_BYTES`, the terminating class of inline materialization, the
eligible checkpoint and its closure, the outcome `lost`, the success receipt forced in the syncer's first pass, the
deferred selection with its copied content and its re-keying, the reminder record, owner question 8. Removed: the
checkpoint rule "drop one request, never the checkpoint", the ordering `syncfs` before the side ref, the start-only
`WAIT_MATERIALIZE` for any work, the "no closing line buffered" drain condition. Schedules below go red under revision
15 and green under revision 16 as far as the owner's answers and the mechanisms allow; a model marked "new" is added
to step 1; the models branch is not edited by this document.
- Owner decisions 15 to 18 (questions 5, 7a, 7b, 7c answered): MG3 composition, MG6c's four tests and window, the
  reminder record. Models: the window starts at MG3's installation; expiry admits no new cleanup and an already
  started one finishes; dismissal does not dispose of saved copies.
- Receipt forcing, R5 after power loss (R15-1; models new "success, receipt lost inside the lag, reset, erased
  reflog", "ref write lost, replay restores"). The exception is reduced to the syncer's lag window and asked as
  question 8, because no mechanism within the owner's rules removes it (the receipt forced before success would put
  two syncs on every commit).
- Checkpoint closure, eligibility, `lost`, disposal order (R15-2; models new "checkpoint and list D, ledger suffix not
  D, predecessor deleted", "incomplete new root", "disposal line V then deletion").
- Level 2 order and eligibility (R15-3; models new "side ref created after the flush", "alternate on another file
  system", "repack during the verification"). Schedule: the ref was created after `syncfs` and lost: red; green (the
  ref is created first and the flush covers it; one file system only; `WAIT_VERIFY` bounds the optional steps).
- Submission accounting (R15-4; model new "write succeeded, client died before the record"). P2b and E1 use the full
  write; `closed-nosubmit` makes "not submitted" final for the whole attempt, binding every decider.
- Deferred selection with copied content and re-keying (R15-5; models new "ancestor disposed after add-back",
  "multiple pending bases").
- Terminating inline class, handoff otherwise (R15-6; models new "begun fallback hangs", "verification worker hangs").
  Schedule: the ineligible-index fallback hangs after the start deadline: red (the foreground cannot leave); green (a
  non-terminating materialization is never inline).
- Draining shutdown configuration (R15-7; models new "hook outlasts the stop timeout", "buffered closing line"). The
  unit's stop timeout is infinity and the upgrade path is a drain request.
- Wording (R15-8): decision 14 and disk faults, upper bound of a command, level-1 qualification in rule B, MG6c
  detection wording, accounting case.
- Questions: 5, 7a, 7b, 7c asked and answered; question 8 open.



## Answers to the fifteenth review, moved from the design in revision 17

## Answers to the fifteenth review

Each finding of `~/temp/agent/reviews/commits-never-fail-sol-15.txt`, with a closure line: closed (a transition
answers the schedule; a model is owed), or owner question. The verdict was: continued model work only; the questions
were accurate after named edits, which the coordinator applied before asking them. This revision builds nothing. The
review did not inspect the cited Git source lines.

- R15-1 (P2), the post-success replay exception is attributed but not shown approved: accepted. The window is narrowed
  by a mechanism (the success receipt is forced in the syncer's first pass), and the remainder is asked as question 8.
  Open as an owner question.
- R15-2 (P1), the checkpoint root is incomplete and a dependency failure has two meanings: accepted. The closure
  includes the ledger and order prefixes; an ineligible checkpoint falls back to the previous root instead of dropping
  one request; the one loss transition is `lost`; disposal is forced before deletion. Closed.
- R15-3 (P1), level 2 does not make the side ref durable: accepted. The ref is created before the flush, one file
  system is required, the optional steps are bounded by `WAIT_VERIFY`, and level 1 is the fallback. Closed.
- R15-4 (P2), submission order is contradicted: accepted and edited; `not-submitted a` is final only as the client's
  last act. Closed.
- R15-5 (P1), deferred selections lack retained content: accepted. The content is copied into the descendant's own
  immutable bytes; re-keying keeps a selection across multiple bases. Closed.
- R15-6 (P1), start-only materialization creates an unbounded foreground phase: accepted. A mechanism within the
  owner's rules: only a terminating, size-limited, child-free class runs inline; everything else is handed to the
  service before any mutation. Closed.
- R15-7 (P1/P2), shutdown must implement draining: accepted. The unit's stop timeout is infinity, the upgrade path is
  a drain, and the wrapper never invokes a finite forced stop; the buffered-line condition is removed. Closed as a
  configuration obligation.
- R15-8 (P2/P3), blanket claims: accepted and edited.
- The questions: owner answers recorded as decisions 15 to 18; question 8 open.
- Not a defect: none of the findings was rejected.



## Owner questions for revision 16, full text as drafted, moved from the design in revision 17

## Owner questions for revision 16 (open)

Questions 5, 7a, 7b and 7c of revision 15 were asked and answered (decisions 15 to 18); their full text as drafted is
in the history record. One question remains, and it exists only because no mechanism removes the risk. It is also
written to a separate file named `commits-never-fail-questions-` followed by the abbreviated hash of the commit that
holds this text.

### Question 8: an old success receipt and a replay after a power loss

Plain context. When a commit succeeds, the wrapper writes a tiny receipt that says "this request already published".
If the power fails shortly afterwards, the receipt can be lost before it reaches the disk. On restart the wrapper
decides from the branch: if the branch still holds the commit, nothing happens; if the branch lost it (the branch
write itself did not reach the disk), the wrapper lands it again, which restores the commit you were told succeeded.
The one bad case needs all of these together: the power fails before the receipt reaches the disk, you had
deliberately removed the commit from the branch in that same span (for example `git reset --hard HEAD~1`), and the log
that records that removal was also erased. Then the wrapper cannot tell your removal from a lost write, and it lands
the commit again with a notice that names it and the command to drop it (your earlier decision 3, "Land it", covers
exactly an indistinguishable publication, but it was not asked about a command that had already returned success). How
long the span is depends on whether a background service is running. With the service running it forces receipts
within a few seconds (the syncer's lag, a named constant that is not yet measured). With no service running (a plain
command that started no service), nothing forces the receipt until the next wrapper command in that worktree, which
has no time limit. Example: you run `git commit`, it prints success, you reset the branch two seconds later, the power
fails one second after that.
- Option A, accept both windows; land it again with the notice. Visible here: after the reboot the commit reappears on
  the branch with a line saying so and how to drop it. Risk: in the conjunction above a commit you removed on purpose
  comes back once; with no service running the window has no time limit. Pros: no extra wait on any commit.
- Option B, force the receipt to disk before the command returns success only when no service is running
  (Recommended). Visible here: with the service the ordinary path is unchanged; with no service the command waits for
  two forced writes (the receipt file and its directory) and the unbounded window does not exist; the few-second
  window of the service path remains. Cost: that wait on plain commands made with no service, which can take seconds
  under disk pressure, against your rule that commits should not take 5 s longer than native Git. Pros: the longest
  window is closed on the only path where nothing else would ever close it, and the common path keeps zero forced
  writes.
- Option C, force the receipt before success returns on every commit. Visible here: no window at all. Cost: two forced
  writes wait on every successful commit, which can take seconds under pressure.
- Option D, in this indistinguishable case do not land it; keep it and ask. Visible here: after the reboot a line says
  a commit may or may not have been published, and the wrapper waits for you to confirm or drop it. Risk: a commit
  whose write was really lost needs your confirmation to come back, which changes decision 3 for after-power-loss
  cases.
- Ranking: B > A > D > C. B beats A because it closes the unbounded window at a cost only where no background process
  exists to do it; A beats D because A restores a lost write without a step from you and the bad case needs a
  conjunction of rare events; D beats C because D costs nothing on the success path, while C puts forced writes on
  every commit.



## Models added from the fifteenth review, moved from the design in revision 17

Models added from the fifteenth review (revision 16; each starts as a failing control with the revision 16 changelog's
schedule; observations, never the completed logical operation). The review's eighteen controls, keyed to the
transition that answers each:
1. Checkpoint log closure: checkpoint and external list D; referenced ledger or order suffix not D; predecessor
   deleted: the checkpoint closure.
2. Incomplete new root: a checkpoint survives before one listed dependency; the predecessor remains; root selection,
   not selective request deletion: eligibility and fallback.
3. Disposed content: an older checkpoint references R; the disposal line V; R removed; power loss loses the disposal
   authority: the disposal order.
4. Exact saved ref: objects flushed, the side ref created afterwards and not forced; reboot, then a prune before the
   ref is rebuilt: level 2 ordering.
5. Multiple object locations: the exact closure includes an alternate on another file system: level 2 eligibility.
6. Representation replacement: repack or maintenance replaces object representations during the level-2 interval: the
   registry comparison.
7. Submission crash window: the full `commit` write succeeds, the client dies before recording; an earlier non-final
   `not-submitted` does not establish F: P2a and E1.
8. Derived selection retention: an ancestor's removal activates the deferred content; the ancestor is disposed; the
   descendant crashes and recovers; a prune follows: the copied content.
9. Multiple-base removal: a deferred selection temporarily becomes a no-op, then another supporting base disappears:
   re-keying.
10. Begun materialization hang: an ineligible fallback hangs after the start deadline; foreground interrupts stay
    available: the terminating class.
11. Departure verification hang: the final flush completes, a connectivity or registry check does not; do not call it
    a disk error: `WAIT_VERIFY`.
12. Real shutdown configuration: a protected hook outlasts the normal manager stop timeout and an upgrade or restart
    request: the drain configuration.
13. Userspace-buffered closing line: the FIFO appears empty after Git's read-ahead; the drain does not infer no
    submission.
14. Expiry during cleanup: the lock is acquired before expiry; preservation or scheduling carries the first exchange,
    or a later path, past expiry: decision 17's boundary.
15. Fingerprint restoration: the index changes away from and back to the recorded bytes before the check.
16. Question 5 scan gap: staging between the refusal scan and P1's sample; worktree-only divergence after the scan:
    decision 15.
17. Question dependency matrix: every combination of decisions 16 to 18 yields consistent notification, restore
    command and retention.
18. Dismissal versus disposal: dismiss notification, reboot, then restore; the sole saved copies remain. Do not make
these pass using atomic abstractions for "save", "checkpoint", "handoff" or "cleanup". Models must not label question
8 green by adopting the recommended answer.



## Evidence for the 120 s commit, moved from the design in revision 17

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


## (moved in revision 17) Item 1: a Git step of the commit crosses its class bound

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



## (moved in revision 17) Item 2: the service does not start within `START_WAIT`

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



## (moved in revision 17) Item 3: the service does not answer within `COMMIT_WAIT`

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



## (moved in revision 17) Item 5: `index.lock` is refused after the service's wait

Revision 12 reading: what follows is derived materialization after a published commit (M1, MG3). It is not a commit
landing later; it is a wait of the command only for the part rule B lists as MATERIALIZE (the terminating class,
started within its deadline), and the rest continues under the service.

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



## (moved in revision 17) Item 6: `ContentPruned`

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

## Revision 17 changelog, moved from the design in revision 18

### Revision 17 changelog (for the step-1 models agent; every changed transition, with the model and the schedule)

Naming. New names: `closed-nosubmit` (the attempt-wide final file), the receipt flusher and its three cases,
`WAIT_PREPARE`, `WAIT_DRAIN`, `INLINE_ENTRIES`, the installation anchor, admission under `index.lock` before the
marker, the replacement-root authority of a checkpoint with its non-drop obligation, the damaged established root, the
owner-only repair of a damaged final, `departure(disposition)`, `StatusUnknown` for a failed initial read, the
outside-fault exit. Removed: the receipt window "a few seconds" as a bound, the `lost` transition inside recovery,
option B of question 8 of revision 16, the claim that a client's never writing again makes `not-submitted` final.
Schedules below go red under revision 16 and green under revision 17 as far as the owner's answers and the mechanisms
allow; a model marked "new" is added to step 1; the models branch is not edited by this document.
- Level 2 (R16a-1; models new "ref store on another file system", "repack between the flush and the check"). The
  eligibility test includes the ref storage root; the registry observation is taken before the side ref is created and
  again after the check; wrapper-started maintenance forces its replacement representations; the optional helper holds
  nothing and is abandoned at `WAIT_VERIFY`.
- Materialization with no manager (R16a-2; model new "next command's fallback hangs"). `WAIT_PREPARE` is an overall
  deadline for the preparation on a private copy; the real mutation is entered only when its remaining steps are of
  the class; the class includes `INLINE_ENTRIES` and the current files.
- Drain request (R16a-3; model new "hook outlasts the upgrade caller"). The caller waits `WAIT_DRAIN` and returns
  "draining, upgrade not applied".
- `closed-nosubmit` (R16a-4; model new "owner takes over after a client's final record"). Every decider checks it
  before writing `commit`.
- Support rule for deferred selections (R16a-5; model new "another pending owner with a different value"). A selection
  is re-keyed only to an equal entry and otherwise added to the delta.
- Damaged established root and runtime `lost` (R16a-6; model new "established root loses a listed file"). Recovery
  stops and reports; `lost` is a runtime transition.
- Receipt responsibility and the flusher (R16a-7, R16a-10; models new "service shuts down with receipts outstanding",
  "no service and no manager"). The opening no longer calls the post-success exception approved.
- Window anchor, admission, reminder dismissal (R16a-8; models new "resumed installation restarts the window", "lock
  taken before expiry, marker set after").
- Wording (R16a-9): the outside-fault qualification of "no new attempt after any normal exit", the materialization
  budget wording, the accounting of eight syncs, Item 5.
- Components: replacement-root authority with validated content identity and durable ancestry (2.1, 2.2); candidate
  activation by recovery's validating selection (2.3); `departure(disposition)` and bound acknowledgments (2.4, 3.4);
  owner-only, serialized repair that keeps unsupported versions (2.5, 3.3); the maximum-generation conflict rule
  (3.1); `StatusUnknown` (3.2); selection serialized with reclamation (3.5); error-path attempt of the saved set after
  a stop failure (3.7).
- Models 6a and 6b: dispatch versus execution, the three completions, failed disposition creation as an outside-fault
  exit (6a 1 to 3); planned exit never causes parent-death termination (6a 4); checkpoint non-drop obligation (6b).
- Question 8 rewritten with R16a-10's corrections; it is asked only after the review of this revision checks it.



## Answers to the sixteenth review, moved from the design in revision 18

## Answers to the sixteenth review (the diff-only part 16a, the component and model reviews)

Closure words: closed (a transition answers the schedule; a model is owed), narrowed, or open. This revision builds
nothing. Part 16b of the whole-design review had not arrived when this was written; its findings are folded in when it
does.

- R16a-1 (P1), level 2: closed. Eligibility covers the ref storage root; the registry observation brackets the flush;
  wrapper-started maintenance forces its replacements; the helper is abandoned.
- R16a-2 (P1), no-manager materialization: closed. `WAIT_PREPARE`, class-only real mutation, `INLINE_ENTRIES`.
- R16a-3 (P1), drain: closed. `WAIT_DRAIN` for the caller; the service's drain is unbounded by design. The installed
  unit and the callers remain verification gates.
- R16a-4 (P1), `not-submitted`: closed by the attempt-wide `closed-nosubmit` that every decider checks.
- R16a-5 (P1), re-keying: closed by the support rule (equal content and mode, else add).
- R16a-6 (P2), `lost`: closed. Recovery stops on a damaged established root; `lost` is a runtime transition.
- R16a-7 (P2), receipt window: narrowed. The flusher removes the unbounded no-service exposure and a shutdown forces
  outstanding receipts; the completion time remains the disk's; the approval claim in the opening is removed. Open as
  question 8.
- R16a-8 (P2), contradictory watch and reminder text: closed. One rule, one anchor, one dismissal rule.
- R16a-9 (P2/P3), blanket claims: closed by edits.
- R16a-10 (P2), question 8: rewritten; it is asked only after the review of revision 17 checks it.
- Components 2.1 to 2.3 (P1), checkpoint authority: narrowed in the design (replacement-root authority, durable
  ancestry, activation by validating selection); the component is the open part.
- Components 2.4 (P1), departure: closed in the rule (`departure(disposition)`, bound acknowledgments); open in the
  component.
- Components 2.5, 3.3 (P1, P2), quarantine: closed in the rule (owner-only repair, inspected inode, preserved archive,
  unsupported versions kept, identity validated); open in the component.
- Components 3.1, 3.2, 3.4, 3.5, 3.7: closed in the rule (maximum-generation conflict, `StatusUnknown`, anchor on
  every chain, selection serialized with reclamation, error-path attempt of the saved set). 3.6 (cost of history
  scans) is a component task: the owner keeps validated state in memory and a lookup structure; open.
- Models 6a (P1, P2): the register's unchecked route and the failed disposition creation are model defects whose rules
  are now explicit (three completions; outside-fault exit); planned shutdown cannot cause parent-death termination
  (rule); the executor-assumption and pipe-composition items remain model work. Models 6b (P1): the checkpoint
  non-drop obligation is a rule now; the model must encode it.
- Part 16b: pending.
- Not a defect: none was rejected.



## Owner question 8 as asked in revision 17, moved from the design in revision 18

### Question 8: a success receipt that did not reach the disk, and a replay after a power loss

Plain context. When a commit succeeds, the wrapper writes a tiny receipt that says "this request already published",
and a background flusher forces it to disk. If the power fails before that, the receipt can be lost. On restart the
wrapper decides from every piece of evidence that survived: the receipt, the recorded `published` state, the
`commit: ok` line of the attempt, and what the branch and its reflog show. Replay is considered only when none of them
survives. If the branch still holds the commit, nothing happens. If the branch lost the commit because its own write
did not reach the disk, the wrapper makes a fresh attempt to publish the request, which is what restores a commit you
were told succeeded. The bad case is the same recovery state arising from a removal you made on purpose: you reset the
branch after the success, the reset reached the disk, but the receipt and the reflog record of the reset did not (the
same power failure can lose both; reflogs can also be disabled, expired or erased). The wrapper cannot tell your
removal from a lost write and makes the fresh attempt. Your earlier decision 3, "Land it", covers an indistinguishable
publication; it was not asked about a command that had already returned success. The fresh attempt is another attempt,
not a guaranteed copy of the original: it may build a different commit (the base may have moved), it may stop on a
replay conflict, a hook refusal or a signing failure, and it ends with a notice that names the commit and the command
to drop it. Example: you run `git commit`, it prints success, you run `git reset --hard HEAD~1` two seconds later, and
the power fails one second after that; the reset reached the disk, its reflog record and the receipt did not. What the
design already does about the window. The receipt is queued first for forcing. With a service running the service's
flusher forces it; with no service the command that returned success starts a detached one-shot flusher for it
(through the service manager, else as a detached child), and only if neither can be started (a process-count cap or
similar, which is outside our control) does the receipt stay unforced until the next wrapper command in that worktree
forces it first. A normal service shutdown forces every outstanding receipt before it exits, and the next start forces
any that remain before it does anything else. So the unbounded exposure of "no service running" is removed by
mechanism. What remains is the time the flusher needs, which the disk sets and which has not been measured (a
scheduling target exists, not a completion bound), and the ambiguity itself.
- Option A, accept the remaining window; make the fresh attempt with a notice (Recommended). Visible here: after the
  reboot the next wrapper command in that worktree prints the retained notice that a fresh attempt was made (the
  notice is stored, so it is delivered even if recovery ran in a service with no terminal), naming the commit and how
  to drop it. Risk: in the bad case a commit you removed on purpose is published again; it is not promised to happen
  only once (another crash before the new attempt's own records are on disk can repeat the ambiguity). No foreground
  receipt barrier is added to any commit; the flusher works in the background, and an inline materialization may still
  force its own preservation copies. If no flusher can be started, the window lasts until the next wrapper command in
  that worktree.
- Option B, force the receipt to disk before the command returns success, on every commit. This extends your "Force at
  leaving, else background" to a barrier before a success return. Visible here: two forced writes (the receipt file
  and its directory) wait on every successful commit, which can push a commit past native Git plus five seconds when
  the disk is busy; disk pressure is not treated as an exemption, so this would relax your rule "commits vs native git
  should not take 5s longer." in those cases. Scope: it closes this post-return replay ambiguity once the forcing
  succeeds. It does not make the branch contents durable (that follows your Git `core.fsync` setting): if the receipt
  is on disk and the branch write is lost, recovery sees "already published" and does not restore the commit you were
  told succeeded. It does not remove outside I/O failures, and if forcing fails after the publication the command
  cannot undo the publication; it reports the success and says the receipt is not durable.
- Option C, in this indistinguishable case do not publish again; keep the request and ask you. Visible here: after the
  reboot a line says a commit may or may not have been published, and the wrapper waits for you to confirm or drop it.
  Risk: a commit whose write was really lost needs your confirmation to come back, which changes decision 3 for
  after-power-loss cases.
- Ranking: A > C > B. A beats C because it restores a lost write without a step from you and the bad case needs a
  power failure inside the flusher's time plus a deliberate removal that persisted without its evidence; C beats B
  because C adds nothing to the success path, while B puts forced writes on every commit and, once the receipt is
  forced, also stops the automatic restoration of a lost branch write.


## Rationale sections moved from the design in revision 18

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
