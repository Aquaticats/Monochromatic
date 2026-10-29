/**
 Example tests for `./mutation-mutant.ts`: report parsing, Stryker position
 arithmetic, mutant splicing, selection, and sweep verdicts.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  applyMutant,
  isPinningFile,
  MutationReportError,
  offsetOf,
  parseMutationReport,
  selectMutants,
  sweepVerdict,
} from './mutation-mutant.ts';

/**
 Two-line source the report below mutates.
 */
const SOURCE = 'const a = b && c;\nreturn a;\n';

/**
 Minimal report in Stryker's shape: one survivor with a replacement, one
 killed mutant without one (a deletion).
 */
const REPORT_TEXT = JSON.stringify({
  files: {
    'src/x.ts': {
      mutants: [
        {
          id: '7',
          location: {
            end: {
              column: 17,
              line: 1,
            },
            start: {
              column: 11,
              line: 1,
            },
          },
          mutatorName: 'LogicalOperator',
          replacement: 'b || c',
          status: 'Survived',
        },
        {
          id: '8',
          location: {
            end: {
              column: 10,
              line: 2,
            },
            start: {
              column: 1,
              line: 2,
            },
          },
          mutatorName: 'BlockStatement',
          status: 'Killed',
        },
      ],
      source: SOURCE,
    },
  },
},);

await describe({
  name: 'mutation report helpers',
  children: [
    it({
      name: 'parseMutationReport keeps each mutant and reads a missing replacement as deletion',
      fn: async () => {
        /**
         Parsed report.
         */
        const report = parseMutationReport(REPORT_TEXT,);
        expect(report.files['src/x.ts']?.mutants.map(function idOf(mutant,) {
          return [mutant.id, mutant.replacement,];
        },),).toEqual([['7', 'b || c',], ['8', '',],],);
      },
    },),
    it({
      name: 'parseMutationReport rejects a report without files and a non-positive position',
      fn: async () => {
        expect(function noFiles() {
          return parseMutationReport('{}',);
        },).toThrow(MutationReportError,);
        expect(function zeroColumn() {
          return parseMutationReport(REPORT_TEXT.replace('"column":11', '"column":0',),);
        },).toThrow(MutationReportError,);
      },
    },),
    it({
      name: 'offsetOf maps 1-based positions and rejects a line past the end',
      fn: async () => {
        expect(offsetOf({ position: { column: 1, line: 1, }, source: SOURCE, },),).toBe(0,);
        expect(offsetOf({ position: { column: 2, line: 2, }, source: SOURCE, },),).toBe(19,);
        expect(function pastEnd() {
          return offsetOf({ position: { column: 1, line: 9, }, source: SOURCE, },);
        },).toThrow(MutationReportError,);
      },
    },),
    it({
      name: 'applyMutant splices the replacement and deletes a span without one',
      fn: async () => {
        /**
         Mutants in report order.
         */
        const selected = selectMutants({ ids: [], report: parseMutationReport(REPORT_TEXT,), statuses: ['Survived', 'Killed',], },);
        expect(selected.map(function apply(entry,) {
          return applyMutant(entry,);
        },),).toEqual([
          'const a = b || c;\nreturn a;\n',
          'const a = b && c;\n\n',
        ],);
      },
    },),
    it({
      name: 'selectMutants filters by status and by id',
      fn: async () => {
        /**
         Parsed report.
         */
        const report = parseMutationReport(REPORT_TEXT,);
        expect(selectMutants({ ids: [], report, statuses: ['Survived',], },)
          .map(function idOf(selected,) {
            return selected.mutant.id;
          },),).toEqual(['7',],);
        expect(selectMutants({ ids: ['8',], report, statuses: ['Survived', 'Killed',], },)
          .map(function idOf(selected,) {
            return selected.mutant.id;
          },),).toEqual(['8',],);
      },
    },),
    it({
      name: 'sweepVerdict separates specifying files from pinning files',
      fn: async () => {
        expect(isPinningFile('src/known-defect-alias.unit.test.ts',),).toBe(true,);
        expect(isPinningFile('src/scale-complexity.local.unit.test.ts',),).toBe(true,);
        expect(isPinningFile('src/model.unit.test.ts',),).toBe(false,);
        expect(sweepVerdict(['src/known-defect.unit.test.ts', 'src/model.unit.test.ts',],),).toBe('detected',);
        expect(sweepVerdict(['src/known-defect.unit.test.ts',],),).toBe('pinningOnly',);
        expect(sweepVerdict([],),).toBe('survived',);
      },
    },),
  ],
},);
