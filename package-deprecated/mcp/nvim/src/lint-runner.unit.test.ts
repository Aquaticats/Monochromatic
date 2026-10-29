import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import { parseOxlintOutput, } from './oxlint-parse.ts';
import type { OxlintJsonOutput, } from './oxlint-types.ts';

//region parseOxlintOutput: converts oxlint JSON to grouped Diagnostic maps

await describe({
  name: parseOxlintOutput.name,
  children: [
    it({
      name: 'parses a single diagnostic with all fields',
      fn: async () => {
        const output: OxlintJsonOutput = {
          diagnostics: [
            {
              message: 'Missing TSDoc comment.',
              code: 'tsdoc(require-tsdoc)',
              severity: 'error',
              causes: [],
              filename: 'src/index.ts',
              labels: [{ span: { offset: 100, length: 20, line: 5, column: 3, }, },],
              related: [],
            },
          ],
          number_of_files: 1,
          number_of_rules: 300,
          threads_count: 8,
          start_time: 0.05,
        };

        const result = parseOxlintOutput({ output, cwd: '/home/user/project', },);
        const diags = result.get('/home/user/project/src/index.ts',);

        expect(diags,).toEqual([
          {
            severity: 'ERROR',
            lnum: 5,
            col: 3,
            end_lnum: 5,
            end_col: 3,
            message: 'Missing TSDoc comment.',
            source: 'oxlint',
            code: 'tsdoc(require-tsdoc)',
          },
        ],);
      },
    },),
    it({
      name: 'maps warning severity to WARN',
      fn: async () => {
        const output: OxlintJsonOutput = {
          diagnostics: [
            {
              message: 'Unused variable.',
              code: 'no-unused-vars',
              severity: 'warning',
              causes: [],
              filename: 'src/foo.ts',
              labels: [{ span: { offset: 0, length: 10, line: 1, column: 1, }, },],
              related: [],
            },
          ],
          number_of_files: 1,
          number_of_rules: 300,
          threads_count: 8,
          start_time: 0.05,
        };

        const result = parseOxlintOutput({ output, cwd: '/tmp', },);
        const diags = result.get('/tmp/src/foo.ts',);
        expect(diags?.[0]?.severity,).toBe('WARN',);
      },
    },),
    it({
      name: 'handles unknown severity gracefully',
      fn: async () => {
        const output: OxlintJsonOutput = {
          diagnostics: [
            {
              message: 'Something.',
              code: 'rule',
              severity: 'info',
              causes: [],
              filename: 'a.ts',
              labels: [{ span: { offset: 0, length: 5, line: 1, column: 1, }, },],
              related: [],
            },
          ],
          number_of_files: 1,
          number_of_rules: 1,
          threads_count: 1,
          start_time: 0,
        };

        const result = parseOxlintOutput({ output, cwd: '/tmp', },);
        expect(result.get('/tmp/a.ts',)?.[0]?.severity,).toBe('UNKNOWN(info)',);
      },
    },),
    it({
      name: 'groups multiple diagnostics by file path',
      fn: async () => {
        const output: OxlintJsonOutput = {
          diagnostics: [
            {
              message: 'First.',
              code: 'rule-a',
              severity: 'error',
              causes: [],
              filename: 'src/a.ts',
              labels: [{ span: { offset: 0, length: 5, line: 1, column: 1, }, },],
              related: [],
            },
            {
              message: 'Second.',
              code: 'rule-b',
              severity: 'warning',
              causes: [],
              filename: 'src/a.ts',
              labels: [{ span: { offset: 50, length: 10, line: 10, column: 5, }, },],
              related: [],
            },
            {
              message: 'Third.',
              code: 'rule-c',
              severity: 'error',
              causes: [],
              filename: 'src/b.ts',
              labels: [{ span: { offset: 0, length: 5, line: 2, column: 3, }, },],
              related: [],
            },
          ],
          number_of_files: 2,
          number_of_rules: 300,
          threads_count: 8,
          start_time: 0.05,
        };

        const result = parseOxlintOutput({ output, cwd: '/project', },);
        expect(result.get('/project/src/a.ts',),).toHaveLength(2,);
        expect(result.get('/project/src/b.ts',),).toHaveLength(1,);
      },
    },),
    it({
      name: 'skips diagnostics with no labels',
      fn: async () => {
        const output: OxlintJsonOutput = {
          diagnostics: [
            {
              message: 'No location.',
              code: 'rule',
              severity: 'error',
              causes: [],
              filename: 'src/a.ts',
              labels: [],
              related: [],
            },
          ],
          number_of_files: 1,
          number_of_rules: 1,
          threads_count: 1,
          start_time: 0,
        };

        const result = parseOxlintOutput({ output, cwd: '/tmp', },);
        expect(result.size,).toBe(0,);
      },
    },),
    it({
      name: 'returns empty map for zero diagnostics',
      fn: async () => {
        const output: OxlintJsonOutput = {
          diagnostics: [],
          number_of_files: 1,
          number_of_rules: 300,
          threads_count: 8,
          start_time: 0.05,
        };

        const result = parseOxlintOutput({ output, cwd: '/tmp', },);
        expect(result.size,).toBe(0,);
      },
    },),
    it({
      name: 'resolves relative filenames against cwd',
      fn: async () => {
        const output: OxlintJsonOutput = {
          diagnostics: [
            {
              message: 'Error.',
              code: 'rule',
              severity: 'error',
              causes: [],
              filename: '../other/file.ts',
              labels: [{ span: { offset: 0, length: 5, line: 1, column: 1, }, },],
              related: [],
            },
          ],
          number_of_files: 1,
          number_of_rules: 1,
          threads_count: 1,
          start_time: 0,
        };

        const result = parseOxlintOutput({ output, cwd: '/project/packages/foo', },);
        expect(result.has('/project/packages/other/file.ts',),).toBe(true,);
        expect(result.has('/project/packages/foo/../other/file.ts',),).toBe(false,);
      },
    },),
    it({
      name: 'appends help text to message when present',
      fn: async () => {
        const output: OxlintJsonOutput = {
          diagnostics: [
            {
              message: 'Empty exports do nothing in module files',
              code: 'typescript-eslint(no-useless-empty-export)',
              severity: 'error',
              causes: [],
              filename: 'src/a.ts',
              labels: [{ span: { offset: 0, length: 9, line: 12, column: 1, }, },],
              related: [],
              help: 'Remove this empty export.',
            },
          ],
          number_of_files: 1,
          number_of_rules: 300,
          threads_count: 8,
          start_time: 0.05,
        };

        const result = parseOxlintOutput({ output, cwd: '/tmp', },);
        expect(result.get('/tmp/src/a.ts',)?.[0]?.message,).toBe(
          'Empty exports do nothing in module files (help: Remove this empty export.)',
        );
      },
    },),
    it({
      name: 'does not append help suffix when help is absent',
      fn: async () => {
        const output: OxlintJsonOutput = {
          diagnostics: [
            {
              message: 'Missing TSDoc comment.',
              code: 'tsdoc(require-tsdoc)',
              severity: 'error',
              causes: [],
              filename: 'src/a.ts',
              labels: [{ span: { offset: 0, length: 5, line: 1, column: 1, }, },],
              related: [],
            },
          ],
          number_of_files: 1,
          number_of_rules: 300,
          threads_count: 8,
          start_time: 0.05,
        };

        const result = parseOxlintOutput({ output, cwd: '/tmp', },);
        expect(result.get('/tmp/src/a.ts',)?.[0]?.message,).toBe(
          'Missing TSDoc comment.',
        );
      },
    },),
    it({
      name: 'does not append help suffix when help is empty string',
      fn: async () => {
        const output: OxlintJsonOutput = {
          diagnostics: [
            {
              message: 'Some error.',
              code: 'rule',
              severity: 'error',
              causes: [],
              filename: 'src/a.ts',
              labels: [{ span: { offset: 0, length: 5, line: 1, column: 1, }, },],
              related: [],
              help: '',
            },
          ],
          number_of_files: 1,
          number_of_rules: 1,
          threads_count: 1,
          start_time: 0,
        };

        const result = parseOxlintOutput({ output, cwd: '/tmp', },);
        expect(result.get('/tmp/src/a.ts',)?.[0]?.message,).toBe('Some error.',);
      },
    },),
    it({
      name: 'always sets source to oxlint',
      fn: async () => {
        const output: OxlintJsonOutput = {
          diagnostics: [
            {
              message: 'Error.',
              code: 'typescript(strict-boolean-expressions)',
              severity: 'error',
              causes: [],
              filename: 'x.ts',
              labels: [{ span: { offset: 0, length: 5, line: 1, column: 1, }, },],
              related: [],
            },
          ],
          number_of_files: 1,
          number_of_rules: 1,
          threads_count: 1,
          start_time: 0,
        };

        const result = parseOxlintOutput({ output, cwd: '/tmp', },);
        expect(result.get('/tmp/x.ts',)?.[0]?.source,).toBe('oxlint',);
      },
    },),
  ],
},);

//endregion parseOxlintOutput
