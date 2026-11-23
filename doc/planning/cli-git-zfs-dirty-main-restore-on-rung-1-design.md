# The dirty-main restore on rung 1 and its frozen proof

Non-final,
unapproved design proposal by an agent.
It is not approved by the project owner and nothing in it is final.
It designs one open item of the accepted ZFS worktree design,
"The dirty-main restore on rung 1",
and it decides nothing the design,
the project's answers or the plan's own records did not decide,
beyond the writer's choices that "Choices this design marks for the project's veto" lists for the project to settle.
It edited no other file.

This revision answers the gate review at `~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/review.md` (tip `1f24dcc10d` of this document) and its probe logs beside it.
"Answers to the gate review" records,
finding by finding,
what changed and where.

This revision also answers the second gate pass (`~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/review-second-pass.md`,
tip `c342dd1a06`,
with `probe-logs-second-pass.md` beside it) and the project's answers of 2026-11-22 to three of its four questions (`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`).
"Answers to the second gate pass" records,
finding by finding,
what changed and where,
and "The project's answers of 2026-11-22" quotes the three answers.

This revision answers the third gate pass (`~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/review-after-the-owners-answers.md`,
tip `610e5da52e`),
whose findings and their fixes "Answers to the third gate pass" records.

What it read:
the decision record `doc/decision/cli-git-zfs-worktree-design.md`,
the accepted design `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
the build plan `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
the accepted staging design `doc/planning/cli-git-zfs-staging-under-cache-design.md` for its shape,
the conversion fence `doc/planning/cli-git-clone-rung-conversion-fence.md`,
the two dirty-source decisions for the other rungs,
the btrfs design `doc/planning/cli-git-btrfs-worktree.md`,
the measurement record `doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
`package/git-policy/cli/SPEC.md`,
the code under `package/git-policy/cli/src/native/`,
`package/git-policy/cli/doc/slot-identity-store-write-cost.md` and `package/git-policy/cli/doc/object-database-and-a-reftable-ref-store.md`,
the gate review and its probe logs,
the second gate pass and its probe logs,
the decision record of 2026-11-22,
and the sibling designs named under "Dependencies on the sibling designs",
each at the commit named there.
Every outside fact is read at a named tag:
Git `v2.56.1`,
OpenZFS `zfs-2.4.4` and Linux `v7.3`.

What it ran,
all on this host and all read-only or confined to a scratch folder:
`findmnt`,
`stat`,
`du`,
`gh api` for five Linux files at `v7.3`,
`git log` and `git show` for the sibling branches at their current tips,
the citation check the "Citations" section describes,
OpenZFS's `zfs_vnops_os.c` from a fresh clone at `zfs-2.4.4` for the exchange's transaction and log lines,
and three probes of the revision before this one:
`clock_getres(CLOCK_REALTIME_COARSE)`,
a read of the main checkout's index and `ls-files --debug` times that counted the racy set,
and a read of the checkout's administration for the worktree registrations,
remotes and hooks (F38 to F40).
The measurements quoted here were taken in this round's first text,
by the reviewer,
by this revision,
or by the records named beside them.

What it did not run:
no clone was created,
because no ZFS clone route exists on this host yet,
and no guest,
container or virtual machine ran.
Nothing here is a measurement of a cloned worktree,
of a mount of a clone,
of the restore inside one,
or of the fallback after a refusal.

## Answers to the gate review

Each line names the finding,
what changed,
and where.

- H1,
the fields the installed index carries.
Fixed by removing the device-and-inode rewrite from step 5:
every entry keeps the fields its own index recorded,
the private write is given a lookup that answers nothing,
and the only situation that would need more is named as a contingency for [M2][m2].
See "The restore" step 5,
"What it reuses,
and where ZFS differs",
F28 and F29,
and T6.
- H2,
the order of the two publications.
Fixed by writing the captured index under a private name and publishing it and the pointer last.
See "The restore" steps 5 to 8,
"What a resume finds",
T10,
and the project's answer on what the slot holds.
- M1,
the change time's tick on ZFS and Git's second.
Fixed by replacing the false claim with the sources and the reviewer's probes:
OpenZFS stamps both times from the coarse clock,
the installed Git compares whole seconds,
a same-tick rewrite is invisible,
and the capture now digests the racy set so the window write is refused.
See F2,
F14 to F16,
F22,
F23,
"The frozen proof",
V1,
V4 and T3.
- M2,
the refusals never applied to the capture.
Fixed:
the whole dirty refusal set is re-run against the frozen capture,
and the departure from the plan's wording is listed under "Where the records differ",
where the record of 2026-11-22 amends that sentence.
- M3,
the fallback against the accepted rule.
Fixed:
proposal lines 427 to 431 are quoted,
the settlement is marked as depending on a callable unmount-and-destroy no sibling offers,
and the project's answer of 2026-11-22 then decided it:
the add settles and finishes by itself,
which this revision applies.
- M4,
the resume list and its tests.
Fixed:
the restore is idempotent once its waiting index and lock are cleared,
a marker file carries the proof's phase,
and the states are keyed to evidence each step leaves.
See "What a resume finds" and T5.
- M5,
the tests.
Fixed:
the host stand-in is the fixture tree itself,
every refusal test carries the no-write control,
T3 advances the tick by state,
and the missing controls are added.
See "The tests a build must have".
- M6,
the exceptions to "becomes a checkout".
Fixed:
the closing paragraph is replaced by the list of cases,
with the nested-repository case,
the metadata case,
the index extensions and the corrected issue 6323 sentences.
- M7,
the walks-and-clean bound and the omitted-mount refusal.
Fixed:
both records are quoted under "Decisions and records that bind this design",
and both are applied in the restore;
the F13 sentence is corrected.
- M8,
the build allocation.
Fixed:
D1 to D4 are the proof of every rung 1 add,
clean ones included,
and belong to step 10;
step 11 is the dirty restore and the fallback;
the plan change is veto entry V6.
- M9,
the capture's specification.
Fixed:
absent and other kinds are recorded,
the format,
the folder,
the phase,
the journal fields and the capture's end on every exit are stated,
D1's byte comparison is a veto entry,
and V3(c) is struck.
- M10,
the cost section.
Fixed:
no interpolated number,
a clean-add arm,
a test-only build instead of a shipped switch,
the rung 1 bar moved to Q-B,
and the rule code corrected.
- L1 to L8 are fixed in the sections they name:
one failure sentence,
the inherited refusals re-worded,
concrete examples in the veto list,
V2's ranking made symmetric,
the report's `USE_STDEV` slip withdrawn,
Q2 and Q3 removed,
the files-read list completed,
and the index's churn noted in F25.
- The seven wrong citations and the two loose ones are corrected,
and every other citation was re-checked by the script "Citations" describes.

## Answers to the second gate pass

The reviewer of the second pass read this document at `c342dd1a06` (`~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/review-second-pass.md`,
with `probe-logs-second-pass.md` beside it) and ranked it "fit to put to the owner after the listed fixes".
Each finding,
and where its fix lives:

- N1,
step 8's cross-dataset rename and the span in which the slot holds no `.git`.
Fixed:
the pointer is written inside the clone's root and exchanged with the copied `.git` in one exchanging rename (F36,
F37),
so no moment and no kill leaves the slot without a `.git`,
and step 8 says what a kill before,
at and after the exchange leaves.
- N2,
`restore_dirty`'s three sites that name the real index.
Fixed:
"What it reuses,
and where ZFS differs" names all three (F41),
and T6b is the control that goes red if any of them reads the real name.
- N3,
the resume's idempotence and its rows.
Fixed:
"What a resume finds" clears the waiting index and its lock with the wrapper's own step (F42),
splits the rows by the real-name index and by the exchange,
names the one state the design's own steps cannot leave,
and makes the proof's phase a marker file instead of a rewrite of the capture.
- N4,
the pre-mount fallback decided without a question.
Fixed,
and now decided:
the project's answer is that the add always finishes by itself ("The project's answers of 2026-11-22"),
so every failure after the claim takes one route.
- N5,
T3 and T3b.
Fixed:
both force their precondition by observed state,
with a bounded retry until two stamps are equal and a `utimensat` on the index file that puts an entry in the racy set,
and T3b states that its expected byte flips when V4's adopted rule lands.
- N6,
what a command run through the copied `.git` can reach.
Fixed:
stated as a limit (F40),
with the post-restore check narrowing the tree half,
and the inert copy put to the project and not chosen,
with its cost recorded under "Limitations".
- N7,
the tick's source and V4's ranking.
Fixed:
the tick is `clock_getres(CLOCK_REALTIME_COARSE)` (F38),
L4's events are corrected,
and the settle's racy rule is the design's default (V4),
with the racy set's measured size beside it (F39).
- N8 to N12 are fixed in the sections they name:
the clean rule's application in step 6,
the plan's step 10 text and V6's clarification,
the three stale phrases,
step 5's seed as the bytes D1 compared,
and the five sibling commits re-read at their tips.
- The citation findings are corrected:
F14's `sa.c` range,
F31's predicate,
and V4's reference to the racy rewrite.
- H1's remaining narrow line and M2's two small faults are fixed too:
the seed's `with_zeroed_stat` is named in step 5,
the untracked listing is written as the specification writes it,
and the second untracked listing is one of M1's timed steps.
- Three findings the pass called sound needed a sentence each when these fixes touched them,
and each is marked where it changed:
M6's low refinement,
so a split or sparse index is refused at the capture rather than after the mount;
L3,
so V7's example moved out from under `.claude`,
which the checkout ignores;
and L4,
so V2's ranking sentence no longer says two options rest on the same unmeasured sharing.

## The project's answers of 2026-11-22

The project answered three of the second pass's four questions,
recorded in `doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md` (accepted 2026-11-22,
recorded 2026-11-22T19:11:12-05:00).
The answers,
in the record's words:

- A failed add finishes by itself:
"A:
always finish itself (Recommended)",
under which "at every step,
the add destroys only what this add made and recorded (the clone,
then its snapshot),
runs Git's own checkout at the same slot,
and says which route made the worktree and why the first was abandoned" (`:35` to `:42`).
The record states what this changes:
"This amends the accepted sentence for this route:
the fallback is not silent,
since the add reports it,
and it finishes only after the mount and the add's own intents are settled by the add itself" (`:50` to `:53`).
- The frozen proof's window:
"A:
from just before snapshot (Recommended)",
with "the add records the state immediately before the snapshot,
runs every dirty-source refusal again on that record,
and refuses any write after it.
A write made before that record is part of what the snapshot holds and is restored away like any other uncommitted edit" (`:62` to `:67`).
"This amends the plan's sentence in both places" (`:69`),
which are the plan's `:1457` and `:1615`.
- What the slot holds:
"A2:
keep copy,
swap at end (Recommended)":
"the copied `.git` stays until the last step,
and the pointer and the copy then change places in one exchanging rename,
so that no moment and no kill leaves the folder without a `.git`" (`:77` to `:82`).
The record keeps the costs and the owed run:
"a Git command run in the folder during the fill acts through the copy,
which can write files into the tree being restored and which names the main repository's other worktree folders,
its remotes and its hooks;
and the exchanging rename is read from OpenZFS's source and has not been run" (`:84` to `:89`),
and "Still owed:
a guest run of the exchange,
with what a kill before,
at and after it leaves" (`:91` to `:94`).
It also records that making the copy inert was not chosen (`:104` to `:107`).

Where each answer acts:
the fallback in "What happens when the proof fails or the restore cannot finish";
the proof's window in "What the captured source is" and "Where the records differ";
the slot in "The restore" step 8 and "Limitations".
The fourth question,
whether rung 1 may decline a very dirty main checkout,
is not decided (`:109` to `:113`) and stays as Q-B,
waiting for the measurement that sizes the cut.

## Answers to the third gate pass

The reviewer of the third pass read this document at `610e5da52e` (`~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/review-after-the-owners-answers.md`) and ranked it "fit to put to the owner after the listed fixes".
Each finding,
and where its fix lives:

- R1,
the post-restore check could not be done by content for converted paths.
Fixed:
step 8 compares each written path's facts to the entry Git recorded,
to the nanosecond,
by the strict comparison,
reads content only where the stat cannot decide and the judge can,
and says what a conversion makes undecisable,
with T12's three arms and the check's cost in M1.
- R2,
the add's destroy path owned by no design.
Fixed:
"What happens when the proof fails or the restore cannot finish" owns it step by step,
calls none of the deferred-destroy checks,
writes no plan,
treats a reaped snapshot's `ENOENT` as done,
releases the lock while keeping the registration,
and closes the binding and releases the claim,
with the requirement each sibling must meet;
"Dependencies on the sibling designs" matches the siblings at the tips this revision read.
- R3,
the copy's place in the trash.
Fixed:
it is the trash's next `d<index>` entry in a forty-hexadecimal-digit trash directory,
its identity is written into the trash's `.manifest` before the exchange,
and a new set-aside-by-exchange function does it.
- R4 and R13,
the resume rows against the kill states.
Fixed:
a row for "the copied `.git` and the real-name index" exchanges and finishes,
the unreachable clause is gone,
a settle marker makes a resume continue the settle,
and a registration with no clone takes a fresh capture before any new snapshot.
- R5,
the add's handles and its unmount intent.
Fixed:
the add closes every handle under the clone before it requests the unmount,
and the intent is put to the project as Q-R5 with its ranking.
- R6,
the settle's racy test's inputs.
Fixed:
"Changed rather than reused" lists the capture's nanosecond modification time and the tick as new inputs,
with T13 as the control.
- R7 to R12 and R14 are fixed where they apply:
the waiting-index function is made callable,
T3's fixture writes its own entry,
the pathspec,
the doubled quotes,
the broken record path,
the corrections' `sa.c` range,
F41's lines,
the snapshot rung's sentence,
"narrows" instead of "closes",
F37's one-transaction and one-log-record lines,
and `doctor`'s line replaced by the binding design's.

## The open item and the words it carries

The accepted design lists the item,
quoted whole (`doc/planning/cli-git-zfs-worktree-design-proposal.md:1749` to `:1753`):

> - The dirty-main restore on rung 1.
> The answer's words:
> "Verified restore (Recommended)".
> A clone of the main
> checkout carries whatever other sessions had uncommitted in it,
> so the add needs a defined Git index and
> ignored-state restore and a frozen proof that the clone matches the captured source,
> and neither the restore nor
> its proof is designed here ([Dirty source on rung 1](#project-decisions)).
> Add steps 4 and 5 verify the frozen
> content and fill the clone but restore nothing yet ([Add and Git registration](#add-and-git-registration)).

The answer the item carries is the same file,
"Dirty source on rung 1" (`:1445` to `:1452`):

> - Dirty source on rung 1.
> Decided:
> "Verified restore (Recommended)",
> a verified dirty-main restore comparable to
> Btrfs.
> The unselected clean-only route would have narrowed the proof but lost clone speed on a dirty main
> checkout,
> and the main checkout here usually holds other sessions' uncommitted edits.
> Dirty support needs a
> defined Git/index and ignored-state restore,
> with a frozen proof,
> and
> neither route accepts an unproved source.
> [VM item 1][vm-register] ran `git read-tree HEAD` then reported empty
> tracked status:
> Git saw the fixture's tracked content matching `HEAD`.
> It did not capture the source index and
> tree before the snapshot or compare that proof to frozen content.
> That comparison,
> conversion checks,
> dirty
> restoration and concurrent writes remain unmeasured.

The plan lists the item as step 11's blocker,
"step 11,
blocking,
writer design work" (`doc/planning/cli-git-zfs-worktree-implementation-plan.md:2836` to `:2839`),
and until step 11 lands an add from a dirty main checkout takes rung 3,
by the answer "Ship step 10 first (Recommended)" (`:1463` to `:1468`).

## Decisions and records that bind this design

Each bullet quotes the lines that decide a point,
with path and line.

- The restore's promise:
`package/git-policy/cli/SPEC.md:1253` says "the new worktree is restored to what `git worktree add` makes from a clean checkout".
- The refusals that survive a dirty source:
`SPEC.md:1264` to `:1282`,
"an operation in progress or a held `index.lock`;
any unmerged,
intent-to-add,
skip-worktree or assume-unchanged entry,
or a submodule,
found anywhere in the index",
"a `.gitattributes` file among the changes or untracked anywhere",
whose listing "runs real Git with `--no-literal-pathspecs`" (`:1274` to `:1276`),
"a sparse checkout",
"a source that is a linked worktree",
and "the clone rung,
which is only for clean sources".
- The fields an installed entry may carry:
`SPEC.md:1289` to `:1293`,
"An entry no walk proved keeps the fields its own index recorded,
never fields taken from the file:
fields of the file itself would vouch for its bytes,
and Git would take an edit for the entry's content."
- The restore's runs:
`SPEC.md:1294` to `:1307`,
the three runs in order and "These three runs and nothing else are run for the restore."
(`:1355`).
- The settle's belief:
`SPEC.md:1316` to `:1331`,
the files the stat data cannot vouch for are read and compared with the blobs their entries name,
and the main checkout is read to be snapshotted and listed (`:1381`),
"never written" (`:1382` to `:1384`).
- The left-out line:
`SPEC.md:1386` to `:1393`.
- Writes during the snapshot are out of scope for the btrfs rung:
`SPEC.md:1396` to `:1399`.
- The ignored state's instant:
`SPEC.md:1518`,
"The ignored files the new worktree carries are the source's as of the instant this add's snapshot was taken".
- The records an add writes for itself:
`SPEC.md:4408` to `:4410`,
"Each is a private file created exclusively beside its final name and renamed over it,
so a reader sees the whole record or none,
and the filesystem makes it durable with its next commit."
- The wrapper never asks for a secret:
`SPEC.md:7345` to `:7352`,
"No command the wrapper runs or forwards asks a person for a credential",
with "Avoid ever asking for someone to type a username or password."
and "Passphrases count too (Recommended)".
- The walk and clean bound:
`doc/planning/cli-git-zfs-worktree-design-proposal.md:1073` to `:1083`,
"never hand `git clean` a path under `.claude/worktrees` or force cleanup across a mount.
A mount or a tree containing one is a refusal for destructive cleanup,
not an ordinary ignored directory",
and "double-force dry runs also show why ordinary nested-repository protection is insufficient".
- The omitted-mount refusal:
the same file `:1036` to `:1037`,
"If tracked output would occupy an omitted mount location,
refuse the clone fill rather than writing through a boundary",
and `:403` to `:404`,
"A snapshot omits nested mount contents but can carry real underlay files and stale Git stubs;
remove only the verified clone-local copies,
never the live mounts or source underlays."
- No fallback over an unresolved state:
the same file `:427` to `:431`,
"If snapshot or clone returns `ENOSPC`,
report the operation,
object and pool,
keep the claim,
and clean up only objects whose creation and identity are proven and whose destruction is allowed.
A clone pins its origin even before it is mounted.
If mutation succeeded but its identity was not recorded before interruption,
keep and report it unless identity can be independently recovered.
No silent fallback over an unresolved managed state.
An explicitly recovered Git-only add may finish at the same slot only after the mount and all outstanding intents are settled."
- The add's steps 4 and 5:
the same file `:396` to `:410`,
including "Compare the frozen index and tracked content in the clone itself against the captured proof,
without running Git against that directory",
"Discard copied Git administration before running Git inside the clone",
"Install the saved Git pointer inside the clone",
and "A mount can be externally visible before the fill;
an interrupted add can expose copied `.git`,
which is neither the linked registration nor a success signal."
- The preflight capture:
the same file `:370`,
"capture the index/tree verification inputs".
- The three answers of 2026-11-22,
and what the record says each amends:
`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
its section "Whether a failed add may finish with Git's own checkout by itself" (`:21` to `:53`),
"Which writes the frozen proof refuses" (`:55` to `:69`) and "What the slot holds while the add fills it" (`:71` to `:94`),
and its "What was not chosen" (`:96` to `:107`).
Its fourth question,
whether rung 1 may decline a very dirty checkout,
is left open (`:109` to `:113`).
- No configuration:
`doc/decision/cli-git-zfs-step-8-sweep-choices.md:19` to `:45`,
the project "first answered that everything must work by default with no configuration",
and for no file at all,
"1:
no file needed (Recommended)".
- No lock for a per-item record:
`AGENTS.md` rule `NLK` at `:1149`,
"Concurrency is lock-free first (atomic rename,
`O_EXCL`,
one owner per item);
an unavoidable lock covers the smallest unit,
never a whole repository,
with its reason written beside it."
- The store precedent:
`doc/decision/cli-git-zfs-slot-identity-store-format.md:132` to `:175`,
"the record is created with `O_EXCL` under a hidden staging name,
linked to its real name in one `linkat` (whose `EEXIST` decides the one winner),
and the staging name is then removed",
and its "Who removes a killed writer's staging file" places that removal in `doctor` at plan step 18.
- Shipped limits cite a measurement:
`AGENTS.md` rule `LQM` at `:1567`,
"Shipped resource limits (unit caps,
timeouts,
size bounds) cite a workload measurement or owner decision beside the value".
This design ships no duration and no bound it has not measured,
and names the measurement that would size each.
- Worktrees live under `.claude/worktrees/` of the main checkout (`AGENTS.md` rule `WTU` at `:758`),
and every path comes from the injected home (rule `HOM` at `:1040`).
- The threat model:
`doc/decision/cli-git-zfs-staging-under-cache.md:221` to `:237`,
"the account is trusted,
and root and the account are not a security boundary here",
from the answer "We have to assume my machine isn't compromised.
sudo is passwordless here anyway."
"Hardening the stated threat model may make unnecessary" places each check against it.
- Noisy comparisons and the reason to repeat them:
`AGENTS.md` rule `QNB` at `:362`,
and the machine's normal pressure in rule `NQH` at `:850`.
- The conversion fence:
`doc/planning/cli-git-clone-rung-conversion-fence.md:1660` to `:1672`,
"no path is recorded clean before its bytes are Git's checkout's",
and its limitations at `:3607` and after.
- The other rungs' dirty-source answers:
`doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md` and `doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`.
Neither decides anything about rung 1's clone of a snapshot.

