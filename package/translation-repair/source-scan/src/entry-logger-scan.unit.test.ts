/**
 Guards against an entry logger that reads nothing from the log context (ledger B963), by reading the source: every
 logger the pass builds for an entry is slice-tagged, so a line it writes inside an attempt names that attempt.

 A logger built as a plain `tagged({ tag: entryId })` carries the entry and never the attempt, lane or slice, which
 the context holds. The docs say the lines an entry's loggers write inside its attempt name it, and the entry loggers
 of the pass printed none of the attempt until each was wrapped by `sliceTagged`. Wrapped loggers do not repeat the
 tags where a lane slice-tags the logger again (`log-context.unit.test.ts`). Test files and test fixtures log outside
 any attempt and are not read.

 WHAT IT CANNOT CATCH. A logger whose tag is built from the entry in another spelling, such as a template literal, is
 not found; the scan reads the two spellings the pass uses, `entryId` and `entry.id`.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import type { SourceText, } from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 How an entry logger begins, in a text whose lines are trimmed and joined by a space, in the two spellings the pass
 uses for the entry.
 */
const ENTRY_LOGGER_SHAPES = [
  'tagged({ tag: entryId',
  'tagged({ tag: entry.id',
];

/**
 What must stand directly before an entry logger so the logger reads the context.
 */
const CONTEXT_WRAPPER = 'sliceTagged({ l: ';

/**
 Entry loggers in one file's text that no `sliceTagged` wraps.

 @param text - file contents

 @returns How many entry loggers stand without the wrapper

 @example
 ```ts
 unwrappedEntryLoggersIn({ text: "const l = tagged({ tag: entryId, },);", },); // 1
 ```
 */
function unwrappedEntryLoggersIn({ text, }: { readonly text: string; },): number {
  /**
   The text with each line trimmed and the lines joined by a space, so a wrapper split over lines still stands directly
   before the logger it wraps.
   */
  const joined = text
    .split('\n',)
    .map(function trimmed(line,): string {
      return line.trim();
    },)
    .join(' ',);
  return ENTRY_LOGGER_SHAPES.reduce(function counted(total, shape,): number {
    /**
     The text before each logger of this shape, which ends where the logger begins.
     */
    const textsBefore = joined.split(shape,);
    textsBefore.pop();
    return total + textsBefore.filter(function lacksWrapper(before,): boolean {
      return !before.endsWith(CONTEXT_WRAPPER,);
    },).length;
  }, 0,);
}

await describe({
  name: 'entry loggers',
  children: [
    it({
      name: 'SEES a plain entry logger and a wrapped one in a line of text, the positive control for the "BUILDS every '
        + 'entry logger on the log context" scan',
      fn: async () => {
        expect(unwrappedEntryLoggersIn({ text: 'const l = tagged({ tag: entryId, },);\n', },),).toBe(1,);
        expect(unwrappedEntryLoggersIn({ text: 'l: tagged({ tag: entry.id, },),\n', },),).toBe(1,);
        expect(
          unwrappedEntryLoggersIn({ text: 'const l = sliceTagged({ l: tagged({ tag: entryId, },), },);\n', },),
        ).toBe(0,);
        expect(
          unwrappedEntryLoggersIn({
            text: 'const l = sliceTagged({\n  l: tagged({\n    tag: entry.id,\n  },),\n},);\n',
          },),
        ).toBe(0,);
      },
    },),
    it({
      name: 'SEES a plain entry logger in each of the spellings the first scan missed, planted one at a time: a '
        + 'template literal, an aliased import, another key order, `pair.id`, a const alias and tight spacing, '
        + 'since the scan reads what the call does and not how it is written (ledger B963)',
      fn: async () => {
        /**
         The spellings, each a file that builds one plain entry logger.
         */
        const planted = [
          'const l = tagged({ tag: `${entryId}`, },);\n',
          'import { tagged as makeLogger, } from \'@monochromatic-dev/module-logger/ts\';\n'
          + 'const l = makeLogger({ tag: entryId, },);\n',
          'const l = tagged({ l: parent, tag: entryId, },);\n',
          'const l = tagged({ tag: pair.id, },);\n',
          'const make = tagged;\nconst l = make({ tag: entryId, },);\n',
          'const l = tagged({tag:entryId,},);\n',
        ];
        expect(planted.map(function found(text,): number {
          return unwrappedEntryLoggersIn({ text, },);
        },),).toStrictEqual(planted.map(function one(): number {
          return 1;
        },),);
      },
    },),
    it({
      name: 'LEAVES ALONE a logger that is wrapped by `sliceTagged` in any spelling, and a logger whose tag is no '
        + 'entry: a string, a function name or a slice index (ledger B963)',
      fn: async () => {
        /**
         Files that build loggers the scan must not find.
         */
        const harmless = [
          'const l = sliceTagged({ l: tagged({ tag: `${entryId}`, },), },);\n',
          'import { sliceTagged as sliced, } from \'@monochromatic-dev/translation-repair-typography/ts\';\n'
          + 'const l = sliced({ l: tagged({ l: parent, tag: pair.id, },), },);\n',
          'const make = tagged;\nconst l = sliceTagged({ l: make({ tag: entryId, },), },);\n',
          'const l = tagged({ tag: \'pass-entry\', },);\n',
          'const l = tagged({ tag: runCorpusPassOver.name, },);\n',
          'const l = tagged({ l: dl, tag: `slice ${String(row.sliceIndex,)}`, },);\n',
        ];
        expect(harmless.map(function found(text,): number {
          return unwrappedEntryLoggersIn({ text, },);
        },),).toStrictEqual(harmless.map(function none(): number {
          return 0;
        },),);
      },
    },),
    it({
      name: 'BUILDS every entry logger on the log context, so its lines name the attempt inside one (ledger B963)',
      fn: async () => {
        /**
         Files the scan reads: every `.ts` under the `src` of every family package but tests and fixtures.
         */
        const files = (await readPackageSource()).filter(function isProduction(file: SourceText,): boolean {
          return (!file.isTest) && (!file.path.endsWith('.d.ts',));
        },);

        // The scan read real loggers, so an empty list of findings is not a scan that read nothing.
        expect(files.filter(function wrapsOne({ text, },): boolean {
          return text.includes(CONTEXT_WRAPPER,);
        },).length,).toBeGreaterThan(0,);

        expectNoFindings({
          findings: files
            .map(function scanOne({ path, text, },): string {
              return `${path}: ${String(unwrappedEntryLoggersIn({ text, },),)}`;
            },)
            .filter(function hasPlain(finding,): boolean {
              return !finding.endsWith(': 0',);
            },),
        },);
      },
    },),
  ],
},);
