/**
 The word that asks a command for its usage line, and what every command of this package answers with.

 `mise run <task> -- --help` used to end in a syntax error: mise appends a word its `usage` spec does not claim to the task's
 script text, and `--help` is mise's own flag, so it was never handed to the command. The tasks now run their command as a plain
 command line, so the word arrives as an argument, and the command answers it here: the usage line on standard output, and a zero
 status, before it reads an option or touches anything.

 @module
 */

/**
 The word that asks a command for its usage line.
 */
export const HELP_WORD = '--help';

/**
 Prints a command's usage line when the caller asked for it.

 @param words - the words the command was given

 @param usage - the command's own usage line, without the `Usage: ` that is printed before it

 @returns whether the caller asked for help, and should therefore stop here

 @example
 ```ts
 if (usageWasAsked({ words: ['--help',], usage: BATCH_USAGE, })) { process.exit(0,); }
 ```
 */
export function usageWasAsked({
  words,
  usage,
}: {
  readonly words: readonly string[];
  readonly usage: string;
}): boolean {
  if (!words.includes(HELP_WORD,)) {
    return false;
  }
  // The usage line is the command's exact terminal output, which is what a reader of `--help` is asking for.
  console.log(`Usage: ${usage}`,);
  return true;
}
