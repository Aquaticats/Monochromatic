# Translation repair handover 2026-09-06: checkpoints of 2026-10-03 to 2026-10-15, the trial model, the TianqiChen666 reads and the consistency pass

The four checkpoints from the handover to a trial model on 2026-10-03 to the consistency pass of 2026-10-15:
the trial handover with one case failing,
the reading of TianqiChen66622 with the owner's answers and the fixes before the next launch,
the four patches merged and reviewed with the relaunch of TianqiChen666,
and the consistency pass and the quoted-tense round that landed before TianqiChen66625 launched.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.

## 2026-10-15: the consistency pass and the quoted-tense round landed, TianqiChen66625 launched

This checkpoint is the lead's account of the state at the launch of `TianqiChen66625`,
updated from its first version at the launch of `TianqiChen66624`,
and it replaces the two checkpoints of 2026-10-13,
now filed in
[their own part](translation-repair-handover-2026-09-06-checkpoints-2026-10-13-hardware-change-and-linter-migration.md).
Every item is state,
not plan,
unless it sits under the heading "Next".
A path written `s<N>/...`,
`w<N>` or `prompts/...` is under `/var/home/user/temp/agent/tr-fan`.

### Branch

- Branch `translation-repair-rebased`,
  worktree `/var/home/user/worktrees/translation-repair`,
  pushed,
  clean.
- The rounds n159 and n161 landed together in two commits,
  oldest first:
  `5fe3173cc5` (tests) and
  `277145a862` (code).
  n159 built the tenth review's deferred findings
  (X1,
  X2,
  X4 to X11 and `'the '90s'`);
  n161 built the lead's rulings after the read of `TianqiChen66624` and the review n160:
  words quoted as their speaker wrote or said them keep the speaker's tense in every sheet that states the tense rule
  (`QUOTED_WORDS_RULE` and `QUOTED_WORDS_REMINDER` in `english-usage-policy.ts`);
  a consistency group line where nothing was carried ends with `because=` and the code's reason;
  and the review findings Y1,
  Y2,
  Y4 and Y6 to Y9.
  Cache versions after them:
  `SLICE_CACHE_VERSION` 36,
  `REFINE_CACHE_VERSION` 7,
  `TRANSLATE_SLICE_CACHE_VERSION` 17,
  `CONSOLIDATE_CACHE_VERSION` 23,
  `LANE_CONTEST_CACHE_VERSION` 8 and
  `PAGE_TITLE_CACHE_VERSION` 2,
  all set in `277145a862`;
  `CONSISTENCY_CACHE_VERSION` 4 stays because its key hashes the sheet,
  and `PAIRING_CACHE_VERSION` 3 because the pairing sheets carry no tense rule
  (read from `mise run //package/module/translation-repair:cache-account-audit` (moved on 2026-10-18 to
  `//package/translation-repair/slice-cache:cache-account-audit`) after the commit).
  The commit that carries this checkpoint changes docs only.
- The consistency pass had landed earlier the same day in three commits,
  oldest first:
  `7ae9914d3d` (tests),
  `1fb06389e1` (code) and
  `44c8f57784` (docs:
  ledger B463 to B574 and M167 to M209,
  the consistency ledger filed by subject,
  and the troubleshooting doc on the V8 first-call compile at the stack edge).
- What that code commit does:
  a consistency splice carries an apostrophe in at an edge only when the whole-page proof reads it as an apostrophe on the page as it ships;
  the refusal run past an edge steps over what the prose view does not read as words;
  a page whose English holds a character reference that decodes to a quotation-like character refuses every splice;
  a span edge treats an escape or a reference as one unit and refuses an edge inside a shown word;
  and typography reads prose from the page parse
  (`typography-prose-mask.ts`),
  one mask for every typography reader.
- Cache versions at the launch of `TianqiChen66624`:
  `SLICE_CACHE_VERSION` 35,
  `REFINE_CACHE_VERSION` 6,
  `TRANSLATE_SLICE_CACHE_VERSION` 16,
  `CONSOLIDATE_CACHE_VERSION` 22 and
  `LANE_CONTEST_CACHE_VERSION` 7,
  all set in `1fb06389e1`;
  `CONSISTENCY_CACHE_VERSION` stays 4,
  because that cache stores ballots only;
  `PAIRING_CACHE_VERSION` 3 and `PAGE_TITLE_CACHE_VERSION` 1 stay,
  because the code commit's parse changes return the same tree for every page that parsed before
  (a stack exhaustion that used to escape is now the grammar's refusal,
  and `parseSliceBody` keeps its old reading of invisible-only lines by default),
  and the linter migration's wrap feeds only stages whose versions moved after it.
  Read from `mise run //package/module/translation-repair:cache-account-audit` (moved on 2026-10-18 to
  `//package/translation-repair/slice-cache:cache-account-audit`) on 2026-10-15.

