import type {
  BashToolInput,
  PreToolUseInput,
  SessionStartInput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  bashOutputFilterHandler,
  type BashOutputFilterOutput,
} from './index.ts';
import {
  BLOCKING_TIMEOUT_ENABLED_ENV,
  BLOCKING_TIMEOUT_MS,
  SLEEP_PREFIX,
} from './blocking-timeout.ts';

/**
 Caller timeout main sessions sent most often among long waits (58 minutes),
 per `doc/planning/claude-code-bash-caller-timeout-cap.md`.
 */
const OBSERVED_LONG_CALLER_TIMEOUT_MS = 3_480_000;

/**
 Start of every command the output filter wraps.
 */
const FILTER_WRAPPER_START = 'set -o pipefail && ';

function environmentOverride(env: Readonly<NodeJS.ProcessEnv>,): { [Symbol.dispose]: () => void } {
  const original = process.env;
  process.env = { ...env };
  return {
    [Symbol.dispose](): void {
      process.env = original;
    },
  };
}

function withEnv<T>(
  { env, fn }: Readonly<{
    env: Readonly<NodeJS.ProcessEnv>;
    fn: () => T;
  }>,
): T {
  using override = environmentOverride(env,);
  return fn();
}

function bashEvent(
  {
    agentId,
    agentType,
    input,
  }: Readonly<{
    agentId?: string;
    agentType?: string;
    input: BashToolInput;
  }>,
): PreToolUseInput {
  return {
    session_id: 'session-test',
    transcript_path: '/tmp/transcript.jsonl',
    cwd: '/tmp',
    permission_mode: 'default',
    hook_event_name: 'PreToolUse',
    tool_name: 'Bash',
    tool_input: input,
    tool_use_id: 'tool-1',
    ...(agentId === undefined ? {} : { agent_id: agentId, }),
    ...(agentType === undefined ? {} : { agent_type: agentType, }),
  };
}

function startupEvent(source: SessionStartInput['source'],): SessionStartInput {
  return {
    session_id: 'session-test',
    transcript_path: '/tmp/transcript.jsonl',
    cwd: '/tmp',
    permission_mode: 'default',
    hook_event_name: 'SessionStart',
    source,
  };
}

function updatedInput(output: BashOutputFilterOutput,): BashToolInput {
  if (output.hookSpecificOutput?.hookEventName !== 'PreToolUse')
    throw new Error('expected PreToolUse output',);
  return output.hookSpecificOutput.updatedInput as BashToolInput;
}

