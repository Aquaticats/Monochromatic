/**
 Tests for poke content assembly in the built bash-poke artifact.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildPokeContent,
  fenceFor,
  jobOutputText,
  jobStatusText,
  longestBacktickRun,
  spoolNote,
  type PokeDetails,
} from '../dist/final/node/index.mjs';

/**
 Fields a test may replace in the baseline outcome details.
 */
type PokeDetailsOverrides = {
  readonly command?: string;
  readonly exitCode?: number;
  readonly cancelled?: boolean;
  readonly truncated?: boolean;
  readonly elidedChars?: number;
  readonly spoolPath?: string;
  readonly outputChars?: number;
};

/**
 Builds outcome details with every optional field present.
 
 @param overrides - fields replacing the complete baseline
 
 @returns details suitable for content assembly
 
 @example
 ```ts
 pokeDetails({ exitCode: 1, });
 ```
 */
function pokeDetails(overrides: PokeDetailsOverrides = {}, ): PokeDetails {
  return {
    command: 'make',
    exitCode: 0,
    cancelled: false,
    truncated: false,
    elidedChars: 0,
    spoolPath: '/tmp/pi-bash-poke/job.log',
    outputChars: 12,
    ...overrides,
  };
}

await describe({
  name: '',
  children: [
    //region jobStatusText

    describe({
      name: jobStatusText.name,
      children: [
        it({
          name: 'reports cancellation before any exit code',
          fn: async () => {
            expect(jobStatusText({ exitCode: 0, cancelled: true, }, ), ).toBe('cancelled');
          },
        }, ),
        it({
          name: 'reports an unknown exit when no code arrived',
          fn: async () => {
            expect(jobStatusText({ cancelled: false, }, ), ).toBe('exit unknown');
          },
        }, ),
        it({
          name: 'reports the numeric exit code',
          fn: async () => {
            expect(jobStatusText({ exitCode: 1, cancelled: false, }, ), ).toBe('exit 1');
            expect(jobStatusText({ exitCode: 0, cancelled: false, }, ), ).toBe('exit 0');
          },
        }, ),
      ],
    }, ),

    //endregion jobStatusText

    //region Fencing

    describe({
      name: longestBacktickRun.name,
      children: [
        it({
          name: 'measures the longest run, not the last one',
          fn: async () => {
            expect(longestBacktickRun({ text: 'plain', }, ), ).toBe(0);
            expect(longestBacktickRun({ text: '`a``b', }, ), ).toBe(2);
            expect(longestBacktickRun({ text: 'a````b`c', }, ), ).toBe(4);
            expect(longestBacktickRun({ text: '', }, ), ).toBe(0);
          },
        }, ),
      ],
    }, ),

    describe({
      name: fenceFor.name,
      children: [
        it({
          name: 'uses the minimum fence for plain output',
          fn: async () => {
            expect(fenceFor({ text: 'no fences here', }, ), ).toBe('```');
          },
        }, ),
        it({
          name: 'outgrows a triple-backtick run inside the output',
          fn: async () => {
            expect(fenceFor({ text: 'before ``` after', }, ), ).toBe('````');
          },
        }, ),
        it({
          name: 'outgrows a longer run inside the output',
          fn: async () => {
            expect(fenceFor({ text: '``````', }, ), ).toBe('```````');
          },
        }, ),
      ],
    }, ),

    //endregion Fencing

    //region spoolNote

    describe({
      name: spoolNote.name,
      children: [
        it({
          name: 'stays empty when nothing was elided',
          fn: async () => {
            expect(spoolNote({ details: pokeDetails(), }, ), ).toBe('');
          },
        }, ),
        it({
          name: 'stays empty when elision has no spool path',
          fn: async () => {
            const { spoolPath: _omitted, ...withoutPath } = pokeDetails({ truncated: true, elidedChars: 9, }, );
            expect(spoolNote({ details: withoutPath, }, ), ).toBe('');
          },
        }, ),
        it({
          name: 'names the spool path after an elision',
          fn: async () => {
            expect(
              spoolNote({
                details: pokeDetails({ truncated: true, elidedChars: 9, }, ),
              }, ),
            ).toBe('[complete output: /tmp/pi-bash-poke/job.log]');
          },
        }, ),
      ],
    }, ),

    //endregion spoolNote

    //region jobOutputText

    describe({
      name: jobOutputText.name,
      children: [
        it({
          name: 'passes recorded output through when the command started',
          fn: async () => {
            expect(jobOutputText({ output: 'hello', }, ), ).toBe('hello');
          },
        }, ),
        it({
          name: 'replaces missing output with the spawn failure',
          fn: async () => {
            expect(jobOutputText({ output: '', spawnError: 'spawn ENOENT', }, ), )
              .toBe('failed to run: spawn ENOENT');
          },
        }, ),
        it({
          name: 'appends the spawn failure to partial output',
          fn: async () => {
            expect(jobOutputText({ output: 'partial', spawnError: 'boom', }, ), )
              .toBe('partial\nfailed to run: boom');
          },
        }, ),
      ],
    }, ),

    //endregion jobOutputText

    //region buildPokeContent

    describe({
      name: buildPokeContent.name,
      children: [
        it({
          name: 'heads with the command and status, then fences the output',
          fn: async () => {
            const content = buildPokeContent({
              details: pokeDetails({ command: 'make test', exitCode: 2, }, ),
              output: 'FAILED',
              instruction: 'continue',
            }, );
            const lines = content.split('\n', );
            expect(lines[0], ).toBe('[bash finished] $ make test (exit 2)');
            expect(lines[1], ).toBe('```');
            expect(lines[2], ).toBe('FAILED');
            expect(lines[3], ).toBe('```');
            expect(lines.at(-1), ).toBe('continue');
          },
        }, ),
        it({
          name: 'says so explicitly when there was no output',
          fn: async () => {
            const content = buildPokeContent({
              details: pokeDetails(),
              output: '',
              instruction: 'continue',
            }, );
            expect(content, ).toContain('(no output)');
            expect(content, ).not.toContain('```');
          },
        }, ),
        it({
          name: 'treats whitespace-only output as no output',
          fn: async () => {
            const content = buildPokeContent({
              details: pokeDetails(),
              output: '\n  \n',
              instruction: '',
            }, );
            expect(content, ).toContain('(no output)');
            expect(content, ).not.toContain('```');
          },
        }, ),
        it({
          name: 'leaves no blank line inside the fence for a trailing newline',
          fn: async () => {
            const content = buildPokeContent({
              details: pokeDetails({ command: 'echo hi', }, ),
              output: 'hi\n',
              instruction: '',
            }, );
            expect(content, ).toBe('[bash finished] $ echo hi (exit 0)\n```\nhi\n```');
          },
        }, ),
        it({
          name: 'omits the instruction when it is empty',
          fn: async () => {
            const content = buildPokeContent({
              details: pokeDetails(),
              output: 'x',
              instruction: '',
            }, );
            expect(content.endsWith('```', ), ).toBe(true);
          },
        }, ),
        it({
          name: 'inserts the spool note between heading and body',
          fn: async () => {
            const content = buildPokeContent({
              details: pokeDetails({ truncated: true, elidedChars: 500, }, ),
              output: 'head\ntail',
              instruction: 'continue',
            }, );
            const lines = content.split('\n', );
            expect(lines[1], ).toBe('[complete output: /tmp/pi-bash-poke/job.log]');
            expect(lines[2], ).toBe('```');
          },
        }, ),
        it({
          name: 'fences with a longer run than the output contains',
          fn: async () => {
            const content = buildPokeContent({
              details: pokeDetails(),
              output: '```sh\necho hi\n```',
              instruction: '',
            }, );
            expect(content.startsWith('````\n', ) || content.includes('\n````\n'), ).toBe(true);
            expect(content, ).not.toContain('`````\n');
          },
        }, ),
        it({
          name: 'reports a cancelled job without an exit code',
          fn: async () => {
            const { exitCode: _omitted, ...cancelled } = pokeDetails({ cancelled: true, }, );
            const content = buildPokeContent({
              details: cancelled,
              output: '',
              instruction: '',
            }, );
            expect(content, ).toContain('(cancelled)');
            expect(content, ).not.toContain('exit');
          },
        }, ),
      ],
    }, ),

    //endregion buildPokeContent
  ],
}, );
