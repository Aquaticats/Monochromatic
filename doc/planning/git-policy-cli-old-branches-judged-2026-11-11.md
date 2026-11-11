# Old wrapper branches judged against the jam-fix integration, 2026-11-11T12:17:25-05:00

What:
 the four local wrapper branches held out of
 `integrate/git-policy-cli-jam-fixes-after-alpha-9`,
 with each branch's own commits grouped,
 a verdict for every group,
 and a proposal for each branch.

Why:
 the integration takes every finished wrapper branch,
 and these four were left out because nobody had judged them.
 An unjudged branch is work at risk:
 the coordinator cannot merge it,
 and nobody knows whether the integration already does what it does.

How to read it:
 every verdict is one of three.
 "Carries the intent" means the integration has the same change or a later design that
 replaces it,
 and the commit or file that does is named.
 A group whose commits serve two designs can be carried in part,
 and each part then carries its own verdict.
 "Still wanted and missing" names the reason it is wanted.
 "Unclear" names what would settle it,
 and no group needed it.
 Each branch section ends with a proposal for the coordinator.

## How the verdicts were reached

- The integration was read at `fc9f7f5f4b` (2026-11-11T10:26:04-05:00).
- `git cherry -v` against the integration marks all four branches as carrying no patch the
  integration has (114,
  223,
  1 and 4 patches).
  Patch identity therefore settles nothing here,
  and no verdict rests on one.
- Every group was checked with `git log -S`,
  `git diff` against the integration's own files,
  and `rg` over those files.
  No verdict rests on a commit subject alone.
- Decision phrases are searched in a whitespace-normalized copy of
  `package/git-policy/cli/SPEC.md`,
  because the file's line breaks fall inside them and split them:
  the owner's answer `End Git, hold hooks` occupies two lines there.
  A line-oriented search reports that answer missing while it is present.
- Line numbers are not cited for the integration's files,
  because they move faster than this record.
  They moved inside this session's own checks:
  the Windows self-shim markers the fourth open-issues pass names at 143 and 148 of
  `package/git-executable/core/src/self-shim.ts` sit at 156 and 165 today.
  Paths,
  headings and commit hashes are cited instead.
- Issue state was read with `gh issue list --state all` on 2026-11-11.

## The four branches, measured

`fix/git-policy-cli-commits-never-fail`:
 tip `1fc759ba80` (2026-11-04T15:51:04-05:00),
 114 commits ahead of the integration and 8988 behind it,
 merge base `5c0b660c27` (2026-11-03T06:44:08-05:00),
 17 files,
 6108 insertions and 166 deletions.

`feat/git-policy-cli-commit-protocol-models`:
 tip `11829f62e5` (2026-11-04T17:57:09-05:00),
 225 commits (2 of them merges) ahead and 7691 behind,
 merge base `ab40458516` (2026-11-04T17:30:30-05:00),
 120 files and 37422 insertions:
 76 under `src/native/model/`,
 39 under `src/native/commit/`,
 one in `src/native/service/`,
 one in `doc/handover/`,
 and `Cargo.toml`,
 `mise.toml` and `README.md`.

`refactor/git-policy-cli-source-layout`:
 tip `671eb1c6d1` (2026-11-04T16:59:41-05:00),
 one commit ahead and 7720 behind,
 merge base `c3f6209fa8` (2026-11-04T16:25:47-05:00),
 2 files and 2136 insertions.

`fix/git-policy-cli-open-issues-next-batch`:
 tip `c2fe2c649a` (2026-11-10T03:17:31-05:00),
 4 commits ahead and 3240 behind,
 merge base `aca920cdf4` (2026-11-09T14:21:53-05:00),
 2 files,
 339 insertions and 2 deletions.

The distance matters for one part of the judgement and no other:
what a rebase or a port costs.
An intent that landed by another route is carried however far apart the branches are.

## `fix/git-policy-cli-commits-never-fail`

This branch is the design record of the commit protocol,
not its implementation.
Its own first line names its subject:
"Commits never fail:
removing each environmental exit of `git commit` by construction".

### The commits, grouped

