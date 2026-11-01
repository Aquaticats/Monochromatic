import type {
  PreToolUseInput,
  PreToolUseOutput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  BUN_TEST_BAN_REASON,
  invokesBunTest,
  PROCESS_NAME_WAIT_BAN_REASON,
  waitsOnProcessName,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer/ts';
import type { ReadonlyDeep, } from 'type-fest';

/**
 Output returned by the guardrail handler.
 
 Either a typed deny response or the empty pass-through `{}`.
 Every {@link PreToolUseOutput} field is optional, so `{}` is itself a valid
 {@link PreToolUseOutput}; no separate empty type is needed.
 */
type GuardrailOutput = PreToolUseOutput;

/**
 Deny response for misleading `bun test` invocations.
 */
const BUN_TEST_DENY_OUTPUT: GuardrailOutput = {
  hookSpecificOutput: {
    hookEventName: 'PreToolUse',
    permissionDecision: 'deny',
    permissionDecisionReason: BUN_TEST_BAN_REASON,
  },
};

/**
 Deny response for loops that wait on a process found by name.
 */
const PROCESS_NAME_WAIT_DENY_OUTPUT: GuardrailOutput = {
  hookSpecificOutput: {
    hookEventName: 'PreToolUse',
    permissionDecision: 'deny',
    permissionDecisionReason: PROCESS_NAME_WAIT_BAN_REASON,
  },
};

/**
 Build deny response for Agent resume polling.
 
 @param resume - Agent resume identifier
 
 @returns deny response explaining automatic background notifications
 
 @example
 ```ts
 agentResumeDenyOutput('agent-id');
 ```
 */
function agentResumeDenyOutput(resume: string,): GuardrailOutput {
  return {
    hookSpecificOutput: {
      hookEventName: 'PreToolUse',
      permissionDecision: 'deny',
      permissionDecisionReason: [
        `Blocked: Agent resume call (agent ID: ${resume}).`,
        'Background agents notify automatically on completion.',
        'Do not poll or resume running agents; wait for the notification.',
        "If you need the result now, use TaskOutput to check the agent's status.",
      ]
        .join(' ',),
    },
  };
}

/**
 Guard for Agent and Bash tool calls.
 
 @param event - parsed {@link PreToolUseInput} event from Claude Code
 
 @returns deny response with reason, or `{}` to allow tool call
 
 @example
 ```ts
 guardrailHandler({ tool_name: 'Bash', tool_input: { command: 'mise run //pkg:test' }, ... });
 ```
 */
function guardrailHandler(event: ReadonlyDeep<PreToolUseInput>,): GuardrailOutput {
  if (event.tool_name === 'Bash') {
    /**
     Bash command string extracted defensively; `undefined` when field is absent.
     */
    const command = 'command' in event.tool_input
      ? event.tool_input
        .command
      : undefined;

    if (((typeof command) === 'string') && invokesBunTest(command,))
      return BUN_TEST_DENY_OUTPUT;

    if (((typeof command) === 'string') && waitsOnProcessName(command,))
      return PROCESS_NAME_WAIT_DENY_OUTPUT;

    return {};
  }

  if (event.tool_name !== 'Agent')
    return {};

  /**
   Agent's `resume` field; presence triggers no-polling deny path.
   */
  const resume = 'resume' in event.tool_input
    ? event.tool_input
      .resume
    : undefined;

  if ((typeof resume) === 'string')
    return agentResumeDenyOutput(resume,);

  return {};
}

/**
 Parses raw stdin as a {@link PreToolUseInput}.
 
 Input is trusted; it comes from Claude Code's hook dispatch system.
 
 @param raw - JSON payload from Claude Code stdin
 
 @returns parsed PreToolUse event
 
 @example
 ```ts
 const event = guardrailParser(await text(process.stdin));
 ```
 */
function guardrailParser(raw: string,): PreToolUseInput {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted JSON contract from Claude Code hook system
  return JSON.parse(raw,) as PreToolUseInput;
}

/**
 Serializes guardrail output for stdout.
 
 No trailing newline; matches Claude Code's wire convention.
 
 @param output - {@link GuardrailOutput} handler result to serialize
 
 @returns JSON string for stdout
 
 @mutates output - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 process.stdout.write(guardrailWriter({}));
 ```
 */
function guardrailWriter(output: GuardrailOutput,): string {
  return JSON.stringify(output,);
}

export type { GuardrailOutput, };

export {
  guardrailHandler,
  guardrailParser,
  guardrailWriter,
};
