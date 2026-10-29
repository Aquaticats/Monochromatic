import type {
  IncumbentKind,
  TranslateJudgeResponsibility,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

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
 The part of a challenge's sentence about a tie or a leader that leaves some candidates behind, which both challenge
 sentences carry.

 Such a round does not end a challenge: `settleAbsentDecline` ships or keeps only when no run-off is left to narrow
 (`narrows` in `translate-stage/src/translate-runoff-tie.ts`), and otherwise raises, and `judgeSlateWithRetry` asks
 the same bench again over the finalists (`translate-stage/src/translate-retry.ts`) for as long as each round narrows.
 The finalists are the candidates tied at the top, or the leader and its runner-up where one judge alone named the
 leader (`runoffFinalists`), and only where that leaves some candidates behind: BOTH HALVES SAY SO (re-check 6, M-B),
 since a leader alone in drawing a ballot and a leader beside its runner-up in a slate of two narrow nothing. The
 sentence says "the panel", not "the same judges": a round asks a window of the bench that a hash of its sheet rotates
 (`stage/src/stage-fanout-window.ts`), and the challenge sheet differs from the first ask's, so the seats asked can
 differ (re-check 5, M-B and M-D).
 */
const ASKS_AGAIN_OVER_THE_LEADERS: string = [
  'when the round leaves some candidates tied at the top with others left behind, or a leader only one judge named',
  'beside a runner-up with others left behind, the panel is asked again first, though a round may seat other members',
  'of it, over those candidates alone, or over that leader and its runner-up, and again for as long as that narrows',
  'the question',
].join(' ',);

/**
 The part of a challenge's sentence about the rounds that leave nothing to narrow, which both challenge sentences
 carry beside the run-off part (re-check 6, M-B).

 `runoffFinalists` returns the whole slate, and `settleAbsentDecline` does not narrow, for a round in which every
 judge declined (`rejection`), for a leader alone in drawing a ballot (`fewer-than-two-backed`), and where the
 candidates to offer are the whole slate (`every-candidate-backed`): the whole slate tied, or a slate of only two, whose
 leader and runner-up are both candidates. Those are the shapes this part names, so that the run-off part is not read as
 a promise in them. How often each occurred in the recorded XingZ6021 run is in the round seven ledger entries.

 The part says "nothing to narrow" and not "no candidate behind" (re-check 7, nit 1): the run-off part says "left
 behind" for the candidates a narrowing would drop, which counts a candidate that drew no ballot, and a lone leader of
 a wide slate leaves such candidates yet narrows nothing, since a run-off needs two candidates to offer. The word
 behind has the one meaning in the sheet, the candidates a narrowing would drop.
 */
const ASKS_NO_QUESTION_WHERE_NOTHING_NARROWS: string = [
  'a round that leaves nothing to narrow (the whole slate tied, every judge declining, a leader alone in drawing a',
  'ballot, or a slate of only two) asks no such question',
].join(' ',);

/**
 What a decline costs in the challenge, whose caller ships past a decline,
 where the passage HAS wording that cannot ship: a
 standing the deterministic rule refused, or an archive rendering the floor
 refused or the adjudicators disputed. A slate declined in the challenge ships a
 candidate only when two or more judges named it (owner, 2026-09-27,
 "Preference + polish", narrowed on 2026-10-25 by "Two judges must name it"),
 and otherwise the archive's wording stays and the page reports it, so a judge
 told the passage stays untranslated, or that trusted text is kept, would be
 told something false about what declining does. Both outcomes wait on the run-off
 a partial tie sends the panel to first (re-check 5, M-B). The first ask of the pair and
 the challenge of a caller that defers are told their own sentences
 ({@link SETTLES_NOTHING_AND_ASKS_AGAIN}, {@link SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN}),
 since neither ships or keeps as this one does: the first ask settles nothing, and the deferring challenge ships only a
 level tie and otherwise sends the passage back (re-check 4, F1; corrected in re-check 5). The sheet says it in order:
 declining does not by itself keep the wording, and a judge's own decline
 names no candidate; a tie or a leader that leaves candidates behind sends the panel to a run-off, and a round that
 leaves nothing to narrow asks no such question (re-check 6, M-B; reworded in re-check 7, nit 1); once none is left,
 the wording in place stays only when no candidate was
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
  `${ASKS_AGAIN_OVER_THE_LEADERS};`,
  `${ASKS_NO_QUESTION_WHERE_NOTHING_NARROWS};`,
  'when no such question is left and no candidate is named by two or more judges, the wording in place stays and the',
  'page reports it, with your reason recorded; when no such question is left and one or more candidates are named by',
  'two or more judges, the preferred of those ships, by a fixed order (the repair lane\'s text, then the translate',
  'lane\'s, then the first on the slate) and not by your judgment',
].join(' ',);

/**
 What a decline does in the first ask of a pair over wording that cannot ship: nothing is settled, and the panel is
 asked again, though a round may seat other members of it. The ask raises its absence (`settleAbsentDecline` ships by
 preference only in a challenge) and `judgeSlateWithRetry` challenges the same bench, over only the candidates tied at
 the top where that leaves some behind, or the leader and its runner-up where one judge alone named the leader and that
 leaves some behind, and otherwise over the whole slate, and again for as long as that narrows the question. It names
 no ship and no stay, since neither is this round's to give (re-check 4, F1; re-check 5, M-B and M-D; re-check 6, M-B).

 @example
 ```ts
 const consequence = SETTLES_NOTHING_AND_ASKS_AGAIN;
 ```
 */
export const SETTLES_NOTHING_AND_ASKS_AGAIN: string = [
  'declining does not by itself keep the wording in place, and your own decline names no candidate:',
  'when the judges decline this round, it settles nothing, and the panel is asked again, though a round may seat other',
  'members of it, over only the candidates tied at the top where that leaves some behind, or over a leader only one',
  'judge named and its runner-up where that leaves some behind, and otherwise over the whole slate, and again for as',
  'long as that narrows the question',
].join(' ',);

/**
 What a decline does in a challenge whose caller defers to a later production (the translate lane's first call, whose
 follow-up round ships past a decline): a tie or a leader that leaves candidates behind sends the panel to a run-off
 first, and a round that leaves nothing to narrow asks no such question (re-check 6, M-B; re-check 7, nit 1); once none
 is left, a candidate ships only where every candidate drew equal weight and two or more judges named it
 (`breakRunoffTie`), the preferred of those by the fixed order; otherwise the round raises its absence and the lane
 produces again, shown the rejected renderings and the findings, and judges again. It names no stay, since the wording
 in place stays only in the round that ships past a decline (re-check 4, F1; re-check 5, M-B).

 @example
 ```ts
 const consequence = SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN;
 ```
 */
export const SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN: string = [
  'declining does not by itself keep the wording in place, and your own decline names no candidate:',
  `${ASKS_AGAIN_OVER_THE_LEADERS};`,
  `${ASKS_NO_QUESTION_WHERE_NOTHING_NARROWS};`,
  'when the judges decline this round and no such question is left, a candidate ships only where every candidate drew',
  'equal weight and two or more judges named it, the preferred of those by a fixed order (the repair lane\'s text,',
  'then the translate lane\'s, then the first on the slate) and not by your judgment;',
  'otherwise this round settles nothing, and the passage goes back for new proposals written against the rejected',
  'renderings and is judged again',
].join(' ',);

/**
 What a translate slate's judges are told a decline does, by the round they sit in.

 Over wording that cannot ship the sentence differs by round, since a decline does a different thing in each: the first
 ask of a pair settles nothing and the panel is asked again; the challenge of a caller that ships past a decline
 ships the preferred candidate two or more judges named, or keeps the wording; the challenge of a caller that defers
 ships only a level tie and otherwise sends the passage back for new proposals. A tie or a leader that leaves some
 candidates behind sends the panel to a run-off first in all three, and a round that leaves nothing to narrow is told it
 asks no such question (the first ask: that it is asked again over the whole slate).

 @param incumbentKind - whether there is wording to keep

 @param withheldStanding - whether absent wording exists and cannot ship
 rather than never having existed

 @param responsibility - whether this ask is the first of a pair or the challenge of a decline

 @param shipPastDecline - whether the caller ships past a decline in its challenge; read only for a challenge

 @returns Consequence the sheet states

 @example
 ```ts
 const consequence = declineConsequenceFor({ incumbentKind: 'absent', withheldStanding: true, responsibility: 'decline-challenge', shipPastDecline: true, },);
 ```
 */
export function declineConsequenceFor(
  {
    incumbentKind,
    withheldStanding,
    responsibility,
    shipPastDecline,
  }: {
    readonly incumbentKind: IncumbentKind;
    readonly withheldStanding: boolean;
    readonly responsibility: TranslateJudgeResponsibility;
    readonly shipPastDecline: boolean;
  },
): string {
  if (incumbentKind === 'present')
    return KEEPS_TRUSTED_TEXT;
  if (!withheldStanding)
    return LEAVES_PASSAGE_UNTRANSLATED;
  if (responsibility === 'initial-selection')
    return SETTLES_NOTHING_AND_ASKS_AGAIN;
  return shipPastDecline ? SHIPS_WHAT_TWO_JUDGES_NAMED : SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN;
}

//endregion Selection decline consequence
