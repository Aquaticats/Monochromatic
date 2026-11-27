/**
 The command line every guest driver reads: one step word and the flags that follow it.

 Each driver takes one step word and the same flags. The three sets of guest helpers each held a reader of it
 (`bin/guest-route-options.ts` answering the four flags every driver shares, and `e2e/guest-host/guest-host-rules-fixture.ts`
 answering any flag as a map), and one scan answers both: {@link runArguments} reads the words, and
 {@link guestDriverOptions} is the four flags on top of it.

 The lookup answers as the drivers always did: the first occurrence of a flag wins, a flag given as the last word has no
 value so the caller's own default stands, and a flag no driver knows is answered like any other word.
 */

/**
 The prefix that makes a word a flag rather than a value.
 */
const FLAG_PREFIX = '--';

/**
 One command line, split the way every guest driver reads it.
 */
export type RunArguments = {
  /**
   The step the command names, which is the first word and is empty text when the command gives none.
   */
  readonly step: string;
  /**
   The word that follows each flag-shaped word, with the first occurrence of a flag winning.
   */
  readonly values: ReadonlyMap<string, string>;
  /**
   Every word after the step, so a bare flag is one lookup and no order matters.
   */
  readonly flags: ReadonlySet<string>;
};

/**
 The four flags every guest driver shares.
 */
export type GuestDriverOptions = {
  /**
   The step the run performs, empty when the command line named no step.
   */
  readonly step: string;
  /**
   The guest this run creates, prepares or runs a script in.
   */
  readonly name: string;
  /**
   The folder every log of this run is written to.
   */
  readonly logDir: string;
  /**
   Whether a run that made the guest keeps it for inspection instead of destroying it.
   */
  readonly keepGuest: boolean;
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
  // The step is the first word and the flags follow it, so a flag-shaped first word is the step and names nothing: a
  // driver says what it runs first.
  for (const [at, word] of afterStep.entries()) {
    /**
     The word after this one, which is this flag's value when it is not a flag.
     */
    const value = afterStep[at + 1];
    /**
     Whether this word is a flag at all.
     */
    const isFlag = word.startsWith(FLAG_PREFIX,);
    /**
     Whether a word follows it to be its value.
     */
    const hasValue = value !== undefined;
    /**
     Whether this is the flag's first appearance, which is the one the drivers have always read.
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

/**
 Read the step and the four flags every guest driver shares.

 Built on {@link runArguments}, so a flag is read the same way whichever of the two a driver asks.

 @param argv - the whole command line, the interpreter's own words included

 @param defaultName - the guest name a line that names none asks for

 @param defaultLogDir - the log folder a line that names none asks for

 @returns the step and the values the line gave it

 @example
 ```ts
 const chosen = guestDriverOptions({
   argv: process.argv.slice(2),
   defaultName: 'forced-act',
   defaultLogDir: resolve(import.meta.dirname, '..', 'target', 'forced-act-guest-logs'),
 });
 ```
 */
export function guestDriverOptions({
  argv,
  defaultName,
  defaultLogDir,
}: {
  readonly argv: readonly string[];
  readonly defaultName: string;
  readonly defaultLogDir: string;
},): GuestDriverOptions {
  /**
   The words, read the way every driver reads them.
   */
  const read: RunArguments = runArguments({ argv, },);
  return {
    step: read.step,
    name: read.values
      .get('--name',) ?? defaultName,
    logDir: read.values
      .get('--log-dir',) ?? defaultLogDir,
    keepGuest: read.flags
      .has('--keep-guest',),
  };
}