- 108 of the 114 commits write the design record:
  `doc/planning/cli-git-commits-never-fail.md` (2711 lines at the tip),
  its `-review-history.md` (2108 lines) and,
  in the commit that created the record,
  `-review-history-archive.md` (703 lines).
  They are one design through revision 19,
  with the answers to its cross-vendor reviews,
  the review rounds up to 25,
  and the owner decisions the design asked for.
- 6 commits record owner decisions in `package/git-policy/cli/SPEC.md`,
  three of them beside an edit of the design:
  `9ea50d6641` (the causes outside our control and the refusals that remain),
  `5f49bd57b1` (a live foreign `index.lock`),
  `d09c58e7d1` (a stuck step,
  deferred staging,
  unclear publication),
  `e716f07ff6` (the correction,
  "not by retries"),
  `f160209787` (a held ref lock,
  hooks and filters,
  unsaved edits in a merge),
  `2c61abcb77` (a branch that keeps moving,
  a `--mirror` push,
  an amend while commits are pending).
- 3 commits are the red controls and one write of code for item 1 of the design,
  a Git step that crosses its class bound:
  `53e40e9fc0` (red,
  a stalled service step must not end the commit),
  `7b7d9870f3` (red,
  a slow but live service must not end the commit after the client's wait;
  the commit message says it was not run),
  `26db9e54af` (the retry-based attempt whose own subject calls it
  "superseded in direction by the owner's correction").

### Group 1, the design record: still wanted and missing

Verdict:
 **still wanted and missing**,
 and blocked by a sequencing the owner set,
 not by anything in the integration.

Evidence:

- No such path exists in the integration:
  `doc/planning/` holds no `cli-git-commits-never-fail*` file at `fc9f7f5f4b`.
- The integration says so itself,
  in `doc/handover/cli-git-rust-implementation.md`,
  "Note 2026-11-08":
  "`doc/planning/cli-git-commits-never-fail.md` is not in this tree;
  it lived on a lane branch and its newest revision is `1fc759ba80` (2026-11-04)."
  That hash is this branch's tip,
  so the record the integration points at is exactly the one on the branch,
  and it is nowhere else.
- The integration reads the design as the live design and cites it six times in its body,
  outside that note,
  for example in
  "Commits,
  merges and pushes that never fail",
  "Commits never fail,
  revision 6 (four questions)",
  "Commits never fail,
  revision 9 (three choices)" and
  "Commits never fail,
  revision 11 (five choices)".
  `doc/planning/cli-git-delegated-picks-status.md` names the design at
  `fix/git-policy-cli-commits-never-fail@6d053403db` (revision 19) as one of the three
  inputs its read-only check was run against.
- The owner answered the pause of 2026-11-04 with "Resume after tokio's step (Recommended)"
  (`doc/handover/questions-for-the-owner.md`,
  "The commit protocol paused on 2026-11-04"),
  and named this branch and
  `feat/git-policy-cli-commit-protocol-models` as the two to compare with main
  "before anything is built".
  That keeps the record wanted and drops it behind the tokio plan.
- The gate of that answer is not met yet.
  `doc/planning/git-policy-cli-full-tokio.md` ends at step T13 and puts the commit path
  last,
  and the handover's tokio entries of 2026-11-10 record steps T12 and T13 as the
  current work.
- The design's own open questions are closed:
  its "Open owner questions" section says "None.",
  and what remains is two unmeasured constants (`WAIT_DRAIN`,
  `ATTACH_WATCH_SPAN`) and steps 4 to 7 of its ordered plan.
  So the record is not waiting for an owner answer;
  it is waiting for the runtime it was written against.
- Merging it as is would put text in the tree that the tree contradicts:
  the record's "What exists on the branch from before the correction" names
  `53e40e9fc0`,
  `7b7d9870f3` and `26db9e54af` as kept,
  and two of those three intents have since landed in the integration by another route
  (group 3 of this branch),
  in shapes the record does not describe.

Proposal:
 keep the branch.
 Do not merge the record now and do not delete it.
 When the tokio commit-path step lands,
 resume with the comparison the owner asked for,
 and carry the 108 commits in one go:
 they touch only `doc/planning` paths the integration lacks,
 so they apply without a conflict,
 and the one edit they need is an in-place note,
 in the record's "What exists on the branch from before the correction" and its
 "Item 1" and "Item 3" headings,
 recording that those two intents landed in the integration as
 `src/native/transaction/transaction_git.rs` and the SPEC section
 "Git steps,
 tools and checks end only on no progress".
 Retire the branch only after that resume,
 or on the owner's word that the design is dead.

