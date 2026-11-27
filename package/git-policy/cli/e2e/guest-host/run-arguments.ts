/**
 The words one guest-driving script's command line gave it.

 Every driver in this folder takes a step as its first word and then flags, so
 one reader serves them all: the step, the word each flag-shaped word is
 followed by, and every word after the step, which a bare flag is looked up in.

 The lookup answers as the drivers always did: the first occurrence of a flag
 wins, and a flag given as the last word has no value, so the caller's own
 default stands.

 @example
 ```ts
 const { step, values, flags } = runArguments({ argv: ['run', '--name', 'guest-one'] });
 ```
 */

/**
 The prefix that makes a word a flag rather than a value.
 */
const flagPrefix = '--';

/**
 One command line, split the way every driver in this folder reads it.
 */
export type RunArguments = {
  /**
   The step the command names, which is the first word after the script's path
   and is empty text when the command gives none.
   */
  readonly step: string;
  /**
   The word that follows each flag-shaped word, with the first occurrence of a
   flag winning.
   */
  readonly values: ReadonlyMap<string, string>;
  /**
   Every word after the step, so a bare flag is one lookup and no order matters.
   */
  readonly flags: ReadonlySet<string>;
};

/**
 Read one command line into the step, the flag values and the bare flags.

 @param argv - the arguments after the script's path

 @returns the step, each flag's value and every word after the step

 @example
 ```ts
 runArguments({ argv: ['all', '--keep-guest'] });
 ```
 */
export function runArguments({
  argv,
}: {
  readonly argv: readonly string[];
},): RunArguments {
  /**
   The step, which is the first word, and every word after it.
   */
  const [step = '', ...afterStep] = argv;
  /**
   The values the command line gave each flag, first occurrence winning.
   */
  const values = new Map<string, string>();
  for (const [at, word] of argv.entries()) {
    /**
     The word after this one, which is this flag's value when it is not a flag.
     */
    const value = argv[at + 1];
    /**
     Whether this word is a flag at all.
     */
    const isFlag = word.startsWith(flagPrefix,);
    /**
     Whether a word follows it to be its value.
     */
    const hasValue = value !== undefined;
    /**
     Whether this is the flag's first appearance, which is the one the drivers
     have always read.
     */
    const isFirst = !values.has(word,);
    if (isFlag && (hasValue && isFirst))
      values.set(
        word,
        value,
      );
  }
  return {
    step,
    values,
    flags: new Set(afterStep,),
  };
}
