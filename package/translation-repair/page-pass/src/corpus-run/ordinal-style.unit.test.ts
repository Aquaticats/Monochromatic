/**
 Tests for how a numbered heading spells its number (ledger T8): the style a
 rendered heading's prefix reads as, the number rendered back in any style,
 and the Han or digit numeral the original numbers the heading with.

 The series pass reached only some of these paths through its own fixtures;
 every form, leader and refusal is driven here directly, and every style
 read back from what it renders.

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
  NO_NUMBER,
  type OrdinalStyle,
  readHanNumeral,
  readOrdinalStyle,
  renderOrdinal,
  styleKey,
} from '../../dist/final/node/index.mjs';

/**
 Every form a numbered heading can take.
 */
const FORMS: readonly OrdinalStyle['form'][] = [
  'cardinal',
  'ordinal',
  'roman',
  'arabic',
];

/**
 Largest number a series heading carries.
 */
const SERIES_MAX = 99;

/**
 Largest number the cardinal and ordinal word tables spell.
 */
const WORD_TABLE_MAX = 20;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readOrdinalStyle.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the form of the last word and every word before it as the leader, lowered',
          fn: async () => {
            expect(readOrdinalStyle({ prefix: 'Part Six', },),).toEqual({ leader: 'part', form: 'cardinal', },);
            expect(readOrdinalStyle({ prefix: 'The Fifth', },),).toEqual({ leader: 'the', form: 'ordinal', },);
            expect(readOrdinalStyle({ prefix: 'Poem 7', },),).toEqual({ leader: 'poem', form: 'arabic', },);
            expect(readOrdinalStyle({ prefix: 'Whisker Song XLIX', },),).toEqual({ leader: 'whisker song', form: 'roman', },);
            expect(readOrdinalStyle({ prefix: 'IV', },),).toEqual({ leader: '', form: 'roman', },);
          },
        },),
        it({
          name: 'READS "No." WITH OR WITHOUT ITS STOP as one leader, so a series mixing the two is one style',
          fn: async () => {
            expect(readOrdinalStyle({ prefix: 'No. 3', },),).toEqual({ leader: 'no', form: 'arabic', },);
            expect(readOrdinalStyle({ prefix: 'No 3', },),).toEqual({ leader: 'no', form: 'arabic', },);
          },
        },),
        it({
          name: 'READS NO NUMBER from an empty prefix or one whose last word is none',
          fn: async () => {
            expect(readOrdinalStyle({ prefix: '', },),).toBe(NO_NUMBER,);
            expect(readOrdinalStyle({ prefix: 'The Sleepy Kitten', },),).toBe(NO_NUMBER,);
          },
        },),
        it({
          name: 'READS ONLY A ROMAN NUMERAL A SERIES CAN CARRY as roman: a capitalised word of roman letters, a '
            + 'malformed numeral and one past ninety-nine are no number (ledger T8)',
          fn: async () => {
            expect(readOrdinalStyle({ prefix: 'Mittens DID', },),).toBe(NO_NUMBER,);
            expect(readOrdinalStyle({ prefix: 'Part IIII', },),).toBe(NO_NUMBER,);
            expect(readOrdinalStyle({ prefix: 'Part IC', },),).toBe(NO_NUMBER,);
            expect(readOrdinalStyle({ prefix: 'Part C', },),).toBe(NO_NUMBER,);
            expect(readOrdinalStyle({ prefix: 'Part iv', },),).toBe(NO_NUMBER,);
          },
        },),
      ],
    },),

    describe({
      name: renderOrdinal.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RENDERS each form with its leader capitalised word by word, and "No." for the number sign',
          fn: async () => {
            expect(renderOrdinal({ value: 6, style: { leader: 'part', form: 'cardinal', }, },),).toBe('Part Six',);
            expect(renderOrdinal({ value: 4, style: { leader: '', form: 'ordinal', }, },),).toBe('Fourth',);
            expect(renderOrdinal({ value: 49, style: { leader: 'whisker song', form: 'roman', }, },),)
              .toBe('Whisker Song XLIX',);
            expect(renderOrdinal({ value: 12, style: { leader: 'no', form: 'arabic', }, },),).toBe('No. 12',);
          },
        },),
        it({
          name: 'RENDERS DIGITS PAST THE WORD TABLES rather than inventing a word, and nothing for no number',
          fn: async () => {
            expect(renderOrdinal({ value: 21, style: { leader: 'part', form: 'cardinal', }, },),).toBe('Part 21',);
            expect(renderOrdinal({ value: 21, style: { leader: '', form: 'ordinal', }, },),).toBe('21',);
            expect(renderOrdinal({ value: 3, style: NO_NUMBER, },),).toBe('',);
          },
        },),
        it({
          name: 'READS BACK EVERY STYLE IT RENDERS, for every number a series carries, so re-rendering a series '
            + 'never changes its style',
          fn: async () => {
            /**
             Every value, form and leader whose rendering did not read back as its style.
             */
            const drifted: string[] = [];
            for (const form of FORMS) {
              for (const leader of ['', 'part', 'no',]) {
                for (let value = 1; value <= SERIES_MAX; value += 1) {
                  /**
                   Style rendered.
                   */
                  const style: OrdinalStyle = { leader, form, };
                  /**
                   What it reads back as.
                   */
                  const read = readOrdinalStyle({ prefix: renderOrdinal({ value, style, },), },);
                  // Past the word tables a word form renders digits, which read
                  // back as arabic by design.
                  const expected: OrdinalStyle = ((form === 'cardinal') || (form === 'ordinal')) && (value > WORD_TABLE_MAX)
                    ? { leader, form: 'arabic', }
                    : style;
                  if (styleKey({ style: read, },) !== styleKey({ style: expected, },))
                    drifted.push(`${String(value,)} ${styleKey({ style, },)}`,);
                }
              }
            }
            expect(drifted,).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: styleKey.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES a style by leader and form, and a style with no leader by its form alone',
          fn: async () => {
            expect(styleKey({ style: { leader: 'part', form: 'cardinal', }, },),).toBe('part cardinal',);
            expect(styleKey({ style: NO_NUMBER, },),).toBe('none',);
          },
        },),
      ],
    },),

    describe({
      name: readHanNumeral.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS digits and Han numerals from one to ninety-nine, a bare or a leading ten included',
          fn: async () => {
            expect(readHanNumeral({ text: '12', },),).toBe(12,);
            expect(readHanNumeral({ text: '三', },),).toBe(3,);
            expect(readHanNumeral({ text: '十', },),).toBe(10,);
            expect(readHanNumeral({ text: '十二', },),).toBe(12,);
            expect(readHanNumeral({ text: '一十', },),).toBe(10,);
            expect(readHanNumeral({ text: '二十', },),).toBe(20,);
            expect(readHanNumeral({ text: '九十九', },),).toBe(99,);
          },
        },),
        it({
          name: 'READS ZERO from a second ten, a second digit in one place, a character that is no numeral, and '
            + 'nothing, so a heading the original does not number is never counted',
          fn: async () => {
            expect(readHanNumeral({ text: '十十', },),).toBe(0,);
            expect(readHanNumeral({ text: '二二', },),).toBe(0,);
            expect(readHanNumeral({ text: '猫', },),).toBe(0,);
            expect(readHanNumeral({ text: '', },),).toBe(0,);
          },
        },),
        it({
          name: 'READS ZERO from a digit run past the largest whole number a double holds exactly, which `Number` '
            + 'reads as a neighbouring number, so two different headings would count as one (ledger B73)',
          fn: async () => {
            expect(readHanNumeral({ text: String(BigInt(Number.MAX_SAFE_INTEGER,) + 2n,), },),).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
