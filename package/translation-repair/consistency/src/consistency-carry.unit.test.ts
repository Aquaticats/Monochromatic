/**
 Tests for what a receiving place's carry decides without a model: which candidate
 texts are distinct and in what order the sheet shows them, and what the choices
 among them come to.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { createHash, } from 'node:crypto';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup';
import {
  carryChoiceOf,
  carryOffersOf,
  carryRunOffFinalistsOf,
  type ConsistencyChoiceVoice,
} from '../dist/final/node/index.mjs';

/**
 Hex digest the candidate order is taken by.

 @param text - candidate text

 @returns Its sha256 digest in hex

 @example
 ```ts
 const digest = digestOf({ text: 'The cat naps.', },);
 ```
 */
function digestOf({ text, }: { readonly text: string; },): string {
  return createHash('sha256',).update(text, 'utf8',).digest('hex',);
}

/**
 Builds one heard choice.

 @param seat - seat that answered

 @param choice - label chosen

 @returns Choice with its seat

 @example
 ```ts
 const voice = voiceOf({ seat: 'a', choice: 'CANDIDATE 2', },);
 ```
 */
function voiceOf({ seat, choice, }: { readonly seat: string; readonly choice: string; },): ConsistencyChoiceVoice {
  return {
    seat,
    reason: 'It reads as the refrain reads.',
    choice,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: carryOffersOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'OFFERS THE CURRENT TEXT ALONE when there is no splice and no writer, labelled CANDIDATE 1',
          fn: async () => {
            expect(carryOffersOf({ current: 'The cat naps.', splice: [], written: [], lineStructured: false, },),).toEqual([
              { label: 'CANDIDATE 1', kind: 'current', writers: [], text: 'The cat naps.', },
            ],);
          },
        },),
        it({
          name: 'LABELS THE DISTINCT TEXTS FROM ONE IN THE ORDER OF THEIR DIGESTS, the current text, the splice and each writer '
            + 'among them, whoever wrote them and whatever order they were given',
          fn: async () => {
            const current = 'The cat naps in the sun.';
            const splice = 'The kitten naps in the sun.';
            const written = [
              { seat: 'qwen', text: 'The kitten dozes in the sun.', },
              { seat: 'kimi', text: 'The kitten sleeps in the sun.', },
            ];
            const forward = carryOffersOf({ current, splice: [splice,], written, lineStructured: false, },);
            const backward = carryOffersOf({
              current,
              splice: [splice,],
              written: written.toReversed(),
              lineStructured: false,
            },);
            const byDigest = [current, splice, ...written.map(function textOf({ text, },): string {
              return text;
            },),].toSorted(function ascending(left, right,): number {
              return compareCodePoints({ left: digestOf({ text: left, },), right: digestOf({ text: right, },), },);
            },);
            expect({
              texts: forward.map(function textOf({ text, },): string {
                return text;
              },),
              labels: forward.map(function labelOf({ label, },): string {
                return label;
              },),
              kinds: forward.map(function kindOf({ text, kind, writers, },): string {
                return `${kind}:${writers.join('+',)}:${text}`;
              },).toSorted(),
              sameBackward: forward.map(function textOf({ text, },): string {
                return text;
              },).join('|',) === backward.map(function textOf({ text, },): string {
                return text;
              },).join('|',),
            },).toEqual({
              texts: byDigest,
              labels: ['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3', 'CANDIDATE 4',],
              kinds: [
                'current::The cat naps in the sun.',
                'splice::The kitten naps in the sun.',
                'writer:kimi:The kitten sleeps in the sun.',
                'writer:qwen:The kitten dozes in the sun.',
              ].toSorted(),
              sameBackward: true,
            },);
          },
        },),
        it({
          name: 'JOINS TWO TEXTS THAT PUBLISH THE SAME PAGE into one candidate, which keeps the name of the current text, then '
            + 'the splice, then the first writer, and lists every seat that wrote it',
          fn: async () => {
            const offers = carryOffersOf({
              current: 'The cat naps in the sun.',
              splice: ['The kitten naps in the sun.',],
              written: [
                { seat: 'qwen', text: 'The cat naps\nin the sun.', },
                { seat: 'kimi', text: 'The kitten naps\nin the sun.', },
                { seat: 'glm', text: 'The kitten naps\nin the sun.', },
                { seat: 'mercury', text: 'The kitten dozes in the sun.', },
                { seat: 'minimax', text: 'The kitten dozes\nin the sun.', },
              ],
              lineStructured: false,
            },);
            expect(offers.map(function describeOffer({ kind, writers, text, },): string {
              return `${kind}:${writers.join('+',)}:${text}`;
            },).toSorted(),).toEqual([
              'current:qwen:The cat naps in the sun.',
              'splice:kimi+glm:The kitten naps in the sun.',
              'writer:mercury+minimax:The kitten dozes in the sun.',
            ].toSorted(),);
          },
        },),
        it({
          name: 'KEEPS A LINE BREAK AS A DIFFERENCE where the line-structure rule governs the slice, so two texts that differ '
            + 'only in their breaks are two candidates',
          fn: async () => {
            const offers = carryOffersOf({
              current: 'The cat naps.\nThe fish swims.',
              splice: [],
              written: [{ seat: 'qwen', text: 'The cat naps. The fish swims.', },],
              lineStructured: true,
            },);
            expect(offers.length,).toBe(2,);
          },
        },),
      ],
    },),
    describe({
      name: carryChoiceOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CHOOSES THE CANDIDATE A STRICT MAJORITY OF THE USABLE CHOICES NAMED when they reach the quorum',
          fn: async () => {
            const decided = carryChoiceOf({
              voices: [
                voiceOf({ seat: 'a', choice: 'CANDIDATE 2', },),
                voiceOf({ seat: 'b', choice: 'CANDIDATE 2', },),
                voiceOf({ seat: 'c', choice: 'CANDIDATE 1', },),
              ],
              labels: ['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3',],
              quorum: 2,
            },);
            expect(decided,).toEqual({ usable: 3, choice: { kind: 'chosen', label: 'CANDIDATE 2', }, },);
          },
        },),
        it({
          name: 'CHOOSES NOTHING BELOW THE QUORUM, and when nobody was heard, and names no majority for a tie',
          fn: async () => {
            const labels = ['CANDIDATE 1', 'CANDIDATE 2',];
            expect([
              carryChoiceOf({ voices: [voiceOf({ seat: 'a', choice: 'CANDIDATE 2', },),], labels, quorum: 2, },),
              carryChoiceOf({ voices: [], labels, quorum: 1, },),
              carryChoiceOf({
                voices: [voiceOf({ seat: 'a', choice: 'CANDIDATE 2', },), voiceOf({ seat: 'b', choice: 'CANDIDATE 1', },),],
                labels,
                quorum: 2,
              },),
            ],).toEqual([
              { usable: 1, choice: { kind: 'none', reason: 'no-quorum', }, },
              { usable: 0, choice: { kind: 'none', reason: 'no-quorum', }, },
              { usable: 2, choice: { kind: 'none', reason: 'no-majority', }, },
            ],);
          },
        },),
        it({
          name: 'CHOOSES NOTHING ON ONE USABLE CHOICE though the gather closed on a quorum of one (a bench of one or two '
            + 'seats), since one judge alone never decides (the lead\'s ruling of 2026-10-25), and runs nothing off; two '
            + 'usable choices under the same quorum decide, the positive control',
          fn: async () => {
            const labels = ['CANDIDATE 1', 'CANDIDATE 2',];
            const one = [voiceOf({ seat: 'a', choice: 'CANDIDATE 2', },),];
            const two = [voiceOf({ seat: 'a', choice: 'CANDIDATE 2', },), voiceOf({ seat: 'b', choice: 'CANDIDATE 2', },),];
            expect([
              carryChoiceOf({ voices: one, labels, quorum: 1, },),
              carryRunOffFinalistsOf({ voices: one, labels, quorum: 1, },),
              carryChoiceOf({ voices: two, labels, quorum: 1, },),
            ],).toEqual([
              { usable: 1, choice: { kind: 'none', reason: 'no-quorum', }, },
              [],
              { usable: 2, choice: { kind: 'chosen', label: 'CANDIDATE 2', }, },
            ],);
          },
        },),
        it({
          name: 'DOES NOT COUNT A CHOICE OF A LABEL THE SHEET NEVER SHOWED or one with a blank reason',
          fn: async () => {
            const decided = carryChoiceOf({
              voices: [
                voiceOf({ seat: 'a', choice: 'CANDIDATE 9', },),
                { seat: 'b', reason: '', choice: 'CANDIDATE 2', },
                voiceOf({ seat: 'c', choice: 'CANDIDATE 1', },),
                voiceOf({ seat: 'd', choice: 'CANDIDATE 1', },),
              ],
              labels: ['CANDIDATE 1', 'CANDIDATE 2',],
              quorum: 1,
            },);
            expect(decided,).toEqual({ usable: 2, choice: { kind: 'chosen', label: 'CANDIDATE 1', }, },);
          },
        },),
      ],
    },),
  ],
},);
