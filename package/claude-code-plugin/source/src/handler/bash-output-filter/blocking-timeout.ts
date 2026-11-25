/**
 Default-on one-second foreground timeout for main-agent Bash calls.

 @module
 */

import { isDigit, } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

/**
 Environment variable that enables or disables the default foreground timeout.
 */
const BLOCKING_TIMEOUT_ENABLED_ENV = 'MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED' as const;

/**
 Environment variable whose `1` value disables Claude Code background tasks.
 */
const DISABLE_BACKGROUND_TASKS_ENV = 'CLAUDE_CODE_DISABLE_BACKGROUND_TASKS' as const;

/**
 Default foreground timeout in milliseconds, chosen by the owner's request.
 */
const BLOCKING_TIMEOUT_MS = 1_000;

/**
 Exact lowercase values that disable the timeout policy.
 */
const DISABLED_VALUES: ReadonlySet<string> = new Set([
  '0',
  'false',
  'off',
]);

/**
 Command word of the one invocation shape that stays exempt from the default timeout.
 */
const SLEEP_WORD = 'sleep' as const;

/**
 Warning shown once at startup when native background tasks and the default timeout conflict.
 */
const disabledBackgroundTasksWarning: string =
  'Claude Code background tasks are disabled while Monochromatic\'s one-second Bash timeout is enabled. '
    + 'A command that reaches its foreground timeout will stop instead of moving to the background. '
    + 'Use a normal session with background tasks enabled, '
    + `or set ${BLOCKING_TIMEOUT_ENABLED_ENV}=0, false, or off to disable the timeout policy. `
    + 'Bare mode has the same termination risk without this warning.';

/**
 Whether the default timeout policy is enabled.

 @param env - process environment holding {@link BLOCKING_TIMEOUT_ENABLED_ENV}

 @returns `false` only for exact lowercase `0`, `false`, or `off`

 @example
 ```ts
 isBlockingTimeoutEnabled({}); // true
 isBlockingTimeoutEnabled({ MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED: 'off' }); // false
 ```
 */
function isBlockingTimeoutEnabled(env: Readonly<NodeJS.ProcessEnv>,): boolean {
  /** Caller-provided policy value, where absence enables the policy. */
  const value = env[BLOCKING_TIMEOUT_ENABLED_ENV];
  return (value === undefined) || (!DISABLED_VALUES.has(value,));
}

/**
 Whether `text` is a plain decimal number: digits with at most one dot, and at least one digit.

 @param text - candidate sleep duration

 @returns whether `text` is digits with an optional single dot
 */
function isPlainNumber(text: string,): boolean {
  /** Dot-separated runs; a plain number has one or two. */
  const parts = text.split('.',);
  if (parts.length > 2)
    return false;

  /** Characters left after removing the optional dot; all must be digits. */
  const digits = parts.join('',);
  if (digits.length === 0)
    return false;

  for (const character of digits) {
    if (!isDigit(character,))
      return false;
  }
  return true;
}

/**
 Whether `command` is exactly `sleep <number>`, with no chaining, redirects, or extra arguments.

 @param command - Bash command text as sent by the model

 @returns whether the whole command is a bare numeric sleep

 @example
 ```ts
 isPlainSleep('sleep 30'); // true
 isPlainSleep('sleep 30 && ls'); // false
 ```
 */
function isPlainSleep(command: string,): boolean {
  /** Command text without surrounding whitespace. */
  const trimmed = command.trim();
  if (!trimmed.startsWith(SLEEP_WORD,))
    return false;

  /** Text after the command word; whitespace here excludes lookalike words. */
  const rest = trimmed.slice(SLEEP_WORD.length,);
  if (rest.trim() === rest)
    return false;

  return isPlainNumber(rest.trim(),);
}

/**
 Whether a Bash call should receive the default foreground timeout.

 Native subagent calls are excluded because the policy is main-agent only.
 Explicit background calls preserve their mode and timeout semantics.
 Caller-supplied timeouts and bare numeric sleeps preserve native handling.

 @param agentId - `agent_id` from the hook input, absent on the main thread

 @param command - Bash command text as sent by the model

 @param env - process environment holding the policy variable

 @param runInBackground - caller's requested background mode

 @param timeout - caller's timeout, with `undefined` meaning absent

 @returns `true` when the hook should supply {@link BLOCKING_TIMEOUT_MS}

 @example
 ```ts
 shouldApplyBlockingTimeout({ command: 'ls', env: {} }); // true
 shouldApplyBlockingTimeout({ command: 'ls', env: {}, timeout: 500 }); // false
 ```
 */
function shouldApplyBlockingTimeout(
  {
    agentId,
    command,
    env,
    runInBackground,
    timeout,
  }: Readonly<{
    agentId?: string;
    command: string;
    env: Readonly<NodeJS.ProcessEnv>;
    runInBackground?: boolean;
    timeout?: number;
  }>,
): boolean {
  if (!isBlockingTimeoutEnabled(env,))
    return false;
  if (agentId !== undefined)
    return false;
  if (runInBackground === true)
    return false;
  if (timeout !== undefined)
    return false;
  return !isPlainSleep(command,);
}

export {
  BLOCKING_TIMEOUT_ENABLED_ENV,
  BLOCKING_TIMEOUT_MS,
  DISABLE_BACKGROUND_TASKS_ENV,
  disabledBackgroundTasksWarning,
  isBlockingTimeoutEnabled,
  isPlainSleep,
  shouldApplyBlockingTimeout,
};
