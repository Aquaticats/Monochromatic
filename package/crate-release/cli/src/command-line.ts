import { shellQuote } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/shell-quote.ts';
import { usesOnly } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/character-set.ts';

import { shellPlainCharacters } from './constants.ts';

/**
 Write one word of a printed command: bare when a shell reads it literally anyway, single-quoted otherwise.

 @param word - program name or argument

 @returns text a POSIX shell expands back to exactly the word

 @throws Error when the word holds a NUL character, which no command line can carry

 @example
 ```ts
 formatWord('it\'s');
 ```
 */
function formatWord(word: string): string {
  return usesOnly({
    text: word,
    allowed: shellPlainCharacters
  }) ? word : shellQuote(word);
}

/**
 Format a program call as one line that can be pasted into a POSIX shell and runs the same call.

 The release helpers never run this text: they start programs with an argument list, so no shell reads anything.
 The line exists for people, who copy a failed step from the log, and it keeps its meaning only because every word
 that is not plainly safe goes through the shared `shellQuote`.

 @param program - program name or path

 @param args - arguments, each one word

 @returns the command line

 @throws Error when a word holds a NUL character

 @example
 ```ts
 formatCommandLine({ program: 'git', args: ['commit', '--message', 'a b'] });
 ```
 */
export function formatCommandLine({
  program,
  args,
}: {
  readonly program: string;
  readonly args: readonly string[];
}): string {
  return [
    program,
    ...args
  ]
    .map(formatWord)
    .join(' ');
}
