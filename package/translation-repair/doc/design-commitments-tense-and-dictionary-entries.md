# Design commitments: tense and dictionary entries

The commitments on the past-tense rule and its deterministic aid,
and on a dictionary entry the owner asked for.

Part of [Design commitments](design-commitments.md),
which lists every file of these commitments.

- **The past-tense rule gets a deterministic aid,
  never a deterministic verdict.**
  Owner,
  2026-10-11:
  "as for tense I suppose some deterministic code can mitigate the problem to some extent."
  Measured over 92 archive pages and 274 settled pages,
  a line-level detector was right about two in three when traits of the person count
  (they do:
  a trait of someone who has died is not still true as the page is read),
  and one in four when only narrated acts count,
  so its hits are shown to judges as labelled candidates on the texts they choose among,
  never used to refuse a wording or to call a text flawed;
  pages that record no death are skipped.
  Decided for quality,
  and confirmed by the owner the same day:
  "Candidates to judges,
  as described".
  The owner's answers later that day,
  each the recommended option:
  every model sees the recorded death date in the identity block
  ("Every model sees it");
  the artifact records the candidates on contest and consolidation slices
  ("Contest and consolidation");
  the subject is the front matter `name` and every declared alias
  ("Name and declared aliases").
  Built as ledger B419.
  The rule has one exception,
  decided by the lead on 2026-10-15 after the read of `TianqiChen66624`:
  words quoted as someone wrote or said them keep the speaker's tense,
  because they are the speaker's own voice and the past belongs to the narration around them.
  A quotation moved into the past is a defect,
  and keeping its present is never a tense inconsistency.
  Every sheet that states the tense rule carries the exception (ledger B627,
  M220).
  One narrow case lets the detector decide what ships,
  recorded in the next commitment.
- **The tense detector decides what ships in one narrow case.**
  Owner,
  2026-10-29,
  "Swap only where flagged (Recommended)",
  asked what to do after `XingZ6022` shipped a life that had ended in the present tense at six places
  (ledger B1050).
  When the final round cannot seat two judges
  (fewer slate judges are reachable than a winner needs,
  or the slate heard fewer than two),
  the two-ballot translate text ships instead of an eligible standing the contest did not endorse,
  only where the detector flags present tense in the standing and does not flag the translate text.
  The owner accepts the detector deciding here,
  a narrow exception to the ruling of 2026-10-11 above.
  Everywhere else the aid stays an aid:
  a slate that seats two or more judges decides as it always did,
  a standing the contest endorsed stays,
  a translate text the detector flags too does not replace the standing,
  and a page that records no death is not read.
  Quoted words are blanked by the detector,
  so present tense inside a quotation never fires the swap.
  Each swap is logged with the slice,
  the candidate count of each text and the rule,
  recorded on the settlement as the reason `present-tense-standing`,
  and named in the page-check defect `tense-swap`.
  The detector's limit stands:
  it flags a present auxiliary only directly after a subject word and reads top-level paragraphs only,
  so it misses the sentence that changes tense across two lines
  and the lines inside `<details>` blocks.
  Built as ledger B1050 (`consolidate-tense-swap.ts`).
- **A dictionary entry for 扑面而来.**
  Owner,
  2026-10-11:
  "Add a dictionary entry".
  The entry says the target is whoever faces it and refuses no form,
  since three of the idiom's four occurrences in the corpus wash over the person herself (ledger M148);
  told of the corrected sense,
  the owner kept it so ("Keep the entry as written").
