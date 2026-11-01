# Audit ledger: XingZ6023 shipped no page

The entry `XingZ60` settled every slice in 8,821,912 ms on the XingZ6023 pass and ended
`TALLY ... status=INCOMPLETE`,
`DONE processed=0 of pending=1; artifacts=0/1`:
no page and no artifact.
Two causes,
a count that missed and a guard that threw,
and the three smaller findings the round of 2026-11-01 (UTC) made on the way.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

The owner's rules that decide every item here:
"no matter what,
we must ship";
"No valid wording:
keep the archive at that slice and ship" (2026-09-27);
a page-agreement check ships with the defect reported;
"Ship interim,
resumable" and "Ship the two-ballot text (Recommended)" (both 2026-10-29).

## B1180: the count of the translate winner's ballots compared the wording the judges saw with the wrapped delivery

Red in `de6170c266`,
fixed in `e081ca5f5f`.

The owner's answer "Ship the two-ballot text" ships a translate-lane winner two judges chose
where the final round cannot seat two.
`translateWinnerBallotsOf` (`corpus-pass/src/corpus-run/pass-translate-winner-ballots.ts`) counted a slice only if
the round's `stageResult.text` equalled the comparison row's `translateText`.
The row holds the semantic wrap's output (`wrapTranslateRecords`),
so every wording the wrap changed read as zero ballots.
XingZ6023 slice 44,
the translate winner "Trans-whatever-Light[^3]" with two ballots,
was one of them,
and so were slices 13,
68 and 69;
slice 67,
whose text opens with a `<details>` block,
was the one the wrap left alone and the rule fired for.

Measured,
not argued.
Over the real XingZ6022 artifact (`artifacts/XingZ60.json`) the old rule counts 38 of the 116 translate records,
the new one 95,
57 gained and none lost,
and slice 44 goes from no count to 2.
Over the 101 fresh translate records of the XingZ6023 cache,
`stageResult.text` equals the unwrapped `outputText` in every one,
so none of the misses was a protected run restored onto the judged text (`restoreTargetOnlyRun`):
the wrap alone caused them.

The fix reads the lane's own delivered text.
The records the consolidation is handed (`lanes.translate.slices`) are the output of `wrapTranslateRecords`,
so a record's `outputText` is the wording the delivery ledger carries,
byte for byte,
and the count compares that with the row.
No fold of the wrap is written,
and none was needed to prove an inverse:
the wrap runs once,
inside the lane,
and this reads its output.
A slice is also required to carry `disposition: 'stage-result'` and `changed: true`.
Without the first,
a choice a guard refused (`refused-quote-loss`,
`refused-declared-name`,
`refused-alignment`)
would be credited,
since the record then delivers the archive and the row equals it.
Without the second,
a wording the wrap turned into the archive's own bytes would be credited,
though the page shows no change.
A trimmed delivery (`trimmedReplacements`) still differs from `outputText` and still earns nothing.

Every other reader that sets a lane text beside a comparison row,
found with `rg` for `translateText` and for `stageResult` under `package/translation-repair`:
`would-ship-text.ts:466` and `slice-cache/settled-tally.ts:222` compare the two lanes' rows with each other;
`artifact-pool/artifact-two-lane-comparison.ts:79` and `:133` compare a row or an outcome with its archive or its twin;
`artifact-pool/artifact-two-lane-row-equality.ts:214` compares two recorded rows;
`consolidate-standing.ts`,
`consolidate-driver.ts:378` to `:500` and `consolidate-wire.ts` pass the row's text on;
`translate-slice.ts:307` compares the round's text with the archive before the wrap exists.
None compares the round's text with a wrapped row,
so none has the fault.

The test builds a quotation with a note mark and a second paragraph in the shape of slice 44,
asserts that the real wrapper (`monochromatic-lint` 0.4.0) turns the judged form into the delivered form,
and expects 2 ballots counted.
Red on the old build:
`expected [] to deeply equal [ [ 44, 2 ] ]`.
The path from the count to the shipped text is the existing cases of
`consolidate/src/translate-winner-ships-unseated-slate.unit.test.ts` and
`corpus-pass/src/consolidate-driver-translate-winner.unit.test.ts`;
they were not duplicated.

