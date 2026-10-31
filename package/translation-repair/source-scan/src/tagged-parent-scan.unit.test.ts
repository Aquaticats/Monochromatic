/**
 Guards against a logger the pass builds that writes to the default logger instead of the run's records (ledger B1110),
 by reading the syntax tree of the family's source: every call of `tagged` is given the logger it wraps.

 `tagged` defaults its `l` to the default logger of logger-core. A pass keeps its records in its run folder by
 installing a run log that the context roots write through (`typography/src/run-log.ts`), and a logger built as
 `tagged({ tag: entryId })` never reaches it: the `round:` lines, the `SLICE-COST` lines, the consolidate driver's lines
 and every entry-level warning and error of a real run went to the console alone, and the reports that read the
 folder read nothing.

 WHAT IT READS. Every call of `tagged` in the source of every family package, under whatever name the file binds it,
 whose options carry no `l` and no spread. Test files and test fixtures log outside any pass and are not read.

 WHAT IT LISTS. The runners that are no pass and keep no run folder (the bench, probe and coverage census commands),
 each file with its count of calls, because they are launched with an operator's redirect and write through the default
 logger until the round that gives them a run log (ledger B1099). A listed file whose count moved fails the scan, so the
 list shrinks as those runners move and cannot grow unseen.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import type { SourceText, } from './source-scan.test-fixture.ts';
import { taggedWithoutParentIn, } from './tagged-parent-finding.test-fixture.ts';
import { expectRecordAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Files of runners that are no pass, with the calls of `tagged` each makes without a parent: the editor fidelity bench
 and the probes launched with an operator's redirect, whose run folder is the later round of ledger B1099.
 */
const RUNNERS_WITHOUT_RUN_LOG: Readonly<Record<string, number>> = {
  'bench/corpus-run/checker-sensitivity-run.ts': 1,
  'bench/corpus-run/displacement-probe.ts': 1,
  'bench/corpus-run/editor-calibrate-lane.ts': 1,
  'bench/corpus-run/editor-width-probe-run.ts': 1,
  'bench/corpus-run/producer-calibrate-round.ts': 1,
  'bench/corpus-run/roster-bench-round.ts': 1,
  'bench/corpus-run/translate-probe.ts': 1,
  'coverage-census/corpus-run/coverage-control-probe.ts': 1,
  'coverage-census/corpus-run/coverage-probe.ts': 1,
  'probe/corpus-run/audit-sensitivity-run.ts': 1,
  'probe/corpus-run/editor-fidelity-pool-draw.ts': 2,
  'probe/corpus-run/editor-fidelity-probe.ts': 1,
  'probe/corpus-run/judge-fidelity-probe.ts': 1,
  'probe/corpus-run/probe-relabel-pair.ts': 1,
  'probe/corpus-run/probe-sensitivity-run.ts': 1,
  'probe/corpus-run/probe-verify-collect.ts': 1,
  'probe/corpus-run/rendering-audit-settled-buy.ts': 2,
};

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
  name: 'tagged loggers',
  children: [
    it({
      name: 'SEES a call of `tagged` given no parent logger in each spelling, the positive control for the "GIVES every '
        + 'call of tagged a parent" scan: an entry tag, a function name, an aliased import, a const alias, two calls and '
        + 'tight spacing',
      fn: async () => {
        /**
         Spellings, each a file with the number of calls the scan must find in it.
         */
        const spellings: readonly (readonly [string, number])[] = [
          [
            'const l = tagged({ tag: entryId, },);\n',
            1,
          ],
          [
            'const l = sliceTagged({ l: tagged({ tag: entry.id, },), },);\n',
            1,
          ],
          [
            'const l = tagged({ tag: runCorpusPassOver.name, },);\n',
            1,
          ],
          [
            'import { tagged as makeLogger, } from \'@monochromatic-dev/logger-core/ts\';\n'
              + 'const l = makeLogger({ tag: \'cat\', },);\n',
            1,
          ],
          [
            'const make = tagged;\nconst l = make({ tag: \'cat\', },);\n',
            1,
          ],
          [
            'const a = tagged({ tag: \'a\', },);\nconst b = tagged({ tag: \'b\', },);\n',
            2,
          ],
          [
            'const l = tagged({tag:\'cat\',},);\n',
            1,
          ],
          [
            'const l = tagged({\n  tag: \'cat\',\n},);\n',
            1,
          ],
        ];
        expect(spellings.map(function found([text,],): number {
          return taggedWithoutParentIn({ file: planted({ text, },), },)
            .length;
        },),).toStrictEqual(spellings.map(function listed([, count,],): number {
          return count;
        },),);
        expect(taggedWithoutParentIn({
          file: planted({ text: 'const a = tagged({ tag: \'a\', },);\n\nconst b = tagged({ tag: \'b\', },);\n', },),
        },),).toStrictEqual([
          1,
          3,
        ],);
      },
    },),
    it({
      name: 'LEAVES ALONE a call given its parent in any spelling: `l: runLogger`, `l` shorthand, another key order, a '
        + 'spread that may carry one, and a call of some other function',
      fn: async () => {
        /**
         Files that build loggers the scan must not find.
         */
        const harmless = [
          'const l = tagged({ tag: \'cat\', l: runLogger, },);\n',
          'const rl = tagged({ l, tag: fn.name, },);\n',
          `const rl = tagged({ l: parent, tag: \`\${entryId}\`, },);\n`,
          'const rl = tagged({ ...options, },);\n',
          'const rl = sliceTagged({ l: tagged({ tag: entryId, l: runLogger, },), },);\n',
          'const rl = taggedTemplate({ tag: \'cat\', },);\n',
          'const rl = other.tagged({ tag: \'cat\', },);\n',
        ];
        expect(harmless.map(function found(text,): number {
          return taggedWithoutParentIn({ file: planted({ text, },), },)
            .length;
        },),).toStrictEqual(harmless.map(function none(): number {
          return 0;
        },),);
      },
    },),
    it({
      name: 'GIVES every call of tagged a parent, so the lines of a pass reach its run folder, but for the runners of '
        + 'ledger B1099 that keep no run folder (ledger B1110)',
      fn: async () => {
        /**
         Every file of the family's source but tests and fixtures.
         */
        const files = (await readPackageSource()).filter(function isSource(file: SourceText,): boolean {
          return (!file.isTest) && (!file.path.endsWith('.d.ts',));
        },);

        // The scan read real calls, so an empty record is not a scan that read nothing.
        expect(files.filter(function callsTagged({ text, },): boolean {
          return text.includes('tagged({',);
        },).length,).toBeGreaterThan(100,);

        /**
         Calls without a parent per file, files without one left out.
         */
        const found: Record<string, number> = {};
        for (const file of files) {
          /**
           Lines of the calls in this file.
           */
          const lines = taggedWithoutParentIn({ file, },);
          if (lines.length > 0)
            found[file.path] = lines.length;
        }
        expectRecordAsListed({
          found,
          listed: RUNNERS_WITHOUT_RUN_LOG,
        },);
      },
    },),
  ],
},);
