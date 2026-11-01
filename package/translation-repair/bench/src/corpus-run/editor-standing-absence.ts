import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Editor standing absence
// The line `editor-standing-read` prints when nothing it read carried a
// judged round, naming which absence it was. Split out of the command, which
// keeps to its line budget.

/**
 The line a report prints when nothing it read carried a judged round.

 THREE DIFFERENT ABSENCES, each named: artifacts settled under an earlier
 roster, rounds recorded that drew no ballot, and no round recorded at all.

 @param offRoster - artifacts naming a model the roster no longer seats,
 which say why the current roster has nothing to read

 @param unjudged - rounds the readings recorded, none of which drew a ballot
 since no digest carried a judged one

 @returns One line naming which of the three absences it was, since each has
 its own remedy

 @example
 ```ts
 console.log(noJudgedRoundLine({ offRoster: 0, unjudged: 1, },),);
 ```
 */
export function noJudgedRoundLine(
  {
    offRoster,
    unjudged,
  }: {
    readonly offRoster: number;
    readonly unjudged: number;
  },
): string {
  if (offRoster > 0) {
    return `  NO ROUNDS UNDER THE CURRENT ROSTER. ${String(offRoster,)} of these artifacts ${
      wordForCount({
        count: offRoster,
        one: 'names a model the roster no longer seats, so it was settled under an earlier one and is',
        many: 'name a model the roster no longer seats, so they were settled under an earlier one and are',
      },)
    } not evidence about the models seated now. This is an absent measurement, not a poor one.`;
  }
  if (unjudged > 0) {
    return `  NO JUDGED ROUNDS. ${String(unjudged,)} ${
      wordForCount({
        count: unjudged,
        one: 'round was recorded here and drew no ballot',
        many: 'rounds were recorded here and none drew a ballot',
      },)
    }: a slate of one candidate, which is what every producer proposing the same wording leaves, needs no `
      + 'vote, and a panel whose every judge abstained or failed casts none.';
  }
  return '  NO ROUNDS. Nothing read here recorded a judged round. Artifacts settled before the '
    + 'rounds were stored carry none, and an entry whose every chunk was left unchanged '
    + 'carries none either.';
}

//endregion Editor standing absence
