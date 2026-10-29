# Audit ledger: the two-ballot translate text and the lead's adoptions of 2026-10-29

The corpus pass `XingZ6022` (build `c6185d4086`,
2026-10-29) shipped the archive at sixteen slices
(14,
44,
54,
60,
67 to 70,
72,
73,
75,
78,
79,
81,
82 and 85)
although the translate lane held a text at each that two judges had chosen
and that passed the deterministic publication rule.
On slices 67 to 70 the archive carries an invented Metal Gear Solid 2 quotation.
This file holds B1020 to B1025,
B1040,
B1050 and B1051.
B1050 and B1051 record how the page of that pass came to tell a life that ended in the present tense at six places
and to write an ambiguous `their`;
B1052,
the heading `Part Nine: __` of the same page,
is filed in
[Audit ledger: found while fixing, names, headings and math on the page](audit-ledger-found-while-fixing-page-names-headings-and-math.md).

The owner's answer of 2026-10-29 (question tool),
choice "Ship the two-ballot text (Recommended)":
"When the final round can't seat two judges,
the translate lane's winner ships if two judges chose it and it passes the publication rule."

Rules that still hold:
- One judge alone never decides.
- Two ballots decide (2026-10-25).
- "Keep archive,
  ship" where no wording passes the rule (2026-09-27).
- A disputed archive is never a candidate or a fallback (2026-09-27).
- A run always ships.

B1022 and B1023 are the lead's adoptions,
not owner answers,
and they are open to the owner's veto:
the footnote guard and the repeated slate.

The rulings are quoted in
[shipping and the archive](design-commitments-shipping-and-the-archive.md)
and in the decision record
`doc/decision/translation-repair-ineligible-standing.md` (addendum 2026-10-29).

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1020: the record counted the slate's ballots and never the translate lane's two

Status:
recorded on 2026-10-29 (UTC),
fixed.

The path in `XingZ6022` (slice 67,
log lines 27713 to 27840):
- The repair lane disputed the archive and the checkers confirmed no repair,
  so the slice had no eligible standing.
- The translate lane excluded the archive and a candidate won there with two ballots.
- The consolidation offered that text on the slate beside the proposals.
- One slate judge was reachable.
  `deepseek` named the faithful candidate twice at weight 0.5,
  and each round closed `winner was named by 1 ballot against a floor of 2; keeping the fallback`.
- The challenge declined,
  and `keepTheArchive` shipped the archive with the text "no wording passed the deterministic publication rule",
  which was false there.

The recorded rulings counted only the slate's ballots
("Ship the slate's wording" and "Ship the slate's pick" need two or more slate ballots),
and the check of the read found no owner answer for a text that two translate judges named.
Slices 14 and 44 were refused by the deterministic floor rather than disputed,
and took the same path.

What changed:
- `consolidate-ineligible-standing.ts`:
  `translateWinnerOf` reads the translate text off the lane offer when two or more judges chose it
  (a text on the offer has passed the rule,
  since `laneTextsForSlate` drops one that fails it).
  `shipTranslateWinner` settles the slice on that text with the mark `translateWinnerShipped`
  and the finding `translate-winner-ships`.
  `keepTheArchive` takes a reason.
- `consolidate-settle-unseated.ts` (new,
  split from `consolidate-settle.ts` at its line cap):
  a slate round that heard fewer than two judges (`decided.tally.ballots`) ships the translate winner
  where the lane holds one,
  and keeps the archive otherwise.
  A slate that heard two or more and declined keeps the archive as before.
