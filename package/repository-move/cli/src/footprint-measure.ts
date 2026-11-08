/**
 The measurement of the checkout's footprint with extent sharing counted once, made again when its walk loses a worktree
 that an agent removed meanwhile. `btrfs filesystem du --summarize --raw <tree>` walks the whole tree, nested subvolumes
 included (plan, section "Measurements"), and one entry that vanishes during the walk fails the whole measurement; worktrees
 come and go all day, so a measurement that lasts minutes meets that. A sum of one measurement per subvolume cannot stand
 in: every worktree shares extents with the others, so such a sum counts shared data once per worktree (the plan's
 per-worktree figure is 3753 GiB against about 264 GiB with sharing counted once), and nested subvolumes inside the main
 one would count twice. So the whole walk is made again, a bounded number of times, and only when every line `btrfs`
 printed says an entry was not there.

 @module
 */

import { MILLISECONDS_PER_SECOND, } from './constants.ts';
import { execFileAsync, } from './exec-file-async.ts';
import { childFailureOf, } from './json-text-fields.ts';

/**
 What the measurement came to: whether it succeeded, and the lines to print (the notes first, the `btrfs` table last).
 */
export type FootprintMeasurement = {
  readonly succeeded: boolean;
  readonly lines: readonly string[];
};

/**
 How many bytes of output one walk may print before it is cut: the table is one row, the warnings a few lines per
 unreadable file, so a megabyte is far above any real output.
 */
const MAX_OUTPUT_BYTES = 1_048_576;

/**
 The words that end a line of `btrfs` output when an entry was not there.
 */
const NOT_THERE = ': No such file or directory';

/**
 The name a `WARNING: cannot access '<name>': No such file or directory` line names, or nothing for any other line.

 @param line - one line of `btrfs` error output

 @returns the name, or the empty text
 */
function vanishedNameOf(line: string,): string {
  /**
   How a warning about an entry that was not there begins.
   */
  const start = "WARNING: cannot access '";
  if (!(line.startsWith(start,) && line.endsWith(`'${NOT_THERE}`,))) {
    return '';
  }
  return line.slice(
    start.length,
    line.length - `'${NOT_THERE}`.length,
  );
}

/**
 Whether a line of `btrfs` output is the closing error of a walk that lost an entry: `ERROR: cannot check space of '<tree>'`.

 @param line - one line of `btrfs` error output

 @returns true for that closing error
 */
function isClosingNotThere(line: string,): boolean {
  return line.startsWith("ERROR: cannot check space of '",) && line.endsWith(NOT_THERE,);
}

/**
 Whether the failure of a program start is the kill the deadline gives.

 @param error - the caught value

 @returns true when `execFile` killed the program at its timeout
 */
function wasKilled(error: unknown,): boolean {
  return ((typeof error) === 'object') && (error !== null)
    && ('killed' in error)
    && (error.killed === true);
}

/**
 What reading the error output of a failed walk found.
 */
type FailureReading =
  | {
    readonly kind: 'entries-vanished';
    readonly names: readonly string[]
  }
    | { readonly kind: 'other'; };

/**
 Reads the error output of a failed walk: it lost entries when every line is a vanished-entry warning or the closing error
 about the tree, and it is another failure when any line says anything else.

 @param stderr - what the walk printed on its error stream

 @returns the names of the entries lost, or that the failure is another one
 */
function readFailure(stderr: string,): FailureReading {
  /**
   The lines the walk printed, trimmed, without the empty ones.
   */
  const lines = stderr.split('\n',)
    .map(trimmed,)
    .filter(isNotEmpty,);
  /**
   The name each vanished-entry warning gave, empty for a line that is not one.
   */
  const named = lines.map(vanishedNameOf,);
  /**
   Whether every line is a vanished-entry warning or the closing error about the tree.
   */
  const allExplained = lines.every(function isExplained(
    line,
    index,
  ) {
    return (named[index] !== '') || isClosingNotThere(line,);
  },);
  if (allExplained && lines.some(isClosingNotThere,)) {
    return {
      kind: 'entries-vanished',
      names: named.filter(isNotEmpty,),
    };
  }
  return { kind: 'other', };
}

/**
 The text without the spaces around it.

 @param text - the text

 @returns the trimmed text
 */
function trimmed(text: string,): string {
  return text.trim();
}

/**
 Whether a text has anything in it.

 @param text - the text

 @returns false for the empty text
 */
function isNotEmpty(text: string,): boolean {
  return text !== '';
}

/**
 Measures a tree, made again when the walk lost entries that were removed meanwhile.

 @param path - the tree
 
 @param attempts - how many walks at most
 
 @param secondsPerAttempt - the deadline of one walk, after which it is killed and the measurement fails

 @returns the lines to print and whether the measurement succeeded

 @example
 ```ts
 import { measureFootprint, } from './footprint-measure.ts';
 
 const result = await measureFootprint({ path, attempts, secondsPerAttempt, },);
 ```
 */
export async function measureFootprint(
  {
    path,
    attempts,
    secondsPerAttempt,
  }: {
    readonly path: string;
    readonly attempts: number;
    readonly secondsPerAttempt: number;
  },
): Promise<FootprintMeasurement> {
  /**
   The names of the entries the walks lost, in the order they were told.
   */
  const lost: string[] = [];
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    try {
      /**
       What the walk printed.
       */
      // oxlint-disable-next-line eslint/no-await-in-loop -- one walk at a time: a second walk starts only when the first lost an entry, and two at once would read the same disk twice; see doc/lint/sequential-awaits.md
      const done = await execFileAsync(
        'btrfs',
        [
          'filesystem',
          'du',
          '--summarize',
          '--raw',
          path,
        ],
        {
          encoding: 'utf8',
          timeout: secondsPerAttempt * MILLISECONDS_PER_SECOND,
          killSignal: 'SIGKILL',
          maxBuffer: MAX_OUTPUT_BYTES,
        },
      );
      /**
       The sentence about what was removed meanwhile.
       */
      const removed = lost.length === 0 ? 'nothing was removed during the measurement' : `removed during the measurement: ${[...new Set(lost,),].join(', ',)}`;
      return {
        succeeded: true,
        lines: [
          `measured: attempt ${String(attempt,)} of ${String(attempts,)}; ${removed}`,
          ...done.stdout
            .trimEnd()
            .split('\n',),
        ],
      };
    }
    catch (error) {
      /**
       The failure in the shape this reads.
       */
      const failure = childFailureOf(error,);
      if (wasKilled(error,)) {
        return {
          succeeded: false,
          lines: [`btrfs filesystem du did not finish within ${String(secondsPerAttempt,)} s`,],
        };
      }
      /**
       What the error output of the walk says.
       */
      const reading = readFailure(failure.stderr,);
      if (reading.kind === 'other') {
        return {
          succeeded: false,
          lines: [`btrfs filesystem du failed: ${failure.stderr
            .trim()
            || failure.message}`,],
        };
      }
      lost.push(...reading.names,);
    }
  }
  return {
    succeeded: false,
    lines: [`the tree lost a worktree during every one of ${String(attempts,)} attempts (${[...new Set(lost,),].join(', ',) || 'no name was printed'}); try again when no worktree is being removed`,],
  };
}
