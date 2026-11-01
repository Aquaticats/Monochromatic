import type { PreToolUseInput, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  guardrailHandler,
  type GuardrailOutput,
} from './guardrail.ts';

/**
 Builds a synthetic Agent PreToolUse event with the given tool_input.
 
 Test-only helper; production events are produced by Claude Code's hook
 dispatcher and parsed via `guardrailParser`.
 
 @param toolInput - shape passed to the Agent tool
 
 @returns event object that satisfies {@link PreToolUseInput}
 */
function makeAgentEvent(toolInput: Record<string, unknown>,): PreToolUseInput {
  return {
    hook_event_name: 'PreToolUse',
    session_id: 'session-test',
    transcript_path: '/tmp/transcript.jsonl',
    cwd: '/tmp/cwd',
    permission_mode: 'default',
    tool_name: 'Agent',
    tool_input: toolInput,
    tool_use_id: 'tool-use-test',
  };
}

/**
 Builds a synthetic Bash PreToolUse event with the given command string.
 
 @param command - shell command that the Bash tool would execute
 
 @returns event object that satisfies {@link PreToolUseInput}
 */
function makeBashEvent(command: string,): PreToolUseInput {
  return {
    hook_event_name: 'PreToolUse',
    session_id: 'session-test',
    transcript_path: '/tmp/transcript.jsonl',
    cwd: '/tmp/cwd',
    permission_mode: 'default',
    tool_name: 'Bash',
    tool_input: { command, },
    tool_use_id: 'tool-use-test',
  };
}

/**
 Returns whether the handler output is a deny response.
 
 @param output - value returned by {@link guardrailHandler}
 
 @returns `true` when `output` carries a deny decision
 */
function isDeny(output: GuardrailOutput,): boolean {
  return ('hookSpecificOutput' in output)
    && (output.hookSpecificOutput.permissionDecision === 'deny');
}

await describe({
  name: 'guardrail handler',
  children: [
    describe({
      name: 'Agent: general-purpose now allowed',
      children: [
        it({
          name: 'allows when subagent_type is missing',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeAgentEvent({},),);
            e(isDeny(result,),).toBe(false,);
          },
        },),
        it({
          name: 'allows when subagent_type is "general-purpose"',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(
              makeAgentEvent({ subagent_type: 'general-purpose', },),
            );
            e(isDeny(result,),).toBe(false,);
          },
        },),
        it({
          name: 'allows specialized subagent_type "Explore"',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(
              makeAgentEvent({ subagent_type: 'Explore', },),
            );
            e(isDeny(result,),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'Agent: resume blocking',
      children: [
        it({
          name: 'denies when resume is a string',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(
              makeAgentEvent({ subagent_type: 'Explore', resume: 'agent-id-123', },),
            );
            e(isDeny(result,),).toBe(true,);
          },
        },),
        it({
          name: 'allows when resume is absent',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(
              makeAgentEvent({ subagent_type: 'Explore', },),
            );
            e(isDeny(result,),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'Bash: bun test blocking',
      children: [
        it({
          name: 'denies "bun test"',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('bun test',),);
            e(isDeny(result,),).toBe(true,);
          },
        },),
        it({
          name: 'denies "bun test foo.ts"',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('bun test foo.ts',),);
            e(isDeny(result,),).toBe(true,);
          },
        },),
        it({
          name: 'denies "cd x && bun test"',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('cd x && bun test',),);
            e(isDeny(result,),).toBe(true,);
          },
        },),
        it({
          name: 'denies "(bun test)"',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('(bun test)',),);
            e(isDeny(result,),).toBe(true,);
          },
        },),
        it({
          name: 'denies "bun test 2>&1 | tail"',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('bun test 2>&1 | tail',),);
            e(isDeny(result,),).toBe(true,);
          },
        },),
        it({
          name: 'denies "f(){ bun test; }; f"',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('f(){ bun test; }; f',),);
            e(isDeny(result,),).toBe(true,);
          },
        },),
        it({
          name: 'allows "node src/foo.unit.test.ts" (direct file invocation)',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('node src/foo.unit.test.ts',),);
            e(isDeny(result,),).toBe(false,);
          },
        },),
        it({
          name: 'allows "bun build" (different subcommand)',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('bun build',),);
            e(isDeny(result,),).toBe(false,);
          },
        },),
        it({
          name: 'allows "bun run test" (script invocation, not test runner)',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('bun run test',),);
            e(isDeny(result,),).toBe(false,);
          },
        },),
        it({
          name: 'allows "mise run //pkg:test"',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('mise run //pkg:test',),);
            e(isDeny(result,),).toBe(false,);
          },
        },),
        it({
          name: 'allows quoted "bun test" inside echo (segment-anchored)',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(
              makeBashEvent('echo "use bun test instead"',),
            );
            e(isDeny(result,),).toBe(false,);
          },
        },),
        it({
          name: 'allows "bunx test" (different binary)',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('bunx test',),);
            e(isDeny(result,),).toBe(false,);
          },
        },),
        it({
          name: 'allows "bun tests" (different word, no test subcommand)',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('bun tests',),);
            e(isDeny(result,),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'Bash: process-name wait loop blocking',
      children: [
        it({
          name: 'denies "until ! pgrep -f build; do sleep 1; done"',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(
              makeBashEvent('until ! pgrep -f build; do sleep 1; done',),
            );
            e(isDeny(result,),).toBe(true,);
          },
        },),
        it({
          name: 'allows straight-line "pgrep -af build"',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(makeBashEvent('pgrep -af build',),);
            e(isDeny(result,),).toBe(false,);
          },
        },),
        it({
          name: 'allows PID-based "timeout 600 tail --pid=123 --follow /dev/null"',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler(
              makeBashEvent('timeout 600 tail --pid=123 --follow /dev/null',),
            );
            e(isDeny(result,),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'other tools pass through',
      children: [
        it({
          name: 'allows Edit tool',
          fn: async ({ expect: e, },) => {
            const result = guardrailHandler({
              hook_event_name: 'PreToolUse',
              session_id: 'session-test',
              transcript_path: '/tmp/transcript.jsonl',
              cwd: '/tmp/cwd',
              permission_mode: 'default',
              tool_name: 'Edit',
              tool_input: {
                file_path: '/tmp/foo.ts',
                old_string: 'a',
                new_string: 'b',
              },
              tool_use_id: 'tool-use-test',
            },);
            e(isDeny(result,),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
