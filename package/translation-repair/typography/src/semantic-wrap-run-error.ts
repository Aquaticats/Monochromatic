import { wordForCount, } from './count-word.ts';
import {
  CONFIG_FILE_NAME,
  LINT_BINARY_NAME,
  PASSAGE_FILE_NAME,
} from './semantic-wrap-binary-constants.ts';

//region Semantic wrap run errors
// WHAT TO SAY WHEN A RUN OF THE PROGRAM FAILS OR A PASSAGE CANNOT BE SENT,
// authored from a status, a signal, a system code and counts, never from the
// program's own words: what it writes to standard error can quote the passage.

/**
 A count of characters, with the noun agreeing with it.

 @param count - how many characters

 @returns The count and the noun

 @example
 ```ts
 const phrase = countedCharacters({ count: 40, },); // '40 characters'
 ```
 */
export function countedCharacters({ count, }: { readonly count: number; },): string {
  return `${String(count,)} ${
    wordForCount({
      count,
      one: 'character',
      many: 'characters',
    },)
  }`;
}

/**
 A count of bytes, with the noun agreeing with it.

 @param count - how many bytes

 @returns The count and the noun

 @example
 ```ts
 const phrase = countedBytes({ count: 1, },); // '1 byte'
 ```
 */
function countedBytes({ count, }: { readonly count: number; },): string {
  return `${String(count,)} ${
    wordForCount({
      count,
      one: 'byte',
      many: 'bytes',
    },)
  }`;
}

/**
 How a run of the program failed, as far as its result says.
 */
export type LintBinaryRunFailure = {
  /**
   The program exited with a status other than zero.
   */
  readonly kind: 'status';

  /**
   The status it exited with.
   */
  readonly status: number;

  /**
   Bytes it wrote to standard error, whose text is not repeated.
   */
  readonly stderrBytes: number;
} | {
  /**
   The program was stopped by a signal.
   */
  readonly kind: 'signal';

  /**
   The signal's name.
   */
  readonly signal: string;
} | {
  /**
   Starting or reading the program failed with a system code.
   */
  readonly kind: 'code';

  /**
   The code, an upper-case word such as `ENOBUFS`.
   */
  readonly code: string;
} | {
  /**
   The program exited cleanly and wrote nothing for a passage that had text.
   */
  readonly kind: 'empty-reply';
};

/**
 Writes the sentence a failed run is stated by.

 @param binary - program that was run

 @param characters - length in characters of the passage it was handed

 @param failure - how it failed

 @returns The sentence

 @example
 ```ts
 const sentence = runFailureSentence({
   binary: 'monochromatic-lint',
   characters: 40,
   failure: { kind: 'signal', signal: 'SIGKILL', },
 },);
 ```
 */
function runFailureSentence(
  {
    binary,
    characters,
    failure,
  }: {
    readonly binary: string;
    readonly characters: number;
    readonly failure: LintBinaryRunFailure;
  },
): string {
  /**
   What the passage was, ending every account of a run.
   */
  const tail = `while wrapping a passage of ${countedCharacters({ count: characters, },)}`;
  if (failure.kind === 'status') {
    return `${binary} exited with status ${String(failure.status,)} and wrote `
      + `${countedBytes({ count: failure.stderrBytes, },)} to standard error, which are not repeated here `
      + `because a finding can quote the passage, ${tail}; run "${binary} --config <package>/src/`
      + `${CONFIG_FILE_NAME} --stdin --stdin-filename ${PASSAGE_FILE_NAME} --fix" with the passage on `
      + `standard input to read them`;
  }
  if (failure.kind === 'signal')
    return `${binary} was stopped by signal ${failure.signal} ${tail}`;
  if (failure.kind === 'code')
    return `${binary} could not be run to completion (${failure.code}) ${tail}`;
  return `${binary} exited cleanly and wrote nothing for a passage that has text, which no wrap can mean, ${tail}`;
}

/**
 Signals that a run of the program failed, stated by an account authored from
 its status, signal or code and never from its own words.

 @example
 ```ts
 throw new LintBinaryRunError({
   binary: 'monochromatic-lint',
   characters: 40,
   failure: { kind: 'status', status: 2, stderrBytes: 120, },
 },);
 ```
 */
export class LintBinaryRunError extends Error {
  /**
   Declares this message safe to forward: it names the program, a status, a
   signal or a system code, and counts, and writes the rest itself. What the
   program wrote to standard error is never repeated, because a finding can
   quote the passage.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the failure naming the program, how it failed and how to see more.

   @param binary - program that was run

   @param characters - length in characters of the passage it was handed

   @param failure - how it failed

   @example
   ```ts
   new LintBinaryRunError({
     binary: 'monochromatic-lint',
     characters: 40,
     failure: { kind: 'signal', signal: 'SIGKILL', },
   },);
   ```
   */
  public constructor(
    {
      binary,
      characters,
      failure,
    }: {
      readonly binary: string;
      readonly characters: number;
      readonly failure: LintBinaryRunFailure;
    },
  ) {
    super(runFailureSentence({
      binary,
      characters,
      failure,
    },),);
    this.name = 'LintBinaryRunError';
  }
}

/**
 Signals a passage that cannot cross standard input as UTF-8 unchanged.

 @example
 ```ts
 throw new LintBinaryTextError({ unpairedIndex: 8, },);
 ```
 */
export class LintBinaryTextError extends Error {
  /**
   Declares this message safe to forward: it names a position and writes the
   rest itself.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the failure naming where the unpaired surrogate stands.

   @param unpairedIndex - UTF-16 index of the first unpaired surrogate

   @example
   ```ts
   new LintBinaryTextError({ unpairedIndex: 8, },);
   ```
   */
  public constructor({ unpairedIndex, }: { readonly unpairedIndex: number; },) {
    super(
      `a passage holds an unpaired UTF-16 surrogate at index ${String(unpairedIndex,)}, which cannot be sent to `
        + `${LINT_BINARY_NAME} on standard input without UTF-8 replacing it by another character, and a wrap `
        + `only ever adds line breaks; it is refused rather than changed, so remove or replace that `
        + `character where the passage is produced`,
    );
    this.name = 'LintBinaryTextError';
  }
}

//endregion Semantic wrap run errors
