/**
 A command's answer when this package needs its text rather than its output: the `PATH` a mise environment holds, the node that
 environment runs, or a git question whose silence would leave a run started wrongly.

 @module
 */

import type { RunCommand, } from './run-command.ts';

/**
 A command that could not answer, so the step it was asked for was not taken.
 */
export class CommandAnswerMissingError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'CommandAnswerMissingError';
}

/**
 Runs one command and answers with what it printed, trimmed.

 @param run - how a command is run, which a test replaces

 @param command - the program, found on `PATH`

 @param words - its arguments, each kept whole

 @returns what it printed, without the newline it ended with

 @throws CommandAnswerMissingError when the command could not run, or exited non-zero

 @example
 ```ts
 const path = await requiredOutputOf({ run: runCommandWithChildProcess, command: 'mise', words: ['exec', '--', 'printenv', 'PATH',], },);
 ```
 */
export async function requiredOutputOf({
  run,
  command,
  words,
  folder,
}: {
  readonly run: RunCommand;
  readonly command: string;
  readonly words: readonly string[];
  readonly folder?: string;
},): Promise<string> {
  /**
   What the command printed and how it ended.
   */
  const answer = await run({
    command,
    words,
    ...(folder === undefined ? {} : { folder, }),
  },);
  if (answer.status !== 0) {
    throw new CommandAnswerMissingError(
      `${command} ${words.join(' ',)} answered ${String(answer.status,)}: ${answer.problem
        ?? answer.stderr
          .trim()}`,
    );
  }
  return answer.stdout
    .trim();
}
