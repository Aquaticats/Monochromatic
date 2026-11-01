# Audit ledger: the stand-in order and the pages that ship

The fourth re-check of 2026-10-31 (UTC) of the re-ask,
in [the third re-check and the cache versions it moved](audit-ledger-the-third-re-check-of-the-re-ask-and-the-cache-versions-it-moved.md),
and the re-check of the round that shipped no page for XingZ6023,
in [XingZ6023 shipped no page](audit-ledger-xingz6023-shipped-no-page.md),
with the fixes of round w398:
the page title lexicon that ordered a stand-in first (B1200),
the loops of the re-ask that a transport failure ended (B1201),
the repair bench that lost a seat with no voice (B1202),
the evidence for the artifact generation (B1203),
the picture labels in the keys (B1204),
two nits (B1205),
the statements corrected in place (B1206),
the two stops that now ship (B1207),
the interim line worded by its reason (B1208),
the providers left off in the meter report and the sample (B1209),
the footnotes of a shipped original (B1210),
the corrections to the XingZ6023 record (B1211),
the consistency index the merge left unrepointed (B1212),
the cache versions (B1213),
and the slips and what stays open (B1214).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

Standing rules that decide every item:
candidates,
voices and ballots carry the answering model's id,
and an order by roster position reads the seat,
since a stand-in is never on the bench;
a thrown stand-in moves on to the next,
and the seat is lost only when every stand-in failed;
a model never answers again on a question it ran away on;
one judge alone never decides;
a run always ships.

## B1200: the page title lexicon ordered its voices by the model that answered

Red in `be61178`,
fixed in `be53ed63ae`.

`settlePageTitles` sorted `gather.voices` by `modelIds.indexOf(voice.modelId)`.
Since B1142 `HeardVoice.modelId` is the model that answered,
and a stand-in is never on the bench (`alternateModelsFor` leaves the bench out),
so its position is -1 and it sorted before every seat.
`mostGiven` keeps the first wording it meets under each comparison key and the earliest key on a tie,
so a stand-in's spelling won a tie against the seats it stood among.
The sort now reads the seat through `seatOrderOf` (`stage-gather.ts`),
one comparison that the page title lexicon and the consistency gather share;
the duplicate-bodies scan found their two equal bodies when the fix first read `seatOfVoice` in each.

The case runs four seats whose last one runs away,
with a stand-in that ties the third seat at two voices apiece against the first two.
On the old build the tie went to the stand-in's wording (`Cat Song`);
it goes to the first seat's (`Song of the Cat`).

Every other roster-position sort keyed on a voice was read:
`translate-candidates.ts` (`seatOfVoice`),
`editor-candidates.ts` (`seatOfVoice`),
`refine-stage-replies.ts` (`seat ?? modelId`) and
`pass-consistency-gather.ts` (`seatOfVoice`) read the seat;
`run-seats.ts` (two `indexOf` calls) removes repeats from a bench and reads no voice;
`reask-alternate-models.ts` orders the roster ids themselves;
the sorts by model in `critic-attribution.ts` and `attribution-report.ts` are alphabetical.
`consolidate-settle.ts` tests `voice.modelId` for membership among the ids the floor passed,
which the floor builds from the same voices,
and is no order.
The page title lexicon was the only one.

Cache version:
the page title version stays 9 (B1213 gives the evidence).

## B1201: a transport failure ended the order in two of the three loops of the re-ask

Red in `be61178`,
fixed in `be53ed63ae`.

B1162 made the provider's loop move on to the next stand-in for every error but the caller's abort and
`EveryProviderDryError`.
Two loops asked the same order and did not.

- The recovery round of the stage (`stage-recovery-round.ts`) moved on only when the voice was `unreachable`.
  A transport failure arrives as a lost voice that never answered (`answered: false`,
  `unreachable: false`),
  which ended the seat.
  It now moves on for every voice that never answered,
  a straggler the round abandoned included,
  and ends the order,
  as the provider's loop does,
  for an answer that arrived and could not be read.
  The caller's abort and the refusal of every provider dry never arrive as a lost voice:
  `attemptStageCall` throws both.
