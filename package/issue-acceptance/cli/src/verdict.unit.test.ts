/**
 Tests for the check verdicts, driven by a fake branch reader.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  evaluateChecks,
  PlanError,
  type FileText,
  type GitReader,
} from '../dist/final/node/index.mjs';

/**
 Returns the error an asynchronous call rejects with.
 
 @param action - Call expected to refuse.
 
 @returns Whatever the call threw.
 
 @throws {@link Error} when the call resolves, which means it refused nothing.
 
 @example
 ```ts
 const refused = await rejectionOf(() => readPlan({ path }));
 ```
 */
async function rejectionOf(action: () => Promise<unknown>,): Promise<unknown> {
  try {
    await action();
  }
  catch (error) {
    return error;
  }
  throw new Error('The call resolved where the test expects a refusal.',);
}

/**
 Branch reader whose every answer a test sets explicitly.
 
 @param answers - Answers the checks ask for.
 
 @returns A branch reader for the test.
 
 @example
 ```ts
 const reader = readerAnswering({ count: 2 });
 ```
 */
function readerAnswering({
  count = 0,
  file = { kind: 'absent', },
  present = false,
  tracked = ['package/a/src/b.ts',],
  merged = true,
}: {
  readonly count?: number;
  readonly file?: FileText;
  readonly present?: boolean;
  readonly tracked?: readonly string[];
  readonly merged?: boolean;
},): GitReader {
  return {
    countMatches: async function countMatches() {
      return count;
    },
    fileText: async function fileText() {
      return file;
    },
    pathExists: async function pathExists() {
      return present;
    },
    trackedPaths: async function trackedPaths() {
      return tracked;
    },
    isOnBranch: async function isOnBranch() {
      return merged;
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: evaluateChecks.name,
      children: [
        //region Count checks

        it({
          name: 'absent holds when the pattern matches nowhere',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [{ kind: 'absent', pattern: String.raw`for \(let `, paths: ['package/a/src',], },],
              reader: readerAnswering({ count: 0, },),
            },);
            expect(verdict.holds,).toBe(true,);
            expect(verdict.outcomes[0]?.detail,).toContain('found 0',);
          },
        },),
        it({
          name: 'absent fails and names the count it found',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [{ kind: 'absent', pattern: String.raw`for \(let `, paths: ['package/a/src',], },],
              reader: readerAnswering({ count: 2, },),
            },);
            expect(verdict.holds,).toBe(false,);
            expect(verdict.outcomes[0]?.detail,).toBe(String.raw`absent /for \(let / in package/a/src found 2`,);
          },
        },),
        it({
          name: 'present fails when the pattern matches nowhere',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [{ kind: 'present', pattern: 'uniqueRuleCodes', paths: ['package/a/src',], },],
              reader: readerAnswering({ count: 0, },),
            },);
            expect(verdict.holds,).toBe(false,);
            expect(verdict.outcomes[0]?.detail,).toContain('found 0',);
          },
        },),
        it({
          name: 'exactly holds at the named count and fails one above it',
          fn: async () => {
            const check = {
              kind: 'exactly',
              pattern: 'shell_quote',
              paths: ['package/a/src',],
              count: 1,
            } as const;
            const held = await evaluateChecks({
              checks: [check,],
              reader: readerAnswering({ count: 1, },),
            },);
            const failed = await evaluateChecks({
              checks: [check,],
              reader: readerAnswering({ count: 2, },),
            },);
            expect(held.holds,).toBe(true,);
            expect(failed.holds,).toBe(false,);
            expect(failed.outcomes[0]?.detail,).toContain('exactly 1 of /shell_quote/',);
          },
        },),
        it({
          name: 'atLeast holds above the named count and fails below it',
          fn: async () => {
            const check = {
              kind: 'atLeast',
              pattern: '//region',
              paths: ['package/a/src/index.ts',],
              count: 2,
            } as const;
            const held = await evaluateChecks({
              checks: [check,],
              reader: readerAnswering({ count: 3, },),
            },);
            const failed = await evaluateChecks({
              checks: [check,],
              reader: readerAnswering({ count: 1, },),
            },);
            expect(held.holds,).toBe(true,);
            expect(failed.holds,).toBe(false,);
            expect(failed.outcomes[0]?.detail,).toContain('at least 2 of ///region/',);
          },
        },),

        //endregion Count checks

        //region Path checks

        it({
          name: 'exists follows the presence answer',
          fn: async () => {
            const check = { kind: 'exists', path: 'README.md', } as const;
            const held = await evaluateChecks({
              checks: [check,],
              reader: readerAnswering({ present: true, },),
            },);
            const failed = await evaluateChecks({
              checks: [check,],
              reader: readerAnswering({ present: false, },),
            },);
            expect(held.holds,).toBe(true,);
            expect(failed.holds,).toBe(false,);
            expect(failed.outcomes[0]?.detail,).toBe('exists README.md answered false',);
          },
        },),
        it({
          name: 'gone is the opposite of exists',
          fn: async () => {
            const check = { kind: 'gone', path: 'Dockerfile', } as const;
            const held = await evaluateChecks({
              checks: [check,],
              reader: readerAnswering({ present: false, },),
            },);
            const failed = await evaluateChecks({
              checks: [check,],
              reader: readerAnswering({ present: true, },),
            },);
            expect(held.holds,).toBe(true,);
            expect(failed.holds,).toBe(false,);
          },
        },),
        it({
          name: 'pathMatching holds when no tracked path matches',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [{ kind: 'pathMatching', pattern: 'zoro|teble', },],
              reader: readerAnswering({ tracked: ['package/toml-edit/core/src/index.ts',], },),
            },);
            expect(verdict.holds,).toBe(true,);
          },
        },),
        it({
          name: 'pathMatching fails and quotes the paths that matched',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [{ kind: 'pathMatching', pattern: 'zoro|teble', },],
              reader: readerAnswering({ tracked: ['package/a/zoro.ts', 'package/b/teble.ts',], },),
            },);
            expect(verdict.holds,).toBe(false,);
            expect(verdict.outcomes[0]?.detail,).toContain('package/a/zoro.ts',);
            expect(verdict.outcomes[0]?.detail,).toContain('package/b/teble.ts',);
          },
        },),
        it({
          name: 'pathMatching summarises the matches past five',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [{ kind: 'pathMatching', pattern: 'zoro', },],
              reader: readerAnswering({
                tracked: ['a1/zoro', 'a2/zoro', 'a3/zoro', 'a4/zoro', 'a5/zoro', 'a6/zoro',],
              },),
            },);
            expect(verdict.outcomes[0]?.detail,).toContain('and 1 more',);
          },
        },),
        it({
          name: 'pathMatching refuses a pattern that does not compile',
          fn: async () => {
            /**
             Value the verdict threw.
             */
            const caught = await rejectionOf(() => evaluateChecks({
              checks: [{ kind: 'pathMatching', pattern: '([', },],
              reader: readerAnswering({},),
            },),);
            expect(caught,).toBeInstanceOf(PlanError,);
            expect((caught as Error).message,).toContain('does not compile',);
          },
        },),

        //endregion Path checks

        //region File text checks

        it({
          name: 'fileContains holds when the file holds the text',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [{ kind: 'fileContains', path: 'mise.toml', text: 'test:unit', },],
              reader: readerAnswering({ file: { kind: 'text', text: '[tasks."test:unit"]', }, },),
            },);
            expect(verdict.holds,).toBe(true,);
          },
        },),
        it({
          name: 'fileContains fails when the file lacks the text',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [{ kind: 'fileContains', path: 'mise.toml', text: 'test:unit', },],
              reader: readerAnswering({ file: { kind: 'text', text: '[tasks.build]', }, },),
            },);
            expect(verdict.holds,).toBe(false,);
            expect(verdict.outcomes[0]?.detail,).toContain('answered false',);
          },
        },),
        it({
          name: 'fileContains fails when the branch has no such file',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [{ kind: 'fileContains', path: 'Dockerfile', text: '.env.local', },],
              reader: readerAnswering({},),
            },);
            expect(verdict.holds,).toBe(false,);
            expect(verdict.outcomes[0]?.detail,).toContain('no such file',);
          },
        },),
        it({
          name: 'fileLacks holds when the file lacks the text',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [{ kind: 'fileLacks', path: 'README.md', text: '\t', },],
              reader: readerAnswering({ file: { kind: 'text', text: '  indented with two spaces', }, },),
            },);
            expect(verdict.holds,).toBe(true,);
          },
        },),
        it({
          name: 'fileLacks fails when the file holds the text',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [{ kind: 'fileLacks', path: 'README.md', text: '\t', },],
              reader: readerAnswering({ file: { kind: 'text', text: '\tindented with a tab', }, },),
            },);
            expect(verdict.holds,).toBe(false,);
          },
        },),
        it({
          name: 'fileLacks holds when the branch has no such file',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [{ kind: 'fileLacks', path: 'Dockerfile', text: '.env.local', },],
              reader: readerAnswering({},),
            },);
            expect(verdict.holds,).toBe(true,);
          },
        },),

        //endregion File text checks

        //region Commit check

        it({
          name: 'onBranch follows the ancestry answer',
          fn: async () => {
            const check = { kind: 'onBranch', commit: '2e00b12885', } as const;
            const held = await evaluateChecks({
              checks: [check,],
              reader: readerAnswering({ merged: true, },),
            },);
            const failed = await evaluateChecks({
              checks: [check,],
              reader: readerAnswering({ merged: false, },),
            },);
            expect(held.holds,).toBe(true,);
            expect(failed.holds,).toBe(false,);
            expect(failed.outcomes[0]?.detail,).toBe('commit 2e00b12885 is on the branch answered false',);
          },
        },),

        //endregion Commit check

        it({
          name: 'holds only when every check holds',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [
                { kind: 'exists', path: 'README.md', },
                { kind: 'exists', path: 'Dockerfile', },
              ],
              reader: readerAnswering({ present: true, },),
            },);
            expect(verdict.outcomes,).toHaveLength(2,);
            expect(verdict.holds,).toBe(true,);
          },
        },),
        it({
          name: 'holds vacuously when the plan names no check',
          fn: async () => {
            const verdict = await evaluateChecks({
              checks: [],
              reader: readerAnswering({},),
            },);
            expect(verdict.holds,).toBe(true,);
            expect(verdict.outcomes,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
