# cli-git implementation handover

This page holds only the open work of the Git wrapper effort (`package/git-policy/cli`) and its current
state.
Nothing on it is approved or final:
it is an agent's account,
and only the owner declares work complete.
It was rewritten on 2026-11-26T18:55-05:00 to the state the coordinating session read then,
when the owner asked for the handover to be updated,
holding the current state and nothing that has landed.

What landed left this page for the record that owns it:
a root cause or a workaround went to `doc/troubleshooting/`,
a decision to `doc/decision/`,
a measured number to the page that measures it,
and unfinished work nobody is on to a GitHub issue.
`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` held the run logs from
2026-11-09 on until they were sorted the same way and the file was deleted (`f0f870e305`).

## How the work is laid out

The levels are Linear's,
by the owner's word of 2026-11-23.

- An **initiative** is a broad effort with a reason behind it.
  It lives on this page,
  one heading each under "Initiatives".
- A **sub-initiative** is one outcome inside an initiative that takes several runs to reach.
  It lives on this page under its initiative:
  the outcome,
  the state,
  what comes next,
  and where its records are.
- An **issue** is one run or one action with one end that can be checked.
  While it is being worked it is an entry in the coordinating session's task list,
  marked in progress only while a run on it is live.
  A defect,
  or work nobody is on,
  is a GitHub issue (`doc/agent/issue-tracker.md`).

Neither an initiative nor a sub-initiative is ever an entry in the task list.

## Standing directions

Each is the owner's,
and each stays in force until the owner changes it.

- The ZFS route comes first.
  Heavy use of worktrees waits for it,
  by the owner's word of 2026-11-18:
  "We shouldn't use worktrees heavily before we implement the zfs parts for the git wrapper."
  For the ZFS route's own work the width is the owner's of 2026-11-23:
  "Up to 64 rounds at once.",
  and later steps' pieces are built ahead of the plan's order,
  unwired:
  "Build them ahead,
  unwired (Recommended)"
  (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).
- A branch may not make anything `main` does well worse.
  The owner waived it once,
  for one case:
  `doc/decision/cli-git-zfs-dirty-main-restore-fallback-proof-window-and-slot.md`,
  "The merge bar is waived for an add above the warning's count".
- The merge bar,
  restated on 2026-11-25 and unchanged:
  a branch merges into `main` when its round has ended and its report is read,
  and no gate,
  pass,
  review or mutation run holds a merge;
  one whole container gate runs on `main` per batch of merges;
  a design branch merges as a proposal and stays one until accepted;
  Sonnet passes run only at a release,
  over everything merged since the last one
  (`doc/decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md`).
