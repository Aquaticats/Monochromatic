/**
 Tests for pi guardrail protected-path matching.
 
 @module
 */

import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createPathGuardMatcher,
  evaluatePathGuard,
} from './path-guard.ts';
import {
  extractToolPath,
  normalizeToolPath,
  TOOL_PATH_NOT_FOUND,
  TOOL_PATH_NOT_MATCHABLE,
} from './path-normalize.ts';
import {
  GUARDRAIL_NOT_BLOCKED,
  type GuardrailDecision,
  type PathRule,
} from './types.ts';

/**
 Project root used by path normalization tests.
 */
const CWD = '/repo';

/**
 Built-in-like path rule used by path guard tests.
 */
const PNPM_RULE: PathRule = {
  pattern: 'pnpm-lock.yaml',
  message: 'run pnpm install',
};

/**
 Evaluates a single path against rules for concise tests.
 
 @param path - tool input path
 
 @param rules - gitignore-style path rules
 
 @returns guard decision
 
 @example
 ```typescript
 decisionForPath({ path: 'pnpm-lock.yaml', rules: [PNPM_RULE] });
 ```
 */
function decisionForPath(
  {
    path,
    rules,
  }: {
    readonly path: string;
    readonly rules: readonly PathRule[];
  },
): GuardrailDecision {
  return evaluatePathGuard({
    input: { path, },
    cwd: CWD,
    matcher: createPathGuardMatcher(rules,),
  },);
}

await describe({
  name: 'path guard',
  children: [
    describe({
      name: 'path extraction and normalization',
      children: [
        it({
          name: 'extracts string path only',
          fn: async function testExtractPath() {
            expect(extractToolPath({ path: 'a', },),).toBe('a',);
            expect(extractToolPath({ path: 1, },),).toBe(TOOL_PATH_NOT_FOUND,);
            expect(extractToolPath(undefined,),).toBe(TOOL_PATH_NOT_FOUND,);
          },
        },),
        it({
          name: 'normalizes relative, absolute, and at-prefixed paths',
          fn: async function testNormalizePath() {
            expect(normalizeToolPath({ cwd: CWD, rawPath: 'pnpm-lock.yaml', },),).toBe('pnpm-lock.yaml',);
            expect(normalizeToolPath({ cwd: CWD, rawPath: join(CWD, 'package/a/pnpm-lock.yaml',), },),)
              .toBe('package/a/pnpm-lock.yaml',);
            expect(normalizeToolPath({ cwd: CWD, rawPath: '@pnpm-lock.yaml', },),).toBe('pnpm-lock.yaml',);
          },
        },),
        it({
          name: 'ignores empty and outside-cwd paths',
          fn: async function testOutsidePath() {
            expect(normalizeToolPath({ cwd: CWD, rawPath: '', },),).toBe(TOOL_PATH_NOT_MATCHABLE,);
            expect(normalizeToolPath({ cwd: CWD, rawPath: '../pnpm-lock.yaml', },),).toBe(TOOL_PATH_NOT_MATCHABLE,);
          },
        },),
      ],
    },),
    describe({
      name: 'gitignore matching',
      children: [
        it({
          name: 'blocks basename pattern at root and nested paths',
          fn: async function testGitignoreBasenamePattern() {
            expect(decisionForPath({ path: 'pnpm-lock.yaml', rules: [PNPM_RULE,], },),)
              .toEqual({
                block: true,
                reason: 'run pnpm install',
              },);
            expect(decisionForPath({ path: 'package/a/pnpm-lock.yaml', rules: [PNPM_RULE,], },),)
              .toEqual({
                block: true,
                reason: 'run pnpm install',
              },);
          },
        },),
        it({
          name: 'uses final matching positive rule message',
          fn: async function testFinalRuleMessage() {
            const decision = decisionForPath({
              path: 'pnpm-lock.yaml',
              rules: [
                PNPM_RULE,
                {
                  pattern: 'pnpm-lock.yaml',
                  message: 'custom message',
                },
              ],
            },);
            expect(decision,).toEqual({
              block: true,
              reason: 'custom message',
            },);
          },
        },),
        it({
          name: 'allows negated patterns to unguard defaults',
          fn: async function testNegatedPattern() {
            const decision = decisionForPath({
              path: 'pnpm-lock.yaml',
              rules: [
                PNPM_RULE,
                {
                  pattern: '!pnpm-lock.yaml',
                  message: '',
                },
              ],
            },);
            expect(decision,).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
        it({
          name: 'allows non-matching and malformed tool inputs',
          fn: async function testAllowsNonMatchingInputs() {
            const matcher = createPathGuardMatcher([PNPM_RULE,],);
            expect(evaluatePathGuard({ input: { path: 'package.json', }, cwd: CWD, matcher, },),)
              .toBe(GUARDRAIL_NOT_BLOCKED,);
            expect(evaluatePathGuard({ input: { path: 1, }, cwd: CWD, matcher, },),)
              .toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
      ],
    },),
  ],
},);
