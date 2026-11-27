/**
 The command line every ZFS guest driver of this package reads.

 Each driver takes one step word and the same flags. Before this module held the parsing, each of them carried its own
 copy of it: the step, `--name` for the guest, `--log-dir` for the folder every log is written to, and `--keep-guest`
 for a run that should leave the guest it made for inspection. A flag with no word after it kept the default, and a
 flag no driver knows was ignored. Both of those stay true here, so a run of the four drivers reads its command line
 the way it always did.
 */

/**
 The words before the first one this module reads belong to the interpreter: the program, then the script.
 */
const FIRST_COMMAND_LINE_WORD = 2;

/**
 How far past a flag the word it was given stands.
 */
const DISTANCE_TO_A_FLAGS_WORD = 1;

/**
 The step a driver performs and the values its command line gave it.
 */
export type GuestRouteOptions = {
  /**
   The step the run performs, empty when the command line named no step.
   */
  readonly step: string;
  /**
   The guest this run creates, prepares or execs in.
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
 Read the step and the flags of one driver's command line.

 @param argv - the whole command line, the interpreter's own words included

 @param defaultName - the guest name a line that names none asks for

 @param defaultLogDir - the log folder a line that names none asks for

 @returns the step and the values the line gave it

 @example
 ```ts
 const chosen = guestRouteOptions({
   argv: process.argv,
   defaultName: 'forced-act',
   defaultLogDir: resolve(import.meta.dirname, '..', 'target', 'forced-act-guest-logs'),
 });
 ```
 */
export function guestRouteOptions({
  argv,
  defaultName,
  defaultLogDir,
}: {
  readonly argv: readonly string[];
  readonly defaultName: string;
  readonly defaultLogDir: string;
},): GuestRouteOptions {
  /**
   The step word, and every flag that followed it.
   */
  const [step = '', ...flags] = argv
    .slice(FIRST_COMMAND_LINE_WORD,);

  /**
   The word a flag was given, or the default when the line gave the flag none.

   A line that ends at its flag gives no word, and one that gives the flag an empty word gives that empty word: the
   drivers read the word after the flag itself, so both answers stay what they were.

   @param flag - the flag to look for

   @param fallback - the value a line that gave no word asks for

   @returns the word the flag was given, or the fallback

   @example
   ```ts
   const name = valueOf({ flag: '--name', fallback: 'forced-act' });
   ```
   */
  function valueOf({
    flag,
    fallback,
  }: {
    readonly flag: string;
    readonly fallback: string;
  },): string {
    /**
     Where the flag stands among the words after the step.
     */
    const at = flags
      .indexOf(flag,);
    if (at === (-1))
      return fallback;
    /**
     The word after the flag, absent when the line ended at the flag.
     */
    const following = flags[at + DISTANCE_TO_A_FLAGS_WORD];
    return following ?? fallback;
  }

  return {
    step,
    name: valueOf({
      flag: '--name',
      fallback: defaultName,
    },),
    logDir: valueOf({
      flag: '--log-dir',
      fallback: defaultLogDir,
    },),
    keepGuest: flags
      .includes('--keep-guest',),
  };
}
