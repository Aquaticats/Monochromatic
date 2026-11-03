import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDefaultBranch } from './origin-containment.ts';

/**
 A well-formed answer of `git ls-remote --symref origin HEAD` for a default branch.

 @param branch - branch name after `refs/heads/`
 @returns the answer text
 */
function symrefAnswer(branch: string): string {
  return `ref: refs/heads/${branch}\tHEAD\n0123456789abcdef0123456789abcdef01234567\tHEAD\n`;
}

await describe({
  name: '',
  children: [
    describe({
      name: parseDefaultBranch.name,
      children: [
        it({
          name: 'reads the branch of the symbolic reference line',
          fn: async () => {
            expect(
              parseDefaultBranch(symrefAnswer('trunk')),
            ).toEqual({ found: true, value: 'trunk' });
          },
        }),
        it({
          name: 'keeps the slashes of a branch name',
          fn: async () => {
            expect(
              parseDefaultBranch(symrefAnswer('release/stable')),
            ).toEqual({ found: true, value: 'release/stable' });
          },
        }),
        it({
          name: 'misses when origin printed nothing',
          fn: async () => {
            expect(parseDefaultBranch('').found).toBe(false);
          },
        }),
        it({
          name: 'misses when origin printed only the hash line, as a detached remote head does',
          fn: async () => {
            expect(parseDefaultBranch('0123456789abcdef0123456789abcdef01234567\tHEAD\n').found).toBe(false);
          },
        }),
        it({
          name: 'misses when the reference is not a branch',
          fn: async () => {
            expect(parseDefaultBranch('ref: refs/tags/v1\tHEAD\n').found).toBe(false);
          },
        }),
        it({
          name: 'misses on a branch name that Git or a command line could misread',
          fn: async () => {
            const result = parseDefaultBranch(symrefAnswer('-x'));
            expect(result.found).toBe(false);
            if (!result.found)
              expect(result.reason).toContain('-x');
          },
        }),
      ],
    }),
  ],
});