### Verified, and where

- Gate `g98` ran on the keyless worktree `w153` at the branch head `277145a862` (status empty):
  2,435 `[PASS]` lines and 0 `[FAIL]` in the suite,
  lint 0 warnings and 0 errors,
  49 scans passed.
- Read-only replays on the n161 build:
  `TianqiChen66623` gave
  `wordings=24 silent=0 chars=8607=expected missing=0 consistency-groups=2 read-at-generation=15`
  and `TianqiChen66624` gave
  `wordings=24 silent=0 chars=8490=expected missing=0 consistency-groups=2 read-at-generation=17`,
  each with `matched=1 settledWithNoPage=0 pageWithNoArtifact=0 declined=0`
  (`s232/verify-66623.log` and `s232/verify-66624.log`).
- "Before a run launches" was walked again on 2026-10-15 for `TianqiChen66625`:
  the branch was clean and level with its upstream;
  the suite and lint lines are gate `g98`'s;
  the cache-account audit's lines are in the Branch list;
  no source file holds a control byte
  (the same search found a scratch file written with one);
  and the plan printed `PLAN ok tip=277145a862` with one entry pending.
- An independent review of the n159 and n161 commits,
  n162,
  runs alongside the pass
  (worktree `w152` at `277145a862`,
  scratch `s233`).
  Source it leads to lands only after `TianqiChen66625` settles.

At the launch of `TianqiChen66624`:

- Gate `g97` ran on the keyless worktree `w153`,
  whose tracked files equal the branch head `44c8f57784` (`git diff` empty)
  and whose untracked files were copied to the branch byte for byte:
  2,394 `[PASS]` lines and 0 `[FAIL]` in the suite,
  lint 0 warnings and 0 errors,
  49 scans passed.
- A read-only replay of the `TianqiChen66623` artifact on `w153` gave
  `matched=1 settledWithNoPage=0 pageWithNoArtifact=0 declined=0` and
  `wordings=24 silent=0 chars=8607=expected missing=0 consistency-groups=2 read-at-generation=15`.
- "Before a run launches" was walked on 2026-10-15:
  no source file holds a control byte
  (the same search found a scratch file written with one),
  and the plan printed `PLAN ok tip=44c8f57784` with one entry pending.

### The launch

- `TianqiChen66624`:
  `--only TianqiChen666` through the runbook's unit form,
  into the fresh runs directory `/var/home/user/temp/agent/corpus-pass-TianqiChen66624`,
  log `/var/home/user/temp/agent/corpus-pass-TianqiChen66624.log`,
  unit `corpus-pass-corpus-pass-TianqiChen66624`.
- It settled on 2026-10-15 with every check clean,
  and the consistency pass carried one repeat.
  Its page tells her last message,
  quoted in her own words,
  in the past,
  as `TianqiChen66623`'s did (ledger M220).
  The read is in
  [the pass log for 2026-10-15](../planning/translation-repair-openrouter-2026-09-03-2026-10-15.md).
- `TianqiChen66625`:
  `--only TianqiChen666` through the runbook's unit form at `277145a862`,
  into the fresh runs directory `/var/home/user/temp/agent/corpus-pass-TianqiChen66625`,
  log `/var/home/user/temp/agent/corpus-pass-TianqiChen66625.log`,
  unit `corpus-pass-corpus-pass-TianqiChen66625`.
  It is the first pass with the quoted-tense rule.
