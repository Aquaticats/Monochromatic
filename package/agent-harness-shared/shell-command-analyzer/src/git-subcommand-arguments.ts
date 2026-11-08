/**
 Splits one Git subcommand invocation into option words and positional words.
 
 Every mutation decision below reads positionals,
 so an option value such as the commit named by `--contains` never counts as a branch name.
 
 @module
 */

//region Types

/**
 Positional and option words of one Git subcommand invocation.
 */
type GitArgumentScan = {
  /**
   Words that name revisions, paths, or subcommand verbs.
   */
  readonly positionals: readonly string[];
  /**
   Option words seen before a `--` terminator.
   */
  readonly options: readonly string[];
};

//endregion Types

//region Scanning

/**
 Separates option words from positional words, dropping values that belong to options.
 
 @param args - subcommand arguments in source order
 
 @param optionsWithValue - option words that consume the next word as their value
 
 @returns positional and option words
 
 @example
 ```ts
 scanGitArguments({ args: ['--contains', 'HEAD', 'topic'], optionsWithValue });
// { positionals: ['topic'], options: ['--contains'] }
 ```
 */
function scanGitArguments({
  args,
  optionsWithValue,
}: {
  readonly args: readonly string[];
  readonly optionsWithValue: ReadonlySet<string>;
},): GitArgumentScan {
  /**
   Offset of the `--` terminator, or -1 when arguments carry none.
   */
  const terminatorIndex = args.indexOf('--',);
  /**
   Words still parsed as options and their values.
   */
  const optionWords = terminatorIndex === (-1)
    ? args
    : args.slice(
      0,
      terminatorIndex,
    );
  /**
   Words after the terminator, which are positionals even when they start with `-`.
   */
  const trailingPositionals = terminatorIndex === (-1)
    ? []
    : args.slice(terminatorIndex + 1,);
  /**
   Positional words in source order.
   */
  const positionals: string[] = [];
  /**
   Option words in source order.
   */
  const options: string[] = [];

  for (let index = 0; index < optionWords.length; index += 1) {
    /**
     Argument word being classified.
     */
    const word = optionWords[index];
    if (word === undefined)
      break;
    if (!word.startsWith('-',)) {
      positionals.push(word,);
      continue;
    }

    options.push(word,);
    if (optionsWithValue.has(word,))
      index += 1;
  }

  return {
    positionals: [
      ...positionals,
      ...trailingPositionals,
    ],
    options,
  };
}

/**
 Reports whether any option word names one of the given options.
 
 Accepts the `--name=value` spelling so an attached value still counts as that option.
 
 @param words - option words to inspect
 
 @param names - option names to look for
 
 @returns whether one of the names appears
 
 @example
 ```ts
 hasAnyOption({ words: ['--delete'], names: new Set(['-d', '--delete']) }); // true
 ```
 */
function hasAnyOption({
  words,
  names,
}: {
  readonly words: readonly string[];
  readonly names: ReadonlySet<string>;
},): boolean {
  for (const name of names) {
    /**
     Whether one option word names the current option.
     */
    const matched = words
      .some(function optionMatches(word,): boolean {
        return (word === name) || word.startsWith(`${name}=`,);
      },);
    if (matched)
      return true;
  }
  return false;
}

//endregion Scanning

export type { GitArgumentScan, };

export {
  hasAnyOption,
  scanGitArguments,
};