### Group 2, the owner's decisions in `SPEC.md`: partly carried, five missing

Verdict:
 **carries the intent in part**,
 and the rest is **still wanted and missing**.

Evidence,
carried:
 the integration's SPEC section "Merges and commits never fail" and the section
 "Git steps,
 tools and checks end only on no progress" hold these of the branch's
 decisions,
 reworded by later commits of their own:
 the correction `never fail doesn't primarily mean retries!`,
 the list of causes outside our control (and the person's editor added to it),
 the refusals that stay,
 the conflicted-path rule,
 "Stack on the new head",
 "Save on a side ref" for a live foreign ref lock,
 "Same as the ref lock" for a live foreign `index.lock`,
 "Allowed refusals" for hooks and filters,
 "Carry edits forward" for a merge over unsaved edits,
 and "End Git,
 hold hooks" for a stuck step.

Evidence,
missing:
 five of the branch's decisions are absent from the integration's SPEC under any wording
 I could construct,
 checked in the whitespace-normalized copy:

- a branch a foreign program keeps moving,
  "Save on the side ref",
  with the pending ref `refs/cli-git/pending/<id>`:
  no `refs/cli-git/pending` string exists in the integration's SPEC,
  and `refs/cli-git` in the integration's source is the auto-push reconcile ref only
  (`src/native/push/auto_push_reconcile_tests.rs` asserts it is empty after a pass).
- `--mirror`,
  "Leave out wrapper refs".
- an amend while commits are pending,
  "Amend the pending commit".
- deferred staging,
  "Update the index".
- unclear publication,
  "Land it".

The integration's handover knows about four of the five.
In "Never fail,
and what is outside our control" it writes:
"Answered by the question tool on 2026-11-03 and not yet in main's SPEC
(they are on `fix/git-policy-cli-commits-never-fail` until it merges)".
That sentence is the recorded plan for this group:
the decisions arrive with this branch,
not before it.

Proposal:
 do not port these five separately.
 The recorded plan puts them in the same merge as the design record,
 and a separate insert would duplicate text that merge would then add again,
 for the coordinator to resolve.
 If the owner wants the decisions in the tree sooner,
 the insertion point is the SPEC section "Merges and commits never fail",
 and the text to insert is what `9ea50d6641`,
`5f49bd57b1`,
`d09c58e7d1` and
 `2c61abcb77` already write.

### Group 3, the red controls and the retry attempt: carried, direction superseded

Verdict:
 **carries the intent**,
 and the retry direction inside it is **superseded by a later design** that the owner's
 correction forced.

Evidence:

- The four files this group edits no longer exist at those paths.
  The layout move took them apart:
  `git diff --stat 5c0b660c27..fc9f7f5f4b` over
  `src/native/child_watch.rs`,
  `src/native/wait_progress.rs`,
  `src/native/transaction_git.rs` and
  `src/native/candidate_reader.rs` is 1773 deletions and no insertion.
  They live under `src/native/push/` and `src/native/transaction/` today,
  which is also why `git cherry` can never show them as applied.
- The commit-path half of the intent is in the integration.
  `src/native/transaction/transaction_git.rs` prints
  "showed no sign of work and was ended;
  running it again",
  and `transaction_git_step_rerun_and_end_rules.rs` decides that only where
  `may_be_rerun` holds.
  The SPEC section "Git steps,
  tools and checks end only on no progress" states the rule
  the branch's red control wanted:
  a step that shows no sign of work for its window is ended and run once more,
  and a step that is quiet again fails with a sentence naming the window and the variable.
  That is a narrower rule than the branch's test asserts
  (`a_git_step_that_stalls_is_started_again_and_the_commit_lands` always lands the commit),
  and the narrowing is the owner's correction,
  which the branch's own `26db9e54af` acknowledges.
- The reader half is not in the integration and is superseded rather than missing.
  The branch stops a stalled `git cat-file --batch` and starts the reader again,
  with twenty doublings of the window (`RESTART_DOUBLINGS` in its
  `candidate_reader.rs`).
  The integration stops the reader at `OBJECT_STEP_CEILING`,
  ten minutes,
  and reports the stop
  (`src/native/candidate/candidate_reader.rs`,
  through `ChildWatch` and `watched_timeout_text`).
  Both shapes follow the SPEC rule "no run ends because it has taken long",
  and the difference is exactly the one the owner's correction decides:
  a retry is the last resort,
  and a stall of a step safe to rerun is reported once rather than retried for ever.

Proposal:
 nothing to port.
 The branch's attempt is the specimen if the owner ever wants a retry beyond one rerun;
 today's design says one rerun and a named failure,
 and the reader's ceiling follows the same rule.

### Proposal for `fix/git-policy-cli-commits-never-fail`

Keep the branch,
do not merge it into the integration now,
do not delete it.
It holds three things the integration does not:
the design record,
five owner decisions,
and the specimen of the retry attempt.
Two of them are unbuilt work the owner has already ordered resumed after the tokio
commit-path step,
and the third is a decision list whose landing plan is recorded in the handover.

## `feat/git-policy-cli-commit-protocol-models`

This branch builds steps 1 to 3 of that design:
the models,
the durable components,
and the lane service behind a service flag.
Its own handover,
`doc/handover/cli-git-commit-protocol-models.md` (2053 lines at the tip),
opens with "Nothing here is wired into the live wrapper".

### The commits, grouped

- The models,
  76 files under `src/native/model/`,
  a new `[[test]]` target `native_models` in `Cargo.toml`,
  a `test-seams` feature,
  and the `native:test:model` and `native:clippy:seams` tasks in `mise.toml`.
  Several `model_*_r13.rs` to `model_*_r23.rs` files are one model per review round,
  so the round history is in the file names.
- The durable components,
  39 files under `src/native/commit/`:
  the state log,
  the ledger,
  the spool,
  the inbox,
  the identity binding,
  the request envelope,
  the disposal,
  the drain,
  the replay,
  the flusher,
  the helper launch,
  and the lane lifecycle and lane state.
- The step 3 cutover,
  the lane service in `src/native/service/service_main.rs` behind `--lane`,
  with the doorbell,
  the startup sweep and the periodic pass.
- The branch's handover,
  reviewed over rounds 13 to 30,
  with its coverage map and its review findings.

### Verdict: still wanted and missing, and blocked by the same owner answer

Evidence:

- Nothing of it is in the integration.
  None of `commit_lane_service.rs`,
  `commit_disposal.rs`,
  `commit_drain.rs`,
  `commit_flusher.rs`,
  `commit_spool.rs`,
  `commit_inbox.rs` or any `model_*` file exists
  under `package/git-policy/cli/src`,
  and `rg -i 'flusher|doorbell|commit_lane|lane_service'` over the whole of
  `package/git-policy/cli` finds nothing,
  docs included.
  The service has no `--lane` mode.
- The owner's answer is the same one that blocks the design record:
  "Resume after tokio's step (Recommended)",
  with both branches named
  (`doc/handover/questions-for-the-owner.md`,
  "The commit protocol paused on 2026-11-04").
  The tokio commit path is not on the runtime yet:
  `doc/planning/git-policy-cli-full-tokio.md` ends at T13,
  and its own order puts the commit path after the landing's steps.
- The work is finished to its own stated scope and reviewed.
  The design's ordered plan records steps 1,
  2 and 3 as built on this branch,
  with the rows they close,
  and the branch's last commit records 662 of 662 tests passing,
  with Clippy repairs in the same commit,
  which is the state its own round 30 entry describes.
  It is not an abandoned sketch.
- Merging as is would be a design change,
  not a port.
  749 commits of the integration touch the paths this branch rewrites
  (`src/native/commit`,
  `src/native/service`,
  `Cargo.toml`,
  `mise.toml`) since its merge base,
  and the branch adds a test target,
  a cargo feature and a service mode that the integration's own Cargo,
  mise and service files have since restructured.
  A merge would also,
  in effect,
  wire the protocol in,
  which the branch's handover says nothing in it does.

Proposal:
 keep the branch,
 do not merge it,
 do not delete it.
It is the implementation half of the paused protocol and the owner has already
ordered the comparison that decides what of it ports.
When the tokio commit-path step lands,
run that comparison,
then port by step rather than by branch:
step 3's lane service is the piece that reaches into the shipped service and is the one
to judge on its own.
The integration's handover's list of kept branches,
"The branches and worktrees this effort still holds",
names this branch's sibling and not this one;
that omission is worth fixing there,
because an inventory that does not name unmerged work is how work gets lost.

## `refactor/git-policy-cli-source-layout`

### The commit, grouped

One commit,
`671eb1c6d1`,
"source layout proposal for the git-policy-cli crate (issue 6261,
third proposal,
nothing moved)":
`doc/planning/cli-git-source-layout.md` (702 lines) and
`doc/planning/cli-git-source-layout.mapping.tsv` (1433 rows of old path,
new path,
kind and module).