## B1181: two page guards threw where the owner's rule is to ship with the defect reported

Red in `de6170c266`,
fixed in `ad31320398`.

After the lanes,
the contest and the consolidation,
`pass-entry.ts` called `assertPageGuards`,
which called `assertCarriedInsertionsRemain` and `assertPageFootnotesIntact`.
Each threw `TranslationRepairInterruptedError`.
`entryErrorOutcome` files every such error as `stopped` (`INCOMPLETE`),
and `shipInterimOnDryStop` shipped an interim page only for the every-provider-dry refusal,
so an orphan definition cost the whole entry its page.
The orphan itself was the miss of B1180:
slice 44 kept the archive,
which has no `[^3]`,
while slice 115 shipped the translate lane's `[^3]: That is, Transcend Lights.`.
The lane assembly had found no withdrawal that clears it and shipped,
reporting `assembly-footnote-reported orphan-definition gfm 3 ... no withdrawal clears it`.
`6262807c3b` (2026-10-29) stopped the useless withdrawal of slice 115 that had hidden the orphan from the throw.

Both guards are now checks of the published page.
`publishFixedPage` runs them as steps (`page-check/src/corpus-run/publish-page-guard-steps.ts`),
their refusals are the naming errors `PageFootnoteIntegrityError` and `CarriedEvidenceLostError`,
and `publishDefects` collects them as the `page-footnote-integrity` and `carried-evidence` defects,
two members added to the `PublishCheck` union.
The DEFECTS line names the check beside the page's own `page-footnotes`,
the log carries `publish: shipping with defect page-footnote-integrity: ... orphan-definition gfm 3`,
and a lost carried region is written to the log in full,
since a defect's message names the entry,
counts and slice numbers only.
The pass no longer calls the guards (`pass-page-guards.ts` and its test are deleted),
and `persistSettledEntry` hands the carried passages to the publisher.
Publish defects are not recorded in the artifact (`rg` finds no defects field in the `artifact` package),
so there is nothing to record there;
adding a field would be an artifact generation change (B1186).

Red:
2 of the 3 cases of `publish-page-guards-report.unit.test.ts` fail on the old build
(`expected [ 'page-footnotes' ] to deeply equal [ 'page-footnotes', ... ]` and the carried case),
and the two rewritten files fail at load for the export they name.
A `settleEntry` case in `pass-entry.unit.test.ts` reaches the stop itself
(an untranslated passage whose original carries the definition of note 3 with no reference):
outcome settled,
`TALLY ... status=SETTLED`,
a DEFECTS line naming the check,
the warning naming
`orphan-definition gfm 3`,
the artifact and the page written.
That case was written after the fix and has no red run.

## B1182: the two footnote readers judged different pages

Fixed in `ad31320398`.

`assertPageFootnotesIntact` read the page with `includeSource: false`,
the page with no original in it.
The assembly proof (`assembly-withdrawal-proof.ts`) counts a withdrawn anchor as carrying its original,
which is how the page ships.
The check now reads the composed page the publisher writes (`composeSourcePage`,
the same string the
`page-footnotes` step reads),
the original of an untranslated passage under its note included.

What that changes:
a reference or a definition that lives in an original shipped under its note is now read,
and counts as introduced over the archive,
so the defect appears in the report where before no throw could name it;
the integrity check and the page-alone check can no longer disagree about what the page is,
since both read one string;
nothing withdraws on the check,
so the old reason for not reading the original
(its footnote graph must not withdraw a slice the archive does not hold) is gone.
The lane assembly's proof reads a lane-level splice,
because the consolidation has not run when the lane assembles,
but it models an anchor the same way,
and on XingZ6022 and XingZ6023 the two now name the same orphan.

`assertCarriedInsertionsRemain` stays on the page with no original.
A region found only in the original an untranslated passage ships under its note is not evidence the lanes wrote,
and a carried passage is one the lanes admitted as already rendered in English;
the existing case that says so still stands,
now expecting the refusal.

## B1183: every reason of the interruption error, and where each can leave an entry with no page

Red in `de6170c266`,
fixed in `ad31320398`.

`TranslationRepairInterruptedError` had three reasons (`translate-stage/src/translation-repair-interrupted-error.ts`).
`entryErrorOutcome` files each as `stopped`,
and the entry's `catch` ships an interim page only for the dry refusal and,
since this round,
for the third reason.

