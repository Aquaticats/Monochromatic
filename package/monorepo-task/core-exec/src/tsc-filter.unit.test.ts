import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildTscArgs,
  filterTscOutput,
  isContinuationLine,
  isDiagnosticLine,
  isNodeModulesDiagnostic,
} from '../dist/final/node/testing.mjs';

/** Iteration count for long-run equivalence cases; large enough to exercise the linear scan, fast to compare. */
const LONG_RUN = 100_000;
/** Occurrence count for the scan-walk stress case; many `): error TS` tokens that never form a valid diagnostic. */
const SCAN_OCCURRENCES = 50_000;

//region Unit tests for pure filtering functions

await describe({
  name: '',
  children: [
    describe({
      name: buildTscArgs.name,
      children: [
        it({
          name: 'defaults to build when no arguments are supplied',
          fn: async () => {
            expect(buildTscArgs({
              cliArgs: [],
            },),).toEqual(['--build',],);
          },
        },),
        it({
          name: 'preserves direct package-local arguments when env is absent',
          fn: async () => {
            expect(buildTscArgs({
              cliArgs: ['--build', '--noEmit',],
            },),).toEqual(['--build', '--noEmit',],);
          },
        },),
        it({
          name: 'injects singleThreaded after build for root fanout env',
          fn: async () => {
            expect(buildTscArgs({
              cliArgs: ['--build',],
              singleThreadedEnv: '1',
            },),).toEqual(['--build', '--singleThreaded',],);
          },
        },),
        it({
          name: 'injects singleThreaded before non-build arguments',
          fn: async () => {
            expect(buildTscArgs({
              cliArgs: ['--noEmit', '--project', 'tsconfig.json',],
              singleThreadedEnv: '1',
            },),).toEqual([
              '--singleThreaded',
              '--noEmit',
              '--project',
              'tsconfig.json',
            ],);
          },
        },),
        it({
          name: 'does not duplicate an explicit singleThreaded flag',
          fn: async () => {
            expect(buildTscArgs({
              cliArgs: ['--singleThreaded', '--build',],
              singleThreadedEnv: '1',
            },),).toEqual(['--singleThreaded', '--build',],);
          },
        },),
        it({
          name: 'treats false env values as opt out',
          fn: async () => {
            expect(buildTscArgs({
              cliArgs: ['--build',],
              singleThreadedEnv: 'false',
            },),).toEqual(['--build',],);
          },
        },),
      ],
    },),
    describe({
      name: isDiagnosticLine.name,
      children: [
        it({
          name: 'matches standard tsc diagnostic format',
          fn: async () => {
            expect(isDiagnosticLine(
              "src/index.ts(1,15): error TS2304: Cannot find name 'foo'.",
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'matches node_modules diagnostic format',
          fn: async () => {
            expect(isDiagnosticLine(
              "node_modules/.bun/@jsr+zod__zod@4.3.6/src/v4/core/schemas.ts(2088,19): error TS2532: Object is possibly 'undefined'.",
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'rejects continuation lines',
          fn: async () => {
            expect(isDiagnosticLine(
              "  Type 'string' is not assignable to type 'number'.",
            ),)
              .toBe(false,);
          },
        },),
        it({
          name: 'rejects blank lines',
          fn: async () => {
            expect(isDiagnosticLine('',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects summary lines',
          fn: async () => {
            expect(isDiagnosticLine('Found 3 errors in 2 files.',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'isDiagnosticLine edge cases',
      children: [
        it({
          name: 'rejects an all-whitespace line',
          fn: async () => {
            expect(isDiagnosticLine('     ',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects a line with no error-code token',
          fn: async () => {
            expect(isDiagnosticLine('just some prose without a diagnostic',),)
              .toBe(false,);
          },
        },),
        it({
          name: 'rejects an error code with no leading (line,col) prefix',
          fn: async () => {
            expect(isDiagnosticLine('): error TS123:',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects a missing error number',
          fn: async () => {
            expect(isDiagnosticLine('src/a.ts(1,2): error TS:',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects a missing trailing colon after the error number',
          fn: async () => {
            expect(isDiagnosticLine('src/a.ts(1,2): error TS123',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects an empty column digit run',
          fn: async () => {
            expect(isDiagnosticLine('src/a.ts(1,): error TS123:',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects a single number with no comma',
          fn: async () => {
            expect(isDiagnosticLine('src/a.ts(1): error TS123:',),).toBe(false,);
          },
        },),
        it({
          name: 'matches a Windows-separator path diagnostic',
          fn: async () => {
            expect(isDiagnosticLine(
              String.raw`src\app.ts(1,15): error TS2304: Cannot find name.`,
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'matches an absolute-path diagnostic',
          fn: async () => {
            expect(isDiagnosticLine(
              '/abs/path/app.ts(3,7): error TS2322: Type error.',
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'matches a long column digit run',
          fn: async () => {
            expect(isDiagnosticLine(
              `src/a.ts(1,${'9'.repeat(LONG_RUN,)}): error TS2304: x`,
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'matches a long line-number digit run',
          fn: async () => {
            expect(isDiagnosticLine(
              `src/a.ts(${'9'.repeat(LONG_RUN,)},1): error TS2304: x`,
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'matches a long error-number digit run',
          fn: async () => {
            expect(isDiagnosticLine(
              `src/a.ts(1,1): error TS${'9'.repeat(LONG_RUN,)}: x`,
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'rejects many error-code tokens that never form a valid diagnostic',
          fn: async () => {
            expect(
              isDiagnosticLine('): error TSx'.repeat(SCAN_OCCURRENCES,),),
            )
              .toBe(false,);
          },
        },),
        it({
          name: 'matches a valid diagnostic after a long non-matching prefix',
          fn: async () => {
            expect(isDiagnosticLine(
              `${'a'.repeat(LONG_RUN,)}(1,1): error TS2304: x`,
            ),)
              .toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: isNodeModulesDiagnostic.name,
      children: [
        it({
          name: 'detects forward-slash node_modules path',
          fn: async () => {
            expect(isNodeModulesDiagnostic(
              'node_modules/.bun/@jsr+zod__zod@4.3.6/src/index.ts(1,1): error TS2532: ...',
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'detects nested node_modules path',
          fn: async () => {
            expect(isNodeModulesDiagnostic(
              '../../node_modules/.bun/@jsr+zod__zod@4.3.6/src/index.ts(1,1): error TS2532: ...',
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'detects backslash node_modules path',
          fn: async () => {
            expect(isNodeModulesDiagnostic(
              String
                .raw`node_modules\.bun\@jsr+zod__zod@4.3.6\src\index.ts(1,1): error TS2532: ...`,
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'rejects project source path',
          fn: async () => {
            expect(isNodeModulesDiagnostic(
              "src/index.ts(1,1): error TS2304: Cannot find name 'foo'.",
            ),)
              .toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isContinuationLine.name,
      children: [
        it({
          name: 'detects space-indented continuation',
          fn: async () => {
            expect(isContinuationLine(
              "  Type 'string' is not assignable to type 'number'.",
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'detects tab-indented continuation',
          fn: async () => {
            expect(isContinuationLine(
              "\tType 'string' is not assignable.",
            ),)
              .toBe(true,);
          },
        },),
        it({
          name: 'rejects diagnostic lines',
          fn: async () => {
            expect(isContinuationLine(
              "src/index.ts(1,1): error TS2304: Cannot find name 'foo'.",
            ),)
              .toBe(false,);
          },
        },),
        it({
          name: 'rejects empty lines',
          fn: async () => {
            expect(isContinuationLine('',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: filterTscOutput.name,
      children: [
        it({
          name: 'passes through output with no node_modules diagnostics',
          fn: async () => {
            const input = [
              "src/index.ts(1,15): error TS2304: Cannot find name 'foo'.",
              "  Did you mean 'bar'?",
              'Found 1 error.',
            ]
              .join('\n',);

            const result = filterTscOutput(input,);

            expect(result.filtered,).toBe(input,);
            expect(result.hasRemainingErrors,).toBe(true,);
          },
        },),
        it({
          name: 'removes node_modules diagnostics and their continuation lines',
          fn: async () => {
            const input = [
              "node_modules/.bun/@jsr+zod__zod@4.3.6/src/v4/core/schemas.ts(2088,19): error TS2532: Object is possibly 'undefined'.",
              "  Type 'number | undefined' is not assignable.",
              "src/app.ts(5,3): error TS2304: Cannot find name 'foo'.",
              'Found 2 errors.',
            ]
              .join('\n',);

            const result = filterTscOutput(input,);

            expect(result.filtered,).toBe([
              "src/app.ts(5,3): error TS2304: Cannot find name 'foo'.",
              'Found 2 errors.',
            ]
              .join('\n',),);
            expect(result.hasRemainingErrors,).toBe(true,);
          },
        },),
        it({
          name: 'returns hasRemainingErrors false when all errors are from node_modules',
          fn: async () => {
            const input = [
              "node_modules/.bun/@jsr+zod__zod@4.3.6/src/v4/core/schemas.ts(2088,19): error TS2532: Object is possibly 'undefined'.",
              "  Type 'number | undefined' is not assignable.",
              "node_modules/.bun/@jsr+zod__zod@4.3.6/src/v4/core/util.ts(930,41): error TS2345: Argument of type 'number | undefined'.",
              'Found 2 errors.',
            ]
              .join('\n',);

            const result = filterTscOutput(input,);

            expect(result.filtered,).toBe('Found 2 errors.',);
            expect(result.hasRemainingErrors,).toBe(false,);
          },
        },),
        it({
          name: 'handles empty output',
          fn: async () => {
            const result = filterTscOutput('',);

            expect(result.filtered,).toBe('',);
            expect(result.hasRemainingErrors,).toBe(false,);
          },
        },),
        it({
          name:
            'handles multiple consecutive node_modules diagnostics with continuations',
          fn: async () => {
            const input = [
              "node_modules/.bun/@jsr+zod__zod@4.3.6/src/v4/core/schemas.ts(2088,19): error TS2532: Object is possibly 'undefined'.",
              '  First continuation.',
              '  Second continuation.',
              "node_modules/.bun/@jsr+zod__zod@4.3.6/src/v4/locales/he.ts(44,17): error TS18048: 'TypeNames.unknown' is possibly 'undefined'.",
              '  Continuation of second diagnostic.',
              'Found 2 errors.',
            ]
              .join('\n',);

            const result = filterTscOutput(input,);

            expect(result.filtered,).toBe('Found 2 errors.',);
            expect(result.hasRemainingErrors,).toBe(false,);
          },
        },),
        it({
          name: 'preserves interleaved project and node_modules diagnostics',
          fn: async () => {
            const input = [
              "src/a.ts(1,1): error TS2304: Cannot find name 'a'.",
              "  Did you mean 'b'?",
              "node_modules/.bun/pkg/src/index.ts(10,5): error TS2532: Object is possibly 'undefined'.",
              '  Type mismatch detail.',
              "src/b.ts(3,7): error TS2322: Type 'string' is not assignable to type 'number'.",
              'Found 3 errors.',
            ]
              .join('\n',);

            const result = filterTscOutput(input,);

            expect(result.filtered,).toBe([
              "src/a.ts(1,1): error TS2304: Cannot find name 'a'.",
              "  Did you mean 'b'?",
              "src/b.ts(3,7): error TS2322: Type 'string' is not assignable to type 'number'.",
              'Found 3 errors.',
            ]
              .join('\n',),);
            expect(result.hasRemainingErrors,).toBe(true,);
          },
        },),
        it({
          name: 'handles relative node_modules paths with ../',
          fn: async () => {
            const input = [
              "../../node_modules/.bun/@jsr+zod__zod@4.3.6/src/index.ts(1,1): error TS2532: Object is possibly 'undefined'.",
              'Found 1 error.',
            ]
              .join('\n',);

            const result = filterTscOutput(input,);

            expect(result.filtered,).toBe('Found 1 error.',);
            expect(result.hasRemainingErrors,).toBe(false,);
          },
        },),
        it({
          name: 'preserves blank lines in output',
          fn: async () => {
            const input = [
              "src/a.ts(1,1): error TS2304: Cannot find name 'a'.",
              '',
              'Found 1 error.',
            ]
              .join('\n',);

            const result = filterTscOutput(input,);

            expect(result.filtered,).toBe(input,);
            expect(result.hasRemainingErrors,).toBe(true,);
          },
        },),
      ],
    },),

    //endregion Unit tests for pure filtering functions
  ],
},);