- It settled on 2026-10-15 with every check clean,
  and her last message keeps her present tense line for line.
  The consistency pass carried the refrain to three of its four places
  (the fourth place's carry choice split three ways)
  and left the repeated sentence of source lines 35 and 55 as it stood
  (its majority quoted three different spans).
  Seven replies were refused for naming a shown label in another form,
  such as `Occurrence 3` for `OCCURRENCE 3`.
  The read is in
  [the pass log for 2026-10-15](../planning/translation-repair-openrouter-2026-09-03-2026-10-15.md).
  Hyper's balance fell from 140 to 27 over the pass.

### Next

1.  The investigation n170
    (brief `prompts/prompt-n170-consistency-pass-voices-66625.md`,
    notes `s241/notes-n170.md`)
    read the consistency pass of `TianqiChen66625`.
    The lead's rulings,
    each fixed in the round after n169:
    a reply naming exactly one shown label in another form counts as that label;
    the sheets list the exact labels;
    the run-off's final result gets its own log line;
    a carry choice that splits is run off among its tied candidates,
    as the owner's rule of 2026-10-11 does for contests;
    a span that a majority of the chosen place's quotes contain is located;
    the abandonment line reports answer and reasoning characters,
    not wire length;
    the re-ask's own failure is logged.
2.  The review n162 of the n159 and n161 commits
    (brief `prompts/prompt-n162-review-of-n159-and-n161.md`,
    findings Z1 to Z11 in `s233/review.md`,
    none of them on a corpus page)
    is ruled:
    every finding is fixed,
    the lead's own Z4 by the commit comments on `5fe3173cc5` and `277145a862`.
    The fix round n166
    (brief `prompts/prompt-n166-z-findings.md`,
    worktree `w161`,
    scratch `s237`)
    handed back green,
    and two follow-up rounds in the same worktree finish the empty-link reading it began:
    n168
    (scratch `s239`,
    green)
    and n169
    (brief `prompts/prompt-n169-empty-link-twins.md`,
    scratch `s240`,
    green:
    suite 2,450 pass and 0 fail,
    no real change on any corpus page).
    None of the three is committed.
    Two rounds run on top of them at the same time:
    n171 in `w161`
    (brief `prompts/prompt-n171-consistency-voices.md`,
    scratch `s242`)
    builds the rulings on the investigation n170,
    and n173 in `w164`,
    a copy of `w161` after n169
    (brief `prompts/prompt-n173-labelled-links-read-as-shown.md`,
    scratch `s244`),
    makes links with a label read as their plain-text twin
    and gives every fold caller the page's definitions.
    The fold files are reserved to n173,
    and the lead merges n173 into `w161` after both hand back.
    Then a review of all five rounds by an agent that wrote none of them,
    a gate,
    the landing,
    the cache-account audit
    (it reads only committed versions),
    and a relaunch.
    The provider Hyper held 27 of balance after `TianqiChen66625`,
    which spent 113;
    the next pass runs with whatever it holds,
    and a dry seat goes dark.
3.  The docs round n163 landed in `537d79eaaf`
    (ledger B616 to B628,
    M221 to M223),
    and the docs round n172 in `3fdfa7422e`
    (M224 to M233:
    the lead's three mistakes of 2026-10-15 and the rule breaches of n162 to n168;
    B619 brought up to n169's result).
    The next docs batch files the B entries of n166 to n173 with their commits once they land,
    the breaches of n169 and n172,
    and the lead's mistakes in the n172 brief;
    the lead's notes for it are in `s228/next-docs-batch-notes.md`.
    On 2026-10-17 the owner,
    shown that 37 docs cite the frozen builds and that the open issues 527 and 541 cite the evidence folders,
    chose to delete as first approved:
    the 195 ignored `.frozen-*` folders and 16 dangling links are gone from the branch worktree (M224).
    Four links in the comparison evidence point into a removed frozen folder and were left in place.
4.  Open items that still stand from the checkpoint headed "2026-10-13 after the hardware change":
    `blockStandingOf` throws `RangeError` on 80,000 inline siblings;
    whether judges quote a year slot bare is unmeasured;
    the history-archive fixes in `730c503608` were read by n165
    (brief `prompts/prompt-n165-history-refiling-third-reading.md`,
    report `s236/review.md`),
    which found the second reading's findings fixed and the cuts exact;
    its seven low findings were fixed in `d6f7f9ae12`
    (round n167,
    hand-back `s238/handback-n167.md`),
    and the top index clauses the lead wrote there have had no independent reading.
    From the review of n159 (Y5):
    the page's definitions miss those inside a list,
    a blockquote or an HTML block,
    and a footnote joined to a link definition by one newline;
    no corpus page has either.
    From the read of `TianqiChen66624`:
    the front matter ships as the archive's,
    whose `desc` drops the game of hide-and-seek,
    and whether the pipeline repairs front matter is undecided;
    the title rule sets the card game Uno in italics.
    The spent scratch worktrees are listed in `remove-worktrees.ts`;
    the owner runs its `force` mode.

The ledger entries for the rounds n150 to n157
(B575 to B615 and M210 to M219)
landed in `2bb5d72bb5`.

### Identifiers

- Next free:
  worktree `w160`,
  agent `n164` (`n162` is the review and `n163` the docs round in Next),
  scratch `s235` (`s233` is the review's and `s234` the docs round's),
  gate `g99`,
  ledger `B616` and mistake `M221`,
  both handed to `n163`,
  which reports the next free ones.

## 2026-10-12: the four patches merged, reviewed, and the relaunch of TianqiChen666

This checkpoint follows the one of 2026-10-11 and says what landed after it.
Times are UTC.
On 2026-10-11 the owner said to make the remaining decisions without asking;
each choice an agent left open is recorded beside its ledger entry.

- LANDED.
  L6's sole-lane verdict for insertions the archive lacks (ledger B425,
  artifact generation 15);
  a seat that throws on every call withheld from its benches (B420 to B422);
  the forfeit read by majority (B424);
  the tense candidates shown to every judge of an entry that records a death
  (B419,
  reviewed in B432 and B433);
  the repeated-source-line finder (B423,
  reviewed in B434 to B436);
  and the page-level consistency pass that asks the wide bench to choose one rendering of each repeat
  and carries it to every place by code (B431,
  reviewed in B443 to B446,
  with tests of its own in B437 to B440).
  A damaged `info.yml` now ships its entry with no death recorded (B441),
  as does a `died` of an unreadable kind (B447),
  and the window trial asks the sheets the pass asks (B442).
- MERGING.
  Pieces of work built apart broke each other's fixtures twice
  (ledger M153 and M156);
  the rule is now in `mistake-prevention.md`,
  "Work merged from parallel briefs".
- THE LAST PASS.
  `TianqiChen66623` at `9c0406f113`,
  launched through the runbook's unit form with stderr kept,
  settled in 7,339,457 ms and was read the same day
  (the pass log's "TianqiChen66623 read,
  2026-10-12"):
  the passage TianqiChen66622 shipped in the present tense now reads in the past;
  the two Gemma seats were refused with an expired credential (HTTP 401) and withheld after their first asks,
  so rounds without quorum fell to 4 of 503;
  and the consistency pass asked the bench about both repeat groups and changed nothing
  (ledger B449,
  B450).
- LANDED.
  The consistency pass's rebuild
  (agreement by located span,
  a run-off on a tie,
  the choice carried through writers,
  the bench and every floor;
  generation 16)
  in `6e5afaa41a` and `3085924af4`,
  gate g80 green on the merged head
  (2207 passing cases and the 8 environmental `bench-sample-draw` failures,
  lint clean,
  49 of 49 scans),
  and the phase's wait on the writer bench in `4ec390fe87` and `f8d47c38aa`.
- LANDED.
  An independent review of the rebuild found no path to the page around the bench and the floors,
  and twelve defects (ledger B451 to B462);
  one agent fixed all of them,
  merged as `d9bcaf2787` (cases) and `1a31a4a244` (code),
  with `CONSISTENCY_CACHE_VERSION` at 3.
- DOCS SPLIT.
  The owner ruled that a Markdown file holds at most 1,000 lines (ledger M163).
  A first split cut the 38 translation-repair docs past it by size into numbered parts in `a2eb509e92`,
  and the owner rejected it (ledger M165).
  Those docs and the history archive are now filed by subject,
  phase or span of dates:
  each keeps its path as an index,
  and its files stand beside it,
  named for what they hold;
  this snapshot is one of them.
  The history archive's numbered segments were joined again by group and filed the same way.
  `doc-line-limit.unit.test.ts` holds the limit,
  refuses a file named by a counter,
  and refuses a file its record's index does not link.
- IN FLIGHT.
  The second review's five defects are fixed in `bfd89d4baa` (cases) and `e91da9ea8f` (code),
  with `CONSISTENCY_CACHE_VERSION` at 4,
  and a third independent review of that fix is running.
- THE GEMMA SEATS.
  Bedrock answers both with HTTP 401 for an expired token.
  The owner does not top up Bedrock (2026-09-07),
  and nothing waits on them:
  the run withholds both after their first failed calls.
  A new key would bring them back,
  and is the owner's to issue or not.
- NEXT.
  Read the gate and the review,
  fix what the review finds,
  read the replay of the TianqiChen66623 artifact's ballots through it,
  then walk "Before a run launches" and relaunch `TianqiChen666` as `TianqiChen66624`,
  reading the refrain and the repeated sentence for one rendering each.

## 2026-10-11: TianqiChen66622 read, the owner's answers, and the fixes before the next launch

The checkpoint of 2026-10-03 handed the package to a trial model;
this one says what changed after it.
Times are UTC.

- THE AUDIT.
  The trial branch merged by fast-forward on 2026-10-05
  ([the trial log](translation-repair-mimo-trial-2026-10-03.md),
  "Outcome:
  merged on 2026-10-05"),
  and the audit went on through ledger entry B416:
  T8 closed on the final whole-suite census
  (2,047 passes;
  4 library files with 10 stretches no case runs,
  each classified in the ledger).
- THE LAST PASS.
  `TianqiChen66622` at `f58e30822`,
  launched through the runbook's task on the owner's go-ahead,
  settled in 7,377,286 ms and was read the same day
  (the pass log's "TianqiChen66622 read,
  2026-10-11"):
  both Bedrock Gemma seats answered none of their 536 calls,
  rounds over six seats needing five then waited for every ask and bought top-up rounds,
  and slice 4 shipped the archive's present-tense sentence over a contest that called it flawed 4 to 1.
  The launch lost every stderr line,
  `warn` lines among them,
  so why the Gemma seats failed is not known (ledger M147);
  the runbook's launch step now gives a systemd unit form that keeps both streams,
  with a check read back from systemd right after launch.
- THE OWNER'S ANSWERS.
  Two batches,
  each in the package's
  [design commitments](../../package/translation-repair/doc/design-commitments.md)
  under 2026-10-11:
  a majority of flawed contest ballots runs the slate off and forfeits an eligible standing past an undecided gate;
  a tie the standing drew no ballot in is run off among the tied;
  repeated source lines are rendered one way on a page,
  a slice where the archive stands included;
  the past-tense rule gets candidates shown to judges,
  never a refusal;
  and a dictionary entry for 扑面而来,
  whose sense the corpus corrected (ledger M148).
- LANDED.
  The run-off rules (`52a38dc6c`,
  `c66cb5e47`)
  and the majority forfeit (`46fe30664`,
  `b640f2392`),
  with the consolidation cache at version 21;
  the 扑面而来 entry and Hyper's price rows read again that day (`6bc235960`,
  `d02c0cfc3`).
- IN FLIGHT.
  Agent patches not yet merged,
  each in its own worktree:
  L6's sole-lane verdict for insertion slices;
  the tense candidates shown to judges;
  withholding a seat that throws on every call,
  and the error text of every throw reaching the log;
  and the first half of the consistency pass,
  which finds repeated source lines and maps them to slices.
  The second half,
  which asks the judges and applies their choice,
  is designed (choices adopted open to veto) and starts once the first merges.
- NEXT.
  Merge each patch as it is handed back,
  red commit before fix,
  run the full gate on the merged head,
  write the docs batch (ledger B417 onward,
  the M134 recurrence of 2026-10-11),
  then walk "Before a run launches" in `mistake-prevention.md` and relaunch `TianqiChen666`
  into a fresh runs directory through the runbook's unit form.

## 2026-10-03: handover to a trial model, one case failing

The checkpoint of 2026-09-29 left T8 with its bundle agents at work;
this one says what changed after it and who works next.

- T8.
  The agents given the first five of the 19 bundles cut from `census-3G9C58` all stopped part way at a usage limit.
  Bundles 1 and 2 landed after review (`23090a91d`,
  `ba6da4640`);
  of bundles 3 to 5,
  only two test files reached the branch,
  committed by the owner in `98e94e584`.
  The cut lived in session scratch,
  so the next batch reads against a fresh whole-suite census.
- FINDINGS.
  B122 (sorts outside code-point order),
  B124 (a wet refuser held for no time ended a call as every provider dry)
  and B125 (a decision seat's distribution recorded as an objection) are fixed.
  B123 (a link literal the parser leaves without a position makes the footnote relabel refuse the page)
  had been queued only in `23090a91d`'s message;
  it is in the audit ledger now,
  reproduced on the build of `98e94e584`.
- RED.
  `98e94e584` fails one case:
  an empty-label link on a contributor line reads as a contributor name.
  It is B126,
  the trial's first step.
- NEXT.
  MiMo 2.6 Pro works the package on the branch `translation-repair-mimo-trial` from 2026-10-03 for about 48 hours,
  and
  [`translation-repair-mimo-trial-2026-10-03.md`](translation-repair-mimo-trial-2026-10-03.md)
  is its start point and log.
  The owner's answer on its authority:
  "Same authority as Claude,
  but I don't think it will finish the audit fast enough (in 48h) anyway."
  The launch still waits for T8 unless the owner says otherwise.