- `carried-evidence-lost` was thrown in one place,
  `carried-insertion-completeness.ts`.
  It stopped an entry with nothing written.
  Now the `carried-evidence` defect (B1181);
  the reason is removed.
- `page-footnote-integrity` was thrown in one place,
  `page-footnote-integrity.ts`.
  It stopped an entry with nothing written.
  Now the `page-footnote-integrity` defect (B1181);
  the reason is removed.
- `provider-unavailable` is thrown at four sites,
  each for a stage that heard no voice:
  `translate-stage/src/translate-stage-repair.ts:255` (the absence judging of the translate lane),
  `consolidate/src/consolidate-slice-buy.ts:445` (every producer of a barred standing silent),
  `archive/src/archive-block-review-stage.ts:322` (the archive block review,
  called from the preparation by
  `corpus-pass/src/corpus-run/pass-prepare.ts:518`)
  and `corpus-pass/src/corpus-run/pass-insertion-admission.ts:312` (an unheard roster at the coverage round).
  Each stopped an entry with nothing written,
  whatever providers were still wet.

The same rule covers the stop as covers a dry one:
the entry stops,
stays open and ships what it settled.
`shipInterimOnDryStop` is renamed `shipInterimOnStop` and ships the interim page for a
`provider-unavailable` interruption too;
the interim record's reason widens to `provider-unavailable` (`glossary/src/corpus-run/interim-entries.ts`)
and the `INTERIM` line prints it.
The translate lane and the consolidation run after the slicing exists,
so a stop there now ships a page.
The archive block review and the insertion admission run in the preparation,
before the slicing exists,
so there is no slice to ship and nothing is written,
for this stop as for a dry one;
the archive stands.
The case `PAUSES when coverage admits source gap but every translator voice is lost`
asserted that the published tree stayed empty,
which was the defect,
and now asserts the page and the record.

This is the narrower reading.
The wider one,
which the owner may take,
is a page record for a stop before the slicing:
ship the archive itself as the entry's interim page and record it.
It is proposed and not built.

## B1184: other throws that can still end an entry with no page

Open.

Outside the brief's list,
found while reading `entryErrorOutcome`:
`NaturalnessCompletenessError` (`page-check/src/corpus-run/final-naturalness-completeness.ts`) is thrown by
`persistSettledEntry` before the page is written when body polish lacks approval;
`UnparseablePageError` (`assertPageParses`) is thrown after the defects are reported,
and is left a refusal on purpose,
since a page that does not parse breaks the site build;
`UnansweredContestSliceError`,
`SliceSpliceError`,
`PublishedPageDisagreesError`,
`CollapsedHeadingError`,
`ContributorCompletenessError`,
`DroppedDestinationError`,
`ArchiveOriginalCompletenessError`,
`FrontMatterCompletenessError`,
`NaturalnessRepairInterruptedError`,
`PromptPayloadStoreError` and
`VisualEvidenceInterruptedError` are in the same `stopped` list.
Several of these are publish checks that `publishDefects` already collects when raised inside it;
which of the rest can fire before a page is written was not read in this round.
Whether any is a verdict on content that the owner's rule would ship with the defect reported is the next census.

## B1185: the pre-launch meter reading printed bedrock wet while the run holds Bedrock off

Red in `de6170c266`,
fixed in `4b74c23aa1`.

The runbook's pre-launch step (`doc/runbook/translation-repair-corpus-pass.md`,
Setup step 5) runs
`run:budget-sample` and `run:meter-report`.
The sampler built a Bedrock client whenever a key existed and required the key,
so the reading said `bedrock wet` while the run's own first `METERS` line reads `bedrock=dry`
and its fourth line says `PROVIDER OFF bedrock`
(`run/src/corpus-run/providers-off.ts:41-47`).

