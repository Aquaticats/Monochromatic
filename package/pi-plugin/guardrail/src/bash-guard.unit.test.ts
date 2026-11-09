/**
 Tests for pi guardrail Bash command blocking.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  evaluateBashGuard,
  GUARDRAIL_NOT_BLOCKED,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Absolute real-Git spelling used by string fixtures,
 so the guard is exercised without depending on this host's installation path.
 */
const FIXTURE_REAL_GIT_PATH = '/usr/bin/git';

/**
 Executable name that marks a path word as the real Git binary.
 */
const FIXTURE_REAL_GIT_BASENAME = 'git';

/**
 Injected executable names that keep fixture commands independent of the host.
 */
const FIXTURE_OPTIONS = { realGitBasenames: [FIXTURE_REAL_GIT_BASENAME,], };

/**
 Repository root passed to the `-C` remedy in fixture commands.
 */
const FIXTURE_REPOSITORY_ROOT = '/repo';

//endregion Fixtures

await describe({
  name: 'bash guard',
  children: [
    describe({
      name: 'bun test guard decision',
      children: [
        it({
          name: 'blocks Bash input invoking bun test',
          fn: async function testBlocksBunTestInput() {
            const decision = await evaluateBashGuard({
              input: { command: 'bun test foo.unit.test.ts', },
              ...FIXTURE_OPTIONS,
            },);
            if (decision === GUARDRAIL_NOT_BLOCKED)
              throw new Error('Expected bun test input to be blocked',);
            expect(decision.block,).toBe(true,);
            expect(decision.reason.includes('mise run //package/<path>:test:unit',),).toBe(true,);
          },
        },),
        it({
          name: 'allows non-object and non-string inputs',
          fn: async function testAllowsMalformedInput() {
            expect(
              await evaluateBashGuard({ input: undefined, ...FIXTURE_OPTIONS, },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
            expect(
              await evaluateBashGuard({ input: { command: 1, }, ...FIXTURE_OPTIONS, },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
      ],
    },),
    describe({
      name: 'unread shell syntax guard decision',
      children: [
        it({
          name: 'blocks bun test hidden inside brace expansion',
          fn: async function testBlocksBunTestInBraceExpansion() {
            const decision = await evaluateBashGuard({
              input: { command: 'printf %s {a,$(bun test)}', },
              ...FIXTURE_OPTIONS,
            },);
            if (decision === GUARDRAIL_NOT_BLOCKED)
              throw new Error('Expected bun test inside brace expansion to be blocked',);
            expect(decision.block,).toBe(true,);
            expect(decision.reason.includes('mise run //package/<path>:test:unit',),).toBe(true,);
          },
        },),
        it({
          name: 'blocks a parameter operation the analyzer cannot read',
          fn: async function testBlocksUnreadParameterOperation() {
            const decision = await evaluateBashGuard({
              input: { command: `printf %s \${value;text}`, },
              ...FIXTURE_OPTIONS,
            },);
            if (decision === GUARDRAIL_NOT_BLOCKED)
              throw new Error('Expected unread shell syntax to be blocked',);
            expect(decision.block,).toBe(true,);
            expect(decision.reason.includes('notUnderstoodNodeKinds',),).toBe(true,);
          },
        },),
        it({
          name: 'allows a parameter operation the analyzer reads',
          fn: async function testAllowsReadParameterOperation() {
            expect(
              await evaluateBashGuard({
                input: { command: `printf %s \${value:-text}`, },
                ...FIXTURE_OPTIONS,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
      ],
    },),
    describe({
      name: 'absolute real-Git path guard decision',
      children: [
        it({
          name: 'blocks a commit through an absolute real-Git path',
          fn: async function testBlocksAbsolutePathCommit() {
            const decision = await evaluateBashGuard({
              input: { command: `${FIXTURE_REAL_GIT_PATH} commit -m x`, },
              ...FIXTURE_OPTIONS,
            },);
            if (decision === GUARDRAIL_NOT_BLOCKED)
              throw new Error('Expected an absolute real-Git commit to be blocked',);
            expect(decision.block,).toBe(true,);
            expect(decision.reason.includes('git -C <repository-root>',),).toBe(true,);
          },
        },),
        it({
          name: 'blocks a chained checkout through an absolute real-Git path',
          fn: async function testBlocksChainedCheckout() {
            const decision = await evaluateBashGuard({
              input: { command: `cd ${FIXTURE_REPOSITORY_ROOT} && ${FIXTURE_REAL_GIT_PATH} checkout -- .`, },
              ...FIXTURE_OPTIONS,
            },);
            expect(decision,).not.toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
        it({
          name: 'allows a read-only probe through the same absolute path',
          fn: async function testAllowsAbsolutePathStatus() {
            expect(
              await evaluateBashGuard({
                input: { command: `${FIXTURE_REAL_GIT_PATH} status`, },
                ...FIXTURE_OPTIONS,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
            expect(
              await evaluateBashGuard({
                input: { command: `${FIXTURE_REAL_GIT_PATH} rev-parse HEAD`, },
                ...FIXTURE_OPTIONS,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
        it({
          name: 'allows the same mutation through the PATH wrapper',
          fn: async function testAllowsWrapperCommit() {
            expect(
              await evaluateBashGuard({
                input: { command: `git -C ${FIXTURE_REPOSITORY_ROOT} commit -m x`, },
                ...FIXTURE_OPTIONS,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
            expect(
              await evaluateBashGuard({
                input: { command: 'git commit -m x', },
                ...FIXTURE_OPTIONS,
              },),
            ).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
      ],
    },),
  ],
},);