## Where the records differ, and what this design does about it

- A write during the snapshot is out of scope for the btrfs rung (`SPEC.md:1396` to `:1399`),
and the answer this design serves requires a proof that refuses such a write.
The btrfs rung's sentence stays true of the btrfs rung,
rung 1 refuses what its own proof catches,
and neither claims to defend against a write after the snapshot's transaction group (L3).
- The plan's acceptance wording was narrower than this design:
"Cover recognized and rejected forms,
dirty sources and a tracked write between preflight and snapshot that the frozen proof refuses" (`doc/planning/cli-git-zfs-worktree-implementation-plan.md:1456` to `:1459`,
repeated at `:1614` to `:1617`).
The project's answer of 2026-11-22 settles it the way this design works:
the capture is taken immediately before the snapshot,
every refusal is re-run against it,
any write after it refuses,
and a write before it "is part of what the snapshot holds and is restored away like any other uncommitted edit" (`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md:62` to `:67`).
The record amends the plan's sentence in both places (`:69`),
and V6 lists that edit with the allocation.
- "The clone rung,
which is only for clean sources" (`SPEC.md:1281` to `:1282`) is the ladder's rung 2,
the same-dataset reflink,
while the proposal's rung 1 is the snapshot clone and the plan's step 3 speaks of "the clone rung" for rung 2.
This document says "rung 1" or "the ZFS snapshot clone" for its own subject and "rung 2" or "the ladder's clone rung" for the other.
- The word "proof" names four different things across these records:
the clean check's stat proof,
the rung 2 proof that a source matches its commit,
the fence's size proof,
and this document's comparison of a clone with a capture.
"Frozen proof" here means D1 to D4 and nothing else.
- Issue 6323 is the conversion defect (`SPEC.md:2736` to `:2738`,
"a clone keeps the source's bytes where the checkout would convert line endings or `ident`"),
and the handover records that it "closes with" the fence's merge (`doc/handover/cli-git-rust-implementation.md:41374`),
while the plan's step 10 and step 11 notes still say they are "also blocked by issue 6323" (`doc/planning/cli-git-zfs-worktree-implementation-plan.md:1499` and `:1628`).
This design takes the handover's reading:
the fence is on `main`,
rung 1 applies it,
and the guest controls T8 are what a dirty restore to a non-`HEAD` commit owes.
It is not 6323's class that a same-size,
same-tick write leaves behind (L4,
V4).
- The step 8 sweep record answers that everything must work with no configuration,
and the handler still reads a root-owned file when one exists.
Nothing in this design reads or writes a configuration file.
- [VM item 1][vm-register] moved the copied administration aside rather than deleting it,
and measured no deletion cost.
This design takes the measured route and owes the deletion's cost to [M3][m3].

## Facts

Each fact names its source,
the lines read,
and,
where it is a measurement,
who ran it and what it showed.
Nothing here is a measurement of a clone.

### Git, at `v2.56.1`

- F1.
An index entry records the path and,
beside it,
the change time,
the modification time,
the device,
the inode,
the mode,
the owner,
the group and the size of the file the entry was written against:
`statinfo.c:24` to `:35` (`fill_stat_data`).
- F2.
Git's own stat comparison reads the seconds of the modification time and the change time (`statinfo.c:69` to `:73`),
the owner,
group and inode and the size (`:83` to `:89`),
with `core.checkStat` and `core.trustctime` on by default (`environment.c:776` to `:777`).
The sub-second parts are compared only under `USE_NSEC` (`statinfo.c:75` to `:81`),
which is opt-in (`Makefile:200` to `:205`),
and the device only under `USE_STDEV` (`statinfo.c:91` to `:99`),
which is opt-in too (`Makefile:213` to `:214`).
The installed Git `2.56.1` compares seconds:
a change below the second was not listed while a change of one whole second was,
as F23 records.
- F3.
`git read-tree --reset -u <commit>` rewrites only the paths whose file Git's own stat check can see changed.
`--reset` is `UNPACK_RESET_OVERWRITE_UNTRACKED` (`builtin/read-tree.c:186`);
an entry under a reset never refuses for being out of date (`unpack-trees.c:2238` to `:2241`);
and the update flag is set only where `lstat` fails or the entry's stat data differs from the file (`unpack-trees.c:3036` to `:3044`).
- F4.
`git clean --force -d` removes the untracked files and the untracked directories that no ignore rule covers (`builtin/clean.c:1071`) and refuses without `--force` when `clean.requireForce` is set (`builtin/clean.c:957`).
It keeps a directory that holds its own repository:
`rm_flags` starts as `REMOVE_DIR_KEEP_NESTED_GIT` (`builtin/clean.c:927`),
is cleared only when `--force` is given twice (`:959` to `:960`),
and `remove_dirs` leaves a directory that `is_nonbare_repository_dir` names (`:176` to `:186`).
- F5.
`git checkout-index --force -u --stdin` writes the named paths out of the index through Git's own conversion and records each written path's stat data:
`builtin/checkout-index.c:101`,
`:178`.
- F6.
`git status` refreshes the index and writes it back unless optional locks are off:
`builtin/commit.c:1634` takes the lock only when `use_optional_locks()` is true,
`:1658` writes the index when it did,
and `environment.c:228` to `:231` reads `GIT_OPTIONAL_LOCKS`,
which `--no-optional-locks` clears.
- F7.
`git worktree add` writes `<path>/.git` as a file holding `gitdir: <administrative directory>` and writes the reciprocal `<administrative directory>/gitdir`:
`builtin/worktree.c:532` to `:544`,
through `write_worktree_linking_files` at `worktree.c:1105` to `:1134`.
With `--no-checkout` nothing else is written into the worktree.
- F8.
An index can be written under a private name and published afterwards,
and one already exists that does so:
`worktree_clone_index.rs:111` to `:145` (`PendingIndex`,
whose comment says "This is the moment the worktree starts vouching for its files;
before it,
no index exists there at all") and `subvolume_index_install_into_new_worktree.rs:190` to `:210` (`publish_index_under_a_name`,
one rename that replaces nothing).
The write refuses an existing private name and refuses when a real `index` appears (`:150` to `:162`),
and the clone rung's seed passes through `with_zeroed_stat` before it is written (`worktree_clone_conversion_beside_the_clone.rs:189` to `:196`,
`seed_without_stat`).
- F9.
The restore's own Git runs name their Git directory and work tree through the environment,
and take an index file when one is named:
`worktree_ladder_target.rs:418` to `:440` (`checkout_environment` and `checkout_environment_with_index`).

### OpenZFS, at `zfs-2.4.4`

