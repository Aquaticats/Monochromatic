import type { IncumbentKind, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Selection decline consequence
// What a judge is told a decline does. The shared sheet says the caller keeps
// trusted text; a caller for whom that is false must say what is true, since a
// judge told something false about the cost of declining is asked for caution
// by a promise nobody can honour.

/**
 What a decline costs where the caller HAS something to fall back on.

 The ordinary case, and the reason declining is safe to encourage: the editor
 and refiner lanes keep the text they were given, and the translate lane keeps
 the archive's own wording. A caller with nothing to keep has to say so
 instead, since a judge told this while it is false is being asked for caution
 by a promise nobody can honour.

 @example
 ```ts
 const consequence = KEEPS_TRUSTED_TEXT;
 ```
 */
export const KEEPS_TRUSTED_TEXT: string = 'the caller keeps text it already trusts when you decline';

/**
 What a decline does on the chunk selection, where declining every candidate
 and failing to agree lead to different texts (ledger L14(c)).

 A decline names no candidate and counts as no vote, as `neither` does at the
 consolidation gate by measurement (`doc/planning/the-third-rendering.md`).
 The existing English is kept only when every judge declines; when the judges
 who named a candidate reach no decision, the editor patch that landed the
 most edits goes on, since the panel ruled its issues real
 (`pickFallbackCandidate`). The shared promise, that the caller keeps trusted
 text when a judge declines, held for neither case: 14 of 2,476 chunk rounds
 over every artifact had declines outnumbering the ballots naming a
 candidate, and 10 of them sent that fallback on.

 @example
 ```ts
 const consequence = CHUNK_DECLINE_CONSEQUENCE;
 ```
 */
export const CHUNK_DECLINE_CONSEQUENCE: string = [
  'a decline names no candidate and counts as no vote: the caller keeps the EXISTING ENGLISH BEFORE REPAIR',
  'only when every judge declines, and when the judges who named a candidate reach no decision, the repair that',
  'landed the most edits goes on instead, since the panel ruled its issues real; whatever goes on must still',
  'pass the resolution checkers and beat the existing English on measurements before it ships',
].join(' ',);

/**
 What a decline costs where the caller has NOTHING to fall back on.

 @example
 ```ts
 const consequence = LEAVES_PASSAGE_UNTRANSLATED;
 ```
 */
export const LEAVES_PASSAGE_UNTRANSLATED: string =
  'there is no existing translation of this passage, so declining every candidate leaves it untranslated '
    + 'rather than falling back on anything';

/**
 What a decline costs where the passage HAS wording that cannot ship: a
 standing the deterministic rule refused, or an archive rendering the floor
 refused or the adjudicators disputed. A slate declined in every round ships a
 candidate only when two or more judges named it (owner, 2026-09-27,
 "Preference + polish", narrowed on 2026-10-25 by "Two judges must name it"),
 and otherwise the archive's wording stays and the page reports it, so a judge
 told the passage stays untranslated, or that trusted text is kept, would be
 told something false about what declining does. The sheet says it in order:
 declining does not by itself keep the wording, and a judge's own decline
 names no candidate; the wording in place stays only when no candidate was
 named by two or more judges; and when one or more were, the preferred of
 those ships by a fixed order (re-check 2, finding F2; reworded in re-check 3,
 M3, because "cannot ship unreported" beside "not a quiet keep" and "stays"
 left a judge able to read its decline as the way to the stay).

 THE CLAUSES ARE PINNED BY MEANING, not by these words
 (`decline-consequence-clauses.test-fixture.ts`).

 @example
 ```ts
 const consequence = SHIPS_WHAT_TWO_JUDGES_NAMED;
 ```
 */
export const SHIPS_WHAT_TWO_JUDGES_NAMED: string = [
  'declining does not by itself keep the wording in place, and your own decline names no candidate:',
  'when no candidate is named by two or more judges, the wording in place stays and the page reports it,',
  'with your reason recorded; when one or more candidates are named by two or more judges, the preferred of',
  'those ships, by a fixed order (the repair lane\'s text, then the translate lane\'s, then the first on the',
  'slate) and not by your judgment',
].join(' ',);

/**
 What a translate slate's judges are told a decline does.

 @param incumbentKind - whether there is wording to keep

 @param withheldStanding - whether absent wording exists and cannot ship
 rather than never having existed

 @returns The consequence the sheet states

 @example
 ```ts
 const consequence = declineConsequenceFor({ incumbentKind: 'absent', withheldStanding: true, },);
 ```
 */
export function declineConsequenceFor(
  {
    incumbentKind,
    withheldStanding,
  }: {
    readonly incumbentKind: IncumbentKind;
    readonly withheldStanding: boolean;
  },
): string {
  if (incumbentKind === 'present')
    return KEEPS_TRUSTED_TEXT;
  return withheldStanding ? SHIPS_WHAT_TWO_JUDGES_NAMED : LEAVES_PASSAGE_UNTRANSLATED;
}

//endregion Selection decline consequence
