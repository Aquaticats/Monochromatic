import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { isMissingPathError, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  namesWithoutQuoting,
  refusalText,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Local program failure
// WHAT A LOCAL PROGRAM'S FAILURE SAYS, authored from a code and never from the
// failure's own words. A program run on a picture from the public corpus (the
// OCR reader) rejects with a message that holds its command line, the scratch
// paths it was handed and whatever it wrote to standard error, so a log line
// that repeated the message repeated the tool's output. The operator still
// needs to know which failure it was: a program that is not installed, a
// device with no space left, or a non-zero exit. Each is a code Node attaches
// to the rejection, and a code carries no path and no output.

/**
 Units a code may start with.
 */
const CODE_LEAD = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';

/**
 Units a code may continue with.
 */
const CODE_TAIL = `${CODE_LEAD}0123456789_`;

/**
 Most units a filesystem code may hold to be repeated.
 */
const CODE_MOST_UNITS = 32;

/**
 Whether a value is a system error code: a short upper-case word such as
 `ENOSPC` or `EACCES`, which carries no path and no output.

 @param value - the `code` a rejection carries

 @returns Whether it is two to 32 capital letters, digits and underscores, led by a capital letter

 @example
 ```ts
 isCodeWord({ value: 'ENOSPC', },); // true
 ```
 */
function isCodeWord({ value, }: { readonly value: string; },): boolean {
  if ((value.length < 2) || (value.length > CODE_MOST_UNITS))
    return false;

  for (let at = 0; at < value.length; at += 1) {
    if (!((at === 0) ? CODE_LEAD : CODE_TAIL).includes(value.charAt(at,),))
      return false;
  }
  return true;
}

/**
 How a local program failed, as far as its rejection says.
 */
export type LocalProgramFailure = {
  /**
   Nothing stands at the program's path: it is not installed.
   */
  readonly kind: 'missing';
} | {
  /**
   The rejection carries a filesystem or spawn code.
   */
  readonly kind: 'code';

  /**
   The code, an upper-case word such as `ENOSPC`.
   */
  readonly code: string;
} | {
  /**
   The program ran and exited with a code other than zero.
   */
  readonly kind: 'exit';

  /**
   The code it exited with.
   */
  readonly exitCode: number;
} | {
  /**
   The rejection names neither.
   */
  readonly kind: 'unnamed';
};

/**
 Reads which way a local program failed off its rejection, never off its message.

 @param error - caught value, of unknown type by construction

 @returns The failure's kind with the code or exit code it carries

 @example
 ```ts
 const failure = localProgramFailureOf({ error, },); // { kind: 'exit', exitCode: 1, }
 ```
 */
export function localProgramFailureOf({ error, }: { readonly error: unknown; },): LocalProgramFailure {
  if (isMissingPathError({ error, },))
    return { kind: 'missing', };

  if (!(Error.isError(error,) && ('code' in error)))
    return { kind: 'unnamed', };

  if (((typeof error.code) === 'number') && Number.isInteger(error.code,))
    return {
      kind: 'exit',
      exitCode: error.code,
    };

  if (((typeof error.code) === 'string') && isCodeWord({ value: error.code, },))
    return {
      kind: 'code',
      code: error.code,
    };

  return { kind: 'unnamed', };
}

/**
 A local program that failed, stated by an account authored from its code.

 @example
 ```ts
 throw new LocalProgramFailedError({ program: 'tesseract', failure: { kind: 'exit', exitCode: 1, }, },);
 ```
 */
export class LocalProgramFailedError extends Error {
  /**
   Declares this message safe to forward: it names the program, a code it
   exited with or a filesystem code, and writes the rest itself.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the failure naming the program and how it failed.

   @param program - program this package ran, named by a constant

   @param failure - how it failed, read off the rejection by `localProgramFailureOf`

   @param cause - the rejection, kept for a reader who owns the process

   @example
   ```ts
   new LocalProgramFailedError({ program: 'tesseract', failure: { kind: 'missing', }, cause: error, },);
   ```
   */
  public constructor(
    {
      program,
      failure,
      cause,
    }: {
      readonly program: string;
      readonly failure: LocalProgramFailure;
      readonly cause: unknown;
    },
  ) {
    super(
      (failure.kind === 'missing')
        ? `${program} is not installed`
        : (failure.kind === 'code')
          ? `${program} failed with filesystem code ${failure.code}`
          : (failure.kind === 'exit')
            ? `${program} exited with code ${String(failure.exitCode,)}`
            : `${program} failed, and its failure names neither a filesystem code nor an exit code`,
      { cause, },
    );
    this.name = 'LocalProgramFailedError';
  }
}

/**
 Renders what a local reader raised for a log line: a marked class's own
 sentence, otherwise the class name with the filesystem code the rejection
 carries, which tells a full disk from a refusal without quoting a path.

 @param error - caught value, of unknown type by construction

 @returns The sentence, or `refused by <class>` with ` with code <code>` where one is carried

 @example
 ```ts
 l.warn(`the reader failed (${readerFailureText({ error, },)})`,);
 ```
 */
export function readerFailureText({ error, }: { readonly error: unknown; },): string {
  if (namesWithoutQuoting(error,))
    return refusalText({ error, },);

  /**
   How the rejection says it failed.
   */
  const failure = localProgramFailureOf({ error, },);
  if (failure.kind === 'code')
    return `refused by ${errorName({ error, },)} with code ${failure.code}`;

  return refusalText({ error, },);
}

//endregion Local program failure