- F10.
A clone's initial object set is its origin snapshot's block pointer:
`module/zfs/dsl_dataset.c:1225` to `:1227` copies the origin snapshot's block pointer into the new clone,
`dsphys->ds_bp = dsl_dataset_phys(origin)->ds_bp`,
when the clone's dataset object is created.
- F11.
A snapshot's block pointer is the dataset's own at the transaction group being synced (`module/zfs/dsl_dataset.c:1782` to `:1784`),
and a second snapshot of the same transaction group is retried rather than taken (`:1534` to `:1539`,
an `EAGAIN` the caller retries).
- F12.
A ZPL inode number is the object number:
`module/os/linux/zfs/zfs_znode_os.c:600` sets `ip->i_ino = zp->z_id`,
and `:546` takes `z_id` from the dnode's object number.
With F10,
a clone's files carry the snapshot's inode numbers.
- F13.
A snapshot that is a branch point cannot be destroyed:
`module/zfs/dsl_destroy.c:55` to `:89` refuses with `EEXIST` when the snapshot is a branch point,
which a live clone makes it.
The measurement record confirms the refusal for a snapshot with a dependent clone (`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`,
"Item 2:
one named snapshot has a dependent clone").
- F14.
The ZPL keeps each of the access,
modification and change times as a sixteen-byte value,
seconds and nanoseconds:
`module/zfs/sa.c:1702` to `:1704`.
The nanoseconds are the coarse clock's,
not the write's own (F15).
- F15.
A ZFS write stamps the modification time and the change time from `gethrestime`,
which is the coarse clock:
`include/os/linux/spl/sys/time.h:68` to `:72` (`ktime_get_coarse_real_ts64`),
`module/os/linux/zfs/zfs_znode_os.c:1420` to `:1446` (`zfs_tstamp_update_setup`),
called from the write path at `module/zfs/zfs_vnops.c:1013`.
Two back-to-back rewrites can therefore carry one stamp,
as F22 measured.
- F16.
ZFS takes a change time from the VFS when the VFS sets attributes (`module/os/linux/zfs/zfs_vnops_os.c:2546` to `:2549`) and it does not declare `FS_MGTIME` (`module/os/linux/zfs/zpl_super.c:1092` to `:1099`),
so the kernel does not give the next write a fine-grained stamp after a `stat` on it (F21).
- F17.
The clone's copied `.git` is a directory,
and the nested copy of the slot holds Git's own pointer file:
measured in the guest by [VM item 1][vm-register] (`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md:56`),
"The clone's root had a `.git` directory;
the nested copy of the slot had the exact underlay pointer,
proving the snapshot included the registration."
- F18.
A snapshot of the checkout dataset omits a nested mount's contents but can carry the underlay's real files and stale Git stubs (`doc/planning/cli-git-zfs-worktree-design-proposal.md:403` to `:404`),
so a nested worktree's copy in the clone may not be an empty directory.

### Linux, at `v7.3`

- F19.
Every write sets the change time to the present unless its caller passes `ATTR_CTIME_SET` (`fs/attr.c:465` to `:478`),
and a buffered write reaches it through `file_update_time` (`fs/inode.c:2501`,
`mm/filemap.c:4472`).
- F20.
No userspace call selects the change time:
`utimensat` passes `ATTR_CTIME`,
but only `ATTR_ATIME_SET` or `ATTR_MTIME_SET` for the values it was given (`fs/utimes.c:41` to `:55`).
- F21.
A file system that declares multigrain timestamps gets a fine-grained stamp on the next write after someone queried the time;
one that does not,
keeps the coarse stamp:
`fs/inode.c:2425` to `:2446` and `:2894` to `:2912`.
btrfs declares `FS_MGTIME` (`fs/btrfs/super.c:2232` to `:2233`);
ZFS does not (F16).

### This host, measured

- F22.
The reviewer rewrote one four-byte file 400 times with no wait:
385 of the 399 consecutive pairs kept both the change time and the modification time although the bytes changed,
the stamps moved in steps of one millisecond,
and the 14 pairs that moved are the probe's own positive control (`~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/probe-logs.md`,
"P1",
on `zpcachyos/ROOT/cos/home`).
A same-size rewrite can therefore leave every compared field of F2 and of the wrapper's settle equal.
- F23.
The reviewer moved only the sub-second part of a file's modification time and change time in one test arm and a whole second in a control arm:
`git diff-files --name-only` listed the control's file and not the test's,
so the installed Git compares seconds (probe logs,
"P2",
with the control stated).
- F24.
The reviewer's dry run of `git clean --dry-run -d` in a worktree holding an untracked directory with its own repository named the plain untracked file and not the directory,
while the listing the restore's proof makes (`ls-files --others --exclude-standard --directory`) named it (probe logs,
"P3").
The real removal was not run;
its behaviour is F4.
- F25.
This round's first text measured the main checkout at 24,685 tracked paths,
an index of 3,436,101 bytes,
and 764,137,159 bytes of tracked content;
the reviewer's later read gave 24,685 paths and 3,436,657 bytes of index,
the difference being the checkout's own churn (its index was rewritten at 17:45:57,
probe logs "P4").
The same text measured the listing of the tracked names at 25.6,
40.9 and 29.4 ms,
one `lstat` per path at 175.7,
111.3 and 134.3 ms,
reading the index at 1.4,
1.4 and 1.5 ms,
and a SHA-256 of every tracked file at 9,380.9,
4,852.3 and 5,218.4 ms,
in scripts in this round's scratch folder,
not in the wrapper.
- F26.
This round's first text measured the main checkout's common Git directory at 2,732,668,640 bytes in 634,426 files and 6,708 directories,
its ignored state at 214,899,299,440 bytes under `.claude`,
2,745,349,203 under `package` and 185,182,920 under `node_modules`,
and its status at 5 changed paths.

### The wrapper, in this tree

