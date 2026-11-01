/**
 Default-on switch that makes the main agent's Bash calls run as background tasks.

 @module
 */

import { isDigit, } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

/**
 Environment variable that opts out of forcing `run_in_background` on main-agent Bash calls.
 */
const FORCE_BACKGROUND_ENV = 'MONOCHROMATIC_BASH_FORCE_BACKGROUND' as const;

/**
 Value of {@link FORCE_BACKGROUND_ENV} that disables forcing; every other value, or absence, forces background.
 */
const FORCE_BACKGROUND_OFF = 'off' as const;

/**
 Command word of the one invocation shape that stays in the foreground.
 */
const SLEEP_WORD = 'sleep' as const;

/**
 Whether `text` is a plain decimal number: digits with at most one dot, and at least one digit.

 @param text - candidate sleep duration

 @returns whether `text` is digits with an optional single dot

 @example
 ```ts
 isPlainNumber('30');   // true
 isPlainNumber('0.5');  // true
 isPlainNumber('1m');   // false
 isPlainNumber('.');    // false
 ```
 */
function isPlainNumber(text: string,): boolean {
  /**
   Dot-separated runs; a plain number has one or two.
   */
  const parts = text.split('.',);
  if (parts.length > 2)
    return false;

  /**
   Characters left after removing the optional dot; all must be digits.
   */
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
 isPlainSleep('sleep 30');          // true
 isPlainSleep('sleep 30 && ls');    // false
 isPlainSleep('sleep 1m');          // false
 ```
 */
function isPlainSleep(command: string,): boolean {
  /**
   Command text without surrounding whitespace.
   */
  const trimmed = command.trim();
  if (!trimmed.startsWith(SLEEP_WORD,))
    return false;

  /**
   Text after the command word; must start with whitespace so `sleeper` does not match.
   */
  const rest = trimmed.slice(SLEEP_WORD.length,);
  if (rest.trim() === rest)
    return false;

  return isPlainNumber(rest.trim(),);
}

/**
 Whether a Bash call should be rewritten to run in the background.

 Subagent calls are excluded: the hook input carries `agent_id` only inside a subagent, and a subagent
 that waits on a command's output would otherwise lose its synchronous result.
 Bare `sleep <number>` commands are excluded too, since backgrounding a pure delay returns nothing to wait on.

 @param agentId - `agent_id` from the hook input, absent on the main thread

 @param command - Bash command text as sent by the model

 @param env - process environment holding {@link FORCE_BACKGROUND_ENV}

 @returns `true` for a main-thread, non-sleep call unless the opt-out is set

 @example
 ```ts
 shouldForceBackground({ command: 'ls', env: {} });                                            // true
 shouldForceBackground({ command: 'sleep 5', env: {} });                                       // false
 shouldForceBackground({ command: 'ls', env: { MONOCHROMATIC_BASH_FORCE_BACKGROUND: 'off' } }); // false
 shouldForceBackground({ agentId: 'a1', command: 'ls', env: {} });                             // false
 ```
 */
function shouldForceBackground(
  {
    agentId,
    command,
    env,
  }: Readonly<{
    agentId?: string;
    command: string;
    env: Readonly<NodeJS.ProcessEnv>;
  }>,
): boolean {
  if (agentId !== undefined)
    return false;
  if (isPlainSleep(command,))
    return false;
  return env[FORCE_BACKGROUND_ENV] !== FORCE_BACKGROUND_OFF;
}

export {
  FORCE_BACKGROUND_ENV,
  isPlainSleep,
  shouldForceBackground,
};
