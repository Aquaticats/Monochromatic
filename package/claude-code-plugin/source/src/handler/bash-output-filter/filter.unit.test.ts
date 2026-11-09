/**
 Pipeline tests for the Bash output filter.
 
 `filterOutput` and the line-collapsing state machine it drives carry the
 branches the filter's own comments argue about: the dedup threshold, runs of
 empty lines, runs interrupted part-way, and the order the transforms run in.
 These cases pin the shipped pipeline rather than the individual predicates,
 which `filter-patterns.unit.test.ts` and `filter-transforms.unit.test.ts`
 cover on their own.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  spawnSync,
} from 'node:child_process';
import {
  fileURLToPath,
} from 'node:url';
import {
  CWD_PREFIX,
  DEDUP_THRESHOLD,
  HOME_DIR,
  MAX_LINE_LENGTH,
  MAX_REPEATED_CHARS,
  PATH_PREFIX_ABSENT,
} from './filter-patterns.ts';
import {
  filterOutput,
  readStdin,
  runFilter,
} from './filter.ts';

/**
 Absolute path of the filter module, resolved from this file so the child
 process imports the same source regardless of the working directory.
 */
const FILTER_PATH = fileURLToPath(new URL('filter.ts', import.meta.url,),);

/**
 Directory whose `node_modules` resolves the package's workspace dependencies
 for the child script.
 */
const PACKAGE_ROOT = fileURLToPath(new URL('../..', import.meta.url,),);

/**
 Text the failing stream delivers before it breaks, so losing it would show.
 */
const PARTIAL_TOOL_OUTPUT = 'first line of tool output\nsecond line\n';

/**
 Child script: replaces `process.stdin` with a stream that delivers
 {@link PARTIAL_TOOL_OUTPUT} and then fails, and runs the real filter over it.
 */
const STDIN_FAILURE_SCRIPT = [
  `const { Readable, } = await import('node:stream');`,
  `const { runFilter, } = await import(${JSON.stringify(FILTER_PATH,)});`,
  `const stream = new Readable({ read() {} });`,
  `stream.push(${JSON.stringify(PARTIAL_TOOL_OUTPUT,)});`,
  `setImmediate(function fail() {`,
  `  stream.destroy(new Error('simulated stdin failure',));`,
  `});`,
  `Object.defineProperty(process, 'stdin', { value: stream, configurable: true });`,
  `await runFilter();`,
].join('\n',);

/**
 Runs the filter with a stdin stream that fails mid-read.
 
 @returns captured stdout, stderr, and whether the run never finished
 */
function runFilterOverFailingStdin(): {
  readonly stderr: string;
  readonly stdout: string;
  readonly timedOut: boolean;
} {
  /**
   Child run, bounded so a hung stream cannot stall the suite.
   */
  const result = spawnSync(
    'node',
    [
      '--input-type=module-typescript',
      '-e',
      STDIN_FAILURE_SCRIPT,
    ],
    {
      cwd: PACKAGE_ROOT,
      encoding: 'utf8',
      input: '',
      timeout: 30_000,
    },
  );
  return {
    stderr: result.stderr,
    stdout: result.stdout,
    timedOut: result.error !== undefined,
  };
}