- F27.
The btrfs rung's dirty restore exists and is reused:
`restore_dirty` (`package/git-policy/cli/src/native/worktree/worktree_dirty_restore.rs:465`) lists the carried ignored paths,
runs `git clean --force -d` captured into the journal (`:478` to `:492`),
removes the files the reset cannot see,
runs `git read-tree --reset -u <commit>`,
and proves the result with `prove_restored` (`:292`,
whose untracked listing is at `:297`).
- F28.
The index install serves a dirty source by keeping the seed's fields:
`install_index` (`worktree_snapshot_index_step.rs:196`) takes `own_index: Option<&[u8]>` and,
for a dirty source,
fills no lookup,
so "An entry the walk did not cover ...
keeps the stat fields the index it came from recorded" (`:268` to `:270`);
`WalkedStats` (`:305` to `:310`) answers nothing for an entry the walk did not cover.
`read_own_index` (`:327`) opens `<copy>/.git/index` below a held root without following a link.
- F29.
The one existing rewrite that takes fields from a file writes the change time too:
when a lookup answers,
the rewrite writes the file's change seconds and nanoseconds,
device,
inode,
owner and group into every entry whose recorded size and modification time already equal the file's (`package/git-policy/cli/src/native/subvolume/subvolume_index.rs:520` to `:570`,
the field list at `:554`),
trusted by `AcceptEveryStat` (`subvolume_index_install_into_new_worktree.rs:116` to `:121`).
There is no mode that writes the device and inode alone.
- F30.
The wrapper's settle does not read a file whose recorded stat data is its own to the nanosecond,
"even when Git calls the entry racy" (`worktree_clean_tree_walk.rs:199` to `:205`,
`:248`).
- F31.
`paths_whose_content_differs` is defined at `worktree_clean_tree_walk.rs:223` and re-exported at `worktree_clean.rs:49`.
Its content test is `matches_stored_exactly` (`:248`),
which C3 needs:
an entry whose recorded fields are the file's own to the nanosecond is vouched for,
and every other entry is read and compared with its blob.
- F32.
The btrfs rung hands the copied `.git` to a trash inside the copy in one rename and lets the disposal empty it after the command returns:
`worktree_snapshot.rs:284` with `hand_over.take_entry`.
- F33.
The btrfs rung's fallback completes the worktree with `git reset --hard` in the destination (`worktree_ladder_finish.rs:324`),
and the clone rung's failure sentence is `the clone rung could not finish (<reason>), and Git's own checkout made the worktree.` (`worktree_slow_route.rs:278`;
`SPEC.md:2787`).
- F34.
A split or sparse index is declined by the rewrite,
with the reason `the index uses the <signature> extension (split or sparse index)` (`subvolume_index.rs`,
the `DECLINED` list and its message),
so an index that carries one cannot be installed.
- F35.
The copy journal's folder sweep reads only the names that end in its journal suffix (`worktree_copy_journal.rs:604`),
so a record in another folder is not read by it.
- F36.
The wrapper has the exchanging rename twice:
`exchange_paths` (`package/git-policy/cli/src/native/subvolume/fs_clone_exchange.rs:66` to `:85`,
`renameat2(AT_FDCWD, …)` with `RENAME_EXCHANGE`) and `exchange_at` (`fs_beneath.rs:348` to `:355`,
the same call from one open folder).
Their own comments state the property the design needs:
"After the call `first` names what `second` named and the other way round,
with no moment at which either path is missing",
and "Both paths must exist and be on one filesystem".
- F37.
OpenZFS `zfs-2.4.4` and Linux `v7.3` allow a directory and a regular file to exchange:
`module/os/linux/zfs/zfs_vnops_os.c:2818` to `:2824` accepts `RENAME_EXCHANGE` and only one of the three flags,
`:3034` to `:3046` skips the same-type requirement "unless exchanging",
and `:3052` to `:3055` requires the target to exist;
`fs/namei.c:6103` to `:6108` and `:6127` to `:6131` judge the exchanged target by its own type,
and `:6182` to `:6185` guards the mixed pair's link counts.
OpenZFS performs the exchange in one transaction and one log record,
which is what the atomicity rests on:
`zfs_vnops_os.c:3084` to `:3089` holds both directories,
the target and the exchanged object under one `dmu_tx`,
and `:3224` to `:3229` logs one `zfs_log_rename_exchange`.
Not run:
[M9][m9]'s guest run is owed.
- F38.
The kernel reports its coarse clock's resolution as one millisecond on this host:
`clock_getres(CLOCK_REALTIME_COARSE)` printed `0.001` at 2026-11-22T19:03:31-05:00 in a scratch command,
which sizes C4's tick and the settle's racy rule.
- F39.
The racy set held one entry of 24,689 at 2026-11-22T19:03:31-05:00:
a scratch script read the main checkout's index mtime and its `ls-files --debug` times and found `doc/handover/cli-git-rust-implementation.md` within one tick of it,
with 36 recorded times carrying a zero sub-second part,
which agrees with the reviewer's P5.
- F40.
A copy of the main checkout's `.git` names places outside a clone of it:
`<common>/worktrees` held 187 registrations whose `gitdir` files spell absolute paths (the reviewer read 188;
the count churns;
the first was `/var/home/user/Monochromatic/.claude/worktrees/add-explains-slow-route/.git`),
one remote named `origin`,
and the repository's four hooks (`post-checkout`,
`post-commit`,
`post-merge`,
`pre-push`);
it holds no alternates file and sets no `core.worktree`.
Read on 2026-11-22T19:03:31-05:00;
the reviewer's P7 read the same shape.
- F41.
`restore_dirty` names the real index in three places and nowhere else:
`ask` builds `checkout_environment(target.admin, work_tree)` for the proof's runs and the ignored listing (`worktree_dirty_restore.rs:185` to `:195`),
`restore_dirty` builds its own environment for the clean and the reset (`:470` to `:473`),
and `remove_the_files_the_reset_cannot_see` reads `target.admin.join("index")` and takes the settle's index seconds from that file's mtime (`:419` to `:421`).
- F42.
The wrapper already clears a waiting index and its lock:
`clear_the_waiting_index` (`worktree_copying/worktree_copy_interrupted_add_settlement.rs:366` to `:372`) removes `index.pending` and `index.pending.lock`,
and the settlement calls it at `:562` and `:597`.

## Assumptions

- A1.
A mounted clone reports the snapshot's own metadata,
because its object set is the snapshot's (F10):
the inode number,
size,
kind,
mode,
and the recorded times of every unchanged file equal the source's at the snapshot.
[M2][m2] measures it.
- A2.
Other sessions write the main checkout through ordinary writes,
editors and Git commands,
so a write between the capture and the snapshot is a VFS write (F19) that lands in some millisecond tick (F15).
A writer that sets the system clock back,
or that writes through a raw device,
is outside the threat model this design designs to.
- A3.
The wrapper's index reader and its stat comparison (F28,
F30) read an index written by Git `2.56.1` and a file on the clone as they read them on btrfs,
so their verdicts carry over.
- A4.
The clone-local trash can be emptied after the add returns by the same disposal that empties a btrfs snapshot's trash,
with the clone mounted at the destination (F32).
- A5.
The capture's cost in the wrapper is the order of the rung 2 proof's own walk,
34 ms over 24,313 tracked files in `doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md:57`,
rather than the node probe's F25 figures,
because the probe pays for a scripting runtime per call.
[M1][m1] measures the wrapper's own arms,
and no figure is claimed here.

## Limits

- L1.
No clone exists on this host:
the checkout is still a directory in `zpcachyos/ROOT/cos/home`,
not the checkout dataset of the design,
and the route's own code is unreleased.
Nothing in this document is a measurement of a clone,
a clone mount,
or a restore inside one.
- L2.
No guest ran in this round,
so every ZFS and mount claim rests on the source at `zfs-2.4.4` or on the guest records,
and is marked unmeasured where the guest has not already measured it.
- L3.
The window write is detected,
never prevented,
and a write after the snapshot's transaction group is a write in the main checkout,
not in the clone;
the design does not catch it,
which is the risk `SPEC.md:1396` to `:1399` accepts for every rung.
- L4.
The proof's detection rests on a one-millisecond tick (F15,
F22,
F38) and on Git's seconds (F2,
F23).
For a write before the capture,
three events must share one tick:
the file's own last write,
the index write that recorded its stat,
and the second write.
This add's `lstat` is not one of them;
a write after the capture's read is refused by C4 and D4.
The design's adopted settle rule reads by content the entries in the racy set,
which is every entry such a coincidence can hide in (V4,
F39),
so the residual is closed unless the project vetoes that rule.
The exposure's size is the racy set's,
one entry of 24,689 at one moment (F39);
[M7][m7] counts it over a day.
- L5.
The capture's cost figures come from scratch scripts and from another round's record (F25,
A5);
neither is the rung's own timer.
- L6.
The proof needs the clone's copied `.git/index`,
or the proof's phase marker.
A resume that finds the copied index gone and no marker cannot prove the frozen clone from the capture alone,
because the capture holds the index's digest and the files' facts,
not the files.
- L7.
The capture's size is an estimate:
1,648,834 bytes of path names and a per-line field span,
so about 2.5 MB for this repository's 24,685 paths,
larger than the first text's 2.2 MB because it now carries the kind,
the execute bit and an absent row per deleted path.
- L8.
A command run in the slot through the copied `.git` before the exchange can reach three places (F40):
the tree being restored,
where `git restore .`,
`git checkout .`,
`git stash` and `git reset --hard` write the copy's HEAD bytes and the writes stay because the tree is the future worktree;
the main repository's other worktree folders,
where `git worktree remove` acts on the absolute paths the copy names;
and the remote,
where `git push` sends whatever the copy's branch holds.
It never writes the main checkout's own files,
its index or its references,
because the copy sets no `core.worktree` and has no alternates.
The post-restore check narrows the tree half;
the other two are stated and not closed.
- L9.
The exchanging rename is read from OpenZFS and Linux sources (F37) and has not been run;
[M9][m9]'s guest run is owed,
with what a kill before,
at and after the exchange leaves.
The inert copy the project did not choose is costed from F40's counts and not measured.

## The parts that need a design

### What "the captured source" is

The captured source is read from the live main checkout by the wrapper itself,
with held handles and no Git run of its own;
the only Git runs the capture needs are the untracked listings the refusal set below makes,
which the wrapper already runs for the btrfs rung:

- C1.
The index:
the bytes of the checkout's own index file,
`<checkout>/.git/index`,
with its size,
its modification time and its SHA-256 digest.
The whole refusal set below reads these bytes.
- C2.
Every path in that index:
its kind (regular file,
symbolic link,
or absent),
its size,
its modification time and its change time to the nanosecond,
and the owner's execute bit;
for a symbolic link also the bytes of its target.
A tracked path the source has deleted is recorded as absent,
so a write in the window that brings it back is comparable.
A path whose kind is neither a regular file nor a symbolic link is refused before the snapshot,
as a kind no reset can undo.
- C3.
A SHA-256 of the content,
or of the link's target bytes,
for every path whose index stat data cannot vouch for the file,
judged by the wrapper's existing predicate (F31).
- C4.
A SHA-256 of the content of every path in the racy set:
the entries whose recorded modification time or change time is not at least one tick older than C1's index modification time.
The tick is `clock_getres(CLOCK_REALTIME_COARSE)`'s answer (F38),
one millisecond on this host,
and an answer the wrapper cannot read treats every entry as racy,
which is the safe direction.
On this host's ZFS that is the set a same-tick write can hide in (L4).
The set held one entry of 24,689 at one moment (F39),
and [M7][m7] counts it over a day.
- C5.
The commit the caller's add checks out,
read from the new worktree's `HEAD` file and the ref it names after Git's registration,
so the record names what the restore will write.
The source's own `HEAD` is recorded beside it,
because the fence's tree reads were made against it,
and a ref moved in the window is reported rather than refused:
the commit the restore writes is the one Git registered.
- A split or sparse index is refused from C1's extension signatures (F34),
so the rung ends before anything is written rather than after the mount.
- The refusal set of `SPEC.md:1264` to `:1282` is re-run against these bytes and against the source immediately before the capture is written:
the index-entry kinds from C1's bytes,
an operation in progress or a held `index.lock`,
the untracked `.gitattributes` listing,
whose pathspec is `:(glob)**/.gitattributes`,
run by real Git with `--no-literal-pathspecs` (`SPEC.md:1274` to `:1276`),
a sparse checkout,
and a linked source.
A refusal here leaves no managed object.

It is taken twice,
and the two readings are different objects:

- the preflight reading,
at plan step 10's step 1,
whose index bytes decide the cheap refusals;
- the frozen capture,
taken immediately before the snapshot command in step 3,
after the fence's reads and after Git's registration,
with the whole refusal set re-run against it.
It is written into the add's record before the snapshot runs,
and it is the one the proof compares.

The record,
its format and its end:

- Home:
`<common>/monochromatic-git-policy-cli/add-capture/v1/<slot>.capture`,
following the wrapper's record convention for its folder,
one family folder and a grammar version,
the slot as the key,
the account's `0700` folder and `0600` file.
It is not the binding record (that design declined to carry the proof) and not the copy journal's folder (whose sweep reads only its own suffix,
F35).
- Format:
a header line `capture=1`,
the repository's common directory as bytes,
the source root as bytes,
the index's size,
modification time and digest,
the captured source `HEAD`,
the checkout's Git-directory identity,
and the entry count;
then one line per entry,
the path as lowercase hexadecimal and its kind,
size,
both times to the nanosecond and execute bit,
absent entries included;
then one line per digest of C3 and C4.
A reader refuses an unknown version,
a line it cannot parse,
or a count that disagrees with the lines.
- The proof's phase:
a marker file `<slot>.capture.passed` beside the record,
made with an exclusive staging name and one link to its real name that replaces nothing,
so a phase flip costs a small file and never a rewrite of 2.5 MB,
and a resume reads the marker rather than the record.
The marker is removed with the record.
- Publication:
an exclusive staging name `<slot>.capture.pending`,
the whole file written,
then one link to the real name that replaces nothing,
then the staging name removed,
which is the store precedent of this repository.
- Size bound:
one line per index entry,
its length bounded by the path the index holds plus a fixed field span,
so the file's bound is the index's own entry count times that span;
no separate total is shipped,
and the shipped span is computed from the index's own bound rather than measured.
The estimate for this repository is L7.
- End:
it is removed when the binding is marked usable,
and on every exit that leaves no resumable add:
a refusal that settles its objects,
a fallback that completes with Git's own checkout,
or a state in which the proof can no longer pass.
It stays while the add is resumable,
which is what a resume proves from.
`doctor` reports and removes a `<slot>.capture.pending` a killed writer left,
as it does for the slot identity store's staging names.

### The frozen proof

The proof runs in the mounted clone after the add's step 4 has verified the mount's identity and the caller's visibility,
and before any byte of the clone is written.
It reads the clone's files below a held handle of the clone's root,
following no link,
and it runs no Git against the clone's copied administration:
a Git run there would read the copied index as its own,
rewrite it,
and could move the copied refs of the checkout.
So the copied index is read directly,
by the wrapper's own reader (F28).

The proof compares,
in this order:

- D1.
The bytes of the clone's copied index,
`<clone>/.git/index`,
against C1:
same length and the same SHA-256,
byte for byte,
with no separate comparison of the entry set,
because equal bytes are equal entries and the entry kinds were judged by the refusal set on these same bytes.
This catches any write to the index between the capture and the snapshot,
which is what a concurrent `git add`,
`git reset`,
`git commit`,
`git stash` or a merge in the main checkout does.
A `git status` in the main checkout that refreshes stat data also rewrites the index and refuses the add,
which is the cost V8 weighs.
- D2.
For every path in the captured index,
the clone's own facts against C2:
the kind,
the size,
the modification time to the nanosecond,
the change time to the nanosecond,
the execute bit,
and the target bytes of a symbolic link.
The device and the inode are not compared,
because a clone is another dataset and F12 and [M2][m2] say its inode numbers are the snapshot's own.
- D3.
For every path C3 recorded a digest for,
the clone file's SHA-256 against it.
- D4.
For every path C4 recorded a digest for,
the clone file's SHA-256 against it.
This is what refuses a write that lands in the same tick as the file's own last write,
the case F22 measures and L4 sizes.

A difference in any of them refuses the clone rung.
The refusal's reason names the path and the field,
and the person reads the one failure sentence (F33) with that reason in it.

How a write between the capture and the snapshot is caught:

- a write to a tracked file in a later tick moves its change time past every compared field of F2 and D2,
so D2 refuses and names it;
- a write in the file's own tick leaves both times and the size equal (F22),
and D4 refuses it because C4 digested that path;
- a write that replaces a file changes its inode,
which D2 does not compare,
but the replacement's change time and often its size differ,
and a replacement inside the same tick is D4's case too;
- a staged change or a commit in the window moves the index,
which D1 catches before D2 runs;
- a write whose bytes and every compared field are equal after the capture is D4's case for the racy set,
and one before the capture that leaves every field equal is the settle's case under V4's adopted rule (L4).

The proof is not a claim about ignored files or untracked files.
Those are carried exactly as the snapshot carried them,
which is the rung's purpose,
and the restore's own proof is what proves the finished tree.

### The restore

The restore turns the clone into Git's own checkout of the commit the caller asked for,
in nine steps.
The order is this revision's answer to H2:
nothing under a real name is published before the finished tree is proven.

1. Verify the mount and the caller's visibility,
which is the add's step 4 and not this design's.
2. Read the clone's own index bytes with `read_own_index` (F28) and run D1 to D4 before any write (see "The frozen proof").
3. Publish the proof's phase marker `<slot>.capture.passed` beside the record,
so a resume after the first write does not have to prove a tree that has already been written (L6,
"What a resume finds").
4. Remove the clone-local leave-outs with the btrfs rung's own rule:
the copies of other adds' trash,
the copies of nested worktree stubs,
and the destination's own slot directory.
Each is a removal inside the clone's own dataset.
A stub that holds more than an empty directory or Git's pointer file is a refusal that ends the rung,
because the snapshot may have carried the underlay's real files (F18) and the design removes only the copies it can name (`doc/planning/cli-git-zfs-worktree-design-proposal.md:403` to `:404`).
5. Write the captured index under the private name `index.pending` in the new worktree's administrative directory,
with a lookup that answers nothing for every path,
so each entry ends with these fields and no others:
the path,
the mode,
the object name,
and the flags and stage from the captured index;
the change time,
the modification time,
the device,
the inode,
the owner,
the group and the size as that same entry recorded them;
zeroes in every stat field of a path the conversion fence hands to Git;
the `UNTR` and `FSMN` extensions dropped;
and a recomputed checksum.
No field is taken from any file,
which is `SPEC.md:1289` to `:1293` and the reason H1 was raised.
The seed is the bytes step 2 read and D1 compared,
not a second read of the copied index (N11),
and it passes through `with_zeroed_stat` with the fence's paths before the write,
as `seed_without_stat` does (F8).
No index exists under the worktree's own name yet.
The entries of the paths the reset or the fence's write rewrite are Git's own records once those runs have written them,
because Git writes an entry's stat data when it writes the file;
the fields above are what the private write holds before them.
If [M2][m2] shows that a clone's inode numbers are not the snapshot's,
the install would need a field mask the wrapper does not have:
a rewrite that writes the clone's own device and inode into an entry only when its recorded size,
modification time and change time already equal the file's,
which the proof established,
and never its change time.
That is a change to the shared rewrite,
and [M2][m2]'s answer is what would put it to the project.
6. Run the btrfs rung's own `restore_dirty` with the clone's root as its stage and with the private index:
list the ignored paths,
run `git clean --force -d` (untracked files that no ignore rule covers go,
ignored files stay),
settle by content the entries the private index cannot vouch for,
run `git read-tree --reset -u <the commit the caller asked for>`,
and prove the result with `prove_restored`.
Three sites in that function name the real index and each changes to the private one (F41):
`ask`'s `checkout_environment`,
which the proof's runs and the ignored listing use;
`restore_dirty`'s own environment for the clean and the reset;
and `remove_the_files_the_reset_cannot_see`'s read of `<admin>/index` and its mtime,
which is the settle's index seconds.
Changing the environment alone leaves two of the three reading a name that does not exist,
which is what N2 found.
The clean at the clone's root meets the walk-and-clean record's sentence:
the clone holds no mount other than itself,
a nested dataset's copy is an empty directory or the underlay's plain files (F18),
the wrapper's trash place and `.claude` under it are ignored so the clean never enters them,
step 4 has removed the nested worktree stubs,
and a stub it could not remove ended the rung before the clean.
7. Write the fence's paths through Git with `git checkout-index --force -u --stdin` (F5) against the same private index,
so no path is recorded clean while it holds the source's unconverted bytes.
8. Check the paths the reset or the fence's write wrote,
and refuse if one of them changed under the add:
a command run in the slot through the copied `.git` can write into the tree while these steps run (L8,
F40),
and Git's own check would not see a same-second rewrite of the same length.
The set is every entry of the private index whose bytes differ from the captured seed's entry for that path,
object name,
mode or stat data,
which the add holds in hand from step 2 and step 5,
so no Git run is needed to know it.
For each of those paths the check compares the file's own facts with the entry Git recorded when it wrote it,
to the nanosecond,
by the wrapper's strict comparison (`matches_stored_exactly`,
F30):
the size,
inode,
owner,
group and both times must be the file's own,
and a difference refuses and names the path.
Content is read only where the stat cannot decide and can decide something:
for an entry whose recorded sub-second count is zero,
and for an entry recorded inside one tick of the private index's own modification time,
the file is read and compared with its blob when the content judge can decide,
which is every path no conversion reaches and which holds no carriage return (`worktree_clean_content.rs`,
`agrees`);
where the judge answers that a conversion may change the bytes,
the strict comparison alone stands and the journal says the path was not compared by content.
A by-content check alone cannot serve here:
the judge refuses the paths the fence hands to Git,
which are 1,578 of this repository's 24,426 paths in the fence's own measurement (`doc/planning/cli-git-clone-rung-conversion-fence.md:913`),
so a content check would refuse almost every add here or skip exactly the paths it exists for (R1).
T12 holds the controls that tell the three answers apart.
Then publish,
and the slot never lacks a `.git` at any instant:
publish the private index under the real name with one rename that replaces nothing (F8);
write the saved pointer bytes to the trash's next entry,
`<clone>/.claude/cli-git-trash/<trash>/d<index>`,
where `<trash>` is the forty-hexadecimal-digit trash directory the leave-out made (or step 4 made for this) and `d<index>` is the name the trash gives its entries (`worktree_trash_manifest.rs`,
`entry_name`),
which the wrapper's exclude line already keeps out of the index;
write that entry's identity into the trash's own `.manifest` before the exchange,
taking it from `<clone>/.git` as it stands,
so the disposal can prove the copy is the wrapper's and a kill after the exchange orphans nothing (R3);
exchange the entry with `<clone>/.git` in one `renameat2(RENAME_EXCHANGE)` (F36,
F37),
so the pointer takes `.git`'s name and the copied administration lands at the entry's name in the same instant,
still the identity the manifest records;
and leave the copied administration there,
because the trash's manifest names it and the disposal empties it after the add returns (F32,
A4).
A new function sets an entry aside by exchange,
beside `take_entry`,
which only renames into the trash (`worktree_snapshot_placeholders.rs`).
Then verify the worktree root,
the common directory,
the reciprocal registration,
the mount identity and the checked-out state.

What a kill leaves at each point of step 8:
before the private name is written,
the slot holds the copied `.git` and the published index,
and a resume writes the name and exchanges;
after the private name is written and before the exchange,
the slot holds the copied `.git` and the trash name holds a small file of pointer bytes,
which a resume overwrites because the trash is the wrapper's own;
at the exchange,
one system call leaves one of the two states and never a mix,
so the slot holds either the copied directory or the saved pointer;
after the exchange,
the slot holds the saved pointer with the published index,
and the copied administration is inside the trash and named by its manifest,
which a resume leaves to the disposal.
A slot with no `.git` at all is not a state these steps leave,
and a resume that finds one takes the fallback,
unless the clone's identity can no longer be proven,
and then it leaves the pending state (L6).
Publishing the index first is what makes every state after the exchange a worktree whose index is whole,
and a resume publishes the private index only while `index.pending` stands:
the private name's absence with the real name present means the rename happened,
because nothing else removes the private name.

9. Mark the binding usable,
queue the dependency repair,
hand the trash to the disposal,
remove the capture record,
and print Git's line,
the line about the left-out changes,
and nothing else.

Why this order:
nothing under a real name is published before the finished tree is proven (H2),
and the slot never lacks a `.git` ("The project's answers of 2026-11-22").
Until step 8 the slot holds the copied `.git` directory,
so a `git status` there answers from a mirror of the main checkout at the snapshot,
and a mutating command acts through the copy (L8):
`git add -A` and `git commit --all` land in the copy's index and refs and are lost with it;
`git restore .`,
`git checkout .`,
`git stash` and `git reset --hard` write the copy's HEAD bytes into the tree being restored and the writes stay,
which the content check at the head of step 8 refuses;
`git worktree remove` acts on the 187 worktree folders the copy names,
and `git push` reaches the remote,
neither of which the design closes.
None of them writes the main checkout's own files,
its index or its references.
At the exchange the slot holds either the copied directory or the saved pointer,
never neither,
and after it the slot is a registered worktree with the finished index.
The state the first text had,
a valid pointer over an unrestored tree,
does not exist at any step.
Making the copy inert was put to the project and not chosen;
"Limitations" states what it would have cost.

What the restore guarantees,
and its exceptions,
are listed under "Where the result differs from Git's own checkout".

### What it reuses, and where ZFS differs from btrfs

Reused whole,
without a change:

- the refusal set for a dirty source (`SPEC.md:1264` to `:1282`),
re-run against the frozen capture;
- the restore's three runs,
its content settle and its proof (`restore_dirty`,
`prove_restored`,
F27);
- the copy's own index read (`read_own_index`,
F28);
- the leave-out rule and the trash of the snapshot rung (`prune_snapshot`,
`remove_stub`,
F32),
and the disposal that empties a trash after the command returns,
with the exchange (F36) taking the place `replace_git_pointer` has there;
- the conversion fence,
decided before the snapshot and written through Git after the reset,
with the index's zeroed stat data for the fenced paths (F5,
F28);
- the clone rung's failure sentence and the fallback that Git's own checkout completes (F33);
- the index's private name and its publication (F8).

Changed rather than reused:

- `restore_dirty`'s three real-name index sites take the private index (F41):
`ask`'s environment,
which the proof's runs and the ignored listing use,
and which `checkout_environment_with_index` already builds (F9);
`restore_dirty`'s own environment for the clean and the reset;
and the settle's read of `<admin>/index` and its mtime.
A `Target` that carries the index path is what all three read,
and T6b is the control that goes red if any of them reads the real name.
- The resume clears the waiting index and its lock with `clear_the_waiting_index` (F42),
before step 5 is re-run,
because `write_index_under_a_name` refuses an existing private name and a Git run killed against it leaves `index.pending.lock` beside it (F8,
F42).
- The pointer's install is `exchange_paths` or `exchange_at` (F36),
not `replace_git_pointer`'s in-place write,
because the slot must never lack a `.git` and the two names are on one dataset.
- The index install is `write_index_under_a_name` (F8) and not `install_index`,
because the private name is what H2 needs.
The rule the dirty form already follows (every entry keeps the fields its own index recorded,
F28) is the rule this write follows,
with a lookup that answers nothing for every path.
`install_index` is left as it is and is not the function this rung calls.
- No device-and-inode rewrite exists and none is added;
[M2][m2] decides whether one is needed,
and "Choices this design marks for the project's veto" carries the contingency.
- The settle's racy test,
which V4's adopted rule adds,
needs two inputs the restore does not have today:
the capture's index modification time to the nanosecond,
which C1 records and which the settle's `index_seconds` carries only as whole seconds (F41),
and the tick,
`clock_getres(CLOCK_REALTIME_COARSE)` (F38).
`paths_whose_content_differs` skips every stat-exact entry and has no racy test of its own (F30),
so the change is a new predicate in the settle,
not a re-use of one,
and T13 is its control (R6).
- `clear_the_waiting_index` becomes callable from the add's own settlement:
it is private today (F42),
and naming it as the reuse is not enough (R7).

Where ZFS differs,
and what the design does about each:

- There is no stage,
so the fill is visible while it runs and the binding,
not the mount,
says whether the worktree is usable;
the project's answer of 2026-11-22 keeps the copy and swaps it at the end ("The project's answers of 2026-11-22").
- The copied administration is on another dataset from the source's trash,
so the exchange and the trash name are inside the clone's own dataset and the disposal reaches the copy through the published worktree path.
- The inode numbers are shared with the snapshot (F10,
F12) and the device is not;
the proof does not compare the device,
and the index keeps the source's inode until [M2][m2] shows it differs.
- A snapshot holds an empty directory where a nested dataset is mounted in the source,
or the underlay's real files (F18),
so the leave-out refuses a stub it cannot name rather than deleting what it did not make.
- The change time is a coarse tick and Git compares seconds (F2,
F15,
F22,
F23),
so the proof digests the racy set (C4) and the settle's own trust is weaker than btrfs's (F21,
F30,
V4).
- The clone's creation is two delegated requests whose GUIDs and `createtxg` values are recorded,
and an origin snapshot cannot be destroyed while its clone lives (F13),
which fixes the order of any cleanup.
- A ZFS snapshot copies nothing,
so the fence's reads sit on the add's waiting path,
as in the btrfs snapshot rung.

### What happens when the proof fails or the restore cannot finish

The project's answer of 2026-11-22 settles this section:
"at every step,
the add destroys only what this add made and recorded (the clone,
then its snapshot),
runs Git's own checkout at the same slot,
and says which route made the worktree and why the first was abandoned" (`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md:35` to `:42`).
The record states what it amends:
the fallback "is not silent,
since the add reports it,
and it finishes only after the mount and the add's own intents are settled by the add itself" (`:50` to `:53`).
So there is one route from every failure after the claim,
and a failure before the claim has nothing to settle.
The add begins by publishing a settle marker,
`<slot>.settle`,
beside the capture record,
with an exclusive staging name and one link to its real name that replaces nothing,
holding the kind of refusal and the instant,
so a resume continues the settle rather than re-entering the fill (R13).

What the add does,
in order:

- it stops writing and reads the state it must settle:
the clone and the origin snapshot its record names,
the mount,
and its own outstanding intents;
- it closes every handle it holds under the clone before it asks for the unmount:
the proof's held root,
the directory chains it opened,
the private index it held open,
and it waits for every Git child it started,
because the add is itself a holder of that mount until then (R5);
- it asks the handler for that clone's unmount and waits for the receipt within the step's bound,
which is the intent the account writes on the clone and the receipt the handler publishes (proposal `:554` to `:579`);
- it re-proves from its own record what it is about to destroy:
the clone's dataset GUID and `createtxg`,
and the origin snapshot's GUID and `createtxg`;
- it destroys that clone with one delegated `zfs destroy` without `-R`,
which refuses a name that holds anything the record does not;
- it destroys the origin snapshot,
and treats `ENOENT` as done:
a mark another same-account process set on that snapshot reaps it in the same transaction as the clone's destroy,
so the snapshot is not there any more,
and the deferred-destroy design requires that the add's fallback treat that one `ENOENT` as done (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
"An add's own destroy does not pass through these checks").
It writes no destruction plan,
no removal intent and no stage,
and it calls none of that design's checks,
which say they do not run for an add.
Any other answer to either destroy is a failed settle;
- it releases the managed Git lock and keeps the registration:
one `unlinkat` of the `locked` name through the held registration descriptor and one sync of the registration directory,
which is the removal's release step alone and none of its emptying or removal (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
"How the wrapper's own removal releases the lock",
step 4),
because the next step needs the registration;
- it runs Git's own checkout at the same slot,
which is `git reset --hard --no-recurse-submodules` under the verified binding (F33),
against the registration it kept and the underlay pointer the mount had covered (F7);
- it closes the binding and releases the claim,
which the binding record design must allow for an add that finishes with Git's own checkout (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
V16's A keeps them only for a resume);
- it prints one sentence naming the route that made the worktree and why the first was abandoned,
with the proof's reason in it,
and removes the capture record,
its phase marker and the settle marker.

A refusal the preflight makes leaves no managed object,
and the rung ends as the btrfs rung's does.

The trash needs no disposal here:
it is inside the clone,
so the clone's destroy takes it and its manifest with it.

What the add calls,
and what each design must expose for it:

- the handler's unmount request and receipt,
for the one clone the record identifies by dataset GUID and `createtxg`:
the account-written desired-state intent and the root-written receipt the accepted design states,
which nothing yet restricts to a removal,
so the staging design and the cancelled-removal design must say an add may use them (R5);
- the delegated `zfs destroy` of a clone and of a snapshot,
which the route's grants already carry (the design's "Scoped delegation"),
so no design has to expose a call for it;
- an unlock that keeps the registration:
the lock design must expose the release of `locked`,
with the registration's directory synced and everything else left,
callable by an add under its claim;
- the binding's close and the claim's release for an add that finished with Git's own checkout:
the binding record design must allow them,
where its V16's A keeps both for a resume.

No sibling design offers the last two today,
and "Dependencies on the sibling designs" says what each must add.

When the settle itself fails,
the add keeps what it made,
writes its record's pending state,
prints the failure sentence with the removal command from the wrapper's existing unfinished-worktree report,
and changes nothing else;
a person leaves the folder,
runs the removal,
and adds again.
Its unmount intent is then the question Q-R5 puts to the project.

A clone or snapshot whose identity cannot be re-proven is not destroyed by the add,
because destroying an object whose identity was not proven is what the removal proof forbids.
`doctor`'s unaccounted origin line does not report such a snapshot:
that line keeps only a snapshot with no clone,
no record and no live binding (`0973f2fc2a`,
`:2129`),
and a pending add keeps its binding.
The binding record design's own lines report a registration without a dataset,
and that is where a person reads it (R14).

The lines the person reads,
in the wrapper's voice,
and what the journal keeps:

- one sentence for both routes,
which is the snapshot rung's own line with the route named,
not the clone rung's (`SPEC.md:1911` to `:1920`,
"this worktree was not made as a snapshot,
so it took longer,
because "):
`monochromatic-git-policy-cli: this worktree was made by Git's own checkout, so it took longer, because <reason>; the next add makes a clone once <what makes the next add quick>.` It names no internal step and gives the cause in the reason's own words (R10).
- the reason's own words name the path and the field,
for example `the frozen proof refused AGENTS.md: its change time changed in the main checkout while the add read it`;
- for a settle that failed,
the unfinished worktree's sentence with the removal command;
- on success,
the line Git's own checkout prints and the line about the left-out changes (F33),
and nothing about the proof.

