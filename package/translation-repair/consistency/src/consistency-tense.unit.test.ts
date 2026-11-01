/**
 Tests for the tense block a consistency sheet carries: which English it
 reads, under which label, and when it reads nothing.

 Fixtures are cat-themed invention; no corpus text appears here.

 @module
 */

import {
  type ConsistencyEnglish,
  consistencyTenseLines,
} from '../dist/final/node/index.mjs';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 English of a place whose slice narrates its subject in the present tense.
 */
const PRESENT: ConsistencyEnglish = { kind: 'present', text: 'She is asleep in the sunny window.', };

/**
 English of a place whose slice reads in the past tense.
 */
const PAST: ConsistencyEnglish = { kind: 'present', text: 'She fell asleep in the sunny window.', };

/**
 The lines the block prints for the present-tense English under the first label.
 */
const PRESENT_LINES: readonly string[] = [
  'These sentences match present-tense shapes in a memorial page about a person who has died.',
  'Check each against the house tense rule. A candidate may be a trait, a standing fact or a quotation',
  'the rule leaves alone, so it is a question for you and not a finding against the text.',
  'OCCURRENCE 1 TRANSLATION:',
  '- "She is asleep in the sunny window." (matched "She is", rule copula-or-auxiliary)',
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: consistencyTenseLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRINTS THE SHARED BLOCK FOR A SLICE\'S ENGLISH under the label the sheet gives it',
          fn: async () => {
            expect(consistencyTenseLines({
              lifeEnded: true,
              places: [{ sliceIndex: 0, english: PRESENT, }, { sliceIndex: 1, english: PAST, },],
            },),).toEqual(PRESENT_LINES,);
          },
        },),
        it({
          name: 'LABELS A LATER PLACE\'S SLICE BY THE PLACE THAT SHOWS IT, and shows a slice only once',
          fn: async () => {
            expect(consistencyTenseLines({
              lifeEnded: true,
              places: [
                { sliceIndex: 0, english: PAST, },
                { sliceIndex: 1, english: PRESENT, },
                { sliceIndex: 1, english: PRESENT, },
              ],
            },),).toEqual(PRESENT_LINES.map(function second(line,): string {
              return line.replace('OCCURRENCE 1 TRANSLATION',  'OCCURRENCE 2 TRANSLATION',);
            },),);
          },
        },),
        it({
          name: 'PRINTS NOTHING for an entry that records no death, a slice with no English, or English with no candidate',
          fn: async () => {
            expect([
              consistencyTenseLines({ lifeEnded: false, places: [{ sliceIndex: 0, english: PRESENT, },], },),
              consistencyTenseLines({ lifeEnded: true, places: [{ sliceIndex: 0, english: { kind: 'absent', }, },], },),
              consistencyTenseLines({ lifeEnded: true, places: [{ sliceIndex: 0, english: PAST, },], },),
            ],).toEqual([[], [], [],],);
          },
        },),
        it({
          name: 'READS THE DECLARED SUBJECT NAMES as subjects',
          fn: async () => {
            expect(consistencyTenseLines({
              lifeEnded: true,
              subjectNames: ['Mittens',],
              places: [{ sliceIndex: 0, english: { kind: 'present', text: 'Mittens is a tabby with a white chin.', }, },],
            },),).toEqual([
              'These sentences match present-tense shapes in a memorial page about a person who has died.',
              'Check each against the house tense rule. A candidate may be a trait, a standing fact or a quotation',
              'the rule leaves alone, so it is a question for you and not a finding against the text.',
              'OCCURRENCE 1 TRANSLATION:',
              '- "Mittens is a tabby with a white chin." (matched "Mittens is", rule copula-or-auxiliary)',
            ],);
          },
        },),
      ],
    },),
  ],
},);
