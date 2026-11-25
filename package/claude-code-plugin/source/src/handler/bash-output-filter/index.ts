import type {
  SessionStartInput,
  SessionStartOutput,
  BashToolInput,
  PreToolUseInput,
  PreToolUseOutput
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import type { ReadonlyDeep, } from 'type-fest';

import {
  BLOCKING_TIMEOUT_MS,
  DISABLE_BACKGROUND_TASKS_ENV,
  disabledBackgroundTasksWarning,
  isBlockingTimeoutEnabled,
  shouldApplyBlockingTimeout,
} from './blocking-timeout.ts';
import {
  isAllowed,
  shouldSkip,
} from './validation.ts';

import {
  makeHookInputParser,
  makeJsonWriter,
  type Parser,
  type JsonWriter,
} from '../../runtime/handler-runtime.ts';

/**
 Type guard that narrows a generic tool input to {@link BashToolInput}.

 @param input - tool input to check

 @returns `true` when `input` has a string `command` property
 */
function isBashToolInput(input: Readonly<Record<string, unknown>>,): input is BashToolInput {
  return (typeof input.command) === 'string';
}

/**
 Output returned for either supported hook event.
 */
type BashOutputFilterOutput = PreToolUseOutput | SessionStartOutput;

/**
 Input accepted from the plugin's PreToolUse and SessionStart registrations.
 */
type BashOutputFilterInput = PreToolUseInput | SessionStartInput;

/**
 Handles the startup warning and Bash PreToolUse rewrites.

 SessionStart returns one warning when native background tasks are disabled while the timeout policy is enabled.
 PreToolUse composes two independent behaviors:
 the existing output-filter rewrite and the default foreground timeout.
 Native subagents retain output filtering but never receive execution-field changes.

 @param event - parsed {@link BashOutputFilterInput} event from Claude Code

 @returns startup warning, rewritten Bash input, or pass-through output

 @example
 ```ts
 bashOutputFilterHandler({ hook_event_name: 'SessionStart', source: 'startup', ... });
 ```
 */
function bashOutputFilterHandler(event: ReadonlyDeep<BashOutputFilterInput>,): BashOutputFilterOutput {
  if (event.hook_event_name === 'SessionStart') {
    if ((event.source === 'startup')
      && isBlockingTimeoutEnabled(process.env,)
      && (process.env[DISABLE_BACKGROUND_TASKS_ENV] === '1')) {
      return { systemMessage: disabledBackgroundTasksWarning, };
    }
    return {};
  }

  if ((event.tool_name
    !== 'Bash') || (!isBashToolInput(event.tool_input,)))
    return {};

  /** Narrowed Bash input refined from the event by {@link isBashToolInput}. */
  const bashInput = event.tool_input;

  /**
   Whether this main-agent call needs the default foreground timeout.
   */
  const applyBlockingTimeout = shouldApplyBlockingTimeout({
    ...(event.agent_id === undefined ? {} : { agentId: event.agent_id, }),
    ...(bashInput.run_in_background === undefined
      ? {}
      : { runInBackground: bashInput.run_in_background, }),
    ...(bashInput.timeout === undefined ? {} : { timeout: bashInput.timeout, }),
    command: bashInput.command,
    env: process.env,
  },);

  if ((!isAllowed(bashInput.command,)) || shouldSkip(bashInput.command,)) {
    if (!applyBlockingTimeout)
      return {};

    return {
      hookSpecificOutput: {
        hookEventName: 'PreToolUse',
        updatedInput: {
          ...bashInput,
          timeout: BLOCKING_TIMEOUT_MS,
        },
      },
    };
  }

  /**
   True when running from the bundled hook entry (`.mjs`); false during source-driven dev runs.
   */
  const isBuilt = import.meta.url
    .endsWith('.mjs',);
  /**
   Sibling path to the filter script chosen by build mode (`.mjs` vs `.ts`).
   */
  const filterPath = isBuilt
    ? `${import.meta.dirname}/filter.mjs`
    : `${import.meta.dirname}/filter.ts`;

  /**
   Rewritten command pipeline that streams merged output through the filter.
   */
  const wrappedCommand =
    `set -o pipefail && ${bashInput.command} 2>&1 | node ${filterPath} && true`;

  return {
    hookSpecificOutput: {
      hookEventName: 'PreToolUse',
      updatedInput: {
        ...bashInput,
        command: wrappedCommand,
        ...(applyBlockingTimeout ? { timeout: BLOCKING_TIMEOUT_MS, } : {}),
      },
    },
  };
}

/**
 Parses raw stdin as a supported hook event.
 */
const bashOutputFilterParser: Parser<BashOutputFilterInput> = makeHookInputParser<BashOutputFilterInput>();

/**
 Serializes the handler output for stdout without a trailing newline.
 */
const bashOutputFilterWriter: JsonWriter<BashOutputFilterOutput> = makeJsonWriter<BashOutputFilterOutput>();

export type { BashOutputFilterOutput, };

export {
  bashOutputFilterHandler,
  bashOutputFilterParser,
  bashOutputFilterWriter,
};
