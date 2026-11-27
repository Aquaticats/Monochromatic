# BTF reader for the ZFS layout checker: technology vet (2026-11-20)

Not final and not approved.
This report recommends no reader and adopts nothing (rule `SGO`).
Only the project declares the work complete.

Status:
complete as a report,
at lifecycle phase 7,
Scored.
The skill's terminal outcome is
[sensitivity unresolved](#terminal-result-sensitivity-unresolved),
so the report gives conditional rankings and recommends none,
for as long as `pahole` is a finalist.
Whether it is one is question 4:
its upstream suite ran here with eleven recorded skips,
its own CI path did not run,
and if the project finds that omission fatal the outcome changes to a recommendation of `btf-rs`,
which the skill then allows,
because all 51 one-input tests between the two remaining libraries keep the order.
Three finalists are ranked meanwhile:
`btf-rs` 2.0.1 and `ebeepf` 0.1.0,
two libraries,
and `pahole` 1.32,
a program whose upstream suite ran here.
`bpftool` 7.8.0 failed the skill's relevant-execution rule:
its functional tests need root,
a BPF-capable kernel and a VM harness,
as the report reads them,
and none of that runs in this container.

What is settled,
and what the second revision's text got wrong:
`btf-rs` is strictly first in every defined test at the frozen weights,
all 66 of them over the three finalists and all 81 in the recorded four-candidate arm,
with a smallest lead of 2 points over the runner-up and 5 over `pahole`.
The maintenance weight reorders places 2 and 3 only:
at weight 4 `ebeepf` ties `pahole` at 30.5,
and at weight 5 `pahole` is second at 34.5 and `ebeepf` third at 32.5.
That weight is a preference the project has never stated,
and it decides nothing about the first place inside the skill's range:
`btf-rs` stays first at every weight from 1 to 5,
one criterion at a time or all eight jointly,
and keeps at least 5 points over `pahole` at every one of those joint vectors.
The lead is bounded and not absolute:
`btf-rs` rates one point below `pahole` on C5,
so a maintenance weight of 10 with the rest at 1 ties at 54.5 each,
and a weight of 11 puts `pahole` first at 58.5 against 57.5.
The choices the project has to make are in
[Open questions for the project](#open-questions-for-the-project),
each with what it changes.
The project then adopted `btf-rs` 2.0.1 on 2026-11-20T20:18:04-05:00,
and the adoption is recorded in the decision record's section
"The reader adopted:
`btf-rs` 2.0.1"
([`doc/decision/cli-git-zfs-runtime-layout-checker.md`][record]);
nothing in this report's outcome changes with that adoption.

Checked by the third revision:
the three-finalist and four-candidate matrices recomputed from the published ratings,
with the script and its output kept;
both libraries' crate archives downloaded again and measured,
with the output kept;
the programs' provenance read from the distribution's package recipes at the release and tag that
built them,
and from the kernel tree at `v7.3.4` that `bpf 7.3.4-1` builds from;
`pahole`'s upstream suite run in a capped container with no home mount,
built at the packaged tag with the packaged patches,
76 cases with 65 passing,
11 skipped and none failing,
with the container's log kept;
the two `bpftool` statements the gate re-check named as wrong,
corrected against the source;
and the report's own lock record reconciled against the commits its hashes name.
Not checked:
the libraries' suite runs,
which the third revision did not repeat,
the `bpftool` functional tests,
which the report reads as needing root,
a BPF-capable kernel and a VM harness,
and an `aarch64` run,
which no machine here can do.

The fourth revision checked nothing by running it.
It read the gate final check whole,
kept at
`~/temp/agent/gate-final-check-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/review.md` (scratch,
deleted on 2026-11-27;
what this record uses from it is stated here),
with the source excerpts,
the count script and the offline rerun kept beside it,
and corrected the report's words where they ran past that evidence:
the workflow inventory and the coverage sentence,
the libbpf pin,
the C2 label's test counts,
the suite's own count,
the first place's bound,
the suite run's network,
the questions' separability,
and the ten lows it earned.
It ran no candidate's code,
took no measurement,
re-rated nothing,
and changed no score:
every figure it prints is the kept evidence's own or arithmetic over the ratings this report already
prints.
Its edits are unapproved and non-final,
and no independent read has reviewed them.

## Report metadata

- Subject:
  BTF reader for the ZFS layout checker.
- Scope:
  which BTF reader the start-time ZFS layout check uses,
  a library the binding crate links or a program the gate starts,
  for question Q1 of [`doc/decision/cli-git-zfs-runtime-layout-checker.md`][record].
  The check compares the loaded ZFS module's declared layout
  with the binding's compiled expectations before the first `/dev/zfs` request,
  and a refusal disables raw requests.
- Started:
  2026-11-20T18:07:38-05:00,
  the clock reading taken as this session's work began.
  Last updated:
  2026-11-20T23:26:39-05:00,
  the write time of the sixth revision.
- Revisions:
  six.
  The first wrote the report;
  the second answered an independent gate review by a different vendor,
  Claude Sonnet 5.5,
  kept at
  `~/temp/agent/gate-review-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/review.md` (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here);
  the third answered that reviewer's gate re-check of the second revision,
  kept at
  `~/temp/agent/gate-recheck-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/review.md` (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here);
  the fourth answered that reviewer's final gate check of the third,
  kept at
  `~/temp/agent/gate-final-check-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/review.md` (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here);
  and the fifth,
  this round,
  answers that reviewer's diff-only check of the fourth,
  kept at
  `~/temp/agent/gate-diff-check-of-the-btf-reader-vet-report-wording-on-sonnet-2026-11-20/keep/review.md` (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
  What the second changed is listed in section "Changes in the second revision";
  what the third changed is listed in section "Changes in the third revision";
  what the fourth changed is listed in section "Changes in the fourth revision";
  what the fifth changed is listed in section "Changes in the fifth revision".
  The sixth revision answers the integration pass on ZFS plan step 6 with the `btf-rs` reader,
  kept at
  `~/temp/agent/integration-pass-zfs-step-6-with-the-btf-rs-reader-2026-11-20/keep/review.md` (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here),
  which measured one sentence of this report false against the binding's own pre-scan;
  what it changed is listed in section "Changes in the sixth revision".
  The gate re-check found the matrix and the arithmetic reproducing and two high findings in what the
  report tells the project:
  a maintenance-first priority described as making a program "the rational pick" though the library
  stays first by at least 5 in every test,
  and the two programs kept as finalists through two departures from the skill that decided the
  result.
  The gate final check found the matrix,
  the first place and the suite result reproducing,
  and the report's words outrunning them:
  an uninventoried upstream CI path and a coverage sentence the suite's own scripts contradict,
  a provenance fact wrong in eight places,
  a matrix that used a C2 label other than the one printed,
  and a suite count stated twice differently.
- Governing skill:
  [`.agents/skills/choosing-technology/SKILL.md`][skill] at commit
  `45d42d9b23d2f2752f9c512683b45905e13afe99`,
  dated 2026-10-29T20:16:36-04:00,
  SHA-256 `f4cfa6bc5d1d71f4b136f92ff9fbdab68575c920bd334f824d253336c8782e1b`.
  The `.claude/skills/choosing-technology/SKILL.md` copy this session read is byte-identical:
  `diff` printed nothing and both hash to that value.
  That is the same revision the OpenZFS vet recorded.
- Compatibility fingerprint:
  `55c2be0183adaa6fbad912f852fb045a915c1593b344aa10095aa71bdec4677f`.
  Input:
  section "Fingerprint input".
  Script:
  `~/temp/agent/zfs-layout-checker-reader-vet-report-2026-11-20/keep/compute-the-vet-report-fingerprint.ts` (scratch,
  deleted on 2026-11-27;
  not kept),
  run twice in the first revision,
  the second at 2026-11-20T18:16:26-05:00,
  both exit 0 and both printing that value;
  the rerun in the second revision is
  `~/temp/agent/zfs-layout-checker-reader-vet-report-gate-review-findings-fixed-2026-11-20/scratch/keep/compute-the-vet-report-fingerprint.ts` (scratch,
  deleted on 2026-11-27;
  not kept),
  a byte-for-byte copy of it,
  run against the second revision's printed input and printing the same value,
  and the third revision recomputed it from the same printed input with the same result.
  Nothing the third revision changed touches a fingerprint field,
  and the fourth did not recompute it,
  because nothing it changed touches one either:
  it corrected wording,
  an inventory,
  a label's counts and a pin,
  and every value the fingerprint carries is the text the third revision printed.
  The value is unchanged because the subject,
  the scope,
  the hard constraints,
  the deployment,
  the trust boundary,
  the base category and the overlays are unchanged:
  the refreeze below changed weights and ratings,
  which the fingerprint does not carry.
- Lock:
  `~/temp/agent/technology-vet-locks/` + the SHA-256 of the absolute report path + `.lock`.
  (scratch,
  deleted on 2026-11-27;
  not kept).
  The first revision created the report with create-new semantics,
  so its first lock record holds `preEditSha256: null`.
  The second revision took the same lock in update mode:
  it wrote a sibling temporary file,
  re-read the report's SHA-256 under the lock,
  aborted if it had changed,
  and renamed the temporary file into place.
  Every record of the first two revisions,
  seven from the first and five from the second,
  is kept together in
  `~/temp/agent/zfs-layout-checker-reader-vet-report-gate-review-findings-fixed-2026-11-20/scratch/keep/lock-record.json` (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
  That file began as a copy of the first revision's file,
  which still sits at
  `~/temp/agent/zfs-layout-checker-reader-vet-report-2026-11-20/keep/lock-record.json` (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here),
  and the second revision's writer appended to it as the first one's did.
  The third revision appended its records,
  record 13 onwards,
  to a copy of that file at
  `~/temp/agent/zfs-layout-checker-reader-vet-report-recheck-findings-fixed-2026-11-20/scratch/keep/lock-record.json` (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
  The fourth appended records 23 to 27 to a copy at
  `~/temp/agent/zfs-layout-checker-reader-vet-report-final-check-wording-fixed-2026-11-20/scratch/keep/lock-record.json` (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here),
  and record 23's `preEditSha256` `dce1ebeb4e07...` is record 22's `postEditSha256`,
  so the chain crosses from the third revision into this one with no break.
  Record 13's `preEditSha256` `965b95ecb0...` is record 12's `postEditSha256`,
  so the chain crosses from the second revision into the third,
  each write verifying the report's hash under the lock before it renames.
  Record 14's `preEditSha256` `a730704b...` is the hash of the third revision's own draft after it had
  edited the working copy outside the lock,
  which is the same defect the second revision has,
  now on one edit of the third.
  Every record after 14 records the previous record's `postEditSha256` as its `preEditSha256`,
  and the last record's `postEditSha256` is the report the third revision committed,
  so the final write is locked and the chain holds from record 14 onward.
  The fourth revision wrote the report five times,
  all under the lock:
  it staged its text in its own scratch folder,
  took the lock,
  re-read the report's SHA-256,
  wrote a sibling temporary file and renamed it into place each time.
  Record 23 installed the corrected text with a clock reading this session's own arithmetic got wrong,
  `2026-11-21T01:27:04-05:00`,
  a UTC date carrying a local offset;
  record 24 replaced that reading with the write's own local time;
  record 25 corrected the paragraph that reports these writes and mended five sentences it had left
  loose;
  record 26 recorded the digest of the image the gate final check's rerun built;
  and record 27 corrected one sentence about `bpftool`'s two program tests,
  which rests on the report's reading and not on a read of the files.
  The fifth revision staged its text in its own scratch folder and wrote under the lock four times:
  it re-read the report's SHA-256,
  wrote a sibling temporary file and renamed it into place,
  as the fourth revision did.
  Record 28 installed the wording fixes with a clock reading this round's arithmetic got wrong,
  `2026-11-21T01:54:29-05:00`,
  a UTC date carrying a local offset;
  record 29 replaced that reading with the write's own local time;
  record 30 carried the pair count fix and this paragraph's account of the writes;
  and record 31 named question 8's section as the record prints it and corrected one sentence's
  attribution.
  The chain runs unbroken from record 22 through record 31,
  and record 31's `postEditSha256` is the report the fifth revision committed,
  from a copy at
  `~/temp/agent/zfs-layout-checker-reader-vet-report-diff-check-wording-fixed-2026-11-20/scratch/keep/lock-record.json` (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
  The sixth revision wrote it three times,
  from the worktree `.claude/worktrees/zfs-step-6-integration-findings`,
  where the file it edits lives,
  so its lock key is the SHA-256 of that worktree's report path:
  record 32's `preEditSha256` `668ae41a...` is record 31's `postEditSha256`,
  so the chain crosses from the fifth revision into this one with no break.
  Record 32 installed the corrected text with a clock reading this session's own arithmetic got
  wrong,
  `2026-11-21T04:26:05-05:00`,
  a UTC date carrying a local offset;
  record 33 replaced that reading with the write's own local time;
  and record 34 carries the paragraph that reports these writes.
  Record 34's `postEditSha256` is the report this round commits,
  from a copy at
  `~/temp/agent/zfs-step-6-integration-findings-fixed-2026-11-20/scratch/keep/lock-record.json` (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).

  The chain does not hold inside the second revision,
  and the report says so rather than leaving it to a reader:
  records 8 to 12 each carry a `preEditSha256` that is not the previous record's `postEditSha256`,
  so the report was rewritten outside the lock between those writes.
  Where the content survives,
  the break is identified:
  record 12's `preEditSha256` `2135c1de...` equals the byte hash of the kept
  `staged-report.md`,
  so a staged copy was written over the report between the locked writes that records 11 and 12
  name.
  The kept `report-before-lint-fix.md` hashes `0bb6e8bc...`,
  which is none of the recorded values.
  What can still be checked is checked here:
  record 7's `postEditSha256` `9600ac9ba4...` is the report at `6ed02591cf`,
  record 8's `37efb1caae...` is the report at `e83c0de1c3`,
  record 11's `2c64862183...` is the report at `c00b8adef2`,
  and record 12's `965b95ecb0...` is the report at `d870b1786e` and at `main...HEAD` before this
  revision.
  The repair is forward,
  not retrospective:
  every write that produced the committed report read the report's hash under the lock,
  verified it,
  and recorded it,
  and the record names the one edit the third revision made outside the lock.
  Nothing claims the second revision's out-of-lock edits were locked.
  One departure is recorded rather than hidden:
  the skill places a report directly in the main worktree,
  while this one sits in the branch worktree where the work happens,
  so its lock key is the SHA-256 of that path and a copy elsewhere would take another key.
- Active audit owner:
  the fifth revision's session,
  a Pi session in the worktree `.claude/worktrees/zfs-layout-checker-reader-validation`,
  on branch `docs/planning-cli-git-zfs-layout-checker-reader-validation`.
  No second agent wrote any line of this revision.
  The discovery enumeration's first twelve queries were run by earlier sessions in the same worktree;
  the project's answer of 2026-11-20 delegated the remaining ones to the first revision's writer,
  which ran them,
  and the first revision was that writer's synthesis,
  not a re-run.
  The finalist validation was run by an earlier session still.
  The gate review above was an independent read of the first revision and wrote nothing here.
- Prior compatible report:
  none.
  No file under `doc/audit/` carries this subject slug,
  and the one report holding ZFS layout evidence,
  [`tech-rust-crate-for-openzfs-vet-2026-11-18.md`](tech-rust-crate-for-openzfs-vet-2026-11-18.md),
  stores fingerprint `407e8ac931da8102b402d2356e322daaef1a0bdf224b8caf612c2c64d1d3ae2e` for a different subject.
  So the skill's rules for a new report apply,
  and no duplicate path exists.
- Report threshold:
  crossed,
  on three of the skill's conditions at once.
  External discovery and screening saturated with four serious alternatives;
  every finalist received execution evidence from this host's own BTF pair;
  and a required source class was blocked for a time
  before alternate enumeration finished it (section "Query schedule").

## Changes in the sixth revision

This round answers the integration pass on ZFS plan step 6 with the `btf-rs` reader,
run by an agent of another vendor after reading the branch that wires the gate,
and it changes one claim and the record of its own write:
nothing was measured here,
nothing re-rated,
and no candidate's code ran.
The pass measured the reader on fixtures the crate's own writer builds in a scratch copy
(finding M1 of that review).

- The `btf-rs` cons list said a future kernel's widened fields are refused by the binding's pre-scan.
  They are not:
  the pre-scan reads the header and the section list and no type entry,
  and `btf-rs` 2.0.1 reads a type entry's kind in five bits and its member count in sixteen,
  so a widened kind is read as the kind its low five bits name,
  and only a widened member count that desynchronises the reader's own parse is refused.
  The bullet now says what was measured,
  that no file on this host sets a widened field,
  and that the adoption record names the lag as a revisit trigger.
  Whether the binding should check an entry before the reader sees it is the project's question,
  and this report neither answers it nor waits on it.
- The lock paragraph says what record 31's hash names,
  names this revision's records 32 to 34,
  and gives the worktree path its lock key hashes.

## Changes in the fifth revision

This round answers the gate diff check named under "Revisions" above,
written by the same reviewer from another vendor after reading the fourth revision whole.
It found every count reproducing and four sentences gating the merge,
and this round fixes words only:
nothing was measured,
nothing re-rated,
and no candidate's code ran.

- The workflow inventory stops saying `clang` built `pahole`:
  the suite run's CMake log names one compiler,
  `gcc` 16.2.1,
  and `build.yml`'s `gcc-12` and `clang` builds stand recorded as uncovered
  ([The upstream CI of `pahole` at `v1.32`](#the-upstream-ci-of-pahole-at-v132)).
- The `ebeepf` against `pahole` pair prints 57 tests with 55 keep in both places,
  which is the count the printed C2 label gives
  ([The finalists that remain](#the-finalists-that-remain)).
- The list of which question is conditional on which is renumbered from question 5 on,
  question 5 joins the group that bears only while the program arm is live,
  and the change notes name questions 1 and 5 for the interaction and question 8's wording
  ([Open questions for the project](#open-questions-for-the-project)).
- The first place's reason is the report's own pair of them:
  the library shape against the programs,
  and maintenance and auditability against `ebeepf`.
- The questions that bear on an order are named by number wherever the report lists them,
  and the count words are gone.
- The inventory promises the two fields its entries hold,
  and the sentence about the other ten skips says that four of them are the CI path's and stay open.
- Question 8 quotes the decision record's Q1 section in the record's own words.
- The lock paragraph counts the fourth revision's records as 23 to 27,
  and the fifth revision's records 28 to 31 are this round's four writes,
  the second replacing the first's wrong clock reading.
- The sentences about `bpftool`'s functional tests say the requirement is the report's reading,
  and the sentences about `pahole`'s standing say they hold while question 4 is answered that way.
- The sentence about what running the CI path would close is written as the expectation it is.
- The status block records that the project adopted `btf-rs` 2.0.1 on 2026-11-20T20:18:04-05:00,
  and nothing in the outcome changes with it.

## Changes in the fourth revision

This revision answers the gate final check named under "Revisions" above,
written by the same reviewer from another vendor after reading the third revision whole.
It found the matrix,
the first place and the suite result reproducing,
and the report's words outrunning that evidence.
Each finding is listed with what became of it and the section that carries the change.
Nothing was measured,
nothing re-rated,
no crate was added to a manifest and no candidate's code ran.

- H1,
  `pahole`'s passage of the skill's relevant-execution rule was asserted rather than shown:
  fixed in what the evidence allows.
  [The upstream CI of `pahole` at `v1.32`](#the-upstream-ci-of-pahole-at-v132)
  inventories its five workflows and records each as run or not run,
  the coverage sentence now says what the suite's scripts do
  (case 12 encodes with a base;
  case 18 is the only split read through the printer and it skipped),
  the CI-equivalent path and case 18 are recorded as not executed with their reasons,
  and the decision the outcome turns on is question 4.
- H2,
  the embedded libbpf was called unpinned in eight places while the tag pins it:
  fixed everywhere it stood.
  The recipe's `git submodule update` checks out the recorded submodule commit,
  `dd92bef7f6c7`,
  libbpf 1.8.0,
  the kept suite run built libbpf's default branch instead and is now said to,
  and the gate final check's rerun at the recorded commit matches it case by case.
- M1,
  the matrix did not use the C2 label the report prints:
  fixed by keeping the printed label and letting the counts follow,
  66 tests over the three finalists and 81 over the four candidates,
  with thirteen of the fifteen flags on the program pair.
- M2,
  the suite count was given twice differently:
  fixed to the log's one count,
  65 `Ok` and 11 `Skip`,
  wherever the report states it.
- M3,
  "no priority moves the first place" was unbounded and its reason was false:
  fixed.
  The stability claim is bounded to weights 1 to 5,
  one criterion at a time or all eight jointly,
  a maintenance weight of 10 ties and 11 puts `pahole` first,
  and the sentence saying no weight can is gone.
- M4,
  the isolation claims for the suite run were stronger than the run:
  fixed.
  The run kept `--network host` throughout and bounded the fetch with the proxy variables,
  the image digest is recorded,
  and the gate final check's two-phase `--network none` rerun at the pinned libbpf gives the same
  result.
- M5,
  the questions were less separable than the report said and one sentence contradicted another:
  fixed.
  "Every weight the project has stated" is gone,
  questions 1 and 5 are stated to interact,
  each question says whether it bears only while the program arm is live,
  and question 4 is the one H1 adds.
- L1 to L10:
  L1 in the tools' C2,
  C3 and C5 effects,
  L2 in case 54's reason,
  L3 in cases 45 and 56,
  L4 in the weight-4 and weight-5 denominators,
  L5 in the terminal result and question 8's wording,
  L6 in the kept suite logs' description,
  L7 in the sources list,
  L8 in the design's two stale statements,
  L9 in `bpftool`'s inventory
  and L10 in `pahole`'s C8 line count are fixed where the sentence stands,
  and each is named in its section.

## Changes in the third revision

What follows is the third revision's own account,
in its own voice;
its "this revision" means the third.

This revision answers the gate re-check named under "Revisions" above,
written by the same reviewer from another vendor after reading the second revision whole.
It found the matrix and the arithmetic reproducing and the report's project-facing statement wrong.
Each finding is listed with what became of it and the section that carries the change.
Nothing was adopted and no crate was added to a manifest.

- H1,
  the open questions and the conditional rankings said a maintenance-first priority makes a program
  "the rational pick",
  while at the frozen weights `btf-rs` is strictly first in every defined test:
  fixed.
  The status block,
  [Open questions for the project](#open-questions-for-the-project),
  [Conditional rankings](#conditional-rankings),
  [Terminal result](#terminal-result-sensitivity-unresolved)
  and the design's Q1 now say what each question changes,
  the maintenance weight is stated as reordering places 2 and 3 for the finalists,
  and places 2 to 4 in the arm that keeps both programs.
- H2,
  the two programs stayed finalists through a provenance gate written as open and a suite gate the
  skill fails one of them on:
  fixed,
  by reading their package recipes and the kernel tree they build from,
  running `pahole`'s suite in a container,
  and applying the skill's relevant-execution rule to what each suite shows.
  `pahole`'s suite ran,
  so it stays a finalist;
  `bpftool`'s needs root,
  a BPF-capable kernel and a VM harness,
  so it fails.
  The matrix is rerun over the three finalists,
  and the outcome is unchanged:
  the maintenance weight still ties and reverses the second and third
  places,
  so the skill's result is still "sensitivity unresolved"
  ([The two programs' suites](#the-two-programs-suites)).
- The two wrong `bpftool` statements:
  fixed.
  The version string comes from the numeric defaults in `main.c` that carry the bundled libbpf's
  version,
  not from a `BPFTOOL_VERSION` the build sets;
  and the audited tree is the kernel's `tools/bpf/bpftool` at `v7.3.4`,
  not the standalone
  repository
  ([Provenance of the two programs](#provenance-of-the-two-programs)).
- M1,
  C8's role unsaid:
  fixed with the seven-criteria arm,
  the anchors its ratings used,
  and the corrected sentence about
  why it is rated 4
  ([C8](#c8-human-auditability-of-the-reader-surface-weight-1-higher-is-leaner)).
- M2,
  the design's Q1 and ranking carried the first revision's text beside the new numbers:
  fixed in the design,
  one score scale,
  no `root-spawned` wording,
  the C1,
  C5 and C6 steps named,
  and the rank labelled
  conditional.
- M3,
  the programs' C5 confidence:
  fixed by recording their release dates,
  saying the audit measured mirrors while both projects develop elsewhere,
  and labelling both programs' C3 and C5 medium,
  in the arm that keeps them and in the finalist that stays,
  `pahole`.
  `bpftool`'s ratings are no longer used for any result.
- M4,
  two sentences mismatched the record:
  fixed.
  The standing answer is quoted as the record words it,
  the sentence saying the pair would decide everything is gone,
  and the in-house confidences and three-bucket counts are published
  ([Sensitivity](#sensitivity)).
- M5,
  forks the report said it recorded were missing:
  fixed.
  Both are questions now,
  C8's weight is stated as changing the library pair's stability and not the order,
  and the program-pair ties are moot because `bpftool` failed and no pair between it and `pahole` is
  published
  ([Open questions for the project](#open-questions-for-the-project)).
- M6,
  saying the kept evidence was more than it is,
  a wrong script path,
  and a broken lock chain:
  fixed.
  The status block and the kept-evidence list say which outputs are kept and which are not,
  the path is corrected,
  and the lock record's break is disclosed and reconciled against the commits it names
  ([Report metadata](#report-metadata)).
- The lows:
  L1 in the programs' provenance,
  L2 in the maintenance audit,
  L3 in the check-in's count,
  L4 in the C8 wording,
  L5 in the terminal result,
  L6 in the arm that keeps the programs,
  L7 in the file count,
  L8 in the normalized gap
  and L9 in the process-spawn count are all fixed where the sentence stands.
  Lows the reviewer did not raise but that follow from the corrected source are named where they
  belong,
  including the suite description that cited two scripts the repository does not carry.

## Changes in the second revision

The second revision answers the gate review named under "Revisions" above,
written by a reviewer from another vendor after reading the first revision whole.
Each finding is listed with what became of it and the section that carries the change.

- H1,
  a recommendation the skill does not allow:
  fixed.
  The report states the skill's terminal outcome,
  [sensitivity unresolved](#terminal-result-sensitivity-unresolved),
  with a conditional ranking under each outcome-changing input,
  and the two choices the project must make are questions
  ([Open questions for the project](#open-questions-for-the-project)).
  The design's Q1 now says the same thing in its own words.
- H2,
  weights the skill does not allow,
  and a matrix never run at its baseline:
  fixed.
  The rubric is refrozen at the skill's weights,
  the complete matrix is rerun at them,
  and the design writer's weights are reported as a sensitivity arm only
  ([Refreeze of the second revision](#refreeze-of-the-second-revision)).
- M1,
  no maintenance audit:
  fixed for all four candidates,
  from the repository hosts' own records,
  with every request kept
  ([Maintenance audit](#maintenance-audit)).
- M2,
  auditability unmeasured and unweighted:
  fixed for the four candidates and carried by a new criterion,
  C8
  ([Human auditability](#human-auditability)).
- M3,
  thinner evidence for the two tools:
  partly fixed.
  Their package provenance and its signature record are added,
  C3's root-child premise is corrected,
  and the one omission that cannot be closed here is stated as a depth gap
  rather than hidden under "equal depth"
  ([The two programs' suites](#the-two-programs-suites)).
- M4,
  `btf-rs` C1's confidence label:
  fixed by lowering it to medium,
  which puts that rating into the perturbation set,
  and the arithmetic is rerun
  ([C1](#c1-bounded-fail-closed-handling-of-untrusted-bytes-weight-1)).
- M5,
  the check-in's timing and scope:
  fixed
  ([The check-in of 2026-11-20](#the-check-in-of-2026-11-20)).
- M6,
  the design's stale Q1 text:
  fixed in the design,
  three statements.
- Lows:
  L1,
  L2,
  L3,
  L4,
  L6,
  L7 and L8 are fixed where they touch a fact this report states;
  L5 is fixed with the omission record the skill asks for;
  L9 needs no change and gains the review's own extra-query probe as corroboration,
  because the report already called its class 3 result an inferred exhaustion.

## Fingerprint input

Every field the skill names,
with its value.
The script sorts the three set-valued arrays (the hard constraints,
the base categories and the overlays)
and the deployment keys by raw UTF-16 code-unit order,
and the block below already shows that order;
it serializes the value as RFC 8785 canonical JSON and hashes the UTF-8 bytes with SHA-256.

```json
{
  "schemaVersion": 1,
  "subject": "BTF reader for the ZFS layout checker",
  "decisionScope": "Which BTF reader the start-time ZFS layout check uses: a library the binding crate links, or a program the gate starts, for question Q1 of doc/decision/cli-git-zfs-runtime-layout-checker.md",
  "hardConstraints": [
    "H1 parses split BTF from a module file with its vmlinux base",
    "H2 license compatible with LGPL-3.0-or-later, the wrapper license",
    "H3 builds for x86_64-unknown-linux-gnu and aarch64-unknown-linux-gnu with no C build step and no load-time library from the distribution",
    "H4 reads a file the caller names, with no BPF syscall, no root requirement and no network",
    "H5 exposes named struct members with their bit offsets and type sizes, and enum members with their values, at the depth the anonymous union needs",
    "H6 source is inspectable and its provenance maps to the published artifact"
  ],
  "deployment": {
    "architecture": "x86_64-unknown-linux-gnu and aarch64-unknown-linux-gnu",
    "distribution": "the wrapper release and the root handler, both relying on no OpenZFS userland at start",
    "input": "the kernel base BTF file and the ZFS module BTF file under /sys/kernel/btf",
    "os": "Linux"
  },
  "trustBoundary": "high-trust: the same check runs in the git wrapper and in the root ZFS event handler process, and parses kernel-produced bytes",
  "incumbent": null,
  "baseCategories": [
    "inspectable open-source local technology"
  ],
  "overlays": [
    "high-trust execution in an agent, plugin, hook, CI runner, or credential boundary",
    "multi-platform or browser-baseline claim",
    "native, Wasm, prebuilt binary, or generated-code boundary"
  ]
}
```

`incumbent` is `null` and that is a measured fact,
not an omission.
`rg --ignore-case 'from_split|hdr_len|0xeb9f|btf_header|parse_btf' package` returns nothing,
and the ten files under the binding crate that mention BTF either name the module's BTF path,
shell out to `/usr/bin/pahole` for the test-time comparison
(`package/zfs/request-to-openzfs-kernel-device/src/kernel_device/live_btf_tests.rs:1-40`),
or carry a layout constant read from that file.
Its `Cargo.toml` declares `libc` alone.
This worktree has no reader:
step 6 of the implementation plan,
which would have held one,
is unbuilt on the branch this worktree carries.
An in-house decoder does exist,
unmerged,
on `feat/git-policy-cli-zfs-start-time-compatibility-gate` at `ac59add52c`:
`btf_decode.rs` 125 lines,
`btf_entry.rs` 225,
`btf_header.rs` 354,
`btf_records.rs` 154 and `btf_types.rs` 242,
1,100 lines before tests,
with `btf_header_tests.rs` 137,
`btf_records_tests.rs` 121 and `btf_types_tests.rs` 164,
422 lines of tests.
The skill's fourth source class counts a parallel hand-rolled alternative,
so the ledger carries it (section "Discovery ledger");
it stays ineligible for the reason that section gives.

## Hard constraints

Each is the design's,
quoted in its own words,
and each was applied before any rating.

- H1:
  parses split BTF from a module file with its vmlinux base.
  A module's BTF extends the kernel's base:
  its type ids and string offsets cross into the base.
- H2:
  license compatible with `LGPL-3.0-or-later`,
  the wrapper's license and the binding's (`package/zfs/request-to-openzfs-kernel-device/Cargo.toml:21`).
- H3:
  builds for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`
  with no C build step and no load-time library from the distribution.
  The wrapper ships as two GNU release archives (`toolchain.lock.json:614-653`)
  and must start on a machine without OpenZFS userland.
- H4:
  reads a file the caller names,
  with no BPF syscall,
  no root requirement and no network.
- H5:
  exposes named struct members with their bit offsets and type sizes,
  and enum members with their values,
  at the depth the anonymous union needs.
- H6:
  source is inspectable and its provenance maps to the published artifact.

H3 is the constraint that narrows the field,
and it is the project's,
not a fact of nature.
Dropping it re-admits `libbpf-rs` and a `dlopen` of the system's `libbpf`,
which is a choice for the owner (section "Open questions for the project").
H5 is why a low-level ELF reader is not a candidate by itself:
`object`,
`goblin`,
`scroll` and `libelf` expose section bytes,
not BTF types.

## Base components and overlays

One base category,
for every candidate that reached finalist validation and for the refused candidates that got as far
as screening:
inspectable open-source local technology.
Two are libraries the binding links (crates) and two are programs the gate starts,
and one of the programs failed the skill's relevant-execution rule after that validation;
the skill's base category does not separate them,
and criterion C2 does.

Overlays,
every one that matches:

- High-trust execution:
  the same check runs in the git wrapper and in the root ZFS event handler process.
- Native,
  Wasm,
  prebuilt binary,
  or generated-code boundary:
  every candidate parses kernel-produced bytes,
  and no candidate links native code.
  For `pahole` and `bpftool` the artifact is a distribution binary,
  so the provenance question is the package's and the release-version matrix is owed (measurement item 14).
- Multi-platform claim:
  the wrapper's two GNU release targets.

Gates marked not applicable,
with reasons:

- Managed service and SaaS gates:
  no hosted component is proposed.
- Sensitive data,
  privacy,
  compliance,
  residency,
  retention,
  geography:
  the input is kernel type metadata and the output is a verdict about struct offsets;
  no personal data crosses the boundary.
- Proprietary local technology:
  every candidate's source is readable.
- Replacement parity:
  there is no incumbent to replace (`incumbent: null` above).
  This is not a swap,
  so the parity overlay and rule `RCO` do not apply.
  The crate's test-time `pahole` call is a fixture,
  not a runtime reader,
  and it is recorded as repository precedent under "Discovery ledger" rather than as an incumbent.
- Open-source precedence:
  satisfied with no exception;
  no proprietary candidate was needed and none was admitted.

## Frozen criteria, refrozen weights

Frozen before any candidate-specific soft evidence was rated,
and applied to every finalist.
The weights below are the refrozen ones (section "Refreeze of the second revision"),
which are the skill's weights and not the design writer's.

- C1,
  bounded,
  fail-closed handling of untrusted bytes,
  weight 1.
- C2,
  release and deployment fit:
  no runtime program,
  no C build,
  both GNU targets,
  weight 1.
- C3,
  privilege surface in the root handler,
  weight 1,
  higher is smaller.
- C4,
  diagnostic precision:
  member-level expected and actual values,
  weight 1.
- C5,
  maintenance and upstream responsiveness,
  weight 1.
- C6,
  integration and test cost,
  weight 1,
  higher is cheaper.
- C7,
  start cost,
  weight 1,
  low confidence because it is unmeasured.
- C8,
  human auditability of the reader surface a root process runs,
  weight 1,
  higher is leaner,
  added at the refreeze from the skill's overlay
  (section "Human auditability").

Ratings run 0 to 4,
every rating carries a confidence,
and a low-signal rating carries the evidence-supported minimum and maximum whose midpoint is the provisional value.
No criterion is removed from the denominator:
each one bears on at least one finalist.
Maximum points are 32:
the weight sum 8 times 4.
At the design writer's weights,
reported below as a sensitivity arm only,
they are 108.

The ratings for the in-house reader price code that does not exist,
and the arithmetic marks that score as priced,
not ranked.

## Refreeze of the second revision

The first revision froze the design writer's weights,
5,
5,
4,
4,
3,
3 and 2,
while stating that the project had set no priority among the criteria.
The skill's rule for that case is weight 1 for every criterion whose priority is unspecified,
and the first revision never ran the one-input matrix at those weights,
so its baseline was outside the skill and its matrix described another rubric.
The gate review re-ran it at those weights and found
a program passing a library.

The second revision refroze the rubric before rerunning anything,
and made three changes,
all recorded here before the rerun below was run:

1.  Weights:
    every criterion 1,
    because the project has stated no priority.
    The design writer's weights are kept as a reported sensitivity arm,
    with C8 at 1 because their text states no priority for it either.
2.  A criterion C8,
    human auditability,
    from the skill's high-trust overlay,
    which the first revision named and then measured nothing against.
    Its evidence is section "Human auditability".
3.  `btf-rs`'s C1 confidence,
    from high to medium.
    The first revision rated it high while rating `ebeepf`'s C1 medium on the same class of evidence,
    and the archive shows no bounds check of its own and no fuzz target
    (section "C1").

All three changes were made after the first revision had named a winner,
and the report states that plainly rather than arguing it away:
C8 was added in answer to the first gate review,
after the first revision had rated,
scored and recommended `btf-rs`,
and the C1 confidence change was made in the same round.
Both are permitted corrections and both depart from the letter of the skill's freeze,
which freezes criteria and weights "before candidate-specific soft evidence is rated" and lists
changing weights after seeing a preferred winner as a violation signal.
The project is told,
because the criterion decides a result it would otherwise not see:
C8 is why the library pair is stable at the frozen weights.
With the seven original criteria and the same ratings at weight 1,
`btf-rs` over `ebeepf` is 23.5 against 22.5 and ties on six one-input steps
(`btf-rs` C1,
C5 and C6 down;
`ebeepf` C1,
C5 and C6 up),
and that seven-criteria family flags more of its tests than the eight-criteria one does
(section [C8](#c8-human-auditability-of-the-reader-surface-weight-1-higher-is-leaner)).
What cannot be confirmed from anything kept is the order of the two events in the second revision:
its claim that all three changes were recorded "before the rerun below was run" is a claim about its own
sequence,
not a measurement a reader can check.
The claim that C8 cannot have changed the winner is right and does not settle the pair:
it favours the leader,
so it cannot create a leader,
and it is still the reason the leader's lead is stable.
The C1 change moves against `btf-rs`;
and the weights are the skill's default rather than a reading of the evidence.
No criterion rating other than that one confidence label changed,
and no weight was chosen to make a preferred candidate win.
The complete matrix over the refrozen rubric is section "Sensitivity".

## Open questions for the project

Each question says what it changes,
and each says whether it bears on an order at all.
An order below the first place can move on questions 1,
3,
4 and 5,
and the first place moves on none of them:
`btf-rs` is first in every defined test at the frozen weights.
One of them decides whether this report recommends anything,
which is question 4:
if `pahole` is not an eligible finalist the family is two libraries,
all 51 of its one-input tests keep the order,
and the skill permits a recommendation of `btf-rs`.
Which questions are conditional on which:
questions 3,
4 and 5 bear only while the program arm is live,
which is question 2;
question 1 bears on places 2 and 3 while `btf-rs` stays first;
question 6 is a weight on a criterion that leaves the order as it is;
question 7 changes the candidate set and not this ranking;
question 8 reopens a candidate this report priced and did not rank;
and question 9 changes no order.

1.  How much maintenance and upstream responsiveness should weigh.
    This is the priority the second revision's question 2 carried,
    and it is the controlling preference now:
    at the skill's default weight of 1,
    the order is `btf-rs` 27.5 > `ebeepf` 24.5 > `pahole` 18.5 of 32.
    At weight 4,
    `ebeepf` and `pahole` tie at 30.5.
    At weight 5,
    `pahole` is second at 34.5 and `ebeepf` third at 32.5.
    What it changes:
    whether the second library or the one surviving program is second.
    What it does not change:
    the first place,
    which stays `btf-rs` at every weight from 1 to 5,
    by at least 2 over the runner-up.
    The report cannot answer it from evidence,
    because a weight is a preference,
    and the project has stated none.
2.  A library the binding links,
    or a program the gate starts.
    This is the design's Q1,
    and it is a live choice again while the project keeps `pahole` a finalist (question 4).
    The two arms are not interchangeable:
    a library runs in the handler's address space,
    where a parser bug is a panic,
    a hang or a large allocation inside the privileged process;
    a spawned program runs in a child,
    and both BTF files are mode `0444` on this host,
    so that child need not run as root and can carry a hard resource limit and a sandbox.
    What it changes:
    the deployment and privilege price the rubric already charges at C2 and C3,
    and the operational shape of the gate.
    What it does not change:
    the arithmetic,
    which puts `pahole` third at weight 1,
    tied with `ebeepf` for second at 4 and second at 5,
    and first under no weight from 1 to 5,
    so choosing it is a preference for the program shape and not a reading of the scores.
    Its being a finalist at all rests on question 4:
    its suite ran and its own CI path did not.
    If the project wants `pahole`,
    question 1 is what would raise it,
    and even at weight 5 it is 5 points behind `btf-rs`.
3.  `bpftool`'s eligibility.
    The report fails it under the skill's relevant-execution rule,
    because its functional tests need root,
    a BPF-capable kernel and a VM harness
    as the report reads them
    ([The two programs' suites](#the-two-programs-suites)).
    What it changes:
    a departure that keeps it eligible adds the `pahole` and `bpftool` pair,
    with thirteen rating-step ties and no reversal,
    and leaves the top of the order where it is;
    the report's choice keeps that pair out of the ranking entirely.
    `bpftool` scored 17.5 of 32,
    last of the four.
4.  `pahole`'s eligibility with its own CI path unrun.
    The skill's finalist validation asks for every upstream task and CI job,
    the complete default CI-equivalent path,
    and evidence for every omitted suite.
    `pahole` has the suite and not the path,
    and the evidence it offers for the one omitted case that reaches the consumed surface is a check
    of another layer
    ([The upstream CI of `pahole` at `v1.32`](#the-upstream-ci-of-pahole-at-v132)).
    What it changes:
    everything below the first place,
    and whether anything is recommended.
    Eligible:
    the finalists stay as they are,
    the maintenance weight still ties and reverses places 2 and 3,
    and the outcome is "sensitivity unresolved" with no recommendation.
    Not eligible:
    the family is `btf-rs` and `ebeepf`,
    all 51 of its one-input tests keep the order,
    the skill's condition for ranking by the baseline is met,
    and the recommendation is `btf-rs` over `ebeepf` in all 51.
    What the report cannot do is answer it from evidence:
    the evidence it has is the 323-check harness on this host's pair,
    a consumer-layer check on one kernel,
    offered as the omission evidence as it already is for `ebeepf`'s `opens_kernel_module_split_btf`.
    Whether that stands for the upstream case is the project's judgement,
    and the skill's words ask for evidence that the omission cannot affect the surface,
    which this is not.
5.  The trust boundary and the child's privilege.
    The first check-in did not put this fork,
    although the skill lists a trust boundary among the
    context inputs that can change the weights.
    It is a question now,
    not a settled fact:
    whether the gate's program arm may run its child unprivileged and bounded,
    or whether the design keeps the root-handler child shape it names.
    What it changes:
    the price of C3,
    and only for a program candidate;
    with the two libraries it changes no order.
    It also moves where question 1's threshold sits,
    so the two are not independent choices:
    with `pahole`'s C3 at 3 it ties `ebeepf` at a maintenance weight of 3,
    and with C3 at 4 it passes `ebeepf` there,
    both below the weight of 4 where the report's own ratings tie them.
6.  Feature breadth against auditability.
    Also not put at the first check-in.
    This fork is C8's weight.
    What it changes:
    how much of `btf-rs`'s lead is auditability.
    C8 at 1 through 5 leaves the order as it is,
    because `btf-rs` rates 4 there against `ebeepf`'s 2 and `pahole`'s 2;
    removing the criterion,
    which is weight 0 and outside the skill's one-input tests,
    ties the library pair on six steps
    (section [C8](#c8-human-auditability-of-the-reader-surface-weight-1-higher-is-leaner)).
    What it does not change:
    who leads at any weight the skill tests.
7.  H3,
    the constraint that narrows the field.
    H3 is the project's,
    not a fact of nature.
    What it changes:
    dropping it re-admits `libbpf-rs` with its C build,
    and a `dlopen` of the system's `libbpf`,
    which is a new candidate set and a new round,
    not a rescoring of this one.
    Keeping it keeps the field this report screened and ranked.
    It does not change this report's result.
8.  Reopening the in-house reader.
    The standing answer of 2026-11-19 was "No,
    not required" to the conjunction,
    that the parser be the project's own code and that the product start no second program;
    the record's section "The reader question,
    Q1" says of that answer,
    "So no hard constraint admits an in-house reader",
    and the skill's existing-tools rule keeps it out.
    What it changes:
    the question becomes a fresh comparison,
    because the priced row ties `btf-rs` at 27.5 and its pair moves in 23 or 28 of 47 or 52 tests
    (section [Sensitivity](#sensitivity)).
    It does not produce an answer from this report's numbers.
9.  The disk-cache question,
    Q3 of the decision record,
    is held and stays separate.
    Its thresholds,
    a 100 ms median above the parse's baseline and a 256 MiB resident peak,
    are the design's proposal from a single probe with no control,
    and measurement item 10 is the run that replaces them.
    Nothing here needs its answer and no answer changes the ranking:
    the per-process cache ships either way.

Two forks are recorded as unasked rather than settled,
where the second revision's check-in section said they were recorded in this section and they were
not:
question 5,
the trust boundary,
question 6,
feature breadth against auditability.

## Context

The ZFS binding sends `/dev/zfs` requests whose command struct,
`zfs_cmd`,
the kernel and the module must agree on byte for byte.
A member that moves turns a destroy into something else,
and userspace cannot see the move.
So the gate reads the module's own type information,
`/sys/kernel/btf/zfs`,
with the kernel's base file `/sys/kernel/btf/vmlinux`,
and compares the declared sizes,
member offsets and enumerator values with the ones the crate's own `#[repr(C)]` types compile to.
A mismatch,
a missing file,
a short read or a parse failure disables raw requests and the wrapper takes the Git route instead (decision record,
"Decision").

The reader is the piece of that gate that is not in this worktree.
The project answered the two questions that bounded it.
It was asked whether it is a hard requirement that the parser is our own code and that the product
starts no second program,
and it answered "No,
not required" on 2026-11-19:
the conjunction is not required,
which is why the gate review called the first revision's "the product may start no second program" an inversion,
and why no crate and no program is ruled in or out by that answer.
No release string decides ("Any version,
layout decides").
The crate placement is settled:
the binding package holds the loader,
the pre-scan,
the expectations,
the comparison and the reader once it is picked (decision record,
Q4).
What is left is which reader.

Since the design was accepted at `22703f23df`,
four items Q1 listed have closed or moved.
Finalist validation ran on 2026-11-20;
discovery finished in a round on the same day with the project's check-in answer and a saturated terminal result;
the check-in artifact is written;
and this report closes the last one.
That leaves two limits,
and this report states them wherever the affected row is scored rather than in a closing caveat:
`aarch64` is built for both libraries and run for none,
and the start cost is unmeasured.

## The check-in of 2026-11-20

The skill asks one check-in per broad vetting process,
**before the first discovery query**,
stating the subject,
the forks settled so far,
the source classes,
the draft schedule,
and who runs the enumeration.
This vetting process is broad:
its names came from registries,
repository hosts and web pages,
not only from memory,
the project's picks or repository precedent.

This check-in came after the fact for most of the enumeration,
and the report says so rather than calling it pre-discovery.
The order of events was:

- the initial round,
  twelve numbered queries,
  the sixth of which is the first web search,
  ran on 2026-11-19 between 18:10:30 and 19:38:00-05:00,
  before any check-in (section "Query schedule",
  items 1 to 12);
- the check-in was put to the project on 2026-11-20 and answered the same day;
- the queries it covered are the ones run after it,
  items 13 to 15 and the three class 2 organization lists,
  which are the remaining web-class queries and list reads;
- the terminal discovery result therefore rests on a schedule whose first twelve queries
  were enumerated before the project was asked,
  and whose rest the project delegated.

The artifact stands under [the design's discovery record][design-check-in]
and is quoted here because this report is written afterwards and must carry it.

- Subject:
  the BTF reader question Q1 picks.
- Settled forks:
  a library the binding links or a program the gate starts;
  no hard requirement that the parser be our own code;
  the two GNU release targets.
- Source classes:
  the skill's four,
  crates.io,
  the repository host,
  the wider web,
  and this repository with its prior reports.
- Draft schedule:
  the three class 3 queries and the two class 2 list reads the earlier rounds had left,
  each with its literal query,
  provider,
  filters,
  page and result count.
- Options offered:
  the earlier sessions' writer;
  a model the project names;
  the project itself.
- Answer:
  "This session's writer (Recommended)",
  described as a `DeepSeek V4.1 Flash` writer through Pi running the remaining queries now.
- Date:
  2026-11-20.

The answer named the first revision's writer,
and the round's three class 3 queries and two class 2 list reads ran there afterwards,
as the answer describes;
no delegated starting brief was needed.
What the answer did not cover is the enumeration itself:
the earlier twelve queries were already run when it was asked.
Forks the skill lists among the context inputs were not put in that question either,
namely the trust boundary and the feature-breadth-versus-auditability tradeoff;
both bear on the weights,
and the second revision records them as unasked rather than as settled
(section "Open questions for the project").

## Query schedule

Frozen in two rounds,
then expanded once for taxonomy terms and frozen again,
exactly as the skill's pagination and expansion rules ask.
Ordering is the provider's default,
no `sort` parameter was sent in any query,
no include or exclude filter was sent in any query,
and no result was truncated after the page rule.
The page rule is `per_page=100`,
the provider's maximum,
for every registry and repository query,
first page only;
each count is the provider's own total,
so that page held every row.

Initial round,
2026-11-19T18:10:30-05:00 to 18:53:37-05:00:

1.  crates.io `q=btf`,
    no filter:
    81 rows of 81.
    New candidates:
    `btf`,
    `btf-rs`,
    `btfparse`,
    `btfdump`,
    `aya-obj`,
    `padlock-dwarf`.
    New survivors:
    `btf-rs`,
    `btfparse`,
    `btfdump`,
    `aya-obj`.
2.  crates.io `q=split btf`:
    17 rows of 17,
    every one also in the `q=btf` page.
    None new.
3.  crates.io `q=btf kernel`:
    50 rows of 50,
    every one also in the `q=btf` page.
    None new.
4.  GitHub repository search `BTF parser in:name,description language:rust`:
    1 row (`retis-org/btf-rs`).
    None new.
5.  GitHub code search `from_split_bytes language:rust`:
    25 rows over 6 repositories.
    None new.
6.  Web search,
    the provider's own ranking,
    one call:
    recorded under source class 3.

Expansion round,
2026-11-19T19:30:00-05:00 to 19:38:00-05:00:

7.  crates.io `q=libbpf`:
    91 rows of 91.
    None new;
    the BTF-related rows are `libbpf-rs`,
    `libbpf-sys`,
    `aya-obj` and its forks.
8.  GitHub repository search `BTF parser in:name,description`,
    no language filter:
    7 rows.
    None new.
9.  GitHub repository search `btf in:name,description language:rust`:
    37 rows.
    None new.
10. GitHub repository search `topic:btf language:rust`:
    3 rows.
    None new.
11. GitHub repository search `topic:btf-parser`:
    0 rows.
    None new.
12. GitHub releases for each audited repository at `per_page=100`:
    5,
    12,
    5 and 31.
    None new.

Round of 2026-11-20,
after the check-in answer,
all on the DeepSeek provider,
each one page:

13. `Rust crate parse BTF split BTF module vmlinux base BTF parser library`:
    44 rows.
    Names the ledger lacked:
    `bpf-compatible-rs`,
    `vock`.
    New survivors:
    none.
14. `C library parse BTF split BTF module vmlinux base BTF parser`:
    46 rows.
    Names the ledger lacked:
    `libabigail`,
    `parttimenerd/btfparse`.
    New survivors:
    none.
15. `alternatives to libbpf btf__parse BTF parser library`:
    49 rows.
    Names the ledger lacked:
    `ebeepf`,
    `cilium/ebpf`,
    `libbpfgo`,
    `gbpf`,
    `coolbpf`,
    `bpf-compatible`.
    New survivors:
    one,
    `ebeepf`.

Class 2 lists of the same round,
from the repository host's API with pagination to exhaustion:
`libbpf` 15 of 15 public repositories,
`trailofbits` 298 of 298,
`eunomia-bpf` 157 of 157.
`projectsV2(first: 1) { totalCount }` answers 1 for each of the three,
and their public boards were read as HTML
because the session's token lacks `read:project`
and the REST project endpoints are sunset.
The boards are "BPF Verifier Visualizer" (no items),
"[ENG] Sigstore Transparency Log Monitoring",
and "eunomia-bpf" (no items).
None names a candidate.

What the filters can hide,
recorded because the skill asks:

- The three class 3 pages are provider-ranked,
  not registries,
  so they are not provably exhaustive.
  Each returned fewer rows than the provider's 50-row cap and offered no cursor,
  so the provider reported its listing ended;
  no page 2 exists for those queries.
- The first attempt at class 3 stopped at that provider's 10-row display cap while the provider reported 44 rows,
  and the first attempt at class 1 read 50 of 81 rows at `per_page=50`,
  a provider cap and not saturation.
  Both were re-run to exhaustion and the re-runs are the records above.
- Class 2 item 4 was ambiguous once:
  the first version's filtered run returned 7 rows
  and today's re-run returns 1 for the filtered query and 7 for the unfiltered one,
  so which filter the index applied then is not knowable now.
  Both listings were screened and neither adds a survivor.
- A release list is only as complete as the API's own total,
  and the `per_page=100` reads equal the totals 5,
  12,
  5 and 31;
  a reader trusting the default page size of 30 would undercount the last.
- Organizations outside the three read were not enumerated,
  and the three were chosen because they hold every repository the ledger audits.
  An organization that holds no candidate is not a candidate index.

Terminal result:
saturated with at least two survivors.
Screening survivors:
`btf-rs` 2.0.1,
`ebeepf` 0.1.0,
`pahole` 1.32,
`bpftool` 7.8.0.

The two numbering schemes in these documents count the same queries differently,
and a reader moving between them should know it.
This report numbers every query of every round in one sequence,
so `ebeepf`'s discovery is query 15;
the design numbers it inside its own section,
where it is the third query of the 2026-11-20 round
(design,
"Class 3 of this round").

The class 3 result is an inferred exhaustion,
not a reported one,
and the second revision keeps that reading.
Its three pages ended below the provider's cap with no cursor,
which is what a finished provider listing looks like and also what a silently capped one can look like.
The gate review tested that risk directly rather than arguing it:
eight further crates.io queries the frozen schedule never used
(`bpf type format`,
`vmlinux`,
`pahole`,
`kernel btf`,
`type format kernel`,
`btf split module`,
`dwarves`,
`bpf debug info types`)
found no BTF reader with a split base that the ledger lacks
(`~/temp/agent/gate-review-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/probe-crates-io-extra-queries-output.txt`).
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
That supports the saturation;
it does not prove it,
and no claim here rests on it alone.

A corroboration this report ran,
read only,
the response written at 2026-11-20T18:08:38-05:00 by its own file modification time:
`curl --silent --user-agent '...' 'https://crates.io/api/v1/crates?q=btf&per_page=100&page=1'`,
HTTP 200,
`total` 81 and 81 rows,
and every row name matches the design's transcription in the same order.
Response kept as `keep/crates-io-q-btf-page-1-2026-11-20.json` beside this report's fingerprint script.
No new parser appeared between the round and this report.

The kept discovery artifacts,
under `/home/user/temp/agent/zfs-layout-checker-reader-discovery-finished-2026-11-20/keep/`:
`query-1-literal-deepseek-page-1.txt`,
`query-2-c-libraries-deepseek-page-1.txt`,
`query-3-alternatives-deepseek-page-1.txt`,
`org-libbpf-repos.json`,
`org-trailofbits-repos.json`,
`org-eunomia-bpf-repos.json`,
the three `org-*-projects-listing.html` pages,
the three `org-*-project-*.html` pages,
and `evidence/checksums.txt` over them and the BTF copies.
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
Two ecosyste.ms reads sit beside them
and one was refused for a missing descriptive User-Agent
(`ecosystems-crates-keyword-btf-page-1.json` holds that error,
and `ecosystems-topic-btf-page-1.json` reports 0 repositories for the topic).
They were attempts,
not required source classes,
and the class that mattered,
crates.io,
finished.

## Discovery ledger

Each row names the candidate,
its discovery source,
its base category and overlays,
and its screening result.
Every version row's license and checksum is the crates.io index entry for that version,
read on 2026-11-20.
The base category is the same for every row and is stated once:
inspectable open-source local technology,
with the high-trust,
native-boundary and multi-platform overlays from section "Base components and overlays".
The exhaustive row-by-row listing of the 81 registry rows
and the 37 repository rows is in [the design's discovery record][design-discovery],
which this report does not duplicate;
what follows is complete at candidate granularity,
so every candidate the listing names is either a row here or inside the screened-together group.

### `btf-rs` 2.0.1

- Discovery:
  crates.io `q=btf`,
  first result by recent downloads among parser crates;
  also returned by the class 2 repository and code searches.
- Version:
  2.0.1,
  tag `v2.0.1`,
  commit `03fd01bd9505d0888b568cd3aaa9b232b7e3fb20`,
  published 2026-11-12;
  license `LGPL-2.1-or-later`;
  archive SHA-256 `fa34ec915ede6b7eaeebcb38fdb749fcac1538ec1b94dcfde63a034ad8f7963b`,
  equal to the index `cksum`.
- Provenance,
  measured in the second revision:
  the archive's `src/`,
  `LICENSE` and `examples/` are byte-identical to the tag's
  (`diff -r --brief` printed nothing),
  so the mapping rests on the bytes and not only on the archive's own `.cargo_vcs_info.json`,
  which a publisher writes and `cargo` never verifies.
  The product's build also runs a second archive,
  `btf-rs-derive` 2.0.1,
  a proc-macro the same authors publish from the same commit with `path_in_vcs` `btf-rs-derive`;
  its SHA-256 is
  `95d548bd1301bf327418190bbe8d63bc34b5671cc01f3ecf04aceb8ac47a7c60`,
  its `src/lib.rs` is 114 lines,
  and it is byte-identical to the tag's `btf-rs-derive/src/lib.rs`.
  It is the same-author dependency the audit surface has to carry.
- Screening:
  pass,
  every screening gate either proven or pending targeted evidence.

### `ebeepf` 0.1.0

- Discovery:
  class 3 query 15 of the round of 2026-11-20,
  whose 49-row page named it.
  Record correction:
  the class 1 page also held the name,
  tagged at the time as "eBPF loader,
  reads BTF as an input" from its description alone;
  that tag predates its source audit and the row below supersedes it.
  The class 3 page is what promoted it to a candidate.
- Version:
  0.1.0,
  published 2026-10-06,
  commit `5de05465bd9445d89b13a967e198d0aeffd71ec4`;
  license `LGPL-2.1-only OR BSD-2-Clause`,
  so the BSD arm applies;
  archive SHA-256 `f5f4162021ff5dce1b6c5a657267523d7d41c6818bae2adc0adbfbf4b5b6fa96`,
  equal to the version entry's `checksum`.
- Screening:
  pass.

### `btfparse` 1.3.9

- Discovery:
  crates.io `q=btf`.
- Version:
  1.3.9,
  commit `3788c4410a6cfd64ff57aa5bca20b2ee4eed3cd5`,
  2026-01-08,
  `Apache-2.0`,
  no dependencies,
  5,876 lines under `src/`.
- Screening:
  H1 fail.
  One `TypeInformation::new` over one `Readable`,
  numbering types from 1,
  with no base file (`src/btf/type_information.rs:435-490`),
  so a module's crossed-in ids and string offsets cannot be represented.
  Exit at screening;
  no score.

### `aya-obj` 0.6.0

- Discovery:
  crates.io `q=btf`,
  by downloads.
- Version:
  0.6.0,
  `aya-rs/aya` at `44a6709164d2ac0c19c7845101e7b033e25b3f3f`,
  `MIT OR Apache-2.0`.
- Screening:
  H1 fail.
  Its reader parses one object's `.BTF` section and has no base;
  the `base` at `aya-obj/src/btf/btf.rs:355` is a byte offset (`:340-365`).
  Exit at screening.

### `btf` 0.5.1

- Discovery:
  crates.io `q=btf`.
- Version:
  0.5.1,
  `arcjustin/btf` at `3b57cdab2ea80d13c3b5abd400bb2e4bfc0d6292`,
  last commit 2023-02-08,
  `MIT`.
- Screening:
  H1 fail,
  one blob parser with a 24-byte header and no base (`src/btf.rs:15-81`);
  no commit since 2023 is noted and the gate alone is enough.
  Exit at screening.

### `btfdump` 0.0.5

- Discovery:
  crates.io `q=btf`.
- Version:
  0.0.5,
  `anakryiko/btfdump` at `e5951d4b2e2e7ce6c36001c1b48c1c239c9759dc`,
  `BSD-2-Clause`,
  with `clap`,
  `object`,
  `memmap2`,
  `scroll`,
  `scroll_derive`,
  `regex`,
  `bitflags`.
- Screening:
  H1 fail as a library:
  it dumps one BTF file and has no split base (`src/types.rs:1089-1120`,
  `src/lib.rs`).
  As a tool it would be the start-a-program route with an extra package;
  the ledger's two tool finalists already cover that route.
  Exit at screening.

### `padlock-dwarf` 0.82.0

- Discovery:
  crates.io `q=btf`.
- Version:
  0.82.0,
  `gidotencate/padlock` at `6a7d045f2c6b0cb64549ddebe5825411e25178fa`,
  `MIT OR Apache-2.0`.
- Screening:
  H1 fail,
  a 794-line BTF reader inside a DWARF and PDB analyzer,
  single blob,
  no base (`crates/padlock-dwarf/src/btf.rs:114-152`);
  H5 fails on fit,
  since its output is a whole-program struct list.
  Exit at screening.

### `libbpf-rs` 0.27.2 and `libbpf` through FFI

- Discovery:
  crates.io `q=libbpf`,
  and repository precedent (the wrapper's prior vet refused the C libraries).
- Version:
  0.27.2,
  `libbpf/libbpf-rs` at `6d771ee9835fcf517dbcf1d29e25a97480425d45`;
  license `LGPL-2.1-only OR BSD-2-Clause` from the crates.io metadata,
  so the BSD arm applies.
- Screening:
  H3 fail.
  Its default feature is `libbpf-sys/vendored-libbpf`,
  which runs libbpf's C build,
  and its manifest says it "requires other necessary libs to be present on the system" (`libbpf-rs/Cargo.toml:20-26`).
  A `dlopen` of the system's `libbpf` avoids the link
  but keeps the native provenance
  and adds a privileged library load inside the root handler,
  so H3 and H6 fail there too.
  Exit at screening on H3.
  This is the row that H3 admits if the project drops it.

### `bpftool` 7.8.0

- Discovery:
  this host (`/usr/bin/bpftool`) and the libbpf reader it carries;
  also on the `libbpf` organization's repository list as its own project.
- Version:
  7.8.0 on libbpf 1.8,
  as the binary reports itself;
  the tool's license is the kernel's (GPL-2.0 with BSD-2 parts) and it runs as a separate process,
  which is not a link.
- Provenance:
  `/usr/bin/bpftool` is the distribution's `bpf 7.3.4-1` package
  (`pacman --query --owns /usr/bin/bpftool`),
  built 2026-11-12,
  installed 2026-11-16 with `Validated By : Signature`,
  and built by Arch's `linux-tools` PKGBUILD at its tag `7.3.4-1`
  (`pkgver=7.3.4`,
  `pkgrel=1`) from `linux-7.3.4.tar.xz`,
  sha256 `b11b6df8295e8e991ff040b71aaa4994a8b55caa23b6b092907f195b5840fd1f`,
  with a PGP signature beside it in the recipe's source list.
  The recipe builds `tools/bpf/bpftool` from that tarball,
  so the binary's source is the kernel tree at `7.3.4`,
  not the standalone `libbpf/bpftool` repository.
  Its self-reported version and its package's version disagree,
  and the mechanism is now read from the source rather than guessed:
  the kernel tree's `tools/bpf/bpftool/main.c:98-106` and `:180-184` fall back,
  when `BPFTOOL_VERSION` is unset,
  to the major number of the bundled libbpf plus 6,
  the libbpf minor and a patch of 0;
  `tools/lib/bpf/libbpf_version.h` at `v7.3.4` declares 1.8,
  so the fallback prints `7.8.0`.
  The Makefile only passes `-DBPFTOOL_VERSION` through when the variable is already set
  (`tools/bpf/bpftool/Makefile:82-83`),
  and the recipe sets nothing.
  So the self-reported version carries the bundled libbpf's version and is not meaningless;
  it just does not name the distribution package.
  The binary's SHA-256 is
  `475a53182f50231c40e40ce002f2e032dc418489d56a2f8f902aa317aed03d20`.
- Screening:
  pass on H1,
  H2 for a separate process,
  H4,
  H5;
  H6 pass on the recipe above;
  H3 is not applicable as a library but the product needs the program at run time,
  which C2 prices.
  Finalist on the hard gates,
  then failed on relevant execution
  ([The two programs' eligibility](#the-two-programs-suites)).

### `pahole` 1.32

- Discovery:
  this host;
  the design's own test-time inspection command;
  repository precedent,
  since the binding's live test already runs `/usr/bin/pahole`.
- Version:
  1.32,
  GPL-2.0-only,
  a separate process.
- Provenance:
  `/usr/bin/pahole` is the distribution's `pahole 1:1.32-2` package
  (`pacman --query --owns /usr/bin/pahole`),
  not a package named `dwarves`,
  built 2026-11-02,
  installed 2026-11-16 with `Validated By : Signature`;
  the binary's SHA-256 is
  `00acbd859cd0cca6e71dca93d885ed2483c5a52a78d3df1a9bcc9da82881503c`.
  CachyOS's `pahole` PKGBUILD has `pkgver=1.32` and `pkgrel=2`,
  which is the installed release,
  and builds the signed git tag `v1.32` with three patches:
  `0001` for the CMake install of `ostra.py`,
  and `0002` and `0003` in `dwarf_loader.c` for merged LTO CUs and kept CUs.
  So the binary is not upstream `v1.32` byte for byte.
  The recipe builds libbpf in with `LIBBPF_EMBEDDED=ON`,
  and its `prepare()` runs `git submodule init`,
  `git submodule set-url lib/bpf "$srcdir/libbpf"` and `git submodule update`,
  with no `--remote` anywhere;
  `git submodule update` without options performs "a checkout",
  of "the commit recorded in the superproject",
  so the embedded libbpf is the submodule entry the tag records and not the branch's tip.
  That entry is `dd92bef7f6c7a00bb0312554119b6d9cf38e4f32`,
  libbpf's commit message `Makefile: bump to v1.8.0`,
  dated 2026-04-09,
  so the package builds libbpf 1.8.0.
  What the tag records is not what the kept suite run built,
  and that run is a separate fact recorded
  under [The two programs' suites](#the-two-programs-suites).
  `pacman --query --check --check pahole bpf` reports 0 altered files for each;
  the manual describes the doubled flag as checking permissions,
  file sizes and modification times
  against the package's mtree,
  and this report does not claim it compares content digests.
  Its BTF parser is libbpf's,
  built into the package rather than loaded from the distribution at run time:
  `readelf --dynamic /usr/bin/pahole` lists only `libdwarves`,
  `libdwarves_emit`,
  `libdwarves_reorganize`
  and `libc`.
- Screening:
  pass on H1,
  H2 for a separate process,
  H4,
  H5,
  H6 on the recipe above.
  Finalist.

### The in-house reader

- Discovery:
  repository precedent (the binding's own unsafe boundary and its "our own small binding" decision)
  and the skill's existing-tools rule,
  which counts parallel systems and hand-rolled alternatives as its fourth source class.
  It is not hypothetical code:
  `feat/git-policy-cli-zfs-start-time-compatibility-gate` at `ac59add52c`
  carries a working decoder under
  `package/zfs/request-to-openzfs-kernel-device/src/compatibility_check/`:
  `btf_header.rs` 354 lines,
  `btf_types.rs` 242,
  `btf_entry.rs` 225,
  `btf_records.rs` 154 and `btf_decode.rs` 125,
  1,100 lines before tests,
  plus 422 lines of tests in the three `*_tests.rs` files beside them.
  It is unmerged,
  so this worktree has none of it,
  and the branch's own gate review left a seam still open
  (`doc/handover/cli-git-rust-implementation.md` lines 40756 to 40757:
  emptying the three decoder files leaves 14 errors in six files).
- Version:
  none;
  the model and walker would follow `include/uapi/linux/btf.h`
  and the kernel's own validation (`kernel/bpf/btf.c:5195-5325`).
- Screening:
  H1 to H6 pass by construction,
  and it is still ineligible.
  The skill admits a custom implementation only when every existing tool fails a named hard constraint,
  and no such constraint exists:
  the project answered on 2026-11-19 that our own parser and no second program is not a hard requirement.
  Conditional candidate,
  priced in the arithmetic so the record shows what the route would have cost,
  and outside the ranking.

### Screened together, and why each is not a candidate

- `zfsdev-rs`'s layout code hard-codes a layout and parses no BTF;
  the crate already reuses its codec and layout code,
  so it is a dependency of the binding's history,
  not a reader.
- `object`,
  `goblin`,
  `scroll`,
  `libelf`:
  ELF sections and byte streams,
  the building blocks of the in-house reader,
  not BTF type models.
  H5 fails for each.
- `trailofbits/btfparse`,
  `parttimenerd/btfparse` (its fork),
  `libabigail`:
  C++ builds,
  H3 fail;
  `libabigail` would also use libbpf's parser,
  which H3 already refuses.
- `cilium/ebpf`,
  `libbpfgo`,
  `gbpf`:
  Go,
  so no link path into a Rust binary,
  H3 fail.
- `coolbpf`:
  a C platform over libbpf whose README needs `elfutils-devel`,
  `gcc`,
  `clang` and `llvm`;
  H3 fail,
  and its parser would be libbpf's.
- `bpf-compatible` and `bpf-compatible-rs`:
  helpers that resolve BTF files from the btfhub archive;
  `bpf-compatible-rs` 0.1.3's `src/lib.rs` holds archive-path helpers and no type model,
  so the category fails.
- `aquasecurity/btfhub` and `btfhub-archive`:
  distribute BTF files,
  so they are data sources.
- `trailofbits/nate`:
  a Rust DWARF parser,
  not a BTF reader.
- `vock`:
  a Debian ITP for a tracer,
  a tool rather than a parser.
- The remaining class 1 rows are eBPF loaders,
  linkers,
  profilers,
  generators,
  tools and crates whose names happen to contain "btf";
  the design's listing tags each one.
- The remaining class 2 rows are tools,
  generators,
  databases and unrelated projects;
  the design's listing tags each one.

## Hard-gate outcomes

- `btf-rs` 2.0.1:
  H1 pass (split constructors and the consumer probe),
  H2 pass,
  H3 pass:
  source,
  an `x86_64` run,
  and an `aarch64` library build,
  with that target still unrun
  ([The `aarch64` build](#the-aarch64-build-and-what-it-does-not-show)),
  H4 pass,
  H5 pass,
  H6 pass on the archive's bytes against the tag.
  Finalist,
  validated on `x86_64`.
- `ebeepf` 0.1.0:
  H1 pass (split constructor and the probe),
  H2 pass on the BSD arm,
  H3 pass on source plus the same `aarch64` build,
  run unexercised,
  H4 pass,
  H5 pass,
  H6 pass on the archive's bytes against its commit.
  Finalist,
  validated on `x86_64`.
- `pahole` 1.32:
  H1 pass,
  H2 pass as a separate process,
  H3 not applicable as a library and recorded as a runtime requirement instead,
  H4 pass,
  H5 pass,
  H6 pass on the package recipe:
  `pahole 1:1.32-2` builds the signed tag `v1.32` with three patches,
  and the installed binary's signature and hash match the package.
  Finalist,
  validated at its output boundary and by its own suite,
  which ran here with eleven recorded skips and no failure,
  and its own CI path is inventoried and not run
  ([The upstream CI of `pahole` at `v1.32`](#the-upstream-ci-of-pahole-at-v132)).
- `bpftool` 7.8.0:
  H1 pass,
  H2 pass as a separate process,
  H3 not applicable as a library,
  H4 pass,
  H5 pass,
  H6 pass on the package recipe:
  `bpf 7.3.4-1` builds `tools/bpf/bpftool` from `linux-7.3.4.tar.xz`,
  whose sha256 and detached signature the recipe pins,
  and whose binary reports the bundled libbpf's version rather than the package's.
  Not a finalist:
  failed on relevant execution.
- `btfparse`,
  `aya-obj`,
  `btf`,
  `btfdump`,
  `padlock-dwarf`:
  H1 fail,
  exit at screening.
- `libbpf-rs`,
  `libbpf` through FFI,
  `trailofbits/btfparse`,
  `parttimenerd/btfparse`,
  `libabigail`,
  `coolbpf`,
  `cilium/ebpf`,
  `libbpfgo`,
  `gbpf`:
  H3 fail,
  exit at screening.
- The in-house reader:
  H1 to H6 pass by construction and excluded by the skill's existing-tools rule.
  Not a gate failure,
  and not a finalist.

No candidate passed a hard gate on metadata alone:
every finalist's gate rests on a source path and,
for the two libraries,
on a run.
For the two programs the provenance gate now rests on the packaging recipe that built the installed
binary,
which maps each binary to a named source archive or tag:
`bpftool` to the kernel tarball `7.3.4` with its own sha256 and signature,
and `pahole` to the signed tag `v1.32` with three recorded patches.
The two mappings are equally tight on the question the gate asks,
which is whether each binary maps to a named source revision,
and neither is a reproducible-build proof:
`pahole`'s embedded libbpf is the submodule commit the `v1.32` tag records,
`dd92bef7f6c7`,
libbpf 1.8.0,
which the recipe's `git submodule update` checks out.
The gate after it is relevant execution,
and it decides the two programs differently:
`pahole`'s suite ran here and it stays a finalist while the project answers question 4 that way,
and `bpftool`'s fails:
its functional tests need root,
a BPF-capable kernel and a VM harness,
as the report reads them and not as a read of the files.
[The two programs' suites](#the-two-programs-suites) records both.

## Host facts

This report re-measured the host read-only at 2026-11-20T18:16:26-05:00,
so the validation evidence below describes today's module and not a file that has since been replaced:

- Kernel `7.3.5-1-cachyos`,
  architecture `x86_64` (`uname --kernel-release --machine`).
- ZFS module version `2.4.4-1`,
  `srcversion` `A3553C308C29FD4B6595A72` (`cat /sys/module/zfs/version /sys/module/zfs/srcversion`).
- `/sys/kernel/btf/vmlinux`,
  6,579,400 bytes,
  SHA-256 `dbf55cc7d4a2027c494cd7fed706ee751d26fe4f5739999d65161d3a1714f17c`,
  modification time 2026-11-20T14:34:52-05:00.
- `/sys/kernel/btf/zfs`,
  570,234 bytes,
  SHA-256 `edb8433af826ea7f5a9fdfb92aa13cd00f0e1f455a3637608e0f00c98cefd509`,
  modification time 2026-11-20T14:34:52-05:00.
- Both files are mode `0444`,
  `-r--r--r--`,
  read by every user (`stat --format='%A %a'` in the second revision).
  A spawned reader therefore needs no privilege to read them,
  which is the fact C3's premise missed in the first revision
  ([C3](#c3-privilege-surface-in-the-root-handler-weight-1-higher-is-smaller)).
- Both hashes equal the ones the validation round recorded for its read-only copies
  (`keep/evidence/checksums.txt` in each round's kept folder),
  so the runs parsed the same bytes this host serves now.
- Boot time 2026-11-19T11:12:11-05:00,
  so no reload separates the runs from this reading.
- The host kernel is not the product's qualified release until step 23 of the implementation plan,
  so these facts qualify the evidence,
  not the product.

The compared set,
as every run reports it:
both files' 32-byte header,
the base's 151,181 types,
the module's 13,141 own types numbered 151,182 to 164,322,
`zfs_cmd` at 13,744 bytes with 32 members,
the anonymous union at 106,368 bits with `zc_inject_record` and its anonymous struct,
`dmu_objset_stats` at 288 bytes with `dds_flags` at 2,296 bits,
`zinject_record` at 368 bytes with 19 members,
and the 104 `zfs_ioc` enumerators with the seven values the design names.

## Execution manifests

This report ran no third-party code.
The four manifests below describe rounds run by earlier sessions in this worktree;
each is reconstructed from the kept logs' own headers,
which record the command line,
start time,
elapsed milliseconds and exit status per step.

### Manifest: `btf-rs` 2.0.1

- Candidate and revision:
  `btf-rs` 2.0.1,
  archive SHA-256 `fa34ec91...f7963b`,
  clone origin `retis-org/btf-rs` at tag `v2.0.1`,
  `HEAD` `03fd01bd9505d0888b568cd3aaa9b232b7e3fb20`.
- Clone policy:
  `gh repo clone ... -- --branch v2.0.1 --depth 1`,
  then `git remote set-url --push origin DISABLED` before any other command.
- Top-level command per step:
  `cargo` from the pinned toolchain,
  exactly as each log's `# podman:` line records it,
  for example `cargo test --locked --offline --verbose --all-features`.
- Statically reached commands beyond `cargo`:
  `rustc`,
  `rustdoc`,
  `rustfmt`,
  `clippy-driver` from the same toolchain;
  a C compiler for the `zstd` and `xz` optional dependencies when `elf-compression` is on;
  no lifecycle script,
  no network client,
  no root operation.
- Files inspected to establish the tree:
  `Cargo.toml`,
  `Cargo.lock`,
  `.github/workflows/ci.yaml`,
  `btf-rs/tests/integration_test.rs`,
  `btf-rs/tests/assets/`.
- Image:
  `docker.io/cachyos/cachyos:latest` at `sha256:ab3e1698b7ce5eaa4f6342d7a0f687ce731c443706238a49338f46239036a185`,
  `podman` 6.2.0.
- Ceilings:
  `--cpus 2`,
  `--memory 2g`,
  `--pids-limit 512`,
  `--rm`.
- Credentials,
  environment,
  home:
  no ambient credentials,
  `HOME=/work/home` inside the container with no host home mounted,
  `--userns=keep-id`,
  the BTF copies mounted read-only at `/btf`.
- Network:
  on for one fetch step (`rustup` 1.29.3 installing stable 1.100.0 with `rustfmt` and `clippy`,
  then `cargo fetch`),
  off for every build and test step with `--network none` and `CARGO_NET_OFFLINE=true`.
- Expected reads and writes:
  the mounted tree and the read-only BTF copies;
  writes confined to the scratch tree,
  the cargo home and the toolchain home.
- Success,
  failure,
  cleanup and stop:
  each step's exit status compared with an expected one;
  the container removes itself;
  the driver stops on a step failure after recording it.
- Undeclared discovery:
  the published archive ships no tests,
  which the manifest did not expect,
  so the suite's run was moved to the repository at the same commit
  and the manifest was updated in the round's own record before the suite ran.
  Nothing else appeared.

### Manifest: `ebeepf` 0.1.0

- Candidate and revision:
  `ebeepf` 0.1.0,
  archive SHA-256 `f5f41620...5b6fa96`,
  clone origin `kxxt/ebeepf` at `HEAD` `5de05465bd9445d89b13a967e198d0aeffd71ec4`,
  push URL disabled.
- Top-level command per step:
  `cargo test`,
  `cargo fmt --check`,
  `cargo clippy --lib -- -D warnings`,
  and `cargo run` for the consumer,
  each recorded in its log's `# podman:` line.
- Statically reached commands beyond `cargo`:
  `rustc`,
  `rustfmt`,
  `clippy-driver`,
  and `clang` plus `llvm` for the crate's BPF fixtures.
- Image:
  `localhost/ebeepf-validation:rust-1.100.0-clang` at
  `sha256:d201d863b813804a9bc89db027eb51ed6f9839c4d8d363a2ab77e1363bed2a05`,
  built for this round from the same CachyOS base digest plus the distribution's Rust 1.100.0 and clang.
- Ceilings,
  credentials,
  home,
  mounts and the network policy:
  the same as the `btf-rs` manifest.
  Deviation recorded:
  the `aarch64` fetch attempt ran with `--network host` because it needed the distribution host,
  and that attempt failed (section "Validation results" records the exit and the error).
- Undeclared discovery:
  the suite needs `clang` for its BPF fixtures.
  Without it 94 unit tests pass and 8 fail at `src/test_bpf.rs:53`.
  The manifest was rewritten to build a clang image before the suite ran,
  and the without-clang runs are kept as failures.

### Manifest: `pahole` 1.32 and `bpftool` 7.8.0

- Candidates and revisions:
  `/usr/bin/pahole` 1.32,
  the distribution package `pahole 1:1.32-2`,
  SHA-256 `00acbd859cd0cca6e71dca93d885ed2483c5a52a78d3df1a9bcc9da82881503c`;
  `/usr/bin/bpftool` 7.8.0 on libbpf 1.8 by its own report,
  the distribution package `bpf 7.3.4-1`,
  SHA-256 `475a53182f50231c40e40ce002f2e032dc418489d56a2f8f902aa317aed03d20`.
  Both are the distribution's packages,
  so the provenance question is the package's and the release-version matrix is owed.
- Top-level command:
  `pahole --btf_base ... -C zfs_cmd|dmu_objset_stats|zinject_record`,
  `pahole --btf_base ... -E -C zfs_ioc`,
  and `bpftool btf dump file ... --base-btf ... format raw`,
  all recorded in `keep/evidence/tool-harness-invocations.txt` with their exit status.
- Environment:
  the host,
  not a container;
  no ceiling,
  because these are the distribution's binaries reading two read-only copies and writing only their own stdout.
  The harness recorded each invocation's exit status.
- Undeclared discovery:
  none;
  the harness observed its six invocations,
  four for `pahole` and two for `bpftool`,
  plus both negative controls.
- Limits of this manifest,
  stated because the skill asks for one:
  the tool route's own product arms,
  owed measurements 13 and 14,
  and the release matrix are unrun,
  so this manifest validates the tools at their output boundary and nothing beyond it.

### Manifest: this report's own runs

- Fingerprint script:
  `node compute-the-vet-report-fingerprint.ts` in `~/temp/agent/zfs-layout-checker-reader-vet-report-2026-11-20/keep` (scratch,
  deleted on 2026-11-27;
  not kept),
  exit 0,
  printed the canonical JSON and the hash;
  the second run at 2026-11-20T18:16:26-05:00 printed the same hash.
- Registry corroboration:
  the `curl` call in section "Query schedule",
  written at 2026-11-20T18:08:38-05:00,
  HTTP 200,
  76,457 bytes kept.
- Host reads:
  `uname`,
  `cat` of the two identity files,
  `stat` and `sha256sum` of the four files,
  `uptime --since`,
  the same readings re-run at 2026-11-20T18:16:26-05:00,
  exit 0,
  output in section "Host facts".
- Report write:
  the lock script under `keep/`,
  create-new,
  exit 0,
  and its record kept.
- Verification:
  `monochromatic-lint` on the two committed files and `pandoc --from=gfm` on this report,
  both after the write,
  with the lint findings filtered to the lines `git diff main...HEAD` adds,
  and both recorded in this session's final report.
- Nothing here fetched,
  built,
  installed or ran a candidate.

### Manifest: the second revision's own reads

The second revision fetched,
measured and read,
and ran no candidate:

- `gh api` GET requests,
  read only,
  authenticated as this machine's user,
  against four repositories:
  `repos/retis-org/btf-rs`,
  `repos/kxxt/ebeepf`,
  `repos/acmel/dwarves` and `repos/libbpf/bpftool`,
  plus their `contributors`,
  `issues`,
  `pulls`,
  `commits` and `releases` endpoints,
  and one detail read per sampled issue or pull request with its comments and reviews.
  Responses are kept as JSON under the second revision's `keep/gh/`.
- `curl` of three published crates.io archives,
  `btf-rs` 2.0.1,
  `btf-rs-derive` 2.0.1 and `ebeepf` 0.1.0,
  from `https://static.crates.io/`,
  each SHA-256 compared with the report's recorded value.
- Three read-only clones at depth 1,
  `gh repo clone ... -- --depth 1`:
  `acmel/dwarves`,
  `libbpf/bpftool` and `libbpf/libbpf`,
  plus `retis-org/btf-rs` at tag `v2.0.1` for the byte diff.
  The `dwarves` submodule `lib/bpf` was initialised to count the vendored libbpf's parser.
- Local reads only:
  `find ... | xargs wc -l`,
  `rg --count-matches '\bunsafe\b'`,
  `diff -r --brief`,
  `pacman --query --owns`,
  `pacman --query --info`,
  `readelf --dynamic`,
  `stat`.
- Ceilings:
  none needed.
  No candidate was executed,
  nothing was built,
  and the network reads are a few hundred API responses and three
  crate archives.
- Everything under
  `~/temp/agent/zfs-layout-checker-reader-vet-report-gate-review-findings-fixed-2026-11-20/scratch/` (scratch,
  deleted on 2026-11-27;
  not kept),
  with the kept reports,
  responses and measurements under its `keep/`.

### Manifest: the third revision's own runs

The third revision fetched,
measured and ran,
and executed one candidate's suite in a container:

- Reads and measurements on this host,
  read only:
  `sha256sum` and `readelf --dynamic` on `/usr/bin/pahole` and `/usr/bin/bpftool`,
  `pacman --query --info pahole bpf`,
  and the line counts and file lists of the two libraries' published archives.
- Three read-only `curl` downloads from `https://static.crates.io/`:
  `btf-rs` 2.0.1,
  `btf-rs-derive` 2.0.1 and `ebeepf` 0.1.0,
  each hashed and compared with the report's recorded value,
  each extracted under the third revision's scratch folder.
- Five read-only `curl` fetches from `raw.githubusercontent.com/gregkh/linux/v7.3.4`:
  `tools/bpf/bpftool/btf.c`,
  `tools/bpf/bpftool/main.c`,
  `tools/bpf/bpftool/Makefile`,
  `tools/lib/bpf/btf.c` and
  `tools/lib/bpf/libbpf_version.h`.
- Four read-only `gh repo clone ... -- --depth 1` clones:
  `acmel/dwarves` at its default branch,
  `acmel/dwarves` at tag `v1.32`,
  `libbpf/libbpf` at its default branch,
  and the three patch files from the CachyOS packaging repository by `curl`.
- Five container starts of the candidate's suite,
  the first exiting 127 on a missing `tests` and running no suite,
  the last kept,
  and every one of them a single `podman run` with `--network host`:
  image `docker.io/cachyos/cachyos:latest`,
  whose tag carries no digest;
  the gate final check's rerun committed its package layer and ran that image,
  `sha256:e5d458da6220679f3f3aab3ec77dbd8884dffeebba6671a19532323dd99ccb8d`,
  `Id` `427517f8262bf1a0f3377862596fa964faa0968cf9edf6cf069435dab47c56d7`,
  built from the local base the check recorded as `sha256:57e7303ae41c…`,
  truncated there,
  `--cpus 2`,
  `--memory 4g`,
  `--pids-limit 1024`,
  no home mount,
  the source and the patches mounted read-only,
  and the suite's perf-tarball fetch bounded by pointing the proxy variables at a closed port,
  so that it failed instead of downloading.
  Records and build ran in the same container,
  so the run is not the skill's default of a fetch phase
  followed by a network-free one;
  the gate final check ran that form at the pinned libbpf and got the same cases.
  Ceilings are the container's;
  the driver is
  `keep/build-and-run-the-pahole-suite-in-a-container-5.ts`
  and the log is its `-output-5.txt`,
  both under the third revision's `keep/`.
  The one undeclared network endpoint the run revealed,
  the kernel.org perf mirror,
  was found,
  bounded and recorded rather than allowed.
- Three script runs with `node`:
  the matrix over the remaining finalists and the recorded arms,
  the crate-archive measurement,
  and the container driver,
  each with its output kept.
- Everything under
  `~/temp/agent/zfs-layout-checker-reader-vet-report-recheck-findings-fixed-2026-11-20/scratch/` (scratch,
  deleted on 2026-11-27;
  not kept),
  with the kept reports,
  measurements,
  logs and responses under its `keep/`.

## Validation results

Equal depth is the claim being checked:
every finalist got the same three layers where they apply,
namely its own upstream build and test path,
its own suite,
and the consumer boundary that the product would use.

### `btf-rs` 2.0.1, validated on `x86_64` Linux

- CI-equivalent path,
  run from the repository at tag `v2.0.1` because the archive ships no tests:
  `cargo fmt --check`;
  `cargo clippy --locked --all-features -- -D warnings` for the library and the example;
  `cargo doc --locked --all-features` under `RUSTDOCFLAGS=-D warnings`;
  six feature builds;
  `cargo test --locked --all-features`.
  Every step exit 0;
  the test step's own elapsed time is 14,982 ms (`ci-cargo-test-all-features.log:1-4`),
  and its suite reports 168 integration tests and four doctests passed in 11.16 s (`:183`,
  `:197`),
  with no `#[ignore]` and no skipped case.
- Default features:
  64 integration tests and four doctests passed in a 3,166 ms step (`default-features-cargo-test.log:1-4`,
  `:79`,
  `:93`).
  So 104 cases need the features the CI enables,
  which is a fact about the suite's reach and not a failure.
- The published archive:
  15 files,
  `autotests = false`,
  `include` holding only `examples/`,
  `src/` and `LICENSE`,
  so `cargo test` there runs four doctests that only compile (`published-crate-archive-cargo-test-all-features.log:19`).
  Its `.cargo_vcs_info.json` names `03fd01bd9505...`,
  which is the tag's commit,
  so the archive maps to the source whose suite passed:
  that is provenance under H6,
  and it is the reason the archive's missing tests are recorded rather than scored.
  The second revision checked the claim on the bytes rather than on that file:
  `diff -r --brief` of the archive's `src/` against the tag's printed nothing,
  and the same holds for `LICENSE` and `examples/`
  (section "Discovery ledger").
- Consumer boundary:
  a program depending on the library by path,
  reading bounded buffers and calling `Btf::from_bytes` for the base and `Btf::from_split_bytes` for the module,
  which is the loader route the product designs.
  183 checks,
  0 failures,
  exit 0,
  731 ms (`design-consumer-check-on-host-btf-copies.log:1-4`,
  `SUMMARY passed=183 failed=0`).
  Negative controls refused with their own errors,
  no panic and no stall:
  a base truncated to 1,024 bytes,
  a module truncated the same way,
  a module whose magic byte changed,
  and a split object passed as a base.
  The first run of this check failed,
  and the failure is recorded here as unexplained rather than passed over:
  `summary.txt` in that round's kept evidence reads
  `STEP FAIL design-consumer-check-on-host-btf-copies exit=101 elapsed-ms=3690`,
  and the log was overwritten by the passing re-run above,
  so the cause is not recoverable from anything kept.
  What is known is that the consumer program or its manifest changed between the two runs,
  since the same source and the same two files give exit 0 in 731 ms on the re-run;
  the re-run's own command line is the one in the log.
  A 3,690 ms step is longer than the passing run,
  which is consistent with a first-time compile failing rather than with a parse failure,
  but that is an inference and not a record.
- Omits `aarch64` as a run:
  the gate review built the library for the target without `rustup`
  and the second revision cites that build
  ([The `aarch64` build](#the-aarch64-build-and-what-it-does-not-show));
  no run happened on any machine here.
- No fuzz target exists in the crate,
  and none was run;
  the bounded pre-scan and the negative controls are what the validation has instead (source quality record,
  section "Evidence for the scores").

### `ebeepf` 0.1.0, validated on `x86_64` Linux

- Archive checked against the registry first:
  the version entry's `checksum` equals the downloaded archive's SHA-256,
  and `.cargo_vcs_info.json` names the clone's `HEAD`.
- Suite,
  on the archive:
  `cargo test` ran 102 unit tests,
  `kernel_smoke` with 1 passed and 22 ignored,
  and 1 doctest,
  all green,
  in a 6,967 ms step with the suite at 6.43 s (`archive-cargo-test-default-with-clang.log:1-4`,
  `:113`,
  `:146`,
  `:152`).
  `cargo fmt --check` clean;
  `cargo clippy --lib -- -D warnings` clean with one manifest warning about a redundant `homepage`.
  The repository's own workspace adds a `bpfcov` member whose `libbpf-rs` dependency does not resolve offline,
  so the archive is the step and the repository's workspace is recorded as unusable there.
- The 22 omitted `kernel_smoke` cases,
  recorded as the skill asks:
  exact command `cargo test`,
  run from the archive's root,
  the same step as the suite above;
  purpose,
  to load BPF programs and maps through the BPF syscall;
  each one's name carries its own requirement,
  and every one of the 22 needs root or `CAP_BPF` and a kernel feature,
  for example
  `opens_kernel_module_split_btf` ("requires root or CAP_BPF and a loaded kernel module exposing BTF")
  and `reads_program_output_streams` ("requires root or CAP_BPF and a kernel with BPF program streams"),
  as the log prints them (`archive-cargo-test-default-with-clang.log:122-143`).
  The run had neither root nor a capability set,
  so none could run there.
  None can affect the surface this report claims:
  that surface is reading two files and comparing their types,
  hard constraint H4 excludes the BPF syscall from the reader path outright,
  and the one of the 22 that touches split BTF,
  `opens_kernel_module_split_btf`,
  reaches it through the loader's syscall path rather than through the file parse.
  The consumer probe above is what covers the split parse on this host's pair.
- Consumer boundary:
  the same program shape,
  on the same copies.
  196 checks,
  195 passed,
  1 failed,
  exit 1,
  478 ms (`design-consumer-check-on-host-btf-copies-with-clang.log:1-4`,
  `:236-237`).
  The failure is a negative control:
  a module passed as the base of another split parse is accepted instead of refused,
  which is the residual the source audit found at `src/btf.rs:411-419`.
  The other controls passed.
- Omits the `aarch64` run,
  and the build is closed separately:
  the image's Rust ships no `aarch64` standard library,
  and the `rustup` route failed,
  with
  `error sending request for url`,
  the URL being `https://static.rust-lang.org/dist/channel-rust-stable.toml.sha256`,
  and `client error (Connect): tcp connect error: Connection timed out (os error 110)`
  exit 1,
  32,633 ms (`fetch-rustup-toolchain-and-aarch64-target.log:1-28`).
  In the same container `curl` reached the rustup installer host,
  so this is the distribution host's connect path and not the container's network policy.
  It happened twice.
  The gate review closed the build half without `rustup`,
  and the second revision cites that
  ([The `aarch64` build](#the-aarch64-build-and-what-it-does-not-show));
  the run stays unavailable.
- No fuzz target exists in the crate,
  and none was run.

### `pahole` 1.32 and `bpftool` 7.8.0, validated at their output boundary

- The harness ran each tool on the same two copies and checked the same members and enumerators the design compares:
  323 checks,
  0 failures (`keep/evidence/tool-harness-checks.txt`).
- Negative controls:
  `pahole` on the module without a base exits 1 with libbpf's "did you forget to provide base BTF" message;
  `bpftool` on the same file exits 234 with "Invalid argument".
- Output facts that bear on C4:
  `pahole` prints absolute byte offsets at every nesting depth,
  so its anonymous containers need no path arithmetic;
  `bpftool` prints `bits_offset` relative to the containing type,
  renders an empty BTF name as `(anon)`,
  and prints no member size,
  so a consumer must resolve each `type_id` through both dumps.
  The harness does that,
  and that resolver is the cost C4 prices.
- Omitted `aarch64`:
  the host has no `aarch64` environment,
  so both binaries are `x86_64` evidence only.

### Provenance of the two programs

The second revision left this gate open,
writing that "neither maps the shipped binary to a source revision";
the gate re-check closed it from the distribution's recipes,
and the third revision read those recipes again and records them here.

- `bpftool`:
  `/usr/bin/bpftool` is Arch's `bpf 7.3.4-1` split package,
  built 2026-11-12 by the `linux-tools` PKGBUILD at its tag `7.3.4-1`
  (`pkgver=7.3.4`,
  `pkgrel=1`)
  from `linux-7.3.4.tar.xz`,
  sha256 `b11b6df8295e8e991ff040b71aaa4994a8b55caa23b6b092907f195b5840fd1f`,
  beside a PGP signature in the recipe's source list.
  The package's `build()` enters `tools/bpf` and runs `make -C bpftool all doc`,
  so the binary comes from the kernel tree at `7.3.4` and not from the standalone
  `libbpf/bpftool` repository.
  Its installed SHA-256 is `475a5318...aed03d20`,
  and `pacman --query --check --check bpf` reports 0 altered files.
- `pahole`:
  `/usr/bin/pahole` is CachyOS's `pahole 1:1.32-2` package,
  built 2026-11-02,
  and its recipe has `pkgver=1.32` and `pkgrel=2`,
  which is the installed release.
  It builds the signed tag `v1.32` with three patches:
  `0001` for the CMake install of `ostra.py`,
  `0002` and `0003` in `dwarf_loader.c` for merged LTO CUs and kept CUs,
  and the second and third also add two test scripts.
  So the binary is not upstream `v1.32` byte for byte,
  and the report says so rather than calling it upstream.
  Its installed SHA-256 is `00acbd85...281503c`,
  and `pacman --query --check --check pahole` reports 0 altered files.
  The recipe builds libbpf in with `LIBBPF_EMBEDDED=ON`,
  and its `prepare()` fetches libbpf by `git submodule` alone:
  `git submodule init`,
  `git submodule set-url lib/bpf`,
  and `git submodule update` with no `--remote`,
  which checks out "the commit recorded in the superproject" (`git-submodule(1)`).
  So the embedded libbpf is the submodule entry the `v1.32` tag records,
  `dd92bef7f6c7a00bb0312554119b6d9cf38e4f32`,
  libbpf's commit message `Makefile: bump to v1.8.0`,
  dated 2026-04-09,
  which is libbpf 1.8.0;
  the gate final check proved the mechanism on a throwaway pair of repositories and read the entry from
  the tag,
  and both records are kept beside its review.
  What the report previously called an unpinned branch was the recipe's source line,
  not the checkout
  it produces.

The version string the second revision explained wrongly:

- It said the string "comes from `BPFTOOL_VERSION`,
  which the build sets from the kernel tree it
  was built in (`src/Makefile:80-81`)".
  The Makefile only passes `-DBPFTOOL_VERSION` through when the variable is already set
  (`tools/bpf/bpftool/Makefile:82-83` at `v7.3.4`,
  the same at `v7.3`),
  and the recipe sets nothing.
  What the binary prints is the other branch:
  `tools/bpf/bpftool/main.c:98-106` defines the major as the bundled libbpf's major plus 6,
  the minor as libbpf's minor and the patch as 0 when `BPFTOOL_VERSION` is unset,
  and `:180-184` prints that as `%s v%d.%d.%d`.
  `tools/lib/bpf/libbpf_version.h` at `v7.3.4` declares 1.8,
  so the printed version is `7.8.0`.
  A read of the shipped binary supports that path rather than a compiled-in literal:
  `strings` finds `using libbpf %s` and no `7.8.0` string.
  So the self-reported version carries the bundled libbpf's version.
  It is not "nothing";
  it names a libbpf line and not the distribution package,
  and the practical point stands:
  a version check on the tool proves nothing about which package shipped it.

The source the second revision audited is not the source the package builds:

- It printed `bpftool`'s `src/btf.c` at 1,895 lines and libbpf's at 6,780.
  Those are the standalone `libbpf/bpftool` mirror and the `dwarves` submodule.
  The package builds the kernel tree's copies,
  which at `v7.3.4` are `tools/bpf/bpftool/btf.c` at 1,562 lines and
  `tools/lib/bpf/btf.c` at 6,376 lines,
  read from the stable tag and kept beside this report's measurements.
  None of the scores moves on the correction,
  and the C8 figure the report prints is now the packaged tree's.

Both mappings are a provenance gate the skill accepts:
a named source archive with its checksum and signature for `bpftool`,
a signed tag plus three recorded patches and a recorded submodule commit for `pahole`.
Neither is a reproducible-build proof,
and neither recipe pins compiler flags for a byte-identical rebuild;
the gate is source mapping,
which is what H6 asks,
and the reasons each mapping stops where it does are above.
One thing the mapping does not carry is the libbpf the kept suite run actually built:
that run copied a fresh clone of libbpf's default branch over `lib/bpf`,
which the log shows as ` M lib/bpf`,
so it tested the packaged parser against a libbpf the package does not ship with.
The gate final check reran the same suite at `dd92bef7f6c7` and got the same cases,
as [The two programs' suites](#the-two-programs-suites) records.

### The two programs' suites

The skill's finalist validation requires every non-default suite relevant to the claimed behaviour,
and it fails a candidate whose relevant suite cannot be inspected or run.
The third revision took each program in turn.

`pahole`:

- The suite is 76 numbered cases over the shell scripts under `tests/` at tag `v1.32`:
  74 scripts at the tag plus the two the recipe patches add,
  with one shared `test_lib.sh` beside them,
  driven by `tests/tests`,
  which needs the build directory at the front of `PATH` and writes under `/tmp/pahole-tests` inside
  the container.
  The recipe's own `check()` runs `ctest`,
  and the build registers no tests:
  the third revision ran it and it printed "No tests were found!!!"
  and exited 0,
  so the distribution's build tests nothing about `pahole`.
  That is a fact about the packaging,
  not about the project,
  and it is why the suite below was run by hand.
- The run:
  the signed tag `v1.32` at `fa725d687c`,
  the three recipe patches applied cleanly,
  built with CMake and Ninja,
  and the suite run in a container
  (`docker.io/cachyos/cachyos:latest`,
  `--cpus 2`,
  `--memory 4g`,
  `--pids-limit 1024`,
  no home mount,
  a read-only mount of the source):
  76 cases,
  65 `Ok`,
  11 `Skip`,
  0 failures,
  exit 0.
  The log is `keep/build-and-run-the-pahole-suite-in-a-container-output-5.txt`,
  and the driver that ran it is part of the same folder.
  The libbpf built into this run is not the one the package ships:
  the driver copied a fresh clone of libbpf's default branch over `lib/bpf`,
  so this run tested the project's own sources against another libbpf.
  The gate final check reran the same suite at the commit the `v1.32` tag records,
  `dd92bef7f6c7`,
  in two phases with `--network none` for the build and the run,
  and got the same result case by case:
  76 cases,
  65 `Ok`,
  11 `Skip`,
  0 failures,
  exit 0,
  with only `lo` inside the container and no name resolution
  (`keep/rerun-the-pahole-suite-offline-with-the-recorded-libbpf-output.txt`).
- The eleven skips,
  each with its reason and its distance from the consumed surface:
  - 15,
    22,
    52 and 72 need a `vmlinux` with DWARF,
    which this host does not have and this container
    does not build.
    That is a choice and not an impossibility:
    `pahole`'s own CI builds the kernel that supplies one,
    and [The upstream CI of `pahole` at `v1.32`](#the-upstream-ci-of-pahole-at-v132) records that path
    as not run;
  - 18,
    the split-BTF test,
    needs `debuginfod-find` plus debuginfo for a kernel module loaded on the running host
    (`tests/btf_split.sh:22`,
    `:33`,
    `:65`),
    so the split *write* path and its `pfunct --btf_base` read are not exercised end to end here;
  - 45 needs `dwz` for an alternate debug file,
    which the distribution ships;
  - 56 needs `ld.lld` for LTO language filtering,
    which the distribution ships too,
    and which is a test one of the three recipe patches adds;
  - 54 needs a fixture whose `pfunct --all --format_path=btf` shows the optimizer's `foo.` clone,
    and `tests/gcc_true_signatures.sh:48` compiles that fixture with `-O2` itself,
    so the skip is the fixture's and not the container's build type;
  - 55,
    69 and 70 try to download a perf tarball from
    `https://mirrors.edge.kernel.org/pub/linux/kernel/tools/perf/`.
    That fetch is the suite's own undeclared network endpoint:
    the run kept `--network host` throughout and the driver pointed the proxy variables at a closed
    port,
    which bounded the fetch rather than isolating the run and left the three cases skipping on the
    failed download.
    The gate final check's rerun above is the offline form of the same run,
    and it skips the same three.
  - What the cases that ran cover on the consumed route:
    cases 9,
    11 and 14 read one BTF file back through `pahole`;
    case 12 encodes with a base (`pahole -J --btf_base=...` at
    `tests/btf_distilled_base.sh:98` and `:124`) and checks the result as ELF sections and through
    `bpftool btf dump`,
    so it exercises the encoder's base handling and not the printer's split read;
    17,
    19,
    20,
    21 and 23 encode and then check with `bpftool` or with logs.
    Case 18 is the only case at the tag that reads a split BTF back through the printer
    (`tests/btf_split.sh:153`,
    `pfunct --btf_base="$base_btf" --all "$split_btf"`),
    and it skipped.
  So the relevant suite ran with eleven skips,
  and one of the eleven is the upstream test of the consumed surface itself.
  The other ten are recorded with their reasons;
  four of them are the CI path's and stay open,
  cases 15,
  22,
  52 and 72 (`vmtest.yml` above);
  case 18's evidence is the opposite and is the decision below.

`bpftool`:

- Its repository CI has build,
  lint,
  docker,
  static-build and release workflows and no functional test job.
  Its functional tests are the kernel's `tools/testing/selftests/bpf`,
  and the suite's `v7.3.4` tree carries four `bpftool` files.
  Two are rootless:
  `test_bpftool_build.sh` builds `tools/bpf/bpftool` from a kernel tree,
  and `test_bpftool_synctypes.py` compares the tree's headers,
  documentation and bash completion,
  and a search of both for `geteuid`,
  `sudo`,
  `root`,
  `bpf(` and `syscall` finds nothing.
  Both need a kernel source tree,
  which this round did not have.
  Two sit under `prog_tests/`,
  `bpftool_maps_access.c` and `bpftool_metadata.c`,
  and the report read them as needing root and a BPF-capable kernel;
  the gate final check did not read those two and says so,
  so that reading is carried here and not verified.
  No run here took those privileges,
  and a container's kernel is this host's kernel;
  a throwaway-VM route exists on this host and the report did not try it.
  So the reading is "did not run here",
  and the two rootless scripts are named rather than folded into it.
- Read strictly,
  the skill fails the candidate:
  a relevant suite that cannot be inspected or run fails it,
  and duration or a mirror's convenience does not justify omission.
  This report therefore fails `bpftool`
  and continues with the other survivors,
  which is what the skill's "relevant execution unavailable" terminal rule says to do when others
  remain.

What that leaves:

- `pahole` remains a finalist only if the project accepts the omission it carries.
  Its suite ran with recorded skips,
  and its output boundary was validated earlier with 323 checks and two negative controls.
  The skill asks for evidence that every omitted case cannot affect a claimed or consumed surface,
  and for case 18 the report does not have that kind of evidence:
  the case is the upstream test of the surface itself.
  What it has is the harness,
  323 checks running `pahole --btf_base` over this host's own pair,
  which is a consumer-layer check on one kernel and not the upstream suite.
  The report offers that harness as the omission evidence for case 18,
  as it already does for `ebeepf`'s `opens_kernel_module_split_btf`,
  says what the evidence does not prove,
  and puts the decision to the project in question 4.
- `bpftool` is failed,
  and its scores are no longer used for any result.
  The arm that keeps it is printed in [Sensitivity](#sensitivity),
  so the project can see what returning it would change:
  it adds the `pahole` and `bpftool` pair,
  whose thirteen rating-step ties no evidence converts into an order,
  and it changes nothing about the first place.
- The C3 premise the first revision used is wrong,
  and it moves here with the whole program discussion:
  it rated both programs' privilege surface 1 with high confidence on the premise that the child runs
  as root.
  Both BTF files are mode `0444` on this host,
  so a child could read them unprivileged,
  sandboxed and under a hard resource and wall-clock limit,
  which a library in the handler's own address space cannot be.
  Nothing in the product's design requires the child to be privileged;
  the root handler can drop to an unprivileged user for the child.
  That is why the confidence is medium now and the rating prices the unshrunken shape only,
  and it is what question 5 of [Open questions for the project](#open-questions-for-the-project)
  puts to the project.

### The upstream CI of `pahole` at `v1.32`

The skill's first step before running a finalist is "inventory every upstream task and CI job",
and the first run is "the complete default CI-equivalent path".
Neither was done for `pahole` before this section,
which records what the tag holds and what was done with each,
with two of the skill's three fields for every omission,
purpose and evidence that the omitted suite cannot reach the consumed surface,
and the exact command only for `vmtest.yml`'s scripts.

`.github/workflows/` at the tag holds five workflows,
and the gate final check lists all five;
the excerpt of the scripts it names is kept beside its review
(`keep/source-lines-in-the-pahole-v1.32-tag.txt`).

- `build.yml`,
  not run.
  It builds `pahole` on Debian with the default compiler,
  `gcc-12` and `clang`.
  Purpose:
  prove the sources build across those compilers and that distribution.
  Omission evidence:
  not met.
  The suite run above compiled the same sources with the default compiler,
  `gcc` 16.2.1,
  which is the only compiler its CMake log names
  ("The C compiler identification is GNU 16.2.1");
  `clang` was installed in that container and no kept log shows it compiling `pahole`'s sources,
  so one of this workflow's three compilers is covered
  and its `gcc-12` and `clang` builds are not,
  and the report has no record showing that another compiler or Debian changes the BTF parse.
  The omission stands recorded and not closed.
- `lint.yml`,
  not run.
  ShellCheck over the repository's shell scripts.
  Purpose:
  catch shell defects in the build and test scripts.
  Omission evidence:
  it reads script text and runs no binary,
  so it cannot reach the parser,
  the printer or the files the product would parse.
  This is the one omission of the five the report can close on evidence.
- `ondemand.yml`,
  not run,
  and the kept evidence does not record what its jobs are.
  Purpose and omission evidence:
  not recorded,
  because the report cannot say what it covers.
- `vmtest.yml`,
  not run.
  The reusable workflow `test.yml` calls,
  and the CI-equivalent path itself:
  it builds `pahole` (`build-pahole.sh`),
  builds a bpf-next kernel with it (`build-kernel.sh`),
  runs `tests/tests` against that kernel's `vmlinux` (`run-selftests.sh:8` and `:16`,
  `vmlinux=$VMLINUX ./tests`),
  and then compares the kernel's BTF functions (`compare-functions.sh`).
  Purpose:
  the upstream suite against a `vmlinux` this host does not have.
  Omission evidence:
  not met.
  It is the path that supplies what cases 15,
  22,
  52 and 72 skip for,
  and the project's own build script says so:
  `build-and-test-cmd.sh:11`,
  "Full vmlinux validation is tested in CI".
- `test.yml`,
  not run.
  The trigger:
  push,
  pull request and a daily schedule,
  and its job is to call `vmtest.yml` (`test.yml:44`).
  It has no suite the others do not,
  so its omission is `vmtest.yml`'s.

Two facts stand where a coverage sentence used to:

- The CI-equivalent path was not executed.
  It is a bpf-next kernel build with the built `pahole`,
  then `tests/tests` against that kernel's `vmlinux`,
  then `compare-functions.sh`.
  The skill says duration alone does not justify omission and points at resource isolation,
  and a capped container could run it;
  no round in this worktree did.
  Running it is expected to close cases 15,
  22,
  52 and 72 and the `compare-functions.sh` job,
  and would still leave case 18 open.
- Case 18 was not executed either,
  and the CI path would not execute it either.
  `tests/btf_split.sh` requires `debuginfod-find` and debuginfo for a kernel module loaded on the
  running host (`:22`,
  `:33`,
  `:65`),
  which the container does not have and a CI kernel build does not supply.
  Its read is the route the product would take:
  `pfunct --btf_base="$base_btf" --all "$split_btf"` at `:153`.

So `pahole` is a finalist on its output boundary,
its suite's other evidence and this record of what did not run,
and not on a closed relevant-execution gate.
Whether that is enough is question 4.

### The `aarch64` build and what it does not show

Both libraries now build for `aarch64-unknown-linux-gnu`,
and neither has run there.
The build is the gate review's,
run after the first revision called the target unrun,
in capped containers with no home mount and the network off:
`podman run --rm --network none --cpus 2 --memory 2g --pids-limit 512 --userns=keep-id`,
`HOME` inside the container and `CARGO_NET_OFFLINE=true`.

- Toolchain:
  the validation round's stable 1.100.0 plus the `rust-std-1.100.0-aarch64-unknown-linux-gnu` archive.
  The reviewer recorded that it took that archive from the channel manifest dated 2026-11-12 and that the
  archive's SHA-256 matches the manifest;
  the second revision did not repeat that fetch and does not rest a claim on the hash.
  `rustup` was not used,
  which is what the `ebeepf` round's failed fetch needed.
- `btf-rs` 2.0.1 archive:
  `cargo build --lib --locked --offline --target aarch64-unknown-linux-gnu`,
  exit 0,
  4,166 ms,
  `Finished \`dev\` profile ... in 3.96s`
  (`~/temp/agent/gate-review-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/aarch64-build-check-btf-rs-lib-build-aarch64-offline.log:1-4`).
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
- `ebeepf` 0.1.0 archive:
  the same command,
  exit 0,
  10,963 ms,
  `Finished \`dev\` profile ... in 10.75s`
  (`.../aarch64-build-check-ebeepf-lib-build-aarch64-offline.log:1-4`).
- No C compiler ran in either step;
  every dependency compiled from Rust source.
- The fetch step that needed the network is kept separately
  (`aarch64-build-check-fetch-ebeepf-dependencies-network-on.log`),
  and the driver that ran them is `aarch64-library-build-check-driver.ts` beside them.

What this closes:
H3's second target is now a build for both libraries,
not source reading alone.
What it does not close:
nothing here executed the `aarch64` artifacts,
so a target-specific miscompile would still be invisible,
and the two tools have no `aarch64` artifact at all.
The evidence is a cross-vendor reviewer's run that the second revision did not repeat;
its logs and driver are cited where they are kept,
and the report does not present them as its own measurements.

## Maintenance audit

The skill asks for a specific audit,
not an issue count,
and the first revision asserted its C5 ratings without keeping the records.
The second revision ran it for all four candidates,
on 2026-11-20,
through the repository hosts' API,
read only,
and kept every response.
The requests are the same shape for each repository:
`repos/<owner>/<name>`,
`.../contributors?per_page=100`,
`.../issues?since=2025-11-20T00:00:00Z&state=all&sort=updated&direction=desc&per_page=100`,
`.../pulls?state=all&sort=updated&direction=desc&per_page=10`,
`.../commits?since=2025-11-20T00:00:00Z&per_page=100`,
`.../releases?per_page=100`,
plus one detail read per sampled issue or pull request with its comments and its reviews.
Every response is at
`~/temp/agent/zfs-layout-checker-reader-vet-report-gate-review-findings-fixed-2026-11-20/scratch/keep/gh/` (scratch,
deleted on 2026-11-27;
what this record uses from it is stated here),
and the two scripts that collected them are kept in the same `keep/` folder,
as `collect-maintenance-records.ts` and `collect-maintainer-actions.ts`.
The sample rule is the skill's:
all issues when at most 20 exist in the window,
otherwise the 10 most recently updated.

### `btf-rs`

- Primary URL:
  `https://github.com/retis-org/btf-rs`,
  read 2026-11-20.
- Repository:
  created 2022-10-07,
  10 stars,
  4 forks,
  2 open items,
  last push 2026-11-12.
- Maintainer concentration:
  four contributors,
  118,
  31,
  18 commits and 3 for `dependabot`,
  so the lead author holds about 69 percent of them.
  Of the 73 commits in the window,
  `atenart` wrote 59,
  `amorenoz` 7,
  `vlrpl` 5 and `dependabot` 2.
- Issues in the window:
  none;
  the 14 rows the window returns are all pull requests.
  The two open items are older:
  issue 1 ("Add some BTF building capabilities") was last updated 2025-11-17,
  three days before the window opens,
  and pull request 37 ("Add kind layout support") last on 2026-10-26.
  So this is the skill's "active releases with weak public issue support" shape,
  and its issue history is too small to measure responsiveness from.
- Pull requests:
  ten of the 14 rows the window returns,
  sampled as the skill asks.
  Every merged one was reviewed and merged by a second maintainer,
  never by its author:
  `amorenoz` merged PR 42,
  35 and 34,
  `vlrpl` merged 41,
  38,
  39,
  40,
  36 and 33,
  and each carries a review by the merger.
  Two of them are external (`dependabot`,
  41 and 40),
  and both were reviewed and merged rather than merged unreviewed.
  Merge latency spans 0.5 days to 91 days,
  the long ones being the 2.0 release itself and the widened-mask work.
  Labels are used sparingly:
  the two dependency PRs carry `dependencies` and `rust`.
- Release record:
  five releases,
  `v2.0.1` on 2026-11-12 and `v2.0.0` on 2026-04-16 among them;
  the maintenance merges reach a release,
  with PR 38's widened masks the known exception,
  which is still open as PR 37.
- Hardening record:
  the window's commits include
  `obj: fix offset overflows` (2026-02-04,
  `381a38d8f1`),
  `obj: cap Vec len estimates` (2026-02-05,
  `bb8c673e01`),
  `obj: check header flags` (2026-02-05,
  `a75c5ddc67`),
  `obj: mmap: fix strings section validation` (2026-02-16,
  `529c3faa92`),
  `tests: add more invalid cases` (2026-02-05,
  `1a01ae7fd9`)
  and `section: mmap: fix oob access in resolve_name_by_offset` (2026-10-23,
  `64e2bdfa42`,
  merged as PR 42 on 2026-10-29).
  That record is maintenance evidence and also the reason C1 is not rated high:
  the crate needed overflow and out-of-bounds fixes this year.
- Classification:
  responsive maintainers with a small backlog,
  evaluated mainly through source,
  releases and tests.

### `ebeepf`

- Primary URL:
  `https://github.com/kxxt/ebeepf`,
  read 2026-11-20.
- Repository:
  created 2026-08-15,
  0 stars,
  0 forks,
  0 open items,
  last push 2026-10-06,
  license detection `NOASSERTION`,
  which matches the archive's two license files and is a detection limit,
  not a defect.
- Maintainer concentration:
  one contributor,
  36 of 36 commits.
- Issues and pull requests:
  none in the window and none older.
  Nothing was merged,
  nothing was reviewed,
  and the repository has no `.github` directory,
  which the API answers 404 for when re-read in the second revision,
  so there is no CI to lean on.
- Release record:
  one release,
  2026-10-06,
  on crates.io;
  the repository host lists no release object.
  No commit after the release.
- Classification:
  effectively unmaintained since its single release,
  with no tracker activity at all;
  the skill says zero tracker activity is low-signal and not proof of health,
  and here the source record agrees with the silence rather than softening it.

### `pahole`

- Primary URL:
  `https://github.com/acmel/dwarves`,
  read 2026-11-20,
  the project's mirror;
  the package points at `https://git.kernel.org/pub/scm/devel/pahole/pahole.git` as its home.
- Repository:
  created 2011-11-05,
  286 stars,
  81 forks,
  49 open items,
  last push 2026-10-02.
- Maintainer concentration:
  62 contributors,
  of which `acmel` holds 1,595 commits and the next four hold 86,
  57,
  33 and 24.
  In the window,
  the most recent 100 commits (the page cap,
  which the report states because the cap was reached),
  covering 2026-06-19 to 2026-10-02,
  and `acmel` wrote 87 of them.
- Issues in the window:
  five,
  all sampled,
  all still open,
  one maintainer reply among them
  (issue 74,
  one maintainer comment out of two),
  no labels,
  no assignees,
  no milestones.
- Pull requests:
  ten sampled of eleven in the window,
  and all ten are open with no review.
  Several carry maintainer comments (`acmel` on 41 and 29,
  `alan-maguire` on 46)
  but none was merged,
  labelled or assigned through the host.
  That is what a mirror looks like:
  patches are reviewed and merged on the project's mailing list,
  and the host's pull requests are a secondary queue.
- Release record:
  no release objects on the host;
  the tool releases from the project's own site and its distribution packages,
  and the package here is `1:1.32-2`.
  The host's tags date the releases the audit could not take from a releases endpoint:
  `v1.32` on 2026-09-13,
  `v1.31` on 2025-11-07 and
  `v1.30` on 2025-04-09.
- Classification:
  active development in a single very experienced maintainer,
  with weak issue triage on the host and no merge path there;
  C5 rates the release cadence,
  not the host's queue.

### `bpftool`

- Primary URL:
  `https://github.com/libbpf/bpftool`,
  read 2026-11-20.
- Repository:
  created 2022-01-16,
  723 stars,
  124 forks,
  31 open items,
  last push 2026-11-09,
  license detection `NOASSERTION`,
  which is the three license files the repository carries
  (`LICENSE`,
  `LICENSE.BSD-2-Clause` and `LICENSE.GPL-2.0`).
- Maintainer concentration:
  the contributors read returned a full page of 100,
  so the count is at least 100;
  the largest are `qmonnet` 324,
  `dependabot` 68,
  `anakryiko` 33,
  `olsajiri` 32 and `borkmann` 17.
  In the window,
  the most recent 100 commits (the page cap),
  which the report states because the cap was reached:
  100 rows covering 2026-06-19 to 2026-10-02,
  of which `dependabot` 22 and `qmonnet` 17;
  the rest is spread over 29 other authors,
  several of them one patch each.
- Issues in the window:
  16,
  all sampled,
  8 closed and 8 open,
  each closed one by a maintainer;
  comments run from 0 to 6 and maintainers answer most of them.
  Labels appear rarely,
  and no assignee or milestone was set.
- Pull requests:
  ten of 75 in the window,
  sampled as the skill asks.
  Nine are merged,
  all by `qmonnet`,
  and five of those are `dependabot` bumps,
  each reviewed by `qmonnet` before it merged.
  The one external code change among the merged (`#297`,
  `include: add u128 to linux/types.h`) was reviewed twice,
  and it merged in 0.7 days,
  so nothing external was merged unreviewed in this sample.
  The periodic "Bpftool sync" pull requests bring the kernel tree's state across and carry no review,
  which is their normal shape;
  the sample's three of them,
  `#301`,
  `#298` and `#294`,
  are maintainer self-merges with no review.
  The one open external fix (`#299`,
  "fix line info offsets in CFG dump") has no review yet.
  Merge latency is short for the mechanical ones,
  hours to days.
- Release record:
  releases on the host,
  driven by tags and a release workflow;
  the audit's own releases response lists `v7.8.1` on 2026-10-20 and `v7.7.0` on 2026-03-21.
  The binary here reports `7.8.0` while its package is `7.3.4-1`,
  which is the version-string finding
  ([Provenance of the two programs](#provenance-of-the-two-programs)).
- Classification:
  active,
  well-staffed maintenance,
  with a real merge path and a dependence on one lead maintainer.

### What the audit changes

`btf-rs` is the only finalist whose pull requests are reviewed and merged by a second maintainer,
and it is the only one whose record shows hardening fixes landing in the window.
`ebeepf`'s C5 at 2 stands and is now backed by its whole record rather than three counts.
The two tools' C5 at 4 was asserted with no audit at all;
the audit supports a mature cadence for both,
but it also shows that `pahole` has no merge or triage path on the host at all,
so that 4 is a rating of the release record and not of the host's responsiveness.
No rating moves as a result of this audit;
what moves is that the ratings now have records behind them,
which is what the skill asks a C5 claim to have.

## Human auditability

The skill's high-trust overlay asks for measurements of the code a root process would run,
and the first revision named the overlay and then measured nothing against it.
The second revision measured the four candidates from their own source,
on 2026-11-20;
the third revision re-ran the two libraries' measurements from freshly downloaded archives,
with the output kept
(`keep/measure-the-three-crate-archives-output-2.txt`),
so the numbers below are the third revision's own reads and not citations only.

- `btf-rs` 2.0.1:
  2,749 lines of Rust in 8 files under `src/`
  (`lib.rs`,
  `btf.rs`,
  `cbtf.rs`,
  `error.rs`,
  `section.rs` and three under `utils/`),
  of which the route the product calls,
  `from_bytes` and `from_split_bytes` through `btf.rs`,
  `cbtf.rs` and `section.rs`,
  is about 2,000;
  the rest is the optional ELF reader and the collection helpers;
  1 `unsafe` token,
  at `src/btf.rs:58`,
  in the file-mapping constructor the product's route does not call;
  5 direct runtime dependencies
  (`btf-rs-derive`,
  `byteorder`,
  `fallible-iterator`,
  `memmap2`,
  `thiserror`),
  one of them same-author (`btf-rs-derive`,
  114 lines);
  no process execution and no network anywhere in `src/`;
  five file-open or mapping sites in `btf.rs`,
  which are the path-based constructors,
  and none on the byte-slice route.
- `ebeepf` 0.1.0:
  28,769 lines of Rust in 25 files under `src/`,
  of which the BTF reader is `src/btf.rs` at 1,926 lines,
  6.7 percent of the crate,
  with the rest an eBPF loader the product would not use;
  57 `unsafe` tokens,
  none in `btf.rs`,
  concentrated in `sys.rs` (16),
  `netlink.rs` (9),
  `user_ringbuf.rs` and `ringbuf.rs` (8 each) and `map.rs` (7);
  7 direct runtime dependencies
  (`bitflags`,
  `flate2`,
  `goblin`,
  `libc`,
  `object` with `write`,
  `sha2`,
  `thiserror`);
  three `Command::new` sites in `src/skel.rs`,
  at lines 603 (`clang`),
  673 (`rustfmt`) and 3397
  (`cargo`,
  inside a test module),
  plus one in `src/test_bpf.rs:37` (`clang`),
  and none of them is reached by the parse route;
  the earlier count of five `clang` calls in `skel.rs` was wrong.
  No network.
- `pahole` 1.32:
  the packaged binary's parse path is libbpf 1.8.0 at `dd92bef7f6c7`,
  the submodule commit the `v1.32` tag records,
  whose
  `src/btf.c` is 6,360 lines,
  plus the project's `btf_loader.c` at 863 lines at `v1.32`;
  there is no `unsafe` count to take,
  because all of it is unchecked by construction;
  the binary needs four shared libraries,
  all the project's own or `libc`;
  the product never links it,
  so its lines are the child's and not the handler's.
- `bpftool` 7.8.0:
  the 6,376 lines of the kernel tree's libbpf C at `v7.3.4` plus the project's
  `tools/bpf/bpftool/btf.c` at 1,562 lines at
  `v7.3.4`,
  which is the tree the package builds;
  the 1,895-line and 6,780-line figures the second revision printed describe the standalone mirror
  and a `dwarves` submodule,
  not the packaged code;
  C again,
  so no `unsafe` count applies;
  the binary needs `libelf`,
  `libcrypto`,
  `libz`,
  `libcap` and `libLLVM`,
  the last a large dependency the product would load into the child.

Names of the measured commands:
`find <crate>/src -name '*.rs' -print0 | xargs --null wc -l`,
`rg --count-matches '\bunsafe\b'`,
`rg --count-matches 'std::process::Command|Command::new'`,
`rg --count-matches 'TcpStream|UdpSocket|reqwest|hyper'`,
`readelf --dynamic`,
`wc -l` over the two C sources.

What the measurements say:

the two libraries differ by a factor of ten in lines and by 57 to 1 in `unsafe` tokens,
and the product would use 7 percent of `ebeepf`'s code.
That is a real and large auditability difference,
and it is what C8 prices.
The tools are in the middle:
their parser is one C file reading bytes,
6,360 lines at the libbpf commit `pahole` packages and 6,376 at the kernel tree `bpftool` comes from,
which is a smaller surface to read than `ebeepf`'s crate but not memory-safe,
and it does not run in the handler's address space at all,
which is an auditability advantage a line count does not show.
The overlay says not to give unused features positive weight;
the C8 ratings price the code the product would actually run or call,
not the crate's total size,
which is why `ebeepf` is not rated lower than the tools.

## Evidence for the scores

Each entry names its criterion,
weight,
rating,
confidence and score effect,
and a range's endpoints
appear where a range exists.
The design's own evidence records stand in its "Scored comparison";
these are the records this report checked,
with the run or path beside each claim.

### C1: bounded fail-closed handling of untrusted bytes, weight 1

- `btf-rs` 3 medium.
  Refuses truncated input and a wrong magic (four negative controls above),
  and its input is kernel-validated;
  it has no section check of its own and its released masks are the 16-bit count and 5-bit kind fields,
  so the pre-scan is the binding's (`btf-rs/src/section.rs:182,206`,
  `btf-rs/src/btf.rs:303-355`).
  The confidence is medium,
  not high as the first revision had it:
  the same evidence class is what `ebeepf` gets medium for,
  the crate has no fuzz target and no mutation testing,
  and its own history carries overflow and out-of-bounds fixes from 2026-02-04 and 2026-10-29
  (section "Maintenance audit").
  A high label would keep this rating out of the perturbation set,
  and one confidence label should not decide whether the library pair has one reversal or two.
  Effect:
  3 of 32.
- `ebeepf` 3 medium.
  Bounds both sections,
  validates every type reference before returning a table (`src/btf.rs:441-477`,
  `:708-751`),
  and refuses truncation and a wrong magic;
  4 is refused because its README calls the tree AI-generated code that has not been thoroughly reviewed,
  and because it accepts a split table as a base.
  Effect:
  3.
- `pahole` 3 medium and `bpftool` 3 medium.
  Each refuses through its own parser and exit status,
  and the wrapper must bound the file before handing it over.
  Effect:
  3 each.

### C2: release and deployment fit, weight 1

- Both libraries 4 high:
  pure Rust,
  default features empty,
  no C build,
  and both GNU targets build from source
  ([The `aarch64` build](#the-aarch64-build-and-what-it-does-not-show)).
  Effect:
  4 each.
- Both tools 1 medium:
  each adds a program the product must find and version,
  and one of them in the root handler.
  Effect:
  1 each.
  `pahole`'s 1 is in its 18.5 and its two rating steps are in the matrix;
  `bpftool`'s is printed in the recorded arm only,
  because the candidate failed.

### C3: privilege surface in the root handler, weight 1, higher is smaller

- Both libraries 4 high:
  in-process code keeps the privileged handler to its own binary.
  Effect:
  4 each.
- Both tools 1 medium,
  and the rating prices the shape the design names,
  a program the root handler starts:
  that shape adds a path and library resolution to the boundary.
  The premise the first revision gave,
  that the child must run as root,
  is wrong:
  both BTF files are mode `0444`,
  so the child can read them unprivileged and bounded
  ([The two programs' eligibility](#the-two-programs-suites)).
  The unprivileged-child shape is a variant the project can choose and this rubric does not price;
  it is one of the open questions.
  The confidence is medium because the rating prices a shape the product need not take.
  Effect:
  1 each.
  `pahole`'s is in its 18.5;
  `bpftool`'s is in the recorded arm only.

### C4: diagnostic precision, weight 1

- `btf-rs` 4 high and `ebeepf` 4 high:
  each exposes names,
  bit offsets and sizes,
  so a refusal can name the member and both values;
  `ebeepf`'s `size_of` also multiplies array elements (`src/btf.rs:576-706`),
  which `btf-rs` leaves to the binding (`btf-rs/src/btf.rs:512` has `len()` only).
  Effect:
  4 each.
- `pahole` 3 medium:
  its `-C` printer maps directly to the compared members.
  Effect:
  3.
- `bpftool` 2 medium:
  the raw dump needs a type-id resolver in the wrapper,
  demonstrated by the harness.
  Effect:
  2.

### C5: maintenance and upstream responsiveness, weight 1

Every claim here is now backed by the sample recorded in section "Maintenance audit".

- `btf-rs` 3 medium:
  releases `v2.0.0` (2026-04-16) and `v2.0.1` (2026-11-12),
  every merged pull request in the window reviewed and merged by a second maintainer,
  merge latency from 0.5 to 91 days,
  hardening fixes landing in the window,
  two open items (PR 37,
  issue 1),
  four contributors with about 69 percent of commits from one of them,
  and the release lagging its own main at `cdf9ea46` on the widened masks.
  Effect:
  3.
- `ebeepf` 2 medium:
  one release,
  2026-10-06;
  one maintainer and 36 of 36 commits;
  no issues,
  no pull requests and no CI;
  no commit after the release;
  a README that declares the tree unreviewed.
  Effect:
  2.
- Both tools 4 medium:
  both are packaged here and both follow a release cadence,
  and the audit shows a real merge path for `bpftool` and a single very active maintainer for `pahole`.
  The release dates the audit's own responses carry,
  and the second revision did not cite,
  are
  `pahole`'s tags `v1.32` on 2026-09-13,
  `v1.31` on 2025-11-07 and `v1.30` on 2025-04-09,
  and `bpftool`'s releases `v7.8.1` on 2026-10-20 and `v7.7.0` on 2026-03-21.
  Two facts keep this out of the high-confidence set the second revision used:
  the audit read GitHub mirrors while `pahole` develops on a mailing list and releases from the
  project site,
  and `bpftool` develops in the kernel tree,
  whose releases are kernel releases;
  and the project's own queue on the host is not the venue the packaged code comes from.
  The audit also shows that `pahole`'s host pull requests are all open with no review,
  so the rating covers the release record and not the host's queue.
  The label is medium here;
  the arm that keeps the two programs scores them on that label
  ([The two programs' eligibility](#the-two-programs-suites)).
  Effect:
  4 each.
  `pahole`'s is in its 18.5;
  `bpftool`'s is in the recorded arm only.

### C6: integration and test cost, weight 1, higher is cheaper

- Both libraries 3 medium:
  the shared shape is the bounded loader,
  the pre-scan and its widest-encoding walk,
  the expectation table with its path grammar,
  the comparison,
  the refusal value and their tests,
  about 350 to 550 source lines and 300 to 450 test lines;
  the dependency route still owes one piece of arithmetic,
  array byte sizes,
  which `btf-rs` leaves to the binding and `ebeepf` supplies.
  `ebeepf` has no CI to lean on,
  which is why it does not rate higher than `btf-rs`.
  Effect:
  3 each.
- Both tools 3 medium:
  a spawn wrapper and an output parser each,
  plus packaging and a version matrix,
  which the unrun release matrix leaves uncertain.
  Effect:
  3 each.

### C7: start cost, weight 1, low confidence and unmeasured

- Both libraries 2.5 low,
  range 2 to 3:
  `btf-rs` parses the base's type offsets and the module's types;
  `ebeepf`'s `parse_split` copies the base's 151,181 types into the module's table,
  which is a memory cost `btf-rs` does not pay.
  No timing was taken by any round,
  so this is a structural reading,
  not a measurement.
  Effect:
  2.5 each,
  endpoints 2 and 3.
- Both tools 1.5 low,
  range 1 to 2:
  a process start and a full dump each.
  Effect:
  1.5 each,
  endpoints 1 and 2.

### C8: human auditability of the reader surface, weight 1, higher is leaner

- `btf-rs` 4 medium:
  2,749 lines,
  of which about 2,000 are the route the product calls,
  1 `unsafe` token outside the route the product calls,
  5 direct dependencies with one same-author proc-macro of 114 lines,
  no process execution and no network.
  The rating scale this criterion was applied on is the anchors below,
  which the second revision left implicit:
  4 is a reader whose whole route is pure Rust,
  under 3,000 lines,
  with no `unsafe` on the route and a small,
  single-author-visible dependency set;
  3 is that shape with an `unsafe` block or a wider dependency surface between the caller and the
  parse;
  2 is a reader that is either not memory-safe,
  or carries ten times the code the route uses,
  or both;
  1 is unchecked native code in the handler's address space with a runtime library load;
  0 is no inspectable route at all.
  `btf-rs` sits at 4 because every anchor of 3 is absent:
  no `unsafe` on the route,
  five dependencies including the 114-line proc-macro,
  and no process or network site.
  The five dependencies are the one thing that would have made this a 3 rather than a 4 on a stricter
  reading;
  the rating records that they are not none.
  Effect:
  4.
- `ebeepf` 2 medium:
  28,769 lines of which the reader is 1,926,
  57 `unsafe` tokens,
  7 direct dependencies,
  and three `Command::new` sites in a module the parse route never calls
  plus one in a test module.
  The product would link ten times the code it uses.
  Effect:
  2.
- Both tools 2 medium:
  the parser is one C file,
  6,360 lines at libbpf `dd92bef7f6c7` and 6,376 at the kernel tree `v7.3.4`,
  which is smaller than `ebeepf`'s crate but unchecked by construction,
  and `bpftool`'s child also loads `libLLVM`.
  They do not rate higher because the child's code is not memory-safe either,
  and do not rate lower because none of it runs in the handler's address space.
  Effect:
  2 each.

The criterion's role,
stated because it decides a result a reader would not otherwise see:
it is what makes the library pair stable at the frozen weights.
The pair's 27.5 against 24.5 rests on a 3-point gap,
and C8 alone contributes 2 of it;
with C8 removed the same ratings put `btf-rs` at 23.5 and `ebeepf` at 22.5,
and six one-input steps tie them
(`btf-rs` C1,
C5 and C6 down by one;
`ebeepf` C1,
C5 and C6 up by one)
in a smaller family that flags more of its tests than the eight-criteria one does.
That arm is a sensitivity and not a second baseline:
the criterion is derivable from the skill's high-trust overlay,
and the second revision added it after rating,
which is the departure
[Refreeze of the second revision](#refreeze-of-the-second-revision) records.

The numbers behind every entry are section "Human auditability".

The in-house reader's ratings price code that does not exist:
C1 4 high,
C2 4 high,
C3 4 high,
C4 4 high,
C5 3 medium,
C6 2 medium,
C7 2.5 low,
range 2 to 3,
C8 4 high.
Its 27.5 of 32 is a ceiling,
not a measurement,
and the design says so.
The four high labels are why its pair with `btf-rs` is not perturbed on those criteria:
labelling all four medium,
which is what code that does not exist supports,
gives a 52-test pair from which 28 tests order it,
where the high labels give 47 tests from which 23 order it
(section [Sensitivity](#sensitivity)).

## Score arithmetic

Earned is the sum of weight times rating.
Two baselines are printed:
the frozen one,
which is the skill's weights and the one the ranking above uses,
and the design writer's weights,
reported as a sensitivity arm and never used to rank.

### The frozen baseline

Every weight 1,
maximum 32,
normalized to one decimal place.
Every figure below was recomputed by
`~/temp/agent/zfs-layout-checker-reader-vet-report-recheck-findings-fixed-2026-11-20/scratch/keep/recompute-the-matrix-over-the-remaining-candidates.ts` (scratch,
deleted on 2026-11-27;
not kept),
whose output is kept beside it,
and the second revision's script is at
`~/temp/agent/zfs-layout-checker-reader-vet-report-gate-review-findings-fixed-2026-11-20/scratch/keep/recompute-refrozen-rubric-matrix.ts` (scratch,
deleted on 2026-11-27;
not kept),
not directly under `scratch/`,
which is the path the second revision printed.
Both run from the ratings in section "Evidence for the scores".

- `btf-rs` 2.0.1:
  `1*3 + 1*4 + 1*4 + 1*4 + 1*3 + 1*3 + 1*2.5 + 1*4` = 27.5 of 32,
  85.9.
- `ebeepf` 0.1.0:
  `1*3 + 1*4 + 1*4 + 1*4 + 1*2 + 1*3 + 1*2.5 + 1*2` = 24.5 of 32,
  76.6.
- `pahole` 1.32:
  `1*3 + 1*1 + 1*1 + 1*3 + 1*4 + 1*3 + 1*1.5 + 1*2` = 18.5 of 32,
  57.8.
- `bpftool` 7.8.0,
  printed because the four-candidate arm needs it and not ranked:
  `1*3 + 1*1 + 1*1 + 1*2 + 1*4 + 1*3 + 1*1.5 + 1*2` = 17.5 of 32,
  54.7.
- In-house reader,
  priced and not ranked:
  `1*4 + 1*4 + 1*4 + 1*4 + 1*3 + 1*2 + 1*2.5 + 1*4` = 27.5 of 32,
  85.9.

Low-signal endpoint totals,
over the two endpoints of C7's range:

- `btf-rs`:
  27 to 28,
  which is 84.4 to 87.5.
- `ebeepf`:
  24 to 25,
  which is 75.0 to 78.1.
- `pahole`:
  18 to 19,
  which is 56.3 to 59.4.
- `bpftool`:
  17 to 18,
  which is 53.1 to 56.3.
- In-house:
  27 to 28,
  which is 84.4 to 87.5.

### The design writer's weights, a reported sensitivity

5,
5,
4,
4,
3,
3,
2 and 1,
the last because that text states no priority for C8 either.
Maximum 108.

- `btf-rs`:
  94 of 108,
  87.0.
- `ebeepf`:
  89 of 108,
  82.4.
- `pahole`:
  62 of 108,
  57.4.
- `bpftool`:
  58 of 108,
  53.7.
- In-house:
  96 of 108,
  88.9.

Endpoint totals are 93 to 95,
88 to 90,
61 to 63,
57 to 59 and 95 to 97,
which is 86.1 to 88.0,
81.5 to 83.3,
56.5 to 58.3,
52.8 to 54.6 and 88.0 to 89.8.

At the seven criteria the first revision used,
the same ratings give 90,
87,
60 and 56 of 104 and 92 for the in-house reader.
The first revision printed 24 of 28 for the in-house reader at equal weights on those seven,
and called it "between" the two libraries;
the sum is 23.5,
which ties `btf-rs`'s 23.5,
as the design's own arithmetic says.
It changed no ranking then and it changes none now,
because that row is priced and never ranked.

Both baselines place `btf-rs` ahead of `ebeepf`,
and the writer's weights narrow that gap on a common scale:
normalized,
the frozen gap is 85.9 minus 76.6,
which is 9.3 points,
and the writer's is 87.0 minus 82.4,
which is 4.6.
The raw 3 of 32 against 5 of 108 compares different denominators and says the opposite.
Neither baseline is a finding:
the frozen one is the skill's and the other is a preference nobody has stated.

## Sensitivity

Method,
one input at a time,
over the refrozen rubric:
every weight from 1 through 5 for each of the eight criteria,
every medium-confidence and low-confidence exact rating one step down and up within 0 through 4,
and both endpoints of every low-signal range.
The ranges' endpoints are the plausible values,
so no step is taken outside a range.
Ranking is by the unrounded fraction,
and equal totals are an exact tie.

Two families are run and the difference matters:

- over the finalists the skill leaves,
  `btf-rs`,
  `ebeepf` and `pahole`,
  66 tests,
  with both tools' C2 at the label the report prints,
  1 medium,
  and `pahole`'s C3 and C5 confidence corrected to medium;
  this is the set the report's result rests on;
- over all four candidates as a recorded arm,
  81 tests,
  because the project may restore `bpftool` and then the earlier conditional rankings are what it
  gets
  ([The two programs' eligibility](#the-two-programs-suites)).

The two families' counts are the gate final check's,
recomputed with its script under the C2 label the report prints,
and they differ from the third revision's only by the C2 rating steps that label adds,
two in the finalists' family and four in the four-candidate arm.
The arm counts at the design writer's weights and the seven-criteria comparison are this revision's,
from that same test definition over the same ratings,
and that script and its output are kept in this revision's `keep/`.

### The finalists that remain

Baseline:
`btf-rs` 27.5 > `ebeepf` 24.5 > `pahole` 18.5 of 32.

- `btf-rs` against `ebeepf`,
  51 tests:
  all 51 keep the order.
  Two facts make that structural rather than lucky:
  `btf-rs`'s rating is at least `ebeepf`'s equal on every criterion,
  so no weight increase can favour `ebeepf`;
  and the only step that brings the margin below 3 is `btf-rs`'s C1 falling from 3 to 2,
  which leaves 26.5 against 24.5.
- `ebeepf` against `pahole`,
  57 tests:
  55 keep,
  1 ties and 1 reverses.
  - weight C5 from 1 to 4 ties at 30.5 each.
  - weight C5 from 1 to 5 reverses:
    `pahole` 34.5 over `ebeepf` 32.5.
    This is the row that decides whether this family recommends anything,
    and it is a weight,
    not a rating:
    a weight is a preference,
    the project has stated none,
    and the skill says to ask rather than to invent one.

Two of those 66 tests tie or reverse an adjacent order,
and both are the same criterion's weight.
So the skill's condition for ranking by the baseline is not met,
and the report gives conditional rankings and recommends none
([Terminal result](#terminal-result-sensitivity-unresolved)).
The first place is not what moves:
`btf-rs` is first in all 66 tests,
by at least 2 over the runner-up and by at least 5 over `pahole`.
The input that settles this family is how much maintenance and upstream responsiveness should
count against deployment fit and privilege,
and it reorders places 2 and 3 only.
Stability here covers one input at a time,
and one thing more is measured and printed rather than claimed:
at the baseline ratings,
over all 390,625 weight vectors with every criterion between 1 and 5 jointly,
`btf-rs`'s smallest lead is 3 over `ebeepf`,
5 over `pahole` and 6 over `bpftool`.
Simultaneous changes of ratings and weights together are not claimed.

At the design writer's weights the same three finalists score 94,
89 and 62 of 108,
and two tests tie an adjacent order,
both on the library pair:

- `btf-rs` C1 from 3 to 2 ties at 89 each;
- `ebeepf` C1 from 3 to 4 ties at 94 each.

Those are C1 steps on the design writer's weights,
which are not the baseline.

### The arm that keeps both programs

Baseline:
`btf-rs` 27.5 > `ebeepf` 24.5 > `pahole` 18.5 > `bpftool` 17.5 of 32.
This arm is recorded,
not scored:
it is what the project gets if it answers question 3 by keeping `bpftool` eligible.

- `btf-rs` against `ebeepf`,
  51 tests:
  all 51 keep the order,
  by the same argument as above.
- `ebeepf` against `pahole`,
  57 tests:
  55 keep,
  1 ties and 1 reverses,
  at C5 weights 4 and 5,
  as in the family above.
- `pahole` against `bpftool`,
  62 tests:
  49 keep and 13 tie,
  with no reversal.
  - `pahole` C1 from 3 to 2 ties at 17.5 each.
  - `pahole` C2 from 1 to 0 ties at 17.5 each.
  - `pahole` C3 from 1 to 0 ties at 17.5 each.
  - `pahole` C4 from 3 to 2 ties at 17.5 each.
  - `pahole` C5 from 4 to 3 ties at 17.5 each.
  - `pahole` C6 from 3 to 2 ties at 17.5 each.
  - `pahole` C8 from 2 to 1 ties at 17.5 each.
  - `bpftool` C1 from 3 to 4 ties at 18.5 each.
  - `bpftool` C2 from 1 to 2 ties at 18.5 each.
  - `bpftool` C3 from 1 to 2 ties at 18.5 each.
  - `bpftool` C4 from 2 to 3 ties at 18.5 each.
  - `bpftool` C6 from 3 to 4 ties at 18.5 each.
  - `bpftool` C8 from 2 to 3 ties at 18.5 each.

Fifteen of those 81 tests tie or reverse an adjacent order,
and thirteen of the fifteen are that program pair;
the other two are the maintenance weight on the second and third places.
The C2 steps are the pair the report's printed label adds:
the third revision's matrix had C2 high for both tools and counted 77 tests and eleven pair ties.
They are rating-step ties,
which the skill resolves with evidence and not with a preference;
they are also moot while `bpftool` is failed,
because no order between it and `pahole` is published
([The two programs' eligibility](#the-two-programs-suites)).

The first place never moves in this arm,
in any of the 81 tests.
Its smallest lead over the runner-up is 2 points,
and over the best program 5 points,
at a maintenance weight of 5:
`btf-rs` 39.5 > `pahole` 34.5 > `bpftool` 33.5 > `ebeepf` 32.5.
That is the row the first revision's text called "the program arm becomes the rational pick";
the row says the library is still first,
by 5,
and the weight reorders places 2 to 4.

Only one of the eight weights moves the whole order,
and only above 3:

- weight C5 at 2:
  `btf-rs` 30.5 > `ebeepf` 26.5 > `pahole` 22.5 > `bpftool` 21.5,
  the baseline order;
- weight C5 at 3:
  `btf-rs` 33.5 > `ebeepf` 28.5 > `pahole` 26.5 > `bpftool` 25.5,
  the baseline order;
- weight C5 at 4:
  `btf-rs` 36.5 > `ebeepf` 30.5 = `pahole` 30.5 > `bpftool` 29.5,
  an exact tie;
- weight C5 at 5:
  `btf-rs` 39.5 > `pahole` 34.5 > `bpftool` 33.5 > `ebeepf` 32.5,
  so both programs pass the second library and neither approaches the first.

Every other criterion's weight,
at every value from 2 through 5,
leaves the order exactly as the baseline has it.
At the design writer's weights this arm is 81 tests with ten flagged:
the `pahole` and `bpftool` C1 and C2 reversals and their C3 and C4 ties,
and the same two C1 ties on the library pair.

### What the evidence could and could not settle

The skill's first remedy for a move is to gather decisive evidence,
refreeze and rerun.
The second revision did the evidence half for three inputs:
the maintenance audit for C5 (section "Maintenance audit"),
the auditability measurements for C8 (section "Human auditability"),
and the C1 confidence label for `btf-rs` (section "C1").
It refroze and reran,
and the rerun still moved at four candidates.
The third revision ran the one suite that could still be run
(`pahole`'s,
in a container),
which removed `bpftool` and kept `pahole`;
the family is smaller and one weight still reorders two of the three.

What could not be settled by evidence,
and what the report therefore does not pretend to settle:

- the maintenance weight,
  which is a preference and not a reading,
  and which decides whether `ebeepf` or `pahole` is second;
- `bpftool`'s functional tests,
  which the report reads as needing root,
  a BPF-capable kernel and a VM harness;
- the start cost,
  which no round measured and which C7 prices as a low-confidence range in every row;
- the `aarch64` run,
  which no machine here has.

The in-house pair is priced and not ranked,
and its matrix is recorded because the project could reopen it:
at the frozen baseline it ties `btf-rs` at 27.5 each.
The second revision printed "35 of its 47 tests move it,
24 to a tie and 11 to a reversal";
that counts a tie as movement,
which is wrong,
because a tie is the baseline.
The three buckets,
at the labels the second revision used
(in-house C1 to C4 and C8 high,
C5 and C6 medium,
C7 low),
are:

- 24 of 47 stay tied;
- 11 of 47 put `btf-rs` ahead;
- 12 of 47 put the in-house row ahead.

With every in-house rating labelled medium,
which is what code that does not exist supports,
the pair runs 52 tests:

- 24 stay tied;
- 16 put `btf-rs` ahead;
- 12 put the in-house row ahead.

So 23 or 28 tests order the pair,
not 35,
and the report prints both labellings rather than the flattering one.

## Pros and cons for every finalist

### `btf-rs` 2.0.1

- Pros:
  split BTF and the type kinds the check needs;
  pure Rust with the default features off,
  no C build and no runtime program;
  a documented API and 168 integration tests plus four doctests on the CI path,
  64 under the default features;
  the consumer probe parses this host's pair and walks the anonymous union,
  183 checks with four negative controls refused;
  the archive maps to the tested commit on a byte diff;
  every merged pull request in the last twelve months was reviewed and merged by a second maintainer;
  2,749 lines,
  one `unsafe` token and no process execution or network in the crate.
- Cons:
  `LGPL-2.1-or-later` adds a copyleft-family dependency;
  no section validation in the crate,
  so the binding keeps the pre-scan;
  the release's masks lag its own main branch on the widened fields;
  overflow and out-of-bounds fixes landed as recently as 2026-10-29,
  which is why C1 is medium and not high;
  a build-time proc-macro from the same authors joins the audit surface;
  the published archive ships no tests,
  so its suite runs from the repository at the tag;
  no fuzz target.
- Cost:
  the shared shape,
  about 350 to 550 source lines and 300 to 450 test lines,
  plus the array-size shim and one fuzz target.
- Its score:
  27.5 of 32 at the frozen baseline,
  the highest of the eligible candidates,
  and 94 of 108 at the design writer's weights.
- Why it leads even though the order below it moves:
  within the skill's range,
  weights 1 to 5 one criterion at a time or all eight jointly,
  no other candidate's total passes its own,
  and `pahole` is at least 5 points behind at every one of those joint vectors.
  The lead is bounded and not absolute,
  and the report says where it ends:
  `btf-rs` rates one point below `pahole` on C5,
  so a maintenance weight of 10 with the rest at 1 ties at 54.5 each and a weight of 11 puts `pahole`
  first at 58.5 against 57.5,
  which is outside the skill's 1 to 5 range and outside its one-input test set.
  Inside it,
  a library needs no program the product must find and version,
  which is the reason against the programs;
  the lead over `ebeepf` is maintenance and auditability
  ([Conditional rankings](#conditional-rankings)).

### `ebeepf` 0.1.0

- Pros:
  a split parser with a byte-level API and a path constructor;
  `size_of` already multiplies array elements,
  so no array-size shim;
  pure Rust with the default features off;
  its archive suite and the consumer probe both ran.
- Cons:
  a dual `LGPL-2.1-only` and `BSD-2-Clause` crate,
  so it has to be taken under the BSD arm;
  one release,
  one maintainer and no CI,
  and no commit after the release;
  its README calls the tree unreviewed AI-generated code;
  released masks are the same 5-bit kind and 16-bit count fields as `btf-rs`'s;
  it accepts a split table as its own base,
  which the consumer probe showed;
  `parse_split` copies the base's types into the module's table;
  28,769 lines of which the reader is 1,926,
  with 57 `unsafe` tokens and seven direct dependencies;
  no fuzz target.
- Cost:
  the shared shape without the array-size shim.
- Its score:
  24.5 of 32 at the frozen baseline,
  three points behind on maintenance and auditability,
  and 89 of 108 at the design writer's weights.
- Why it is second and not third at the baseline:
  the pair with `btf-rs` separates on maintenance,
  auditability and the C1 confidence the archive supports,
  and no single rated input at the frozen baseline moves it;
  its place against `pahole` is the one input that moves,
  and it holds only while maintenance counts no more than three times deployment fit.

### `pahole` 1.32

- Pros:
  the command the design already names for its test-time comparison and the binding's live test already runs;
  `-C` prints the compared members;
  no new Rust dependency,
  and nothing of it runs in the handler's address space;
  a mature upstream with a release cadence and one very experienced maintainer;
  its BTF files are mode `0444`,
  so a child needs no privilege.
- Cons:
  a runtime package;
  a program the root handler starts,
  in the shape this rubric prices;
  a text format;
  an invocation each for the struct,
  the nested type and the enum;
  the product must find and version it;
  its embedded libbpf is the submodule commit the `v1.32` tag records,
  libbpf 1.8.0 at `dd92bef7f6c7`;
  its host pull requests get no review because the project merges on a mailing list;
  its upstream suite skipped eleven cases here,
  including its split-BTF case,
  whose artefacts this machine does not have;
  its own CI path was inventoried and not run;
  its parser is libbpf's 6,360 lines of C at the commit the tag pins.
- Its suite:
  76 cases at the packaged tag with the packaged patches,
  65 `Ok` and 11 `Skip`,
  none failing
  ([The two programs' suites](#the-two-programs-suites)).
- Its score:
  18.5 of 32 at the frozen baseline,
  the runtime program costing the deployment and privilege points the libraries do not.
- Why it is third at the baseline and second at a maintenance weight of 5:
  the whole of its deficit is C2 and C3,
  and both are criteria a project can rank below maintenance.

### `bpftool` 7.8.0, failed on relevant execution

- Pros:
  libbpf's reader;
  a documented base option and a raw type listing;
  a mature upstream with a real merge path and many contributors,
  and nothing of it in the handler's address space.
- Cons:
  the same runtime-program and root-path costs;
  the raw listing needs a type-id resolver before a member can be compared;
  member sizes are absent from the dump;
  its self-reported version and its package's version disagree,
  so a version check on the tool proves nothing;
  its child loads `libLLVM` beside `libelf`,
  `libcrypto`,
  `libz` and `libcap`;
  its functional suite was not run here.
- Its score:
  17.5 of 32 at the frozen baseline,
  the lowest of the four,
  for the resolver its output needs and its medium diagnostic rating;
  the score is not used for any result,
  because the candidate failed
  ([The two programs' suites](#the-two-programs-suites)).

### The in-house reader, priced and not ranked

- Pros:
  it can enforce every bound in one place;
  no dependency,
  no runtime program and no new license;
  it can read lazily,
  so C7 would be its to win once a reader exists;
  a partial decoder already exists on an unmerged branch
  (section "Discovery ledger").
- Cons:
  about 350 to 450 source lines plus 100 to 150 test lines,
  a fuzz target and a review of all of it;
  the branch's own decoder is 1,100 lines plus 422 of tests already,
  and its seam does not hold;
  every existing tool would still pass every hard gate,
  which is the skill's bar for writing one.
- Why unranked:
  the project answered on 2026-11-19 that our own parser is not a hard requirement,
  so the existing-tools rule keeps it ineligible while its 27.5 of 32 stays on record as what the route would have cost.

## Conditional rankings

No ranking here is the report's own.
The skill's rule is to rank by the provisional baseline only when every defined one-input test preserves order,
and at the frozen baseline two of 66 tests do not,
both of them the maintenance weight on the second and third places.
What follows is therefore conditional in the skill's sense:
the order on each side of every input that moves it,
with the reason each adjacent pair sits that way.

### With every criterion at weight 1, the frozen baseline

`btf-rs` 27.5 > `ebeepf` 24.5 > `pahole` 18.5 of 32.

- `btf-rs` leads `ebeepf` because both are libraries in the wrapper's address space,
  and the pair separates on maintenance and on auditability:
  reviewed and merged pull requests,
  hardening fixes landing,
  2,749 lines and one `unsafe` token,
  against one release,
  one maintainer,
  no CI and ten times the code for 7 percent of it used.
  All 51 one-input tests on that pair leave the order,
  which is the only adjacent pair the matrix does not move at these weights.
- `ebeepf` leads `pahole` only while maintenance counts no more than three times deployment fit.
  The reason it leads at all is that a library needs no program the product must find and version
  and no binary the handler starts;
  at a maintenance weight of 4 the pair ties,
  and at 5 it reverses.
- The `pahole` and `bpftool` pair has no published order:
  `bpftool` failed the skill's relevant-execution rule,
  so only the arm that keeps it prints that pair,
  with thirteen rating-step ties and no reversal
  ([Sensitivity](#sensitivity)).

### If maintenance and responsiveness weigh 4

`btf-rs` 36.5 > `ebeepf` 30.5 = `pahole` 30.5 of 44.
No order between the second library and the program is supportable at that weight,
and the skill says to ask which the project prefers rather than to break the tie.
That is question 1 of [Open questions for the project](#open-questions-for-the-project).

### If maintenance and responsiveness weigh 5

`btf-rs` 39.5 > `pahole` 34.5 > `ebeepf` 32.5 of 48.
The program passes the second library and is still 5 behind the first,
which is the fact the first revision's sentence about "the rational pick" got wrong.
In the arm that keeps `bpftool`,
the same weight puts it third at 33.5 and `ebeepf` fourth at 32.5.

### If the project reopens the in-house reader

That row is priced and not ranked because it is ineligible,
not because it lost.
At the frozen baseline it ties `btf-rs` at 27.5 of 32,
and 23 of its 47 tests order it at the labels the second revision used,
11 to `btf-rs` ahead and 12 to the in-house row ahead,
with 24 staying tied
(section [Sensitivity](#sensitivity)),
so a reopened in-house route would begin with an unresolved comparison rather than an answer.

## Confidence and evidence limits

Stated beside the rows they qualify,
gathered here so nothing is buried:

- `aarch64` builds and does not run,
  for both libraries.
  H3's second target has a build for each and no execution,
  and the two tools have no `aarch64` artifact at all.
  The build is the gate review's run,
  cited from its kept logs.
- The start cost is unmeasured,
  so C7 is a low-confidence range in every row.
  Measurement item 10 is the run that would settle it,
  together with the disk-cache thresholds.
- The weights are the skill's default because the project stated no priority.
  The design writer's weights are reported as a sensitivity arm and never ranked,
  and both arms are unstable on some pair.
- `pahole`'s validation is two layers,
  not three:
  its output boundary was exercised with negative controls on this host's pair,
  and its upstream suite ran here with eleven recorded skips and no failure.
  `bpftool`'s is one layer,
  its output boundary,
  because its functional suite needs root,
  a BPF-capable kernel and a VM harness,
  as the report reads it;
  that is why it failed.
  Both C5 ratings rest on the records in section "Maintenance audit",
  and both are labelled medium now.
- The programs' provenance is a recipe that names its source and a signature:
  `bpftool` maps to `linux-7.3.4.tar.xz` with its sha256,
  `pahole` to the signed tag `v1.32` plus three patches.
  Neither recipe proves a byte-identical rebuild,
  and `pahole`'s embedded libbpf is the submodule commit the tag records,
  `dd92bef7f6c7`.
  The kept suite run built libbpf's default branch instead,
  and the gate final check's rerun at that commit gives the same cases.
  `bpftool`'s self-reported version carries that libbpf's version,
  which is the same cost C2 prices,
  seen in a measurement.
- `ebeepf`'s provenance is its weakest point:
  one release,
  one maintainer,
  no CI,
  a README that declares the tree unreviewed,
  and a split base accepted as a base.
  Its 24.5 is a rating on that record,
  not a forecast.
- `btf-rs`'s published archive ships no tests and the suite ran from the repository at the tag;
  its release lags its own main on the BTF masks,
  and the lag is the reader's own:
  2.0.1 reads a type entry's kind in five bits and its member count in sixteen,
  so a kernel that widened either field is read as the narrower one where only a kind's high bits
  are set,
  and refused only when the spilling bytes stop its own parse,
  because the binding's pre-scan reads the header and the section list and no type entry.
  The integration pass measured both directions on fixtures the crate's own writer built:
  a type entry whose kind field is 33 passes the comparison as the integer its five bits name,
  a member count past sixteen bits stops the reader at the module file,
  and the approved module passes either way
  (`~/temp/agent/integration-pass-zfs-step-6-with-the-btf-rs-reader-2026-11-20/keep/experiments.log`).
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
  No file on this host sets a widened field,
  and the adoption record names the lag as a revisit trigger
  (`doc/decision/cli-git-zfs-runtime-layout-checker.md`,
  "Exit,
  rollback and revisit triggers").
- Neither library has a fuzz target,
  and no mutation testing exists for either;
  the negative controls are the only adversarial evidence.
- CI was reproduced on the CachyOS image with stable 1.100.0,
  not on GitHub's `ubuntu-latest`,
  so the two differ in distribution and in what else is installed.
- The class 3 web class is an inferred exhaustion and not a reported one,
  though each query ended below the provider's own cap,
  none sent a filter,
  and the review's eight extra queries found nothing new.
- The host kernel `7.3.5-1-cachyos` is not the product's qualified release until step 23,
  so the passing runs qualify the readers on this host's pair,
  not the product on every kernel.
- The 32-byte BTF header shape is what these files carry;
  the 24-byte shape a 6.8 guest's tools emit is owed measurement item 6 and no run here covers it.
- The gate re-check the third revision answered was written by a different vendor from the authoring
  session,
  so it counts under rule `IRW` as an independent read,
  and so did the gate review before it.
  The suite run,
  the archive measurement and the two matrices are the third revision's own,
  and their logs are kept rather than cited.
  The third revision's own edits are not independently reviewed:
  the same session made them,
  and a fresh cross-vendor read,
  or the project's own reading,
  is what would make them independent.

## Terminal result: sensitivity unresolved

The skill's terminal outcome for this evidence is "sensitivity unresolved":
record conditional rankings for each outcome-changing input and recommend none.
That is this report's result while `pahole` is a finalist,
and it depends on that one question.
If the project finds `pahole` not eligible,
with its own CI path unrun and case 18 unrun,
the family is two libraries,
all 51 one-input tests between them keep the order,
the skill's condition for ranking by the baseline is met,
and the terminal outcome becomes a recommendation of `btf-rs` over `ebeepf`,
27.5 against 24.5.
The report therefore states both:
the result it reaches on the evidence it has,
and the one the project's answer to question 4 produces.
It is the same outcome as the second revision's for a different finalist set and a different
reason,
which the report states rather than implying.

Why the recommendation the first revision made is withdrawn:

- The skill allows a ranking only when every defined one-at-a-time test preserves order.
  At the frozen weights two of 66 tests do not:
  a maintenance weight of 4 ties the second library with the remaining program,
  and a weight of 5 puts the program second and the library third.
  Both move one preference,
  not a rating,
  and a preference is the project's to state.
- The first revision's grounds do not survive the refreeze.
  Its refusal of one reversal left that rating's confidence untouched,
  so the matrix never changed;
  two of the four ties it explained away as maintenance ties are C6 steps that no maintenance
  evidence touches;
  and its separation of the libraries rested on evidence it had not recorded.
- The evidence the skill's loop asks for was gathered and applied,
  and the rerun still moves.
  What is left is a preference:
  how much maintenance should weigh against deployment fit and privilege.
  That is the project's to state,
  and question 1 of [Open questions](#open-questions-for-the-project) asks it.

What the report therefore carries:

- the frozen baseline's order,
  `btf-rs` 27.5 > `ebeepf` 24.5 > `pahole` 18.5 of 32,
  as the order at those weights and under no claim of stability;
- the conditions that change it,
  spelled out in section [Conditional rankings](#conditional-rankings);
- the questions that bear on an order below the first place:
  question 1 (the maintenance weight),
  question 3 (`bpftool`'s eligibility),
  question 4 (`pahole`'s eligibility)
  and question 5 (where question 1's threshold sits);
  question 2 changes no order here,
  being the choice between a library and a program.

What does not change,
stated as a fact and not as a recommendation in a conditional:

- `btf-rs` 2.0.1 is first in every defined test at the frozen weights,
  over all three finalists and over the recorded four-candidate arm,
  with a smallest lead of 2 points over the runner-up and 5 over `pahole`;
- it is the only finalist whose pull requests are reviewed and merged by a second maintainer,
  and the leanest reader any of the four offers;
- `bpftool` failed the skill's relevant-execution rule and cannot be ranked,
  and `pahole` stays a finalist only while the project accepts the omission recorded for it
  (question 4).

Beside the result,
in the same breath:

- `aarch64` is built and unrun for both libraries;
- the start cost is unmeasured;
- `pahole`'s suite skipped its `vmlinux`-based cases and its split-BTF case,
  with the reasons and their distance from the consumed route recorded,
  and its own CI path was inventoried and not run;
- and this is not adoption:
  no crate is added to any manifest,
  and the decision record's Q1 section still says the pick is the project's.

## What this report did not do

- It adopted no candidate,
  added no crate to any manifest,
  edited no decision record,
  and wrote nothing under `doc/handover/`.
- The third revision ran one candidate's code,
  in a container with no home mount and capped resources:
  `pahole`'s upstream suite,
  built from the packaged tag with the packaged patches.
  Nothing it ran could reach this machine's home directory.
  That run kept `--network host` throughout,
  because the same container installed the packages,
  and the proxy variables pointed at a closed port bounded the suite's own fetch rather than
  isolating the run;
  the fetch failed and the three cases skipped on it.
  The offline form of the same run is the gate final check's,
  two phases with `--network none` for the build and the suite,
  and it gives the same cases.
  Every other execution record above is a kept log of a round run earlier in this worktree or by a
  reviewer.
- The fourth revision ran no code at all,
  measured nothing and re-rated nothing.
  It read the gate final check and the records kept beside it,
  counted the tests that the report's printed labels imply,
  using a script over the ratings the
  report already prints,
  and changed text.
- It ran nothing on `aarch64` and took no timing,
  and its `aarch64` build evidence is the gate review's,
  cited where it is kept.
- It did not re-run the two libraries' suites,
  `bpftool`'s functional suite,
  or the class 3 queries;
  it re-ran the crate-archive measurements and `pahole`'s suite.
- It did not open an issue,
  a pull request or a comment on any candidate's repository,
  and it sent no query carrying this repository's name or this machine's addresses.
- It is not final and not approved;
  the project declares that.

## Sources

Internal:

- [`doc/decision/cli-git-zfs-runtime-layout-checker.md`][record],
  read whole,
  including its Q1 section and the answers of 2026-11-19.
- [`doc/planning/cli-git-zfs-runtime-layout-checker-design.md`][design],
  "Candidate ledger",
  "Discovery record",
  "Finalist validation",
  "Scored comparison",
  "Sensitivity",
  "Ranking and pros and cons",
  "The vet artifacts this brief could not write",
  "Owed measurements" and Q1.
- [`doc/audit/tech-rust-crate-for-openzfs-vet-2026-11-18.md`](tech-rust-crate-for-openzfs-vet-2026-11-18.md),
  for the wrapper's release model and the struct difference between OpenZFS releases.
- `package/zfs/request-to-openzfs-kernel-device/Cargo.toml`,
  `README.md` and `src/kernel_device/live_btf_tests.rs`,
  read for the incumbent question.
- `toolchain.lock.json:614-653`,
  for the two GNU release archives.

Kept evidence:

- `/home/user/temp/agent/zfs-layout-checker-reader-vet-report-final-check-wording-fixed-2026-11-20/scratch/keep/`,
  this revision's own records:
  the count script that recomputes every family and arm the report prints under the C2 label the
  report prints,
  its output,
  the lock record copy,
  and the staged text this revision wrote into place.
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
- `/home/user/temp/agent/zfs-layout-checker-reader-vet-report-recheck-findings-fixed-2026-11-20/scratch/keep/`,
  the third revision's own records:
  the matrix over the remaining finalists,
  its arms and its output,
  the crate-archive measurement and its output,
  the container driver,
  the five pahole-suite logs and their outputs:
  the run that never found `tests` on `PATH` and exited 127,
  the online run without `bpftool` or `clang` that skipped 24 and passed 52,
  the run killed at exit 137 before a single case line,
  and the two identical runs at 65 `Ok` and 11 `Skip`;
  the five `v7.3.4` source files,
  the three packaging patches,
  the lock record copy,
  the edit scripts the third revision used,
  and the lint mapping script.
  The scratch folders that held the extracted archives,
  the clones and the container's build are
  not kept,
  because they are reproducible from the kept records;
  the archives' hashes and line counts are the kept output.
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
- `/home/user/temp/agent/zfs-layout-checker-reader-vet-report-gate-review-findings-fixed-2026-11-20/scratch/keep/`,
  the second revision's own records:
  the recomputed matrix and its output,
  the staged report and the pre-lint copy,
  the repository-host responses under `gh/`,
  the per-sample maintainer records under `gh/detail/`,
  the fingerprint and lock scripts,
  and the lint mapping script.
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
- `/home/user/temp/agent/gate-recheck-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/`,
  the gate re-check the third revision answered:
  its review,
  its independent matrix and output,
  its crate-archive measurement,
  the packaging recipes it fetched,
  the `v7.3` kernel sources,
  the maintenance spot checks,
  and the lint mapping.
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
- `/home/user/temp/agent/gate-final-check-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/`,
  the gate final check this revision answers:
  its review,
  its count script and output for both matrices in both readings of the C2 label,
  its two-phase offline rerun of `pahole`'s suite at the pinned libbpf with the script and output,
  its proof that `git submodule update` checks out the recorded commit with the output it printed,
  the `v1.32` submodule entry and the libbpf commit it names,
  its excerpt of the tag's scripts and workflows,
  the three patches' BLAKE2 sums,
  and the `git-submodule(1)` passage it quoted.
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
- `/home/user/temp/agent/gate-review-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/`,
  the independent review the second revision answers,
  its recomputation of the first revision's matrix,
  the two `aarch64` build logs and their driver,
  its extra crates.io probe,
  and its consumer re-run.
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
- `/home/user/temp/agent/zfs-layout-checker-reader-finalist-validation-2026-11-20/keep/`,
  its `README.md`,
  `report-from-summary.md`,
  `evidence/btf-rs-2.0.1/`,
  `evidence/checksums.txt`,
  `evidence/tool-harness-checks.txt` and `evidence/tool-harness-invocations.txt`.
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
- `/home/user/temp/agent/zfs-layout-checker-reader-discovery-finished-2026-11-20/keep/`,
  its `README.md`,
  `report-from-summary.md`,
  the three query listings,
  the organization response files and the board pages,
  and `evidence/ebeepf-0.1.0/`.
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
- `/home/user/temp/agent/zfs-layout-checker-reader-vet-report-2026-11-20/keep/`,
  the first revision's fingerprint script,
  registry response and lock record.
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).

External,
all read 2026-11-20:

- crates.io [crate pages][crates-io],
  the sparse index for `btf-rs`,
  the `q=btf`,
  `q=split btf`,
  `q=btf kernel` and `q=libbpf` search endpoints,
  and the three archives both revisions downloaded and hashed.
- the kernel's stable tree at tag `v7.3.4`,
  read through
  `raw.githubusercontent.com/gregkh/linux/v7.3.4/` for the four files named under
  [Provenance of the two programs](#provenance-of-the-two-programs).
- the distribution packaging repositories:
  Arch's `linux-tools` PKGBUILD at tag `7.3.4-1`,
  and CachyOS's `pahole` PKGBUILD with its three patches.
- [docs.rs][docs-rs] for both library crates.
- the repository hosts' records for
  [`retis-org/btf-rs`](https://github.com/retis-org/btf-rs),
  [`kxxt/ebeepf`](https://github.com/kxxt/ebeepf),
  [acmel/dwarves][pahole-repo],
  [`libbpf/bpftool`](https://github.com/libbpf/bpftool) and
  [`libbpf/libbpf`](https://github.com/libbpf/libbpf),
  through their API,
  and four read-only clones at depth 1.
- the distribution's package database through `pacman`,
  for both tools' packages,
  their versions,
  their declared dependencies and their signature validation.
- [bpftool documentation][bpftool-docs] for `btf dump` and `--base-btf`,
  and the [pahole repository][pahole-repo] for `--btf_base` and `-C`.
- [Linux kernel BTF documentation][kernel-btf] for the header and the split-file format,
  at tags `v6.8` and `v7.3` as the design records.

[skill]: ../../.agents/skills/choosing-technology/SKILL.md
[record]: ../decision/cli-git-zfs-runtime-layout-checker.md
[design]: ../planning/cli-git-zfs-runtime-layout-checker-design.md
[design-discovery]: ../planning/cli-git-zfs-runtime-layout-checker-design.md#discovery-record
[design-check-in]: ../planning/cli-git-zfs-runtime-layout-checker-design.md#the-check-in-of-2026-11-20
[crates-io]: https://crates.io/crates/btf-rs
[docs-rs]: https://docs.rs/btf-rs
[bpftool-docs]: https://docs.kernel.org/bpf/bpftool-btf.html
[pahole-repo]: https://github.com/acmel/dwarves
[kernel-btf]: https://docs.kernel.org/bpf/btf.html