- An alpha release of any package may be bumped and dispatched unasked;
  any other release is asked each time
  (the same record,
  "Release dispatches,
  answered 2026-11-23").
- This session answers for the wrapper,
  the Pi driver,
  mpm and the way Cargo is run here.
  The owner,
  2026-11-13:
  "You're responsible for the driver,
  the wrapper,
  mpm,
  cargo.
  If you find a issue you or a subagent fix it."
- Every run is kept moving:
  when one ends,
  its result is read,
  recorded here,
  and its next step started in the same turn.
- No limit on runs is in force for memory
  (`doc/handover/agents-at-once-on-this-machine.md`,
  "The freeze of 2026-11-22 set no limit on runs").
- Who writes and who reviews:
  the skill `pi-as-subagent`,
  section "Who writes and who reviews".
- The owner,
  2026-11-23:
  "Do whatever you need to do,
  but don't abandon any work."
- Independent work runs at once.
  The owner,
  2026-11-27,
  between 20:10 and 20:12 -05:00,
  to a session that had five rounds live and ran its own steps one after another:
  "Update handover,
  tasks list,
  and parallelize better."
  Every ready item gets its round in the same turn,
  independent commands go out together,
  and a step waits only for an input it names or for a failure on record
  ("How work is run side by side").

## The state at 2026-11-27T21:42:08-05:00

`main` was `efae1c2767`.
What landed before this time is in its own records and closed issues,
and the merges are in `git log --merges`.
Sixteen rounds merged on the evening of 2026-11-26,
and `git log --first-parent --merges --since='2026-11-27 00:00'` counts 315 more by this time.

### Rounds running

At 2026-11-27T21:42:08-05:00 these 21 rounds are live,
counted from the process list.
Each runs on Pi's virtual writer,
with its brief at `~/temp/agent/pi-runs/<round>-prompt.md`,
its run under `~/temp/agent/pi-runs/<round>-1`,
its worktree `.claude/worktrees/pi-<round>`
and its report at `~/temp/agent/<round>-scratch/keep/report.md`.
The time after each name is that of its first feed line.

Guest rounds:

- `issue-6615-the-adoption-drivers-holder-scan-arm-runs-twice-in-one-guest-and-its-snapdir-pattern-means-a-dot`,
  2026-11-27T20:00:25-05:00;
- `issue-6644-the-template-guests-prepare-says-which-module-it-loaded`,
  2026-11-27T21:38:04-05:00:
  the one place issue 6638's round left,
  in guests made from plan step 23's template;

For what the reads of the wrapper's `0.6.1-alpha.51` found
("What the wrapper's alpha.51 reads found"):

- `issue-6641-a-command-behind-a-git-alias-is-judged-for-every-place-git-acts-in`,
  2026-11-27T20:32:58-05:00:
  the two High points;

For what the read of `mpm-shadow`'s `0.2.7-alpha.18` found
("What `mpm-shadow`'s alpha.18 read found"):

- `mpm-shadow-the-exposure-removal-joined-with-the-rust-rows-one-directory`,
  2026-11-27T21:39:47-05:00:
  joins the ended exposure-removal branch with `main`,
  where the Rust row's merge moved `exposure.rs`,
  and changes behaviour nowhere;

For what the read of `cargo-shadow`'s `0.1.5-alpha.17` found
("What `cargo-shadow`'s alpha.16 round did,
and its alpha.17";
issue 6645):

- `cargo-shadow-alpha-17-release-pass-a-start-that-heals-and-a-queued-stop-keep-the-delegated-route`,
  2026-11-27T21:37:37-05:00:
  the two Medium points and the control that holds them;
- `cargo-shadow-alpha-17-release-pass-five-record-sentences-say-what-is-so-and-two-controls-can-fail`,
  2026-11-27T21:36:31-05:00:
  the Low points;

For the ZFS route
("The ZFS worktree route"),
the first fourteen from the list `batch-of-2026-11-27-the-fourteen-zfs-route-rounds`,
briefed from a digest of what is left with every quoted record line checked against its file:

- `zfs-removal-command-path-runs-the-whole-removal-with-force-still-refused`,
  2026-11-27T21:12:04-05:00:
  the switch itself:
  `git worktree remove` of a bound clone runs the whole removal,
  the cut test goes,
  and the request budget gets its constant;
- `zfs-management-grammar-admits-the-cancel-and-settle-commands`,
  2026-11-27T21:12:07-05:00:
  merged before the switch,
  whose sentences name the cancel word;
- `zfs-doctor-gathers-the-machines-route-facts-from-the-live-system`,
  2026-11-27T21:13:06-05:00;
- `zfs-doctor-gathers-each-managed-clones-facts-from-the-live-system`,
  2026-11-27T21:13:15-05:00;
- `zfs-doctor-checks-that-steps-19-and-20-owe-and-the-cancel-commands-line`,
  2026-11-27T21:13:20-05:00;
- `zfs-private-propagation-step-built-ahead-unwired`,
  2026-11-27T21:13:38-05:00:
  the step and its guest arm,
  with no call from a root mode;
- `zfs-shutdown-unmount-body-built-ahead-unwired`,
  2026-11-27T21:13:44-05:00:
  the body and its guest arm,
  with the mode still refusing;
- `zfs-lost-event-triggers-wrapper-side-built-with-the-nudge-off`,
  2026-11-27T21:14:03-05:00;
- `zfs-lost-event-triggers-handler-side-built-ahead-unwired`,
  2026-11-27T21:14:23-05:00:
  the event filter and the merge device,
  with no call from the sweep;
- `zfs-forced-arm-ends-a-holder-in-a-taskless-mount-namespace`,
  2026-11-27T21:14:41-05:00:
  in a guest made from plan step 23's template,
  whose kernel has `listns`;
  steered once,
  at 2026-11-27T21:20:23-05:00,
  to read which module that guest runs after its prepare;
- `zfs-forced-arm-stops-a-locked-holders-own-user-unit`,
  2026-11-27T21:15:02-05:00;
- `zfs-step-19-written-guest-arms-and-the-reboot-arms-run`,
  2026-11-27T21:15:26-05:00;
- `zfs-step-21-guest-arms-for-dependency-repair-and-watches`,
  2026-11-27T21:15:44-05:00;
- `zfs-step-22-guest-half-gaps-pinned-output-and-a-parent-above-64-clones`,
  2026-11-27T21:16:02-05:00;
- `zfs-records-say-what-main-holds-where-seven-status-lines-were-passed`,
  2026-11-27T21:38:04-05:00:
  records only:
  the lines of other records that `main` has passed,
  and the two list entries `SPEC.md` runs together on one line;

The four rounds on this file's history work from the ledgers four read-only Claude Sonnet sorters wrote
(same vendor,
none independent),
`~/temp/agent/handover-prune-ledgers-2026-11-27/keep/ledger-part-<a|b|c|d>.md`,
scratch that expires a day after its last change.
The coordinator read part A's opening and none of them whole:
each round opens every home a ledger names before it deletes an entry,
writes a fact that has no home into a canonical doc first,
and keeps every open entry and every quotation of the owner.
Each edits this file on a branch,
so its report is read against the cut before its branch merges,
and `main`'s text stands wherever the two differ outside that round's part.

A branch that took `main` in before another merge changed its package is checked on the tree that holds both,
in a detached worktree of the coordinating session's own under `.claude/worktrees/`,
and pushed from that worktree's root:
the wrapper refuses a push typed below it,
which a check's own log shows and its summary line may not.
A round's worktree is gone once its run ends,
and the local branch it leaves stands at the round's tip,
so a branch pushed again after such a check is merged from `origin`'s ref.

When a round ends:
its report is read whole,
its branch merged as "Next" says,
its issue closed by hand,
and what its merge frees is started.
After the wrapper's last round of five:
one whole gate on `main`,
the install proof over alpha.51,
`0.6.1-alpha.52`,
and its two second passes.
After `mpm-shadow`'s join:
`0.2.7-alpha.19`.
After `cargo-shadow`'s two:
`0.1.5-alpha.18`.
The ZFS rounds all add lines to `src/native/lib.rs`,
the list of published names,
`SPEC.md`,
the README and the reach control,
so each is checked on the tree that holds the ones merged before it,
and the management grammar's round merges before the removal's switch.

### How work is run side by side

The owner's "parallelize better" of 2026-11-27 ("Standing directions") was earned.
The coordinating session had held `main` still through every release,
run its own commands one after another,
and left the ZFS route's next step unstarted behind a sequencing of its own that had ended.
None of those three holds named a failure on record.
What runs side by side now,
and the few orders that stay,
each with the failure it prevents:

- Every ready item gets its round in the same turn,
  all in one start list.
  One list,
  because three lists side by side overlapped 27 worktree adds and sixteen of thirty starts failed (issue 6563).
  A further list starts once `ps -eo args=` shows no `git worktree add` in flight.
- A release does not hold `main`.
  `crate-release.yml` commits its bump through GitHub's API on whatever `main` is at that moment,
  and when `main` has moved it sends the commit again,
  unless a path the release depends on changed
  (the crate,
  its lock directories,
  and the path dependencies `cargo metadata` reports;
  the workflow's own first comment says so).
  That is read from the workflow's comment and not from its step,
  and `main` did not move during the one release dispatched since,
  `cargo-shadow` `0.1.5-alpha.17`:
  the first release during which `main` moves is its test,
  and that run's log is read before the rule is leaned on again.
  So merges of other packages and commits of records go on while a release runs,
  and only a merge into the released crate's own paths waits for the bump commit.
  Before each commit or merge on `main` the session fetches,
  and takes origin's new commits by fast-forward when it is only behind.
- A release's follow-up runs as one background chain up to the point where `main` must move:
  the dispatch,
  the wait for `PUBLISHED`,
  and the lock on a branch in a worktree of its own.
  The session merges and installs when the chain reports.
- Releases are dispatched one behind another.
  `crate-release.yml` and `cargo-publish.yml` each sit in a `concurrency` group with `cancel-in-progress: false`,
  and the session dispatches the next once the one before has started its publish;
  what GitHub does with a third dispatch while one waits was not read.
- Merges into `main` are made back to back,
  and origin is waited for once,
  after the last.
  Each merge is still checked by itself (`HEAD^2` is the branch's tip),
  because on 2026-11-25 a `git merge` returned 0 and landed nothing;
  and the one wait is for the commit service's push,
  which a `GH009` refusal does not repeat.
- Branches of one package that ended together share one integration worktree and one run of the package's checks,
  because three `mpm-shadow` branches merged without a textual conflict on 2026-11-27 and the merged tree did not compile.
- Reads,
  measurements,
  issue filing,
  briefs and record edits that do not move `main` go out in one message and run at once.
  Two commands that move `main` in the main checkout run one after the other:
  it has one `HEAD` and one index.
- Every branch and worktree the coordinating session makes itself is made detached or with `--no-track`,
  a lock branch and an integration worktree alike,
  and `git rev-parse --abbrev-ref @{upstream}` has to fail there before the first commit.
  A branch that tracks `origin/main` has its commits pushed to `main` by the commit service,
  so an integration worktree made that way would put unchecked merges on `main`.
  The lock branch of `cargo-shadow` `0.1.5-alpha.17` was made with `git worktree add -b <branch> <path> origin/main`,
  which set its upstream to `origin/main`,
  and its commit (`7f21ae4ef9`) was on origin's `main` at 2026-11-27T20:30:27-05:00,
  the second the commit service's push of that branch ended:
  a fast-forward with no merge commit,
  which nobody chose.
  The commit holds only the lock row's version,
  address and checksum lines.

### Rounds that ended, and what each left

Every round runs on Pi's virtual writer,
pinned to no member
(`doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`),
whose members the owner changed on 2026-11-27 at about 16:15 -05:00:
"Change writer's list to match the reviewer's list."
On `main` the reviewer asks the qwen plan,
then Synthetic,
then `opencode-go`,
then Neuralwatt's flex lane,
and the writer asks those without Synthetic
(`package/pi-plugin/rate-limit-fallback/src/model-pools.ts`);
the round `the-virtual-writer-asks-the-members-the-virtual-reviewer-asks` built that
and measured what it costs:
a pool advertises the smallest window among its registered members,
Synthetic's member has 524,288 tokens where the others have about 1,000,000,
so the writer's window would fall from 1,000,000 to 524,288 and its output limit from 384,000 to 65,536.
Asked with those figures through the question tool at about 16:46 -05:00,
the owner chose "Skip too-small member (Recommended)":
the shared list stays,
the router passes over a member whose own window cannot hold the request,
and the pool advertises its head member's window,
so the writer keeps 1,000,000 and the reviewer rises to it.
Not chosen:
"Keep three members" and "Live as built".
The round `the-router-passes-over-a-member-too-small-for-the-request-and-the-pool-advertises-its-heads-window`
built that on top of the first branch and is merged (`45b9aaeb94`),
with the first branch inside it.
Pi loads that extension from the main checkout's built copy,
which the coordinator rebuilt at 2026-11-27T17:28:33-05:00
(`mise run //package/pi-plugin/rate-limit-fallback:build`),
so the change is live for every Pi run started after that,
and a run started before it keeps the list it loaded.
Asked before and after the rebuild,
that built copy answered:
before,
the writer asks three members and the reviewer advertises 524,288 and 65,536;
after,
both pools ask the four members and advertise 1,000,000 tokens of window and 384,000 of output,
and with the head member unregistered both advertise the next member's 524,288 and 65,536.
The head is the first member of the pool's order that the session's registry resolves.
The size of a request is Pi's own estimate,
which the round measured against the counts real providers reported:
over 200 sessions and 48,805 requests the estimate divided by the reported count had a median of 1.000,
with 0.999 at the 5th percentile and 1.002 at the 95th.
The router compares the input alone with a member's window,
because Pi fits the answer into what the window leaves,
and it keeps no margin.
Those three picks wait in `doc/handover/questions-for-the-owner.md` with the veto open.
A case of the driver's suite failed once in a whole run of that round and passed alone and on `main`:
issue 6631,
whose round is merged (`ae937863bc`) and whose issue is closed.
One timed whole run showed the case's bound,
three polls,
left 177 ms of room while its own process was held up for as long as 94 ms by its sibling cases.
With one named step made a poll slower,
the case ended on its wall clock,
which is the failure the issue quotes.
The cases of that file that end on an event now take their bound from the looks they need,
each look counted with the longest wait a look can be handed,
so no loaded host decides them;
the cost is that a watch that never reaches its event is reported after as long as 181.8 s.
The watch itself is unchanged.
The cases that assert the wall clock keep their bounds,
and one of them still fails if a timer fires about 285 ms late.
The suite's 228 lines passed after the merge of `main`.
Three things of that round wait for the owner in `doc/handover/questions-for-the-owner.md`.
The rounds started at that minute are the first on the rebuilt copy.
The first of them to end,
the round of issue 6630,
recorded a context window of 1,000,000 tokens in its `summary.json`,
with no retry and no rate-limit error,
which closes the check of the owner's change at a real run.
The owner also wrote,
of one `advisor` tool result in a round's session that names `neuralwatt/nw-large`:
"nw-large is advisor,
and that's fine."
The rounds run
under the owner's "Up to 64 rounds at once."
(`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).
All were started through the driver package,
with the brief at `~/temp/agent/pi-runs/<round>-prompt.md`,
the run under `~/temp/agent/pi-runs/<round>-<n>`,
the unit `pi-<round>`,
the worktree `.claude/worktrees/pi-<round>`
and the whole report at `~/temp/agent/<round>-scratch/keep/report.md`:

- rounds started from the lists of 2026-11-27T15:47:54-05:00,
  2026-11-27T15:51:32-05:00 and 2026-11-27T16:01:42-05:00,
  each for one tracked piece of work and each named under its tool:
  the linter's alpha.21 read,
  the `mpm-shadow` sentences,
  issue 6623 (`cargo-shadow`),
  and the rounds of the wrapper's alpha.50 reads.
  The router's round is merged (`45b9aaeb94`),
  and the passage on the writer's members says what is live.
  The round for issue 6622 is merged (`e56b8abaec`) and the issue is closed.
  Nothing warns about bytes that end with one line feed:
  on `main`'s code,
  in each state it built,
  the `final-newline` check read the bytes the commit writes,
  and the released alpha.44 and alpha.50 answered the same,
  each through its own service in a fixture,
  since a fixture without a manager refuses a commit before any finding is printed.
  The recorded line carries the hint a lifecycle prints when it corrects nothing,
  which is what `git add` prints,
  and the commit that follows corrects the file;
  so the warning was most likely true when it was read.
  The branch added controls and no release code.
  That hint tells a person to edit a file the next commit corrects,
  which is issue 6627;
  its round is merged (`0f69dbfe93`) and the issue is closed.
  At an `add` and at a direct check the hint now says a commit that takes the file corrects the ending,
  and names the `fix` command for correcting it now;
  the sentence that says to edit the file stays for a commit whose selection is read-only,
  a landed commit and a manual push,
  where nothing corrects the bytes.
  The round measured both steps on fixtures,
  and found no `add` at which nothing corrects the file,
  which the brief had asked a control for.
  The push of that merge was refused by GitHub with `GH009` and cleared by one push by hand,
  as `doc/troubleshooting/github-lfs-integrity-rejection.md` records with the others.
  Every round of the list of 2026-11-27T14:54:37-05:00 is merged:
  issue 6620 (`bc24b2e688`),
  issue 6619 (`1ef6287246`),
  issue 6621 (`71f1c34d94`),
  and the rounds of the `mpm-shadow` alpha.15 read.
  Issue 6621's round changed no release code.
  Its controls show the bound-slot refusal for a removal and for a move typed below the top behind a shell alias,
  with `policies.require-root` at `off`,
  at `warn`,
  and behind `--no-enforce-require-root`,
  each shown red by handing the planning the typed directory;
  `SPEC.md` says what such a command meets.
  It found that a refused move drops the root rule's `warn` lines,
  which a refused removal prints (issue 6624);
  the wrapper round on a bare shell alias takes that issue,
  because both change `move_refusal_of`.
  A command standing in the repository's Git directory or in a bare repository has no such control:
  the round's plant changes nothing there,
  so a control of those shapes needs another plant to be shown red
  (issue 6625,
  which starts when the wrapper round on a bare shell alias has merged,
  because both change one test file).
  The round's first edits went to the main checkout's copy of its test file,
  by a path written without the worktree's prefix;
  it reversed them,
  and the main checkout shows only the files other sessions modified.
- the guest drivers.
  Issue 6612's round is merged (`98996296fb`) and the issue is closed:
  the three sets of shared guest-step helpers are one,
  `bin/mvm-guest-*.ts`,
  with `bin/guest-route-*` and `e2e/guest-host/` deleted and every case their tests held restated against the survivor.
  A stand-in `mvm` and `node` recorded every step of the eleven drivers on `main` and on the branch:
  of 44 recordings 41 are identical line for line,
  and three differ on purpose.
  The managed-lock driver builds only the target its arm runs
  (11m 31s where it took 22m 22s,
  and a guest peak of 2.83 GiB where it was 6.70 GiB),
  and two drivers build the library's test binary at one job,
  since two left under half a gibibyte available in a guest of 8 GiB.
  Every driver's main step ran in a guest and passed with its own lines,
  and the start probe's three arms nobody had run after the rewrite ran once each and passed.
  The merged tree passed `lint:types`,
  oxlint and the package's unit suite (177 suites) in a worktree of its own before the merge.
  Three rounds followed it from one list.
  Issue 6639's is merged (`5d91792960`) and the issue is closed:
  the start probe's guest build runs as a detached script,
  the host learns its end from the log's own `DONE=<status>` line and pulls the log,
  and no guest command's return carries the result.
  In one guest `probe-dataset` exited 0 and `green` built and ran its arm in 7m 11s;
  a stand-in `mvm` shows the steps that do not build unchanged line for line,
  and the script the guest runs the same text on both sides.
  A build that fails is reported with its own status and the tail of its own log,
  shown against a stand-in and held by unit tests.
  The round listed every other plain guest command bounded past five minutes
  (`package/git-policy/cli/e2e/README.md`,
  "Guest work that runs longer than the agent's safe length")
  and changed none.
  Its four open points wait in `doc/handover/questions-for-the-owner.md`.
  Issue 6638's is merged (`1471bdf6ad`) and the issue is closed.
  Measured in the recipe's own guest before any edit,
  a second `prepare` alone exits 0;
  what fails is a `prepare` while a dataset is mounted,
  because a mounted dataset is what holds the module
  (`/sys/module/zfs/refcnt` at 1,
  where a pool imported with nothing mounted leaves it at 0 and the removal succeeds).
  One step,
  `e2e/load-the-built-module-step.ts`,
  now stands at the four places that removed and inserted modules:
  it reads which module is loaded against the version the installed file states,
  leaves a guest that already runs the built module alone,
  and refuses in words when a module of another version is held.
  A `prepare` with a pool imported and a dataset mounted exited 0 and left the pool,
  the version and the count as they were,
  and one driver's whole `all` step passed in a fresh guest.
  A host check,
  `test:tooling`,
  runs the step's shell against stand-in trees.
  Left as it was:
  the template's own prepare,
  `prepareFromTemplate`,
  which still loads with `modprobe --quiet zfs 2>/dev/null || true`;
  that is issue 6644 and has its round ("Rounds running").
  The round of issue 6615 is live ("Rounds running");
- the linter,
  whose round on what the read of `0.5.2-alpha.22` found
  ("What the linter's alpha.22 read found"),
  `monochromatic-lint-alpha-22-release-pass-a-list-lines-sentence-is-true-for-the-name-it-wrote-and-its-controls-can-fail`,
  ended 2026-11-27T19:09:36-05:00 with its report read whole.
  Its branch is merged (`f7b79c5aa8`),
  after the wrapper's bump commit,
  because the wrapper names the linter by path.
  The release run of `0.5.2-alpha.23` was started at 2026-11-27T19:21:22-05:00
  (<https://github.com/Aquaticats/Monochromatic/actions/runs/46087442769>);
  its release commit is `9fc0e387c3`,
  which changes version rows in eight files,
  the wrapper's and its fuzz crate's among them,
  and `main` is fast-forwarded to it.
  It is published
  (publish run 46087537570;
  the follower's log holds `PUBLISHED monochromatic-lint-v0.5.2-alpha.23`),
  locked (`6d99c6a2da`) and installed here,
  where `.cache/toolchain-bin/monochromatic-lint --version` says `0.5.2-alpha.23`;
  the install named no unresolved bin,
  and the session's `git` link was put back at 2026-11-27T19:39:58-05:00.
  Its Claude Sonnet second pass,
  a same-vendor read,
  returned and was read whole
  ("What the linter's alpha.23 read found").
  It took Lows 5,
  7,
  8,
  9,
  1 and 2,
  the absolute-path half of Low 4 and the dead-line help of Low 6.
  Measured by the round with the read's own generators,
  against the released `0.5.2-alpha.22` and a byte-identical copy of it:
  of 10,081 lists none moved between read and refused,
  and none changed its exit status or its set of findings;
  of the read's 5,600 matrix lines,
  the 600 written for an arrow-named folder whose own arrow a tab,
  a no-break space or two spaces sets off now name that folder,
  where none did,
  and the 200 for a folder whose name begins with a space all name it,
  where 72 did.
  The package's tests ran 1172 and passed 1172,
  and its binary reports the same 19 findings on the wrapper's package as the locked release.
  The choices its brief did not decide wait in `doc/handover/questions-for-the-owner.md`.
  The round for the Claude Sonnet read of `0.5.2-alpha.21`
  ("What the linter's alpha.21 read found") is merged (`83d10dd296`),
  and `0.5.2-alpha.22` is released
  (release commit `f10132d60f`,
  published by the follower's log,
  whose run ended 2026-11-27T17:21:26-05:00),
  locked (`0b90e1d87e`) and installed,
  with the session's `git` link put back at 2026-11-27T17:23:24-05:00.
  Its Claude Sonnet read,
  a same-vendor second pass,
  started 2026-11-27T17:21:51-05:00 and wrote its review at 2026-11-27T18:12:31-05:00:
  `~/temp/agent/monochromatic-lint-alpha-22-release-review-2026-11-27/keep/review.md`.
  A `file-uses` line of six words now names the folder its words name for every spelling of its gaps:
  1,152 of the read's 1,296 lines,
  against 324 in alpha.21,
  and the other 144 name a folder that holds no whitespace and read as an entry.
  A five-word line with another word in the arrow's place is told which word belongs there.
  Every character that prints as nothing is spelled in a message:
  4,266 of 4,266 candidates of Unicode 17.0.0,
  against 7 in alpha.21,
  decided by general category and by ranges of `Default_Ignorable_Code_Point`,
  with a control that fails when the linked tables move to another release.
  A character's place is a column of the line.
  Over 6,293 lists no line moved between read and refused,
  with a byte-identical copy of alpha.21 as the control.
  Its whole suite passed 1,158 of 1,158 after its last merge of `main`,
  and its four picks wait in `doc/handover/questions-for-the-owner.md` with the veto open.
  The round for the `0.5.2-alpha.20` read is merged (`2261d05b14`),
  and `0.5.2-alpha.21` is released
  (release commit `117514dfd7`,
  locked `6e3314812e`,
  installed).
  What that merged round did,
  as its report gave it and before the alpha.21 read qualified the first point:
  A `file-uses` line for a folder named with a tab,
  a no-break space or two spaces names the folder again;
  each of the five behaviours the read named has a test shown red by the edit it guards;
  a byte order mark or a zero-width space is spelled,
  with its place and the edit that takes it out;
  the dead-line help names the one edit;
  each shape of arrow name gets a fix that fits it;
  a mistyped `file-uses` line says what is wrong with it;
  and three README sentences say what the releases did.
  The reader does not take a byte order mark off by itself,
  which the README's known limits record;
  `test:layout` passed 326 of 326;
- rounds for the wrapper's `0.6.1-alpha.50` reads,
  started 2026-11-27T16:01:42-05:00,
  each named `wrapper-alpha-50-release-pass-` and then:
  `a-bare-shell-alias-is-judged-where-its-own-options-point`
  (with issue 6624),
  `a-hunk-a-side-also-wrote-into-keeps-no-line-a-side-removed`,
  `two-unwired-sentences-show-a-name-safely-and-three-records-say-what-holds`,
  `the-surface-control-reads-associated-constants-and-long-headers`,
  and `each-list-guard-says-what-it-matches-and-an-excuse-names-its-use`
  ("What the wrapper's alpha.50 reads found").
  All change `package/git-policy/cli`,
  each brief names the others' files,
  and the published-surface list is where their merges meet.
  After their merges come the tip checks,
  one whole gate,
  and `0.6.1-alpha.51`.
  The round on the list guards is merged (`eaef6b630c`),
  after two checks of the merged tree in a worktree:
  the linter's release commit landed between the first check and the merge and changed four of the package's files,
  so the tree was checked again with it,
  and format,
  check,
  Clippy,
  the whole `source_guard` module (225 of 225) and the five other modules whose guards changed passed both times.
  All five rounds for the reads are merged,
  the round on a bare shell alias last (`461d441ef7`).
  A guarded command behind a bare `!git` alias is judged for the worktree its own words name,
  whether they say `-C`,
  `--git-dir` or `--work-tree`:
  before,
  a reset of a dirty main checkout typed from a clean linked worktree ran,
  and the reverse was refused.
  One control compares the alias spelling with the direct one for 17 guarded commands,
  in both directions and three spellings of the target.
  The slot refusals read the records of the repository those words name.
  A removal,
  a move or a prune behind a `!` alias whose shell directory Git could not be asked for is refused,
  with words saying what could not be read,
  where it was forwarded with no refusal;
  a prune only when its own words name a repository.
  Every refused byte of the alias reader has a row,
  each shown red by widening the reader with it,
  and `SPEC.md` and the README state the bytes the reader trusts.
  Every slot refusal keeps the lines the policy stage printed before it,
  which closed issue 6624.
  That round ran the modules it changed and the whole `source_guard` module,
  and no `binary::` module and no gate.
  It put two more files of `invocation/` over the line budget,
  `entry_forward_action_planning.rs` and `global_arguments.rs`,
  and added one `group-file` line to the folder rule's allow list for a file it made;
  nine files of that folder are over the budget now,
  and their splits run after the crate's move
  (`doc/planning/cli-git-source-layout-target-tree.md`,
  the note of 2026-11-26T20:03:52-05:00).
  Its three picks wait in `doc/handover/questions-for-the-owner.md`.
  The tip checks passed on `beb61ec46b`,
  from 2026-11-27T18:40:16-05:00 to 2026-11-27T18:46:23-05:00 in a worktree no merge moved:
  `native:check`,
  the four Clippy tasks,
  the `source_guard` module (225 of 225),
  `native:ts-checks` and `lint:oxlint` with no warning and no error.
  The whole gate passed on `main` at `beb61ec46b`,
  started 2026-11-27T18:39:13-05:00 and read 2026-11-27T19:05:33-05:00:
  `12307 tests run: 12307 passed (7 slow), 51 skipped`,
  `gate-exit-status=0`,
  and no failed,
  flaky or signalled line in its log
  (unit `gpc-gate-wrapper-next-alpha-on-beb61ec46b-20261127T183913-0500`,
  log `~/temp/agent/gate/gpc-gate-wrapper-next-alpha-on-beb61ec46b-20261127T183913-0500.log`,
  the tip checks' logs in `~/temp/agent/wrapper-alpha-51-gate-and-tip-checks-2026-11-27/keep/`).
  Measured 2026-11-27T19:10:12-05:00 against `main` at `af1647ef36`:
  no path changed since the gated commit under the wrapper's package,
  under the sources of the crates it names by path
  (the ZFS request crate,
  `jsonc-edit`,
  `catalog-specifier`,
  the scanner's `src/` and the linter),
  under `package/crate-release` or in the release workflow,
  so the tree a dispatch builds is the gated one.
  The release run of `0.6.1-alpha.51` was started at 2026-11-27T19:16:27-05:00 from `e4c3117285`
  (<https://github.com/Aquaticats/Monochromatic/actions/runs/46087009444>);
  its release commit is `81b06993bd`,
  which changes one version row in each of four files,
  the crate's and the fuzz crate's `Cargo.toml` and `Cargo.lock`,
  and nothing else,
  and `main` is fast-forwarded to it.
  It is published for both targets
  (publish run 46087105171;
  the follower's log holds `PUBLISHED monochromatic-git-policy-cli-v0.6.1-alpha.51`),
  and unpacked at `.cache/wrapper-release-0.6.1-alpha.51/`,
  where the binary's version line says `0.6.1-alpha.51`.
  Its install proof over alpha.50 passed in a disposable container at 2026-11-27T19:27:02-05:00
  (`THE PROOF PASSED: the instance the older release loaded started as this release, and the pair of commits each landed once`;
  log `~/temp/agent/wrapper-alpha-51-install-proof-over-alpha-50-2026-11-27/keep/install-proof.log`).
  Two Claude Sonnet second passes of it,
  same-vendor reads and not independent reviews,
  one on the merges that change what a person meets and one on those that claim to change nothing,
  returned and were read whole
  ("What the wrapper's alpha.51 reads found").
  The round of issue 6625 is merged (`3caae5b48d`) and the issue is closed.
  It changed no release code:
  measured with Git 2.56.1 at a repository's top,
  in a folder below it,
  in its Git directory,
  in a folder inside that,
  in a bare repository,
  in a folder inside it and at a linked worktree's top,
  the directory the shell of a `!` alias prints
  and the directory `alias_run_directory` answers are the same everywhere.
  Its controls hold the bound-slot refusal for a removal,
  a move and a prune,
  typed directly and behind a shell alias,
  in a Git directory and in a bare repository,
  each removal and move compared word for word with the spelling typed at the top;
  three plants turned them red.
  `SPEC.md` says what a bare repository's records are.
  One shape is left to Git as before:
  a slot command typed with `-C` at a directory that resolves to no repository.
  The next gate on `main` is the first to run these controls in the container.
  Nine planted spellings passed their guard on `main`:
  a renamed import,
  a function taken as a value,
  a call with its types written out,
  another crate's name for the same call,
  a handle reached through a module,
  a macro defined outside the guard's scope,
  and a key passed through a constant.
  The call reader reads each now but the macro,
  which every guard's top doc names as unseen,
  beside a glob import and a line break between a name and its parenthesis.
  An excuse names the line of the one use it covers,
  where it named a count,
  so a use removed and another added fails twice;
  and the two exemptions that named no file are entries now.
  Its question about the line-break rule is answered by "The Markdown the line-break sweep left out":
  the package's findings were that sweep's,
  and the linter's own fix took them out after the alias round's merge (`beb61ec46b`).
  It reported that the rule broke an inline code span;
  a probe of `0.5.2-alpha.21` on a Markdown file found no finding for a comma inside one,
  beside two findings for a plain sentence on the next line,
  so the round's instance is not reproduced there.
  The round on a hunk a side also wrote into is merged (`85be73acf1`).
  An answer of the word merge is refused when it holds more copies of a line the base held than the two sides leave,
  for every shape of region,
  where the earlier rule judged only a region both sides deleted from;
  and a file a rule refuses keeps both sides of every region the sides disagree about.
  Over the read's own 2,300 seed cases,
  run through this crate's merge entry,
  21 of 427 resolved answers broke that bound on `main` and none does now;
  those 21 are conflicts now,
  and no answer moved the other way.
  Where both sides wrote more copies of one line than the base held,
  nothing judges the answer,
  as before.
  Its merge time against `main` sat inside the spread of two byte-identical copies.
  Its planning record's heading carries a round time,
  which the coordinator corrects from the commit's own.
  The round on the surface control is merged (`35fdc1797c`),
  after a check of the merged tree in a worktree,
  because two merges had changed the package after the branch last took `main` in:
  format,
  check and Clippy were clean,
  the whole `source_guard` module passed 219 of 219,
  and the modules the two merges and the narrowing touched passed.
  Planted shapes showed what the control missed on `main`:
  a constant of a published type,
  a member whose body opens on a later line,
  a gate that names the test inside a negation,
  a re-export under another name,
  a bare re-export read from the wrong module,
  and two functions qualified twice that collapsed into one wrongly keyed entry.
  The reader sees each now,
  with a control per shape that failed first.
  Ten names it newly saw are narrowed,
  because nothing outside their module reads them;
  one alias with no reader is deleted;
  and one line is added for an alias the binary's integration target names.
  Both rounds' picks wait in `doc/handover/questions-for-the-owner.md` with the veto open.
  The round on the two unwired sentences is merged (`e772d8ad66`).
  Both sentences show the name through `shown_so_it_cannot_rewrite_the_line` and stand in the guard's list;
  `CommandWord::of` keeps the kernel's bytes and escapes nothing,
  so a name is escaped once,
  at the sentence.
  A sentence that shows a name through a local of another spelling is not seen by the guard:
  the round planted one,
  measured two wider scans over the release sources and found each reports the program's own words,
  and the guard's top doc now says what a reader checks.
  `SPEC.md` says which lines the rule covers and what a record written as JSON does with a name.
  No process that serves more than one command reads the identity file
  (`main.rs` chooses the service and the handler's modes before the entry runs;
  one run of a serving process said the sentence zero times,
  with no commit through it,
  because a fixture has no manager),
  and the README says the sentence is said once for a command.
  The README's bullet on other targets says one thing,
  resting on the sources' imports and on one dated check that cannot be repeated on this machine;
  a clause that called two dependencies unix-only was false and is gone.
  The whole `source_guard` module passed 213 of 213 after its last merge of `main`,
  and its picks wait in `doc/handover/questions-for-the-owner.md` with the veto open.
  Of the `0.6.1-alpha.49` behaviour pass,
  all four rounds are merged,
  and the whole gate on `main` at `3aab810dd9` passed
  (unit `gpc-gate-wrapper-next-alpha-on-3aab810dd9-20261127T142758-0500`,
  started 2026-11-27T14:27:58-05:00,
  `gate-exit-status=0`):
  `Summary [ 883.520s] 12261 tests run: 12261 passed (6 slow), 51 skipped`
  and "Clippy passed beside the test run.".
  No file of the wrapper package changed between that commit and the release.
  `0.6.1-alpha.50` is released
  (release commit `3458afd18e`,
  published 2026-11-27T14:58:49-05:00,
  unpacked at `.cache/wrapper-release-0.6.1-alpha.50/`,
  never locked or installed).
  Its install proof over alpha.49 passed in a disposable container at 2026-11-27T15:01:26-05:00,
  on the gate's own image:
  "starts that failed at the EXEC step:
  0",
  no unit at the start limit,
  none left failed,
  and "THE PROOF PASSED:
  the instance the older release loaded started as this release,
  and the pair of commits each landed once".
  Two Claude Sonnet reads of it run,
  same-vendor second passes:
  of the four merges that change behaviour since 2026-11-27T15:02:11-05:00,
  writing `~/temp/agent/git-policy-cli-alpha-50-release-review-behaviour-2026-11-27/keep/review.md`,
  and of the two merges on guards and reach since 2026-11-27T15:02:41-05:00,
  writing `~/temp/agent/git-policy-cli-alpha-50-release-review-reach-and-guards-2026-11-27/keep/review.md`.
  A merge that lands after that commit and changes only the package's TypeScript,
  its guide or test code leaves the gate standing,
  as one did for alpha.49 ("Releases of 2026-11-27"),
  with the tip checks run again on the pinned tree;
  issue 6612's branch changes only `bin/` and `e2e/`.
  The round on the four small points is merged (`203bd65854`):
  the four reads of the session's runtime directory and bus address go through the one file,
  so a test build handed the process's whole environment cannot reach the account's manager;
  the repository link's sentence is said once a command;
  a failed start is held by a control at each place that answers it,
  and the scan of `exec` calls judges each call;
  and a control holds what a failed memory reading does today,
  with no change to it.
  The round on shell aliases is merged (`a137924570`):
  a slot removal or move behind a shell alias reads its operand from the top of the working tree,
  where Git runs the alias,
  and the same one line fixed the move;
  the five unread spellings are read
  (a shell alias naming another to four deep,
  `! git`,
  a tab after `!git`,
  a bare `!git` before the caller's own global option,
  and a chain of plain aliases to 64 where it was 16);
  a chain past either bound is passed to Git as an alias the wrapper cannot follow was before,
  with no new refusal;
  and `SPEC.md` says which spellings are read.
  The read's subdirectory case never reaches the slot rule:
  the wrapper's own rule that a command runs at the top of its repository refuses it first,
  and a control pins that.
  The guest arm gained one alias case,
  and one guest run of that arm passed on the tree at `203bd65854`,
  whose wrapper source is that of `0.6.1-alpha.50`
  (`mise run //package/git-policy/cli:e2e:zfs-slot-state -- all`,
  from 2026-11-27T14:45:24-05:00 to 2026-11-27T15:08:52-05:00):
  the arm printed `REMOVAL-REFUSAL wrapper-through-a-shell-alias status=refused` among its lines,
  skipped none,
  and ended `Summary [ 0.753s] 1 test run: 1 passed, 1369 skipped`;
  the driver destroyed its guest.
  A repository can switch the root rule off
  (`policies.require-root` takes `off`,
  `warn` or `error`),
  and controls now show the slot refusal holding below the top then
  (issue 6621,
  merged as `71f1c34d94`).
  The round on process names is merged (`946cbeb0a2`):
  it found eighteen places that printed a name another program chose with no escape,
  where the read named five,
  and each now escapes for what wraps the name in its sentence,
  parentheses or backticks,
  through one function;
  the weaker second escaper is gone,
  and its masking of a URL's user name stays as its own function;
  a guard lists the eighteen sentences and fails on a `format!` that reads such a name without the escape.
  With a program named to close the parenthesis and erase the line,
  the released alpha.49 printed the name raw in one of two sentences of one wait,
  and the branch's build escapes it in both.
  The whole `source_guard` module passed on the pinned tree at `946cbeb0a2`,
  203 of 203,
  which is the first run of the three merged rounds' guards together.
  The round on the merge pass is merged (`739bccb167`):
  the merge pass no longer writes back a line a side removed when both sides only deleted lines.
  It refuses an answer that holds more copies of a line than both sides hold,
  or whose lines are in an order one side does not hold,
  and keeps both sides between markers.
  The read's three searches reproduced on `main` to the case
  (6 of 75,
  32 of 199 and 47 of 521 resolved answers out of bounds);
  with the fix 82 of them became conflicts,
  none moved the other way,
  and no answer within bounds became a conflict.
  The three left over are Git's own answers,
  where no conflict reaches the pass.
  The cause was the rule's shape:
  it fired only where one side held the line as often as the base did,
  and it counted inside the region Git left.
  No timing was taken;
  `native:check` passed on the pinned tree after the merge;
- nothing for the wrapper's `0.6.1-alpha.49` structure pass:
  both rounds are merged.
  The round on the surface control (`b929e8c39b`) first planted the read's five shapes against the Rust control on `main`,
  and each passed unseen while a plain `struct` failed.
  The control now reads a function whatever qualifies it,
  fields and methods of a published type,
  and a `pub use` over any count of lines,
  each keyed by the module path that publishes it;
  its list went from 6,695 entries to 12,395;
  four names a split had widened are narrowed,
  and none of the rest;
  and its top doc states the five shapes it still cannot see.
  The merge restored seven platform gates,
  counted in its diff.
  The round on the list guards is merged (`e8d3ba3736`):
  the hook ban fails on every spelling of a call that makes a link,
  and on no call that only reads one;
  an excuse names how many uses of its text its file holds,
  so a use added to an excused file fails;
  two guards read a whole folder where they read a name prefix;
  each guard's top doc says what a file would have to look like to be missed;
  the package guide holds the rules the two merges brought;
  and the README says which targets the release builds.
  Each of the read's plants was first run against the Rust guards on `main`,
  and each finding held.
  Two scopes stay open by the round's own account:
  a link written outside `commit/`,
  `landing/` and the three named files is no finding,
  and the commit path's notes guard is chosen by the words a file names.
  On the pinned tree at `203bd65854`,
  all six merged rounds together,
  `native:check`,
  the four Clippy tasks and the whole `source_guard` module (212 of 212) passed,
  and `native:ts-checks` passed with 90 unit test files and 189 suites.
  That last run reached the account's service manager (issue 6620):
  a unit test commits in fixture checkouts through whichever `git` the worktree's launcher names,
  which there was the locked `0.6.1-alpha.44`,
  so four commit services started for fixtures at 2026-11-27T14:31:32-05:00
  and the shared program link named that release until 14:33:15.
  Nothing was left:
  the four instances read inactive and the fixtures were gone.
  The pinned tree's launcher link names the reviewed build since then,
  as the main checkout's does,
  so the shared link stays where it is;
  issue 6620's merge ended the services as well.
  The task cannot be kept off the manager from outside,
  because it starts each check in a scope of its own:
  with a fixture runtime directory both checks ended `exit 1` before running anything.
  The whole gate named under the behaviour pass covers these too;
- no round is left for the wrapper's two alpha.48 release passes ("Releases of 2026-11-27").
  Every finding of the two passes that earned a fix is merged:
  the guards that named a file code had left now read the folders the splits made,
  ten pairs in eight files,
  and no plant found a violation in release code (`97d9afb21c`);
  the slot refusals read every shell alias the guard can follow (`cd62517318`);
  the merge pass reaches a last line with no line end (`f583a0d9bf`),
  where a generated control also found and fixed a correct merge that was refused;
  a notice's record stays until its line is known written (`be93c14734`),
  kept in 20 of 20 runs with the reader gone against none of 20 on `main`;
  the README's layout sentences say how to count each list in place of seven stale counts (`0e7297a9fd`);
  the items a split had made bare `pub` are narrowed,
  seven platform gates are back over the 35 items they reached before the splits,
  the seven platform gates the splits dropped are back,
  and a control with a list beside it fails when a name becomes nameable from outside the crate (`02d0a6e8e6`);
  a merge record that cannot be read says which file to remove,
  each batch of the service's work reads the room it has when it starts
  (its width is decided once,
  so every wave of that batch runs on the room of its start),
  and a holder's name cannot rewrite the line that prints it (`14e54207de`).
  The gate on `main` at `97d9afb21c` passed and `0.6.1-alpha.49` is released ("Releases of 2026-11-27");
  both reads of it have returned.
- rounds for `cargo-shadow`:
  both rounds of its `0.1.5-alpha.14` read are merged,
  and `0.1.5-alpha.15` is released
  (release commit `d9f0c8ea32`,
  locked `bb47735936`,
  installed);
  its Claude Sonnet read returned
  (`~/temp/agent/cargo-shadow-alpha-15-release-review-2026-11-27/keep/review.md`;
  "What `cargo-shadow`'s alpha.15 read found"),
  and two rounds started from it at 2026-11-27T15:20:50-05:00,
  each named `cargo-shadow-alpha-15-release-pass-` and then:
  `the-populated-read-answers-as-the-kernel-is-shown-to-and-each-row-is-held`
  and `the-refusal-says-once-which-later-runs-settle-and-the-records-match`;
  and one more,
  `issue-6623-the-route-questions-asking-again-is-sized-from-a-measured-manager-reexec`,
  is merged (`e595486ec8`).
  It measured every ask of the discovery across reexecs of a user manager,
  in a container on Linux 7.3.5 with systemd 262
  and in a guest on Linux 6.8 with systemd 255:
  every refusal fell inside a reexec window,
  the longest 0.588 s,
  none in the five seconds after one,
  and a reload refused none.
  The question is now asked five times,
  200 ms apart,
  which spans 0.800 s.
  One control rides a stand-in refusal of 0.600 s;
  another reexecutes a container's own manager,
  and on a host it prints one line and passes,
  so only its run inside a container shows anything.
  The ten failures of that morning came after the reexec had ended,
  no measurement reproduced them,
  and their words were not kept:
  issue 6628 holds that,
  with nothing to build until a red run's output is kept whole.
  The commit's `Closes` line closed issue 6623 at the merge;
  a comment on that commit says `Refs` is the accurate word.
  That span cost a call 883.1 ms at the median where no user manager can be reached,
  against 43.2 ms for `0.1.5-alpha.15`,
  so `0.1.5-alpha.16` waited for the round of issue 6629,
  which is merged (`aa7b4eb927`).
  The discovery now reads in its own process whether anything on the host could carry an ask:
  the runtime directory and the bus address,
  the manager's own socket and directory,
  and the bus socket.
  Where nothing could,
  the first failed ask is the answer.
  It matches no words of another program.
  Measured again with the same harness,
  in both ways of having no manager:
  one ask per call in all 72 calls,
  and the branch's call at 1.01 to 1.28 times the released binary's per round with neither variable set
  (median 1.11),
  and at 0.87 to 1.05 times with a runtime directory that holds nothing
  (median 1.00),
  while byte-identical copies moved as much.
  The control that reexecutes a container's own manager passed three times with the change,
  and failed there with the asking again taken out.
  Sampled across real reexecs,
  the manager's socket was absent for 0.17 ms and for 0.28 ms in one run and never in another,
  and the manager's directory was there in every sample,
  which is why a reexec does not read as no manager.
  One residue:
  a host whose manager died and left its socket file or its directory behind is still asked five times.
  Its commit's `Closes` line closed issue 6629 at the merge,
  and the coordinator reopened the issue for that residue.
  The release run of `0.1.5-alpha.16` was started by 2026-11-27T17:51:43-05:00
  (<https://github.com/Aquaticats/Monochromatic/actions/runs/46079151776>);
  its release commit is `4338272860`,
  which changes the crate's own `Cargo.toml` and `Cargo.lock` and nothing else,
  and `main` is fast-forwarded to it.
  It is published,
  locked (`49ca48000e`) and installed here,
  where `.cache/toolchain-bin/cargo-shadow --version` says `0.1.5-alpha.16`,
  and the session's `git` link is back.
  Its Claude Sonnet second pass returned and was read whole
  ("What `cargo-shadow`'s alpha.16 read found"),
  and one round took its findings:
  `cargo-shadow-alpha-16-release-pass-the-asking-again-is-spent-where-the-records-say-and-its-controls-can-fail`,
  which is merged (`05e0758cb7`) and released as `0.1.5-alpha.17`
  ("What `cargo-shadow`'s alpha.16 round did,
  and its alpha.17").
  Both rounds of the read got a note in their inbox at 2026-11-27T15:51:09-05:00,
  saying what issue 6619's merge changed in the package and that a third round works in the route discovery.
  Issue 6619's round is merged (`1ef6287246`).
  Each of the ten tests asks first whether the host can give a delegated scope;
  one failed ask of `systemctl` was read as a host that cannot;
  and a stand-in that refuses the first ask of each process reproduces the ten failures at about ten milliseconds each.
  The question is now put up to three times,
  fifty milliseconds apart,
  and the package's `test` task keeps a failing run's whole output in a file its last lines name.
  The suite passed 950 of 950 on that branch,
  and the package's record is `package/cargo-shadow/cli/doc/the-route-question-that-is-asked-again-when-one-ask-fails.md`.
  What made an ask fail that day is not measured (issue 6623).
  The journal holds this much:
  a system upgrade's hook re-executed the account's manager from 14:07:27.786 to 14:07:28.150;
  the manager logged no unit start from 14:07:27.639 to 14:07:28.533,
  nor from 14:07:28.709 to 14:07:29.935;
  and the red run's first scope starts are at 14:07:28.533 and 14:07:28.664,
  for the two controls that finished just before the ten.
  So the ten asked no earlier than half a second after the reexec had ended,
  and not inside it;
  that is a lower bound,
  and how much later they asked is not on record.
  The coordinator first wrote on issues 6619 and 6623 that the reexec's own 365 ms was the trigger,
  corrected both with a comment once the journal was read whole,
  and then took back one clause of that correction,
  which had placed the asks inside the second span without a unit start.
  Issue 6623's round measures asks across a reexec and after one in a container or a guest,
  never on this account's manager,
  and sizes nothing if every ask there answers.
  The round on the `populated` read is merged (`92a10aa7dc`).
  It measured what two readings of the kernel's source disagreed about,
  on this host's kernel `7.3.5-2-cachyos`:
  of 3,090 reads made while a group was being created,
  none answered "No such file" with the group's directory there,
  against 1,640 such answers from the same harness on a plain directory whose file is written after it;
  and of 300 removals,
  24 put the step's own read at "No such device",
  with the directory still there in 20 of them.
  So a group's directory and its files appear together,
  and "No such device" is a group the kernel is removing:
  that row answers as a collected scope again,
  which is alpha.14's answer and reverses the pick merged as `3bf99cd706`.
  The row for "No such file" with the directory there keeps its warning,
  and is recorded as one this kernel is not seen to produce;
  the same check is live at the `pids.peak` read.
  The control of the bound at a run's end allows one second over the promise where it allowed five.
  The suite passed 951 of 951 on its merged tree,
  and its picks wait in `doc/handover/questions-for-the-owner.md` with the veto open.
  The round on the refusal's words is merged (`7eb08ea6b9`).
  It staged a real refusal and made each kind of later run:
  `cargo check`,
  `cargo metadata`,
  `cargo update`,
  `cargo-shadow install`,
  `cargo-shadow update` and both dry runs settle the kept journal;
  a frozen run,
  a run on the fast path,
  a run that resolves nothing and a run outside the catalog's repository leave it and print nothing.
  The refusal now says each fact once and states that as a rule,
  from one constant both refusals use;
  the recovery's sentences say "a run that did not finish" where they said "killed";
  each dry run's help says it settles a kept journal first;
  and the README's and the troubleshooting file's sentences the read found wrong are corrected against their own figures.
  It changed one sentence in the READMEs of `mpm-shadow`,
  `pnpm-shadow` and `search-shadow`,
  on which suites a changed identity copy fails.
  The two rounds met in one sentence of the troubleshooting file,
  which the coordinator resolved by hand,
  keeping the refusal round's fact and `main`'s pointer;
  the suite passed 964 of 964 on the merged tree.
  The round on the refusal is merged (`87399f8462`).
  It staged a real refusal and ran each kind of next run:
  an ordinary resolving run and `cargo-shadow update` settle the kept journal,
  while a pass-through command and a frozen run leave it and print nothing about it,
  and the refusal's sentence now says so.
  A control shows that a scope whose directory was never made is read as collected,
  with its unit left loaded in silence;
  the fix belongs to the retire gate and is not made
  (`doc/handover/questions-for-the-owner.md`).
  The two rounds met in one sentence of `control_group_stop_report.rs`;
  the session's wrapper merged it by word when `main` was merged into the branch
  (its README,
  "Word-level second merge pass"),
  the result holds both rounds' clauses word for word,
  and the package's suite passed 946 of 946 on it.
  The round on the retire gate is merged (`3bf99cd706`).
  The read's finding held:
  with either edit it named,
  all 15 tests of the gate passed.
  A control now reaches the new decision through `retire` and fails under each edit;
  three warnings say only what the look had read;
  and a `populated` read that answers "No such file" while the group's directory is there is no longer read as the manager's collection,
  which is a changed decision with the owner's veto open
  (`doc/handover/questions-for-the-owner.md`).
  One whole run of the package's suite in that round failed ten tests that need a delegated scope,
  each in about ten milliseconds,
  and the next two runs passed 941 of 941;
  the round's own command had cut the messages off;
  issue 6619's round measured the mechanism,
  and issue 6623 holds what made an ask fail that day.
  `0.1.5-alpha.14` is released
  (release commit `8673ec1309`),
  locked (`c652b74260`) and installed,
  and every round of the `0.1.5-alpha.13` pass is merged.
  The round on the retire's gate is merged (`67103f2285`):
  seven controls,
  each shown red by the edit it guards and reading the real gate through a group directory of its own;
  a group without `pids.peak` is asked about once,
  and the README names the kernel that file came in;
  the warning says only what was read;
  the comment beside the reap grace says what the code polls;
  and the README and the troubleshooting file say the manager can lose the notification an ordinary run relies on.
  The round on the refusal's listing is merged (`cf3210724e`):
  the last leftover a stop can report reaches the refusal's decision in a control that rests on no race;
  a branch no input could reach is gone;
  the listing's order is stated as the code gives it,
  the scope's own group first,
  and pinned across two groups;
  and the refusal for a scope the manager collected says what becomes of the kept journal.
  The proposal round of that list is merged too (`17a5efdf9a`):
  in a guest on systemd 262.2,
  with the account's inotify watches used up,
  a scope started the way `cargo-shadow` starts one stayed `loaded active running` with no member after its program ended,
  and the released `0.1.5-alpha.13` left its four scopes that way and exited 0;
  a re-execution of the manager collects such units and can also lose one
  (two of forty in one of two timed attempts).
  A leftover holds no process;
  twenty of them grew the manager by about 130 KiB each in one reading without a control.
  `doc/planning/cargo-shadow-a-scope-unit-a-killed-shadow-leaves.md` now holds this second road:
  the option that stops the unit a journal recorded cannot reach it,
  the two sweeps can,
  and the ranking stays as it was,
  with its first pair closer;
- a follow-up round for `mpm-shadow`:
  every round of its `0.2.7-alpha.14` pass is merged,
  and `0.2.7-alpha.15` is released
  (release commit `9480230fea`),
  locked (`481a09a3ae`) and installed,
  with the session's `git` link put back at 2026-11-27T13:29:21-05:00.
  Its Claude Sonnet read returned at 2026-11-27T14:35:51-05:00
  (`~/temp/agent/mpm-shadow-alpha-15-release-review-2026-11-27/keep/review.md`;
  "What `mpm-shadow`'s alpha.15 read found"),
  and rounds started from it at 2026-11-27T14:54:37-05:00,
  each named `mpm-shadow-alpha-15-release-pass-` and then its subject.
  All are merged
  (`cda9f0c7b3`,
  `2c6513069b`,
  `7a216deafc`),
  and so is the follow-up round on the report's sentences (`a49bf155cd`).
  `0.2.7-alpha.16` is released
  (release commit `5db66260eb`,
  published 2026-11-27T16:24:48-05:00 by the follower's log),
  locked (`b2afb5b775`) and installed,
  with the session's `git` link put back at 2026-11-27T16:26:38-05:00;
  the install named `rustup` as an unresolved bin,
  which the lock's Rust toolchain row declares and no entry of the exposure folder holds.
  The line is older than this release:
  four installs of 2026-11-27 before it printed the same,
  one of 2026-11-19 named three bins,
  and every install through 2026-11-11 whose kept output shows its last lines printed none.
  Issue 6626 holds that evidence,
  and its round is merged (`7aa3e5e61f`).
  No commit began the line:
  on 2026-11-17 this machine's `rustup` became the system's package,
  and the proxies in the cargo home that had resolved every bin of the row went with the old one.
  The row stops declaring `rustup`,
  which is the manager and no program of the toolchain,
  and nothing in the repository reaches it through the exposure folder.
  The lock followed through `mpm-shadow install --lockfile-only --offline`;
  the branch's lock met the linter's lock row at the merge,
  so the coordinator regenerated it the same way in a worktree,
  and against `main`'s lock it differed in the row's `bins` and the digest alone.
  The install after the merge reported nothing installed,
  nothing failed and no unresolved bin.
  The row's `cargo-fmt` and `rustfmt` resolve here only because the `rustfmt` component was added by hand on 2026-11-25,
  which no component the plan declares installs:
  issue 6630,
  with the pick in `doc/handover/questions-for-the-owner.md`,
  and its round is merged (`b7414d5939`).
  The row declares the `rustfmt` component,
  the only one that installs those two programs,
  as the installed toolchain's own component manifests show.
  A control builds a toolchain from exactly what the row declares and the profile installs,
  and names no unresolved bin;
  it named the two before the change.
  Two more hold that an install asks the manager for a declared component the toolchain lacks,
  and for nothing where the toolchain holds them all.
  The lock followed through the offline rewrite,
  with the row's components and the digest as its whole diff.
  Its whole test task passed 910 of 910 on its merged tree.
  The two rounds of the alpha.16 read got a note in their inbox at 2026-11-27T17:31:52-05:00,
  saying what issue 6626's merge changed and which files that round has.
  Its Claude Sonnet read returned,
  a same-vendor second pass,
  and was read whole
  ("What `mpm-shadow`'s alpha.16 read found").
  Two rounds started from it at 2026-11-27T17:10:06-05:00,
  each named `mpm-shadow-alpha-16-release-pass-` and then:
  `each-sentence-a-person-acts-on-says-what-the-run-read`
  and `a-link-move-that-fails-once-and-what-a-put-back-does-to-current`.
  The round of issue 6626 got a note in its inbox at 2026-11-27T17:10:09-05:00,
  naming the files those two have.
  The first of the two,
  on the sentences,
  is merged (`21c91bc9fd`):
  each of its seven points held on `main` and was shown red there by a test that reads the sentence from a run.
  A failed listing names the hidden names an earlier install left;
  the second-copy clause says the version holds its completion marker and its program at its name,
  where it said nothing was lost;
  the step by hand is a copy of the copy's contents,
  printed as a command,
  where a move nested the copy inside a directory already at the version's name;
  the failed download's clause names a download that matches the lock's digest and that the install can keep;
  and `prune --toolchain` says one of five things it read.
  The README says which sentences its list holds.
  Its whole test task passed 901 of 901 on its merged tree.
  It left the rule behind the second-copy clause alone,
  as its brief said,
  and priced it:
  keeping a second copy whose program differs means reading both programs on every recovery,
  and still cannot tell a program cut short from one rebuilt at the same size.
  The second,
  on the move of `current`,
  is merged (`d46572181c`).
  A test can now make that move fail,
  through a seam compiled out of a shipped build,
  and two controls hold that nothing is removed when the link cannot be moved;
  both fail with the `return` after the failed move taken out.
  A control holds a hidden name whose boot and namespace both differ as ended,
  and fails with the two comparisons swapped.
  The warning after a failed listing no longer speaks of a copy,
  because none was read.
  Two behaviours stay as they are,
  each held by a control and said in the README,
  and each waits for the owner:
  a move of `current` that fails once ends the run 1 and leaves the older version until `prune --toolchain`,
  and a recovery that puts an unfinished copy back points `current` at the newest finished version.
  Its whole test task passed 906 of 906 on its merged tree.
  Every round of the package has merged,
  and `0.2.7-alpha.17` is released
  (<https://github.com/Aquaticats/Monochromatic/actions/runs/46079725275>,
  release commit `04d5150e0b`),
  locked (`50e4b07543`) and installed here:
  the install named no unresolved bin,
  and the session's `git` link is back.
  What backs the claim that an install adds a declared component a toolchain lacks:
  the round's controls,
  whose manager is a stand-in.
  This machine's toolchain already holds `rustfmt`,
  so its install is the arm that asks for nothing.
  No workflow that runs on a push installs the Rust row
  (the release workflows install a toolchain with `rustup` themselves,
  and `music-player-release.yml` alone runs the toolchain install);
  the drift gate in `advisory-core.yml` passed on the merge and after it.
  The real arm passed,
  in the release's Claude Sonnet second pass
  ("What `mpm-shadow`'s alpha.17 read found"):
  with a real `rustup` and the pinned toolchain at the `minimal` profile holding no `rustfmt`,
  `0.2.7-alpha.17` asked `rustup component add rustfmt --toolchain nightly-2026-11-16`,
  the two programs were in the toolchain's `bin` afterwards and ran,
  and the report named no unresolved bin;
  `0.2.7-alpha.16` in the same state asked nothing and named three.
  Two rounds took that read's findings,
  with files of their own.
  `mpm-shadow-alpha-17-release-pass-a-warning-promises-only-what-the-next-run-does-and-each-prune-line-has-a-test`
  is merged (`f4d888d5a1`).
  Each of the five warnings of a recovery whose publish failed now ends at what the run read,
  and where the install that followed put the version at its name,
  it says that a later run publishes nothing from the copy and removes it,
  which the second run of each control then did,
  with no second download.
  The two sentences about a second copy end at the read
  ("because <version> holds its completion marker and its program at its name"),
  since a program cut short under its marker reads as finished too.
  Two `prune` lines and the cleanup after a real failed rename have a test each,
  shown red by the read's own edits;
  the real rename is made to fail by a destination that is a directory.
  The package's task ran 918 and passed 918.
  What it chose beyond its brief waits in `doc/handover/questions-for-the-owner.md`.
  `mpm-shadow-alpha-17-release-pass-the-exposure-folder-links-the-pinned-toolchain-and-only-what-the-plan-declares`
  is merged (`d557ebbe20`) and issue 6634 is closed.
  The two branches met in three files,
  so the second was merged with `main` in a worktree of its own and checked there first:
  format,
  check,
  Clippy and the package's task,
  which ran 927 and passed 927.
  A `rustup-toolchain` row now searches the `bin` of the toolchain its channel names and of no other,
  by the directory's name;
  the way not taken,
  asking `rustup which` for each program,
  costs a process per program and is priced in the package's README and the round's commits.
  A real `rustup` in a container,
  holding the pin and an older toolchain that sorts first,
  put every link of the row into the pin.
  The rebuild removes a symbolic link for a name no row declares and names it in the report,
  and `list` reports a declared component or target the toolchain lacks as drift,
  at the cost of three `rustup` calls where it made one.
  This machine's exposure folder holds exactly the names the plan declares
  (174 entries,
  read 2026-11-27T18:57:36-05:00 with no run against the folder),
  so the rebuild removes nothing here;
  an install of the new release that names a removed link is a surprise to read before going on.
  The second round's choices wait in `doc/handover/questions-for-the-owner.md` too.
  The release run of `0.2.7-alpha.18` was started at 2026-11-27T19:31:24-05:00
  (<https://github.com/Aquaticats/Monochromatic/actions/runs/46088339190>);
  its release commit is `39e60f48cc`,
  which changes the crate's own `Cargo.toml` and `Cargo.lock` and nothing else,
  and `main` is fast-forwarded to it.
  It is published
  (publish run 46088436580;
  the follower's log holds `PUBLISHED monochromatic-mpm-shadow-v0.2.7-alpha.18`),
  locked (`e7bf60a263`) and installed here.
  The install ran twice,
  so that the second run was the new release's own:
  each said `0 installed, 67 unchanged, 0 failed`,
  and named no unresolved bin and no removed link;
  the exposure folder held the same 174 names before and after,
  with no link that does not resolve,
  and `rustc` through it is the pin's (`5f4effd57 2026-11-13`).
  The session's `git` link was put back at 2026-11-27T19:42:39-05:00.
  Its Claude Sonnet second pass,
  a same-vendor read that ran a real `rustup` in a container and tried the removal as an adversary,
  returned and was read whole
  ("What `mpm-shadow`'s alpha.18 read found").
  That round made a removed hidden directory that held a finished copy say which of two things happened:
  a second copy went and nothing was lost,
  or the one copy the version had here went,
  with the reason this run did not publish it.
  The sentences that said "a killed install" say "an earlier install",
  or "an install that has ended" where the system answered that;
  each is read from a run by a test that was red on `main`'s code;
  the README's list was checked against the code's own literals;
  and the suite passed 888 of 888 on its merged tree.
  Its two wording choices wait in `doc/handover/questions-for-the-owner.md`,
  and the package's other files held 76 line-break findings of the linter,
  none on a line that round wrote;
  the linter's own `--fix` took them out (`ee8871b190`),
  as "The Markdown the line-break sweep left out" says.
  The rounds merged with no textual conflict and met in meaning:
  one made the list of removed leftovers carry whether each held a finished copy,
  and reworded the sentence for a publish a recovery finished,
  while controls of the other rounds compared an entry with a path and asserted the earlier words.
  The merged tree did not compile,
  and then two controls failed on the reworded sentences.
  The coordinator changed those test lines on the branch (`af35ea923d`),
  no release code,
  and the suite passed 884 of 884 on the merged tree.
  The merge exposed one sentence the follow-up round corrects before the release:
  after a recovery that failed and that the same run repaired by installing,
  the next run says a finished copy "could not" be published and "is gone",
  while the version is installed and nothing is lost.
  The same round takes the sentences that say "a killed install" after an install that failed without being killed,
  none of which a test read.
  The coordinator read both sides of that merge in the Rust files each side touched
  (`archive_staging.rs`,
  `archive_staging_tests.rs`,
  `toolchain/mod.rs`,
  `sync/what_the_sync_did.rs`;
  `git diff 468c2450a7 b5034231f0` and `git diff 2c6513069b b5034231f0`):
  each side's changes stand in places of their own and nothing of either was lost.
  The README section "What a kill leaves and what the next install does about it" was read whole on the merged tree,
  and the locked linter finds nothing in the package's README on `main`.
  The round on a raced publish is merged (`2c6513069b`):
  a publish that finds a copy at the version's name between its own check and its rename replaces it,
  in the step it uses for a copy that was there all along;
  a name of another boot is ended whatever its start time says,
  which is a changed decision with the owner's veto open
  (`doc/handover/questions-for-the-owner.md`);
  and the suite passed 878 of 878 on its merged tree.
  The round on a hidden name nobody can judge is merged (`7a216deafc`),
  with messages and the README changed and no rule:
  a wait for such a name prints which hidden name holds it and whether that name holds a finished copy;
  `prune --toolchain` says what deleting such a directory costs;
  and an install that cannot download says where a finished copy it cannot judge sits.
  Its proposals and its wording pick wait in `doc/handover/questions-for-the-owner.md`.
  The round on the recovery's order is merged (`cda9f0c7b3`):
  a recovery points `current` at the newest finished version before it removes an older one,
  and removes nothing when the link cannot be moved;
  one control starts the tool between the two steps and another kills the run there,
  and both failed on the earlier order and again with the steps swapped back.
  The sentence after a removal says of each version whether this run recovered it or it was installed before;
  the warning of a recovery that failed ends with what the run then did;
  and the README says when such a run ends 0.
  The warning's wording for a lock that is satisfied already has no test:
  the round could not build that state,
  and worded it to promise nothing about the copy.
  The suite passed 871 of 871 on the merged tree against 866 of 866 on `main`'s code.
  The two rounds that changed one module are joined (`d3f0628c0b`),
  with both rounds' rules kept,
  and the package's suite passed 866 of 866 on the joined tree against 857 of 857 on `main`.
  A hidden name carries the process id,
  its start time,
  its process namespace and the boot's identifier
  (`/proc/sys/kernel/random/boot_id`,
  the target of `/proc/self/ns/pid`);
  a name from another boot is an installer that ended,
  one from another namespace or in an earlier build's shape is never judged;
  the completion marker carries the archive's SHA-256 and a copy is published only when the lock's row matches;
  and `prune --toolchain` removes hidden directories whose owner provably ended and names what it cannot judge.
  Where the two rounds' words could not both hold,
  the joining round chose,
  and both choices wait in `doc/handover/questions-for-the-owner.md` with veto open:
  a finished copy of an older version stays hidden only when the lock vouches for it,
  and one no lock vouches for is removed;
  the copy a kill moved aside waits while a name of this boot whose installer cannot be judged may hold the version,
  and does not wait for a name in an earlier build's shape,
  which nothing ever removes.
  A run that holds no row for an archive removes no finished copy it could not judge.
  A failed or raced publish no longer costs the finished copy (`1add32a8fb`):
  a claimed copy that could not be published stays at the claim for the next install,
  which publishes it with no download;
  the copy a kill moved aside is not put back while a live installer holds a claim or a staging name for that version,
  and a publish that finds a copy put back meanwhile replaces it;
  a recovery never publishes an older version beside a newer installed one;
  a run whose recovery failed still installs what it can;
  and every message on those paths says what is on disk afterwards.
  Its controls stop at a planted step or a real live process,
  never by timing,
  and were shown red by putting the release's decisions back,
  twelve of them at once.
  The record of a start refused because the program is being written says what the code and the trace show
  (`9e13d87bbd`):
  it names no usual cause,
  quotes a test that exists,
  and gives the publish's absence as measured in the installer's own process:
  none where the file system can exchange,
  and 24 to 48 microseconds over twelve publishes on the two-rename fallback,
  where the record had said 5.025 ms from two separate `mv` commands.
  Four start messages name the file and the remedy that fit (`2ff1248256`):
  a compiled program's dynamic linker is named as such,
  the last arm offers no step that cannot show the error,
  the shadow prints the `mpm` candidates it saw with the reason it took or skipped each
  (where the message had named `command -v -a mpm`,
  which bash and zsh reject),
  and an `mpm` without its execute bit is named with its remedy;
  17 scenarios kept their exit status and standard output.

Released again on 2026-11-27:
`cargo-shadow` `0.1.5-alpha.13`
(release commit `c53db2c4d1`,
locked `428fc984a9`,
installed),
`mpm-shadow` `0.2.7-alpha.14`
(release commit `bf8adbb8e3`,
locked `bf626a6017`,
installed)
and the wrapper's `0.6.1-alpha.49`
(release commit `dcb2bcd6a5`,
unpacked at `.cache/wrapper-release-0.6.1-alpha.49/`,
never locked or installed).
`cargo-shadow` alpha.13 holds the retire that asks the manager nothing on an ordinary run (`d0b821414c`)
and the refusal's branches held by controls (`9c4c5aac42`).
`mpm-shadow` alpha.14 holds the record that claims only what was traced (`45cac92526`),
a refused or vanished start that says only what is known (`c388b2b725`),
a refused shadow's own words (`cf1c46518d`)
and a republish that replaces a version in one exchange and repairs what a killed install left (`08ae962d40`).
The wrapper's alpha.49 holds every fix the two alpha.48 passes earned,
issue 6586 and issue 6607.
The install proof of alpha.49 over alpha.48 passed in a disposable container:
no start failed at its `EXEC` step,
no unit reached the start limit,
and the pair of commits made at once each landed once.

What the `cargo-shadow` alpha.13 pass found
(a Claude Sonnet read,
same vendor,
so a second pass;
`~/temp/agent/cargo-shadow-alpha-13-release-review-2026-11-27/keep/review.md`):
no High finding,
and nothing that changes a run's decision or stops a live scope.
In six interleaved rounds a trivial shadow call took 1.085 times alpha.11's time at the median
(0.667 to 2.205),
against 1.097 (0.776 to 2.364) for a same-binary control and 1.982 for alpha.12;
it asked the manager two questions a call where alpha.12 asked ten,
and no stop in 24 scoped runs.
Its three Medium findings:
systemd 262.2 can lose the notification that a scope emptied
(an inotify queue overflow,
a watch that could not be added,
a scope that empties between a reload's one read and the watch being added again),
and then the unit stays loaded with no member and no warning,
read in the source and not reproduced;
on a kernel before 6.1 a scope has no `pids.peak`,
so every scoped run still asks the manager once,
which no control reads and the README does not say;
and seven edits to the gate would stay green,
because the path controls arm a switch that answers before the gate reads anything.
Its Low findings are words and records,
and one window:
a same-user process that writes itself into the scope after the look is killed by the stop in silence.

What the `mpm-shadow` alpha.14 pass found
(the same kind of read;
`~/temp/agent/mpm-shadow-alpha-14-release-review-2026-11-27/keep/review.md`):
no High finding,
five Medium and ten Low.
Right as claimed:
the exchange leaves no absence
(none in 25 rounds on ZFS and on tmpfs,
where alpha.13 showed one in 3 and in 1 of 25,
and 4 of 4 with its first rename slowed);
an unfinished copy and another version's finished copy were never published;
whether a refused start is repeated did not change;
53 command lines are identical between the two binaries.
Its Medium findings are all in the repair a later install makes after a kill,
and each needs a killed install first:
a publish that fails after the claim deletes the finished copy and says both copies are as they were;
two installs recovering one kill can destroy the finished copy with only a delay between them,
no error injected;
a live installer this reader cannot see
(another process namespace,
`hidepid`)
is read as ended,
against the README's own sentence,
and its directory removed or published;
the recovery publishes an older locked version beside a newer installed one;
and no test would catch an edit that publishes the wrong copy.
A plain `install` with the network up repairs each.
This machine ran alpha.14 until `0.2.7-alpha.15` was installed at 2026-11-27T13:29:
`findmnt /proc` showed no `hidepid` here,
and the race needed a killed install's leftovers and two installs at once.
Two merges met in one file each and were resolved by keeping both sides,
with the package's test task run on the merged tree
(`package/mpm-shadow/cli/README.md`,
799 of 799;
`package/cargo-shadow/cli/src/failpoints.rs`,
925 of 925).

Issue 6586 is merged (`7d6bd90c92`).
A build made for tests now reads the session's runtime directory and bus address only from two switches a control sets,
and offers the account's service manager no program,
so a library test run from a tree that holds that merge needs no fixture runtime directory.
A run of a built wrapper executable on the host still does
(a release binary in a pass,
a binary-target test):
`env --unset=DBUS_SESSION_BUS_ADDRESS XDG_RUNTIME_DIR=<a fixture folder>`.

The reads of the wrapper's `0.6.1-alpha.51`,
of the linter's `0.5.2-alpha.23` and of `mpm-shadow`'s `0.2.7-alpha.18` returned and were read whole
("What the wrapper's alpha.51 reads found",
"What the linter's alpha.23 read found",
"What `mpm-shadow`'s alpha.18 read found").
The reads of the linter's `0.5.2-alpha.22`,
of `mpm-shadow`'s `0.2.7-alpha.17` and of `cargo-shadow`'s `0.1.5-alpha.16` returned and were read whole
("What the linter's alpha.22 read found",
"What `mpm-shadow`'s alpha.17 read found",
"What `cargo-shadow`'s alpha.16 read found").
Both Claude Sonnet reads of the wrapper's `0.6.1-alpha.50` returned and were read whole
("What the wrapper's alpha.50 reads found"),
as was the linter's
("What the linter's alpha.21 read found").
Each was a same-vendor second pass and not an independent review.
The coordinator rewrites what lasts from each into this file before scratch expires.

What the wrapper's alpha.50 reads found
(`~/temp/agent/git-policy-cli-alpha-50-release-review-behaviour-2026-11-27/keep/review.md`
for the four merges that change behaviour,
`~/temp/agent/git-policy-cli-alpha-50-release-review-reach-and-guards-2026-11-27/keep/review.md`
for the merges that claim to change none;
same-vendor second passes,
returned 2026-11-27 before 16:00 -05:00):
no High finding in either,
and two Medium and five Low in each.
Neither built the crate or ran its tests,
and neither ran a slot command,
because its scratch was on ZFS:
the slot refusals are argued from code.
What the behaviour read confirmed with both released binaries:
of the spellings of a shell alias it tried against real Git,
every one alpha.50 newly reads is refused where alpha.49 let `reset --hard` run;
over 4,256 merge cases alpha.50 resolves none outside the bounds a union of two deletions keeps,
resolves none that alpha.49 left,
and every conflict it leaves is Git's own bytes;
a process's name that would rewrite the waiting line is shown escaped on one line;
the identity sentence is said once where alpha.49 said it twice;
and of 60 commands run through both binaries,
58 were identical and the other two differ as a merge explains.
Its Medium findings.
A bare `!git` followed by the caller's own `-C <another worktree>` is read,
and judged for the caller's worktree:
from a clean linked worktree it lets `reset --hard` reach a dirty main checkout,
where the direct spelling is refused,
and from the main checkout it refuses a reset of a clean linked worktree;
alpha.49 let the first run too.
By the same construction,
argued,
the slot refusals take their records from the caller's repository.
And a hunk of repeated lines that a side also wrote a line into still resolves to text a side removed
(base `c b b a a a a b`,
ours `c a a a b`,
theirs `b b b a b` gives `b a a a b`);
`SPEC.md` states that exception and the README does not.
Its Low findings:
two refusal sentences nothing calls yet show a name through a weaker escape and stand outside the guard;
when Git cannot be started to ask where an alias runs,
the slot refusal reads no target and says nothing;
the controls of the shell-byte list hold no row for a quote,
a backslash,
a brace or a parenthesis;
the records name fewer unread bytes than the code leaves unread;
and a chain of 64 aliases takes 3.25 times a chain of 16.
Also from its lists:
edits to the merge pass that no test would catch;
the question whether the commit service,
one process for many commands,
ever says the identity sentence,
whose memory is per process;
and the release commit's move of the embedded linter from alpha.19 to alpha.21,
which no merge names and by which no new message reaches the binary.
What the structure read confirmed:
the merges it read change no release code beyond four names narrowed from `pub`,
every use of which lies inside the module each is now visible in;
the published-surface list's entries all name something that exists;
and no commit landed through either binary in its environment,
which has no service manager.
Its Medium findings:
the surface control leaves nameable names without a line,
seven associated constants and three fields of a struct whose header spans lines among them;
and "each guard fails on every spelling of the call it bans" holds for one guard,
while the others match text on one line.
Its Low findings:
an excuse counts text per file,
so a use swapped for another passes,
and two guards excuse without a count;
the guide says every guard reads through the shared scan,
which some do;
more shapes the surface control would miss,
with no instance today;
the list's header and the reader's doc are off in details;
and the guide still describes the surface control's earlier reader.
The coordinator's weighing:
five rounds take all of it but two points.
The cost of a long alias chain is dismissed,
because alpha.50 over alpha.49 on that input is 1.15 with a range of 0.90 to 1.18,
inside the spread of one binary against its own copy.
A raise of the bound constants that no control catches is dismissed,
because the controls derive from the named constant by the repository's rule on numbers,
and `SPEC.md` and the README state the value.
The first half of one Low,
that nothing pins the slot side's directory function below the top,
was answered by issue 6621's controls after the read's revision.
The merge finding is taken as a fix,
by the owner's request of 2026-11-10 that wrong text land less often,
and its round returns the pick and the other way for the owner.

What `mpm-shadow`'s alpha.16 read found
(`~/temp/agent/mpm-shadow-alpha-16-release-review-2026-11-27/keep/review.md`,
a same-vendor second pass):
no High,
no Medium and ten Low.
Shown with both released binaries on fixtures with real kills:
the order of a recovery's last two steps,
where a start in the window exits 127 on alpha.15 and runs the newer version on alpha.16;
the raced publish,
where alpha.15 ends 1 with both hidden copies kept and alpha.16 ends 0 with the other install's copy as the version;
every message of the round on a hidden name nobody can judge;
and all twelve text blocks of the README's section,
byte for byte.
On input with nothing left behind,
38 commands compared between the two binaries differ in the version string alone.
Each point of the alpha.15 read is closed,
or open on purpose with its entry in `doc/handover/questions-for-the-owner.md`.
The ten Low points,
each weighed by the coordinating session and each taken by one of the two rounds:
one sentence still says "a killed install",
for a directory listing that failed;
the README lacks five sentences the code prints;
"nothing was lost with that copy" is printed while the program at the version's name is cut short under its marker;
the advice to move a finished copy by hand nests it when the version's name already holds a directory;
the failed-download sentence promises a download where the failure was a digest mismatch;
`prune --toolchain` says "a finished copy" from the marker alone for a tool no lock names,
and says a deletion loses a copy the version does not need;
a move of `current` that fails once ends the run 1 although a later step of the same run moves it,
and the older version then stays until `prune`;
a warning after a failed listing speaks of a copy that does not exist;
and one decision no claim named:
a recovery that puts an unfinished copy back now also points `current` at the newest finished version,
which the round holds with a control and writes into the README,
and which waits for the owner in `doc/handover/questions-for-the-owner.md`.
The read also named edits every test would still pass,
which the rounds turn into controls:
the `return` after a failed move of `current` deleted,
the namespace compared before the boot,
a second copy decided by whether the directory is there,
and each filter of the failed-download sentence dropped.

What `mpm-shadow`'s alpha.15 read found
(`~/temp/agent/mpm-shadow-alpha-15-release-review-2026-11-27/keep/review.md`,
a same-vendor second pass that a second agent finished from the first one's transcripts,
rerunning three of them):
no High,
three Medium and nine Low.
Shown with both released binaries on fixtures with real kills:
a claimed finished copy that could not be published stays for the next install,
which publishes it with no download;
the copy a kill moved aside is not put back beside a live claim or a live staging name;
a recovery never publishes an older version beside a newer one in either toolchain root;
and each of the launcher's five start messages names the file that is missing or busy.
With no killed install behind a run,
nothing differs between the two releases but the version and the marker's text.
Its Medium findings,
all in what alpha.15 added:

- a recovery removes the older version before it moves `current`,
  so a start in between exits 127,
  and a kill there leaves `current` naming a version that is gone until the next `install`;
  alpha.14 removed nothing after a recovery;
- alpha.15 never judges a name in the shape alpha.14 wrote:
  after a kill under alpha.14 an offline `install` leaves the version's name empty,
  says only "download failed",
  and no command ever removes the finished copy that sits hidden;
- the copy a kill moved aside waits for ever for an installer that died in another process namespace
  (shown with a hand-made name),
  the sentence printed claims a live install and names no path,
  and the deletion `prune --toolchain` advises loses the only finished copy.

Its Low findings are sentences that say more than the run knew,
a publish raced between its two renames that fails where the file system has no exchange
(alpha.14 did the same),
what "vouched" means to `prune` against what it means to the recovery,
one checkout's install removing a finished copy another checkout's lock vouches for,
names this release cannot judge when it cannot read the boot identifier,
and four edits no control catches.
It found two changes no claim listed:
a warning when the exchange call is refused,
which the README documents,
and an `install` that ends 0 after a failed recovery where alpha.14 ended 1.
The rules on which names are judged,
held or removed stay as the joining round picked them,
with the owner's veto open and the read's costs written beside them
(`doc/handover/questions-for-the-owner.md`);
the three rounds change the order of one step,
the sentences,
the README and the controls.

What `cargo-shadow`'s alpha.16 read found
(`~/temp/agent/cargo-shadow-alpha-16-release-review-2026-11-27/keep/review.md`,
a same-vendor second pass,
its end state taken 2026-11-27T19:12:03-05:00):
no High and no Medium finding,
and four Low.
What it confirmed with both released binaries:
across real reexecs of a container's own manager,
13 of 13 calls per arm that began inside a reexec took the weaker path under `0.1.5-alpha.15`
and the delegated route under `0.1.5-alpha.16`;
the kinds of later run after a kept journal,
and the fast path,
behave the same on both releases;
and where the manager answers at once the two differ in 12 of 240 compared files,
the version lines and the help of `install` and `update`,
which now prints the long layout for every option,
a change no record names.
Its findings.
The five asks,
about 0.9 s against about 0.04 s,
are spent in 11 of 24 states of a host with no manager,
where the README says the span is spent where a manager answers "and nowhere else";
plain files at the socket names and a live bus with no manager are named by no record.
The read of the host counts the bus socket and the bus address,
transports that `systemctl --user` of systemd 262.2 does not take when `XDG_RUNTIME_DIR` is set,
where the code's comments say the reverse;
a guest on systemd 255 was answered over the bus,
so the versions may differ.
A manager that is reachable and refuses at once,
shown with `app.slice` masked,
is asked five times (941 ms against 68 ms),
and so is an answer that is not a path,
which no control and no record names.
And three edits leave every control green:
the new `ENODEV` row skipping the check that the absence is verified,
one ask fewer or a shorter pause,
and the removal of the check for the manager's directory.
The read broke its brief in two places and said so:
four `--version` calls ran outside a scope,
and it looked at one running batch more than twice.
Its containers are gone and it left no scope.
The coordinator's weighing:
all four are taken by the one round the release's passage names,
since they meet in the same files.
Its picks,
the coordinator's own with the veto open:
the read counts the transports the ask can take,
shown from systemd's source at both versions;
an ask the manager answered with a refusal ends the asking,
if a real reexec shows no such answer inside its window;
the records name every state in which the span is spent;
and each of the three edits gets a control shown red by it.

What `cargo-shadow`'s alpha.15 read found
(`~/temp/agent/cargo-shadow-alpha-15-release-review-2026-11-27/keep/review.md`,
a same-vendor second pass):
no High,
one Medium and ten Low.
Between the two released binaries nothing differs but the version and the sentences the merges name
(226 of 232 compared outputs identical,
the rest version lines;
alpha.12 as the case that must differ did,
in 21),
and a trivial call asks the manager two questions in alpha.11,
alpha.14 and alpha.15 against six to ten in alpha.12.
Timing cannot see a small change on this host:
a byte-identical copy's own ratio ranged from 0.414 to 1.500.
Its Medium finding is about the decision merge `3bf99cd706` changed:
by its reading of the kernel at three versions,
a new control group's directory and its files become visible together,
so the "No such file with the directory there" the round guarded against is a state nothing produces,
and the answer the kernel can give there is "No such device",
for which alpha.14's silence was right and alpha.15 warns about a unit the manager is freeing.
The round's reason was a reading too,
and nothing was observed by either,
so a round measures it on this host.
Its Low findings:
the printed refusal says two things twice;
more kinds of later run settle a kept journal than the sentence names
(a plain install,
`cargo metadata`,
both dry runs),
and more leave it in silence
(a frozen install,
a run on the fast path);
a dry run settles a kept journal before it looks,
and can write;
a scope that was never made is reachable in a release build and silent,
and a unit left loaded was not reproduced in 56 launcher kills;
three controls stay green under an edit each;
and sentences of the README,
the troubleshooting file and three sibling READMEs say what the code or the figures do not.

What `cargo-shadow`'s alpha.14 read found
(`~/temp/agent/cargo-shadow-alpha-14-release-review-2026-11-27/keep/review.md`):
no High finding,
two Medium and ten Low,
and no decision or output that differs from alpha.13 beyond what the merges name.
The manager is asked two questions a call in alpha.11,
alpha.13 and alpha.14 alike,
over 260 timed calls;
the cost of a call against alpha.11 had a median ratio of 1.021 for alpha.14
(0.409 to 2.072 over 20 rounds),
with a byte-identical copy of alpha.11 as the control at 0.907
(0.457 to 2.019),
so the harness tells alpha.14 from neither alpha.13 nor alpha.11.
Its Medium findings,
each with a round:
the refusal says the next run settles the kept journal by itself,
while a pass-through run such as `cargo fmt --check` and a frozen run under `CI=true` leave it and print nothing
(shown with a journal a killed run left,
not with a real refusal);
and the release's one new decision,
a `pids.peak` read that fails because the group's directory went away,
is held only at its helper,
so putting alpha.13's behaviour back would pass every test
(argued from the test files;
the round applies the edit once to settle it).
Its Low findings are sentences that say more than was read
(two warnings of the retire gate,
four sentences of the README,
one of which gives a ratio outside its own control's range),
two controls that stay green under named edits,
and a state outside these merges:
a scope that does not exist yet may be read as collected and its unit left loaded in silence.

What `mpm-shadow`'s alpha.17 read found
(`~/temp/agent/mpm-shadow-alpha-17-release-review-2026-11-27/keep/review.md`,
a same-vendor second pass,
written 2026-11-27T18:45:20-05:00):
no High finding,
one Medium and six Low.
What it confirmed with the released binaries:
the real `rustup` arm of issue 6630,
as the release's passage says;
that 38 commands on an input with nothing left behind differ in the version string alone;
that a move of `current` that fails once and a put-back behave in alpha.17 as in alpha.16;
and that the README's blocks for a kill match what alpha.17 prints.
Its Medium finding,
shown with a real `rustup` and two real toolchains:
the exposure folder links each program of the Rust row from the first toolchain in name order that holds it,
and never looks at the row's own channel,
so an older toolchain that sorts before the pin owns the folder's `rustc`,
`cargo` and `rustfmt` while `list` says `ok`.
The code is older than alpha.16.
On this machine,
measured between 2026-11-27T18:45:20-05:00 and 2026-11-27T18:46:47-05:00
(the review's own time and the clock of the command after the measurement),
the toolchains are the pin and an unpinned `nightly`,
the pin sorts first,
and every Rust link of the folder points into the pin;
the fault bites when the pin moves to a later date while the older toolchain stays,
and the daily update moves the pin.
It is issue 6634.
Its Low findings:
the warning after a publish that failed once says running the command again publishes the copy,
where the next run removes it,
and alpha.16 said so rightly;
a version whose program is cut short under its marker is still called the holder of a second copy,
by the install and by `prune`;
the folder keeps a link an earlier row made for a name no row declares;
two README sentences about `rustup` are not so where rustup's proxies stand in the cargo bin directory;
`list` says a toolchain is in sync while a declared component is missing;
and two of the five `prune` lines,
and the cleanup after a real failed rename,
are asserted by no test.
The read broke its brief in one place and said so:
one driver ran an install that tried to resolve a fixture tool through `api.github.com` from the host,
from a throwaway home with no credential,
and nothing was fetched.
The coordinator's weighing:
all seven are taken,
by the two rounds the release's passage names;
their picks are the coordinator's with the veto open,
and are written into `doc/handover/questions-for-the-owner.md` when the rounds' reports are read.

What the linter's alpha.22 read found
(`~/temp/agent/monochromatic-lint-alpha-22-release-review-2026-11-27/keep/review.md`,
a same-vendor second pass,
written 2026-11-27T18:12:31-05:00):
no High finding,
no Medium and nine Low.
What it confirmed with the released binaries:
over 10,081 lists no line moved between read and refused between alpha.21 and alpha.22,
and no exit status or set of findings changed,
with a probe first that shows the comparison sees a move (alpha.19 against alpha.21 moves 163 lists);
on a clean tree only `--version` differs;
the table of unseen characters equals Unicode 17.0.0's own data,
and over every scalar value alpha.22 spells 4,288 where alpha.21 spelled 29;
a `file-uses` line of six words names the folder it wrote in 1,152 of 1,296 lines against 324;
and the help of a dead line names every character a gap kept,
in 992 of 992 lines.
Its Low findings:
a folder named with an arrow that a tab,
a no-break space or two spaces set off is still told its line holds two arrows;
a folder whose own name begins with whitespace is named for single gaps only,
72 of 200 lines,
and with a sibling the sibling is named;
a line written for a folder the tree does not hold names the folder its words make;
the name is judged against any directory on disk,
so an absolute path in a six-word line is told to be renamed for a space;
a dead `file-uses` line is told to take a character out whenever the line holds one,
also where the file is really named so,
which alpha.21 answered rightly;
the loop and missing-link messages,
the commands they give and the help of a dead `uses` line still print such characters raw;
three refusals quote no line and one names no character;
the README says the place is "the column an editor shows",
where `vim` shows a byte column and a tab's width,
and a sentence says a line "reads as a `uses` line" that is refused after the edit;
and three controls stay green under a named edit,
the ones on the range table among them,
because they read the table they hold.
Of the alpha.21 read's points,
its two Medium are closed for the shapes it named,
and the rename for each arrow shape and the wall time of a long dead list are as they were,
the second inside its control's spread.
The read's `vim` runs,
headless,
rewrote `~/.viminfo` in the account's home at 2026-11-27T18:09:40-05:00;
the brief of the round that follows forbids running an editor.
The coordinator's weighing:
one round takes the dead `file-uses` advice,
the refusals that quote no line,
the README's and the sentences' overstatements,
the controls,
the two shapes of the readings,
and the absolute path in a six-word line;
a line for a folder the tree does not hold keeps its reading,
and the README says so;
and the raw characters in the other messages wait for the owner,
because a command a person copies must hold the name raw
(`doc/handover/questions-for-the-owner.md`).

What the linter's alpha.21 read found
(`~/temp/agent/monochromatic-lint-alpha-21-release-review-2026-11-27/keep/review.md`,
a same-vendor second pass,
returned 2026-11-27T15:31:26-05:00):
no High finding,
two Medium and eight Low.
What it confirmed with the released binaries:
over 5,161 lists no line changed from read to refused or back between alpha.20 and alpha.21,
so only texts changed;
the repository copy's findings are byte-identical in alpha.19,
alpha.20 and alpha.21;
and no cost shows on a package-sized run.
Its Medium findings:
a six-word `file-uses` line names its folder only when the gap after the word,
the gap before the file and the gap before the arrow are each one character.
Of 1,296 such lines over real folders named with a space,
a tab,
two spaces,
a no-break space and an arrow,
558 lost the folder's name that alpha.20 gave them:
with only the gap before the arrow doubled the line is told it holds six words where five are wanted,
a folder named with an arrow is told the line holds two arrows,
and with another gap doubled a folder that does not exist is named;
alpha.21 did put right the lines for which alpha.20 named a wrong folder.
And a five-word line whose arrow is misspelt or misplaced is told that five words are wanted and that it holds 5.
Its Low findings:
alpha.21 spells six unseen code points and prints the rest raw,
4,232 by the read's count,
the bidirectional marks,
embeddings and overrides among them;
the spelling and the take-it-out advice reach dead key lines only,
so a refusal,
which stops the whole rule,
and the sentence for a repeated entry still print the character raw;
the byte order mark's sentence names `uses` on a `file-uses` line,
counts the character's place from the trimmed entry,
and tests the word by prefix;
the help of a dead line names one surplus character,
so a longer gap needs the help followed again;
a folder name that holds two arrow shapes needs a rename for each;
a list of 30,000 dead lines takes 1.18 times alpha.20's wall time
(range 1.10 to 1.34,
its control 0.96 to 1.14),
and 1.06 where both print sentences of equal size;
README sentences say more than the releases did;
and tests of the merge stay green under named edits,
with nothing holding the lines of the first Medium finding.
Changes no claim accounted for:
every dead line's quote now spells a tab and a no-break space as well as the format characters,
and the refusals for a name no list line could hold spell format characters too.
Not reviewed:
the closure check,
the rules outside the layout rules beyond a byte comparison,
the test suite as run,
Windows and macOS,
and how `package/git-policy/cli` uses the library,
whose pin the release commit moved to alpha.21.
The coordinator's weighing:
one round takes the Medium findings,
the unseen characters,
the dead line's help,
the README sentences and the tests;
the rename for each arrow shape is dismissed,
because each sentence such a name gets is true and such names are rare;
and the wall time of a long dead list gets no change,
because the longer sentences are the cause the read could show.
The coordinator's brief for that read named a README correction the merge never made
(the figures of the walk's shape),
which the read reported as absent.

What the linter's alpha.20 read found
(`~/temp/agent/monochromatic-lint-alpha-20-release-review-2026-11-27/keep/review.md`):
no High finding,
two Medium and eight Low.
What it confirmed with both binaries:
of 220 spellings of a `uses` line alpha.20 reads 31 and reports 189 dead,
with none refused,
where alpha.19 refused 163;
over 4,092 lists,
none that alpha.19 read clean is unclean in alpha.20;
the keys it prints read back with no dead line;
the two allow lists of `package/git-policy/cli` give identical output in alpha.16,
alpha.19 and alpha.20;
and CPU time against alpha.19 was 0.89 to 1.08 over five interleaved rounds,
with a byte-identical copy of alpha.19 as the control at 0.90 to 1.11,
so no cost was measurable.
One real change of behaviour beyond the release's notes:
a tree whose folder name sets `->` off with a tab or a no-break space is refused with exit 2,
where alpha.19 read it clean.
Its Medium findings:
a `file-uses` line for a folder whose name holds a tab,
a no-break space or two spaces gets only the shape sentence,
where alpha.19 named the folder;
and five behaviours of the new code are held by no test,
argued from the test files and not by running the edits.
Its Low findings are messages that point the wrong way
(a byte order mark gets the advice to delete a line that records a real loop),
three README sentences,
and arms no input reaches.
The closure step did not run on this host for either binary,
because its Dylint driver is absent and fetching it needs the network.
One round holds all of it.


What the wrapper's alpha.49 behaviour pass found
(`~/temp/agent/git-policy-cli-alpha-49-release-review-behaviour-2026-11-27/keep/review.md`):
no High finding,
four Medium and three Low,
none a regression of alpha.49.
Demonstrated with both released binaries:
with a reader of standard error gone,
alpha.48 removed a notice's record and alpha.49 kept it;
after a failed start of the real Git with standard error a closed pipe,
alpha.48 was ended by `SIGPIPE` and alpha.49 exited 2;
over 56 merge cases alpha.48 resolved 6 conflicts and alpha.49 resolved 14,
each extra one matching an oracle;
and 44 of 45 commands compared were identical,
the other being the intended change.
Its Medium findings,
each with a round:
a slot removal or move reached through a shell alias reads its operand from the directory the wrapper was started in,
where Git runs the alias at the top of the working tree after `-C`,
so `./wt/two` from a subdirectory or `../one` with `-C` passes the refusal and removes a bound slot's registration
(argued from the code and from Git's own behaviour;
the refusal itself was not run);
a process's name is escaped at two of the seven sentences that print it;
five alias spellings made only of readable forms stay unread
(a shell alias naming a shell alias,
`! git`,
a tab after `!git`,
a bare `!git` before the caller's own global option,
a chain past 16),
while `SPEC.md` says every spelling is read;
and the merge pass,
when both sides only delete and the base repeats a line,
can write back a line one side removed
(base `b a a`,
ours `a`,
theirs `b` resolves to `a`),
the same in alpha.48,
in 11 of 199 and 15 of 521 answers of a seeded search over unusually repetitive files.
That last one lands wrong text with no marker,
which is what the merge pass's own rule exists to stop.

What the wrapper's alpha.49 structure pass found
(`~/temp/agent/git-policy-cli-alpha-49-release-review-reach-and-gates-2026-11-27/keep/review.md`),
on the two merges that claim to change no behaviour (`02d0a6e8e6` and `97d9afb21c`):
no High finding,
two Medium and four Low,
and no change of behaviour on Linux from either merge.
A Linux release build differs through `02d0a6e8e6` in two ways only,
argued from the source text and not from a build:
one unused constant leaves it,
and one tag constant goes from 141 copies to 123 with every value the same.
Every file `97d9afb21c` changed compiles only in a test build.
Of 61 commands run through alpha.48,
alpha.49 and alpha.49 again as the control,
57 were identical and the control differed in none;
the four that differed are the merge pass's intended change.
Its Medium findings,
each with a round:
the control that fails when a name becomes nameable from outside the crate cannot see a function with `async` before `fn`
(154 of 155 such published names have no line in its list),
a name another listed item already has,
a field,
a method,
or a `pub use` written over several lines;
and the guards that now read the folders the splits made gained scope and lost two things:
the hook ban no longer fails on `libc::symlink(` or on an imported bare `symlink(`,
and an excuse covers every use of a text in its file.
Both rest on Python ports of the Rust controls' text,
not on runs of the controls,
so each round first plants the shapes against the Rust controls.
Its Low findings:
the merge restored seven platform gates
(seven such attributes in its diff outside tests,
counted again by the coordinator),
where the commit says 29 items and this file said 28;
two merges outside both passes' lists change release code
(`1cda665610` and `0dda4eccb7`),
and the behaviour pass read both as identity changes;
nothing shows that the crate builds on a target other than Linux,
and the README does not say which targets build;
and the package guide records none of the rules the two merges brought.

Beyond their findings,
the two reads name changes that no merge's description carried.
`cd62517318` made the refusal of a removal behind an unknown global option need a slot record in the record folder,
where alpha.48 refused on the folder alone:
with an empty folder alpha.48 exits 1,
and alpha.49 passes the command to Git,
which exits 129.
That is the decision's own wording
(`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`):

```text
In a repository with no slot records the command is forwarded as before.
```

The control `the_home_says_whether_it_holds_a_record` holds it for an empty folder,
an unrelated file,
a record and a record being written;
the read says a folder that holds only an add's claim also counts as holding no record,
and no control shows that case.
`7d6bd90c92` changes a real run in three edge cases:
a runtime directory whose name is not UTF-8 is used where it was ignored,
an empty bus address is passed on and treated as before,
and an empty `XDG_RUNTIME_DIR` on the ZFS add route is refused as not set.
The release binary's code grew by about 112 KiB between the two releases,
and neither read says which merge grew it.

The coordinator corrected four of the six briefs by a note to each round at 2026-11-27T13:11:19-05:00,
because each stated the coordinator's own design as the requirement:
the shell-alias round adds no refusal for an alias the wrapper cannot follow,
which is the open choice of `doc/planning/cli-git-slot-refusals-and-shell-aliases-they-cannot-read.md`;
the four-points round writes the control for a failed memory reading and leaves what the code does with one;
the surface round lists the items its control newly sees and narrows only one a split widened;
and the list-guards round is held to the outcome,
a use added to an excused file fails,
and not to the coordinator's shape for it.
Each report is read against its brief and its note.
After each of the two structure branches merges,
`native:check` runs on the pinned tree before the next merge:
a narrowed item or a changed excuse can break a build through a merge with no textual conflict.

The whole gate on `main` at `97d9afb21c` passed
(unit `gpc-gate-wrapper-next-alpha-on-97d9afb21c-20261127T113315-0500`,
started 2026-11-27T11:33:15-05:00,
`gate-exit-status=0`):
`Summary [ 919.977s] 12191 tests run: 12191 passed (7 slow), 51 skipped`
and "Clippy passed beside the test run.".
Its worktree is `.claude/worktrees/gate-wrapper-next-alpha`,
which `native:gate:main-tip` expects to exist and does not make.
The three feature Clippy arms exited 0 in that worktree at the same commit.
Between that commit and the release,
only test code,
the package guide and TypeScript changed in the wrapper package
(issue 6595,
part 2 of issue 6581),
and on `main`'s tip `7e215a38c2`,
in a second worktree no merge moves
(`.claude/worktrees/typescript-checks-on-main-tip-before-wrapper-alpha-49`),
`native:check` and the four Clippy tasks exited 0
and the 232 tests of the modules issue 6595 touched passed;
`native:ts-checks` exited 0 there at `7976b3daff`.
Tip checks run in such a worktree:
on 2026-11-27 three Clippy arms run in the main checkout exited 0 while Rust merges landed under them,
and stood for no commit.

Round 3 of the crate split's order is done but for three lines.
Every folder's file loops are broken and merged,
the last being the removal files of `worktree/` (`ecb3486d9f`);
`doc/planning/cli-git-source-layout-target-tree.md` lists each folder's merge in its notes of 2026-11-27 under "Next",
and that round's note there says where the plan's reading of the moves did not fit.
`layout-allow-list/file-rule.txt` went from 544 lines to 25,
with no layout finding on `main`.
Its three `file-uses` lines are `foundation/`'s loop of `diagnostic_lines_off_the_calling_thread.rs`,
`diagnostic_log.rs` and `lines_written_by_a_thread.rs`,
which no move of items closes and which waits for the owner
(`doc/handover/questions-for-the-owner.md`).
Merges that meet in that list are resolved by hand as the union of the two sides' deletions,
with `cargo check --all-targets` and the linter run on the merged tree before the commit:
the installed session wrapper's merge pass keeps deleted lines there
(issue 6529;
the source is fixed in `5914dd3810` and the installed build is older).

Round 4,
the plan's five call changes,
is not started:
each is a changed signature in release code,
and no record shows the owner accepting them
(`doc/handover/questions-for-the-owner.md`,
"does the crate split's step 4 run now?").

Waiting on a named merge:
the two `truepeak` crates' one file each,
on the branch `refactor/flac-profiler-keeps-the-rust-one`.

`main`'s tip is checked after a batch of merges and not by each merge:
the merge routine runs `git merge-tree` and compiles nothing.
At `7a830014b3` (2026-11-27T07:34:30-05:00),
with the tip read before and after each run and found unchanged,
`native:check` exited 0,
the four Clippy tasks exited 0
(`native:clippy`,
`native:clippy:test-only-switches`,
`native:clippy:measurement`
and `native:clippy:start-probe-raised-deadline`),
and the `source_guard` module passed 179 of 179.
The gate that passed ran the two wrapper merges after it
(issue 6562's and the split of `worktree/worktree_remove.rs`).
That check never ran the package's TypeScript side,
and it does from now on,
until the owner chooses where those checks run (issue 6584):
`mise run //package/git-policy/cli:native:ts-checks` runs them each in a capped scope,
and `lint:oxlint` is run beside it.
At `2218beae78` `test:unit` exited 0 with 138 suites passing and `lint:types` exited 0;
`lint:oxlint` reported 594 errors and 1212 warnings,
all in the guest scripts issue 6581's rounds hold;
and `package/container-limit/core`'s `test:unit` failed its ledger test (issue 6588).
A tip check that a merge lands under is void and is run again.
A threaded `worktree::` run on this host fails 155 controls unless `CLI_GIT_BTRFS_ABSENT_ACKNOWLEDGED=1` is set,
every one with the refusal of `subvolume/fs_btrfs_test_support.rs` on a home that is on ZFS;
with it set,
1015 pass and none fails,
on `main` and on the last loop-break branch alike.

The whole container gate on `main` at `d9ec2443bd` ended at 2026-11-27T02:37:12-05:00 and refused:
its suite passed 12089 of 12089,
and its Clippy container was killed at the 2 GiB bound the two containers shared.
Issue 6562's round measured Clippy's container in six runs (4,574.4 to 5,278.7 MiB with a bound that did not bind)
and gave it a bound of its own,
8 GiB,
with the readings beside the value in `bin/native-gate-container-limits.ts`
and in `doc/planning/git-policy-cli-cachyos-test-containers.md` (`79b93df956`).
Two of its five gates passed both checks,
the last on its branch merged with `main` at `88a63ef565`:
`12115 tests run: 12115 passed (5 slow), 51 skipped` and "Clippy passed beside the test run."
The three that refused each named its test run alone,
under a host load average of 48 to 96:
one control held 31.9 s by a `gpg` in an uninterruptible wait,
one that read "the system refused a thread",
one slow transfer ended at its two-second window,
the notice control another test's spans disturb
(`doc/troubleshooting/git-policy-cli-commit-index-span-control-counts-another-tests-spans.md`),
and the stale source-reading control that issue 6573 has since fixed.
The suite's container read 1,149.3 to 1,516.5 MiB of its 2 GiB in those runs,
which issue 6583 holds.

A package-launched round has ended when `pi-runs/<round>-<n>/summary.json` exists,
which the package's `list-runs-that-ended-since` lists,
and it merges by its branch,
once `origin/<branch>` equals the local tip.
A run whose latest run folder holds no `summary.json` and has no process never started:
read the launch unit's `.out` file and the unit's journal.
The driver refuses a run folder that already holds anything,
so a steering note goes into `<run folder>/inbox/` only after the run's `feed.jsonl` exists:
a note written between the launcher making the folder and the driver starting
stopped issue 6566's first start on 2026-11-27.

#### What the wrapper's alpha.51 reads found

Both are Claude Sonnet's,
same-vendor second passes and not independent reviews
(`~/temp/agent/git-policy-cli-alpha-51-release-review-behaviour-2026-11-27/keep/review.md`
and `~/temp/agent/git-policy-cli-alpha-51-release-review-no-change-merges-2026-11-27/keep/review.md`,
scratch).
The behaviour read ran alpha.50,
alpha.51 and a byte-identical copy of alpha.51 beside real Git 2.56.1,
and found two High and six Low points;
the no-change read found seven Low points.
No session on this machine runs alpha.51:
the session wrapper is the build of `main` at `ee0cf2268b` of 2026-11-22 and the lock row is alpha.44,
both older than the merge that brought the two High points.

- High,
  both from the merge for issue 6624 (`461d441ef7`),
  filed as issue 6641 with the reproductions.
  With `alias.g = !git`,
  options typed before the alias name are dropped once the words after it name a place:
  `git --git-dir=<main>/.git --work-tree=<main> g -C <wt1> reset --hard`,
  typed in another linked worktree,
  is refused by alpha.50 and run by alpha.51,
  and the main checkout loses its unstaged edit and the staged new file is gone from disk.
  And from a linked worktree Git hands the alias's shell the caller's `GIT_DIR`,
  so a nested `-C` moves the files and not the repository:
  `git g -C <wt1> reset --hard` typed in a dirty worktree overwrites an untracked file of `wt1` with the caller's committed text,
  where alpha.50 refused because it judged the dirty caller.
  Of 110 rows,
  22 moved from run to refused as the merge meant,
  7 false refusals went,
  and 12 moved from refused to run with a change of state.
  The merge's one behaviour control compares verdicts through `plan_invocation`,
  which never starts Git.
- The merge pass's copies bound (`85be73acf1`) does what it says:
  in 1,672 merges four moved from resolved to conflict,
  none the other way,
  and no resolved byte differs.
- The `final-newline` hint (`0f69dbfe93`) was shown at an add,
  a direct check and a manual push.
  At an add refused at `error` it promises a commit that cannot take the file,
  and at a push it still says to stage a file whose commit is made
  (Low 3 and Low 4).
- The two sentences of `e772d8ad66` stand in a module nothing in the release calls;
  neither binary holds their text.
- The no-change read:
  of the 6,789 Rust lines `beb61ec46b` changed every one is a doc comment,
  the code is byte-identical in 471 of 471 files,
  and every changed Markdown file renders to the same HTML;
  the ten names `35fdc1797c` narrowed are used nowhere outside their new reach;
  no changed linter code is in the built wrapper;
  and the released binaries differ in nine of 53 compared commands,
  each by the version or by the `final-newline` hint.
- Weighed by the coordinator.
  Five rounds take what earned one ("Rounds running").
  Dismissed:
  the behaviour read's Low 2,
  the alias spellings `SPEC.md` names as the guard's boundary,
  documented and unchanged;
  and the no-change read's Low 2,
  that the internal-error notice prints line numbers every source edit moves.
  Its Low 7,
  the lone words the linter's fix left in doc comments,
  is the linter's and is issue 6642.
- The first of the five rounds is merged (`570402952b`):
  the three lint findings in `check_outcome.rs` are gone,
  cleared by what each rule asks for and by no suppression,
  and the package's findings on `main` are the thirteen files over the line budget and nothing else.
  The crate's one doc test compiled and passed when run alone;
  its assertion is a unit test the gate runs now,
  shown red by a plant,
  and the guide says a doc fence holds TypeScript pseudocode because no task runs doc tests.
  The round found that `src/native/policy/policy_forbidden_strings.rs` opens a doc fence at line 23 that no line closes,
  which no rule of the linter reports;
  it is noted on issue 6642 for the round that puts the wrapper's doc comments right.
- The `final-newline` round is merged (`dce0e58a56`).
  At an add,
  at `warn` and at `error` alike,
  the hint leads with `fix -- <path>` and then `git add -- <path>`,
  and says a commit corrects the file only when it takes the whole file;
  a landed commit and a manual push say to edit the file and commit the correction;
  the direct check's hint is unchanged.
  Its controls were shown red by two plants,
  and the read-only one derives its answer from `decide_selection`.
  Its open points wait in `doc/handover/questions-for-the-owner.md`.
- The round on the two failures and three records is merged (`efae1c2767`),
  after its branch passed format,
  check,
  clippy and its modules on the tree that held the `final-newline` merge,
  and check and the `source_guard` module again on the tree that held the surface control's.
  It changes no release code.
  Two controls hold the two failures the policy stage's lines lead,
  each handing `forward_action` the text a real stage rendered,
  because no state reaches either branch through `plan_invocation` with a finding printed;
  a third reaches the margin of the copies bound,
  red when the bound reads the smaller count.
  The README states the copies bound for one region and not for the file,
  and `SPEC.md` says a plain `git worktree prune` behind an unplaced alias is not refused.
- The round on the surface control and three guards is merged (`ee6dc2cd2f`),
  and changes no release code.
  The reader of module lines refuses a `mod` statement at a file's first column that it cannot place,
  where it used to step over one;
  the surface reader reads the public members of a type a `pub use` carries,
  through the root's globs,
  so the list of published names gained fifteen lines,
  `GuardedCommand::ALL` among them,
  and holds 12,415;
  and three guards read spellings they could not:
  a direct `libc::syscall` in the commit path's files,
  a `Command::new` of any literal that ends in `systemctl`,
  and a write to `libc::STDERR_FILENO` in the note lane.
  Each new control was red first,
  and the `source_guard` module went from 225 tests to 231.
  A name behind a feature gate still counts as published,
  and the list's header names the three.
  What it chose waits in `doc/handover/questions-for-the-owner.md`.
- Of the five rounds one is left,
  issue 6641's ("Rounds running").

#### What the linter's alpha.23 read found

A Claude Sonnet read,
same vendor and not independent
(`~/temp/agent/monochromatic-lint-alpha-23-release-review-2026-11-27/keep/review.md`,
scratch):
no High,
one Medium and eight Low points.
Over 16,081 lists and 48,243 runs no list moved between read and refused,
between exit statuses or in its set of findings against alpha.22,
with a comparison first shown able to see a move.

- Medium:
  a long run of whitespace after `file-uses` in a six-word line costs time and memory with the square of the run.
  At 64,000 characters alpha.23 took 3.3 s and 2.17 GiB where alpha.22 took 11 MiB,
  so the reader's four-mebibyte bound no longer bounds the cost
  (`readings_of_a_written_folder`,
  `layout_allow_entries.rs`).
- Low:
  five sentences that promise an edit the next run refuses or finds dead again;
  a README sentence the list reader contradicts;
  two points of the alpha.22 read neither fixed nor listed as left;
  and three controls that would stay green under a named edit.
- One round took all nine.
  The coordinator's pick,
  with the veto open,
  stays:
  the read,
  fix and release loop of this package ends at the first read that finds no High and no Medium.
- That round is merged (`de1bd36d08`).
  Each point was reproduced on the released binary first.
  The names a gap could have left are offered to the file system one at a time and none is kept,
  and no name is offered whose first folder is longer than the package folder's longest name.
  On a line of 64,033 characters the branch took 1.10 times alpha.22's wall time (range 0.73 to 1.21)
  and 0.97 times its peak memory,
  where alpha.23 took 981 times and 180 times,
  over three rounds of three arms in rotating order.
  Every sentence a dead line or a refusal prints is true for the edit it names,
  and the three controls the read named were each shown red.
  Over 16,081 lists and 48,243 runs no list moved between read and refused against alpha.23;
  2,370 differ in message text alone.
  U+2800 stays as written and the README says why:
  the set a message spells is Unicode's `Cf`,
  `Cc` and `Default_Ignorable_Code_Point`,
  and none of them names it.
  The README dates that measurement 2026-11-28,
  the day in UTC of evidence written at 2026-11-27T21:03:10-05:00;
  the merge of issue 6642's branch carries the correction.
  The round's first release build ran outside a capped scope,
  by its own report.
- Issue 6642's round is merged (`cdf69930fd`),
  after its branch passed the package's `lint`,
  `lint:rustfmt` and `test` tasks (1196 of 1196) on the tree that holds the other round,
  with the README's measurement day corrected to the host's own.
  The released fix replaced one space with a break and touched nothing else,
  so a word that ended the old line became a line of its own,
  written at the paragraph's opening depth.
  Now a fragment of one or two words that this pass's own break cut,
  and that ends at no break-point character,
  joins the line that continues its clause while the joined line stays under 120 characters;
  a line the fix writes takes the indent of the paragraph's first continuation line;
  and no line the author wrote is rewritten.
  Over the wrapper's source as it stood before the merge that brought the lone words,
  the released fix wrote 834 doc lines of one or two words and the branch 732,
  every one of which ends where the rule wants a line to end.
  The words of all 4,141 files are unchanged,
  by a comparison first shown to see one changed word.
  Six controls were each shown red.
  The wrapper's own doc comments are not repaired:
  that is a round of the wrapper's once a linter holding the fix is released.
  Its proposal,
  `doc/planning/monochromatic-lint-a-line-break-at-no-semantic-boundary-is-or-is-not-a-finding.md`,
  measured 199,796 breaks at no boundary in paragraphs the rule reports nothing for,
  in 8,623 files,
  and ranks first the reporting of the 158,514 a removal fix can join;
  its three questions wait in `doc/handover/questions-for-the-owner.md`.
- `rust/avoidable-closure` runs on no host run here:
  its Dylint driver is absent and the linter does not download one,
  so every count of findings taken on this machine leaves that rule out.
  Started through `mise run //:lint:monochromatic-lint` it is skipped the same way,
  measured on `package/mpm-shadow/cli` at 2026-11-27T20:15:12-05:00.
  It is issue 6643,
  whose round built nothing and whose proposal is merged (`e63f6d9226`):
  `doc/planning/monochromatic-lint-who-provides-the-dylint-driver-and-what-a-skipped-rule-returns.md`.
  Nobody in this repository provides the driver:
  the plan holds the nightly,
  `cargo-dylint` and `dylint-link`,
  and no step builds a driver;
  this machine has no `~/.dylint_drivers`,
  and its `cargo` on `PATH` is the repository's shadow and no rustup proxy.
  A skip is a warning of the rule's own code at the package's `Cargo.toml`,
  merged into the findings,
  so neither code nor position tells it from a finding;
  it exits 0 unless `--max-warnings` is passed.
  In a container the round built a driver on its fifth attempt
  (it needed `llvm-tools-preview` beside `rustc-dev`,
  and a `TMPDIR` the image's `git` could write),
  and the rule then found six findings in `package/wg/cli-quicker-exempt` in about two seconds.
  It ranks first that the toolchain install builds the driver,
  which waits on the owner's pause of that install (issue 6316),
  and says a code of its own for a skip is worth taking with whatever else is picked.
  The issue stays open,
  and the choice is the owner's (`doc/handover/questions-for-the-owner.md`).

#### What `mpm-shadow`'s alpha.18 read found

A Claude Sonnet read,
same vendor and not independent
(`~/temp/agent/mpm-shadow-alpha-18-release-review-2026-11-27/keep/review.md`,
scratch),
with a real `rustup 1.29.3` in a container:
no High,
two Medium and seven Low points.
It demonstrated that the Rust row's programs come from the pinned toolchain
(`rustc` through the exposure folder prints the pin's `5f4effd57 2026-11-13` where alpha.17 printed an older toolchain's),
and that the removal deletes symbolic links no row declares and leaves files,
directories and hard links.

- M-1:
  the removal deletes another install's in-flight temporary link,
  and a second remover fails on a link the first already removed,
  so two installs at once can exit 1 where alpha.17's both passed;
  the folder is right afterwards and a rerun passes.
- M-2:
  the exposure folder's `mpm-shadow` link is deleted when the running binary is no installed copy,
  although an installed one exists and the link resolves.
  This repository's plan declares the row,
  so a repository scope keeps it:
  the three installs made in the main checkout since alpha.18 each named no removed link,
  with 174 names in the folder before and after.
- Low:
  the directory rule accepts the channel,
  a hyphen and any text that is no date,
  so `mine-two` answers for `mine` and a second host triple answers for the pin;
  a removal made before a later failure is never named;
  an ordinary failed install's message lost the true sentence that the next run publishes without downloading;
  "a later run removes it" does not hold across process namespaces;
  a stale doc comment;
  three controls that stay green under named edits;
  and `list` starts `rustup` three times with no bound on the wait.
- Three rounds take them,
  by area ("Rounds running").
- The round on the failed install's message is merged (`27b745b6dd`).
  All six messages of an ordinary install's failed publish,
  where the read counted five,
  end with one sentence:
  a later install of that version publishes the kept copy without downloading while the version's name holds no finished copy,
  and removes it as a second copy once the name holds one;
  an install in another process namespace leaves it;
  and `prune --toolchain` prints its name.
  Each message was built on alpha.17,
  on alpha.18 and on the branch,
  with the install that follows.
  The other namespace was a real one,
  a rootless container sharing one scratch folder:
  the install inside printed nothing and left the copy,
  `prune --toolchain` inside printed its `left` line,
  and an install outside then removed it.
  The recovery's flag that no test read is now a value of three kinds,
  held by controls for the two kinds a fixture can build.
  No control holds the message that needs two failed renames in one publish,
  nor either recovery arm inside a run's warning.
  The package's task passed 934 of 934 after the merge of `main`.
  What the round chose waits in `doc/handover/questions-for-the-owner.md`.
- The round on the Rust row is merged (`b16e1fdca2`).
  The three wrong directories the read named reproduced on this machine's own `rustup 1.29.3`,
  with two of six links,
  and then all of them,
  taken from a neighbour.
  The row's directory is now the parent of the path `rustup which --toolchain <channel> rustc` prints,
  one `rustup` start for each such row in an install,
  and the name rule is gone:
  rustup's own source shows the host triple cannot be worked out by another program.
  `list` reads one listing,
  the targets being its `rust-std-<triple>` components,
  so it starts `rustup` twice for such a row where alpha.18 started it three times.
  No deadline ships for `list`:
  the measurement says what an answer costs and not where a late one is cut off,
  and the README says so.
  A control holds the failing listing's line,
  red under the plant the read named.
  The package's task passed 938 of 938.
- The round on the exposure folder's removal ended 2026-11-27T21:30:28-05:00 with its report read whole,
  and its branch does not merge:
  the Rust row's merge changed `exposure.rs` under it,
  and a round joins the two ("Rounds running").
  Both races of M-1,
  M-2 and the unnamed removal reproduced with the released binary.
  On the branch a temporary link's name carries the install that holds it,
  by the four facts the hidden names already carry,
  so a rebuild leaves the link of an install that runs or cannot be judged,
  and removes and names one whose install ended;
  the `mpm-shadow` link stays while it resolves into an installed copy,
  whatever binary runs the rebuild;
  a link another run took first is no failure;
  and a run that fails at a later removal still names the ones it made.
  Eleven controls,
  six plants,
  and the package's task 945 of 945 on its own tree.
  Its three questions wait in `doc/handover/questions-for-the-owner.md`.

#### What `cargo-shadow`'s alpha.16 round did, and its alpha.17

The round on the four Low points of the alpha.16 read is merged (`05e0758cb7`).
It reproduced each with the released binary in containers:
the asking again is spent in 11 of 24 states,
five asks and about 0.86 s a call,
each now named in the README with its cost.
An ask the manager answers with a refusal of its own ends the asking:
three asks and a median of 56.5 ms with `app.slice` masked in a container's own manager,
where alpha.16 made five asks and took 895 ms.
Because a failed start that names the unit can be a transport that died,
a refused start is followed by one more ask,
which ends the asking only if the manager answers it.
The read of the host counts the transports systemd takes at versions 255 and 262,
quoted from systemd's own source.
Ten controls were each shown red,
and 48 calls begun inside 48 real reexecs of a container's manager all took the delegated route.
The package's task passed 978 of 978 after the merge of `main`.
`0.1.5-alpha.17` is release commit `2ff68958b1`
(dispatched 2026-11-27T20:26:16-05:00 from `05e0758cb7`,
release run 46093112468 and publish run 46093197842;
the follower's log holds `PUBLISHED`),
locked (`7f21ae4ef9`,
which reached `main` without a merge commit:
"How work is run side by side")
and installed here,
where `.cache/toolchain-bin/cargo-shadow --version` says `0.1.5-alpha.17`;
the install named no unresolved bin and no removed link,
and the session's `git` link was put back at 2026-11-27T20:35:23-05:00.
Its second pass,
a Claude Sonnet read,
same vendor and not independent
(`~/temp/agent/cargo-shadow-alpha-17-release-review-2026-11-27/keep/review.md`,
scratch),
was read whole.
It ran the two released binaries and a byte-identical copy of alpha.17 in containers whose own manager it could reexec,
mask and stop,
and found no High,
two Medium and three Low points.

- Medium 1:
  after a failed start,
  any answer to the next ask ends the asking,
  a path among them,
  which is then thrown away
  (`control_group_route_discovery.rs`,
  the arm that answers `may_be_put_again: false` for an answered confirm ask).
  A unit load that fails once,
  a start whose reply is lost and a start refused once each took the weaker path in every call of alpha.17,
  where alpha.16 delegated in every call,
  each state built with a stand-in in the ask path.
  With a real reexec aligned to the start,
  2 of 24 calls of the alpha.17 binaries took the weaker path and none of alpha.16's 12;
  with the reexec fired from outside,
  none of 12.
  The round's own calls in real reexecs never sent a start from the branch binary,
  because the recipe's layout keeps `app.slice` active,
  so they did not exercise this arm.
- Medium 2:
  with a stop job queued on `app.slice`,
  the first ask answers the path,
  no start waits behind the stop,
  and the scope's own start is refused as destructive:
  10 of 10 calls of the alpha.17 binaries failed (exit 1,
  or 101 in shadow mode),
  where alpha.16 waited about 3.7 s and ran in 7 of 7.
- Low:
  three edits no control would catch
  (the arm after a failed confirm ask and the path it discards,
  the sleep at the call site,
  an address beside a runtime directory);
  five record sentences that are wrong or imprecise;
  and a reexec that begins after the discovery fails the whole call in both releases,
  which no record says.
- Borne out:
  one `systemctl` program a call where alpha.16 started two;
  three asks where a masked slice took five and an answer that is no path took ten;
  the span spent in the eleven named states;
  and of 240 compared files 21 differ,
  each by the version or by the missing `start` row.
- `0.1.5-alpha.17` is the copy installed on this machine,
  so a build begun here while `app.slice` has a stop job queued fails until the next release.
- Weighed by the coordinator:
  every point earns a fix,
  in two rounds by area ("Rounds running"),
  one on the discovery's two arms with the control that holds them,
  one on the records and the two other controls.
  Issue 6645 states the two Medium points.
What the round chose that its brief did not decide waits in `doc/handover/questions-for-the-owner.md`.

### Releases of 2026-11-27

Alphas of the wrapper and of the tools on its path are published unasked
(`doc/decision/cli-git-wrapper-owner-answers-2026-11-12.md`,
"How far the permission to publish reaches").
A release is dispatched when `main` equals `origin/main`,
which the dispatch task checks itself,
and `main` may move while it runs
("How work is run side by side").
After every `mise run toolchain:install` in the main checkout the session's `git` link points at the toolchain launcher,
and it is put back on the build of 2026-11-22
(last at 2026-11-27T20:35:23-05:00).
What the wrapper's alpha.48 behaviour pass found,
with no High finding:

- Medium:
  a shell alias still takes a slot removal or a prune past the refusals,
  in alpha.47 and alpha.48 alike
  (`git -c 'alias.x=!git worktree remove' x -f -f <slot>`),
  because the planner reads the alias's name where the destructive-command guard reads its resolved command.
- Medium,
  a regression:
  since `6ea539bc91` a notice is handed to the stream that writes off the command's thread,
  which answers success before anything is written,
  so the record of a failed push or a kept worktree is removed although its line never reached anyone.
  With standard error a pipe whose reader is gone,
  alpha.47 keeps the record and alpha.48 does not.
- Low:
  three sentences of the slot refusals
  (one fires for an empty record folder,
  one advises a settling command no grammar admits,
  one calls `--` an option);
  the merge pass does not reach a last line without a final newline;
  a named pipe among the merge records stops every command with no way out named;
  a drain sizes its batches from one reading of the host per round;
  the kept-copy line prints a holder's name unfiltered;
  the README holds a pasted handover excerpt.
- Right as claimed,
  exercised with both binaries:
  the slot refusals for the spellings they name,
  the merge pass's union of deletions,
  the self-check's memory (62.7 to 66.1 MB in alpha.47,
  9.3 to 11.0 MB in alpha.48),
  named pipes answered and not waited on,
  and the typed ssh relay ending about 100 ms sooner at a terminal.
- Read only,
  because nothing outside the process can start them:
  the prune race,
  the conversion environment's precedence,
  the service-only changes.

`cargo-shadow` `0.1.5-alpha.12` followed:
release commit `581f7754a8`,
locked (`e3c3dd43f1`) and installed.
Over alpha.11 it holds issue 6578's retirement of a run's scope unit when the run ends (`5daa8810b9`),
a refusal that lists the members of every control group below the scope,
each read through a pidfd,
within a bound of 500 ms beside its measurement (`c59a4f5fd5`),
and the manifest and surface tidy (`d134b5f836`).
Its release note's last clause is wrong
(it names issue 6576,
a test-only change that alpha.11 already held);
a comment on the release commit corrects it.
Before a release note is written,
`git merge-base --is-ancestor <merge> <the last release commit>` says what the release before already held.

What the wrapper's alpha.48 splits pass found,
with no High finding.
Its verdict:
"no change of behaviour" is not demonstrated.
Of the items compared at both ends,
97.9 percent have identical text,
no test-only switch is in either release binary,
no static became two and no trait impl was lost;
but 318 of the 344 refactor commits carry changed lines that are not moves,
and the pass read three of them in full.
Neither pass ran `git commit` in a release binary,
because without a session bus every commit form stops at one refusal,
so for the service paths the evidence is the reading,
the gate,
and the service-manager install proof,
which issue 6587's second round ran to its end (`fdd9ce0e83`):
alpha.48 installed over alpha.47 in a container of the gate image,
and the proof printed

```text
THE PROOF PASSED: the instance the older release loaded started as this release, and the pair of commits each landed once
```


- Medium:
  at least 31 definitions became nameable from outside the crate,
  seven of them private helpers a split made bare `pub`
  (one removes directories).
- Medium:
  the extract-function refactors are unread but for ten commits.
  A second read of the seven the pass named first,
  all on the client's and the service's commit paths,
  found each equivalent by reading
  (`e90482f100`,
  `dde5c4be3b`,
  `bad45d0420`,
  `01c2f32e2c`,
  `6a24c6342b`,
  `f4e8d2fb9b`,
  `8af8f56255`),
  with one guard that now covers less
  (`source_guard/hook_self_invocation_tests.rs` reads 263 of the 699 code lines the commit preparation spans),
  which the list-guards round takes first.
  About three hundred refactor commits stay unread.
- Medium:
  71 pairs of a source guard and a file it names,
  where code left the file in a split and the guard does not name where it went.
  Two were checked and neither was a hole.
- Low:
  the sweep's warnings carry four new log tags;
  a commit labelled `docs` changed the `shutdown-unmount` help row (`066e33a47d`,
  noted on the commit);
  the panic notice's source locations moved;
  34 items lost a platform gate on split-off files,
  with no effect on x86_64 Linux;
  tag constants are copied into more files;
  the README's allow-list counts are stale;
  two commits in the range cannot build alone;
  the linter the binary carries moved four alphas.
- Its count of tests that run under the gate's build is the gate's own,
  12,115.

What `mpm-shadow`'s alpha.13 pass found,
with no High finding:

- The new message for a start refused as busy says an upgrade or an install is the usual cause,
  which the release's own record contradicts;
  it says "the program's file" when the held file can be a script's interpreter;
  and it promises the same command works afterwards,
  which is false after a writer that died half way.
- "Every writer this tool starts publishes by rename" is false for `go install` across file systems
  (the pass traced it writing the published path in place for about 4 ms),
  though neither the shadow nor the launcher starts that path;
  `uv tool upgrade`,
  which the self-upgrade runs,
  was read and not traced.
- "No measured window" leaves out windows this repository measured,
  0.5 to 3 ms while a forked child holds a copied write descriptor.
  So the pick not to repeat such a start is before the owner again
  (`doc/handover/questions-for-the-owner.md`).
- A program that vanishes during a start is reported three ways,
  one of which repeats advice the release removed.
- Right as claimed:
  43 commands compared between the two binaries,
  41 identical and two differing only in the version string.

What `cargo-shadow`'s alpha.12 pass found,
with no High finding:

- "Never what is decided" holds by reading:
  both releases refuse exactly when the direct child was not reaped or the report is not empty,
  and nothing in release code reads the report's words.
- The retire of a run's unit makes the release slower.
  A trivial shadow call took 1.52 times alpha.11's time at the median
  (range 0.89 to 2.17 over 18 interleaved rounds,
  against 1.00 and 0.68 to 1.25 for a same-binary control),
  about 15 to 27 ms more per scoped run:
  in ten of ten logged retires the manager had already collected the unit,
  so both questions to it bought nothing.
  With a manager that does not answer,
  each run's end waits two five-second bounds.
- The retire's gate reads a missing scope directory as empty without the proof the stop itself requires,
  so it could stop a unit the same run refused as possibly still writing.
  Read from the source,
  not reproduced.
- No control reaches the decision for two of the leftovers a stop can report,
  so an edit that accepted such a run would stay green;
  the listing's bound is looked at only between reads and has no control.
- A program that outlives its run is treated alike by both releases:
  killed on the scope path,
  left alive on the process-group fallback.
- Right as claimed:
  41 commands compared between the two binaries,
  identical but for the version line.

What the linter's alpha.19 pass found:
no High and no Medium finding,
and both of alpha.18's Medium findings fixed
(the near-miss search is linear again,
0.93 to 1.07 times alpha.17 where alpha.18 was 69 times).
Every printed key reads back with the same binary,
and the layout gate on `package/git-policy/cli` and on the ZFS binding reports nothing,
with a control that removes the allow keys reporting 558 and 16 findings.
Its six Low findings had one round,
merged as `2fcfbfc9f8` and released in `0.5.2-alpha.20`
(release commit `d4a9bb5544`).
Of 220 spellings of one `uses` line,
alpha.19 refused 163 and stopped the whole rule,
with a message that named no whitespace;
alpha.20 refuses none:
31 read as the use and 189 are reported as a dead line that spells the whitespace it read,
since a second whitespace character beside a name belongs to the name,
which is the rule the README states.
alpha.16 read all 220 as the use,
so a list aligned with more than one character in a gap is reported where alpha.16 took it.
A mistyped `file-uses` line is told what is wrong with it,
the two wording gaps and the README mapping gap are closed
(the README gains "How a list written before alpha.20 maps"),
and the edits no test would catch have tests.
The constructed walk shape that costs 1.64 times alpha.16 is left:
no package of this repository has it.

The lock names each tool by its own id:
`cargo:monochromatic-lint`,
`cargo:monochromatic-cargo-shadow`,
and plain `mpm-shadow`.
`toolchain:update` refuses an id the plan does not hold and writes nothing,
which is how `f8f678c26d` came to name `mpm-shadow` in its message and lock only the linter
(a comment on that commit corrects it).
A command line that changes into a worktree ends there when the change fails (`cd <worktree> || exit`):
on 2026-11-27 one `toolchain:update` ran in the main checkout because the worktree it meant was gone,
and only that refusal left the main checkout's lock as it was.

### What the split rounds left

On `main` at `6037c4e0f8`,
measured with the installed linter at 2026-11-27T20:12 -05:00,
the wrapper has no layout finding and 13 files over the budget,
down from 264 before the splits:
`foundation/lines_written_by_a_thread.rs`,
`health/doctor.rs`,
`health/doctor_filesystem.rs`,
`worktree/mod.rs`,
and nine files of `invocation/`.

- `foundation/lines_written_by_a_thread.rs` waits for the owner,
  with its loop.
- The rest wait for the move of group-folder files into child folders,
  round 7 of the plan's order,
  which follows rounds 4 to 6:
  the files of `invocation/`,
  `health/doctor.rs` and `health/doctor_filesystem.rs`,
  where a new sibling file needs a `group-file` line,
  and `worktree/mod.rs` (339 lines of module declarations).

A split is not always a pure move.
The landing split's last commit (`3b9ada357b`) boxed the commit inside the service's answers
and made the commit command pass its own copy of its client configuration,
both for Clippy after the move;
the wrapper's next release pass reads it as a change.

A brief names a file over the budget only from the linter's listing of `main` taken when the brief is written:
the round for `command/rule_linked_worktree.rs` was briefed from an earlier count,
found the loop break had already brought the file to 298 lines,
and split nothing (`bb5d25980b`).

The coordinator's reading,
not the owner's:
a branch is open when a round or a session works on it or this handover names it,
because each crate's reformat already conflicts with every older branch in these files
(`57bf0918fc` for the wrapper,
`f262797330` for `mpm-shadow`,
`b4ae047c05` for `cargo-shadow`,
`639f90484d` for the linter).
The briefs say so,
and tell a round to leave a file only for a branch whose newest commit is later than 2026-11-26T12:00-05:00.

Outside the wrapper,
`mpm-shadow`,
`cargo-shadow` and the linter have no file over the budget,
and the two `truepeak` files wait on their branch.

### What the rounds found

- What the rounds reported as failing or misbehaving is on GitHub:
  issues 6524 to 6617,
  each filed after a search.
  Closed by a merged round:
  all of them except those with a round ("Rounds that ended,
  and what each left") and these,
  which stay open on purpose and wait in `doc/handover/questions-for-the-owner.md`.
  Issues 6555,
  6557,
  6559 and 6563 have merged proposals.
  Issue 6556 (Pi starts that end without a run) has its explanation and its printed line merged (`faa52ab2af`);
  sizing the two bounds from a measurement under load is the owner's.
  Issue 6566 (a landing reads and hashes every changed path,
  about 30 s for a change list past `MAX_CHANGES` under load,
  against native Git's 0.6 s) names a cheaper form that is a decision,
  not a repair
  (`doc/planning/cli-git-landing-index-reset-reads-every-changed-path.md`).
  Issue 6571 (a passing control wrote under the real home cache) is fixed for every writer but one (`c1a3b33db5`):
  a worktree removal's service-side Git still writes there,
  and reaching it changes a signature release callers see
  (`doc/planning/cli-git-removal-work-cache-place-the-caller-hands.md`);
  the issue was closed by the fix's commit and reopened for that writer.
  Issue 6577 follows issue 6528,
  `cargo-shadow`'s install step refusing a command because a process "could not be stopped":
  the round measured that the kernel does not count a member that has exited and waits to be reaped,
  so the zombie reading was not the cause,
  and made the refusal say how long the stop waited and name each member left (`41b9597adf`);
  the 5 s the stop gives a killed scope was sized on a quiet host and is unchanged,
  and what refused under heavy disk load on 2026-11-26 is not known.
  Issue 6523 (`package/ide/app` does not compile on `main`) has its compile fix on `main` since `4df352241d`,
  and what is left of it is the owner's (a comment on the issue).
  Issue 6560 is closed as a duplicate of 6543.
- Fixes that changed release code,
  each with a control that fails without it,
  which the next release pass of its tool reads:
  issue 6544 (`b5696a3eb0`),
  where the discarded prune removed a directory whose change time had not moved
  although a file had been added inside the tick of its listing,
  and now reads what the directory holds on every removal;
  issue 6529 (`5914dd3810`),
  where the word-level merge pass wrote back lines one side had deleted;
  issue 6547 (`ccea7826d5`),
  where the wrapper's self-check now compares two files a chunk at a time instead of reading both whole;
  issue 6571 (`c1a3b33db5`),
  where the conversion environment of a landing,
  an add,
  a ladder snapshot and a removal's clean check now lets a variable its caller named win over the process's own
  (the caller's entries were appended and the first match won),
  and the clean check reads that environment where it read the bare process's;
  and,
  in `cargo-shadow`,
  issue 6528 (`41b9597adf`),
  the stop's report,
  whose new test-only switch sits behind `cfg(test)` at its call site in `control_group_stop.rs`.
- The wrapper package's TypeScript side was red on `main` and nothing ran it.
  Issue 6562's round reported it,
  and at `1c8acd5bfb` `test:unit` exits 1 on two files:
  `e2e/service-mode-words.unit.test.ts` looks for a declaration line that became a slice (issue 6579),
  and `bin/native-planted-controls.unit.test.ts` dies with "Invalid string length"
  because a recursive read of `src/native` follows the `uses/` links
  and returns 163,079 entries for a tree of 3,809 files (issue 6580).
  `lint:oxlint` exits 1 with 850 errors and 1856 warnings in fifteen files:
  eleven guest scripts in `e2e/`,
  two fence measurements in `perf/`
  and `bin/service-manager-install-proof.ts` (issue 6581).
  The release gate runs the Rust suite and Clippy,
  a round runs the Rust modules it touched,
  and the coordinator's tip check ran `native:check`,
  Clippy and `source_guard`;
  where the TypeScript checks should run is a proposal for the owner (issue 6584).
- A threaded test run of the wrapper on the host reached this account's own service manager (issue 6586).
  In a run of every test whose path holds `worktree`,
  a control's add asked the manager to start `monochromatic-git-policy-cli@<instance>` units for its fixture,
  the program the manager ran was the test binary
  (100 starts refused with "Unrecognized option:
  'journal'" between 07:31:27 and 08:01:43 on 2026-11-27),
  and the shared link `/run/user/1000/monochromatic-git-policy-cli-bin` pointed at that binary
  until another session's wrapper command pointed it back.
  Release code remembers the running executable,
  and the route provisions whenever the process's own runtime directory and session bus exist.
  Issue 6582 put its two controls in a child with a runtime directory of its own (`eeebbfbe22`);
  issue 6586's round makes it impossible for the whole test process
  and writes a proposal on whether the release should refuse to provision from a program that is not an installed wrapper.
  The coordinator stopped and reset the ten failed units the runs of 07:31 to 07:40 left,
  and removed their sockets and notes,
  each note naming a Git directory under a cargo build folder's `test-fixtures/`.
- Issue 6580's fix found seven planted controls whose text no longer occurred in the file their table named
  (four moved by splits,
  three reflowed by the reformat);
  the test that would have said so had been dying before it compared anything.
  They are repaired (`64e3eb1a55`),
  and that they still compile and that a campaign notices each is owed to the next `native:mutation:planted` run.
- This host's AppArmor profile for Git confines a container's `/usr/bin/git` as well,
  since it attaches by path.
  The service-manager install proof runs Git as the container's root in a directory its user owns,
  which needs a capability the profile refuses on purpose,
  so the proof has not run to its end since 2026-11-13 (issue 6587;
  `doc/troubleshooting/git-cannot-mkdir-inside-the-container-proof.md`).
  A round's session cannot read `/var/log/audit/audit.log`,
  where these records are and the kernel journal's are not:
  when a round reports a refusal only one program meets,
  the coordinator reads that log.
- `package/container-limit/core`'s ledger of scripts that name `podman` fell behind again and is brought up to date (`f69208a309`);
  its scan reads no Rust file,
  which issue 6594 holds.
- The service-manager install proof passes end to end (issue 6587,
  `6b1a8e91f4` and `fdd9ce0e83`).
  Two things stopped it:
  Git wrote `/work` as a user that did not own it,
  and this host's Git profile refused the marks file `git fast-import` writes beside the journal
  when the service cannot hand it an unnamed scratch file,
  as in a container's root home.
  The coordinator's first rule for that name matched nothing
  (`@{int}` stops at ten digits and the marks carry sixteen);
  the corrected rule is proved with a refused control name
  (`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
  seventeenth batch).
  The reload proof's control arm cannot pass:
  it needs a release from before the remedy of 2026-11-12,
  and its choices are
  `doc/planning/git-wrapper-reload-proofs-control-arm-without-a-pre-remedy-release.md`,
  ranked C > A > B there.
- The gate's suite container peaked at 1,486.0 MiB in a measured run,
  and 892 MiB of it was one control for 25 seconds,
  `service::service_main::tests::the_service_setup_and_its_words_are_used_only_by_the_service_mode`,
  which reads the source tree with a walk of its own that follows the `uses/` links (issue 6598).
  The suite's bound is 4 GiB since `2da7ccd13a`,
  beside that measurement.
- The five controls that failed once under load are each fixed by what the control owns
  (issues 6589 to 6593,
  merged `69b4c4dc3e` to `c6e5d5846c`),
  each with its failure planted and not waited for.
  Two left proposals:
  `doc/planning/cli-git-notice-of-a-stopped-step-keeps-promising-a-stop.md`
  and `doc/planning/git-policy-cli-a-step-whose-watch-cannot-start.md`.
- The container ledger reads Rust sources now (`ff1ae3e5c7`,
  issue 6594):
  four Rust files name the tool and none starts a container.
- A round's `edit` tool wrote a planning file into the main checkout and not its worktree
  (issue 6586's round,
  twice seen untracked there,
  gone later).
  The branch that fences a run's writes is held (issue 6378).
- Rust's `CommandExt::exec` sets `SIGPIPE` back to its default in the calling process,
  and a failed `execve` leaves it there.
  Two controls did that inside the test process,
  which is what ended a ten-module threaded run by signal
  (issue 6604,
  `e4d7915941`,
  `doc/troubleshooting/rust-exec-leaves-sigpipe-at-its-default-in-the-calling-process.md`).
  What the release does in the same state is issue 6607.
- `gh issue view <number> --comments`,
  piped,
  prints the comments alone and nothing for an issue without any;
  `doc/agent/issue-tracker.md` now gives the `--json` form.
- Three more controls that failed only in wide or loaded runs are fixed by what each owns,
  none through a fault of release code:
  issue 6599 (`0a2b5b0176`,
  a recycled time-namespace number another process took first),
  issue 6602 (the moved-log control met a journal lock a fork of another test's thread had copied)
  and issue 6603 (`286f9d6dfc`,
  a deadline control armed its hold after the read had started).
- `native:ts-checks` now sizes its scope from a measurement (`71d9ce980d`,
  issue 6605):
  1,024 tasks against a measured peak of 463,
  4 GiB against 2,218.8 MiB,
  and 800% of one CPU,
  where 200% made the unit suite take 2.66 to 2.79 times as long;
  it says so when a bound and not a test failed.
  Two unit tests that give a child a fixed 1.5 s to start are issue 6608.
- Two rounds wrote into the main checkout by mistake and undid it
  (issue 6586's through its edit tool,
  the four-small-points round through a command without its own `cd`);
  both are on issue 6378,
  which holds the branch that fences a run's writes.
- After a failed `exec` of the real Git the wrapper now puts the `SIGPIPE` action back
  and ends by the status its exit contract gives a command that ran no Git (`65bffa15ec`,
  issue 6607):
  with standard error a pipe whose reader was gone,
  the parent read `SIGPIPE` before and status 2 after.
- The slot-state guest arm runs to its end again (`3823f2101a`,
  issue 6595):
  it plants its plan through the removal's own writer,
  and in two guest runs every marker after `SLOT-STATE pending-add` printed and no later assertion failed,
  so the release code was not at fault.
  A host control now fails when a planted plan is one the add's reader refuses,
  and the package guide says a control plants a plan through the writer.
- The four shadow crates' copies of their shared rule agree again (`7e215a38c2`,
  issue 6609),
  and each package's own test task runs the comparison,
  shown red in all four by one planted byte.
  `pnpm-shadow` and `search-shadow` print a refused start with the system's own answer;
  neither is released,
  because the standing answer on alphas does not name them
  (`doc/handover/questions-for-the-owner.md`).
  `cargo-shadow`'s start uses a check of its own with two outcomes the shared refusal has no word for,
  and keeps its own line.
- A check scope's last counters keep every part they read (`07f9232af0`,
  issue 6610).
  The kernel takes a dying control group's files away one at a time before its directory,
  so a reading can lose `pids.events` while `pids.peak` still answers;
  one real dying scope read 144 times gave one such reading.
  The runner kept that reading in place of a whole one,
  which is why `native:ts-checks` could miss saying that a bound and not a test had failed.
  The record is `doc/troubleshooting/a-cgroup-gives-its-files-up-one-at-a-time-before-its-directory-goes.md`.
- Two verification-process unit tests wait for their child by the record it writes (`3dbb56c5a6`,
  issue 6608),
  and arm the bound they assert only then;
  a two-second delay planted in the child's start leaves them green,
  and each still fails when the code under test is broken the way it exists to catch.
  `runCommand` was split at the seam where a bound is armed,
  with two exports more and no change of behaviour.
- On `main` at `07f9232af0`,
  in a worktree no merge moved,
  the package's whole `test:unit` exited 0 with 176 `[PASS]` suite lines and none failing,
  and `lint:types` exited 0.
- The wrapper package's `lint:oxlint` exits 0 on `main` (`0f33063f26`,
  2026-11-27):
  `Found 0 warnings and 0 errors.` on 341 files,
  where issue 6581 had counted 850 errors and 1,856 warnings in fifteen TypeScript files with nothing running the check.
  Every guest driver was shown to run in its guest against `main`'s version,
  with the lines that say what an arm did compared and found the same.
  The rewrite found that the start probe's shared header had been broken since `dcf5877982`:
  an unquoted handler assignment ran the wrapper as a command,
  so every arm after `green` stopped at its fourth line,
  in `main`'s driver and the rewritten one alike;
  it is quoted now,
  and those arms run.
  Three of that probe's arms
  (`m3-kill`,
  `timings`,
  `timings-boot-first`)
  were compared byte for byte and not run in a guest;
  issue 6612's round ran each once,
  and each passed.
- `native:ts-checks` says what a passing check ran (`8dcc43ed1e`,
  issue 6613),
  and refuses a unit test run that reported no suite or fewer suites than the package holds unit test files.
  On `main` at that commit it printed
  `89 unit test files, and 186 suites reported [PASS]`.
  The type check can report no count of files without `--extendedDiagnostics` in the shared task template,
  and says so.
- The gate's reader of a container's control group keeps a whole reading (`9ea0d98dda`,
  issue 6614),
  by the rule issue 6610 gave the check scope's reader:
  a reading that lost part of its group neither replaces a whole one nor throws,
  so a container killed inside its memory bound whose `memory.events` went before a read is still reported as killed.
  The reader answers two things apart,
  a directory it cannot find and a reading that is not whole,
  because only the first ends a role in the recorder.
  No real gate has run on it yet:
  the next whole gate is its first.
- The managed-lock guest driver builds within its guest and its arm ran to its end (`239b91c5ee`,
  issue 6611):
  at four jobs the guest's kernel killed a `rustc` at 6.67 GiB of resident memory in an 8 GiB guest with no swap;
  at one job with the 6 GiB swap file the sibling drivers add,
  the build's peak was 7,019,956 kB of 8,131,780 kB,
  with 7,216 kB of swap used,
  in 22 minutes;
  the arm printed its 15 markers and passed.
  The driver and the Sanoid driver each have a `mise` task
  (`e2e:managed-git-lock`,
  `e2e:sanoid-coverage`),
  and the readings are under `doc/planning/mvm-guest-memory-held-for-a-rounds-life-evidence/`.
- Since 2026-11-27T12:49 a hook another session merged (`88632f16aa`,
  the owner's "Clamp + prefix (Recommended)") moves a main-session Bash command that runs past two seconds to the background,
  and its result arrives as a task notification;
  `doc/planning/claude-code-bash-caller-timeout-cap.md` holds the design and the probes.
  A wait loop is started with `run_in_background: true`,
  and a step that depends on a merge or a commit is issued after that command's notification,
  never beside it.
- `native:ts-checks` prints the commit it ran on (`3c88496446`,
  issue 6617):
  it asked `git -C <the package folder>`,
  which the wrapper refuses away from a checkout's root,
  and kept the empty answer in silence;
  it now asks from the checkout's root and says why when the commit cannot be read.
  No other `git -C` in the package's `bin/` is refused in silence:
  each names a checkout's root,
  or runs `/usr/bin/git`,
  or throws with the refusal's words.
- `mpm-shadow`'s launcher names the interpreter that is missing (`4e858efdee`,
  issue 6616):
  for a start the system answers with "No such file or directory",
  it names the program after `#!` for a script and the dynamic linker for a compiled program,
  and names both kinds,
  claiming neither,
  when the program's own bytes are gone.
  Each refusal in its fixture is the kernel's own.
  `being_written_cause_and_remedy` takes a program where it took an option.
  `being_written_cause_and_remedy_for` lived only in commits between the two releases and never shipped,
  by the alpha.15 read's `git log -S`;
  against `0.2.7-alpha.14` that read lists the changed signatures as `publish_staged`,
  `write_complete_marker`,
  `being_written_cause_and_remedy`,
  `describe_nesting_refusal` and a new field `StartFailure.lookup`.
- The wrapper package's unit tests no longer reach the account's service manager (`bc24b2e688`,
  issue 6620).
  Every fixture's `git` is the standing Git on a search path of the fixture's own,
  with no bus address and a runtime directory of the fixture's own
  (`bin/fixture-git-environment-for-tests.ts`).
  Unsetting the bus alone would not have done:
  the wrapper with no bus refuses a commit outright,
  which the round measured with a released binary.
  A control records what each fixture `git` was given,
  and failed with eight calls carrying a bus address when the change was taken out.
  One whole `native:ts-checks` with the account's own environment then left no line for a fixture repository in the journal
  and the shared link as it was.
  The round's commit drew a `final-newline` warning for two new files that end with one newline,
  not reproduced (issue 6622).
- The driver's launcher clears a finished run's scratch folder beside `keep/` (`a40a99467c`,
  issue 6618),
  as the owner's answer of 2026-11-13 has it.
  It clears once the run's unit has ended,
  the run's own work has stopped,
  no process works in the folder and no newer run of the name has started;
  it never follows a link,
  and an empty run name or a home that is not absolute is refused before anything is read.
  A round started before the merge was started by the earlier launcher,
  so its folder stays whole until `~/temp` expires it.
  A launcher killed before its unit ends,
  and a run started by hand,
  clear nothing.
- Claude Code restarted this session unasked at 2026-11-27T14:02:39-05:00
  (`doc/troubleshooting/claude-code-restarts-a-session-unasked-and-stops-its-tracked-background-commands.md`;
  rules `RZN` to `RZG` of `CLAUDE.md`).
  Rounds in their own units ran on;
  the session's background wait was killed and was started again;
  an earlier subagent was still reachable by a message to its id.
- An alpha's release workflow runs no test:
  it commits the bump without compiling,
  and `cargo-publish.yml` builds the binaries.
  The evidence for an alpha is the local run,
  and for the wrapper the gate on `main`
  (`package/cargo-shadow/cli/README.md`,
  "Which host each check runs on").
- `START_RETRY_BUDGET`,
  the five seconds a refused start is repeated for in `mpm-shadow` and in the wrapper's `push/start_retry.rs`,
  cites no measurement and no decision (issue 6597,
  no round yet).
- The two fence measurement scripts of 2026-11-21 under `perf/` can no longer measure what they measured:
  the crate no longer reads their switch,
  and this host takes another rung
  (`doc/planning/cli-git-fence-size-read-beside-source-check.md`,
  "The harness after 2026-11-27").
  They are lint-clean and kept;
  whether they are removed is the owner's.
- `cargo-shadow`'s tests left processes and units behind.
  At 2026-11-27T07:05-05:00 the host held 25 orphaned shells,
  each polling a release file every 50 ms for up to twelve hours,
  left by the killed-wrapper control whenever it ended before its release line;
  the coordinator ended them with `SIGTERM` after checking each one's command line,
  parent and age,
  and issue 6576's round gave every gate-waiting test program a count of its own
  and the control a guard and a leak control (`cf98f01e22`).
  That round also saw `cargo-shadow-<id>.scope` units that stay active with no member,
  twelve of 11,360 started that morning.
  Issue 6578's round found the cause,
  which `doc/troubleshooting/systemd-scope-adopting-an-exiting-pid-stays-running-empty.md` already held:
  the manager attaches the `systemd-run` launcher to the scope,
  and when the run kills that launcher while the start job runs,
  the manager counts an exiting process as attached and never hears the group empty.
  The runner now asks the manager to stop its unit when its run ends and the scope is empty (`5daa8810b9`;
  300 starts at a 40 ms bound left 34 units before and none after,
  and a stop of such a unit took 4 to 5 ms).
  The issue is reopened for one unit the round's last whole-suite run left,
  which no single module reproduced.
  The coordinator stopped the twelve.
- Worktrees of ended,
  merged rounds are not all removed.
  A removal of eleven on 2026-11-27 removed none:
  five were held by those orphaned shells,
  three are dirty,
  and three are refused with "validation failed,
  cannot remove working tree:
  '.git' does not exist".
- A second writer makes issue 6339's shape:
  a build in a worktree whose add was interrupted writes that worktree's own `cargo-shadow` stamp,
  and the owed ignored-state copy can then never finish
  (a comment on that issue has the round's lines).
- One blocking pool per process,
  one thread per runtime worker (`foundation/runtime_plan.rs`),
  is what several threaded-only failures in `landing::` wait on:
  issue 6565's round measured it for the prestage controls with a planted full pool,
  and issues 6567 and 6569 are the two modules it read the same way without measuring.
- The Slint round (`f1957916b0`,
  the owner's "Finish the move (Recommended)",
  `doc/decision/rust-dependency-catalog.md`) left three points open:
  the IDE's two `wrap_keys` fixtures need re-measuring for parley 0.13.0 (issue 6523),
  the IDE needs one container run before it is called green,
  and the three regenerated locks were written by `cargo fetch`,
  not through `cargo-shadow`'s trust policy.
- The driver's pool gate counts no virtual run (`88a9363636`).
  A member a caller closes with `PI_RUNS_POOL_LIMITS` therefore no longer stops a run on a virtual selection.
- `source_guard/standards_review_boundary_tests.rs` now treats the declaration of a module
  whose name begins with a service-only module's name as a declaration,
  not a mention (`3fd13e17fe`);
  the round marked it for the owner to weigh.
- Two rounds split the four byte-identical copies of `src/shadow_identity.rs` in different ways.
  A file that several crates must keep identical belongs to one round.
- `monochromatic-lint.config.jsonc` turns `rust/max-lines` off for `**/tests/**/*.rs`,
  so the linter never reported the test files the briefs named.

### Next

- Merge each round when it has ended and its report is read,
  by the merge bar in "Standing directions".
- The last whole container gate ran on `main` at `6037c4e0f8`,
  started 2026-11-27T20:13:13-05:00:
  12321 of 12321 passed,
  51 skipped,
  exit status 0,
  the first whole gate to run issue 6625's twelve controls
  (log `~/temp/agent/gate/gpc-gate-wrapper-next-alpha-on-6037c4e0f8-20261127T201313-0500.log`,
  scratch).
  The next runs on the tip that holds the wrapper's five rounds,
  before `0.6.1-alpha.52`.
- A new brief names `package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md`
  and,
  for the wrapper,
  its guide.
- A round starts through the driver package,
  one batch list for all the rounds started together:
  `mise run //package/pi-shared/subagent-driver:start-a-batch-in-its-own-unit -- --batch-file=<list> --deadline-seconds=7200 --output-file=<list's name>.out`,
  the list a JSON array of entries with `name`,
  `branch`,
  `prompt`,
  `cpu`,
  `memory`,
  `tasks`,
  `wall`,
  `provider` (`rate-limit-fallback`) and `model` (`writer`),
  the briefs beside it in `~/temp/agent/pi-runs`.
  Never several lists side by side:
  three at once on 2026-11-27 overlapped 27 worktree adds,
  each passed its fifteen-minute bound twice,
  and sixteen of thirty starts failed (issue 6563,
  `doc/planning/pi-round-start-failures-and-what-would-stop-them.md`).
  A start that failed that way leaves its worktree,
  and starting its entry again needs no add.
  The launcher removes the worktree of a run that ended idle with nothing uncommitted,
  and a run's whole report is in `~/temp/agent/<round>-scratch/keep/report.md`,
  which each brief asks for.
  A merge is made by hand in the main checkout,
  one branch after another
  ("How work is run side by side"),
  since the cleanup of 2026-11-27T15:26:37-05:00 deleted the coordinator's scratch merge script:
  the branch's tip equals its tip on origin and `main` is not behind origin;
  `git merge-tree --write-tree --name-only main <branch>` shows no conflict;
  `git merge --no-ff --no-edit <branch>`;
  then the branch's tip is an ancestor of `HEAD` and `HEAD^2` is that tip,
  before the merge is called landed.
  A branch whose package changed on `main` since the branch last took `main` in is merged with `origin/main` in a worktree first,
  and the package's format check,
  compile,
  Clippy and whole test task run there:
  on 2026-11-27 three `mpm-shadow` rounds merged with no textual conflict,
  and the merged tree did not compile.
  After the last of a run of merges,
  origin holds the tip before anything is built on it:
  when the commit service's push of `main` is refused with `GH009`,
  it does not try again,
  and one `git push origin main` clears it
  (`doc/troubleshooting/github-lfs-integrity-rejection.md`).
  A release commit of the linter counts as such a change for the wrapper:
  it writes the new version into `Cargo.toml` and `Cargo.lock` of `package/git-policy/cli` and of its fuzz sidecar,
  so a wrapper branch that took `main` in before it is checked again with it.
  That last check is not ceremony:
  on 2026-11-25T17:24:39-05:00 `git merge` returned 0 and no merge commit landed,
  while another session committed to `main` in the same second,
  and the script of that day printed the other session's commit as the merge.
  The wrapper removes the worktrees that hold a branch once it has landed
  (its README,
  "A merge that lands a branch also removes the worktrees that hold it"),
  and the merged lock branches of 2026-11-27 were gone from origin before the coordinator asked for it,
  by a step that was not read.
  A merged branch is then deleted by hand,
  locally and on origin:
  deleting issue 6620's branch locally left origin's copy
  (2026-11-27T15:19:09-05:00).
  Since issue 6620's merge the package's unit tests give every fixture's `git` the standing Git and no session place,
  so the TypeScript checks at `main`'s tip start nothing on the account's manager:
  at `bc24b2e688` `test:unit` and `lint:types` exited 0 on the pinned tree,
  the journal held no line for a fixture repository,
  and the shared link was as before.
- The wrapper's gate has tasks of its own,
  which the coordinator's gate of 2026-11-27 did not use and the next one does:
  `mise run //package/git-policy/cli:native:gate:main-tip` starts it on `main`'s tip in a capped unit,
  and `native:gate:verdict -- <log path> <deadline seconds>` reads its verdict.
- The `forbidden-strings` workflow has failed or been cancelled on every push to `main`
  since run 44656950451 of 2026-11-20T01:54:13Z,
  which is issue 6632.
  The workflow hands the scanner every changed entry as a file,
  and each of the 139 tracked links under `uses/` folders points at a folder,
  so each prints `read error: Is a directory`;
  run 46079689494 scanned 8149 entries and printed those 139 lines and no other finding.
  Its round is merged (`fc1c84b58e`),
  and run 46083398424 on that commit was green,
  the first since 2026-11-20:
  it left out the 139 folder links,
  said so,
  scanned 8017 entries and found nothing.
  The issue is closed.
  The step reads each changed entry's mode from `git diff --raw`
  and leaves out a gitlink and a link whose target on disk is a folder;
  it became two steps,
  because the linter caps a shell step's length and the old one stood at the cap.
  One thing is narrower than before:
  the old step checked a folder link's own name against the name rules before its read failed,
  and the new one does not.
  The released scanner's `--name-path` takes a link's target text under the link's name,
  so the workflow hands it both with no release of the scanner:
  issue 6633,
  whose round is merged (`af1647ef36`) and whose issue is closed.
  A third step,
  `Scan the entries left out of the file scan`,
  gives the scanner each left-out entry's own name,
  and for a link its stored target text,
  read from Git's object for the link and never through the link.
  Run 46086113778 on that commit was green in all three steps;
  its delta held no link,
  so what shows the step on real links is the round's run of the workflow's own step bodies
  with the released `0.6.2` binary on the delta `82d6a89596..60ecd08eef`:
  139 of 8158 changed entries left out,
  8019 files scanned,
  the 139 scanned by name and target text,
  every step exit 0,
  and a planted rule for `../` gave 139 findings there.
  The record is `doc/troubleshooting/forbidden-strings-scan-step-treats-tracked-folder-links-as-files.md`.
  The workflow cancels a run in progress when a later push arrives
  (`concurrency` with `cancel-in-progress: true`),
  so a run that must be read is waited for before the next push to `main`.
  Its open choices wait in `doc/handover/questions-for-the-owner.md`,
  beside what a positional link means inside the scanner.
  What the record's limits left was issue 6640:
  a changed link to a file had its stored target text scanned by nothing,
  and the whole-tree arm inspected no link and no gitlink.
  Its round is merged (`bed3564da4`),
  with both inspections in the workflow and the scanner unchanged at `0.6.2`:
  one step writes a record for each link and gitlink,
  a delta's from its changed entries and a whole-tree run's from the index,
  and the next hands the scanner each one's name and each link's stored target text in one call.
  The module holds 34 cases,
  each control red under one of seven plants,
  and on this repository's tree the whole-tree arm scans 139 links,
  clean,
  after a planted rule found 139.
  The issue is closed by hand once a push run of the workflow on `main` that holds the merge has ended green and its two new steps' lines are read;
  later pushes cancel a run in flight,
  so that run is looked for when the merges pause.
- `cargo-shadow`'s `src/lib.rs` holds 218 `mod` declarations and no function,
  and is the package's one `rust/max-lines` finding.
  Its round is merged (`60ecd08eef`),
  under the owner's "Keep 300,
  split everything"
  (`doc/decision/rust-files-keep-the-300-line-budget-after-the-vertical-reformat.md`):
  the file holds 201 code lines,
  because 52 test files are declared by the module they test,
  as the crate's newest modules already were,
  and a module that brings tests costs the file one line where it cost three.
  The crate's 731 public paths,
  and the release binary's help and version,
  are byte-identical before and after;
  the same 919 unit test names are there,
  527 under a new module path;
  and the package's task ran 968 and passed 968.
  Two tests that start their own binary with a filter naming their old path matched nothing after the move,
  failed in the first run,
  and name the new path now.
  A round that starts in this package from the alpha.16 read declares a test file the same way.
- The store fixture that tests of several packages build with `createSopsFixture`
  joined a tool's output when the tool exited,
  before its pipe had closed,
  so a store was now and then written empty or cut:
  issue 6410.
  Its round is merged (`25dec7b462`),
  with the branch of 2026-11-18 merged into it first;
  that branch and its review branch are deleted,
  and issue 2031 is closed by it.
  The runner joins at `close`,
  stops the tool's whole process group at a deadline,
  and names the failure.
  Measured by the round at 2026-11-27T18:35:53-05:00,
  with eight children in flight and 80 waits per arm:
  joined at `exit` the whole document arrived in 0 of 80 waits,
  joined at `close` in 80 of 80.
  The default deadline of 2,000 ms then failed four `sops --encrypt` calls falsely
  when the driver's unit suite ran under `CPUQuota=400%`,
  so a second round sized it from a measurement and is merged (`e948095a53`);
  the issue is closed.
  With the deadline lifted by a loader that rewrote the fixture module in memory,
  one run of the driver's suite per cap logged every fixture call:
  at `CPUQuota=400%`,
  36 calls,
  median 104.2 ms,
  slowest 4,597.5 ms;
  at `CPUQuota=1000%`,
  36 calls,
  median 39.3 ms,
  slowest 513.7 ms.
  The default is 15,000 ms,
  the deadline the reader gives the same `sops` child,
  and the suite then passed whole at `CPUQuota=400%` with the shipped default
  (228 `[PASS]` lines and no `[FAIL]`).
  A true hang costs 15 s once where it cost 2 s.
  Why 32 test files run under a 400% cap:
  the repository's own test runner sizes its workers from `availableParallelism()`,
  which reads the affinity mask and not the control group's `cpu.max`;
  the cure and its cost are in `doc/troubleshooting/subagent-driver-suite-fails-under-host-load.md`.
  What both reports left waits in `doc/handover/questions-for-the-owner.md`.
- The driver package's unit test task ran its tests against the built copy and did not build first:
  issue 6635,
  whose round is merged (`8e4a0505bf`) and whose issue is closed.
  The task declares `depends = ["build"]`,
  as `package/zip-writer/core` and other packages already do,
  so mise builds before the first test file starts;
  a test asks mise for the task's dependency list and was red before the declaration.
  Shown before the change,
  with no edit:
  a built copy older than a source file ended the task with status 1,
  two files failing to load while 72 ran against the stale copy and passed;
  with no built copy 75 files failed to load.
  After it both states print 229 `[PASS]` lines and no `[FAIL]`.
  The cost is the build's own 5.053 s per run.
  `depends` and not a `run` list,
  because a `run` entry is handed the words after `--`,
  which name test files here;
  the root `mise.toml` says never to use `depends`,
  and that sentence against the tree is a question for the owner.
  The round counted 96 more packages whose unit tests import a built copy their task does not build,
  and the root aggregate `test:unit` builds nothing:
  issue 6637 holds the list,
  and none was changed.
- Rounds wrote edits into the main checkout on 2026-11-27:
  the round of issue 6632 by its own report,
  and the round of issue 6625 as the coordinating session found
  (the wrapper's `SPEC.md`,
  one of its test files and a planning document).
  Each brief said to read a file "on `main`",
  the round opened the main checkout's copy,
  and its next edit landed on the path it had opened.
  The coordinating session saved the files' differences as one patch in the round's `keep/`,
  put the main checkout's copies back with `git restore --no-enforce-worktree -- <paths>`
  (the way the wrapper's refusal of `git checkout -- <path>` names,
  once a patch is saved),
  and told the round,
  which applied the patch in its worktree.
  Its first note said the restore was done before it was;
  a second note corrected it.
  The gate did not build the stray test:
  its image is built from the gate's own worktree,
  and the probe's name is nowhere in the gate's log.
  A brief names a file to read as the worktree's copy,
  never as "on `main`";
  the brief rules say so now,
  and every live round was sent a note.
  Issue 6636 asked for such an edit to be refused;
  its round is merged (`6037c4e0f8`) and the issue is closed.
  The driver hands every run's Pi a second extension that refuses an `edit` or `write` call naming the main checkout,
  outside `.claude/worktrees/`,
  and answers with the worktree's copy of the path;
  a Pi the driver did not start loads nothing,
  and a run with no worktree is handed the word `off`.
  A run's record keeps the path of every `edit` and `write` call whole.
  The round's tests use a stand-in Pi,
  so the coordinator showed it with a real one after the merge:
  a probe round started 2026-11-27T20:11:05-05:00 had its edit and its write of two ignored files under the main checkout's `.cache/` refused
  (the one file still held its old word and the other did not exist),
  and its write in its own worktree succeeded.
  A shell command that writes into the main checkout is not refused,
  which the package's README and the brief rules say.
  The round's three open points wait in `doc/handover/questions-for-the-owner.md`.
- Worktrees and branches of ended rounds were cleared at 2026-11-27T20:15 -05:00 and after:
  110 local branches and 9 of origin's whose tip is on `main` are deleted,
  and the worktrees whose branch was merged by a merge commit,
  that hold nothing uncommitted and that no process names or stands in,
  are removed.
  Kept,
  each for its reason:
  six whose tip is not on `main`
  (`clone-rung-partial-clone-of-a-dirty-source`,
  `fence-faster-size-reader`,
  `fence-size-read-beside-source-check`,
  `integrate-worktree-removal-held-commit-control-failed-in-two-gates`,
  `pi-prewarm-of-a-new-worktree-acts-only-on-the-entry-it-checked`
  and the review worktree of the driver's queue of runs);
  and six whose branch is merged but that hold uncommitted paths,
  from 4 to 25,522 of them,
  which nobody has read
  (the three `pi-break-wrapper-file-loops-*`,
  `pi-split-cargo-shadow-cli-lean-files-and-crate-root-over-the-line-budget`,
  the second part of `pi-split-mpm-shadow-cli-toolchain-files-over-the-line-budget-second-wave`,
  and `pi-split-wrapper-merge-files-over-the-line-budget-4`).
- A merge of a branch whose worktree still stands makes the wrapper hand that worktree's removal to the commit service,
  and says so in a notice;
  check that the folder,
  the local branch and origin's branch are gone afterwards.
- Owner questions wait in `doc/handover/questions-for-the-owner.md`,
  among them the next step a typed network command names at a new host key
  (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`).

### Scratch expires after a day

The owner set `/etc/tmpfiles.d/temp.conf` on 2026-11-26 to `d /home/user/temp - - - 24h -`.
`systemd-tmpfiles-clean.timer` runs daily (next at 2026-11-27T15:26:37-05:00) and 15 minutes after a boot,
and the home dataset is mounted `noatime`,
so a file under `~/temp` survives only if it changed in the last 24 hours.
The owner,
the same evening:
"Don't promote anything in ~/temp that breaks repo rules or plain broken into the repo.
If you can't move everything before the scheduled cleanup,
too bad.
That is for you to learn."

So the coordinator's private launchers,
merge lanes and gate runner under `~/temp/agent` expire
and are not carried into the repository.
What replaces them is built in the repository by its own rules;
until then a round starts through the driver package's commands
and a merge runs by hand under the merge bar.
801 repository documents cite paths under `~/temp/agent` (3827 citations,
counted 2026-11-26),
and those citations point at nothing once the cleanup reaches them.

### The session wrapper is a build of 2026-11-22

`.cache/monochromatic-git-policy-cli-bin/git` links to
`.cache/wrapper-built-by-cargo-from-main-ee0cf2268b-2026-11-22/monochromatic-git-policy-cli`.
That build predates `4acb1b04b6` (2026-11-24T19:39:21-05:00),
so every `git worktree remove` warns and keeps the removed worktree's service journal
because it "holds rules-repository,
a file this release does not know";
the kept journals are harmless and sit in `~/.local/state/monochromatic-git-policy-cli/service/`.
`0.6.1-alpha.47` (release commit `68af165851`) is unpacked at `.cache/wrapper-release-0.6.1-alpha.47/`,
and its sweep,
run by absolute path,
removes them.
The session link stays on the reviewed build:
the AppArmor rules name that exact path,
and pointing them at a newer build takes the fresh review that waits in `doc/handover/questions-for-the-owner.md`.
`mise run toolchain:install <tool>` in the main checkout re-points that link at the toolchain's launcher,
whichever tool it names:
on 2026-11-27 at 01:47 an install of `cargo:monochromatic-lint` did,
and the coordinator put the link back on the reviewed build about a minute later.
After any toolchain install in the main checkout,
read the link (`ls -l .cache/monochromatic-git-policy-cli-bin/`) and put it back if it moved.

### Threaded-only test failures

A control that passes in the gate and fails in a threaded `cargo test` is that control's defect
(`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`,
"Tests hold as threads of one process").
A `panicked at` line from a thread named `git-policy-worker` in a combined run is not a failure:
it is the wrapper or the service started as a child by the fault-limit controls,
by the same guide's section.

### Memory

Memory is judged by the stall and by `MemAvailable` plus the ZFS ARC above `c_min`
(`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`).
No start or merge check waits on memory or on a spacing between starts:
the coordinator had applied both,
and the owner,
2026-11-26:
"I didn't tell you to apply the 3 minutes apart and memory test rules."
At 18:25 that evening,
with 58 worktree adds copying at once,
the disk stall read `some avg10=76.24` and `MemAvailable` 10290 MiB:
each add through the session wrapper copies about 39,000 ignored entries,
the cost the owner's hold of 2026-11-18 on heavy worktree use names.

## Initiatives

### The ZFS worktree route

Why:
on this machine's ZFS pool `git worktree add` falls back to slower routes,
and the owner made a ZFS route the first priority on 2026-11-17.
The design is decided in `doc/decision/cli-git-zfs-worktree-design.md`
and built step by step from `doc/planning/cli-git-zfs-worktree-implementation-plan.md`.

State,
at 2026-11-26T18:55-05:00,
by the plan's 23 steps.
Steps 1 to 16 are on `main`,
with 12 to 16 wired;
and the removal command path is still cut:
`SPEC.md` gives the switch two conditions,
the cancel command,
which step 17 built and no grammar admits,
and a release whose printed sentences name only commands,
records and states it has;
and the snapshot batch's request budget has no value in release code.
So nothing calls `enter_the_removal` yet and `source_guard`'s
`removal_command_path_writes_nothing_tests.rs` holds the cut.
`doctor` still answers the cancel command's check as unknown
(`health/doctor_zfs_route/later_step_checks.rs` says step 17 builds it).
Nothing merged between that time and 2026-11-27T21:11-05:00 did any part of that wiring,
by a read-only digest of the route against `git log`;
the rounds that do started then ("Rounds running").

- Step 9's unmount,
  staging and reattach is done (`5d93d9cfd7`),
  behind the pin plan step 19 writes,
  and nothing calls it yet.
- Step 10's add route serves an add whose source is the main checkout on a checkout dataset only step 19
  makes (`SPEC.md`,
  "The add that registers a ZFS clone before it fills the slot").
  Its build matches its accepted designs and its guest arms pass.
  The three points that kept a person's add off it are closed:
  6462 as `d169557d37`,
  2C as `98cc2c9420`,
  and the attach path's A5 as `d647717f85`.
- Steps 11 to 16 are each merged with their guest arms,
  and their limits are `SPEC.md`'s "What this release does not yet do".
- Step 17's cancel command merged as `df15f27a7a`,
  and the forced arm's decision half as `64a71a4c9b`;
  and its kernel calls are proven in a guest (`3ebc78c3dc`).
  The forced removal is measured for the holder kinds the guest showed and refuses the rest
  (`doc/decision/cli-git-zfs-worktree-design.md`,
  "What still gates shipping").
- Step 18's `doctor` checks merged unwired (`838b7f5e71` and `d463965dcd`),
  as did the store and `snapdir` delegation check (`2bf72fb230`).
  No release code gathers the facts those checks read;
  the add route calls the delegation check itself
  (`worktree_clone_add_route/add_grants.rs`),
  so that one is reached.
- Step 19's unit texts and its immutable entry's verdict are built ahead (`72a0be07ef`).
  The migration is no program:
  it is `doc/runbook/zfs-checkout-migration-and-boot-setup.md`,
  marked not run,
  not approved and not final,
  and it has not run on this host.
  The owner answered every choice
  (`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`),
  and the chosen design merged as `1ef4afefa8`.
- Step 20 merged unwired (`a7502138a4`);
  writing `sanoid.conf` is the owner's act.
  Step 21's watch comparison merged and nothing calls it
  (`worktree_clone_watch_rebinding/`);
  the skip of the separate ignored-file copy after a clone fill is not built.
- Step 22's guest half is measured (`2ddf2853ad`);
  its storage half is the agent's act,
  logged,
  by the owner's answers of 2026-11-26.
- Step 23's host-kernel qualification run merged as `0c4064831d`
  (`doc/planning/cli-git-zfs-step-23-host-kernel-qualification-run.md`);
  the owner declares the release qualified or not.

Before a person's add takes the route:
wiring steps 17 and 18 with step 22's request budget and lifting the removal path's cut;
step 19's migration on this host;
steps 19,
22 and 23;
and a release carrying them,
installed.
Nothing of the route is installed on this machine.
The checkout migration of plan steps 12,
18 and 19 is the agent's act,
run with sudo when no process has the checkout open,
by the owner's answer of 2026-11-24 ("You run it whenever you feel convenient.",
`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
"Who runs the checkout migration").
The other host changes the plan keeps for the owner
(`zfs allow`,
the units,
the zedlet,
the immutable entry,
`sanoid.conf`,
other datasets' properties,
step 22's storage half,
step 23's image build and any reboot)
are the agent's act too,
logged,
by the owner's answers of 2026-11-26
(`doc/decision/cli-git-zfs-host-setup-measurement-and-image-build-are-the-agents-act.md`);
the owner chooses the moment of the migration and of any reboot,
which the agent prepares.

Next:
the rounds of the list of 2026-11-27T21:11-05:00 ("Rounds running"),
each read and merged as it ends.
They were briefed from a read-only digest of what is left of the route,
scratch,
`~/temp/agent/zfs-route-remaining-pieces-digest-2026-11-27/keep/digest.md`;
what of it must last is written here.

The switch itself is one of those rounds,
`zfs-removal-command-path-runs-the-whole-removal-with-force-still-refused`.
It merges after the management grammar's round.
Its brief takes 65,536 bytes of packed request input from the measurement in
`doc/planning/cli-git-zfs-step-22-guest-half-measurements.md`,
an agent's draft that left each of its values to the owner.
The owner accepted that one on 2026-11-27,
through the question tool
(`doc/decision/cli-git-zfs-removal-batch-request-budget-is-65536-bytes.md`),
so the round's branch merges with it;
the draft's other values stay unset.
`--force` stays refused in that round:
the forced arm's waits are unset,
and the owner's decision is that it does not ship before they are measured.
The rounds on the private propagation step,
on the body of `shutdown-unmount` and on the lost-event merge device each build the piece with its guest arm and add no call from a root mode,
because those calls are step 19's wiring,
where the step's "Depends on" still holds
(`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`).

Held,
each with what it waits for.
The orders are the coordinator's own unless a record is named:

- printing the ZFS report from `doctor`:
  the three `doctor` rounds,
  whose gatherers and checks it composes;
- the origin cleanup look's calls from `git worktree add`,
  `list`,
  `prune` and `repair`:
  the removal's switch,
  because a blocked-cleanup record exists only after a removal,
  and to keep that round's files apart;
- offering `--force`:
  the switch,
  the two forced-arm rounds,
  and the waits' values,
  by the owner's decision on step 17;
- step 21's wiring,
  the watch comparison after a fill and the skip of the separate ignored-file copy:
  step 19,
  which step 21 depends on;
- the values of step 22 in code,
  the mount and unmount deadlines and the forced waits:
  the owner's acceptance,
  by the step 22 record's own sentence;
- the lost-event numbers and the nudge's switch:
  both lost-event rounds and a guest run of the design's measurements,
  by the lost-event decision's choice 17;
- Sanoid 2.3.0's coverage arm in a guest:
  a way to put Sanoid 2.3.0 into a guest,
  which plan step 23's template does not hold.

The host's own acts wait for more than the switch.
The checkout migration waits for removal to exist
(`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
"After removal exists (Recommended)"),
which the coordinator reads as the switch and the grammar released and installed;
the units,
the immutable entry,
`snapdir=disabled` and the zedlet wait for step 19's wiring and its guest proof;
`sanoid.conf` waits for step 20's wiring and the Sanoid arm;
and step 22's storage half runs after the migration.

Outside steps 17 to 23,
by `SPEC.md`'s list under "What this release does not yet do" for the add route,
and needed by none of the rounds:
the `.gitattributes` fence's own write;
the check after a restore;
finishing an add with Git's own checkout when its proof or its restore fails;
and the warning above the measured count of changed paths.
That list's last two entries are run together on one line of `SPEC.md`,
``(rule `LQM`).- A record that says``,
which the round on the records' status lines corrects.

Lines of other records that `main` has passed,
found by the digest and confirmed by reading each,
which the round on the records' status lines corrects ("Rounds running"):
`doc/decision/cli-git-zfs-worktree-design.md` says the forced arm's kernel calls are not built,
and `3ebc78c3dc` built and proved them;
`doc/planning/cli-git-zfs-step-22-guest-half-measurements.md` and
`doc/runbook/zfs-sanoid-coverage-and-the-space-guard.md` call the storage half and the Sanoid page the owner's act,
which the answers of 2026-11-26 made the agent's;
`doc/planning/cli-git-zfs-step-23-host-kernel-qualification-run.md`,
"What is left",
lists the gate arm's pin,
which the digest says `bd180d7de9` settled and the coordinator has not checked;
and the plan's opening status is that of 2026-11-19.

Rounds keep their files under `~/temp/agent/<round name>-<date>/`,
which expires a day after its last change ("Scratch expires after a day");
a round's durable findings land in its branch's records.

#### Accepted designs that wait for their build steps

- Private propagation after a replacement mount,
  for step 19:
  `doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`.
- The triggers a lost last event waits for:
  `doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`.
  The design and its decision give parts to plan steps 10,
  14,
  18,
  19 and 22
  (`doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`,
  choice 17),
  where the plan's own text names steps 7 and 22;
  none of the parts given to steps 10 and 14 was built with those steps.
  Two rounds build the wrapper's side and the handler's side ("Rounds running").
  That design's own worktree is gone,
  by `git worktree list` at 2026-11-27T21:25-05:00.
- The forced removal's two kept refusals,
  of a locked holder and of a taskless namespace:
  `doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`.
  The owner answered "Keep the refusal (Recommended)" to each on 2026-11-26T00:05-05:00
  (`doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`,
  "The two refusals the built arm keeps"),
  so each waits for a guest arm and not for the owner.
  Two rounds measure them ("Rounds running"):
  the stop of a locked holder's own user unit,
  and the walk that finds a taskless namespace on a kernel that has `listns`.

Each one's acceptance is the section "The design is accepted" of its decision record.

#### Later steps' pieces built ahead, unwired

Pieces that need only work on `main` are merged as code no person's command reaches,
so their steps wait only for their wiring.
What is still built ahead:
step 17's cancel command (`df15f27a7a`) and forced arm (`64a71a4c9b`,
`3ebc78c3dc`),
the settling command,
the removal's finish pass,
the origin cleanup look and the snapshot batch's callers,
each reached by no command;
step 18's `doctor` lines (`838b7f5e71`,
`d463965dcd`,
`2bf72fb230`);
step 19's unit texts and immutable entry (`72a0be07ef`);
step 20's `sanoid.conf` writing (`a7502138a4`);
and step 21's watch comparison.
Step 16's deferred-destroy checks,
step 15's deferred-origin check and step 13's managed lock are wired,
though no command reaches them while the cut stands.
Step 12's move refusal and unmounted report are wired and in use
(`7cf926dc56`;
`3ea4fa16c3` is the merge that built them ahead).

### The crate split, and the layout rules that land before it

Why:
the ZFS route's rounds build this crate alone,
and one build of it peaked around 6.6 GiB,
of which half its test code left out measured 4.7 GiB
(`package/git-policy/cli/doc/development-profile-measurements.md`,
"Memory per build,
measured 2026-11-25").
The owner chose how to cut it on 2026-11-25 at about 22:05:
"Window after live rounds (Recommended)".

State,
at 2026-11-26T18:55-05:00:
the split's round 1 merged as `3418cdaebc`,
round 2's 19 files that were over the budget before the reformat merged in five branches,
and the owner's "Keep 300,
split everything" put every Rust file over 300 lines in a round
("Rounds that ended,
and what each left").

The layout rules `AGENTS.md` sets for every crate are switched on for this one,
with every finding of the two rules on a list that only shrinks,
by the owner's answer "Everything now,
untangle over time."
(`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`,
"What the list holds").
`toolchain.lock.json` locks `monochromatic-lint` `0.5.2-alpha.17` (`d413a71714`).

Every crate outside this one is formatted by the root `rustfmt.toml`,
which the owner set on 2026-11-25.

### The wrapper's test gate tells the truth

Why:
one gate on `main` per batch of merges is the check the batch rest on afterwards,
and a control that fails once in a gate for no fault of the product costs a round each time.

State:
the gate's known causes are lettered in
`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
with what each fixes and the runs each appeared in.
Cause H,
the two standing failures,
is diagnosed and fixed in its controls
(issues 6466 and 6468).
AppArmor enforcement costs `main` no test now,
and every rule the work needed is in
`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
with the owner's "Always widen apparmor."
A denial the work meets is answered by a rule in the profile's `local/` file,
never by moving the work aside and never by naming the profile.

Next:
run the next whole gate on `main`'s tip,
read its window's denials with `list-denials-in-a-window.ts`,
and start a fix round for any failure outside the recorded baseline.

#### The Markdown the line-break sweep left out

`monochromatic-lint`'s `markdown/semantic-line-breaks` still reports in the files the whitespace-only
sweep of 2026-11-23 left out,
because the branches that were changing them could not take the diff:
`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md` alone reports 683 findings on
`main`,
and everything under `package/git-policy/cli` was the same.
Both are fixed (`beb61ec46b`):
4,062 findings in 480 files,
with `bin/` and `e2e/` left as they were while the round of issue 6612 worked there;
that round fixed the two findings one file kept.
The installed linter applies that fix itself with `--fix`,
which changes whitespace only,
so the scratch script of 2026-11-23 is not needed.
On 2026-11-27 it took 76 findings out of `package/mpm-shadow/cli` (`ee8871b190`)
and 11 out of `package/cargo-shadow/cli` (`f6b4dc0947`),
every one in a Rust doc comment,
and 778 out of five Markdown files of `package/pi-shared/subagent-driver` (`e6026dbd46`),
whose six Markdown files render to the same text as before,
in a window when no round worked in those packages:
in a worktree per package,
each changed file was equal to its old text once whitespace and comment markers were taken off,
`cargo fmt --check`,
`cargo check` and Clippy exited 0,
and the whole test task passed,
910 of 910 and 968 of 968.
The wrapper's fix was checked the same way,
and three ways more:
of 6,789 changed Rust lines none began with anything but a doc marker;
the linter reported no layout finding and no processing failure,
as before;
and the `source_guard` module (225 of 225) and the two modules whose tests read `SPEC.md` passed.
The same count,
of changed lines that are no doc comment,
is zero for the two packages' merges too.
`bin/` and `e2e/` need no sweep:
measured on `main` after issue 6612's merge,
the package holds no `markdown/semantic-line-breaks` finding.
The driver package's build,
`lint` and `lint:types` exited 0 in its worktree,
and its unit suite printed 228 `[PASS]` lines beside one failure that is not the fix's:
the store fixture of issue 6410,
which the Next list names.
`cargo-shadow`'s `src/lib.rs` is over the line budget,
305 code lines against 300,
and the Next list names its round.

#### The release passes

`0.6.1-alpha.47` is released from `68af165851`.
Its two Sonnet passes are read and their fixes are on `main`,
except the owner's two follow-up refusals,
which are in their merge check
(`doc/decision/cli-git-alpha-47-release-pass-two-removal-answers.md`).
The next alpha follows the next batch's gate.

### The desktop stays usable under agent runs

Why:
the desktop froze for minutes on 2026-11-22 under this session's runs.

State:
two kernel settings,
`vm.watermark_scale_factor` 125 and `lru_gen` `min_ttl_ms` 1000,
are in force and kept across reboots by `/etc/sysctl.d/90-vm-watermark-scale-factor.conf` and
`/etc/tmpfiles.d/lru-gen-min-ttl.conf`,
by the owner's "Install both files (Recommended)" of 2026-11-24
(`doc/decision/memory-settings-persist-and-lfs-url-mask-shows-short-segments.md`).
The swapping that followed was `kswapd` and not the ARC,
and the fix that held was Madvise defrag with `compaction_proactiveness` 20;
the measurement,
its limit,
and the zram writeback to the old NVMe are in
`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`.
`systemd-oomd` reads the cgroups again since AppArmor's rules allow it.

Next:
after the next reboot,
read both values once to confirm the files applied them.

## Held work

The wrapper and driver backlog waits under the owner's hold of 2026-11-18,
and other packages' work waits for the owner to name it.
It lives on GitHub,
by the owner's standing instruction of 2026-11-23
(`doc/agent/triage-labels.md`,
"Blocked work"):
an issue that waits on something carries a `blockedBy:<specific>` label,
most of them `blockedBy:zfs-route`,
some `blockedBy:owner-decision`,
some `blockedBy:wrapper-focus`,
and a few another issue's number.

What no public tracker holds,
or that is no issue:

- When the held queue is drained,
  the coordinating session re-reviews the whole wrapper effort once.
- Nothing of the WireGuard tools (`package/wg`) is deployed.
  Deploying is the owner's act,
  it takes a live connection down and up,
  and its steps were written for the owner on 2026-11-11 in a scratch digest,
  not in the repository.
- A scan on 2026-11-14 found secret values in the saved sessions of six Pi runs.
  Those sessions are never resumed or opened,
  and rotating the values is the owner's.

## Things a new session would not find elsewhere

- Do not run `pnpm install` in a new worktree (issue 6339).
- The coordinating session makes a host change the owner has authorised itself,
  with the passwordless `sudo`,
  and does not hand it back.
  The owner,
  2026-11-23,
  after enabling `audit-rules.service` when the session had left that to the owner:
  "I didn't need to run it myself but I just ran it as a courtesy."
  The rule against acting on the system service manager in the skill `pi-as-subagent` binds the agents a
  session briefs.
  The checkout migration is the agent's act by the owner's answer of 2026-11-24;
  the other ZFS host changes are the agent's act,
  logged (owner,
  2026-11-26),
  and the owner picks the moment of the migration and of any reboot.
- No round runs `mise run toolchain:install`,
  not even for one tool in its own worktree:
  it waits for issue 6316,
  in the owner's words
  "Because we need to make mpm-shadow prefer system package managers on non-atomic distros first."
- An answer from the owner changes the bar for reviewers that are already running.
  Send it to each of them in the turn it is recorded.
- A question to the owner blocks the coordinator until it is answered.
  Start every round that can run before asking one.
- A round's report is not evidence of its edits.
  Read the diff before a design goes to the owner or a branch is merged.
- A Pi round can hang in one command of its own without the driver noticing.
  The coordinator's watcher for that was a script in scratch,
  and it expires with it.
- The repository's tool links live in the main checkout's `.cache/toolchain-bin`,
  so `monochromatic-lint` and `file-enforcer` are there and not in a fresh worktree.
- Orphaned cargo build folders under `~/.cargo/build`,
  left by integration passes,
  are not removed without the owner.
- The checkout migration is the agent's act (owner,
  2026-11-24);
  the other ZFS host changes are the agent's act,
  logged (owner,
  2026-11-26),
  and the owner picks the moment of the migration and of any reboot.
  Old drives stay read-only and unwiped until the ZFS route is released.