### Verdict: superseded, by a recorded decision of the owner

Evidence:

- The integration's `doc/planning/cli-git-source-layout.md` says it in its status
  paragraph:
  "The child session's third proposal (branch `refactor/git-policy-cli-source-layout`,
  `671eb1c6d1`) is not used;
  only its measured edge data was looked at."
  The commit hash and the branch name are the same ones this branch carries,
  so the supersession is not inferred from a similar title,
  it names the branch.
- The same file records the owner's decision that produced it:
  "I told the layout subagent to stop because it couldn't ever produce something good
  enough.
  Lets do this ourselves."
  A design the owner stopped is not work at risk;
  it is a design the owner set aside with a record saying so,
  which is what "Don't discard work" asks for in this shape.
- The integration carries the second design's own artifacts instead:
  `cli-git-source-layout.md` (28918 bytes against the branch's 702 lines),
  `cli-git-source-layout-groups.json` (the folder groups),
  `cli-git-source-layout-target-tree.md` and
  `cli-git-source-layout-target-tree-files.md`,
  all written after this branch's commit,
  as their own commits of 2026-11-07 and 2026-11-10 show.
- The branch's mapping is a different target from the one the integration now plans.
  It maps `src/native/*` into `tests/binary/...`,
  `tests/commit/...` and a `commands/` tree,
  while the integration's record keeps `src/native/` for now:
  "Removing the `native` layer is deferred (issue 6261 option A,
  the coordinator's pick under the delegation,
  veto open),
  so `package/git-policy/cli/src/native/` still holds the subsystem folders."