await describe({
  name: filterOutput.name,
  children: [
    it({
      name: 'returns the empty string for empty input',
      fn: async () => {
        expect(filterOutput('',),).toBe('',);
      },
    },),
    it({
      name: 'keeps a run below the dedup threshold verbatim',
      fn: async () => {
        /**
         Line repeated one time fewer than the collapse threshold.
         */
        const repeated = Array.from(
          { length: DEDUP_THRESHOLD - 1, },
          function repeatLine(): string {
            return 'hello';
          },
        ).join('\n',);
        expect(filterOutput(repeated,),).toBe(repeated,);
      },
    },),
    it({
      name: 'collapses a run that reaches the dedup threshold',
      fn: async () => {
        /**
         Line repeated exactly the collapse threshold number of times.
         */
        const repeated = Array.from(
          { length: DEDUP_THRESHOLD, },
          function repeatLine(): string {
            return 'hello';
          },
        ).join('\n',);
        expect(filterOutput(repeated,),)
          .toBe(`hello (x${String(DEDUP_THRESHOLD,)} repeated lines)`,);
      },
    },),
    it({
      name: 'keeps a single occurrence of a line',
      fn: async () => {
        expect(filterOutput('hello',),).toBe('hello',);
      },
    },),
    it({
      name: 'collapses a run of empty lines past the threshold',
      fn: async () => {
        /**
         Blank run exactly at the collapse threshold.
         */
        const blankRun = '\n'.repeat(DEDUP_THRESHOLD - 1,);
        expect(filterOutput(blankRun,),)
          .toBe(` (x${String(DEDUP_THRESHOLD,)} repeated lines)`,);
      },
    },),
    it({
      name: 'keeps a run that is interrupted before the threshold',
      fn: async () => {
        expect(filterOutput('a\na\nb\na\na',),).toBe('a\na\nb\na\na',);
      },
    },),
    it({
      name: 'collapses two separate long runs independently',
      fn: async () => {
        /**
         Marker the pipeline emits for a collapsed run of `line`.
         */
        const marker = (line: string,): string => `${
          line
        } (x${String(DEDUP_THRESHOLD,)} repeated lines)`;
        /**
         Two runs of the threshold length, written as one input buffer.
         */
        const twoRuns = [
          ...Array.from(
            { length: DEDUP_THRESHOLD, },
            function firstRun(): string {
              return 'a';
            },
          ),
          ...Array.from(
            { length: DEDUP_THRESHOLD, },
            function secondRun(): string {
              return 'b';
            },
          ),
        ].join('\n',);
        expect(filterOutput(twoRuns,),)
          .toBe(`${marker('a',)}\n${marker('b',)}`,);
      },
    },),
    it({
      name: 'strips noise lines before deduplication',
      fn: async () => {
        /**
         Noise line the git-mode predicate claims, appearing as a run so a
         dedup-before-strip order would leave a marker behind.
         */
        const noiseRun = Array.from(
          { length: DEDUP_THRESHOLD, },
          function noiseLine(): string {
            return ' create mode 100644 foo.ts';
          },
        ).join('\n',);
        expect(filterOutput(`${noiseRun}\nreal output`,),)
          .toBe('real output',);
      },
    },),
    it({
      name: 'trims trailing whitespace before comparing lines',
      fn: async () => {
        expect(
          filterOutput(
            Array.from(
              { length: DEDUP_THRESHOLD, },
              function paddedLine(): string {
                return 'hello   ';
              },
            ).join('\n',),
          ),
        ).toBe(`hello (x${String(DEDUP_THRESHOLD,)} repeated lines)`,);
      },
    },),
    it({
      name: 'truncates an over-long line through the pipeline',
      fn: async () => {
        /**
         Single output line one character over the truncation limit.
         */
        const overLong = 'x'.repeat(MAX_LINE_LENGTH + 1,);
        expect(filterOutput(overLong,),)
          .toBe(`${'x'.repeat(MAX_LINE_LENGTH,)}... [${String(MAX_LINE_LENGTH + 1,)} chars]`,);
      },
    },),
    it({
      name: 'collapses a line-initial working-directory path',
      fn: async () => {
        if (CWD_PREFIX === PATH_PREFIX_ABSENT) {
          // Nothing to collapse here; the filter must then pass the line through.
          expect(filterOutput('/repo/src/index.ts',),).toBe('/repo/src/index.ts',);
          return;
        }
        expect(filterOutput(`${CWD_PREFIX}src/index.ts`,),).toBe('src/index.ts',);
      },
    },),
    it({
      name: 'collapses a line-initial home path',
      fn: async () => {
        if (HOME_DIR === PATH_PREFIX_ABSENT) {
          expect(filterOutput('/home/alice/notes.md',),).toBe('/home/alice/notes.md',);
          return;
        }
        expect(filterOutput(`${HOME_DIR}/notes.md`,),).toBe('~/notes.md',);
      },
    },),
    it({
      name: 'collapses long character runs in a surviving line',
      fn: async () => {
        /**
         Run of a non-word character long enough for the character collapse.
         */
        const separator = '='.repeat(MAX_LINE_LENGTH,);
        expect(filterOutput(separator,),)
          .toBe(`${'='.repeat(MAX_REPEATED_CHARS,)} (x${String(MAX_LINE_LENGTH,)} repeated characters)`,);
      },
    },),

    describe({
      name: readStdin.name,
      children: [
        it({
          name: 'keeps the bytes that arrived when the stream fails part-way',
          fn: async () => {
            const { Readable, } = await import('node:stream');
            /**
             Stream that delivers one line and then fails.
             */
            const stream = new Readable({ read() {} });
            stream.push(PARTIAL_TOOL_OUTPUT,);
            /**
             Read over that stream, which must report both the text and the failure.
             */
            const read = readStdin(stream,);
            setImmediate(function fail() {
              stream.destroy(new Error('simulated stdin failure',),);
            });
            const stdin = await read;
            expect(stdin.text,)
              .toBe(PARTIAL_TOOL_OUTPUT,);
            expect(stdin.failure?.message,)
              .toBe('simulated stdin failure',);
          },
        },),
        it({
          name: 'reports no failure when the stream ends normally',
          fn: async () => {
            const { Readable, } = await import('node:stream');
            /**
             Read of a stream carrying one complete payload.
             */
            const stdin = await readStdin(Readable.from([PARTIAL_TOOL_OUTPUT,],),);
            expect(stdin.text,)
              .toBe(PARTIAL_TOOL_OUTPUT,);
            expect(stdin.failure,)
              .toBeUndefined();
          },
        },),
        it({
          name: 'decodes a multi-byte character split across two chunks',
          fn: async () => {
            const { Readable, } = await import('node:stream');
            /**
             UTF-8 bytes of one character, split mid-sequence across two chunks.
             */
            const encoded = Buffer.from('✳', 'utf8',);
            /**
             Read of a stream whose chunks each carry half the character.
             */
            const stdin = await readStdin(Readable.from([
              encoded.subarray(
                0,
                1,
              ),
              encoded.subarray(1,),
            ],),);
            expect(stdin.text,)
              .toBe('✳',);
          },
        },),
      ],
    },),

    describe({
      name: 'runFilter when the stdin read fails',
      children: [
        it({
          name: 'forwards the partial tool output and reports the failure on stderr',
          fn: async () => {
            /**
             Filter run over the failing stream.
             */
            const run = runFilterOverFailingStdin();
            expect(run.timedOut,)
              .toBe(false,);
            expect(run.stdout,)
              .toBe(PARTIAL_TOOL_OUTPUT,);
            expect(run.stderr,)
              .toContain('simulated stdin failure',);
          },
        },),
      ],
    },),
  ],
},);
