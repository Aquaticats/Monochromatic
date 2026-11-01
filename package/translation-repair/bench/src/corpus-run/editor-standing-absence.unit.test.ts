/**
 Tests for the line `editor-standing-read` prints when nothing it read
 carried a judged round: which of the three absences it names, and its words
 at a count of one and of several.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { noJudgedRoundLine, } from '../../dist/final/node/index.mjs';

/**
 What every off-roster line says after its count and verb.
 */
const NOT_EVIDENCE = ' not evidence about the models seated now. This is an absent measurement, not a poor one.';

/**
 What every line about rounds that drew no ballot says after its count.
 */
const NO_VOTE_NEEDED = ': a slate of one candidate, which is what every producer proposing the same wording leaves, '
  + 'needs no vote, and a panel whose every judge abstained or failed casts none.';

await describe({
  name: noJudgedRoundLine.name,
  children: [
    it({
      name: 'NAMES ONE ARTIFACT SETTLED UNDER AN EARLIER ROSTER with the verbs in the singular',
      fn: async () => {
        expect(noJudgedRoundLine({
          offRoster: 1,
          unjudged: 0,
        },),).toBe(
          '  NO ROUNDS UNDER THE CURRENT ROSTER. 1 of these artifacts names a model the roster no longer seats, so '
            + `it was settled under an earlier one and is${NOT_EVIDENCE}`,
        );
      },
    },),
    it({
      name: 'NAMES SEVERAL ARTIFACTS SETTLED UNDER AN EARLIER ROSTER with the verbs in the plural, before any '
        + 'round that drew no ballot',
      fn: async () => {
        expect(noJudgedRoundLine({
          offRoster: 2,
          unjudged: 3,
        },),).toBe(
          '  NO ROUNDS UNDER THE CURRENT ROSTER. 2 of these artifacts name a model the roster no longer seats, so '
            + `they were settled under an earlier one and are${NOT_EVIDENCE}`,
        );
      },
    },),
    it({
      name: 'COUNTS ONE ROUND THAT DREW NO BALLOT in the singular',
      fn: async () => {
        expect(noJudgedRoundLine({
          offRoster: 0,
          unjudged: 1,
        },),).toBe(`  NO JUDGED ROUNDS. 1 round was recorded here and drew no ballot${NO_VOTE_NEEDED}`,);
      },
    },),
    it({
      name: 'COUNTS SEVERAL ROUNDS THAT DREW NO BALLOT in the plural',
      fn: async () => {
        expect(noJudgedRoundLine({
          offRoster: 0,
          unjudged: 2,
        },),).toBe(`  NO JUDGED ROUNDS. 2 rounds were recorded here and none drew a ballot${NO_VOTE_NEEDED}`,);
      },
    },),
    it({
      name: 'SAYS NO ROUND WAS RECORDED where nothing names an earlier roster and no round was recorded',
      fn: async () => {
        expect(noJudgedRoundLine({
          offRoster: 0,
          unjudged: 0,
        },),).toBe(
          '  NO ROUNDS. Nothing read here recorded a judged round. Artifacts settled before the rounds were stored '
            + 'carry none, and an entry whose every chunk was left unchanged carries none either.',
        );
      },
    },),
  ],
},);
