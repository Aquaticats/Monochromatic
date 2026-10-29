/**
 Tests for the decoded readings of a reply text the credential mask searches,
 each decoded unit keeping the stretch of the original it came from.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { decodedViewsOf, } from '../dist/final/node/index.mjs';

/**
 Test that wants every decoded unit.

 @returns True

 @example
 ```ts
 wantsEveryUnit();
 ```
 */
function wantsEveryUnit(): boolean {
  return true;
}

/**
 Test that wants only the hyphen.

 @param unit - decoded unit

 @returns Whether it is a hyphen

 @example
 ```ts
 wantsHyphen({ unit: '-', },); // true
 ```
 */
function wantsHyphen({ unit, }: { readonly unit: string; },): boolean {
  return unit === '-';
}

/**
 Texts of the views of a reply.

 @param text - reply text

 @param wanted - which decoded units count

 @returns Each view's decoded text

 @example
 ```ts
 textsOf({ text: 'a%2Db', wanted: wantsEveryUnit, },); // ['a-b']
 ```
 */
function textsOf(
  {
    text,
    wanted,
  }: {
    readonly text: string;
    readonly wanted: (parameters: { readonly unit: string; },) => boolean;
  },
): readonly string[] {
  return decodedViewsOf({
    text,
    wanted,
  },)
    .map(function textOfView(view,): string {
      return view.text;
    },);
}

await describe({
  name: decodedViewsOf.name,
  children: [
    it({
      name: 'READS NO VIEW for a text with neither a backslash nor a percent sign, and for escapes no needle could hold',
      fn: async () => {
        expect(textsOf({ text: 'the cat napped', wanted: wantsEveryUnit, },),).toEqual([],);
        expect(textsOf({ text: String.raw`a\nb \" c %41 d`, wanted: wantsHyphen, },),).toEqual([],);
      },
    },),
    it({
      name: 'DECODES JSON UNICODE ESCAPES of either hex case, and leaves a backslash that opens no escape',
      fn: async () => {
        expect(textsOf({ text: String.raw`a\u002db\u002Dc\u00zz\x\u00`, wanted: wantsEveryUnit, },),).toEqual([
          String.raw`a-b-c\u00zz\x\u00`,
        ],);
      },
    },),
    it({
      name: 'DECODES EVERY SIMPLE JSON ESCAPE, an escaped backslash leaving a second view for what it uncovers',
      fn: async () => {
        expect(textsOf({ text: String.raw`\"\\\/\b\f\n\r\t`, wanted: wantsEveryUnit, },),).toEqual([
          '"\\/\b\f\n\r\t',
          '"/\b\f\n\r\t',
        ],);
      },
    },),
    it({
      name: 'DECODES PERCENT ESCAPES of either hex case, and leaves a percent sign that opens none',
      fn: async () => {
        expect(textsOf({ text: 'a%2db%2Dc%zz%4', wanted: wantsEveryUnit, },),).toEqual(['a-b-c%zz%4',],);
      },
    },),
    it({
      name: 'READS A CHAIN of decoders: an escaped backslash before a unicode escape, and a unicode escape of a percent sign',
      fn: async () => {
        expect(textsOf({ text: String.raw`x\\u002dy`, wanted: wantsHyphen, },),).toEqual([
          String.raw`x\u002dy`,
          'x-y',
        ],);
        expect(textsOf({ text: String.raw`x\u0025002Dy`, wanted: wantsHyphen, },),).toEqual([
          'x%002Dy',
        ],);
        expect(textsOf({ text: String.raw`x\u0025\u0032Dy`, wanted: wantsEveryUnit, },),).toEqual([
          'x%2Dy',
          'x-y',
        ],);
      },
    },),
    it({
      name: 'LEAVES AN ESCAPE ALONE whose decoded unit no needle holds while decoding the ones that are wanted',
      fn: async () => {
        expect(textsOf({ text: String.raw`a\u002db\u0041c`, wanted: wantsHyphen, },),).toEqual([
          String.raw`a-b\u0041c`,
        ],);
      },
    },),
    it({
      name: 'KEEPS WHERE EACH DECODED UNIT CAME FROM, an escape standing for the whole stretch it was written as',
      fn: async () => {
        /**
         The one view of a text with a six-unit escape between two plain units.
         */
        const [view,] = decodedViewsOf({ text: String.raw`a\u002db`, wanted: wantsHyphen, },);
        expect(view?.text,).toBe('a-b',);
        expect([...view?.starts ?? [],],).toEqual([0, 1, 7,],);
        expect([...view?.ends ?? [],],).toEqual([1, 7, 8,],);
      },
    },),
    it({
      name: 'READS ONE VIEW for chains that decode independent escapes in either order to the same text',
      fn: async () => {
        expect(textsOf({ text: String.raw`\u002d%2D`, wanted: wantsHyphen, },),).toEqual([
          String.raw`-%2D`,
          String.raw`\u002d-`,
          '--',
        ],);
      },
    },),
  ],
},);