- The picture re-ask of a cut reader (`image-reading-past-refusal.ts`) rethrew anything but
  `NoProviderForModelError`.
  It now rethrows the caller's abort and `EveryProviderDryError` and moves on for the rest,
  with a warning that names the model and the failure.
  The loop of the other readers moved to `image-reading-other-readers.ts`,
  because the file passed its 300-line bound with the new branch.

A claim taken for a model the router refused before any exchange is now given back
(`ReaskClaims.release`,
`releaseFor`),
in all three loops.
That was the nit about a tally-withheld model:
`alternateModelsFor` is a pure function of the roster and cannot know the run's tally,
so the withheld model is listed to be asked,
the wrapper refuses it,
and the claim stayed taken for a model that was never asked,
which made the log of a later seat say it "was asked for another seat".
A model whose exchange failed stays claimed.

Red:
the stage case fails with `expected [ 'gemma-4-26b-a4b-it' ] to deeply equal [ 'gemma-4-26b-a4b-it', ...(1) ]`,
the picture case rethrows the transport error,
and the two claim cases fail on the model that stayed claimed.
B1162's sentence is corrected in place.

## B1202: the repair bench lost a seat that ran away and left no voice

Red in `be61178`,
fixed in `be53ed63ae`.

B1161 put the model and the seat of every voice on the bench of the repair's re-ask.
A seat that ran away,
whose every re-ask failed,
has no voice,
so it was on no bench,
and a repair ask that ran away could be re-asked of it.
`repairInvalidCandidates` takes `benchModelIds` (required),
the whole bench of the gather,
and the bench of the re-ask is that list and then the model and the seat of every voice.
`produceTranslateSlate` passes `translatorModelIds` and `produceConsolidations` its `roster`.
The comment at the bench is corrected,
and the 14 test calls that did not name a bench name none (`[]`),
which is what they did before.

The case:
seat A runs away and has no voice,
B's candidate fails the rule,
and the ask of B's repair carries a bench with A;
`alternateModelsFor` for B with that bench leaves A out,
and with the bench of the voices alone lists it,
the control that A is otherwise a model B's role may take.

## B1203: the evidence for generation 22 (B1160)

B1160 keeps the artifact generation at 22.
The call is right,
and its evidence is:

- The XingZ6022 artifact is `artifactSchemaVersion = 21`,
  with tip `c6185d4086` (committed 2026-10-29T07:29Z) and written 2026-10-29T11:44Z,
  before `fdf3201f3f` (2026-10-29T14:30Z) set 22.
- No generation 22 artifact exists:
  the `artifacts/` directory of XingZ6023 is empty.
- XingZ6023's tip `bf5d1f00fa` (2026-10-31T07:14Z) holds no re-ask of another model.
- The rule is in `doc/mistake-prevention-stored-files-and-cached-decisions.md`:
  a change rides inside a number only when no record was written at it.
  No record was written at 22.

The readers of the field that B1160 listed leave one out:
the set-size check of the run-off in `artifact-two-lane-read-consistency-span-winner.ts`
counts the seats of the ballots against their number,
and holds for the same reason as `assertSeatsHeardOnce` (a model gives one voice to a question).
B1160 counted the callers of `assertSeatsHeardOnce` as three;
there are four call sites in three files.

## B1204: how the picture labels reach the sheets and the keys

B1164,
B1168 and the version TSDocs said the slice keys name a picture reading
"by its inputs and not by its text or its label".
That is false:

