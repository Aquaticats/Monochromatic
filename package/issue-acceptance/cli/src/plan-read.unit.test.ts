/**
 Tests for plan reading and validation.

 @module
 */

import { mkdtempDisposable, writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parsePlan,
  PlanError,
  readPlan,
} from '../dist/final/node/index.mjs';

/**
 Returns the error a synchronous call throws.
 
 @param action - Call expected to refuse.
 
 @returns Whatever the call threw.
 
 @throws {@link Error} when the call returns, which means it refused nothing.
 
 @example
 ```ts
 const refused = failureOf(() => parsePlan({ text: '{}', at: 'plan.json' }));
 ```
 */
function failureOf(action: () => unknown,): unknown {
  try {
    action();
  }
  catch (error) {
    return error;
  }
  throw new Error('The call returned where the test expects a refusal.',);
}

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
 Plan text that is valid, with one issue whose checks cover both a count and a commit.
 */
const VALID_PLAN = JSON.stringify({
  repository: 'Aquaticats/Monochromatic',
  checkout: '/var/mnt/encrypted/repository/Monochromatic',
  branch: 'main',
  issues: [
    {
      issue: 1_251,
      action: 'close',
      checks: [
        {
          kind: 'exactly',
          pattern: String.raw`for \(let `,
          paths: ['package/cloudflare-worker/rand/src',],
          count: 0,
        },
        { kind: 'onBranch', commit: '2e00b12885', },
      ],
      comment: 'Fixed on main as 2e00b12885.',
    },
  ],
  commitCorrections: [{ commit: '2b309f8f25', body: 'Correction.', },],
},);

/**
 Parses a plan and returns the failure message it raised.
 
 @param text - Plan text to parse.
 
 @returns The failure message.
 
 @example
 ```ts
 const message = await refusalOf('{}');
 ```
 */
function refusalOf(text: string,): string {
  /**
   Value the parse threw.
   */
  const caught = failureOf(() => parsePlan({
    text,
    at: 'plan.json',
  },),);
  expect(caught,).toBeInstanceOf(PlanError,);
  return (caught as Error).message;
}

