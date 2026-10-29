/**
 Guards against an entry logger that reads nothing from the log context (ledger B963), by reading the syntax tree of the
 pass's source: every logger the pass builds for an entry is slice-tagged, so a line it writes inside an attempt names
 that attempt.

 A logger built as a plain `tagged({ tag: entryId })` carries the entry and never the attempt, lane or slice, which
 the context holds. The docs say the lines an entry's loggers write inside its attempt name it, and the entry loggers
 of the pass printed none of the attempt until each was wrapped by `sliceTagged`. Wrapped loggers do not repeat the
 tags where a lane slice-tags the logger again (`log-context.unit.test.ts`). Test files and test fixtures log outside
 any attempt and are not read.

 WHAT IT READS. Every call of `tagged` in the source of the packages that run the pass (`corpus-pass`, `slice-cache`),
 under whatever name the file binds it, whose `tag` names an entry in any spelling (`entry-logger-finding.test-fixture.ts`
 says which), unless it is the `l` of a call of `sliceTagged`.

 WHAT IT CANNOT CATCH. A tag built from the entry by a name the scan does not know (not `entryId`, `.id` or `.entryId`),
 and a function that receives an `l: Logger` from its caller and never calls `tagged`: only the behavioural cases of the
 whole run, the seams and the hooks (`pass-entry.unit.test.ts` and its neighbours) see those.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  SLICE_TAGGED,
  unwrappedEntryLoggersIn,
} from './entry-logger-finding.test-fixture.ts';
import { readPackageSource, } from './family-source.test-fixture.ts';
import type { SourceText, } from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Packages that run the pass, whose source builds the loggers of an entry.
 */
const PASS_PACKAGES = [
  'corpus-pass/',
  'slice-cache/',
];

/**
 A file of the given text, as the scans read one.

 @param text - file contents

 @returns File, named for the pass

 @example
 ```ts
 const file = planted({ text: 'const l = tagged({ tag: entryId, },);', },);
 ```
 */
function planted({ text, }: { readonly text: string; },): SourceText {
  return {
    path: 'corpus-pass/corpus-run/planted.ts',
    text,
    isTest: false,
  };
}

await describe({
  name: 'entry loggers',
  children: [
    it({
      name: 'SEES a plain entry logger and a wrapped one in a line of text, the positive control for the "BUILDS every '
        + 'entry logger on the log context" scan, in the two spellings the pass uses',
      fn: async () => {
        expect(unwrappedEntryLoggersIn({ file: planted({ text: 'const l = tagged({ tag: entryId, },);\n', },), },),)
          .toBe(1,);
        expect(
          unwrappedEntryLoggersIn({ file: planted({ text: 'const parts = { l: tagged({ tag: entry.id, },), };\n', },), },),
        ).toBe(1,);
        expect(
          unwrappedEntryLoggersIn({
            file: planted({ text: 'const l = sliceTagged({ l: tagged({ tag: entryId, },), },);\n', },),
          },),
        ).toBe(0,);
        expect(
          unwrappedEntryLoggersIn({
            file: planted({ text: 'const l = sliceTagged({\n  l: tagged({\n    tag: entry.id,\n  },),\n},);\n', },),
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
        const spellings = [
          `const l = tagged({ tag: \`\${entryId}\`, },);\n`,
          'import { tagged as makeLogger, } from \'@monochromatic-dev/logger-core/ts\';\n'
            + 'const l = makeLogger({ tag: entryId, },);\n',
          'const l = tagged({ l: parent, tag: entryId, },);\n',
          'const l = tagged({ tag: pair.id, },);\n',
          'const make = tagged;\nconst l = make({ tag: entryId, },);\n',
          'const l = tagged({tag:entryId,},);\n',
          'const id = entry.id;\nconst l = tagged({ tag: id, },);\n',
          'const l = tagged({ tag: String(entry.id,), },);\n',
          // Not wrapped: the call sits under another key of the wrapper's options, or inside another call.
          'const l = sliceTagged({ other: tagged({ tag: entryId, },), },);\n',
          'const l = sliceTagged({ l: decorate(tagged({ tag: entryId, },),), },);\n',
        ];
        expect(spellings.map(function found(text,): number {
          return unwrappedEntryLoggersIn({ file: planted({ text, },), },);
        },),).toStrictEqual(spellings.map(function one(): number {
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
          `const l = sliceTagged({ l: tagged({ tag: \`\${entryId}\`, },), },);\n`,
          'import { sliceTagged as sliced, } from \'@monochromatic-dev/translation-repair-typography/ts\';\n'
            + 'const l = sliced({ l: tagged({ l: parent, tag: pair.id, },), },);\n',
          'const make = tagged;\nconst l = sliceTagged({ l: make({ tag: entryId, },), },);\n',
          'const l = sliceTagged({ l: (tagged({ tag: entryId, },) as Logger), },);\n',
          'const l = tagged({ tag: \'pass-entry\', },);\n',
          'const l = tagged({ tag: runCorpusPassOver.name, },);\n',
          `const l = tagged({ l: dl, tag: \`slice \${String(row.sliceIndex,)}\`, },);\n`,
          'const tag = row.sliceIndex;\nconst l = tagged({ tag, },);\n',
        ];
        expect(harmless.map(function found(text,): number {
          return unwrappedEntryLoggersIn({ file: planted({ text, },), },);
        },),).toStrictEqual(harmless.map(function none(): number {
          return 0;
        },),);
      },
    },),
    it({
      name: 'BUILDS every entry logger on the log context, so its lines name the attempt inside one (ledger B963)',
      fn: async () => {
        /**
         Files the scan reads: every `.ts` under the `src` of the packages that run the pass but tests and fixtures.
         */
        const files = (await readPackageSource()).filter(function isPassSource(file: SourceText,): boolean {
          return (!file.isTest)
            && (!file.path.endsWith('.d.ts',))
            && PASS_PACKAGES.some(function isPassPackage(prefix,): boolean {
              return file.path.startsWith(prefix,);
            },);
        },);

        // The scan read real loggers, so an empty list of findings is not a scan that read nothing.
        expect(files.filter(function wrapsOne({ text, },): boolean {
          return text.includes(`${SLICE_TAGGED}(`,);
        },).length,).toBeGreaterThan(0,);

        expectNoFindings({
          findings: files
            .map(function scanOne(file,): string {
              return `${file.path}: ${String(unwrappedEntryLoggersIn({ file, },),)}`;
            },)
            .filter(function hasPlain(finding,): boolean {
              return !finding.endsWith(': 0',);
            },),
        },);
      },
    },),
  ],
},);
