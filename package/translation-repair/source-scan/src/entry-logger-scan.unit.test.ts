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
    return total + joined
      .split(shape,)
      .slice(
        0,
        -1,
      )
      .filter(function lacksWrapper(before,): boolean {
        return !before.endsWith(CONTEXT_WRAPPER,);
      },).length;
  }, 0,);
}

/**
 Source files the scan reads: every `.ts` under the `src` of every family package but tests and fixtures.

 @returns Files, each located by its package directory and path under `src`

 @example
 ```ts
 const files = await sourceFiles();
 ```
 */
async function sourceFiles(): Promise<readonly SourceText[]> {
  return (await readPackageSource())
    .filter(function isSource({ path, isTest, },): boolean {
      return (!isTest) && (!path.endsWith('.d.ts',));
    },);
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
      name: 'BUILDS every entry logger on the log context, so its lines name the attempt inside one (ledger B963)',
      fn: async () => {
        /**
         Files the scan read.
         */
        const files = await sourceFiles();

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