The journal keeps one entry with the destination,
the rung,
the route that made the worktree,
the kind of refusal,
the path,
the field and the capture record's name,
in the shape the left-out note already has (`CLI_GIT_WORKTREE`,
`CLI_GIT_DIRTY_PATHS`,
`CLI_GIT_DIRTY_FIRST`,
`worktree_dirty_restore.rs:104` to `:113`),
so a later reading can tell which field of which path refused without the person's screen.

### What a resume finds

The restore is idempotent from step 4 on:
the leave-out removes what is there,
`git clean --force -d` removes what is untracked,
the settle reads and removes,
`git read-tree --reset -u` rewrites what differs,
and the fence's write rewrites.
Three things are not idempotent by themselves and a resume handles each first:
`index.pending` and `index.pending.lock`,
which it clears with the wrapper's own `clear_the_waiting_index`,
and the real-name index,
which it never publishes again.
So a resume reads the evidence on disk and the three markers:

- the phase marker `<slot>.capture.passed`,
which step 3 publishes and which says the tree may already hold written bytes;
- the settle marker `<slot>.settle`,
which the fallback publishes before its first destroy and which says the add was settling (R13);
- the private and real index names,
which say how far step 8 got.

`clear_the_waiting_index` is a private function today (F42),
so the build makes it callable from the add's own settlement,
which "What it reuses,
and where ZFS differs from btrfs" names (R7).

Each state is told from that evidence:

- a claim and no registration:
the add has not run Git;
the same add resumes,
or the removal clears the claim;
- the settle marker:
the add was settling;
the resume continues the settle from its own steps,
destroying by the recorded identity whatever of the clone and the snapshot still stands,
treating `ENOENT` on the snapshot as done,
releasing the lock and keeping the registration,
closing the binding,
releasing the claim,
and running Git's own checkout,
so it never re-enters the fill (R13);
- a registration and no clone,
with no settle marker:
the snapshot or clone did not happen;
a fresh capture is taken with its whole refusal set before any new snapshot,
because the old record describes a checkout that has since moved,
and the add finishes with Git's own checkout when the capture cannot be taken;
- a clone and no mount:
the clone's identity is checked against the binding and the mount is requested again;
the proof runs after the mount arrives,
from the capture record,
so a source that moved since the snapshot does not fail it;
- a mounted clone with the copied `.git` and no phase marker:
the proof runs from the capture record and the restore then runs whole;
- a mounted clone with the copied `.git`,
the phase marker and no real-name index:
the waiting index and its lock are cleared,
and the restore runs whole without the proof,
because the tree may already hold written bytes and a second proof would refuse the add's own work;
- a mounted clone with the copied `.git` and the real-name index:
the tree was restored and checked,
and step 8 stopped before the exchange;
the resume writes the pointer's entry and exchanges it,
then verifies the worktree and finishes step 9,
which is one rename from done and never a second restore (R4);
- a mounted clone with the saved pointer in `.git`,
the copied administration in the trash and the real-name index:
step 8 happened;
the resume verifies the worktree and finishes step 9;
- a mounted clone with the saved pointer in `.git` and no real-name index:
the exchange happened and the publication did not,
which step 8's order cannot leave,
so the resume treats it as damage and takes the fallback;
- a slot with no `.git` at all:
no step of this design leaves it,
so the resume takes the fallback too;
- a clone whose identity the record no longer proves,
or whose unmount and destruction cannot be confirmed:
the add does not destroy what it cannot prove,
so it leaves the pending state and names the removal (L6);
- an index under the real name and a pointer,
with the binding not usable:
step 8's verification runs again,
and on success the binding is marked usable without a second restore;
- a binding marked usable:
nothing is resumed;
it is a worktree.

The capture record's end follows the states:
it stays while any row above is still reachable,
and it is removed on the success of step 9,
when the add's fallback finishes the worktree with Git's own checkout,
or when the add leaves a state in which the proof can no longer pass and the clone is settled.
Its phase marker and the settle marker are removed with it.
A `doctor` line reports a record whose add left none of those states,
and a staging name a killed writer left.

### The cost, against a clean add and against rung 3

No number here is claimed unmeasured,
and each owed measurement names the arm it must be compared against.

- The capture and the proof against a clean add.
[M1][m1] runs three arms in rotating order over this repository's paths:
a dirty source added on rung 1,
a clean source added on rung 1,
and the same adds on a build that never takes rung 1,
which is the identical-binary control.
A fourth arm,
built with the crate's existing `measurement-routes` feature and run after the three,
keeps the capture and skips the proof,
so the proof's own share is visible;
it is never a shipped switch.
Each add's own step timers report the capture,
the walk,
the digests,
the compare,
the two untracked listings and the post-restore check separately,
because that check costs one `lstat` for each path the reset or the fence wrote,
which on a restore to a commit far from the source's `HEAD` is every path of the tree (R1).
It reports each arm's median and range and the ratio of the dirty arm to the clean arm and of both to the control.
No figure is quoted here:
the only numbers in hand are F25's script timings and the rung 2 record's walk (A5),
and neither is this rung's cost.
- The copied administration against inline deletion.
[M3][m3] measures the add with the trash hand-over (F32,
A4) against a control arm that deletes the copied `.git` in place,
on a fixture whose administration holds the files F26 measures,
and reports both ratios and their spread.
- The restore against Git's own checkout.
[M4][m4] measures adds from sources with 5,
300 and 6,000 changed tracked paths,
alternating arms with an identical-binary control,
and reports the ratio to Git's own checkout at each shape.
Whether rung 1 should decline a very dirty source as rung 2 does is Q-B and not a bar this design sets.
- The fallback after a refused proof.
[M5][m5] measures the settled add the project's answer names:
an add whose proof is forced to refuse at T1's seam,
timed from the command's start to the end of Git's own checkout at the same slot,
against the same fixture added on a build that never takes rung 1,
with its separate arm for a settle that fails and leaves the pending state.
- The exchange.
[M9][m9] is the guest run the record owes:
the exchange with what a kill before,
at and after it leaves,
and the exchange's own cost against the two-rename sequence the first text had.
- The refusal rate on a busy checkout.
[M6][m6] measures how often a proof refuses when other sessions write the checkout during the add,
at the window this design leaves,
and how often the main checkout's own `git status` refuses it through D1 (V8).

### The tests a build must have

Each race is forced at a seam the code already offers or gains,
never by timing,
and every refusal test carries its no-write control,
because a test that cannot pass on an unchanged window proves nothing.

