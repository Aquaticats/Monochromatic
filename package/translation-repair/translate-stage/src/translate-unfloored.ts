import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  type IncumbentKind,
  NOT_ON_SLATE,
  TranslateAbsenceError,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  EMPTY_TALLY,
  type TranslateStageResult,
} from './translate-stage-result.ts';

//region Unfloored slice
// What the translate stage settles on, before any round, for a slice the
// deterministic source floor can compare nothing on (ledger B43): an original
// no grammar reads, or a page none does.
//
// NOTHING IS ASKED, because nothing asked could be used. The floor passes no
// candidate on such a slice; the author repair let one stand unvalidated, so
// an admitted insertion shipped a rendering nobody could check, and on a slice
// the archive translates the consolidation refused every text alike and
// shipped the archive, so the calls bought nothing. A finding about an
// unreadable original is also one no translator can act on, so a revision
// round is not the place either.

/**
 Names a slice a stage settled without asking, the way every stage finding
 is named.

 @param stage - which stage settled it: the translate stage (ledger B43),
 the consolidation (ledger B45) or the consolidation polish (ledger B48)

 @param detail - why the floor could compare nothing, in its own words

 @returns Finding in scorecard-stable wording

 @example
 ```ts
 const finding = unflooredFinding({ stage: 'translate', detail: 'original could not be read: ...', },);
 ```
 */
export function unflooredFinding(
  {
    stage,
    detail,
  }: {
    readonly stage: 'translate' | 'consolidate' | 'consolidation-polish';
    readonly detail: string;
  },
): string {
  return `${stage}-unfloored (${detail})`;
}

/**
 Settles a slice the floor can compare nothing on: on the archive where
 there is one, unfilled where there is none.

 @param incumbentText - archive wording as the stage judges it

 @param incumbentKind - whether the archive holds a translation here

 @param detail - why the floor could compare nothing

 @param l - stage logger

 @returns Archive standing, with nobody asked

 @throws {@link TranslateAbsenceError} carrying `unfloored` where the archive
 holds no translation, since nothing may be written and nothing kept

 @example
 ```ts
 return settleUnflooredSlice({ incumbentText, incumbentKind, detail: reach.detail, l, },);
 ```
 */
export function settleUnflooredSlice(
  {
    incumbentText,
    incumbentKind,
    detail,
    l,
  }: {
    readonly incumbentText: string;
    readonly incumbentKind: IncumbentKind;
    readonly detail: string;
    readonly l: Logger;
  },
): TranslateStageResult {
  /**
   Logger tagged with this settlement.
   */
  const ul = tagged({
    tag: settleUnflooredSlice.name,
    l,
  },);

  /**
   What the slice reports either way.
   */
  const findings = [
    unflooredFinding({
      stage: 'translate',
      detail,
    },),
  ];
  if (incumbentKind === 'absent') {
    ul.warn(`translate stage: asked nobody, since the floor can compare nothing here (${detail}); `
      + 'the passage stays unfilled',);
    throw new TranslateAbsenceError({
      reason: 'unfloored',
      findings,
    },);
  }
  ul.warn(`translate stage: asked nobody, since the floor can compare nothing here (${detail}); `
    + 'the archive stands',);
  return {
    text: incumbentText,
    origin: 'incumbent',
    producer: {
      kind: 'incumbent',
      matched: [],
    },
    decision: 'unfloored',
    voteWeight: 0,
    tally: EMPTY_TALLY,
    ballots: [],
    heardTranslators: 0,
    candidateCount: 0,
    findings,
    slate: [],
    selectedIndex: NOT_ON_SLATE,
    shippedIndex: NOT_ON_SLATE,
    perCandidate: [],
  };
}

//endregion Unfloored slice
