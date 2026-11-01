import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  producerStandings,
  rankStandings,
  standingLine,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { DigestGroup, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { judgedRoundsOf, } from './editor-calibrate-standing.ts';
import type { StandingReading, } from './editor-standing-read-artifact.ts';

//region Editor standing read seat
// WHAT `editor-standing-read` PRINTS FOR ONE PIPELINE DIGEST: its entry count,
// then each seat's judged rounds and the standing of every model that wrote a
// candidate there.
//
// NEVER POOLED ACROSS PIPELINE DIGESTS. `artifact-pool.ts` refuses to pool
// results whose built output differs, and a standing is a result. Each digest
// is reported alone, with its entry count beside its round count: rounds inside
// one entry are correlated, so entries are the denominator that matters.

/**
 Prints one seat's standing within one digest.

 JUDGED MEANS A BALLOT WAS CAST, read through `judgedRoundsOf` and so
 `roundWasJudged`, the predicate the calibration's standing and closing
 paragraph read. Until 2026-10-06 this
 counted every recorded round as judged, so a slate of one candidate, which
 needs no vote, printed as a judged round with an `UNJUDGED` row.

 @param seat - seat the standing is about

 @param perChunk - that seat's rounds, grouped by the chunk that bought them

 @example
 ```ts
 printSeat({ seat: 'EDITOR', perChunk, },);
 ```
 */
function printSeat(
  {
    seat,
    perChunk,
  }: {
    readonly seat: string;
    readonly perChunk: readonly (readonly SelectionRound[])[];
  },
): void {
  /**
   Every round this seat produced under this digest, judged or not.
   */
  const bought = perChunk.flat();

  /**
   The rounds a ballot was cast on.
   */
  const rounds = judgedRoundsOf({ rounds: bought, },);

  /**
   Rounds no ballot was cast on, counted beside the judged ones.
   */
  const unjudged = bought.length - rounds.length;

  /**
   Chunks that produced a judged round, which the round count alone hides: a
   chunk carrying no accepted issue never asks an editor to write.
   */
  const paid = perChunk.filter(function contributed(chunk,): boolean {
    return judgedRoundsOf({ rounds: chunk, },)
      .length
      > 0;
  },);

  console.log(
    `  ${seat}: ${String(rounds.length,)} judged ${
      wordForCount({
        count: rounds.length,
        one: 'round',
        many: 'rounds',
      },)
    } from ${String(paid.length,)} `
      + `of ${String(perChunk.length,)} ${
        wordForCount({
          count: perChunk.length,
          one: 'chunk',
          many: 'chunks',
        },)
      }${
        (unjudged === 0)
          ? ''
          : `; ${String(unjudged,)} ${
            wordForCount({
              count: unjudged,
              one: 'round',
              many: 'rounds',
            },)
          } drew no ballot`
      }`,
  );

  if (rounds.length === 0)
    return;

  for (const standing of rankStandings({ standings: producerStandings({ rounds, },), },)) {
    console.log(`      ${standingLine({ standing, },)}`,);
  }
}

/**
 Prints both seats for one digest, with the denominator that governs them.

 @param group - readings sharing one built output

 @example
 ```ts
 printDigestStanding({ group, },);
 ```
 */
export function printDigestStanding(
  { group, }: { readonly group: DigestGroup<StandingReading>; },
): void {
  console.log(
    `\n${group.digest} over ${String(group.readings
      .length,)} ${
      wordForCount({
        count: group.readings
          .length,
        one: 'entry',
        many: 'entries',
      },)
    }`,
  );

  for (
    const seat of [
      {
        name: 'EDITOR ',
        perChunk: group.readings
          .flatMap(function editorOf(reading,) {
            return reading.editor;
          },),
      },
      {
        name: 'REFINER',
        perChunk: group.readings
          .flatMap(function refinerOf(reading,) {
            return reading.refiner;
          },),
      },
    ]
  ) {
    printSeat({
      seat: seat.name,
      perChunk: seat.perChunk,
    },);
  }
}

/**
 Every round one artifact recorded, at either seat.

 @param reading - artifact read under the current roster

 @returns Its editor rounds, then its refiner rounds, judged or not

 @example
 ```ts
 const rounds = everyStandingRound({ reading, },);
 ```
 */
export function everyStandingRound({ reading, }: { readonly reading: StandingReading; },): readonly SelectionRound[] {
  return [
    ...reading.editor
      .flat(),
    ...reading.refiner
      .flat(),
  ];
}

//endregion Editor standing read seat