### Proposal

Retire the branch once the owner allows branch deletions,
which needs the owner's permission and is not this session's to take.
Nothing to port.
If the measured edge data behind the mapping is ever wanted again,
it is in the branch's single commit,
and the integration's `doc/planning/cli-git-source-layout.md` names that commit in its
status paragraph,
so the path back to it stays visible.

## `fix/git-policy-cli-open-issues-next-batch`

### The commits, grouped

- 2 commits fix positional pointers in
  `doc/handover/cli-git-rust-implementation.md`:
  `08cdb8979a` ("Still to do" two items said "closed by the section below";
  both now name the section) and
  `67eea12838` (the two line breaks the Markdown linter requires of those sentences).
- 2 commits add the fourth open-issues pass,
  `doc/planning/git-wrapper-open-issues-round-4.md` (331 lines):
  `395a70d4a5` (the 15 oldest open issues that name the wrapper,
  read against the tree at `aca920cdf4`) and
  `c2fe2c649a` (the two issues outside the 15 that the plan calls live,
  `#4487` and `#6007`).

### Group 1, the two pointer fixes: carries the intent

Verdict:
 **carries the intent**,
 by a later and wider commit.

Evidence:

- `aeb87871b3` (2026-11-10T14:52:36-05:00,
  "docs(handover):
  name the target instead of pointing at a position") made the same edit
  across `doc/handover`:
  "ten lines in eight files that still pointed at a position ...
  Each now names the heading or record it means",
  and its rescan reports no line left.
- The integration's handover,
  section "Still to do",
  carries both items naming
  "The landing records the stat data it can prove,
  and the staleness question walks its entries in parallel".
- The branch's exact bytes no longer apply:
  the integration later dropped the timestamp from those two bullets and added a
  "Closed 2026-11-09" item to the same list,
  so the two commits are carried in substance and superseded in text.

Proposal:
 nothing to port from these two commits.

### Group 2, the fourth open-issues pass: still wanted and missing, and still accurate

Verdict:
 **still wanted and missing**.

Evidence that it is wanted:

- Rounds 1 to 3 of the same pass are in the integration
  (`doc/planning/git-wrapper-open-issues-against-main.md`,
  `-round-2.md`,
  `-round-3.md`),
  and no fourth pass exists there.
  The pass exists so the next agent does not read the same issues again,
  which is the argument for the fourth one as much as for the first three.
- 13 of its 15 issues are still open on 2026-11-11:
  `#492`,
  `#574`,
  `#630`,
  `#1332`,
  `#1698`,
  `#1717`,
  `#1768`,
  `#3024`,
  `#3258`,
  `#3275`,
  `#3276`,
  `#3745`,
  `#3796`.
  `#1907` and `#1911` closed on 2026-11-10T23:35Z,
  hours after the pass was written,
  so those two verdicts are historical rather than live.

Evidence that its findings still hold,
each re-checked against `fc9f7f5f4b`,
with the three the tree has resolved since named in their own place:

- `#492`:
  `rg -i 'post.?push'` over `package/git-policy/cli/src/native` finds no stage.
- `#630`:
  `package/git-policy/cli/e2e` holds 83 entries,
  24 of them `scenario-*-fixture.ts`.
- `#3024`:
  `default_allowed_worktree_dirs` in
  `src/native/invocation/effective_target.rs` still adds only uv's cache and mise's data
  directory,
  and no cargo build directory.
- `#6007`:
  no `parse_layout` function exists in the crate.
- `#4487`:
  the request list is written from a thread of its own with a poll-based deadline,
  and the answers are read under `RANGE_QUERY_CEILING`
  (`src/native/push/manual_push_range.rs`).
  The issue closed on 2026-11-10T19:48Z as fixed by `73ae19e83c` (2026-11-09),
  which is in the integration,
  and the control the pass named was re-run here:
  `manual_push_range` gives 14 tests run,
  14 passed,
  6854 skipped.
- `#3258`,
  `#3275`,
  `#3276` and `#3745`:
  the named
  `doc/decision/` sentences and paths stand as the pass states them,
  including the two issue ranges and the missing superseded note.
- `#574`:
  the five built-in policies are still `require-root`,
  `add-explicit`,
  `linked-worktree-only`,
  `branch-worktree-only` and `final-newline`.
- `#1698`:
  `doc/migration/no-class.md` still cites `package/` paths that no longer exist.
- `#1332`:
  resolved after the pass by `512bd213b7`,
  which is in the integration:
  `package/git-executable/core/src/self-shim.ts` now spells the package once and
  computes `WINDOWS_GIT_POLICY_CLI_PACKAGE_PATH` from it.
  The pass's reading was of the two hand-written literals at its lines 143 and 148.
- `#1768`:
  the triage comment the pass quotes is unchanged and the wrapper still cannot detect
  its own absence,
  and a comment of 2026-11-10T18:26Z,
  written after the pass,
  corrects the link it names:
  `mise.no-env.toml` and `.cache/cli-git-bin/` are gone,
  the link is `.cache/monochromatic-git-policy-cli-bin/git`,
  it resolves to the main checkout's launcher in a protected session,
  and the comment asks the owner which surface carries the check.

What the pass can no longer claim without edit:
 its line numbers for `#1332`,
 its Method line ("returned 89 issues on 2026-11-10";
 77 open wrapper issues on 2026-11-11),
 its heading "The one fix this branch carries",
 because the fix landed as `aeb87871b3` and the branch's own two commits are not
 carried,
 the two verdicts whose issues closed after it,
 the `#1332` and `#4487` verdicts the tree has resolved,
 and the `#1768` triage whose link name is stale.

Proposal:
 port the pass,
 with the header note and the corrections this record lists,
 on a new branch from the integration.
That is the port this session made,
recorded in the section "The port this session made".

## The port this session made

Branch `fix/git-policy-cli-open-issues-fourth-pass`,
started from `integrate/git-policy-cli-jam-fixes-after-alpha-9` at `fc9f7f5f4b`.

Commit `7f84c90b1b`.
It carries `doc/planning/git-wrapper-open-issues-round-4.md` as the branch
`fix/git-policy-cli-open-issues-next-batch` wrote it,
with these edits:

- a header note recording where the pass came from,
  which commits wrote it,
  that its tree reading is `aca920cdf4`,
  and what was re-checked at `fc9f7f5f4b`;
- the two issues that closed after the pass,
  `#1907` and `#1911`,
  named as historical in their verdicts;
- `#4487` marked closed as fixed by `73ae19e83c`,
  and `#1332` marked fixed by `512bd213b7`;
- `#1768` carrying the correction its own comment of 2026-11-10T18:26Z makes;
- the open-issue count refreshed from 89 to 77;
- the pointer section retitled to "The fix this pass named",
  saying the fix landed as `aeb87871b3` rather than as the branch's own
  `08cdb8979a` and `67eea12838`,
  and recording that one data row in `doc/handover` is all the wider sweep left;
- the `#1698` count reproduced (33 paths cited,
  16 missing).

Build,
Clippy and tests:
 the port changes one Markdown file and no Rust source,
 so the crate's build and its tests are untouched by it.
The repository's only Markdown gate is `monochromatic-lint`,
run as `mise run lint:monochromatic-lint -- <paths>`,
and the ported file is clean under it.
One filtered test run was made to re-check the `#4487` control the pass names:
`mise run //package/git-policy/cli:native:test:host -- manual_push_range` gives
14 tests run,
14 passed,
6854 skipped,
`a_request_list_larger_than_a_pipe_is_written_while_the_answers_are_read` among them.
The crate was built for that run.
`mise run //package/git-policy/cli:native:clippy` on the same branch finishes in
1m 43s with warnings denied and none printed,
although no Rust file of this port differs from the integration's.

## What was run

- `git cherry -v`,
  `git log --oneline --no-merges`,
  `git log -S`,
  `git diff --name-status`,
  `git diff --stat` and `git diff --shortstat` against
  `integrate/git-policy-cli-jam-fixes-after-alpha-9` for all four branches.
- `git rev-list --left-right --count` and `git merge-base` for the distances in
  "The four branches,
  measured".
- `git rev-list --count` over the integration for the two port-cost measurements:
  749 commits over the paths
  `feat/git-policy-cli-commit-protocol-models` rewrites,
  and 11 over the four files
  `fix/git-policy-cli-commits-never-fail` edits,
  whose diff at the integration is 1773 deletions.
- `rg` over `package/git-policy/cli/src`,
  `package/git-policy/cli/SPEC.md`,
  `doc/handover/`,
  `doc/planning/` and `doc/decision/` for each decision phrase,
  each file name and each symbol the verdicts rest on,
  with a whitespace-normalized copy of SPEC for the phrases its line breaks split.
- `gh issue list --repo Aquaticats/Monochromatic --state all --search 'git-policy in:title,body'`
  for the state of the 15 issues and the open count,
  `gh issue view --json number,state,updatedAt` for each of the 15 and the two outside
  them,
  and `gh issue view --comments` for the comments the verdicts cite
  (2026-11-10T18:26Z on `#1768`,
  2026-11-10T19:48Z on `#4487`,
  2026-11-10T19:54Z on `#1332`).
- `git merge-base --is-ancestor` for `aeb87871b3`,
  `512bd213b7` and `73ae19e83c`,
  the three commits whose presence in the integration the verdicts claim.
- A `python3` pass over `doc/migration/no-class.md` for the `package/` paths it cites
  and a `test -e` over each:
  33 cited,
  16 missing.
- `mise run //package/git-policy/cli:native:test:host -- manual_push_range`
  on branch `fix/git-policy-cli-open-issues-fourth-pass`,
  which built the crate and ran the filtered tests:
  14 tests run,
  14 passed,
  6854 skipped.

## What this record does not do

- It merges nothing into `integrate/git-policy-cli-jam-fixes-after-alpha-9` or `main`.
- It deletes,
  renames and resets no branch:
  two of the four proposals need the owner's word before a branch goes away.
- It edits nothing in `doc/decision/`.
- It does not judge the quality of the design or of the models,
  only whether the integration already carries them and whether they are still wanted.
  The design's review verdicts ("not ready to implement as written",
  then the round 21 review's "not approval of the live protocol") are in the record on
  the branches and in the handover,
  and they still stand.
