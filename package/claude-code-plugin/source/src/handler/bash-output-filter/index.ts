import type {
  SessionStartInput,
  SessionStartOutput,
  BashToolInput,
  PreToolUseInput,
  PreToolUseOutput
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import type { ReadonlyDeep, } from 'type-fest';

import {
  canMoveToBackground,
  clampedTimeout,
  DISABLE_BACKGROUND_TASKS_ENV,
  disabledBackgroundTasksWarning,
  isBlockingTimeoutEnabled,
  isTimeoutPolicyCall,
  withSleepPrefix,
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
 the output-filter rewrite and the timeout policy.
 For a main-agent foreground call the policy clamps the timeout to at most one second
 and,
 when the command runs unwrapped,
 adds a no-op prefix to a leading `sleep` so a timeout moves it instead of stopping it;
 a filter-wrapped command already starts with `set -o pipefail`.
 Native subagents and explicit background calls retain output filtering but never receive execution or prefix changes.

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

  /**
   Narrowed Bash input refined from the event by {@link isBashToolInput}.
   */
  const bashInput = event.tool_input;

  /**
   Whether the timeout policy changes this call: a main-agent foreground call with the policy enabled.
   */
  const policyCall = isTimeoutPolicyCall({
    ...(event.agent_id === undefined ? {} : { agentId: event.agent_id, }),
    ...(bashInput.run_in_background === undefined
      ? {}
      : { runInBackground: bashInput.run_in_background, }),
    env: process.env,
  },);

  if ((!isAllowed(bashInput.command,)) || shouldSkip(bashInput.command,)) {
    if (!policyCall)
      return {};

    /**
     Unwrapped command text, prefixed when Claude Code would see a leading `sleep`.
     */
    const command = canMoveToBackground(process.env,)
      ? withSleepPrefix(bashInput.command,)
      : bashInput.command;
    /**
     Timeout after the clamp.
     */
    const timeout = clampedTimeout({
      ...(bashInput.timeout === undefined ? {} : { timeout: bashInput.timeout, }),
      env: process.env,
    },);
    if ((command === bashInput.command) && (timeout === bashInput.timeout))
      return {};

    return {
      hookSpecificOutput: {
        hookEventName: 'PreToolUse',
        updatedInput: {
          ...bashInput,
          command,
          timeout,
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
        ...(policyCall
          ? {
            timeout: clampedTimeout({
              ...(bashInput.timeout === undefined ? {} : { timeout: bashInput.timeout, }),
              env: process.env,
            },),
          }
          : {}),
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