`readProviderKeys` and `sampleBudgets` take the providers left off,
`PROVIDERS_LEFT_OFF` by default,
as the three other sites that honour it do (`run-providers.ts`,
`required-providers.ts`,
`index-run-setup.ts`).
An off provider's key is read as unset and never required,
no client is built for it,
so the budgets read it as unconfigured,
dry with nothing to report,
and the summary says `avoid bedrock`.
The sample writes the run's `PROVIDER OFF` line for each,
one spelling shared through `providerOffLine`.
No existing `meter-sample-read` grammar changed:
`bedrock` still parses as a meter state.
Tests use a fake transport only.
Red:
4 of the 5 new cases fail on the old build (the key read as unset,
the absent key,
the reading and the sample
without a key);
the case that names off as an empty list and still expects the refusal passes on both builds.
One further case,
which pins the left off wording of the refusal for another key,
was added with the fix.

## B1186: cache versions and the artifact generation

Fixed in `e081ca5f5f`.

The cache account audit (`slice-cache:cache-account-audit -- --runs-under` the agent folder) lists eight constants:
consolidate 43 (set 2026-10-25T11:05Z),
consistency 10 (2026-10-31T06:36Z),
translate slice 36,
slice (repair) 50,
pairing 3,
lane contest 18,
page title 9 and refine 19.
It counts 14,049 slice-cache records under 430 runs directories,
the newest 2026-10-31T10:04Z (`XingZ6023`),
and 0 of 8 versions set after it.
So a change to what a stage accepts moves its version,
and a change that nothing cached reads does not.
The 295 commits the audit prints as named by no account were not re-read;
this round answers for its own changes.

- B1180 changes the count the consolidation reads at the unseated final round and in the present-tense swap.
  The key does not carry it,
  and records exist after the number was set.
  `CONSOLIDATE_CACHE_VERSION` moves 43 to 44,
  with its account beside the constant
  and the pinned key literal moved.
- B1181 to B1183 and B1185 act on the published page,
  the interim record and a sampler,
  after every cache and outside every key.
  No version moves.
- The translate slice version stays at 36:
  the lane's records are the same,
  the count is read off them after the wrap.
  The repair slice,
  refine,
  lane contest,
  pairing,
  consistency and page title versions are not reached.
- The artifact generation stays at 22 (`ARTIFACT_SCHEMA_VERSION_V22`):
  no field is added,
  removed or reinterpreted;
  the artifact records no publish defect (B1181) and no winner-ballot count.
  The pipeline digest moves with the build,
  as it does for every change.

## B1187: what stays open, and the slips

Open:
the effect of the new count on a shipped page was measured as a count,
not as a page,
and needs a run to see (95 of 116 slices of XingZ6022 now carry ballots where 38 did,
which changes the decision only where the final round cannot seat two judges);
B1184;
the pre-slicing stop (B1183);
`page-check/package.json` still lists `translation-repair-translate-stage` among its dependencies,
and only a test imports it now;
moving it is a lockfile change and was not made.

Slips of the round:

- The subject of `de6170c266` files 5 cases of the page guards where it holds 14
  (5 footnote,
  6 carried,
  3 publish) and says all failed,
  when three controls pass on the old build.
  Corrected in the body of `e081ca5f5f`.
- The subject of `ad31320398` says 25 files where git reports 24.
- The subject of `4b74c23aa1` says 6 files in `run` where it changes 7 (8 files in all).
- The subject of `9f51528e5a`,
  the commit that cleared the findings of the source-scan suite,
  says 8 files where git reports 7.
  Four of this round's five commit subjects miscounted their files or cases,
  because the counts were written from memory and not read off `git diff --stat` first.
- The whole source-scan suite was run twice in full:
  once after the fixes,
  where it found the 7 cases `9f51528e5a` clears,
  and once at the end.
- The tree had no `.cache/toolchain-bin`,
  so the wrap-dependent tests could not find the pinned linter.
  A symlink `.cache/toolchain-bin/monochromatic-lint` to the installed `monochromatic-lint` was made in the tree
  (git-ignored;
  nothing under `~/.cargo/bin` was changed);
  the program's own line `confirmed version 0.4.0` is the check that it is the pinned one.
  The lock was not touched.
  The directory listing of a sibling tree's `toolchain-bin` (`tr-w387`) was read to learn that setup.
- The first fix of B1180 was written before its red test had run and was put back from `git show HEAD:` to take the
  red run on the old build.
- `coverage-census`,
  `bench` and `probe` had no `dist`,
  so their `lint:types` failed on missing modules until their
  builds were made;
  the builds are in the tree's ignored output.
