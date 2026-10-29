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
This file holds B1020 to B1025.

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
