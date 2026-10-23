/**
 Guards the premise of ledger M28 for the five prompt-keyed caches (R4 of the third review of the Claude route): the key
 of a stored record hashes a version, the shape of the run and the texts, and reads no sheet builder, so a change to a
 sheet's wording moves no key and only the version number retires a record written under the old wording.

 The census is the imports of the five key files. A key file that began to import a wire or sheet module would hash the
 sheet itself, and the hand bump this change makes would stop being the only way to retire a record; the case then fails
 and the account of each version has to be read again.

 @module
 */

import { readFileSync, } from 'node:fs';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { sourcePathOf, } from './family-source.test-fixture.ts';

/**
 The key files of the five caches whose key does not read the sheet, with the constant each hashes.
 */
const KEY_FILES: readonly {
  readonly path: string;
  readonly constant: string;
}[] = [
  { path: 'repair/repair-slice-key.ts', constant: 'SLICE_CACHE_VERSION', },
  { path: 'repair/refine-slice-key.ts', constant: 'REFINE_CACHE_VERSION', },
  { path: 'translate-lane/translate-slice-key.ts', constant: 'TRANSLATE_SLICE_CACHE_VERSION', },
  { path: 'artifact/consolidate-key.ts', constant: 'CONSOLIDATE_CACHE_VERSION', },
  { path: 'lane-contest/lane-contest-key.ts', constant: 'LANE_CONTEST_CACHE_VERSION', },
];

/**
 Words in a module specifier that name a sheet or a wire.
 */
const SHEET_WORDS: readonly string[] = ['wire', 'sheet', 'prompt', 'candidate-select',];

/**
 The module specifiers a source text imports or re-exports from, read by line.

 @param text - source file

 @returns Text between the quotes after each ` from `

 @example
 ```ts
 const specifiers = specifiersOf({ text: "import { a, } from './a.ts';", },); // ['./a.ts']
 ```
 */
function specifiersOf({ text, }: { readonly text: string; },): readonly string[] {
  return text.split('\n',)
    .filter(function isFromLine(line,): boolean {
      return line.includes(' from \'',);
    },)
    .map(function specifierOf(line,): string {
      /**
       The text after the opening quote.
       */
      const after = line.slice(line.indexOf(' from \'',) + ' from \''.length,);
      return after.slice(0, after.indexOf('\'',),);
    },);
}

/**
 The specifiers of a source text that name a sheet or a wire.

 @param text - source file

 @returns Specifiers holding one of the sheet words

 @example
 ```ts
 const sheets = sheetImportsOf({ text: "import { a, } from './cat-wire.ts';", },); // ['./cat-wire.ts']
 ```
 */
function sheetImportsOf({ text, }: { readonly text: string; },): readonly string[] {
  return specifiersOf({ text, },).filter(function namesSheet(specifier,): boolean {
    return SHEET_WORDS.some(function inSpecifier(word,): boolean {
      return specifier.includes(word,);
    },);
  },);
}

await describe({
  name: 'the imports of the five prompt-keyed cache keys',
  concurrency: 1,
  children: [
    ...KEY_FILES.map(function caseOf({ path, constant, },) {
      return it({
        name: `READ NO SHEET OR WIRE MODULE in ${path}, which hashes ${constant}: a change to the sheets' wording `
          + `moves no key, so the number is moved by hand (ledger M28)`,
        fn: async () => {
          /**
           The key file's text.
           */
          const text = readFileSync(
            sourcePathOf({ path, },),
            'utf8',
          );
          expect(sheetImportsOf({ text, },),).toEqual([],);
          // THE VERSION IS KEY INPUT, which is what makes a moved number retire a record: the constant is read
          // inside the file.
          expect(text.includes(`${constant},`,),).toBe(true,);
        },
      },);
    },),

    it({
      name: 'FINDS A KEY FILE THAT IMPORTS A WIRE (the positive control): the filter can tell a sheet import from a '
        + 'plain one',
      fn: async () => {
        expect(sheetImportsOf({
          text: 'import { a, } from \'./candidate-select-wire.ts\';\nimport { b, } from \'./b.ts\';',
        },),).toEqual([
          './candidate-select-wire.ts',
        ],);
      },
    },),
  ],
},);