await describe({
  name: bashOutputFilterHandler.name,
  concurrency: 1,
  children: [
    it({
      name: 'passes through a non-Bash tool',
      fn: async () => {
        const event = bashEvent({ input: { command: 'printf ok', }, });
        expect(bashOutputFilterHandler({ ...event, tool_name: 'Edit', },),).toEqual({},);
      },
    },),
    it({
      name: 'passes through malformed Bash input',
      fn: async () => {
        const event = bashEvent({ input: { command: 'printf ok', }, });
        expect(bashOutputFilterHandler({ ...event, tool_input: {}, },),).toEqual({},);
      },
    },),
    it({
      name: 'adds the timeout to a filtered main-agent command and preserves description',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'printf ok', description: 'quick check', }, }),
        ) },);
        expect(updatedInput(output,).timeout,).toBe(BLOCKING_TIMEOUT_MS,);
        expect(updatedInput(output,).description,).toBe('quick check',);
        expect(updatedInput(output,).command,).toContain('set -o pipefail && printf ok',);
      },
    },),
    it({
      name: 'adds the timeout without wrapping a filter-skipped command',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'xxd file.bin', }, }),
        ) },);
        expect(updatedInput(output,).command,).toBe('xxd file.bin',);
        expect(updatedInput(output,).timeout,).toBe(BLOCKING_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'clamps a long filter-skipped caller timeout without changing the command',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'xxd file.bin', timeout: OBSERVED_LONG_CALLER_TIMEOUT_MS, }, }),
        ) },);
        expect(updatedInput(output,).command,).toBe('xxd file.bin',);
        expect(updatedInput(output,).timeout,).toBe(BLOCKING_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'passes through a filter-skipped caller timeout unchanged',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'xxd file.bin', timeout: 500, }, }),
        ) },);
        expect(output,).toEqual({},);
      },
    },),
    it({
      name: 'passes through a filter-skipped explicit background call unchanged',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'xxd file.bin', run_in_background: true, }, }),
        ) },);
        expect(output,).toEqual({},);
      },
    },),
    it({
      name: 'filters a subagent without changing execution fields',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ agentId: 'agent-1', input: { command: 'printf ok', }, }),
        ) },);
        expect(updatedInput(output,).command,).toContain('printf ok',);
        expect(updatedInput(output,).timeout,).toBeUndefined();
        expect(updatedInput(output,).run_in_background,).toBeUndefined();
      },
    },),
    it({
      name: 'keeps a subagent caller timeout',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ agentId: 'agent-1', input: { command: 'printf ok', timeout: OBSERVED_LONG_CALLER_TIMEOUT_MS, }, }),
        ) },);
        expect(updatedInput(output,).timeout,).toBe(OBSERVED_LONG_CALLER_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'leaves a subagent leading sleep unprefixed',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ agentId: 'agent-1', input: { command: 'sleep 5 > result.txt', timeout: OBSERVED_LONG_CALLER_TIMEOUT_MS, }, }),
        ) },);
        expect(output,).toEqual({},);
      },
    },),
    it({
      name: 'passes through a filter-skipped subagent',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ agentId: 'agent-1', input: { command: 'xxd file.bin', }, }),
        ) },);
        expect(output,).toEqual({},);
      },
    },),
    it({
      name: 'preserves an explicit background call and its absent timeout',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'printf ok', run_in_background: true, }, }),
        ) },);
        expect(updatedInput(output,).run_in_background,).toBe(true,);
        expect(updatedInput(output,).timeout,).toBeUndefined();
      },
    },),
    it({
      name: 'preserves an explicit background call with a supplied timeout',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'printf ok', run_in_background: true, timeout: 500, }, }),
        ) },);
        expect(updatedInput(output,).run_in_background,).toBe(true,);
        expect(updatedInput(output,).timeout,).toBe(500,);
      },
    },),
    it({
      name: 'keeps an explicit background call with a long timeout',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'printf ok', run_in_background: true, timeout: OBSERVED_LONG_CALLER_TIMEOUT_MS, }, }),
        ) },);
        expect(updatedInput(output,).timeout,).toBe(OBSERVED_LONG_CALLER_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'leaves an explicit background leading sleep unprefixed',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'sleep 5 > result.txt', run_in_background: true, timeout: OBSERVED_LONG_CALLER_TIMEOUT_MS, }, }),
        ) },);
        expect(output,).toEqual({},);
      },
    },),
    ...([1, BLOCKING_TIMEOUT_MS / 2, BLOCKING_TIMEOUT_MS,] as const).map(function keptTimeout(timeout,) {
      return it({
        name: `keeps foreground caller timeout ${timeout}`,
        fn: async () => {
          const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
            bashEvent({ input: { command: 'printf ok', timeout, }, }),
          ) },);
          expect(updatedInput(output,).timeout,).toBe(timeout,);
        },
      },);
    },),
    ...([0, BLOCKING_TIMEOUT_MS + 1, OBSERVED_LONG_CALLER_TIMEOUT_MS,] as const).map(function clampedCallerTimeout(timeout,) {
      return it({
        name: `clamps foreground caller timeout ${timeout} to the default`,
        fn: async () => {
          const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
            bashEvent({ input: { command: 'printf ok', timeout, }, }),
          ) },);
          expect(updatedInput(output,).timeout,).toBe(BLOCKING_TIMEOUT_MS,);
        },
      },);
    },),
    it({
      name: 'adds the default to explicit foreground input',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'printf ok', run_in_background: false, }, }),
        ) },);
        expect(updatedInput(output,).run_in_background,).toBe(false,);
        expect(updatedInput(output,).timeout,).toBe(BLOCKING_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'times a bare numeric sleep and filters it without a prefix',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'sleep 5', }, }),
        ) },);
        expect(updatedInput(output,).command.startsWith(`${FILTER_WRAPPER_START}sleep 5`,),).toBe(true,);
        expect(updatedInput(output,).timeout,).toBe(BLOCKING_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'filters a leading sleep chain without a prefix',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'sleep 5 && printf ok', timeout: OBSERVED_LONG_CALLER_TIMEOUT_MS, }, }),
        ) },);
        expect(updatedInput(output,).command.startsWith(`${FILTER_WRAPPER_START}sleep 5 && printf ok`,),).toBe(true,);
        expect(updatedInput(output,).timeout,).toBe(BLOCKING_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'prefixes and times an unwrapped leading sleep',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'sleep 5 > result.txt', description: 'wait then write', }, }),
        ) },);
        expect(updatedInput(output,).command,).toBe(`${SLEEP_PREFIX}sleep 5 > result.txt`,);
        expect(updatedInput(output,).timeout,).toBe(BLOCKING_TIMEOUT_MS,);
        expect(updatedInput(output,).description,).toBe('wait then write',);
      },
    },),
    it({
      name: 'prefixes an unwrapped leading sleep and clamps its long caller timeout',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: '# wait\nsleep 300 && check > out', timeout: OBSERVED_LONG_CALLER_TIMEOUT_MS, }, }),
        ) },);
        expect(updatedInput(output,).command,).toBe(`${SLEEP_PREFIX}# wait\nsleep 300 && check > out`,);
        expect(updatedInput(output,).timeout,).toBe(BLOCKING_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'prefixes an unwrapped leading sleep that keeps a short caller timeout',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'sleep 5 > result.txt', timeout: BLOCKING_TIMEOUT_MS / 2, }, }),
        ) },);
        expect(updatedInput(output,).command,).toBe(`${SLEEP_PREFIX}sleep 5 > result.txt`,);
        expect(updatedInput(output,).timeout,).toBe(BLOCKING_TIMEOUT_MS / 2,);
      },
    },),
    it({
      name: 'keeps a long caller timeout when native background tasks are disabled',
      fn: async () => {
        const output = withEnv({ env: { CLAUDE_CODE_DISABLE_BACKGROUND_TASKS: '1', }, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'printf ok', timeout: OBSERVED_LONG_CALLER_TIMEOUT_MS, }, }),
        ) },);
        expect(updatedInput(output,).timeout,).toBe(OBSERVED_LONG_CALLER_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'passes through an unwrapped leading sleep when native background tasks are disabled',
      fn: async () => {
        const output = withEnv({ env: { CLAUDE_CODE_DISABLE_BACKGROUND_TASKS: '1', }, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'sleep 5 > result.txt', timeout: OBSERVED_LONG_CALLER_TIMEOUT_MS, }, }),
        ) },);
        expect(output,).toEqual({},);
      },
    },),
    it({
      name: 'leaves a leading sleep unprefixed when the policy is disabled',
      fn: async () => {
        const output = withEnv({ env: { [BLOCKING_TIMEOUT_ENABLED_ENV]: 'off', }, fn: () => bashOutputFilterHandler(
          bashEvent({ input: { command: 'sleep 5 > result.txt', timeout: OBSERVED_LONG_CALLER_TIMEOUT_MS, }, }),
        ) },);
        expect(output,).toEqual({},);
      },
    },),
    it({
      name: 'keeps the policy enabled when only the legacy variable is off',
      fn: async () => {
        const output = withEnv({ env: { MONOCHROMATIC_BASH_FORCE_BACKGROUND: 'off', }, fn: () =>
          bashOutputFilterHandler(bashEvent({ input: { command: 'printf ok', }, })) },);
        expect(updatedInput(output,).timeout,).toBe(BLOCKING_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'filters without a timeout when the new policy is disabled',
      fn: async () => {
        const output = withEnv({ env: { [BLOCKING_TIMEOUT_ENABLED_ENV]: 'off', }, fn: () =>
          bashOutputFilterHandler(bashEvent({ input: { command: 'printf ok', }, })) },);
        expect(updatedInput(output,).command,).toContain('printf ok',);
        expect(updatedInput(output,).timeout,).toBeUndefined();
      },
    },),
    it({
      name: 'treats agent_type without agent_id as a main session',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(
          bashEvent({ agentType: 'custom', input: { command: 'printf ok', }, }),
        ) },);
        expect(updatedInput(output,).timeout,).toBe(BLOCKING_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'warns at startup without requiring a model field',
      fn: async () => {
        const output = withEnv({ env: { CLAUDE_CODE_DISABLE_BACKGROUND_TASKS: '1', }, fn: () =>
          bashOutputFilterHandler(startupEvent('startup',),) },);
        expect(output.systemMessage,).toContain(BLOCKING_TIMEOUT_ENABLED_ENV,);
      },
    },),
    ...(['resume', 'clear', 'compact',] as const).map(function nonStartupSource(source,) {
      return it({
        name: `does not warn on ${source}`,
        fn: async () => {
          const output = withEnv({ env: { CLAUDE_CODE_DISABLE_BACKGROUND_TASKS: '1', }, fn: () =>
            bashOutputFilterHandler(startupEvent(source,),) },);
          expect(output,).toEqual({},);
        },
      },);
    },),
    it({
      name: 'does not warn when native background tasks are enabled',
      fn: async () => {
        const output = withEnv({ env: {}, fn: () => bashOutputFilterHandler(startupEvent('startup',),) },);
        expect(output,).toEqual({},);
      },
    },),
    it({
      name: 'does not warn when the timeout policy is disabled',
      fn: async () => {
        const output = withEnv({ env: {
          CLAUDE_CODE_DISABLE_BACKGROUND_TASKS: '1',
          [BLOCKING_TIMEOUT_ENABLED_ENV]: '0',
        }, fn: () => bashOutputFilterHandler(startupEvent('startup',),) },);
        expect(output,).toEqual({},);
      },
    },),
    it({
      name: 'warns despite the removed legacy opt-out',
      fn: async () => {
        const output = withEnv({ env: {
          CLAUDE_CODE_DISABLE_BACKGROUND_TASKS: '1',
          MONOCHROMATIC_BASH_FORCE_BACKGROUND: 'off',
        }, fn: () => bashOutputFilterHandler(startupEvent('startup',),) },);
        expect(output.systemMessage,).toBeDefined();
      },
    },),
  ],
},);
