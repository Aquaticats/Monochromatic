import { spawnSync } from 'node:child_process';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { adversarialValues } from './adversarial-fixture.ts';
import { CargoMutantsToolingError } from './error.ts';
import { shellQuote } from './shell-quote.ts';

/**
 Expand a quoted word with a real `sh` and return what the shell produced.

 @param word - shell word under test
 @returns the bytes `printf` wrote
 */
function expandWithShell(word: string): string {
  /**
   Shell run that prints its one argument unchanged.
   */
  const result = spawnSync('sh', ['-c', `printf %s ${word}`], { encoding: 'utf8' });
  return result.stdout;
}

await describe({
  name: '',
  children: [
    describe({
      name: shellQuote.name,
      children: [
        ...adversarialValues.map(function roundTrip(value) {
          return it({
            name: `a real sh expands the quoted form of ${JSON.stringify(value)} to the value`,
            fn: async () => {
              expect(
                expandWithShell(shellQuote(value)),
              ).toBe(value);
            },
          });
        }),
        it({
          name: 'wraps a plain value in single quotes',
          fn: async () => {
            expect(shellQuote('plain')).toBe(`'plain'`);
          },
        }),
        it({
          name: 'spells each single quote as close, escaped quote, reopen',
          fn: async () => {
            expect(shellQuote(`a'b`)).toBe(String.raw`'a'\''b'`);
          },
        }),
        it({
          name: 'quotes the empty value as an empty word',
          fn: async () => {
            expect(shellQuote('')).toBe(`''`);
          },
        }),
        it({
          name: 'rejects a NUL character',
          fn: async () => {
            expect(() => shellQuote('a\0b')).toThrow(CargoMutantsToolingError);
          },
        }),
      ],
    }),
  ],
});