- T1.
`the_frozen_proof_refuses_a_tracked_write_between_the_capture_and_the_snapshot`.
The rung's snapshot step gains a seam between the capture and the snapshot request,
in the shape `PlaceSeams` already has (`worktree_snapshot.rs`),
and the control writes a tracked file in it.
The stand-in snapshot is the fixture tree itself,
not a copy:
a copy cannot carry the source's change time,
so a copying stand-in would refuse every file for the wrong reason (M5).
The no-write control asserts the unchanged fixture passes the proof.
Assert the refusal,
the path and the field it names,
and the failure sentence.
Runs on the host.
- T2.
`the_frozen_proof_refuses_an_index_written_between_the_capture_and_the_snapshot`.
The same seam,
with a control that runs `git add` in the fixture.
Assert D1 refuses,
names the index,
and the restore does not run.
Its no-write control is T1's.
- T3.
`a_write_in_the_file's_own_tick_is_refused_by_the_racy_set`.
Two things are forced,
both on observed state and neither by a sleep.
The entry is put in the racy set on purpose:
the fixture `utimensat`s the index file's modification time to a moment inside one tick after the file's recorded time,
which is what C4's rule reads.
The fixture writes its own index entry in process right after its first write,
because a `git add` takes milliseconds and the stamps would never meet otherwise,
and the entry records the first write's stamp (R8).
The window write is put in the file's own tick by a bounded retry:
the control writes the file,
reads its change time and the entry it wrote,
and repeats a fixed small number of times until the two stamps are equal,
declaring the case inconclusive rather than sleeping;
it writes the same length.
Assert D4 refuses through C4's digest and names the path.
The arm that must be told apart from it:
a write after the tick moved is refused by D2 and not by D4.
The no-write control is T1's.
- T3b.
`a_same_tick_rewrite_of_a_vouched_path_before_the_capture_is_settled_by_content`.
The same two forcings,
with the second write made before the capture takes its facts,
so the capture reads the rewritten bytes and D4 cannot help.
With V4's adopted rule the settle reads the entry by content,
finds bytes its own index entry does not name,
and the reset writes the target commit's bytes,
which the test asserts.
Before that rule the same fixture asserts the opposite,
that the clone carries the second write's bytes,
and the test states both so the flip is never silent (L4,
V4).
- T12.
`the_post_restore_check_refuses_a_write_after_the_restore_and_accepts_an_unchanged_converted_path`.
Three arms,
all in the guest,
because a conversion needs Git's own checkout:
a fixture whose `text` rule converts one path,
restored and then written again in a later tick through the copy,
which the strict stat comparison refuses and names;
the same fixture untouched,
which the check passes and which a by-content check would have refused,
so the arm that shows why the check is not by content;
and an unconverted written path rewritten inside one tick,
which the content read refuses because the judge can decide there.
The journal's line says which of the three answers each path met.
Its control is the unchanged fixture (R1).
- T13.
`the_settle_reads_by_content_only_the_entries_in_the_racy_set`.
A fixture whose index modification time is put inside one tick of one entry's recorded time by `utimensat`,
and one entry older than that:
assert the settle reads the first by content,
takes the second's stat as its own,
and rewrites the first when its bytes differ from its blob.
Its control is the same fixture with both entries older than a tick,
where neither is read (V4,
R6).
- T4.
`the_capture_digests_only_the_unvouched_and_racy_entries`.
A fixture with one edited path,
one path written in the current tick and many untouched ones;
assert one digest per set member,
and that a control which touches every file records every path.
This is the test that keeps the capture's cost from drifting to F25's digest arm.
- T5.
`a_resumed_add_takes_the_verdict_of_its_row`.
One case per row of "What a resume finds":
kill the add at that step's boundary through the seam the step has,
resume,
and assert the row's verdict and the capture record's presence or absence.
The kill points are the claim,
the registration,
the snapshot,
the clone,
the mount,
the phase marker,
each restore step,
the waiting index,
the index's publication,
the pointer's private name,
the exchange,
and the trash's leftover.
- T6.
`the_index_the_clone_installs_takes_no_field_from_a_file`.
After step 5,
assert for one edited path that the private index's change time,
modification time,
size,
device and inode equal the captured index entry's,
not the file's;
and that a same-size edit whose modification time was put back is still rewritten by the reset.
The test is red if the install copies a change time,
which is H1's control.
- T6b.
`the_restore_reads_the_private_index_and_never_the_real_name`.
Three arms,
one per site of F41:
no index under the real name at all,
a decoy index under the real name whose entries name other blobs,
and a real-name index whose modification time differs from the private one's by more than a tick.
Each asserts the restore's verdict is the private index's,
and the test goes red if `ask`'s environment,
`restore_dirty`'s environment or the settle's read names the real file.
Its control is the ordinary restore,
which passes all three.
- T7.
`an_untracked_repository_in_the_source_is_refused_before_the_snapshot`.
A fixture whose source holds an untracked,
not ignored directory with its own repository:
assert the preflight refusal,
no managed object,
and Git's own checkout.
Its control is the same fixture without the nested repository,
which the rung serves.
V7 is the choice this test rests on.
- T8.
`a_converting_path_in_a_dirty_clone_is_written_by_git`,
in the guest,
over the four classes issue 6323 named:
a `text` rule,
`core.autocrlf`,
`ident`,
and `text=auto eol=lf` over a CRLF file,
each restored to a commit other than the source's `HEAD`,
compared with a real Git checkout of that commit.
- T9.
`the_clone_restores_a_dirty_source_to_the_commit_the_caller_asked_for`,
in the guest:
one staged edit,
one unstaged edit,
one deletion,
one mode change,
one untracked file that is not ignored,
one ignored file,
one symbolic link,
and one tracked path whose kind is refused.
Assert the served tree equals Git's own checkout plus the ignored file,
and the person reads the left-out line with the count.
- T10.
`no_moment_or_kill_leaves_the_slot_without_a_dot_git`,
in the guest,
because the exchange is the filesystem's:
kill before the pointer's private name,
after it,
at the exchange and after the exchange before the trash is left,
and assert for each leftover that `<clone>/.git` is either the copied directory or the saved pointer,
that a slot with no `.git` at all never appears,
and that the next add's verdict matches T5's rows.
The exchange's own arm is [M9][m9].
- T11.
`a_refused_proof_settles_the_add_and_gits_own_checkout_makes_the_worktree`,
in the guest:
the T1 refusal at the mount,
the unmount,
the clone's destruction and then the snapshot's,
the printed sentence naming the route,
and Git's own checkout at the same slot.
Its control is the same fixture whose proof passes;
its second control forces the settle to fail and asserts the pending state,
the removal command and nothing destroyed.

One control is run beside T1 to T4,
`the_unchanged_window_passes_the_proof`,
because a proof that refused an unchanged fixture would make every refusal test meaningless.

Which need a guest:
T8,
T9,
T10,
T11 and the mount-dependent half of T5,
and the exchange control inside T10,
which is [M9][m9].
T1 to T4,
T6,
T6b and T7 run on the host,
over the fixture tree itself.

### Where the result differs from Git's own checkout

These are the cases the design knows,
each with its evidence and what the design does:

- An untracked,
not ignored directory holding its own repository stays after `git clean --force -d` (F4,
F24),
and the restore's proof then names it as untracked and refuses.
On btrfs that costs a destroyed stage;
on ZFS,
without a preflight refusal,
it would happen after the snapshot,
the clone and the mount.
The design refuses it before the snapshot,
from the untracked listing the preflight already makes (`SPEC.md:1381`),
with no managed object (V7).
- The modification times of unchanged files and the directories' modes are the source's,
not the checkout's time and umask;
`git status` is unaffected and build tools see older times.
The bytes are equal.
- The ignored state is the snapshot's instant (`SPEC.md:1518`),
so a path the target commit stops ignoring is carried and listed as untracked (`SPEC.md:1335` to `:1342`).
The proof counts it as carried.
- A tracked file whose recorded stat data is the entry's own to the nanosecond can keep the source's bytes if the source was written in the same tick as the entry (L4,
F30).
V4's adopted rule reads it by content and prevents that,
and a veto of V4 restores the residual,
which the design then states as its one silent byte.
- A split or sparse index cannot be installed at all (F34),
so the capture refuses it from its extension signatures before anything is written,
and the add takes Git's own checkout with no managed object.
- Issue 6323's conversion class is the fence's,
and the fence applies to rung 1;
T8's guest controls are what a dirty restore to a non-`HEAD` commit owes.
- A hidden trash holding a copy of the checkout's administration stays in the worktree until the disposal runs (V5).

## Which build step builds which part

- Plan step 10 builds the add's route from registration to verified fill,
and it builds the frozen proof for every rung 1 add,
clean ones included:
C1,
C2 and C5's commit,
D1 to D4,
the private write of the index and its publication,
the copy's exchange with the pointer,
and the resume states.
For a clean source C3 and C4 are empty or nearly so,
and "neither route accepts an unproved source" is met by the same proof.
This clarifies the plan's two step notes and needs its test sentence amended,
which is veto entry V6.
- Plan step 11 builds the dirty half:
the full refusal set re-run against the frozen capture,
C3 and C4's digests,
the restore's wiring to `restore_dirty` at its three index sites (F41,
T6b),
the settle's racy rule (V4),
the fallback and its pending state,
T1 to T7,
T6b and the guest tests T8 to T11.
- The steps a failed add calls to settle itself,
the unmount,
the two destructions and the Git settle,
belong to the removal's build (steps 14 to 17),
because no sibling design offers them to an add,
and "Dependencies on the sibling designs" states the requirement each of those designs must meet.
- `SPEC.md` gains,
under the add section and beside the btrfs rung's dirty paragraph:
what a clone of a dirty main checkout restores,
what the frozen proof compares (with the racy set),
the refusal when it differs,
the private-name index and the pointer's publication,
the resume states,
and the one failure sentence with its reason.

## Measurements owed

Each is a probe a later round runs as written,
with its control arm,
its rounds and the spread reported (rules `QNB` and `NQH`),
and each says where it runs.

- M1.
The capture and the proof's cost,
with a clean-add arm.
Three arms in rotating order,
ten rounds each:
a clean source on rung 1,
a dirty source on rung 1,
and the same adds on a build that never takes rung 1,
which is the identical-binary control.
A fourth arm,
built with the crate's `measurement-routes` feature and run after the others,
skips the proof so the proof's own share is visible;
it is never shipped.
Each add's own step timers report the capture,
the walk,
the digests and the compare separately.
Report each arm's median and range,
the ratio of the dirty arm to the clean arm,
and both to the control,
with the count of rounds where rung 1 was slower.
Runs on the host.
- M2.
The clone's metadata identity.
In the guest,
on a file-backed pool:
a fixture of 2,000 paths with mixed kinds,
sizes,
modes and one symbolic link;
snapshot,
clone,
mount,
and compare inode,
size,
kind,
mode and both times to the nanosecond for every path between the mount's root and the source;
report every path that differs and how.
The positive control writes one file in the window and asserts the comparison reports exactly that path,
and the reviewer's tick probe (F22) is repeated inside the guest,
because the tick is the kernel's and a guest's need not be one millisecond.
- M3.
The copied administration's hand-over.
In the guest:
an add whose clone carries an administration of 634,426 files in 6,708 directories,
one arm handing it to the clone-local trash and one arm deleting it in place,
alternating with the identical-binary control,
ten rounds;
report each arm's median and range and the disposal's cost separately,
from the journal's own timestamps.
- M4.
The restore against Git's own checkout.
In the guest:
sources with 5,
300 and 6,000 changed tracked paths,
each added with rung 1 and with rung 3,
alternating arms with the identical-binary control,
ten rounds a shape;
report the ratio for each shape with its range.
A shape where rung 1 is slower than rung 3 is material for Q-B.
- M5.
The fallback's cost.
In the guest:
an add whose proof is forced to refuse at the T1 seam,
timed from the command's start to its end,
against the same fixture added on a build that never takes rung 1.
Report the ratio and the range,
with a second arm for a settle that fails and leaves the pending state.
- M6.
The refusal rate on a busy checkout.
On this host:
100 adds from the main checkout while other sessions write it,
with the capture immediately before the snapshot,
and 100 with the capture at preflight;
report how many of each refused,
which field refused,
and how many refused through D1 because a `git status` in the main checkout refreshed the index.
Nothing here generates load:
the adds are real adds,
and the sessions writing the checkout are whichever sessions are there (rule `NQH`).
- M7.
The racy set's size.
On this host,
at ten moments of a working day:
read the main checkout's index,
list the entries whose recorded modification time or change time is not at least one tick older than the index's own modification time,
and report the count and the sizes of those entries.
This is the cost C4 adds and the cost V4's adopted rule pays in reads,
and it is the number that decides that rule's veto if it is large.
- M8.
The settle's exposure on ZFS.
In the guest:
a fixture whose tracked file is written twice inside one tick,
with the index written before the second write;
assert whether the btrfs rung's settle believes the file and publishes the second write's bytes.
The positive control writes in a later tick,
which the settle must read.
This is the positive control for V4's adopted rule:
with it,
the settle reads the file and the served tree holds the target commit's bytes;
without it,
the served tree holds the second write's.
- M9.
The exchange,
in the guest,
on a file-backed pool:
snapshot,
clone,
mount,
write the pointer's private name inside the clone's trash place,
exchange it with `<clone>/.git` (F36,
F37),
and report that `<clone>/.git` is the saved pointer,
that the copied administration stands at the trash name,
and that the disposal empties it.
Kill before the private name,
after it,
at the exchange and after it,
and report what each kill leaves in the slot,
which is the run the project's answer of 2026-11-22 says is still owed (`:91` to `:94`).
Also time the exchange against the two-rename sequence the first text had,
alternating arms on one fixture.

## Choices this design marks for the project's veto

Each entry says what the design does,
which options it had,
a concrete example for each option and the result a person would see,
the pros and cons,
and the ranking with the reason for each adjacent pair.
A veto returns the entry to the writer;
taking none leaves the design as it stands.

- V1.
What the frozen capture holds.
The design records the index's bytes and digest,
every tracked path's facts,
a digest for the entries the stat data cannot vouch for,
and a digest for the entries in the racy set (C1 to C4).
  - Option (a),
this one:
an add from the main checkout as it stood on 2026-11-22 reads 3,436,101 bytes of index,
one `lstat` for each of 24,685 paths,
and the content of the 5 changed paths plus whichever entries the racy set holds,
which [M7][m7] counts.
  - Option (b),
a content digest of every tracked path:
the same add reads 764 MB and pays 4.9 to 9.4 s (F25),
and it refuses a same-tick write for every path rather than the racy set's.
  - Option (c),
the index's bytes alone:
the same add pays 1.5 ms and catches a concurrent `git add`,
but a session that edits `AGENTS.md` in the window is carried into the clone and published as Git's checkout.
  - Pros and cons:
(a) closes the window at a cost [M7][m7] measures,
and leaves only the pre-capture tick coincidence (L4);
(b) closes it for every path at nine seconds of the add's time;
(c) is nearly free and leaves the window the answer's own words call unproven.
  - Ranking:
a > b > c,
because a proves the tracked content the answer names at a measured small cost,
b buys a narrower residual for the add's whole time,
and c leaves the window unproven.

- V2.
Which fields the per-path comparison uses.
The design compares the kind,
size,
modification time,
change time,
the execute bit and a link's target,
and does not compare the device or the inode.
  - Option (a),
this one:
a session that edits `AGENTS.md` in the window moves its change time into a later tick,
and the add refuses naming `AGENTS.md` and its change time (F22,
F23).
  - Option (b),
the inode compared and the device not:
the same add refuses every path whose inode the clone did not share,
which [M2][m2] decides;
if the clone shares inodes as F12 says,
every add passes and the comparison is stronger.
  - Option (c),
the size and modification time only:
a session that rewrites a file,
keeps its length and restores its modification time is carried into the clone,
because the change time is not compared.
  - Pros and cons:
(a) rests on the times the kernel sets and compares no field whose sharing is a measurement,
and its equal-times case is the tick (L4),
covered by C4 and by V4's rule;
(b) adds a field whose sharing F12 and [M2][m2] make a measurement rather than a fact,
and it adds no protection against a same-tick write,
which only C4 and V4's rule catch;
(c) drops the compared field that catches a write in a later tick.
  - Ranking:
a > b > c,
because a needs no unmeasured sharing,
b depends on a sharing only [M2][m2] settles and buys nothing against the tick,
and c loses the field the kernel moves on every write.

- V3.
Where the frozen capture lives.
The design writes it in its own family folder under the wrapper's record parent,
`<common>/monochromatic-git-policy-cli/add-capture/v1/<slot>.capture`.
  - Option (a),
this one:
an add killed after the snapshot is resumed the next morning and proves the frozen clone even though the checkout moved meanwhile.
  - Option (b),
