import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { CoverageControlResult, } from './coverage-control.ts';

//region Coverage control probe lines
// What the coverage control probe prints, as text and nothing else: the line
// before the roster is asked, and the reading after it.

/**
 The line printed before the roster is asked.

 @param caseCount - cases gathered

 @param rosterSize - models in the roster

 @returns One line

 @example
 ```ts
 console.log(offeringLine({ caseCount: 3, rosterSize: 8, },),);
 ```
 */
export function offeringLine(
  {
    caseCount,
    rosterSize,
  }: {
    readonly caseCount: number;
    readonly rosterSize: number;
  },
): string {
  return `COVERAGE control offering ${String(caseCount,)} ${
    wordForCount({
      count: caseCount,
      one: 'case',
      many: 'cases',
    },)
  } to a roster of ${String(rosterSize,)}`;
}

/**
 The lines printed once the control has run: the reading, the cases that
 could not be damaged, and the sentence saying what it means.

 @param control - what deleting each rendering did

 @returns Lines in print order

 @example
 ```ts
 for (const line of controlLines({ control, },)) console.log(line,);
 ```
 */
export function controlLines({ control, }: { readonly control: CoverageControlResult; },): readonly string[] {
  /**
   Whether an absence vote proved reachable, with the cases it was read over.
   */
  const {
    held,
    rows,
    refusals,
    sawAbsenceOnTarget,
    sawAbsenceOnDecoy,
    decoysTaken,
  } = control;

  /**
   Cases the roster declined to call covered before anything was damaged.

   REPORTED SEPARATELY FROM CUTS THAT LEFT NOTHING because these are the wire
   voting absence on text nobody touched, which is a stronger reading than any
   damaged case can give.
   */
  const notCarried = refusals
    .filter(function declinedUndamaged(refusal,): boolean {
      return refusal.reason === 'not-carried';
    },);

  /**
   The two lines every run prints, whatever the control showed.
   */
  const reading = [
    `COVERAGE control ${held ? 'HELD' : 'DID NOT HOLD'} over ${
      String(rows.length,)
    } damaged ${
      wordForCount({
        count: rows.length,
        one: 'case',
        many: 'cases',
      },)
    }: absence votes appeared on ${
      String(sawAbsenceOnTarget,)
    } targeted ${
      wordForCount({
        count: sawAbsenceOnTarget,
        one: 'cut',
        many: 'cuts',
      },)
    } and on ${String(sawAbsenceOnDecoy,)} of ${String(decoysTaken,)} equally large ${
      wordForCount({
        count: decoysTaken,
        one: 'cut',
        many: 'cuts',
      },)
    } taken elsewhere`,
    `COVERAGE control ${String(refusals.length,)} ${
      wordForCount({
        count: refusals.length,
        one: 'case',
        many: 'cases',
      },)
    } could not be damaged: ${
      String(notCarried.length,)
    } because the roster never called them covered, ${
      String(refusals.length - notCarried.length,)
    } because what the roster anchored on was the whole page`,
  ];

  /**
   The warning about passages the roster declined to call covered, empty when
   there were none.
   */
  const undamagedAbsence = (notCarried.length > 0)
    ? [
      'THE ROSTER DECLINED TO CALL THOSE PASSAGES COVERED WITH NO DAMAGE DONE, which is an '
        + 'absence reading on standing corpus text rather than on a page this probe cut.',
    ]
    : [];
  if (held)
    return [
      ...reading,
      ...undamagedAbsence,
      'The roster voted absence once the rendering it pointed at was gone, so an absence '
        + 'vote is reachable and a run that produced none is reporting the corpus rather '
        + 'than the instrument.',
    ];
  if (rows.length === 0)
    return [
      ...reading,
      ...undamagedAbsence,
      'NOTHING WAS DAMAGED on this entry, so it says nothing either way about whether a '
        + 'deleted rendering is noticed. Read the refusals this probe printed instead: they are what the '
        + 'roster says about this page as it stands.',
    ];
  if (decoysTaken === 0)
    return [
      ...reading,
      ...undamagedAbsence,
      'NO DECOY CUT COULD BE TAKEN on any damaged case, so whether the absence votes follow the '
        + 'passage or any cut of the same size was never asked; the targeted cuts printed here show '
        + 'only whether an absence vote is reachable.',
    ];
  return [
    ...reading,
    ...undamagedAbsence,
    'THE ROSTER DID NOT VOTE ABSENCE even with the rendering it pointed at deleted, or it '
      + 'voted absence on an unrelated cut of the same size. Either way its coverage readings '
      + 'here are a property of the wire rather than of the translation.',
  ];
}

//endregion Coverage control probe lines
