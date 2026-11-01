/**
 Tests for what a failed run of the linter program, or a passage that cannot
 be sent to it, says: authored from a status, a signal, a code and counts,
 never from the program's own words.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  countedCharacters,
  LintBinaryRunError,
  LintBinaryTextError,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: countedCharacters.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'AGREES THE NOUN WITH THE COUNT, one character at one and characters at any other, zero included',
          fn: async () => {
            expect([
              countedCharacters({ count: 0, },),
              countedCharacters({ count: 1, },),
              countedCharacters({ count: 2, },),
            ],).toEqual([
              '0 characters',
              '1 character',
              '2 characters',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: LintBinaryRunError.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STATES EACH FAILURE KIND by its status, signal or code and the length of the passage, and '
            + 'declares itself safe to forward',
          fn: async () => {
            const status = new LintBinaryRunError({
              binary: 'purr-lint',
              characters: 1,
              failure: {
                kind: 'status',
                status: 2,
                stderrBytes: 1,
              },
            },);
            const signal = new LintBinaryRunError({
              binary: 'purr-lint',
              characters: 40,
              failure: {
                kind: 'signal',
                signal: 'SIGKILL',
              },
            },);
            const code = new LintBinaryRunError({
              binary: 'purr-lint',
              characters: 40,
              failure: {
                kind: 'code',
                code: 'ENOBUFS',
              },
            },);
            const empty = new LintBinaryRunError({
              binary: 'purr-lint',
              characters: 40,
              failure: { kind: 'empty-reply', },
            },);

            expect({
              name: status.name,
              marked: status.messageNamesOnly,
            },).toEqual({
              name: 'LintBinaryRunError',
              marked: true,
            },);
            expect(status.message,).toContain('purr-lint exited with status 2 and wrote 1 byte to standard error',);
            expect(status.message,).toContain('while wrapping a passage of 1 character;',);
            expect(status.message,).toContain('--stdin --stdin-filename passage.md --fix',);
            expect(signal.message,).toBe(
              'purr-lint was stopped by signal SIGKILL while wrapping a passage of 40 characters',
            );
            expect(code.message,).toBe(
              'purr-lint could not be run to completion (ENOBUFS) while wrapping a passage of 40 characters',
            );
            expect(empty.message,).toContain('exited cleanly and wrote nothing for a passage that has text',);
          },
        },),
      ],
    },),

    describe({
      name: LintBinaryTextError.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES WHERE THE UNPAIRED SURROGATE STANDS and why the passage is refused, never the passage',
          fn: async () => {
            const error = new LintBinaryTextError({ unpairedIndex: 8, },);

            expect({
              name: error.name,
              marked: error.messageNamesOnly,
            },).toEqual({
              name: 'LintBinaryTextError',
              marked: true,
            },);
            expect(error.message,).toContain('unpaired UTF-16 surrogate at index 8',);
            expect(error.message,).toContain('refused rather than changed',);
          },
        },),
      ],
    },),
  ],
},);
