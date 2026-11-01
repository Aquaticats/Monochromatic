# Translation repair handover 2026-09-06: checkpoints of 2026-10-13, the hardware change and the linter migration

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/test` is now `package/test-harness/core`.

The two checkpoints written on 2026-10-13,
one when the owner paused the session to change the machine's processor with the third review's fixes uncommitted,
the other after the change,
when the linter migration had landed and the sixth review's fixes were in flight.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.

## 2026-10-13 after the hardware change: the linter migration landed, the sixth review's fixes in flight

This checkpoint is the lead's account of the state after the owner's hardware change,
and it replaces the checkpoint headed "2026-10-13 at the hardware change",
which stays in this document as history.
Every item is state,
not plan,
unless it sits under the heading "Next".
A path written `s<N>/...`,
`w<N>` or `prompts/...` is under `/var/home/user/temp/agent/tr-fan`.

### Branch

- Branch `translation-repair-rebased`,
  worktree `/var/home/user/worktrees/translation-repair`,
  head `5db89c1b61`,
  pushed,
  clean.
- Commits since the checkpoint headed "2026-10-13 at the hardware change",
  oldest first:
  - `c5982de7f8`,
    a merge of main (main at `b92cdef651`).
    Main had deleted `package/cli/markdown-lint` (replaced by the Rust `monochromatic-lint`)
    and split `module-test` into `module-test-diagnostic`,
    `module-test-expect` and `module-test-sandbox`.
    The branch kept `package/cli/markdown-lint` at that merge because `semantic-wrap.ts` imported it.
  - `211bb37537`:
    the export list in `package/module/test/src/package-exports.unit.test.ts` gains `DEFAULT_CONCURRENCY` and `caught`.
  - `72d5730bef`,
    a second merge of main (automatic,
    no conflict),
    which brought `monochromatic-lint` 0.1.2 and its pin
    (`cargo:monochromatic-lint` in `mise.no-env.toml`,
    `mise.toml` and `mise.lock`).
  - `9e0dfb68df` (tests) and `5db89c1b61` (code):
    `semantic-wrap.ts` wraps passages by running the pinned binary
    (`monochromatic-lint --config <file> --stdin --stdin-filename passage.md --fix`);
    `package/cli/markdown-lint` is deleted
    (the owner's direction:
    do not keep it,
    do not vendor it);
    the `satteri` dependency and the `satteri` and `@types/unist` catalog entries are removed;
    the lockfile and `package/config/pnpr/config.yaml` are regenerated.
- Verified for the linter migration,
  and where:
  the evidence is in scratch `/var/home/user/temp/agent/tr-fan/s174`
  (hand-backs `handback-n119.md` and `handback-n119-second.md`).
  Parity over 598 inputs:
  591 are equal byte for byte;
  the 7 that differ are one class,
  a break after a character outside the Basic Multilingual Plane,
  where the Rust side is right.
  A lone surrogate in a passage is refused before any child starts.
  In the agent's keyless worktree the whole unit suite gave 2,244 `[PASS]` lines and 0 `[FAIL]`,
  lint gave 0 warnings and 0 errors,
  49 scans passed,
  and a read-only replay of `TianqiChen66623` was unchanged.
  Not verified:
  none of these checks was rerun on the branch worktree itself.
- Consequences of the migration:
  stored text that the old rule wrapped without a break after an astral character gains that break when replayed.
  The pipeline digest changed (as with any code change),
  so a pass rebuys per-entry caches,
  and a pass into a runs directory that holds artifacts of another build is refused
  unless `TRANSLATION_REPAIR_ALLOW_GENERATION_DRIFT=yes`.
  The `monochromatic-lint` binary is outside what the digest covers,
  and the pinned version is checked once per process.
- The Rust linter's defect
  (its fix for `markdown/semantic-line-breaks` kept the space it should replace)
  was reported to the session that owns the Rust linter,
  at the owner's direction,
  and fixed there in 0.1.2 (main commits `58fd0ed375`,
  `c12da70140` and `a0a2f11da8`).

### In flight: the page-level consistency pass (uncommitted)

- All of it is uncommitted in the keyless worktree `/var/home/user/temp/agent/tr-fan/w148`,
  which sits at `211bb37537`:
  two merges behind in root files only,
  because the second merge and the linter commits touch no file the consistency work touches except through the suite.
  Nothing of it is on the branch.
- History of this round.
  The review counts reconcile with the checkpoint headed "2026-10-13 at the hardware change".
  That checkpoint recorded the third review's fixes uncommitted in `w140`,
  `w141` and `w142`.
  After it came the fourth review
  (n108,
  `/var/home/user/temp/agent/tr-fan/s160/review.md`)
  with fixes by n111 and n112 (`s163/handback-n111.md` and `s164/handback-n112.md`);
  those fixes and the earlier uncommitted work are the 49 uncommitted paths held as the baseline in
  `/var/home/user/temp/agent/tr-fan/w144`.
  Then came the fifth review,
  the merge of its four authors' fixes into `w148`,
  and the sixth review.
  So `w140` to `w142` are superseded by `w144` and `w148`.
  - The fifth independent review (agent n113,
    `/var/home/user/temp/agent/tr-fan/s166/review.md`) found F1 to F9.
  - Fixes by four authors were merged into `w148`:
    the mark rule (n116,
    `/var/home/user/temp/agent/tr-fan/s170/handback-n116.md`),
    nested blocks of one group
    (n114,
    `s168/handback-n114.md`,
    `s168/handback-n114-second.md` and `s168/handback-n114-third.md`),
    word joints beside digits (n115,
    `s169/handback-n115.md`),
    and four follow-ups (n117,
    `s171/handback-n117.md`).
  - Gate g94 on that tree:
    2,250 `[PASS]`,
    0 `[FAIL]`,
    lint clean,
    49 scans.
  - The sixth independent review (n118,
    `/var/home/user/temp/agent/tr-fan/s172/review.md`):
    F1 to F9 are fixed or turned into refusals;
    the new findings are G1 to G10.
    G1 is a wrong page:
    a splice could attach a loose apostrophe to a letter (`the cats's bowl`).
  - Fixes for the sixth review,
    first round
    (n120,
    `/var/home/user/temp/agent/tr-fan/s175/handback-n120.md`):
    G1 (a splice is `unavailable` when the edge character class beside a mark would change),
    G7 (a carry keeps the receiving place's straight or curly marks),
    G3 (a place that can never be spliced no longer makes a neighbouring paragraph unwritable),
    G4 (no stand inside a web or mail address),
    and the replay line now prints `consistency-groups=N read-at-generation=G`
    (for `TianqiChen666`:
    2 groups,
    generation 15).
    64 consistency and replay test files pass,
    lint is clean,
    and 49 scans pass.
  - Running at the time of this checkpoint:
    the second round of n120,
    for G2.
    A unit's spliced candidate is built from the edits that exist,
    and a place whose code cannot move keeps its words.
    This changes what generation 17 means;
    no pass has written generation 17,
    and generations 15 and 16 read as before.
    Its result is not yet known.
- Lead decisions and accepted limits:
  `/var/home/user/temp/agent/tr-fan/s159/ledger-notes.md` and
  `/var/home/user/temp/agent/tr-fan/s159/ledger-notes-sixth-review.md`.
  The briefs run from `/var/home/user/temp/agent/tr-fan/prompts/prompt-n113...` to
  `/var/home/user/temp/agent/tr-fan/prompts/prompt-n120-sixth-review-fixes.md`.
- The mark rule in one paragraph.
  A span is the words.
  The single marks are `'`,
  `‘`,
  `’` and `ʼ`;
  a single mark with a letter or digit on both sides is attached (part of a word),
  and other single marks,
  double quotation marks and corner brackets are loose.
  Every single mark at an edge of a judge's quote is dropped from the needle;
  a judge's edge mark only chooses between stands,
  and only when a strict majority of usable ballots copied it.
  A located rendering records `markBefore` and `markAfter`.
  A splice is `unavailable` when the two places differ in a beside mark,
  when the loose-mark sequences differ,
  or when the edge class beside a mark would change.
  Readers of generations 15 and 16 compare no marks.

### Next

Each step names where it runs and what it reads.
The worktree `w148` is `/var/home/user/temp/agent/tr-fan/w148`;
the branch worktree is `/var/home/user/worktrees/translation-repair`.

1.  Receive the second hand-back of agent n120 (the G2 round on `w148`).
    Its brief is `/var/home/user/temp/agent/tr-fan/prompts/prompt-n120-sixth-review-fixes.md`;
    the first round's hand-back is `/var/home/user/temp/agent/tr-fan/s175/handback-n120.md`;
    the second round's hand-back will be `/var/home/user/temp/agent/tr-fan/s175/handback-n120-second.md`,
    which did not exist when this checkpoint was written.
2.  Resolve apostrophes deterministically where no guess is needed,
    instead of refusing.
    This is the owner's question of 2026-10-13.
    It is planned as the next round on `w148`,
    and its brief is not written yet:
    the lead writes it as `/var/home/user/temp/agent/tr-fan/prompts/prompt-n122-apostrophes-resolved-without-a-guess.md`
    after n120's second hand-back.
    The plan:
    (a) a keyless count over the corpus's English pages of each shape;
    (b) an edge single mark is provably an apostrophe when its Markdown block holds no single mark
    that could be its partner,
    and then it travels with the word;
    (c) an apostrophe followed directly by a letter after emphasis or a bracket (`**cat**'s`) is a joint,
    so the token is one word;
    (d) the refusal stays as the fallback where a possible partner exists.
    It must land before the launch,
    because it changes the generation 17 record.
3.  Have an independent reviewer who wrote none of it read the rounds since the sixth review
    (the apostrophe resolution and G2 change the record).
    It reads `w148`,
    the sixth review `/var/home/user/temp/agent/tr-fan/s172/review.md`
    and the hand-backs of steps 1 and 2;
    the next free scratch directory is `s177`.
4.  Lay the changed paths of `w148` onto the branch head in the branch worktree,
    and gate there with `CPUQuota=2800%`.
    This gate is also the first run of the whole suite on the branch worktree after the linter migration.
5.  Write the ledger and mistake-prevention entries from
    `/var/home/user/temp/agent/tr-fan/s159/ledger-notes.md` and
    `/var/home/user/temp/agent/tr-fan/s159/ledger-notes-sixth-review.md`.
    The next free ids are B463 and M167.
    B463 goes in the file that holds B462,
    the last audit entry:
    `audit-ledger-tense-check-and-consistency-pass.md`.
    That file was split on 2026-10-14 (UTC) by subject:
    B462 is now in
    [`audit-ledger-consistency-pass-repeat-groups-and-the-carry.md`](../../package/translation-repair/doc/audit-ledger-consistency-pass-repeat-groups-and-the-carry.md),
    B463 to B477 in
    [`audit-ledger-consistency-pass-span-and-marks-beside-it.md`](../../package/translation-repair/doc/audit-ledger-consistency-pass-span-and-marks-beside-it.md),
    B478 to B491 in
    [`audit-ledger-consistency-pass-apostrophes-and-block-floors.md`](../../package/translation-repair/doc/audit-ledger-consistency-pass-apostrophes-and-block-floors.md),
    and B492 onward in
    [`audit-ledger-consistency-pass-page-wide-proof-forms-floors-and-limits.md`](../../package/translation-repair/doc/audit-ledger-consistency-pass-page-wide-proof-forms-floors-and-limits.md).
    M167 goes in the file that holds M166,
    the last process mistake:
    [`audit-ledger-process-agents-and-briefs.md`](../../package/translation-repair/doc/audit-ledger-process-agents-and-briefs.md).
6.  Commit the tests,
    then the code,
    with path lists generated from the worktree status immediately before each commit.
7.  Walk "Before a run launches" in `mistake-prevention.md`
    (the root `mise.toml` changed with the merges),
    then launch `TianqiChen66624` through the keyed task of
    [the corpus pass runbook](../runbook/translation-repair-corpus-pass.md)
    into a fresh runs directory.

### Open items

- `blockStandingOf` throws `RangeError` on 80,000 inline siblings
  (`/var/home/user/temp/agent/tr-fan/s172/probes/p7-standing-perf.out` and `p8-standing-stack.out`).
  It was present before this work;
  the owner of the item is the lead,
  and it waits until after the launch.
- The sixth reviewer overwrote the fifth review's probe outputs in `/var/home/user/temp/agent/tr-fan/s166/probes/`
  on 2026-10-13,
  because its copied probes wrote to the old paths.
  `s166/review.md` and the probe sources are intact.
- A year slot quoted bare (`1980` inside `1980's`) gets no slot,
  by the joint rule;
  quoted whole,
  it locates.
  Whether judges quote it bare is unmeasured.
- Recorded marks cannot be checked by a reader,
  which holds no English:
  a stated limit.
- Scratch worktrees `w128` to `w150` under `/var/home/user/temp/agent/tr-fan` are not yet in `/var/home/user/temp/agent/tr-fan/remove-worktrees.ts`;
  the owner runs the forced removal with `! node /var/home/user/temp/agent/tr-fan/remove-worktrees.ts force`.
- A second independent reading of the history-archive fixes is still owed
  (from the checkpoint headed "2026-10-13 at the hardware change").

### Environment

- Shared limits were set by another session at the owner's direction:
  a CPU cap of 28 threads since 2026-10-13 20:15 EDT
  (file `~/.config/claude-cpu-cap/threads`;
  [`claude-cpu-cap-and-thermal-watch.md`](claude-cpu-cap-and-thermal-watch.md)),
  and a shared `claude-code-bash` memory limit of 32 GB.
  Gate units start with `--property=CPUQuota=2800%` and `MemoryMax=16G`.
- The owner swapped the CPU and changed BIOS settings for stability before this segment.
- Markdown on main is linted by `monochromatic-lint`;
  the branch has no TypeScript Markdown linter any more.

### Identifiers

- Next free:
  worktree `w151`,
  agent `n121`,
  scratch `s177`,
  gate `g95`,
  ledger `B463`,
  mistake `M167`.
- `TianqiChen66623` is the last pass run;
  `TianqiChen66624` is not launched.

## 2026-10-13 at the hardware change: paused with the third review's fixes uncommitted

The owner stopped the session at about 18:00 UTC to change the machine's processor.
This checkpoint follows the one of 2026-10-12.
It is history now:
the checkpoint headed "2026-10-13 after the hardware change" holds the present state,
and the last list of this checkpoint,
which gives the next steps,
is superseded by it.
Nothing was running when it was written:
no pass,
no gate,
no agent.

- LANDED.
  The history archive is filed by what each file holds,
  in `c05ac36b8c` (the guard) and `33d0c875ae` (the docs),
  both pushed:
  twelve groups named by phase or span of dates,
  each an index over files named for one subject,
  none past 916 lines.
  An independent reading of the first re-filing found a file of unrelated items,
  four group names untrue of part of their files,
  a cut in the middle of a thought and index headings with nothing under them;
  each is fixed in that commit.
  A second independent reading of those fixes is still owed.
- THE THIRD REVIEW.
  The third independent review of the consistency pass proved eight defects in `e91da9ea8f`.
  Their fixes are written in three agent worktrees under `~/temp/agent/tr-fan`,
  each at `e91da9ea8f`,
  and none is committed,
  merged or recorded in the ledger:
    - `w140`,
      quote marks and word joints:
      a single mark at a quote's edge that pairs with nothing is taken off,
      the located span holds the apostrophe of the word at its edge and leaves a quotation mark where it is,
      and the Unicode hyphens and the modifier letter apostrophe join as the ASCII ones do.
      New modules `consistency-single-marks.ts`
      (named `consistency-corner-brackets.ts` since 2026-10-13 on the uncommitted consistency tree;
      it reaches the branch with that work)
      and `consistency-word-joints.ts`;
      `consistency-edge-marks.ts` rewritten;
      `consistency-fold.ts` and `text-fold-barrel.ts` changed.
      Build,
      type check,
      57 consistency test files and lint were clean (`s157/cycle-1.log`);
      the source scans were not run there.
      `CONSISTENCY_CACHE_VERSION` stays 4,
      on the word of `consistency-cache-version.ts` that a change to how quotes are located moves no key.
    - `w142`,
      standing and four cases that could not fail:
      a setext heading told by its lines,
      HTML table markup and inline preformatted text given a standing.
      Complete:
      lint clean,
      49 scans pass,
      patch in `s156/patch.diff`,
      report with four ledger paragraphs in `s156/handback-n106.md`.
      This one changes what a choice sheet says,
      so whether the cache version moves is decided at the merge.
    - `w141`,
      splice arbitration and the overlap record,
      stopped part way:
      a splice is held to every located place of another group,
      a place whose words an applied replacement changed is recorded `overlapped` and names what it gave way to,
      and the artifact generation moves from 16 to 17.
      Lint and the scans were clean after its last edit;
      the type check,
      the touched test files,
      the four probes and all 26 mutants were not run on its last build.
      `s155/handback-n105.md` lists every file,
      what is proven and the exact steps left,
      and the four choices its author left open to veto.
- THE GEMMA SEATS.
  Unchanged from the checkpoint of 2026-10-12:
  nothing waits on them.
- NEXT.
  After the restart,
  unlock the volume that holds `~/temp/agent` and confirm the three worktrees are as described
  (`git status` in each;
  nothing else holds their work).
  Finish `w141` by the steps in its report.
  Merge the three into one keyless worktree;
  they change disjoint files except the barrels,
  where `w141` adds `consistency-apply-barrel.ts` and `w140` adds exports to `text-fold-barrel.ts`.
  Run the full gate on the merged tree,
  have a reviewer who wrote none of it read the merged fixes,
  write the ledger entries
  (the first review's and the second review's decisions,
  the third review's eight defects,
  and the slips made on the way),
  then commit the cases first and the code second;
  a draft of the cases commit's message is in `s157/msg-tests.txt`.
  Then walk "Before a run launches" and relaunch `TianqiChen666` as `TianqiChen66624`,
  reading the refrain and the repeated sentence for one rendering each.
  The second reading of the history re-filing can run beside any of this.
