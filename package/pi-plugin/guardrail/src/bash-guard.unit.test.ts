/**
 Tests for pi guardrail Bash command blocking.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { evaluateBashGuard, } from './bash-guard.ts';
import { GUARDRAIL_NOT_BLOCKED, } from './types.ts';

await describe({
  name: 'bash guard',
  children: [
    describe({
      name: 'guard decision',
      children: [
        it({
          name: 'blocks Bash input invoking bun test',
          fn: async function testBlocksBunTestInput() {
            const decision = evaluateBashGuard({ command: 'bun test foo.unit.test.ts', },);
            if (decision === GUARDRAIL_NOT_BLOCKED)
              throw new Error('Expected bun test input to be blocked',);
            expect(decision.block,).toBe(true,);
            expect(decision.reason.includes('mise run //package/<path>:test:unit',),).toBe(true,);
          },
        },),
        it({
          name: 'allows non-object and non-string inputs',
          fn: async function testAllowsMalformedInput() {
            expect(evaluateBashGuard(undefined,),).toBe(GUARDRAIL_NOT_BLOCKED,);
            expect(evaluateBashGuard({ command: 1, },),).toBe(GUARDRAIL_NOT_BLOCKED,);
          },
        },),
      ],
    },),
  ],
},);
