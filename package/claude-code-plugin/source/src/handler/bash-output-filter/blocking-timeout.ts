/**
 One-second foreground timeout, caller-timeout clamp, sleep prefix, and startup-warning policy
 for main-agent Bash calls.

 Claude Code moves a foreground command to the background when it reaches its timeout,
 unless the command starts with `sleep`, which it stops instead.
 This module decides which calls get the one-second timeout
 and which command text needs a no-op prefix so that Claude Code moves it rather than stopping it.
 Decision: `doc/decision/claude-code-bash-blocking-timeout.md`.

 @module
 */

//region Timeout and startup-warning policy
// Owns environment switches, the fixed timeout, and the warning text.

/**
 Environment variable that enables or disables the timeout policy.
 */
const BLOCKING_TIMEOUT_ENABLED_ENV = 'MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED' as const;

/**
 Environment variable whose `1` value disables Claude Code background tasks.
 */
const DISABLE_BACKGROUND_TASKS_ENV = 'CLAUDE_CODE_DISABLE_BACKGROUND_TASKS' as const;

/**
 Foreground timeout in milliseconds, chosen by the owner's request;
 also the largest caller timeout the policy leaves in place.
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
 Warning shown once at startup when native background tasks and the timeout policy conflict.
 */
const disabledBackgroundTasksWarning: string =
  'Claude Code background tasks are disabled while Monochromatic\'s one-second Bash timeout is enabled. '
    + 'A command that reaches its foreground timeout will stop instead of moving to the background. '
    + 'Use a normal session with background tasks enabled, '
    + `or set ${BLOCKING_TIMEOUT_ENABLED_ENV}=0, false, or off to disable the timeout policy. `
    + 'Bare mode has the same termination risk without this warning.';

/**
 Whether the timeout policy is enabled.

 @param env - process environment holding {@link BLOCKING_TIMEOUT_ENABLED_ENV}

 @returns `false` only for exact lowercase `0`, `false`, or `off`

 @example
 ```ts
 isBlockingTimeoutEnabled({}); // true
 isBlockingTimeoutEnabled({ MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED: 'off' }); // false
 ```
 */
function isBlockingTimeoutEnabled(env: Readonly<NodeJS.ProcessEnv>,): boolean {
  /**
   Caller-provided policy value, where absence enables the policy.
   */
  const value = env[BLOCKING_TIMEOUT_ENABLED_ENV];
  return (value === undefined) || (!DISABLED_VALUES.has(value,));
}

//endregion

//region Policy eligibility and timeout clamp
// Decides which calls the policy changes and what timeout they carry.

/**
 Whether the timeout policy changes this Bash call's execution.

 Native subagent calls are excluded because the policy is main-agent only.
 Explicit background calls are excluded because their `timeout` is a background lifetime,
 not a foreground wait.

 @param agentId - `agent_id` from the hook input, absent on the main thread

 @param env - process environment holding the policy variable

 @param runInBackground - caller's requested background mode

 @returns `true` when the hook should clamp the timeout and prefix a leading `sleep`

 @example
 ```ts
 isTimeoutPolicyCall({ env: {} }); // true
 isTimeoutPolicyCall({ agentId: 'agent-1', env: {} }); // false
 ```
 */
function isTimeoutPolicyCall(
  {
    agentId,
    env,
    runInBackground,
  }: Readonly<{
    agentId?: string;
    env: Readonly<NodeJS.ProcessEnv>;
    runInBackground?: boolean;
  }>,
): boolean {
  if (!isBlockingTimeoutEnabled(env,))
    return false;
  if (agentId !== undefined)
    return false;
  return runInBackground !== true;
}

/**
 Foreground timeout a policy call carries after the clamp.

 A caller value from just above zero up to {@link BLOCKING_TIMEOUT_MS} survives,
 since such a call already moves no later than the default.
 Every other value becomes the default:
 an absent timeout,
 a longer one,
 and zero,
 which Claude Code 2.1.332 treated as no early move (an 8-second command with `timeout: 0` waited in front).

 @param timeout - caller's timeout, with `undefined` meaning absent

 @returns timeout in milliseconds to send to Claude Code

 @example
 ```ts
 clampedTimeout(undefined); // 1000
 clampedTimeout(3_480_000); // 1000
 clampedTimeout(500); // 500
 ```
 */