beside the claim as `<identifier>.capture` in the copy journal's folder:
the same resume works,
and the record sits where a sweep already lists names,
though the sweep reads only its own suffix (F35).
  - Option (c),
memory only:
the same resume cannot prove the clone,
so it settles it and finishes with Git's own checkout (L6).
  - Pros and cons:
(a) buys the resume,
follows the wrapper's own record convention for one family per folder,
and keeps the record out of the sweep's folder;
(b) is the same data beside records whose folder has a sweep that would have to learn its name;
(c) costs nothing and loses every resume.
  - Ranking:
a > b > c,
because a needs no change to the sweep and matches the convention the binding record sets out,
b works but widens a folder another design owns,
and c throws away the clone (a) proves.

- V4.
The settle's racy rule on ZFS.
The design adopts the rule its own ranking puts first:
the settle judges by content the entries in the racy set,
at the tick `clock_getres(CLOCK_REALTIME_COARSE)` reports (F38),
so a same-tick write before the capture is rewritten rather than believed.
The wrapper's rewrite already names the same idea at Git's second granularity (F29,
`subvolume_index.rs:520` to `:537`,
"an entry recorded in the same second as the new index"),
and this rule is that idea at the tick.
The racy set held one entry of 24,689 at one moment (F39),
which is this rule's cost on this repository,
and [M7][m7] counts it over a day.
  - Option (a),
adopted,
with the veto open:
the settle reads by content the racy entries and rewrites a same-tick write before the capture.
  - Option (b),
the first text's choice,
vetoed in effect:
nothing in the shared settle changes,
the residual L4 states stays,
and a same-tick write before the capture can publish that session's byte as clean,
which the decision's own words,
"neither route accepts an unproved source",
weigh against.
  - Option (c),
the capture digests every path and the settle judges by content every path it doubts:
the same closure at F25's 4.9 to 9.4 s.
  - Pros and cons:
(a) closes the silent byte at a measured count of files and a change to shared code;
(b) is free and leaves a byte no one can see;
(c) closes it at the add's whole time.
  - Ranking:
a > c > b,
because a closes the silent byte at a handful of reads,
c closes it at nine seconds,
and b leaves it.
A veto of (a) returns the residual to L4 and the shared settle to what it is,
and [M8][m8] measures what the residual can land.

- V5.
How the copied administration is discarded.
The design moves it into a trash inside the clone and lets the disposal empty it after the add returns,
as the btrfs rung does.
  - Option (a),
this one:
an add renames 634,426 files in one operation and returns with a hidden trash in the new worktree until the disposal has emptied it.
  - Option (b),
delete it in place:
the same add pays the unlinking of 634,426 files on its waiting path,
which [M3][m3] measures.
  - Pros and cons:
(a) costs one rename and reuses the mechanism the rung has (F32),
at the price of a copy of the checkout's administration living in the worktree until the disposal runs;
(b) leaves nothing behind and pays the whole deletion before the worktree is usable.
  - Ranking:
a > b,
because the btrfs rung already chose one rename over a deletion on the add's path (F32),
and the same account holds every byte of the copy.

- V6.
The plan's allocation of the work,
and its test sentence.
This design assigns the frozen proof to step 10,
because it is the proof of every rung 1 add,
and leaves step 11 the dirty restore and the fallback.
The plan is ambiguous rather than contrary:
step 10 already lists "the frozen-content comparison inside the clone,
the omission of the boundaries the design names,
and the discarded copied Git administration;
the pointer install,
the verified fill" (`doc/planning/cli-git-zfs-worktree-implementation-plan.md:1417` to `:1421`),
while step 11 speaks of "the frozen proof that the clone matches the captured source before fill" (`:1592`).
The project's answer of 2026-11-22 also amends the plan's test sentence in both places,
"a tracked write between preflight and snapshot that the frozen proof refuses" (`:1457` and `:1615`),
to a tracked write after the frozen capture.
  - Option (a),
this one:
a clean add on rung 1 is proved from the first build,
the plan's two steps keep their words except the amended test sentence,
and step 11 changes only the capture's dirty facts,
the restore and the fallback.
  - Option (b),
the plan's step 11 note read as the whole proof's home:
step 10 ships clean adds on rung 1 unproved,
and step 11 adds the proof for both,
which leaves "neither route accepts an unproved source" unmet between the two builds.
  - Pros and cons:
(a) meets the decision from the first build and needs two sentences of the plan changed;
(b) keeps both of the plan's step notes as they stand and leaves a window in which an unproved clone route is shipped.
  - Ranking:
a > b,
because the decision forbids an unproved source at any release and no record says step 10 may ship one.

- V7.
Refusing an untracked nested repository before the snapshot.
The design refuses such a source at preflight,
from the untracked listing it already makes,
and takes rung 3.
  - Option (a),
this one:
a source holding `package/odd-project/.git` is refused before anything is written,
and the person gets Git's own checkout,
which keeps the directory as it is.
A path under `.claude` would not serve as the example,
because the checkout ignores `.claude` and the listing never names it (F26).
  - Option (b),
let the restore meet it:
`git clean --force -d` keeps the directory (F4),
the proof names it,
and the add pays the snapshot,
the clone,
the mount and the settle,
then finishes with Git's own checkout.
  - Option (c),
give the clone the `clean`'s second `--force`:
the directory goes,
which the walk-and-clean record's sentence against nested-repository protection warns about (`doc/planning/cli-git-zfs-worktree-design-proposal.md:1078` to `:1079`).
  - Pros and cons:
(a) costs nothing and keeps the rung out of a case it cannot finish;
(b) needs no refusal code and wastes the whole add;
(c) finishes the add by deleting a repository the source holds.
  - Ranking:
a > b > c,
because a refuses before paying and names Git's own checkout,
b at least keeps the person's repository,
and c deletes what the record warns against.

- V8.
Comparing the copied index byte for byte.
D1 compares the clone's copied index with C1 as bytes.
  - Option (a),
this one:
a `git status` in the main checkout that refreshes one entry's stat data refuses the add,
which settles and finishes it with Git's own checkout;
[M6][m6] counts how often.
  - Option (b),
comparing only the entries that matter,
the paths in C2 and C3:
a stat-only refresh passes,
and a staged change is still refused;
the reader is larger and a format change passes unnoticed.
  - Pros and cons:
(a) is exact and simple,
and refuses legitimate work another session did;
(b) tolerates refreshes and needs a field reader whose every corner is a control.
  - Ranking:
a > b,
because a proof one byte wide is what the decision asked for,
and b's tolerance buys a reader whose misses would be silent.

## Questions only the project can answer

Two questions are left.
Three of the second pass's four are answered,
and their answers are recorded in "The project's answers of 2026-11-22" and applied in the sections it names.
Q-R5 is the third gate pass's,
and it is put beside Q-B.

- Q-B.
May rung 1 decline a very dirty main checkout in favour of Git's own checkout,
as rung 2 does?
The partial-clone decision cuts rung 2 at a changed-file count and states "no add is slower than Git's own checkout" (`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`,
"What was decided");
no record says it of rung 1,
and the decision record of 2026-11-22 leaves this question open (`:109` to `:113`).
  - Case:
a main checkout with 6,000 changed tracked files,
where [M4][m4] would show rung 1 slower than rung 3.
  - Option (a),
decline above a count [M4][m4] measures,
with rung 2's bar.
  - Option (b),
always take rung 1 for a dirty main.
  - Pros and cons:
(a) is never slower than Git's own checkout,
and gives up the clone's gain above the cut;
(b) keeps one route and the clone's disk saving,
and can be slower than Git's checkout on a very dirty source.
  - Ranking:
a > b,
because the owner's own rung 2 answer chose "no add slower than Git's own checkout" for a dirty source,
and rung 1's stated advantage is speed.
The cut itself is a measurement,
not a question.

- Q-R5.
May an add that could not unmount its own new clone cancel its own unmount intent?
The accepted rule "No implicit remount after a failure or restart" has one exception,
"the checked reattach of a removal that cancels itself without `--force`" (`doc/planning/cli-git-zfs-worktree-design-proposal.md:1492` to `:1493`),
which does not name an add,
while the project's answer of 2026-11-22 says the add finishes "only after the mount and the add's own intents are settled by the add itself" (`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md:50` to `:53`).
  - Case:
another session's shell sits in the slot while the add settles after a refused proof,
so the receipt comes back busy.
  - Option (a),
yes,
as a removal that cancels itself does:
the add writes its intent back to `mount` (nothing was unmounted;
the receipt said busy),
prints the pending state and the removal command,
and the clone stays mounted as it was.
The person sees one state,
and the accepted exception is extended to an add in the cancelled-removal design's words.
  - Option (b),
no:
the add leaves the `unmount` intent standing,
and the handler unmounts the clone when the holder leaves,
after the person has been told the clone is pending.
Nothing is added to the exception,
and the pending state then describes a clone that may be unmounted by the time the person acts,
so a resume must remount it,
which the rule otherwise refuses.
  - Pros and cons:
(a) settles the intent in the invocation that raised it and leaves one state to describe;
(b) changes no accepted sentence and leaves an unmount nobody now wants to happen later.
  - Ranking:
a > b,
because (b) turns an unmount nobody now wants into one that happens later,
while the add's own intents are settled by the add itself,
which is the owner's record.
If the owner reads the exception as closed to anything but a removal,
the order is b > a,
and the pending state's words must say the clone may unmount by itself.

## Dependencies on the sibling designs

Each is a place where this design answers with an assumption,
or states a requirement,
at the commit this revision read.
The sibling branches move as their own rounds run,
so the commit is named with each.

- The add's settle needs two primitives from designs that own them,
and asks nothing else:
the handler's unmount request and receipt,
for the one clone the record identifies by dataset GUID and `createtxg`,
which is the account-written desired-state intent and the root-written receipt the accepted design states (`doc/planning/cli-git-zfs-worktree-design-proposal.md:554` to `:579`);
and an unlock that keeps the registration,
which the lock design must expose (below).
The `zfs destroy` of the clone and of the snapshot needs no design's call:
the route's grants already carry it ("Scoped delegation").
- The deferred-destroy checks (`1f7a0800fa`,
read last,
as this design's round ran beside it):
its "An add's own destroy does not pass through these checks" says both checks read a destruction plan,
a removal intent and a stage,
"none of which an add writes",
and leaves "the add's own path,
order and reporting" to this design (`:1328` to `:1336`).
This design therefore calls none of its checks and writes no plan,
and takes its one requirement on an add:
a mark set on the snapshot the add made reaps it in the same transaction as the clone's destroy,
so "The add's fallback has to treat that one `ENOENT` as done" (`:1349` to `:1358`),
which the settle's fourth and fifth steps do.
- The destruction plan (`b4dc758a76`):
it lists this design as "no dependency.
None reads or writes this record" (`:3343` to `:3347`),
which the settle now matches:
the add writes no plan.
- The add-time lock (`86f122c2b1`):
its "A failed add" keeps the locked registration for a resume,
and its "How the wrapper's own removal releases the lock" releases `locked`,
empties the registration and removes it (steps 4 to 6,
`:2138` to `:2160`).
The add's fallback needs the first step and none of the others,
so this design requires an unlock that deletes `locked`,
syncs the registration directory,
and leaves the registration and its files for the Git checkout that follows (`:2160` to `:2166` are what it must not do).
Its D6 describes a slot with no `.git` between the discard and the pointer,
which this design's exchange never passes through.
- The durable binding record (`ec2b0b14f3`):
its V16's A keeps the claim,
the slot and the locked registration for a resume (`:3810` to `:3831`),
which the project's answer replaces for this route;
this design requires that an add which finished with Git's own checkout may close the binding and release the claim,
and that the record's lines report a registration without a dataset,
which is where a person reads a settle that failed (its V16's example,
`doctor`'s "registration without a dataset").
Its content-proof sentence,
"A resumed add whose proof is gone refuses" (`:4176` to `:4180`),
differs from this route's decided fallback,
which settles and finishes;
the refused case stands only for a clone whose identity cannot be proven.
Its record convention gives the parent folder `<common>/monochromatic-git-policy-cli/` and one family folder per record (`:932`,
`:1157` to `:1162`),
which this design follows with `add-capture/v1/`.
- The cancelled or interrupted removal (`1fee10f2ce`):
its "one answer covers an add's retry" is about a mount's boundary refusal (`:2323`),
and nothing in it says an add may write an unmount intent or cancel one,
which Q-R5 puts to the project.
- The forced arm (`6871053b79`):
it lists "the dirty-main restore" among what it does not cover (`:2383`),
so the add has no forced arm and never ends a holder,
which its Hardening section and this design agree on.
- The origin cleanup record (`a95a179338`):
its unaccounted-snapshot line says of a snapshot nothing accounts for that it "destroys nothing and names the command a person may run" (`:2141`),
and "a snapshot with a live binding is an add and not litter" (`:2129`).
So a pending add's snapshot is not named by that line,
and the binding design's lines are where a person reads it.
- The private propagation after a replacement mount (`c7708953c0`) and the lost-last-event triggers (`88f6a15194`):
this design's step 1 assumes the mount of a fresh clone is verified in the caller's namespace by the add's step 4,
and adds no trigger of its own,
which the lost-last-event design's D10 records at `:3864`.
- The start probe and the ABI gate (step 6):
nothing here reads the kernel's layout;
the proof reads files and `stat` only.

## How the accepted sentences are met

- "a verified dirty-main restore comparable to Btrfs":
the restore is the btrfs rung's own,
run with the clone's root as its stage and an index under a private name,
and its proof is the frozen proof.
Where it is weaker than btrfs is stated:
the change time's tick and the settle's trust (L4,
F21,
F30,
V4),
and the guest probe [M8][m8].
- "neither route accepts an unproved source":
the proof is D1 to D4 for every rung 1 add,
clean ones included (V6),
and a refusal serves the worktree from Git's own checkout and says so,
or leaves the pending state when its own settle fails.
- "capture the index/tree verification inputs" (design step 1):
C1 to C5,
taken at preflight for the cheap refusals and again as the frozen capture before the snapshot,
with the whole refusal set re-run.
- "Compare the frozen index and tracked content in the clone itself against the captured proof,
without running Git against that directory.
The in-snapshot check proves the bytes" (design step 4):
D1 to D4,
run before any write and before any Git run inside the clone.
- "Discard copied Git administration before running Git inside the clone" (design step 4):
no Git run of the restore is given the copied administration,
which stays until step 8's exchange,
as the project's answer of 2026-11-22 chose,
and it is discarded in the same instant the pointer takes its name.
- "Install the saved Git pointer inside the clone.
Use the verified index and the existing conversion checks and tree difference in the new worktree context" (design step 5):
the private-name index of step 5 and the fence's write of step 7.
- "No silent fallback over an unresolved managed state":
the fallback is not silent,
because the add prints the route that made the worktree and the reason the first was abandoned,
and it finishes only after the add's own mount and intents are settled by the add itself,
which is what the project's answer of 2026-11-22 records as the amendment of that sentence.
A settle that fails leaves the pending state and the removal command.
- "No configuration":
nothing here has a setting;
every threshold it needs is a measurement it names.
- The nearest lock-free record shape:
the capture is written with an exclusive staging name and linked into place,
and `doctor` reports a staging name a killed writer left.

## Hardening the stated threat model may make unnecessary

The project's sentence treats the account as trusted and root and the account as no security boundary on this machine.
Each check of this design is placed against it here,
and none is dropped on the strength of the sentence alone.

- The frozen proof's refusal (D1 to D4).
It survives:
the writes it catches are this machine's other sessions saving files and running Git in the main checkout,
which happen daily.
- Comparing the copied index byte for byte (V8).
It survives:
byte equality is the cheaper comparison,
and it also catches a reordering or an extension a field comparison might not read.
- The racy set's digest (C4).
It survives:
it exists because of the kernel's coarse clock (F15) and Git's seconds (F2),
not because of a hostile account.
- Reading the clone's files below a held root without following a link.
It survives as an accident guard,
and the helper is the wrapper's own (`open_root`,
`DirectoryChain`).
- Removing only a clone-local stub it can name,
and refusing a stub that holds more (step 4).
It survives:
the underlay's files are a person's,
and a snapshot may have carried them (F18).
- Verifying the mount's identity at the slot before the proof runs.
It survives:
a mount stacked at the slot is as much a person's own shell as an attacker's,
and the add's step 4 already decides it.
- Checking the capture record's owner and mode before reading it.
This one exists only for a hostile account:
the record lies in the account's own folder,
where the account can write it,
and a mode check stops another account,
of which this machine has none.
It stays,
labelled as such.
- Refusing to end a holder that keeps the clone,
which is what makes a settle fail into the pending state rather than a forced unmount.
It survives:
the add has no authority to end another process and seeks none.
- The content check at the head of step 8.
It survives:
it exists because Git's own check cannot see a same-second rewrite (F2,
F23),
which is an accident of two sessions on one tree before it is an attack.
- `doctor` reporting a staging name or a record an add left.
It survives as tidiness.

