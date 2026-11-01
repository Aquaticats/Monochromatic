/**
 Tests for the class of the character at an edge of a text, as far as a single mark beside the text can tell: a Latin letter
 with its accent or an ASCII digit attaches, any other character is another class, an empty text is none, and two texts have
 a character of one class at the same edge only when the classes are equal.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  edgeClassOf,
  sameEdgeClass,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: edgeClassOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CLASSES A WHOLE CHARACTER WHOSE NFKC FORM IS LATIN AS ATTACHING, at both edges, and one whose form is not as other',
          fn: async () => {
            expect([
              // A mathematical bold script capital Q, beyond the first plane, at the start.
              edgeClassOf({ text: '\u{1D4E0}uan', side: 'start', },),
              // A fullwidth capital A at the end.
              edgeClassOf({ text: 'the \u{FF21}', side: 'end', },),
              // A fullwidth digit one at the end.
              edgeClassOf({ text: 'room \u{FF11}', side: 'end', },),
              // A micro sign, NFKC Greek mu, at the start.
              edgeClassOf({ text: '\u{00B5}s', side: 'start', },),
              // A circled digit one (category No) at the end, and a trade mark sign at the start.
              edgeClassOf({ text: 'room \u{2460}', side: 'end', },),
              edgeClassOf({ text: '\u{2122}s', side: 'start', },),
            ],).toEqual(['attaches', 'attaches', 'attaches', 'other', 'other', 'other',],);
          },
        },),
        it({
          name: 'CLASSES A LATIN LETTER, WITH ITS ACCENT, OR AN ASCII DIGIT AS ATTACHING, any other character, a letter of another '
            + 'script included, as other and an empty text as none, at the start and at the end',
          fn: async () => {
            const small = String.fromCodePoint(0x1_04_28,);
            expect([
              edgeClassOf({ text: 'the cats', side: 'start', },),
              edgeClassOf({ text: 'the cats', side: 'end', },),
              edgeClassOf({ text: '3 cats', side: 'start', },),
              edgeClassOf({ text: '猫猫', side: 'end', },),
              edgeClassOf({ text: `${small} cat`, side: 'start', },),
              edgeClassOf({ text: `a cat ${small}`, side: 'end', },),
              edgeClassOf({ text: '**cat**', side: 'start', },),
              edgeClassOf({ text: 'the (grey cat)', side: 'end', },),
              edgeClassOf({ text: '"cat"', side: 'end', },),
              edgeClassOf({ text: '(cat) naps', side: 'start', },),
              edgeClassOf({ text: '', side: 'start', },),
              edgeClassOf({ text: '', side: 'end', },),
              edgeClassOf({ text: 'the café', side: 'end', },),
            ],).toEqual([
              'attaches',
              'attaches',
              'attaches',
              'other',
              'other',
              'other',
              'other',
              'other',
              'other',
              'other',
              'none',
              'none',
              'attaches',
            ],);
          },
        },),
      ],
    },),
    describe({
      name: sameEdgeClass.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS TRUE WHEN BOTH EDGES ARE ATTACHING, BOTH ARE OTHER OR BOTH ARE EMPTY, and false when they are of different '
            + 'classes, an empty edge being a class that no character equals',
          fn: async () => {
            expect([
              sameEdgeClass({ left: 'the cats', right: 'the kittens', side: 'end', },),
              sameEdgeClass({ left: '**cat**', right: '(grey cat)', side: 'end', },),
              sameEdgeClass({ left: '', right: '', side: 'start', },),
              sameEdgeClass({ left: 'the cats', right: 'the **cat**', side: 'end', },),
              sameEdgeClass({ left: 'the cats', right: '(the cat', side: 'start', },),
              sameEdgeClass({ left: '', right: 'cats', side: 'end', },),
              sameEdgeClass({ left: '', right: '**cat**', side: 'start', },),
              sameEdgeClass({ left: 'the cats', right: '(the cat', side: 'end', },),
            ],).toEqual([true, true, true, false, false, false, false, true,],);
          },
        },),
      ],
    },),
  ],
},);
