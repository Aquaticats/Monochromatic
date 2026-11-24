/**
 Reads the script strings a command line hands to a shell through its `-c` option.

 A guard that reads only one command's own name misses what `bash -c '…'` runs,
 because the script arrives as a word rather than as parsed syntax.
 The words are read wherever the shell name stands, so a command the caller already
 unwrapped (`env bash -c '…'` reaching `bash`) hands over the same string.

 The flag is read as the cluster Bash accepts,
 so `bash -lc '…'` and `bash -o pipefail -c '…'` hand over their script too,
 and an option whose value is the next word never reads that value as the flag.

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
  'ksh',
  'mksh',
  'sh',
  'zsh',
]);

/**
 Shell options whose value is the next word and is not a flag cluster.
 */
const SHELL_OPTIONS_WITH_VALUE: ReadonlySet<string> = new Set([
  '+O',
  '+o',
  '-O',
  '-o',
]);

/**
 Letter whose presence in a flag cluster makes the shell read its next word as a script.
 */
const SCRIPT_FLAG_LETTER = 'c';

/**
 Word that ends a shell's own options.
 */
const OPTIONS_TERMINATOR = '--';

//endregion Constants

//region Payload reads

/**
 Checks whether one shell flag word selects `-c`, as in `-c` or `-lc`.
 
 @param word - shell argument word
 
 @returns whether the word is a short flag cluster containing `c`
 
 @example
 ```ts
 isShellCommandFlag('-lc'); // true
 isShellCommandFlag('-e'); // false
 ```
 */
function isShellCommandFlag(word: string,): boolean {
  return word.startsWith('-',)
    && (!word.startsWith(OPTIONS_TERMINATOR,))
    && word.includes(SCRIPT_FLAG_LETTER,);
}

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
 shellScriptPayloads({ words: ['bash', '-lc', 'bun test'], }); // ['bun test']
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
      for (let offset = 0; offset < rest.length; offset += 1) {
        /**
         Word inspected as the shell's own option or as its script source.
         */
        const option = rest[offset];
        if (option === undefined)
          break;
        if (option === OPTIONS_TERMINATOR)
          break;
        if (!option.startsWith('-',))
          break;
        if (SHELL_OPTIONS_WITH_VALUE.has(option,)) {
          offset += 1;
          continue;
        }
        if (!isShellCommandFlag(option,))
          continue;
        /**
         Script string following the `-c` flag.
         */
        const script = rest[offset + 1];
        return (script === undefined)
          ? []
          : [script,];
      }
      return [];
    },);
}

//endregion Payload reads

export {
  shellScriptPayloads,
  wordLaunchesShellScript,
};