await describe({
  name: '',
  children: [
    describe({
      name: parsePlan.name,
      children: [
        it({
          name: 'reads a plan that names its checks, actions, and corrections',
          fn: async () => {
            const plan = parsePlan({
              text: VALID_PLAN,
              at: 'plan.json',
            },);
            expect(plan.repository,).toBe('Aquaticats/Monochromatic',);
            expect(plan.issues,).toHaveLength(1,);
            expect(plan.issues[0]?.action,).toBe('close',);
            expect(plan.issues[0]?.checks,).toHaveLength(2,);
            expect(plan.commitCorrections,).toHaveLength(1,);
          },
        },),
        it({
          name: 'accepts a plan with no commit corrections',
          fn: async () => {
            const plan = parsePlan({
              text: JSON.stringify({
                repository: 'OWNER/NAME',
                checkout: '/repo',
                branch: 'main',
                issues: [],
              },),
              at: 'plan.json',
            },);
            expect(plan.commitCorrections,).toEqual([],);
          },
        },),
        it({
          name: 'refuses text that is not JSON',
          fn: async () => {
            const message = refusalOf('not json',);
            expect(message,).toContain('is not valid JSON',);
          },
        },),
        it({
          name: 'refuses a plan that is not an object',
          fn: async () => {
            const message = refusalOf('[]',);
            expect(message,).toContain('must be a JSON object',);
          },
        },),
        it({
          name: 'refuses a key the plan does not define',
          fn: async () => {
            const message = refusalOf(
              JSON.stringify({ repository: 'OWNER/NAME', checkout: '/repo', branch: 'main', issues: [], commitCorrection: [], },),
            );
            expect(message,).toContain('plan.json.commitCorrection',);
          },
        },),
        it({
          name: 'refuses a repository that is not OWNER/NAME',
          fn: async () => {
            const message = refusalOf(
              JSON.stringify({ repository: 'Monochromatic', checkout: '/repo', branch: 'main', issues: [], },),
            );
            expect(message,).toContain('an OWNER/NAME repository slug',);
          },
        },),
        it({
          name: 'refuses a missing branch',
          fn: async () => {
            const message = refusalOf(
              JSON.stringify({ repository: 'OWNER/NAME', checkout: '/repo', issues: [], },),
            );
            expect(message,).toContain('plan.json.branch must be a non-empty string',);
            expect(message,).toContain('missing',);
          },
        },),
        it({
          name: 'refuses an action the command does not implement',
          fn: async () => {
            const message = refusalOf(
              JSON.stringify({
                repository: 'OWNER/NAME',
                checkout: '/repo',
                branch: 'main',
                issues: [{ issue: 1, action: 'reopen', checks: [], comment: 'x', },],
              },),
            );
            expect(message,).toContain('plan.json.issues[0].action',);
            expect(message,).toContain('one of close, comment',);
          },
        },),
        it({
          name: 'refuses a check kind the command does not implement',
          fn: async () => {
            const message = refusalOf(
              JSON.stringify({
                repository: 'OWNER/NAME',
                checkout: '/repo',
                branch: 'main',
                issues: [{ issue: 1, action: 'close', checks: [{ kind: 'compiles', },], comment: 'x', },],
              },),
            );
            expect(message,).toContain('plan.json.issues[0].checks[0].kind',);
            expect(message,).toContain('one of absent, present, exactly',);
          },
        },),
        it({
          name: 'refuses a field a check kind does not take',
          fn: async () => {
            const message = refusalOf(
              JSON.stringify({
                repository: 'OWNER/NAME',
                checkout: '/repo',
                branch: 'main',
                issues: [{
                  issue: 1,
                  action: 'close',
                  checks: [{ kind: 'exists', path: 'README.md', pattern: 'x', },],
                  comment: 'x',
                },],
              },),
            );
            expect(message,).toContain('plan.json.issues[0].checks[0].pattern',);
          },
        },),
        it({
          name: 'refuses an empty path list, which would match nothing anywhere',
          fn: async () => {
            const message = refusalOf(
              JSON.stringify({
                repository: 'OWNER/NAME',
                checkout: '/repo',
                branch: 'main',
                issues: [{
                  issue: 1,
                  action: 'close',
                  checks: [{ kind: 'absent', pattern: 'x', paths: [], },],
                  comment: 'x',
                },],
              },),
            );
            expect(message,).toContain('a non-empty array of paths',);
          },
        },),
        it({
          name: 'refuses a count below zero',
          fn: async () => {
            const message = refusalOf(
              JSON.stringify({
                repository: 'OWNER/NAME',
                checkout: '/repo',
                branch: 'main',
                issues: [{
                  issue: 1,
                  action: 'close',
                  checks: [{ kind: 'atLeast', pattern: 'x', paths: ['a',], count: -1, },],
                  comment: 'x',
                },],
              },),
            );
            expect(message,).toContain('an integer of at least 0',);
          },
        },),
        it({
          name: 'refuses a commit id that is not hexadecimal',
          fn: async () => {
            const message = refusalOf(
              JSON.stringify({
                repository: 'OWNER/NAME',
                checkout: '/repo',
                branch: 'main',
                issues: [{
                  issue: 1,
                  action: 'close',
                  checks: [{ kind: 'onBranch', commit: 'zooo', },],
                  comment: 'x',
                },],
              },),
            );
            expect(message,).toContain('a lowercase hexadecimal commit id',);
          },
        },),
        it({
          name: 'refuses an empty comment, which would close an issue with no evidence',
          fn: async () => {
            const message = refusalOf(
              JSON.stringify({
                repository: 'OWNER/NAME',
                checkout: '/repo',
                branch: 'main',
                issues: [{ issue: 1, action: 'close', checks: [], comment: '', },],
              },),
            );
            expect(message,).toContain('plan.json.issues[0].comment must be a non-empty string',);
          },
        },),
        it({
          name: 'refuses one issue named twice',
          fn: async () => {
            const message = refusalOf(
              JSON.stringify({
                repository: 'OWNER/NAME',
                checkout: '/repo',
                branch: 'main',
                issues: [
                  { issue: 7, action: 'close', checks: [], comment: 'x', },
                  { issue: 7, action: 'comment', checks: [], comment: 'y', },
                ],
              },),
            );
            expect(message,).toContain('names issue #7 twice',);
          },
        },),
        it({
          name: 'refuses an issue number below one',
          fn: async () => {
            const message = refusalOf(
              JSON.stringify({
                repository: 'OWNER/NAME',
                checkout: '/repo',
                branch: 'main',
                issues: [{ issue: 0, action: 'close', checks: [], comment: 'x', },],
              },),
            );
            expect(message,).toContain('an integer of at least 1',);
          },
        },),
      ],
    },),
    describe({
      name: readPlan.name,
      children: [
        it({
          name: 'reads a plan from a file',
          fn: async () => {
            await using directory = await mkdtempDisposable(join(scratchRoot(), 'issue-acceptance-plan-',),);
            /**
             Plan file the test reads back.
             */
            const path = join(directory.path, 'plan.json',);
            await writeFile(path, VALID_PLAN, 'utf8',);
            /**
             Plan as the reader validated it.
             */
            const plan = await readPlan({ path, },);
            expect(plan.branch,).toBe('main',);
          },
        },),
        it({
          name: 'names the file it could not read',
          fn: async () => {
            /**
             Plan file that does not exist.
             */
            const path = join(scratchRoot(), 'issue-acceptance-absent', 'plan.json',);
            /**
             Value the read threw.
             */
            const caught = await rejectionOf(() => readPlan({ path, },),);
            expect(caught,).toBeInstanceOf(PlanError,);
            expect((caught as Error).message,).toContain(path,);
            expect((caught as Error).message,).toContain('could not be read',);
          },
        },),
      ],
    },),
  ],
},);
