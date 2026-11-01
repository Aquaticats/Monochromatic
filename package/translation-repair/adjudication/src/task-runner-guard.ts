import {
  isCredentialName,
  rendersAsNothing,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Task runner guard
// A BUILT COMMAND OF THIS PACKAGE REFUSES TO RUN IN A PROCESS THAT HOLDS
// PROVIDER KEYS UNLESS THAT PROCESS NAMES THE COMMAND AS THE ONE IT MEANS TO
// START. Twice in one day an agent started a built command by hand from a
// shell that held the operator's real keys, and one of those starts bought
// model calls before it was stopped. A rule written for agents is not a guard,
// so the command itself refuses.
//
// WHAT COUNTS AS A KEY: a variable whose name ends in `_API_KEY` (the same
// name test the child environments use, `isCredentialName`) and whose value is
// not empty. A name that merely contains `_API_KEY` elsewhere is no key, and a
// variable that is absent or empty is none either, so a process that holds no
// key runs as it always did, marker or no marker.
//
// WHAT NAMES THE COMMAND: the variable `TRANSLATION_REPAIR_STARTED_BY`, set to
// the command's own name. The task of each command sets it (`mise.toml`), the
// test world's one child fixture sets it for the command it starts, and an
// operator who must start a built file without its task sets it by hand for
// that one start: every task builds the package first, and nothing may rebuild
// while a pass is running (`doc/runbook/translation-repair-corpus-pass.md`).
// A marker naming another command does not start this one.
//
// WHAT THE GUARD IS FOR, AND WHAT IT IS NOT: it stops the start nobody meant,
// a command run by hand from a shell that happens to hold keys. It is no
// access control. Whoever can set a variable can set this one, and setting it
// is the statement the guard asks for.
//
// PURE: the environment and the command's name come in, a verdict goes out.
// What only a real process has, its environment, is handed in by the entry
// file through `reportingRefusals`, which turns a refusal into the stated
// refusal every entry already ends in (exit 6).

/**
 Name of the variable that names the command a process means to start: a
 package task sets it to the name of the command it starts, and so does an
 operator starting a built file without its task.
 */
export const STARTED_BY_VARIABLE = 'TRANSLATION_REPAIR_STARTED_BY';

/**
 Longest name this package may repeat when a marker names another command.
 */
const COMMAND_NAME_LIMIT = 64;

/**
 Whether one character is a lower case letter of the Latin alphabet.

 @param character - one character, empty where there was none to read

 @returns Whether it lies between a and z

 @example
 ```ts
 const letter = isLowerLetter({ character: 'p', },); // true
 ```
 */
function isLowerLetter({ character, }: { readonly character: string; },): boolean {
  return (character >= 'a') && (character <= 'z');
}

/**
 Whether a text has the shape of a command's file name: a lower case letter,
 then lower case letters, digits and hyphens, short enough that nothing but a
 name fits. One pass, so the cost is the text's length.

 @param text - value of the marker variable

 @returns Whether the package may repeat it

 @example
 ```ts
 const shaped = isCommandNameShaped({ text: 'purr-report', },); // true
 ```
 */
function isCommandNameShaped({ text, }: { readonly text: string; },): boolean {
  if (text.length > COMMAND_NAME_LIMIT)
    return false;
  // The first character must be a letter; an empty text has none, so it fails here.
  if (!isLowerLetter({ character: text.charAt(0,), },))
    return false;
  for (let at = 1; at < text.length; at += 1) {
    /**
     The character at this place, which is present because `at` is below the length.
     */
    const character = text.charAt(at,);
    /**
     Whether it is a digit.
     */
    const isDigit = (character >= '0') && (character <= '9');
    /**
     Whether it is the hyphen words of a name are joined with.
     */
    const isHyphen = character === '-';
    /**
     Whether the character may stand after the first.
     */
    const continuesName = isLowerLetter({ character, },)
      || isDigit
      || isHyphen;
    if (!continuesName)
      return false;
  }
  return true;
}

/**
 Whether a process may run a built command.
 */
export type TaskRunnerVerdict =
  | {
    /**
     The process holds no provider key, or names this command as the one it
     means to start.
     */
    readonly allowed: true;
  }
    | {
    /**
     The process holds provider keys and does not name this command.
     */
      readonly allowed: false;

    /**
     The refusal's whole text, which names no variable's value.
     */
      readonly says: string;
    };

/**
 Counts the variables of an environment that hold a provider key.

 @param env - environment of the process the command runs in

 @returns How many variables have a name ending in `_API_KEY` and a value that
 is not empty

 @example
 ```ts
 const held = countKeyVariables({ env: { WHISKER_API_KEY: 'x', }, },); // 1
 ```
 */
function countKeyVariables({ env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },): number {
  return Object
    .entries(env,)
    .filter(function holdsKey([name, value,]: ReadonlyEntry<string, Readonly<NodeJS.ProcessEnv>[string]>,): boolean {
      if (!isCredentialName({ name, },))
        return false;
      if (value === undefined)
        return false;
      return !rendersAsNothing({ text: value, },);
    },)
    .length;
}

/**
 The sentence that says the process names something other than this command,
 or nothing when the marker is empty.

 @param marker - value of the marker variable, empty where the environment has none

 @returns The sentence with its leading space, or an empty string; the marker's
 value is repeated only when it has the shape of a command name

 @example
 ```ts
 const note = anotherMarkerNote({ marker: 'purr-report', },);
 ```
 */
function anotherMarkerNote({ marker, }: { readonly marker: string; },): string {
  if (rendersAsNothing({ text: marker, },))
    return '';
  if (!isCommandNameShaped({ text: marker, },))
    return ` ${STARTED_BY_VARIABLE} is set in this process, but not to the name of a command.`;
  return ` ${STARTED_BY_VARIABLE} is set in this process to ${marker}, which names another command.`;
}

/**
 Decides whether a built command may run in a process.

 @param env - environment of the process the command runs in, handed in
 because only the entry file holds the real one

 @param command - the command's own name, which names its task

 @param owner - directory name, under `package/translation-repair/`, of the package that owns the command, whose
 `mise.toml` holds the task; handed in because only the packages that list the runners know it

 @returns Allowed when the process holds no provider key or the marker names
 this command; otherwise a refusal whose text says how many key variables the
 process holds, both ways to start the command (its task, which builds first,
 and the marker set by hand for a start that must not build), and what the
 marker holds when it holds something else

 @example
 ```ts
 const verdict = taskRunnerVerdict({ env: process.env, command: 'spend-report', owner: 'run', },);
 ```
 */
export function taskRunnerVerdict(
  {
    env,
    command,
    owner,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly command: string;
    readonly owner: string;
  },
): TaskRunnerVerdict {
  /**
   How many variables hold a provider key.
   */
  const held = countKeyVariables({ env, },);
  if (held === 0)
    return { allowed: true, };

  /**
   The marker as the environment holds it.
   */
  const marker = env[STARTED_BY_VARIABLE] ?? '';
  if (marker === command)
    return { allowed: true, };

  /**
   Noun the count takes.
   */
  const noun = wordForCount({
    count: held,
    one: 'variable',
    many: 'variables',
  },);
  /**
   Verb the count takes.
   */
  const verb = wordForCount({
    count: held,
    one: 'has',
    many: 'have',
  },);
  return {
    allowed: false,
    says: `will not run in a process that holds provider keys (${held} ${noun} whose name ends in _API_KEY ${verb} `
      + 'a value) unless that process names this command as the one it means to start. '
      + `The command's task does that: mise run //package/translation-repair/${owner}:${command} `
      + '(the task builds the package first). '
      + 'To start the built file without building, as when a pass is running and nothing may rebuild, '
      + `set ${STARTED_BY_VARIABLE}=${command} for that one start. `
      + `A process that holds no provider key needs neither.${anotherMarkerNote({ marker, },)}`,
  };
}

//endregion Task runner guard