function clampedTimeout(timeout?: number,): number {
  if ((timeout !== undefined) && (timeout > 0)
    && (timeout <= BLOCKING_TIMEOUT_MS))
    return timeout;
  return BLOCKING_TIMEOUT_MS;
}

//endregion

//region Leading-sleep prefix
// Mirrors Claude Code's own leading-sleep check so a timed-out sleep moves instead of stopping.

/**
 Command word Claude Code stops instead of moving to the background.
 */
const SLEEP_WORD = 'sleep' as const;

/**
 No-op prefix that makes the first command `true`,
 so Claude Code moves the command to the background at its timeout.
 Each operator keeps its meaning:
 `true && a; b`,
 `true && a & b`,
 and `true && a | b` run as the unprefixed forms do.
 */
const SLEEP_PREFIX = 'true && ' as const;

/**
 Characters that end a command word in Claude Code's subcommand split
 or in shell syntax next to it.
 */
const WORD_END_CHARACTERS: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
  '\r',
  ';',
  '&',
  '|',
  '<',
  '>',
  '(',
  ')',
]);

/**
 Characters skipped before the first command word.
 */
const LEADING_SPACE_CHARACTERS: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
  '\r',
]);

/**
 Index of the first character after leading blanks and whole comment lines.

 Claude Code 2.1.332 stopped `# note\nsleep 5 && ...` at its timeout,
 so a comment line does not hide a leading `sleep` from it.

 @param command - Bash command text as sent by the model

 @returns index of the first command character, or `command.length` when none remains
 */
function firstCommandIndex(command: string,): number {
  /**
   Scan position; advances over blanks and comment lines only.
   */
  let index = 0;
  while (index < command.length) {
    /**
     Character at the scan position.
     */
    const character = command.charAt(index,);
    if (LEADING_SPACE_CHARACTERS.has(character,)) {
      index += 1;
      continue;
    }
    if (character !== '#')
      return index;
    /**
     End of the comment line, or -1 when the comment runs to the end.
     */
    const lineEnd = command.indexOf(
      '\n',
      index,
    );
    if (lineEnd === (-1))
      return command.length;
    index = lineEnd + 1;
  }
  return index;
}

/**
 Whether Claude Code would see `command` as starting with `sleep`.

 Claude Code 2.1.332 takes the first word of the first subcommand
 and stops the command at its timeout when that word is `sleep`.
 Probes stopped `sleep 5 && x`,
 `sleep 5;x`,
 and a `sleep` after a comment line,
 and moved `X=1 sleep 5`,
 `{ sleep 5; }`,
 and `(sleep 5)`.
 This check also matches a few shapes Claude Code would move anyway;
 prefixing those changes nothing.

 @param command - Bash command text as sent by the model

 @returns whether the first command word is `sleep`

 @example
 ```ts
 startsWithSleepWord('sleep 5 && make'); // true
 startsWithSleepWord('sleeper 5'); // false
 ```
 */
function startsWithSleepWord(command: string,): boolean {
  /**
   Index where the first command word begins.
   */
  const start = firstCommandIndex(command,);
  if (!command.startsWith(
    SLEEP_WORD,
    start,
  ))
    return false;
  /**
   Character after the candidate word; empty at the end of the text.
   */
  const next = command.charAt(start + SLEEP_WORD.length,);
  return (next === '') || WORD_END_CHARACTERS.has(next,);
}

/**
 Command text with {@link SLEEP_PREFIX} added when Claude Code would see a leading `sleep`.

 @param command - Bash command text that Claude Code will run as given

 @returns command unchanged, or prefixed so a timeout moves it instead of stopping it

 @example
 ```ts
 withSleepPrefix('sleep 300 && check > out'); // 'true && sleep 300 && check > out'
 withSleepPrefix('make test'); // 'make test'
 ```
 */
function withSleepPrefix(command: string,): string {
  return startsWithSleepWord(command,) ? `${SLEEP_PREFIX}${command}` : command;
}

//endregion

export {
  BLOCKING_TIMEOUT_ENABLED_ENV,
  BLOCKING_TIMEOUT_MS,
  clampedTimeout,
  DISABLE_BACKGROUND_TASKS_ENV,
  disabledBackgroundTasksWarning,
  isBlockingTimeoutEnabled,
  isTimeoutPolicyCall,
  SLEEP_PREFIX,
  startsWithSleepWord,
  withSleepPrefix,
};
