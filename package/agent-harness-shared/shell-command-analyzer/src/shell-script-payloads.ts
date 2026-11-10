/**
 Reads the script strings a command line hands to a shell through its `-c` option.

 A guard that reads only one command's own name misses what `bash -c '…'` runs,
 because the script arrives as a word rather than as parsed syntax.
 The words are read wherever the shell name stands, so a command the caller already
 unwrapped (`env bash -c '…'` reaching `bash`) hands over the same string.

 @module
 */

import { lastPathSegment, } from './executable-path-words.ts';

//region Constants

/**
 Shell programs whose `-c` argument is a script string, whatever path spells them.
 */
const SHELL_NAMES: ReadonlySet<string> = new Set([
  'bash',
  'dash',
  'sh',
  'zsh',
],);

/**
 Option word that hands a shell its script string as the next word.
 */
const SCRIPT_OPTION = '-c';

//endregion Constants

//region Payload reads

/**
 Checks whether one command word launches a shell whose `-c` argument is a script string.

 A caller that reads a raw word run needs this answer on its own,
 because only the program word launches a shell there:
 `echo bash -c 'bun test'` prints text rather than running a script.

 @param word - command word, from parsed syntax or from a raw word run

 @returns whether the word names a shell, whatever path spells it

 @example
 ```ts
 wordLaunchesShellScript('/bin/bash'); // true
 wordLaunchesShellScript('echo'); // false
 ```
 */
function wordLaunchesShellScript(word: string,): boolean {
  return SHELL_NAMES.has(lastPathSegment(word,),);
}

/**
 Collects the `-c` script strings a command line passes to a shell.
 
 @param words - program word followed by its arguments, with program wrappers already stripped
 
 @returns script strings the launched shells execute, in source order
 
 @example
 ```ts
 shellScriptPayloads({ words: ['bash', '-c', 'bun test'], }); // ['bun test']
 ```
 */
function shellScriptPayloads({ words, }: { readonly words: readonly string[]; },): readonly string[] {
  return words
    .flatMap(function scriptAfterShellOption(
      word,
      index,
    ): readonly string[] {
      if (!wordLaunchesShellScript(word,))
        return [];

      /**
       Words after the shell name.
       */
      const rest = words.slice(index + 1,);
      /**
       Position of the script option within `rest`.
       */
      const optionIndex = rest.indexOf(SCRIPT_OPTION,);
      if (optionIndex === (-1))
        return [];

      /**
       Word the option hands to the shell as its script.
       */
      const script = rest[optionIndex + 1];

      return (script === undefined)
        ? []
        : [script,];
    },);
}

//endregion Payload reads

export {
  shellScriptPayloads,
  wordLaunchesShellScript,
};
