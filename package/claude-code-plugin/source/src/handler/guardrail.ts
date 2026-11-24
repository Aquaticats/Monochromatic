import type {
  PreToolUseInput,
  PreToolUseOutput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  BUN_TEST_BAN_REASON,
  findLiteralRmPathsInCommand,
  hasNotUnderstoodShellSyntax,
  invokesBunTest,
  invokesMutatingGitThroughRealGitPath,
  literalRmPathFindingsAnEditWouldAdd,
  literalRmPathFindingsAWriteWouldAdd,
  literalRmPathRefusal,
  type LiteralRmPathFinding,
  NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON,
  PROCESS_NAME_WAIT_BAN_REASON,
  REAL_GIT_MUTATION_BAN_REASON,
  waitsOnProcessName,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer/ts';

import {
  fileChangeOf,
  NO_FILE_CHANGE,
  type FileChangeRequest,
} from './guardrail-file-change.ts';

import {
  makeHookInputParser,
  makeJsonWriter,
  type Parser,
  type JsonWriter,
} from '../runtime/handler-runtime.ts';
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
 Deny response for shell syntax the analyzer cannot read.
 */
const NOT_UNDERSTOOD_SHELL_SYNTAX_DENY_OUTPUT: GuardrailOutput = {
  hookSpecificOutput: {
    hookEventName: 'PreToolUse',
    permissionDecision: 'deny',
    permissionDecisionReason: NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON,
  },
};

/**
 Deny response for mutating Git commands that run the real Git binary by path.
 */
const REAL_GIT_MUTATION_DENY_OUTPUT: GuardrailOutput = {
  hookSpecificOutput: {
    hookEventName: 'PreToolUse',
    permissionDecision: 'deny',
    permissionDecisionReason: REAL_GIT_MUTATION_BAN_REASON,
  },
};

/**
 Build deny response for command lines that delete a path they compute.

 @param findings - what the call was found to run

 @returns deny response naming each finding and the spelling the rule asks for

 @example
 ```ts
 literalRmPathDenyOutput([{ command: 'rm -rf "$d"', word: '"$d"' }]);
 ```
 */
function literalRmPathDenyOutput(findings: readonly LiteralRmPathFinding[],): GuardrailOutput {
  return {
    hookSpecificOutput: {
      hookEventName: 'PreToolUse',
      permissionDecision: 'deny',
      permissionDecisionReason: literalRmPathRefusal({ findings, },),
    },
  };
}

/**
 Reports the violations one file change would add, whichever text it changes.

 A write replaces the file and an edit replaces strings inside it,
 and the shared analyzer judges the text each call leaves on disk.

 @param change - file change the call asks for

 @returns deny response when the change adds a violation, `{}` when it adds none

 @example
 ```ts
 await fileChangeDenyOutput({ kind: 'write', path: '/repo/clean.sh', content: 'rm -rf "$d"\n' });
 ```
 */
async function fileChangeDenyOutput(change: FileChangeRequest,): Promise<GuardrailOutput> {
  /**
   Violations the call would add to the file.
   */
  const findings = (change.kind === 'write')
    ? await literalRmPathFindingsAWriteWouldAdd({
      path: change.path,
      content: change.content,
    },)
    : await literalRmPathFindingsAnEditWouldAdd({
      path: change.path,
      edits: change.edits,
    },);
  return (findings.length === 0)
    ? {}
    : literalRmPathDenyOutput(findings,);
}

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
 
 The Git mutation check resolves the real Git executable,
 so the handler answers asynchronously and the hook runtime awaits it.
 
 @param event - parsed {@link PreToolUseInput} event from Claude Code
 
 @returns deny response with reason, or `{}` to allow tool call
 
 @example
 ```ts
 await guardrailHandler({ tool_name: 'Bash', tool_input: { command: 'mise run //pkg:test' }, ... });
 ```
 */
async function guardrailHandler(
  event: ReadonlyDeep<PreToolUseInput>,
): Promise<GuardrailOutput> {
  if (event.tool_name === 'Bash') {
    /**
     Bash command string extracted defensively; `undefined` when field is absent.
     */
    const command = 'command' in event.tool_input
      ? event.tool_input
        .command
      : undefined;

    if ((typeof command) === 'string') {
      if (hasNotUnderstoodShellSyntax(command,))
        return NOT_UNDERSTOOD_SHELL_SYNTAX_DENY_OUTPUT;

      if (invokesBunTest(command,))
        return BUN_TEST_DENY_OUTPUT;

      if (waitsOnProcessName(command,))
        return PROCESS_NAME_WAIT_DENY_OUTPUT;

      if (await invokesMutatingGitThroughRealGitPath({ command, },))
        return REAL_GIT_MUTATION_DENY_OUTPUT;

      /**
       Command lines this call would run that delete a path they compute,
       including the script files it runs and the here-document bodies it writes.
       */
      const literalRmPathFindings = await findLiteralRmPathsInCommand({
        command,
        cwd: event.cwd,
      },);
      if (literalRmPathFindings.length > 0)
        return literalRmPathDenyOutput(literalRmPathFindings,);
    }

    return {};
  }

  /**
   File change this call asks for, which every editing tool reports.
   */
  const fileChange = fileChangeOf({
    toolName: event.tool_name,
    input: event.tool_input,
  },);
  if (fileChange !== NO_FILE_CHANGE)
    return await fileChangeDenyOutput(fileChange,);

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
 
 @example
 ```ts
 const event = guardrailParser(await text(process.stdin));
 ```
 */
const guardrailParser: Parser<PreToolUseInput> = makeHookInputParser<PreToolUseInput>();

/**
 Serializes guardrail output for stdout.
 
 No trailing newline; matches Claude Code's wire convention.
 
 @example
 ```ts
 process.stdout.write(guardrailWriter({}));
 ```
 */
const guardrailWriter: JsonWriter<GuardrailOutput> = makeJsonWriter<GuardrailOutput>();

export type { GuardrailOutput, };

export {
  guardrailHandler,
  guardrailParser,
  guardrailWriter,
};
