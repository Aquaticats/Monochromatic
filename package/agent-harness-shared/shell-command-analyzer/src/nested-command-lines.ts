/**
 Reads the command lines a program runs from among its own arguments.
 
 `find . -exec /usr/bin/git commit {} \;` names Git between `-exec` and the
 `;` that ends that command,
 and `find` runs it with every `{}` replaced by a matching path.
 A guard that reads only `find` sees a search, not a commit.
 
 Each such command is reported as one shell source line naming exactly those
 words, so the caller reads it the way it reads any other command line.
 
 @module
 */

import { lastPathSegment, } from './executable-path-words.ts';
import {
  NESTED_COMMAND_PROGRAMS,
  type NestedCommandProgram,
} from './nested-command-programs.ts';
import { shellSourceLine, } from './shell-source-words.ts';

//region Nested command lines

/**
 Reads one command the program named by `words` was given in its arguments.
 
 @param argumentWords - program arguments, without its name
 
 @param commandOptionIndex - offset of the option that introduces a command
 
 @param program - program whose option introduced the command
 
 @returns the words between that option and its terminator
 
 @example
 ```ts
 // arguments of `find . -exec ls -l {} \; -print`, with the option at offset 1
 commandWordsAfterOption({ argumentWords, commandOptionIndex: 1, program }); // ['ls', '-l', '{}']
 ```
 */
function commandWordsAfterOption({
  argumentWords,
  commandOptionIndex,
  program,
}: {
  readonly argumentWords: readonly string[];
  readonly commandOptionIndex: number;
  readonly program: NestedCommandProgram;
},): readonly string[] {
  /**
   Option that introduced the command, whose own grammar settles its terms.
   */
  const option = argumentWords[commandOptionIndex] ?? '';
  /**
   Words after the option, which run to the end when no terminator follows.
   */
  const afterOption = argumentWords.slice(commandOptionIndex + 1,);
  /**
   Whether this option has a batching form at all, as `find -ok` does not.
   */
  const batches = program
    .batchingCommandOptions
    .has(option,);
  /**
   Offset of the word ending the command, or -1 when the arguments hold none.
   */
  const terminatorIndex = afterOption.findIndex(function endsCommand(
    word,
    offset,
  ): boolean {
    if (program
      .terminators
      .has(word,))
      return true;
    return batches
      && (word === program.batchingTerminator)
      && ((afterOption[offset - 1] ?? '') === program.batchingPlaceholder);
  },);
  return terminatorIndex === (-1)
    ? afterOption
    : afterOption.slice(
      0,
      terminatorIndex,
    );
}

/**
 Collects every command line a nested-command program was given in its arguments.
 
 @param words - command name followed by its arguments
 
 @returns one shell source line per command the program runs
 
 @example
 ```ts
 nestedCommandLines({ words: ['find', '.', '-exec', '/usr/bin/git', 'push', ';'], });
 // [`'/usr/bin/git' 'push'`]
 ```
 */
function nestedCommandLines({ words, }: { readonly words: readonly string[]; },): readonly string[] {
  /**
   Command word that may name a program running commands from its arguments.
   */
  const [name,] = words;
  if (name === undefined)
    return [];

  /**
   Program specification matching the command word, when it is one.
   */
  const program = NESTED_COMMAND_PROGRAMS
    .find(function programNamed(candidate,): boolean {
      return candidate
        .names
        .has(lastPathSegment(name,),);
    },);
  if (program === undefined)
    return [];

  /**
   Program arguments, without the program name.
   */
  const argumentWords = words.slice(1,);
  /**
   Command lines the arguments name, in the order the program reads them.
   */
  const commandLines: string[] = [];
  for (let index = 0; index < argumentWords.length; index += 1) {
    /**
     Argument inspected as an option that introduces a command.
     */
    const word = argumentWords[index];
    if ((word === undefined)
      || (!program
        .commandOptions
        .has(word,)))
      continue;

    /**
     Words of the command that option introduces.
     */
    const commandWords = commandWordsAfterOption({
      argumentWords,
      commandOptionIndex: index,
      program,
    },);
    if (commandWords.length > 0)
      commandLines
        .push(shellSourceLine(commandWords,),);
    index += commandWords.length;
  }
  return commandLines;
}

//endregion Nested command lines

export { nestedCommandLines, };