- `consolidate-slice-buy.ts`:
  fewer reachable slate judges than a winner needs,
  over a refused standing with a two-ballot translate winner,
  buys neither writers nor slate and ships the winner directly
  (marked `benchBelowFloor`,
  terminal `incumbent-only`).
  The stale comment ("An ineligible standing is still asked,
  since its slate may be the only wording that ships") is replaced:
  such a standing is asked only where no translate winner exists.
  The check measured 65 SPEND lines and 10.679 Wh spent for nothing at these slices.
- The ballots come from the lane's own records.
  `translateWinnerBallotsOf` (`corpus-pass/src/corpus-run/pass-translate-winner-ballots.ts`) counts the ballots
  that named the shipped position of `TranslateSliceRecord.stageResult`,
  and credits the slice only where the round chose fresh wording
  and that wording is what the delivery ledger carries.
  An assembly guard that trimmed the shipped text leaves wording the judges never saw,
  and it earns no credit.
  The count flows through `pass-entry.ts`,
  `pass-settlement.ts`,
  `pass-consolidate.ts` and `consolidate-driver.ts` to `buyConsolidationSlice`.
  It is not in the lane texts and not in the cache key.
- The shipped winner is never cached (`consolidationWorthResuming`,
  `consolidate-cache-store.ts`),
  never reported as a standing kept unendorsed (`standingKeptUnendorsed`),
  and polishes nothing (its polish record is `not-run`).

Choices made by the lead's worker,
open to the lead:
- "Cannot seat two judges" means a slate whose reachable judges,
  or whose heard ballots,
  are fewer than two.
  A gate that heard fewer than two keeps its own reason (`final-round-unseated`)
  and does not ship the translate winner.
  That path needs a fresh consolidation chosen by the slate with fewer than two slate ballots,
  which `XingZ6022` never reached.
- The two-ballot credit applies only to the text the judges saw (no trimmed text).

Guards shown to fail first:
`consolidate/src/translate-winner-ships-unseated-slate.unit.test.ts`
(five settle cases and the purchase case failed,
the controls passed),
`corpus-pass/src/consolidate-driver-translate-winner.unit.test.ts` and
`corpus-pass/src/corpus-run/pass-translate-winner-ballots.unit.test.ts`.

The tests ran against a typography build whose lint pin was set to 0.4.0 locally and never committed:
the shared `monochromatic-lint` is 0.4.0 and `typography` still pins 0.3.0 until the typography round lands,
so every case that builds a slate fails with `LintBinaryVersionError` on this tree.

## B1021: the defect note said no wording passed for every kept archive

Status:
recorded on 2026-10-29 (UTC),
fixed.

`archiveKeptDefects` (`page-check/src/corpus-run/publish-defects.ts`) built "no wording passed the deterministic
publication rule" from `shipped.kind === 'archive'` alone.
Three things can keep the archive,
and the sentence was true of one:
- No wording passed the rule (`no-wording-passed`).
- The final round could not seat two judges and no wording two judges chose passed it (`final-round-unseated`).
- The slate seated two or more judges,
  declined every wording that passed,
  and none drew two ballots (`slate-declined`).

The settlement now carries `archiveReason`,
and artifact generation 22 records it on the archive shipped kind (B1025).
The defect groups the slices by reason in one message,
and says "the artifact records no reason" for a record written before generation 22.
The finding on the settlement keeps the prefix `no-valid-wording`,
so that a search over a log still finds every kept archive.

Guards shown to fail first:
`page-check/src/corpus-run/publish-defects.unit.test.ts`
(two cases failed against the old message,
and the case for no wording passed was the control).

## B1022: the footnote guard withdrew a slice whose withdrawal could not clear the orphan

Status:
recorded on 2026-10-29 (UTC),
fixed.
This is the lead's adoption and it is open to the owner's veto.

In `XingZ6022` the guard withdrew slice 115 for `orphan-definition gfm 3`,
which slice 44 had caused by keeping an archive text that dropped `[^3]`.
Slice 115 is an anchor the archive never translated,
and its original holds definition 3.
A page ships an untranslated anchor's original under a note,
so the withdrawal could not clear the orphan and a good translation was lost.

The rule now (`assembly/src/assembly-withdrawal-proof.ts`):
- A suspect is withdrawn only when the page with its replacement gone no longer introduces that same finding,
  a withdrawn anchor being counted as carrying its original.
- A missing reference is blamed on the slice that lost it first,
  and the definition beside it stays.
- A finding that has suspects and no proof withdraws nothing
  and is recorded as `assembly-footnote-reported`,
  which the publish check `page-footnotes` then names.
- A finding nobody can be blamed for keeps the blanket withdrawal.

Not changed:
the baseline of an introduced finding is still the archive text with empty anchors,
not the page with every untranslated anchor carrying its original.
Under that baseline the orphan of `XingZ6022` would not be introduced by slice 115 at all.
Moving the baseline is a wider change than the question asked for,
and it is left to the lead.

Guards shown to fail first:
`assembly/src/assembly-withdrawal-proof.unit.test.ts`
(the `XingZ6022` shape and the lost-reference shape failed,
and the anchor without a definition in its original was the control).

## B1023: the follow-up asked the same two judges the same slate again

Status:
recorded on 2026-10-29 (UTC),
fixed.
This is the lead's adoption and it is open to the owner's veto.

Slices 107 and 114 of `XingZ6022` each ran four select rounds that heard two of eight judges
(log lines 24684 to 25370),
split one ballot each,
and shipped the source text under a note.
The outcome rules stand
("Source text + note",
2026-10-23,
and "Two judges must name it",
2026-10-25).
Only asking the same question four times is dropped.

`translate-stage-repair.ts`:
a follow-up round whose writers propose the same renderings in the same order as the slate the judges declined
is not judged again when the archive has no wording to keep or ship.
The writers are still asked,
a follow-up slate that differs is judged as before,
and a passage whose archive wording cannot ship still asks,
because its follow-up ships past a decline.
The absence is settled with `translate-followup-slate-repeated`.

Calls saved on the shape of `XingZ6022`:
the fixture of two judges splitting across two renderings made 8 judge calls before and makes 4 after.
In the recorded log each of the two slices ran 4 rounds of 2 answers,
so up to 4 of 8 answers per slice,
and 8 of 16 in all,
are not asked where the follow-up slate was the same as the first.
The log does not record the follow-up slate's texts,
so how many of the 4 rounds repeated a slate is not measured here.

Guards shown to fail first:
`translate-stage/src/translate-followup-repeated-slate.unit.test.ts`
(the repeated slate failed with 8 calls,
and the fresh slate and the withheld archive were the controls).

## B1024: why the archive beat a patch that resolved every accepted issue (slice 44)

Status:
recorded on 2026-10-29 (UTC),
traced,
logging fixed,
no change to the ranking.

The chunk closed `unchanged, the archive won, 3/3 served accepted issues resolved` (log line 11711)
after a checker round of 3 of 3 with every ballot `fixed` (11692 to 11701)
and a clean probe (11710).
`settleChunkVerdict` ranks the patch against the unchanged candidate with `selectRepairCandidate`:
integrity,
then high-severity issues resolved,
then regressions,
then total resolved,
then the size of the touched region,
and the unchanged candidate wins a full tie.
The line says `the archive won` exactly when the patch did not win the ranking.
A patch that resolved three accepted issues beats the unchanged candidate on resolution,
so only the first rank,
integrity,
can have gone against it,
because `regressedKnownIssues` ranks third and every checker ballot was `fixed`.
Integrity (`measurePatchedCandidate`) is true when the patched text's grammar downgrades and footnote findings
are no higher than the archive's,
and both are counted over the chunk alone
("Chunk-scoped like every other measurement" in `chunk-measure.ts`).

The probable cause,
inferred and not measured:
the patch restores the reference `[^3]` that the archive chunk dropped,
the definition lives in slice 115,
so the chunk alone reads the restored reference as a new unresolved footnote and its integrity fails.
The log did not record the measurements,
so this cannot be proved from `XingZ6022`.

It is the design as written,
not a recorded owner ruling,
and it was not changed.
What changed:
the verdict carries `patchLostOn` (`integrity`,
or `regression` when integrity held and no high-severity issue was resolved)
and the settlement line prints it,
so the next run says why.
A question stays open for the lead:
whether the chunk-scoped footnote count should treat a reference that the slice's original also carries as
restoration rather than damage.
The owner answered it on 2026-10-29 and B1040 records the measurement of the cause and the change.

Guards shown to fail first:
`repair/src/repair-chunk-verdict.unit.test.ts` and `repair/src/repair-chunk-settlement-line.unit.test.ts`
(three cases failed).

## B1025: artifact generation 22 and the cache versions

Status:
recorded on 2026-10-29 (UTC).

Artifact generation 22 (`ARTIFACT_SCHEMA_VERSION_V22`):
a consolidation slice records `reason` on the archive shipped kind
and the shipped kind `translate-winner` with its text.
The reader requires the key of a generation 22 file and refuses the key and the kind in an older one.
The pass refuses to resume a run directory stamped 21 or earlier (`assertResumableSchemaGeneration`),
and `readSettledArtifact` still reads those files under their own generation's rules.
A translate-winner slice records no tense candidates,
because the contest record holds the translate candidate's.

Cache versions:
none moved.
- The consolidation cache (`CONSOLIDATE_CACHE_VERSION`) stores settlements that asked a slate and settled.
  The two new marks are never stored
  (`consolidationWorthResuming`,
  `consolidate-cache-store.ts`),
  as `archiveKept` and `benchBelowFloor` are not,
  and no stored settlement changes its meaning.
  The translate winner's ballots are not in the key,
  because the settlements that depend on them are the ones never stored.
- The translate slice cache (`TRANSLATE_SLICE_CACHE_VERSION`) stores settled records.
  A repeated follow-up slate ends unfilled,
  which stores nothing,
  and a record that a follow-up filled still says what two judges chose.
- The footnote guard runs at assembly over records and stores nothing.
- The repair chunk change only words a log line.
- Every cache is opened under the pipeline digest of the build,
  which changes with any source change.

Ids used:
B1020 to B1025.
B1026 to B1029 are unused.

## B1040: a footnote reference restored from the source counted as a new break in the chunk-scoped integrity count

Status:
recorded on 2026-10-29 (UTC),
measured offline on the real case,
fixed.
Settles the open question of B1024.

The defect:
`measurePatchedCandidate` (`repair/src/chunk-measure.ts`) compares `footnoteBreakCount` of the patched chunk with
the translation's chunk,
and the count is chunk-scoped.
A patch that puts back a footnote reference the translation dropped,
when the definition lives in another chunk,
adds an unresolved reference inside this chunk,
so the count rose,
`integrityOk` went false,
and the patch lost the first rank of `selectRepairCandidate` though it fixed real damage.
In `XingZ6022` slice 44 the archive dropped `[^3]`,
the patch restored it,
and the definition of 3 is in slice 115.

The owner's ruling,
2026-10-29,
"Count as restoration (Recommended)":
in the repair lane's chunk-scoped footnote integrity count,
a reference that the slice's own source (the original-language text of that slice) also carries
counts as a restoration and not as a new break.

The evidence,
rebuilt offline from the run's artifact (`artifacts/XingZ60.json`).
The run's slice cache is empty,
the artifact keeps no `patchLostOn`,
and the close line 11711 does not name why the patch lost.
- The archive chunk (`comparison[44].incumbentText`) has a footnote count of 0.
- The patched chunk is the three repair regions of `lanes.repair.result.chunks[44]` applied to the archive,
  and it is byte-equal to the chunk patch the editor stage selected
  (round 3,
  slate slot 2,
  "chunk patch from deepseek-v4.1-flash won weight 3.5 of 4 ballots",
  log line 11680).
  Its count is 1:
  one `unresolved-reference` finding,
  convention `gfm`,
  identifier `3`.
- The slice's source (`lanes.repair.delivery[44].sourceText`) carries the same reference,
  convention `gfm`,
  identifier `3`.
- The grammar downgrade counts are 0 for both chunks,
  so the footnote leg alone made `integrityOk` false.
  `measurePatchedCandidate` on these two documents with no source references returns `integrityOk: false`.
- The checkers heard 3 of 3 with every ballot `fixed` and the probe was clean (log lines 11692 to 11710),
  so integrity was the only rank the patch could lose.
- The round that logged this case recorded the cause as probably the `[^3]` "orphan count".
  The finding is an unresolved reference,
  not an orphan definition.

After the fix the same call,
given the source's references,
leaves the one finding out (effective count 0 against 0) and returns `integrityOk: true`.

The fix:
`restoredFootnoteReferences` (`shared-contract/src/footnote-break-count.ts`) names the `unresolved-reference` findings
of the patched chunk whose key the source's references carry and the translation's references lack.
The key is the syntax family plus the label folded as Markdown folds it (`normalizeFootnoteIdentifier`),
so `[^Note]` and `[^note]` are one footnote.
`measurePatchedCandidate` takes the source's references and subtracts those findings from the patched count;
`settleShippedPatch` requires the slice's parsed source and logs each restoration through its tagged logger
with the slice and the identifiers.
Orphan definitions,
duplicated definitions,
a reference the source lacks and a key the translation already carried stay counted as before.

Readings the count takes,
each tested:
- The allowance is per key and not per mention.
  A source citing `[^3]` twice excuses a patch citing it three times:
  the signal counts classes of structural damage,
  and how often a cited footnote is mentioned is content fidelity that the checkers judge.
- A key the translation already carried is not excused,
  resolved or not,
  because it is not something the patch put back.
- A full-width marker in the source (`〔3〕`) does not restore a `[^3]` in the patch:
  the footnote graph keys the two syntax families apart,
  and this change leaves that as it was.

What the page-level guard still covers:
a reference with no definition anywhere on the page is still caught by `page-check`'s footnote resolution
and by the `assembly` integrity guard (B1022),
which this change does not touch.
A restored reference whose definition exists in another slice is the case this count now lets through,
and the assembly guard still reads the whole page.

Cache versions:
- `SLICE_CACHE_VERSION` moves from 49 to 50.
  The change is a gate change of the kind its note names under version 6:
  a patch that lost on integrity now wins for the same texts,
  and the key hashes the version,
  the shape of the run and the texts,
  not the gate.
- `REFINE_CACHE_VERSION` (19) and `CONSOLIDATE_CACHE_VERSION` (43) stay:
  their keys hash the repaired text and the repair text they are offered,
  so a slice whose repair verdict changes reaches them under a different key,
  and neither reads the footnote count.
- The translate,
  lane-contest,
  consistency,
  pairing and page-title versions stay:
  none of them runs this measurement.

Commits:
`097e60cdb5` (eight cases for `measurePatchedCandidate`,
three of them red on the old code and no fix),
`54eddafbe5` (the fix,
two cases for the settlement line,
and the version move),
`145dc0411a` (the seven cases for `restoredFootnoteReferences`,
which the fix commit put in `footnote-break-count.unit.test.ts` against ledger B99,
moved to `footnote-restored-references.unit.test.ts`).

Guards shown to fail first:
`repair/src/chunk-measure.unit.test.ts`
(three cases failed on the old code:
the restored reference,
the case-differing spelling and the twice-against-thrice case).

Ids used:
B1040.
B1041 to B1044 are unused.

## B1050: the XingZ6022 page told a life that ended in the present at six places, by two paths

Status:
recorded on 2026-10-29 (UTC),
read offline from the artifact and the log,
no code changed.
Group A is reached by B1020.
Group B is open as an owner question.

The defect:
the page shipped by `XingZ6022` tells the life of a person who has died in the present tense
at page lines 95,
231 to 232,
379,
390,
392 and 396,
though the house rule (`house-rule/src/house-policy.ts:90`) tells it in the past.
Page line 231 as printed is past tense ("complained");
the present perfect is "and has expressed" on line 232,
so that defect is one sentence that changes tense across two lines.
None of the six is quoted words.
Quoted fragments inside them ("finally judged" and "Anyone can write whatever they want" in 95,
"take it slow" in 390)
keep their tense,
as the rule asks.

Which path shipped each line,
and why no sheet stopped it.
The translate select heard two judges at all six slices
and chose the past-tense text each time
(log lines 21222,
22172,
23558,
23581,
23743,
23828).
The three later benches heard one judge each,
because the Synthetic weekly credit ran dry at 10:36:27 UTC on 2026-10-29:
the lane contest opened no round at any of these slices
(`1 of 7 seats reachable, below the floor of 2`,
log lines 25427,
25478,
25535,
25542),
every slate round heard 1 of 7,
and the page consistency round heard 1 of 7 (log line 29635).
Each of those rounds needs a second judge to decide,
so the sheets that state the tense rule were never read by two judges after the translate select.

Group A,
the archive kept (slices 14,
79,
82;
page lines 95,
390,
396).
Slice 14's archive failed the deterministic floor for an untranslated `TA` (log line 25983);
the repair lane's adjudicators disputed the archives of 79 and 82.
The translate lane held a past-tense text that two judges chose
(slice 14 at log line 21222,
weight 2;
slice 79 at 23581 and slice 82 at 23743,
weight 1.5)
and the consolidation offered it on the slate (lines 26023,
28283,
28553).
One slate judge was reachable,
so the round closed on one ballot and `keepTheArchive` shipped the archive
(lines 26355,
28427,
28674).
The archive tells these lives in the present ("XingZ60 is somewhat similar",
"XingZ60 is a very mature person",
"XingZ60 is more complicated"),
and the translate text has "was" in each.
B1020 changes this outcome:
`a30ac44878` (the winner ships where the final round cannot seat two judges),
its red cases in `0107ed57ca`,
and `fdf3201f3f` with its cases in `dd6bb21c6f` (the pass carries the winner's ballots and the archive's reason).
The conditions of `translateWinnerBallotsOf` hold at all three:
none of 14,
79 and 82 is among the 40 slices the assembly guard trimmed,
so the text the judges saw is the text the ledger carries.
A replay in which the slate again hears fewer than two judges ships the translate text there;
a replay that seats two judges decides as the slate decides.
This is stated from the code and the artifact,
not from a replay.

Group B,
a standing the rule passed (slices 43,
77,
80;
page lines 231 to 232,
379,
392).
The standing was the repair stand-in at slice 43 and the archive unchanged at 77 and 80.
The translate lane again held a text two judges chose (log lines 22172,
23558 and 23828),
with "had expressed" at 43,
"was a rational person" at 77 and "I learned" at 80.
One slate judge named it alone (`winner named by one judge alone; keeping the incumbent`,
lines 27008,
28466, 28490)
and the standing shipped.
B1020 does not reach these:
`shipWinnerBeforeSlate` returns `asks` for an eligible standing,
and `settleRefusedStanding` runs only over a refused one,
because the owner's answer of 2026-10-29 was asked about a refused archive.
The standing was unendorsed,
no judge had called it flawed,
and so none of the rulings of 2026-10-25 reaches it either.
Whether a two-ballot translate text may replace an unendorsed standing when neither the contest nor the slate can seat
two judges is the owner question at the end of this entry.

Measured on the artifact:
the standing was eligible and the slate declined at 59 slices (49 archives unchanged,
10 repair stand-ins),
and at 51 of them the translate lane's text differs from the shipped text.
The ballots behind those 51 texts were not counted.
The tense detector flagged the standing and not the translate text at six of the 51
(slices 11,
16,
42,
43,
77 and 80).
That count measures the aid's hits,
not defects:
the owner measured the detector right about two in three (2026-10-11).

A detector limit,
stated:
the detector flags a present auxiliary only directly after a subject word,
so "and has expressed" after a past verb (slice 43,
page line 232) is not flagged.
Run offline over the page lines as slices,
it flags 95,
379,
390,
392 and 396,
and flags nothing in 231 to 232.
Run over the whole page it flags only line 379,
because lines 95,
390,
392 and 396 sit inside `<details>` blocks (page lines 80 to 267 and 387 to 406),
and the reading takes top-level paragraphs only.
No change was made:
an extension needs a remeasure against the corpus by the owner's own method,
and the tense candidates reach every judge sheet,
so it moves several cache versions.

Still stands:
the aid is shown to judges and never decides (design commitment,
2026-10-11),
so a short bench leaves a present-tense line where no judge reads it.
A deterministic correction of tense on a kept archive is therefore not a question:
the owner declined it on 2026-10-11.

Owner question (open),
with the options as the lead ranks them:
- Option 1,
  ship the two-ballot translate text over an unendorsed standing only where the standing carries tense
  candidates the translate text lacks and neither the contest nor the slate seated two judges (six slices here).
  It fixes the known lines at the smallest scale.
  It lets the aid decide,
  which the ruling of 2026-10-11 forbids,
  so the owner must lift that for this case.
- Option 2,
  keep the standing (today).
  It breaks no ruling and keeps every repair-lane edit.
  It leaves the present-tense lines whenever the bench is one judge deep.
- Option 3,
  ship the two-ballot translate text over every unendorsed standing on those terms (51 slices here).
  It follows the words of the owner's 2026-10-29 answer most literally.
  It re-decides 51 slices on the translate lane's ballots alone,
  and the repair lane's verified edits lose.
- Ranking:
  Option 1 > Option 2 > Option 3.
  Option 1 beats Option 2 because it removes the defect class this page showed at the scale it showed it.
  Option 2 beats Option 3 because Option 3 overturns repair work at 45 slices where no tense fault was flagged.

Cache versions:
none moved,
because no stage output changed.
The consolidation,
translate-stage,
lane-contest and page-check versions stay.

## B1051: `their` shipped for the person beside a plural subject, as the neutral pronoun rule asks

Status:
recorded on 2026-10-29 (UTC),
read offline,
no code changed,
open as an owner question.

The defect:
page line 88 reads "XingZ60's parents dressed up their appearance to match a traditional male gender expression at
their funeral",
and line 90 "threw their ashes",
where `their` means the person who died but the parents are the subject of the same sentence.
The archive had `her`.

Which path shipped it:
slice 13,
the repair stand-in.
The archive was disputed (two accepted claims,
log line 25959),
the repair lane and the translate lane wrote the same `their` in all three places,
the contest opened no round (1 of 7 seats),
and the slate's single judge named the translate text alone,
which differs from the stand-in only in the spelling
`behaviour` (lines 21067,
26091).
So neither lane's text could have mended it.

The cause is not a stage fault.
The original writes `TA` at each of these places ("在 TA 的葬礼上把 TA 的外貌",
"把 TA 的骨灰",
"TA 父母"),
and the house rule renders `TA` as singular they and calls a pronoun left in the other form a fault
(`house-rule/src/house-policy.ts:91`;
the floor that withheld the archive at slice 14 is
`translate-floor/src/translate-neutral-pronoun.ts`).
The archive's `her` was the hand translation breaking that rule.
Both lanes obeyed it,
and nothing in the rule or in any sheet tells a writer to name the person where singular they sits beside a plural
noun it could be taken for (`rg 'ambigu' house-rule/src translate-floor/src` finds no such text).
No deterministic check can decide the ambiguity,
because it turns on who the sentence is about.

Owner question (open):
- Option A,
  keep the rule as it stands.
  It breaks no ruling,
  and costs nothing.
  It leaves sentences like line 88,
  whose number was not counted (19 page lines hold `their`;
  how many are ambiguous
  was not measured).
- Option B,
  add one sentence to the pronoun rule in `house-policy.ts`:
  where singular they sits in a sentence with a plural noun it could be taken for,
  the sentence repeats the person's name or the word "the deceased" for that pronoun.
  It mends line 88 for any run that reads the rule.
  It changes sheet text that every stage showing the house rule carries,
  so the cache versions of those stages move by the cached-decisions rule,
  and it cannot be checked without model calls.
- Ranking:
  Option A > Option B.
  Option A leads because the prevalence is unmeasured and Option B moves many caches for one sentence of one page.
  Measure how many corpus sentences put singular they beside a plural noun first,
  and the order may change.

Cache versions:
none moved,
because no stage output changed.

Ids used:
B1050 and B1051.
B1052 is in
[Audit ledger: found while fixing, names, headings and math on the page](audit-ledger-found-while-fixing-page-names-headings-and-math.md).
B1053 to B1059 are unused.