- `slice-pictures.ts` writes each transcription as `${one.modelId}:\n${text}`.
- `translateSliceKey` hashes the picture context as the words the translator was shown
  (`translate-slice-key.ts`,
  with the deliberate comment that the words and not the reading's key are hashed),
  and the consolidate key hashes the same (`consolidate-key.ts`).
  The label is inside both keys.

What that means for item 8 of round w396 (picture readings recorded under the answering reader):
it changes the label for a picture whose cut reply was asked of another model,
so it changes the key for such a picture and for no other.
A record stored under the old label is not found under the new key:
it is missed and the picture is asked again,
which is correct.
It does not move the translate (37) or the consolidate (44) version again.
`ed3f7229cb` is an ancestor of `945af136ba`,
which set 37 and 44 after it,
and no record was written at 37 (B1213),
so the number already stands for the new label.

Corrected in place:
B1164 (the keys),
the `TRANSLATE_SLICE_CACHE_VERSION` TSDoc,
B1168's reason for the pairing stay
(the pairing sheet does carry names where a section has pictures:
`pairing-pictures.ts` renders the readings through `slicePictures`;
the stay is still right because `pairingQuestionKey` digests the picture context),
the repair 51 TSDoc and B1168's repair line
(the repair and the lane contest read no picture reading:
`rg` for the picture context finds none in either package's sources),
and the newest record time,
which is 2026-10-31 10:04:22 UTC and not 08:24 UTC
(B1168 and the consolidate TSDoc).

## B1205: two nits

- `shippingParagraphOf` passed the seat as `modelId` to `foldTranslation`.
  Its findings are not kept,
  so nothing wrong was recorded,
  but the parameter named the wrong thing.
  It is now `writer`,
  and the caller passes the model that answered for the seat.
- The tally-withheld model in the list of alternates is the claim of B1201.

## B1206: statements of the records of rounds w396 and w397 corrected in place

- B1145 said that no cache version moves and that findings are "no stored key or record".
  B1168 moved translate slice (37) and consolidate (44) for that rewording.
  The entry now says so.
- B1165 and the comment of `seat-role-pool.unit.test.ts` said a derivation of the editors or the refiners
  "would fail here".
  That holds for a derivation that changes the list in one of the 8 readings.
  A derivation that only filters the static list by who is seated passes the equalities.
  Both say exactly that now.
- B1163 said "about 32" lines gained a `reask` key.
  The 12 test files of `1b84d1b079` have 38 insertions,
  37 of them added lines that name `reask`.
- B1160 counted three callers of `assertSeatsHeardOnce` (B1203).
- B1168 said of the page title stay that no sheet,
  floor or weight of the stage changes.
  B1200 shows the tie order of the stage did change (a stand-in voice sorted first),
  and the entry now says so in place (round w399,
  B1230).
- B1164 cited lines 926 to 930 of `consolidate-key.ts` for the picture context in the key;
  they are lines 941 to 945 at `abb1e953bc`.
- Two lines of the third re-check file passed 120 columns (the B1165 bullet and a line of B1166) and are wrapped.
- The main index split the link text of the third re-check file across two lines.
- `choiceBallotOf` (`4b973d9a5e`) was in no entry:
  it is the one body the three equal mappings from a stored choice to a ballot became in `corpus-pass`,
  with a case of its own in `pass-consistency-gather.unit.test.ts`.

## B1207: the two page-level stops that ended an entry with no page now ship

Red in `c12d80e7da`,
fixed in `be53ed63ae`.

The owner's rules are "no matter what,
we must ship" and "No valid wording:
keep the archive at that slice and ship" (2026-09-27).
`entryErrorOutcome` filed both of these as `stopped`,
and `shipInterimOnStop` ships only for a run-ending refusal and `provider-unavailable`.

- `assertPageParses` (`publish-fixed.ts`) threw after the defects were logged and before the page was written.
  `pageThatParses` (`page-grammar-fallback.ts`) now reads the composed page under the strict grammar.
  If it is refused,
  slices go back to the archive's text one by one in order until the page parses,
  and each one the page does not need is put back.
  The defect `page-grammar-archive-kept` names the slices,
  and the refusal goes to the log.
  If even the archive's page is refused,
  the page ships as composed with the defect `page-unparseable`,
  and the site build will not take it,
  which the line says.
  The owner's rule asks for a page,
  and a missing page costs more.
  The page-agreement check is one-sided and reports a page that carries the archive where the artifact promised a
  wording,
  so a kept slice can also appear there.
- `NaturalnessCompletenessError` came from `assertFinalNaturalnessComplete` in `persistSettledEntry`.
  `finalNaturalnessDefects` collects it as the defect `final-naturalness`,
  which names the first slice that lacks approval,
  and the page and the artifact are written.
  A fault the guard raises other than its refusal (a contest or consolidation recorded as never run,
  a review the parser cannot read) still stops the entry.

Red:
the curled-literal case of `publish-fixed.unit.test.ts` (it expected the throw and now expects the archive page with
the defect) and the two readiness cases of `pass-readiness-boundaries.unit.test.ts` fail on the old build.

The `stopped` list of `entry-error-outcome.ts` was read class by class.
Which can still end an entry with nothing written,
and why:

- `ArchiveOriginalCompletenessError`,
  `ContributorCompletenessError`,
  `DroppedDestinationError`,
  `FrontMatterCompletenessError`,
  `PublishedPageDisagreesError` and
  `CollapsedHeadingError` cannot.
  Each is thrown by one `assert` whose only production caller is a step of `publishFixedPage`,
  which `publishDefects` collects.
- `UnparseablePageError` cannot from the publisher now.
  `assertPageParses` has no production caller;
  the class stays in the list because the function is exported.
- `NaturalnessCompletenessError` cannot from `persistSettledEntry` now,
  for the same reason.
- `UnansweredContestSliceError` can.
  `wouldShipTextPerSlice` throws it for a slice the contest was bound to decide and no row names,
  before any page exists (`finalSelectionFindings`,
  `composeSourcePage`).
  There is no reading of what ships for that slice.
  Shipping the archive's page is the wider reading of B1183.
- `SliceSpliceError` can.
  `spliceSlices` throws it for replacements it cannot place
  (one slice named twice,
  and the offsets it refuses),
  which is a fault of the data or the code,
  in the splice of `composeSourcePage`.
- `NaturalnessRepairInterruptedError` can.
  The consolidation throws it (`consolidation-polish-apply.ts`),
  mid-pipeline,
  and only a run-ending refusal and `provider-unavailable` ship an interim page.
- `PromptPayloadStoreError` can.
  It is a failure of the disk store of replies,
  at any call.
- `VisualEvidenceInterruptedError` can.
  It stops the entry before the lanes,
  when no slice has settled and nothing exists to ship.
- `TranslationRepairInterruptedError` with `provider-unavailable`,
  and the run-ending refusal,
  ship an interim page from the slicing on,
  and write nothing before it (B1183).

The five that can are open,
for the owner (round w399 closes them,
B1220 to B1223):
the wider reading is a page record for every stop,
the archive itself shipped as the interim page.

## B1208: the interim line is worded by the reason of its record

Red in `c12d80e7da`,
fixed in `be53ed63ae`.

`verify-published-interim.ts` printed "A dry stop shipped it with ..." for every interim record,
and its header said the same,
though a record may carry the reason `provider-unavailable`.
The line now reads "A dry stop shipped it" for `dry-stop`,
and "A stage that heard no voice stopped the entry and shipped it" for the other reason.
The stale words "every provider goes dry" of `runs-layout.ts`,
`pass-entry.ts` (two places) and `pass-entry-settle.ts` are worded by the reason too.

## B1209: the providers left off, in the meter report and in the sample

Red in `c12d80e7da`,
fixed in `be53ed63ae`.

- `run:meter-report`,
  the second tool of the runbook's pre-launch step,
  never read the `PROVIDER OFF` line.
  For Bedrock it printed `wet=0 dry=N`,
  `spendable on 0.0%` and a longest outage for a provider nothing asks.
  `meter-report-off.ts` reads the line from the same logs as the readings,
  and the report says `LEFT OFF, not asked (PROVIDER OFF line: ...)` and gives no figures for it.
  A log from before a provider was left off,
  merged with a later one,
  is reported as left off for the whole record,
  which is a limit.
- `budget-sample-run.ts` built the conditional client for Bedrock alone.
  It now reads the off list for NeuralWatt,
  Synthetic,
  Hyper and OpenRouter too,
  and `readProviderKeys` reads the three optional keys (DeepSeek,
  Xiaomi,
  Anthropic) as unset for a provider left off.

## B1210: footnotes that sit in a shipped original

Red in `c12d80e7da`,
fixed in `be53ed63ae`.

`assertPageFootnotesIntact` reads the composed page since B1182,
originals included.
Measured on the page of the XingZ6022 artifact
(`fixed/people/XingZ60/page.en.md`,
against `preparation.archiveText`):
**1** report,
`orphan-definition gfm 3`.
The artifact reads 118 slices,
115 of them wordings and 3 shipping their original (slices 107,
114 and 115).
The definition of note 3 lies in the original of slice 115,
and the archive's slice 44 has no reference to it.
So the count is not zero,
and the report read as a footnote the lanes introduced.

The finding now names the slice:
`page-footnote-orphan-definition gfm 3 (inside the original shipped at slice 115)`.
A footnote outside every shipped original is named as before.
The reader is `footnoteMentions` over the original as the page carries it,
in any role and in either convention.

## B1211: the XingZ6023 record corrected in place

- The message of `a5eee6d51f` says B1187 holds "7 slips";
  the entry lists 8.
- The introduction said "three smaller findings";
  B1182 to B1185 are four.
- B1186 said the translate slice,
  repair slice,
  lane contest and refine versions stay at 36,
  50,
  18 and 19.
  In the merged tree they stand at 37,
  51,
  19 and 20,
  which round w396 set (B1168).
  It now says what holds.
  Consolidate is 44 in both rounds (B1213).
- B1185 named three other sites that honour the off list.
  `rg PROVIDERS_LEFT_OFF` finds four importers besides the sampler's two files:
  `run-providers.ts`,
  `required-providers.ts`,
  `index-run-setup.ts` and `corpus-pass-run.ts` (line 331),
  which B1185 missed
  (the re-check counted five sites in all,
  where this search finds four importers).
- `ad31320398` edited the evidence string of a red test
  (`'the cat sleeps...'` to `'The cat sleeps...'` in `publish-page-guards-report.unit.test.ts`)
  and its message does not say so.
  Not amended.

## B1212: the consistency index left its moved files pointing at the main index

The merge `aec54dc84f` moved the 20 consistency-pass items of `audit-ledger.md` into the sub-index
`audit-ledger-the-consistency-pass.md`,
and left each moved file saying `Part of [Audit ledger](audit-ledger.md)`.
The gate was red in two source-scan files
(`doc-line-limit.unit.test.ts`:
each file part of an index that does not link it;
`markdown-lint-clean.unit.test.ts`:
two semantic line breaks on line 5 of the sub-index).
It was the lead's slip:
the index was split without repointing the moved files and without running the doc scans.

Fixed as round w391 did for the weight floors:
19 of the 20 files now say
`Part of [Audit ledger: the consistency pass](audit-ledger-the-consistency-pass.md),`,
the twentieth (`...page-definitions-recorded-and-carries-gated.md`) had no part-of line and none was added
(round w399 added one,
pointing at the sub-index,
B1238),
every file the sub-index lists is one of the 20 files named `audit-ledger-consistency-pass-*`
and none of them is missing from the list,
and line 5 of the sub-index is broken at its clause boundaries.
Both scan files pass.

## B1213: the cache versions

The rule is step 4 of `mistake-prevention.md` (M25,
M28):
a change that alters what a stage asks or accepts moves its version,
unless no cache file was written after the change,
and a change rides inside a number only when no record was written at it.
The audit
(`slice-cache:cache-account-audit -- --runs-under` the agent folder,
run on the tree of round w398)
lists eight constants and 14,049 slice-cache records under 430 runs directories,
the newest 2026-10-31T10:04:22Z (`XingZ6023`),
and counts 5 of 8 versions set after it:
consolidate 44,
lane contest 19,
refine 20,
repair slice 51 and translate slice 37.
No record exists at any of those five.
The other three are consistency 10 (set 2026-10-31T06:36Z),
page title 9 (2026-10-25T07:01Z) and pairing 3 (2026-09-28T04:30Z).

- Page title 9 stays.
  The fix of B1200 changes the order of one stand-in voice against the seats on a tie.
  The page title records on disk at 9 are those of XingZ6016,
  XingZ6018 and XingZ6019 only,
  with times from 2026-10-21T06:17Z to 2026-10-22T18:58Z,
  before commit `4347b354bb` set the number (2026-10-25T07:01Z).
  XingZ6023 holds only `page-titles-generation.txt`,
  and an empty settled directory is no evidence (`doc/mistake-prevention-stored-files-and-cached-decisions.md`,
  M703),
  so the argument does not rest on the runs after XingZ6019 writing none.
  What holds is that no build that wrote at 9 could produce a stand-in voice:
  `98d5b63eb3` is not an ancestor of `bf5d1f00fa`,
  and before it a voice's `modelId` was its seat,
  so the tie order was identical.
  No record stands at 9 that the new order could contradict,
  so the change rides inside it.
- Translate slice 37,
  consolidate 44,
  lane contest 19,
  repair slice 51 and refine 20 stay.
  B1201 changes which model is asked after a failure of an ask,
  in the three loops that reach every one of the five,
  and B1202 the bench of a repair ask,
  which only translate 37 and consolidate 44 call (`repairInvalidCandidates`,
  called from `translate-stage/src/translate-produce.ts` and `consolidate/src/consolidate-produce.ts`);
  lane contest 19,
  repair slice 51 and refine 20 are reached by B1201 alone;
  the key hashes the version,
  the shape of the run and the texts,
  not the order of the asks,
  and a record made without a seat that failed is as valid as any record with a lost voice.
  No record stands at any of the five numbers,
  so the changes ride inside them.
  This follows B1161 and B1162,
  which moved none.
- Consistency 10 and pairing 3 stay.
  B1201 reaches them only through the shared recovery round and the same failure path;
  the stored voice and the key are as they were,
  and B1204 shows the pairing key already carries the picture label.
- Consolidate 44 was set by both rounds that merged:
  round w396 (`945af136ba`) and round w397 (`e081ca5f5f`) each moved it from 43 and neither knew of the other.
  One move retires every record at 43,
  and no record was written at 44 by either alone,
  so the number means both changes.
- The artifact generation stays 22 (B1203).

## B1214: the slips of the round and what stays open

Slips in how the work was run:

- The whole source-scan suite ran once on `be53ed63ae` and found 7 red files:
  an exported guard no production code calls
  (`assertPageParses` is listed as a seam now),
  four new modules without an own test
  (each has one now),
  eight `@param` or `@returns` lines that opened with an article,
  one reference by position in a test comment,
  two clock times with no zone in this record,
  one body in two places
  (`byRoster` is `seatOrderOf` now),
  and one import nothing reads.
  `3c8cdd02d6` clears them,
  and the suite then ran once more on that commit with one finding left,
  a comma inside parentheses in this record,
  after which its Markdown scan alone ran green.
  The brief asks for the suite at the end,
  and the cost is a second code commit.
- `d7db6a1c8c` says it holds 8 files and 709 insertions with the new ledger file of 569 lines.
  It holds the 7 other files (140 insertions and 32 deletions),
  because the new file was staged and left out of the pathspec of the commit (the slip the rule on new files in a
  pathspec names).
  `abb1e953bc` adds the file;
  the message of `d7db6a1c8c` is not amended.
- Six edits to one test file were sent in one block,
  where the brief says one edit per file per block.
- A read of the output file of a background task named the file under `/tmp/claude-1000/`,
  which the brief forbids;
  it held only the process id of the build that was started.
- A command ended in `; echo`,
  a `;` chain.
- Two evidence reads were capped by `head`.
- A build of corpus-pass was started with `&` inside a background command,
  so its first completion notice was for the shell and not for the build;
  the wait that followed was on the process id it printed.

Gates:

- Each touched package's unit suite ran once after its fix,
  after a rebuild of its dependencies:
  provider,
  stage,
  slice-reading,
  artifact,
  translate-floor,
  translate-stage,
  consolidate,
  page-check,
  corpus-pass and run,
  every one with exit code 0 and no failed case.
  The test files `3c8cdd02d6` changed or added reran once each,
  all green.
- `lint:types` passed for the touched packages and the 16 packages that depend on them,
  the dependents read again on the final tree,
  and oxlint for the touched ones with 0 warnings and 0 errors after the findings it made were cleared
  (a 300-line bound in `slice-reading` and in the `page-check` index,
  the nullish-union rule in two tests,
  the style rules of the new files).
  `bench`,
  `coverage-census` and `probe` had no `dist`,
  so their types were read after their builds were made.
- The whole source-scan suite ran twice (see the slips),
  and its Markdown scan alone and the doc scans reran green on the final text.
- The `cache-account-audit` ran twice on the tree,
  before and after the first code commit,
  and printed the same newest record and 5 of 8.

What stays open:

- The five classes of B1207 that can still end an entry with nothing written (round w399,
  B1220 to B1223).
- A merged record that mixes logs from before and after a provider was left off is reported as left off (B1209).
- The measured count of B1210 is for one artifact and the page on disk,
  which the run's build wrote with the composition of its time.
- The lint rule `no-nullish-union` refused a `let` holding a result or nothing in the two readiness cases,
  so they lost their `try` and `catch`:
  a throw from `persistSettledEntry` now fails the case by rejection and not by a named expectation.
