import type {
  HookInput,
  PostToolUseInput,
  PreToolUseInput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import {
  buildTerminalTitle,
  buildToolTitle,
  safeTerminalTitlePayload,
  terminalTitlePath,
} from '@monochromatic-dev/agent-harness-shared-terminal-title/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ReadonlyDeep, } from 'type-fest';
import { open, } from 'node:fs/promises';

import {
  NO_STDOUT,
  type WriterOutput,
} from '../../runtime/handler-runtime.ts';
import { TOOL_TITLES, } from './tool-titles.ts';

//region Logging

/**
 Logger root for terminal-title handler.
 */
const parentLogger = tagged({ tag: 'claude-terminal-title', },);

/**
 Module logger for terminal-title handler.
 */
const moduleLogger = tagged({
  tag: 'handler',
  l: parentLogger,
},);

//endregion Logging

/**
 Prefix prepended to every terminal title to identify Claude Code activity.
 */
const TITLE_PREFIX = '✳';

/**
 OSC sequence prefix for setting terminal title text.
 */
const OSC_TITLE_SEQUENCE_PREFIX = '\u001B]0;';

/**
 OSC string terminator used after terminal title payload text.
 */
const OSC_STRING_TERMINATOR = '\u0007';

//region Tool titles

/**
 Builds a human-readable title body from a tool-use event.
 
 @param event - PreToolUse or PostToolUse hook event payload
 
 @returns descriptive lifecycle title without prefix
 
 */
function titleForTool(event: ReadonlyDeep<PreToolUseInput | PostToolUseInput>,): string {
  /**
   Tool name and input pulled from hook event for downstream formatting.
   */
  const {
    tool_name: toolName,
    tool_input: input,
  } = event;
  /**
   Tense selected from hook lifecycle.
   */
  const tense = event.hook_event_name === 'PreToolUse'
    ? 'pre'
    : 'post';
  return buildToolTitle({
    registry: TOOL_TITLES,
    toolName,
    input,
    tense,
    context: { cwd: event.cwd, },
  },);
}

//endregion Tool titles

//region Event title bodies

/**
 Formats title text for notification hooks.
 
 @param event - notification hook payload
 
 @returns notification title body
 */
function notificationTitle(
  event: ReadonlyDeep<Extract<HookInput, { hook_event_name: 'Notification' }>>,
): string {
  return `Notified: ${event.title ?? event.message}`;
}

/**
 Throws for hook variants not handled by titleForEvent.
 
 @param hookEvent - impossible hook event after exhaustive narrowing
 
 @throws when a new hook event reaches runtime without a title mapping
 */
function unexpectedHookEvent(hookEvent: never,): never {
  void hookEvent;
  throw new Error('Unhandled terminal-title hook event.',);
}

/**
 Builds a human-readable title body from any hook event.
 
 @param hookEvent - parsed hook event payload
 
 @returns short descriptive title body for terminal tab
 
 */
function titleForEvent(hookEvent: ReadonlyDeep<HookInput>,): string {
  if ((hookEvent.hook_event_name === 'PreToolUse')
    || (hookEvent.hook_event_name === 'PostToolUse'))
    {
    return titleForTool(hookEvent,);
  }
  if (hookEvent.hook_event_name === 'PermissionRequest')
    return `Requesting permission: ${hookEvent.tool_name}`;
  if (hookEvent.hook_event_name === 'PostToolUseFailure')
    return `Failed tool: ${hookEvent.tool_name}`;
  if (hookEvent.hook_event_name === 'SessionStart')
    return `Started session: ${hookEvent.source}`;
  if (hookEvent.hook_event_name === 'InstructionsLoaded') {
    return `Loaded instructions: ${
      terminalTitlePath({
        filePath: hookEvent.file_path,
        cwd: hookEvent.cwd,
      },)
    }`;
  }
  if (hookEvent.hook_event_name === 'UserPromptSubmit')
    return `Received prompt: ${hookEvent.prompt}`;
  if (hookEvent.hook_event_name === 'Notification')
    return notificationTitle(hookEvent,);
  if (hookEvent.hook_event_name === 'SubagentStart')
    return `Starting subagent: ${hookEvent.agent_type}`;
  if (hookEvent.hook_event_name === 'SubagentStop')
    return `Finished subagent: ${hookEvent.agent_type}`;
  if (hookEvent.hook_event_name === 'TeammateIdle')
    return `Marked idle: ${hookEvent.teammate_name}`;
  if (hookEvent.hook_event_name === 'TaskCompleted')
    return `Completed task: ${hookEvent.task_subject}`;
  if (hookEvent.hook_event_name === 'ConfigChange')
    return `Updated config: ${hookEvent.source}`;
  if (hookEvent.hook_event_name === 'WorktreeCreate')
    return `Created worktree: ${hookEvent.name}`;
  if (hookEvent.hook_event_name === 'WorktreeRemove')
    return 'Removed worktree';
  if (hookEvent.hook_event_name === 'PreCompact')
    return `Compacting: ${hookEvent.trigger}`;
  if (hookEvent.hook_event_name === 'Stop')
    return 'Stopped agent';
  if (hookEvent.hook_event_name === 'SessionEnd')
    return 'Ended session';
  return unexpectedHookEvent(hookEvent,);
}

//endregion Event title bodies

//region Terminal title output

/**
 Writes an OSC 0 escape sequence to `/dev/tty` to set the terminal tab title.
 Fails silently if `/dev/tty` is unavailable.
 
 @param titlePayload - already-safe title payload text placed between OSC delimiters
 */
async function setTerminalTitlePayload(titlePayload: string,): Promise<void> {
  try {
    /**
     Write-mode file handle for `/dev/tty`; closed by async disposal on scope exit.
     */
    await using tty = await open(
      '/dev/tty',
      'w',
    );
    await tty.write(`${OSC_TITLE_SEQUENCE_PREFIX}${titlePayload}${OSC_STRING_TERMINATOR}`,);
  }
  catch (error: unknown) {
    moduleLogger.debug(`terminal title tty write skipped: ${String(error,)}`,);
  }
}

/**
 Output is `void`: the handler writes its OSC sequence to `/dev/tty` and emits no stdout.
 */
type TerminalTitleOutput = void;

/**
 Builds final safe title payload text for a Claude Code hook event.
 
 @param event - parsed hook event from Claude Code
 
 @returns prefixed title payload text safe to place inside an OSC 0 sequence
 
 
 @example
 ```ts
 terminalTitleForEvent({ hook_event_name: 'Stop', session_id: 's', transcript_path: 't', cwd: '.' });
 ```
 */
function terminalTitleForEvent(event: ReadonlyDeep<HookInput>,): string {
  return safeTerminalTitlePayload({
    value: buildTerminalTitle({
      prefix: TITLE_PREFIX,
      body: titleForEvent(event,),
    },),
  },);
}

/**
 Builds title payload for hook event and writes OSC 0 sequence to `/dev/tty`.
 
 @param event - parsed hook event from Claude Code
 
 @returns nothing; title is set as side effect via `/dev/tty`
 
 
 @example
 ```ts
 await terminalTitleHandler(event);
 ```
 */
async function terminalTitleHandler(event: ReadonlyDeep<HookInput>,): Promise<TerminalTitleOutput> {
  await setTerminalTitlePayload(terminalTitleForEvent(event,),);
}

//endregion Terminal title output

//region Runtime adapter

/**
 Parses raw stdin as a hook event union.
 
 @param raw - JSON payload from Claude Code stdin
 
 @returns parsed hook event union
 
 @example
 ```ts
 terminalTitleParser('{"hook_event_name":"Stop"}');
 ```
 */
function terminalTitleParser(raw: string,): HookInput {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted JSON contract from Claude Code hook system
  return JSON.parse(raw,) as HookInput;
}

/**
 Returns {@link NO_STDOUT}; title output is written directly to `/dev/tty`.
 
 @param _output - ignored handler result
 
 @returns sentinel instructing runtime to emit no stdout bytes
 
 @example
 ```ts
 terminalTitleWriter();
 ```
 */
function terminalTitleWriter(_output: TerminalTitleOutput,): WriterOutput {
  return NO_STDOUT;
}

//endregion Runtime adapter

export type { TerminalTitleOutput, };

export {
  terminalTitleForEvent,
  terminalTitleHandler,
  terminalTitleParser,
  terminalTitleWriter,
};
