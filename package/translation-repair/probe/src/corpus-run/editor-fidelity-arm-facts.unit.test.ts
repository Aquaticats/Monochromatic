/**
 Tests for how the facts of an editor fidelity arm are read off its log: the logger's prefix taken off, a number or a word
 read after a marker, and the facts the checks need. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  bodyOf,
  countWithNoun,
  factsOf,
  numberAfter,
  usdSaid,
  wordAfter,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'editor fidelity arm facts',
  children: [
    it({
      name: 'TAKES THE LOGGER\'S LEVEL, STAMP AND TAG OFF A LINE and leaves a line that has none as it is, the tag\'s own '
        + 'brackets inside the words kept',
      fn: async () => {
        expect(bodyOf({ line: '[info] [2026-10-23T12:00:00.000Z] [probe] pool: 40 trials', },),).toBe('pool: 40 trials',);
        expect(bodyOf({ line: '[info] [2026-10-23T12:00:00.000Z] [probe] [cat] purr', },),).toBe('[cat] purr',);
        expect(bodyOf({ line: 'SEAT claude asked=1', },),).toBe('SEAT claude asked=1',);
        expect(bodyOf({ line: '[info] [only two] groups', },),).toBe('[info] [only two] groups',);
      },
    },),

    it({
      name: 'READS THE NUMBER AFTER A MARKER with its decimal point, and says minus one when the marker or a digit is absent',
      fn: async () => {
        expect(numberAfter({ text: 'pool: 40 trials', marker: 'pool: ', },),).toBe(40,);
        expect(numberAfter({ text: 'pass --max-usd 3.0000; the', marker: '--max-usd ', },),).toBe(3,);
        expect(numberAfter({ text: 'pass --max-usd 12.5;', marker: '--max-usd ', },),).toBe(12.5,);
        expect(numberAfter({ text: 'no number here', marker: 'pool: ', },),).toBe(-1,);
        expect(numberAfter({ text: 'pool: many', marker: 'pool: ', },),).toBe(-1,);
      },
    },),

    it({
      name: 'READS THE WORD AFTER A MARKER up to the next space, and nothing when the marker is absent',
      fn: async () => {
        expect(wordAfter({ text: 'manifest: 36 specifications, digest abc def', marker: 'digest ', },),).toBe('abc',);
        expect(wordAfter({ text: 'no digest', marker: 'hash ', },),).toBe('',);
      },
    },),

    it({
      name: 'WRITES A COUNT WITH ITS NOUN and dollars to the six decimals the guard prints',
      fn: async () => {
        expect(countWithNoun({ count: 1, one: 'row', many: 'rows', },),).toBe('1 row',);
        expect(countWithNoun({ count: 39, one: 'row', many: 'rows', },),).toBe('39 rows',);
        expect(usdSaid({ usd: 0.875, },),).toBe('0.875000',);
      },
    },),

    it({
      name: 'READS THE FACTS OF A LOG: the card, the pool\'s trials, the guard\'s total, the cap and the digest, minus one '
        + 'or empty for what is not there',
      fn: async () => {
        const prefix = '[info] [2026-10-23T12:00:00.000Z] [probe] ';
        const lines = [
          `${prefix}editors: claude-cat`,
          `${prefix}manifest: 36 specifications, digest d1g`,
          `${prefix}pool: 40 trials over 24 entries (insertion 22, alteration 18)`,
          `${prefix}spend guard: each call is refused, and the arm ends, when the spend would pass --max-usd 3.0000; `
            + 'the arms run before this one spent --bench-spent-usd 1.2500; x',
          `${prefix}GUARD total=0.875000 calls=40 atCeiling=0`,
        ];
        const facts = factsOf({ log: { arm: 'low', lines, exit: { recorded: false, }, }, },);
        expect(facts.card,).toBe('claude-cat',);
        expect(facts.poolTrials,).toBe(40,);
        expect(facts.guardUsd,).toBe(0.875,);
        expect(facts.maxUsd,).toBe(3,);
        expect(facts.benchSpentUsd,).toBe(1.25,);
        expect(facts.digest,).toBe('d1g',);
        const empty = factsOf({ log: { arm: 'none', lines: [], exit: { recorded: false, }, }, },);
        expect(empty.card,).toBe('',);
        expect(empty.poolTrials,).toBe(-1,);
        expect(empty.guardUsd,).toBe(-1,);
        expect(empty.maxUsd,).toBe(-1,);
        expect(empty.benchSpentUsd,).toBe(-1,);
        expect(empty.digest,).toBe('',);
        expect(empty.planned,).toBe(-1,);
        expect(empty.completed,).toBe(-1,);
        expect(empty.completedOf,).toBe(-1,);
        expect(empty.capped,).toBe(false,);
      },
    },),

    it({
      name: 'READS HOW MANY TRIALS THE ARM PLANNED AND COMPLETED and whether the guard capped it, minus one and false for '
        + 'lines that are not there',
      fn: async () => {
        const prefix = '[info] [2026-10-23T12:00:00.000Z] [probe] ';
        const facts = factsOf({
          log: {
            arm: 'high',
            lines: [
              `${prefix}planned: 80 of 88 trials`,
              `${prefix}CAPPED at unit 42 of 80: 41 units completed; the guard said so`,
              `${prefix}TRIALS completed 41 of 80`,
            ],
            exit: { recorded: false, },
          },
        },);
        expect(facts.planned,).toBe(80,);
        expect(facts.completed,).toBe(41,);
        expect(facts.completedOf,).toBe(80,);
        expect(facts.capped,).toBe(true,);
      },
    },),
  ],
},);