## Limitations

- The proof detects a window write and cannot prevent one,
and its detection rests on a millisecond tick (F15,
F22,
F38) and on Git's seconds (F2,
F23).
- A tracked write in the same tick as the file's own last write before the capture can leave every compared field equal,
and on ZFS the settle would believe the file (F30).
L4 states the case,
V4's adopted rule closes it for the racy set,
and the set is one entry of 24,689 at one moment (F39).
A veto of V4 returns it,
its size unmeasured over a day.
- The clone's fill is visible and writable before it is proven,
and the project's answer keeps the copied administration in the slot until step 8 ("The project's answers of 2026-11-22").
So a command run there acts through the copy and can reach three places (L8):
the tree being restored,
which the content check at the head of step 8 refuses;
the 187 other worktree folders the copy names,
which no step of this design closes;
and the remote,
which no step of this design closes either.
It never writes the main checkout's own files,
index or references.
- Making the copy inert was put to the project and not chosen (`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md:104` to `:107`).
Its cost,
were it taken,
is one read of the copy's `worktrees` folder,
187 removals of registration directories,
one rewrite of the copy's configuration without its remote section,
and four hook removals (F40),
which no measurement sizes.
- The ignored state the clone carries is the ignored state of the snapshot's instant (`SPEC.md:1518`),
and a path the target commit stops ignoring is carried and listed as untracked.
- A tracked file under a nested dataset's mount location:
the capture reads the source's mounted content and the clone holds the snapshot's underlay,
so the proof refuses such a path,
and the add settles and finishes with Git's own checkout.
Refusing before the snapshot would need a check of the source's mount table against the tracked paths,
which this design does not build.
- A split or sparse index is refused at the capture,
from its extension signatures,
before anything is written,
because the rewrite cannot install one (F34).
- The copied administration lives inside the new worktree until the disposal empties it (V5),
so a `du` of the new worktree reports it,
and a person who deletes it deletes only a copy.
- The clone shares its blocks with the origin snapshot,
so the origin cannot be destroyed while the clone lives (F13),
which fixes the order of every cleanup and of the fallback.
- The exchange is read from the sources (F37) and not run;
a guest run is owed ([M9][m9]).
- Nothing here measures a real clone,
a mount,
a restore inside one or a fallback;
every such claim carries its probe.
- The capture's record is about 2.5 MB on this repository (L7),
a per-add write the add did not have before.

## What this design does not cover

- The mount and unmount themselves,
the handler's sweep,
the staging folder and the reattach,
which their own designs own.
- The removal's lineage proof,
the destruction plan,
the deferred-destroy checks,
the cancel command and the forced arm.
- The durable binding record's home and format,
the managed Git lock's write path,
and `doctor`'s report of the states this design names.
- The rung 2 partial clone of a dirty source,
issue 6323's own fix,
and the conversion fence's cost.
- The dependency repair after a clone fill (plan step 21),
which step 9 queues as before.
- The one-time move of the checkout to its own dataset,
the boot units and the migration (plan step 19).

## Citations

Every quotation and every path-and-line citation in this document was checked by a scratch script that printed each cited range beside the text of the lines it names,
and a second pass compared each blockquote with its source.
The document holds 143 citations.
The gate review named 7 wrong and 2 loose,
and those 9 are the ones corrected below.
Every one of the other 134 was printed beside the lines it names and read,
and each says what this document says it says.
A second pass matched each blockquote's words against the lines it quotes.
The corrections are:
`SPEC.md:1355` for the three-runs sentence (the first text cited `:1333` to `:1340`);
`statinfo.c:64` to `:99` for Git's comparison,
with the `USE_NSEC` branch at `:75` to `:81` and the `USE_STDEV` branch at `:91` to `:99` (the first text cited `:64` to `:96` and claimed the nanoseconds);
F15 for the ZFS stamp,
replacing the generic write path the first text cited as F14 (`mm/filemap.c:4472`,
`fs/inode.c:2501`);
F17 for the snapshot's nested copy,
replacing the first text's F13 use of the measurement record;
`SPEC.md:1518` for the ignored state's instant,
without the `:1468` to `:1480` range the first text added;
`AGENTS.md:362` for `QNB` (the first text wrote `NQB`,
which does not exist);
issue 6323's own lines (`SPEC.md:2736` to `:2738`) for the conversion defect it names;
`sa.c:1702` to `:1704` with the words they hold (the first text's sentence named `:1704` to `:1706`,
the change and creation times,
for a sentence about the access and modification times);
and `SPEC.md:4408` to `:4410` for the record sentence (the first text cited the heading's line `:4398`).
The script's output,
printing every citation beside the lines it names,
is kept at `~/temp/agent/zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/citations-2026-11-22.md`.

The second pass's citation findings are corrected in this revision:
F14 now cites `sa.c:1702` to `:1704`,
the access and modification times,
where it had the change time and the creation time;
F31 now names `matches_stored_exactly` at `worktree_clean_tree_walk.rs:248` as the predicate C3 uses,
rather than the walk that contains it;
and V4 no longer offers the wrapper's own racy rewrite as Git's rule,
naming it as the same idea at Git's second granularity instead.
The citations this revision adds,
F36 to F42 and those under "The project's answers of 2026-11-22",
were read at the lines they name before they were written.

## Revisions

- `069fa35d98` at `2026-11-22T17:35:38-05:00`:
the first text.
The design as reviewed:
the capture taken immediately before the snapshot,
the proof as three comparisons,
the device-and-inode rewrite in step 5,
the pointer and index published before the restore,
V1 to V6 and Q1 to Q3.
- `1f24dcc10d` at `2026-11-22T17:36:49-05:00`:
named the package docs the design read,
in "What it read" and "Sources read".
- `414d9901fc` at `2026-11-22T18:38:11-05:00`:
the gate-review revision.
H1's rewrite removed,
H2's order,
the tick and Git's seconds,
the refusals re-run against the capture,
Q-A to Q-D replacing V5 and Q1 to Q3,
the capture's format and end,
the allocation to step 10,
the cost and test sections rebuilt,
the citations rechecked,
and the sibling passages read at their commits.
- `b46a064030` at `2026-11-22T18:38:49-05:00`:
says in "What it reuses,
and where ZFS differs from btrfs" that the clone rung's index install is `write_index_under_a_name` and not `install_index`.
- `f73e12ae1e` at `2026-11-22T19:16:30-05:00`:
`main` merged with `--no-ff`,
bringing the owner's three answers (`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`) and the removal record `doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
which says nothing of an add's calls.
- `f12ef2b94e` at `2026-11-22T19:18:12-05:00`:
the second-pass revision,
which applies the three answers and answers the pass's findings:
the pointer exchanged with the copied `.git` in one exchanging rename,
with the kill states at each point of step 8 and the post-restore content check;
`restore_dirty`'s three real-name index sites named and T6b added for them;
the resume's waiting index and its lock cleared with `clear_the_waiting_index`,
its rows split by the real-name index and by the exchange,
and the proof's phase moved to a marker file;
the fallback decided,
so every failure after the claim settles and finishes with Git's own checkout,
with the requirement stated on the designs that own the unmount and the destruction;
the proof's window decided and the plan's test sentence amended in V6;
V4's racy settle adopted as the design's default,
with the tick's source and the racy set's measured size;
the copy's reach stated as a limit,
with the inert copy costed and the record's refusal of it quoted;
T3 and T3b rebuilt,
T10 and T11 rewritten,
and the split or sparse index refused at the capture;
the five sibling tips re-read and the binding record's statement on the proof's home quoted;
F14's and F31's citations corrected and V4's reference to the racy rewrite reworded;
and "Answers to the second gate pass" recording each finding with where it was fixed.

- `cee3211ae6` at `2026-11-22T19:58:00-05:00`:
the third-pass revision.
The post-restore check compares each written path's facts to the nanosecond against the entry Git recorded,
reads content only where the stat cannot decide and the judge can,
and says what a conversion makes undecisable,
with T12's three arms and its cost in M1;
the add's settle owns its own destroy path,
calls none of the deferred-destroy checks,
writes no plan,
treats a reaped snapshot's `ENOENT` as done,
releases the lock while keeping the registration,
closes the binding and releases the claim,
states its requirement on the lock and binding designs,
and publishes a settle marker a resume continues;
the copy is set aside as the trash's next `d<index>` entry with its identity in the trash's `.manifest` before the exchange;
the resume gains the "copied `.git` and the real-name index" row and loses the unreachable clause;
the add closes its handles before it requests the unmount,
and Q-R5 brings the intent to the project;
the settle's racy test's two inputs are listed as a fourth change with T13;
the waiting-index function is made callable,
T3's fixture writes its own entry,
the pathspec,
the doubled quotes,
the record path,
the corrections' `sa.c` range and F41's lines are corrected,
the printed sentence is the snapshot rung's,
"narrows" replaces "closes",
F37 cites the one-transaction and one-log-record lines,
and `doctor`'s line is replaced by the binding design's;
the sibling commits are read at the tips this revision names.

## Sources read

Git `v2.56.1`,
cloned into this round's scratch folder from `git/git` at tag `v2.56.1`:
`statinfo.c`,
`read-cache.c`,
`read-cache-ll.h`,
`unpack-trees.c`,
`builtin/read-tree.c`,
`builtin/clean.c`,
`builtin/checkout-index.c`,
`builtin/commit.c`,
`builtin/worktree.c`,
`worktree.c`,
`environment.c`,
`Makefile`,
`config.mak.uname`.

OpenZFS `zfs-2.4.4`,
cloned into this round's scratch folder from `openzfs/zfs` at tag `zfs-2.4.4`:
`module/zfs/dsl_dataset.c`,
`module/zfs/dsl_destroy.c`,
`module/zfs/zfs_ioctl.c`,
`module/zfs/sa.c`,
`module/zfs/zfs_vnops.c`,
`module/os/linux/zfs/zfs_znode_os.c`,
`module/os/linux/zfs/zfs_vnops_os.c`,
`module/os/linux/zfs/zpl_super.c`,
`include/os/linux/spl/sys/time.h`,
`include/sys/zfs_znode.h`.

Linux `v7.3`,
six files read at tag `v7.3`:
`fs/attr.c`,
`fs/utimes.c`,
`fs/inode.c`,
`mm/filemap.c`,
`fs/namei.c` (this revision,
for the exchange's type rules),
`fs/btrfs/super.c`.
The first round fetched five of them with `gh api`;
this revision cloned the tag into its scratch folder for the sixth.

This repository,
read at `5dd8883a60`:
`AGENTS.md`;
`doc/decision/cli-git-zfs-worktree-design.md`;
`doc/decision/cli-git-zfs-staging-under-cache.md`;
`doc/decision/cli-git-zfs-step-8-sweep-choices.md`;
`doc/decision/cli-git-zfs-slot-identity-store-format.md`;
`doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`;
`doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`;
`doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md` (read at `46ed57af59` on `main`);
`doc/planning/cli-git-zfs-worktree-design-proposal.md`;
`doc/planning/cli-git-zfs-worktree-implementation-plan.md`;
`doc/planning/cli-git-zfs-staging-under-cache-design.md`;
`doc/planning/cli-git-clone-rung-conversion-fence.md`;
`doc/planning/cli-git-btrfs-worktree.md`;
`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`;
`doc/handover/cli-git-rust-implementation.md`;
`package/git-policy/cli/SPEC.md`;
`package/git-policy/cli/src/native/worktree/worktree_dirty_restore.rs`,
`worktree_snapshot_index_step.rs`,
`worktree_snapshot.rs`,
`worktree_snapshot_source.rs`,
`worktree_ladder_finish.rs`,
`worktree_ladder_target.rs`,
`worktree_slow_route.rs`,
`worktree_clean.rs`,
`worktree_clean_tree_walk.rs`,
`worktree_clean_stat_verdicts.rs`,
`worktree_clone_index.rs`,
`worktree_clone_conversion_beside_the_clone.rs`,
`worktree_clone_conversion_fence.rs`,
`worktree_copy_journal.rs`,
`worktree_copying/worktree_copy_interrupted_add_settlement.rs`,
`subvolume/subvolume_index.rs`,
`subvolume/subvolume_index_install_into_new_worktree.rs`,
`subvolume/subvolume_index_files.rs`,
`subvolume/fs_beneath.rs`,
`subvolume/fs_clone_exchange.rs`;
`package/git-policy/cli/doc/slot-identity-store-write-cost.md` and `package/git-policy/cli/doc/object-database-and-a-reftable-ref-store.md`.

The gate review and its probe logs at `~/temp/agent/gate-review-zfs-dirty-main-restore-on-rung-1-design-2026-11-22/keep/review.md` and `probe-logs.md`,
the second gate pass and its probe logs at `review-second-pass.md` and `probe-logs-second-pass.md` beside them,
and the sibling designs at the commits named under "Dependencies on the sibling designs",
the forced arm and the cancelled removal read last.

## Links

- The item's decision:
`doc/decision/cli-git-zfs-worktree-design.md`.
- The accepted design:
`doc/planning/cli-git-zfs-worktree-design-proposal.md`.
- The build plan:
`doc/planning/cli-git-zfs-worktree-implementation-plan.md`.
- The accepted staging design,
for this document's shape:
`doc/planning/cli-git-zfs-staging-under-cache-design.md`.
- The conversion fence:
`doc/planning/cli-git-clone-rung-conversion-fence.md`.
- The other rungs' dirty-source answers:
`doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md` and `doc/decision/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`.
- The measurement record:
`doc/planning/cli-git-zfs-measurements-owed-before-acceptance.md`.

[m1]:
#measurements-owed [m2]:
#measurements-owed [m3]:
#measurements-owed [m4]:
#measurements-owed [m5]:
#measurements-owed [m6]:
#measurements-owed [m7]:
#measurements-owed [m8]:
#measurements-owed [vm-register]:
../planning/cli-git-zfs-measurements-owed-before-acceptance.md
