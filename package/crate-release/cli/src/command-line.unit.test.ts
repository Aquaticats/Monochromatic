import { spawnSync } from 'node:child_process';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { adversarialLines } from './adversarial-fixture.ts';
import { formatCommandLine } from './command-line.ts';

/**
 Let a real `sh` split a formatted command line into words, then report the words.

 @param line - command line under test
 @returns the words the shell produced, in order
 */
function wordsOfShell(line: string): readonly string[] {
  /**
   Shell run that prints each word followed by a NUL, so words with line breaks stay whole.
   */
  const result = spawnSync('sh', ['-c', `printf '%s\\0' ${line}`], { encoding: 'utf8' });
  return result.stdout.split('\0').slice(0, -1);
}

await describe({
  name: '',
  children: [
    describe({
      name: formatCommandLine.name,
      children: [
        it({
          name: 'leaves plain words bare so the line stays readable',
          fn: async () => {
            expect(formatCommandLine({
              program: 'cargo',
              args: ['check', '--offline', '--bin', 'cli-git-native'],
            })).toBe('cargo check --offline --bin cli-git-native');
          },
        }),
        it({
          name: 'keeps path-like and assignment-like words bare',
          fn: async () => {
            expect(formatCommandLine({
              program: 'systemd-run',
              args: ['--property=MemoryMax=12G', 'package/a/Cargo.toml', '//package/a:lock:members'],
            })).toBe('systemd-run --property=MemoryMax=12G package/a/Cargo.toml //package/a:lock:members');
          },
        }),
        it({
          name: 'quotes an empty argument so it stays one word',
          fn: async () => {
            expect(formatCommandLine({ program: 'p', args: [''] })).toBe(`p ''`);
          },
        }),
        it({
          name: 'quotes a word with a space',
          fn: async () => {
            expect(formatCommandLine({ program: 'p', args: ['a b'] })).toBe(`p 'a b'`);
          },
        }),
        ...adversarialLines.map(function roundTrip(note) {
          return it({
            name: `a real sh splits the line holding ${JSON.stringify(note)} back into the same words`,
            fn: async () => {
              const args = ['commit', '--message', `chore(x): release 1.0.0: ${note}`, '--', note];
              expect(wordsOfShell(formatCommandLine({ program: 'git', args }))).toEqual(['git', ...args]);
            },
          });
        }),
        it({
          name: 'keeps a line break inside one word',
          fn: async () => {
            const args = ['a\nb', 'c'];
            expect(wordsOfShell(formatCommandLine({ program: 'p', args }))).toEqual(['p', ...args]);
          },
        }),
        it({
          name: 'quotes the program name too',
          fn: async () => {
            expect(wordsOfShell(formatCommandLine({ program: 'my prog', args: [] }))).toEqual(['my prog']);
          },
        }),
        it({
          name: 'rejects an argument that holds NUL, which no command line can carry',
          fn: async () => {
            let caught: unknown;
            try {
              formatCommandLine({ program: 'p', args: ['a\0b'] });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(Error);
          },
        }),
      ],
    }),
  ],
});
