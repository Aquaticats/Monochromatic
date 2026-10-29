/**
 Tests that every rendered sheet of the page consistency pass that asks for a label names, in plain words, the exact
 labels the reply may use (the lead's ruling of 2026-10-15): the first round's three label fields, the run-off's
 finalists only and a choice among candidates.

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
  renderedSheets,
  type RenderedSheet,
} from './rendered-sheets.test-fixture.ts';

/**
 Marker of the sentence that names the labels a reply may use.
 */
const LABELS_MARKER = ' are exactly: ';

/**
 The sheets that end by asking for a label in a reply of JSON.

 @returns Consistency sheets whose reply shape names a choice

 @example
 ```ts
 const sheets = sheetsAskingForALabel();
 ```
 */
function sheetsAskingForALabel(): readonly RenderedSheet[] {
  return renderedSheets()
    .filter(function asksForALabel({ name, text, },): boolean {
      return name.startsWith('page consistency',) && text.includes('"choice":"<',);
    },);
}

/**
 Whether a text has the shape of a label a consistency sheet shows: `OCCURRENCE` or `CANDIDATE`, one space, and a number.

 @param label - text the sheet names as a label

 @returns Whether it is a word of the two, a space and ASCII digits, and nothing else

 @example
 ```ts
 const shaped = isLabelShape({ label: 'CANDIDATE 2', },); // true
 ```
 */
function isLabelShape({ label, }: { readonly label: string; },): boolean {
  /**
   The label's word, its number, and anything after a second space.
   */
  const [word, number, ...rest] = label.split(' ',);
  return ((word === 'OCCURRENCE') || (word === 'CANDIDATE'))
    && (rest.length === 0)
    && (number !== undefined)
    && (number !== '')
    && Array.from(number,).every(function isDigit(character,): boolean {
      return (character >= '0') && (character <= '9');
    },);
}

/**
 The labels a sheet names as the ones its reply may use.

 @param text - sheet

 @returns Labels, in the order the sentence lists them

 @example
 ```ts
 const labels = namedLabelsOf({ text, },);
 ```
 */
function namedLabelsOf({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Where the sentence's list starts.
   */
  const marker = text.indexOf(LABELS_MARKER,);
  if (marker === (-1))
    return [];
  /**
   The list, up to the full stop that ends it.
   */
  const list = text.slice(marker + LABELS_MARKER.length,)
    .split('.',)[0] ?? '';
  return list.split(', ',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'the rendered consistency sheets that ask for a label',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COVER THE FIRST ROUND, THE RUN-OFF AND THE CHOICE AMONG CANDIDATES, each more than once',
          fn: async () => {
            const names = sheetsAskingForALabel()
              .map(function nameOf({ name, },): string {
                return name;
              },);
            expect(names.filter(function isFirstRound(name,): boolean {
              return name.startsWith('page consistency, ',);
            },).length,).toBeGreaterThanOrEqual(4,);
            expect(names.filter(function isRunOff(name,): boolean {
              return name.startsWith('page consistency run-off',);
            },).length,).toBeGreaterThanOrEqual(3,);
            expect(names.filter(function isChoice(name,): boolean {
              return name.startsWith('page consistency carry choice',);
            },).length,).toBeGreaterThanOrEqual(3,);
          },
        },),
        it({
          name: 'NAME THE EXACT LABELS THE REPLY MAY USE, each label one the sheet shows beside a place or a candidate',
          fn: async () => {
            /**
             What is wrong with a sheet's naming of labels, one entry per fault.
             */
            const faults: string[] = [];
            for (const { name, text, } of sheetsAskingForALabel()) {
              const labels = namedLabelsOf({ text, },);
              if (labels.length === 0)
                faults.push(`${name} names no labels`,);
              for (const label of labels) {
                if (!isLabelShape({ label, },))
                  faults.push(`${name}: ${label} is no label`,);
                // The label is also a fence header of a block of the sheet, so the reply may use only what the sheet shows.
                if (!text.includes(` ${label} `,))
                  faults.push(`${name}: ${label} is shown nowhere`,);
              }
              if (new Set(labels,).size !== labels.length)
                faults.push(`${name} repeats a label`,);
            }
            expect(faults,).toEqual([],);
          },
        },),
        it({
          name: 'NAME ONLY THE FINALISTS ON A RUN-OFF AMONG SOME OF THE PLACES, so the second place is no label of the sheet that '
            + 'runs off the first and the third',
          fn: async () => {
            const subset = sheetsAskingForALabel()
              .find(function isSubset({ name, },): boolean {
                return name === 'page consistency run-off, repeated sentence, first and third of three places';
              },);
            expect(subset === undefined,).toBe(false,);
            expect(namedLabelsOf({ text: subset?.text ?? '', },),).toEqual(['OCCURRENCE 1', 'OCCURRENCE 3',],);
            expect((subset?.text ?? '').includes('OCCURRENCE 2',),).toBe(false,);
          },
        },),
        it({
          name: 'SAY THAT A LABEL IS WRITTEN AS SHOWN, WITH ITS WORD, since a number alone is what two seats of the last run wrote',
          fn: async () => {
            expect(
              sheetsAskingForALabel()
                .filter(function saysNothing({ text, },): boolean {
                  return !text.includes('a number alone is not a label',);
                },)
                .map(function nameOf({ name, },): string {
                  return name;
                },),
            ).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
