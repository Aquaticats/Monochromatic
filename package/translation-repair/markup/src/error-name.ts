//region Error name
// Names what a thrown value IS, without asserting it into a shape.
//
// A CATCH BINDING IS `unknown`, and asserting it to `Error` is a claim about
// something nobody checked. Anything at all can be thrown, and the one occasion
// this matters is the one occasion the report is worth reading.
//
// LIFTED OUT OF FOUR COPIES on 2026-08-24: whole functions in
// `verify-published.ts` and `anthropic-delta-scan.ts`, and the same ternary
// written inline in `provider-budget.ts` and `producer-calibrate.ts`. A fifth
// caller was about to be written, which is the point at which duplication
// stops being incidental.

/**
 Stand-in for a thrown value that carries no class name.

 Phrased as a sentence rather than as a token, because it lands in a log line
 where a reader is already asking what went wrong and a bare `unknown` answers
 nothing.
 */
const NAMELESS_THROW = 'a thrown value that is not an Error';

/**
 Class token of a thrown value that is no Error.
 */
const NON_ERROR_CLASS = 'NotAnError';

/**
 Names the class of whatever was caught.

 @param error - caught value, of unknown type by construction

 @returns Class name, or a stand-in for a value that has none

 @example
 ```ts
 console.error(errorName({ error, },),);
 ```
 */
export function errorName(
  { error, }: { readonly error: unknown; },
): string {
  if (Error.isError(error,))
    return error.name;
  return NAMELESS_THROW;
}

/**
 Whether a character may stand in a one-token field of a log line.

 @param character - one character of a class name

 @returns True for an ASCII letter or digit and for the marks `_`, `$`, `.` and `-`

 @example
 ```ts
 const kept = isTokenCharacter({ character: 'E', },); // => true
 ```
 */
function isTokenCharacter({ character, }: { readonly character: string; },): boolean {
  return ((character >= 'a') && (character <= 'z'))
    || ((character >= 'A') && (character <= 'Z'))
    || ((character >= '0') && (character <= '9'))
    || '_$.-'.includes(character,);
}

/**
 Class a stop record carries for the error that ended an entry, as one token.

 THE FIELD OF A LOG LINE A PERSON COUNTS BY (B1223). `errorName` answers a sentence for a thrown value that is no Error,
 and a sentence with spaces splits into several fields when a line is read by field, so this answers `NotAnError` for
 it. A class name holding anything but letters, digits and the marks `_`, `$`, `.` and `-` has each such character
 written as `_`, and an empty name reads as `Error`, so the token is never empty and never splits.

 @param error - caught value, of unknown type by construction

 @returns One token naming the class

 @example
 ```ts
 console.log(`TALLY Cat status=ERROR class=${errorClassOf({ error, },)}`,);
 ```
 */
export function errorClassOf(
  { error, }: { readonly error: unknown; },
): string {
  if (!Error.isError(error,))
    return NON_ERROR_CLASS;
  return classTokenOf({ name: error.name, },);
}

/**
 One token for a class name that arrives as data, such as the `errorClass` of a record read off disk or the `name` of an
 error whose author set it to anything.

 TOTAL (B1248): a name that is not a string reads as `Error`, where `errorClassOf` once threw a `TypeError` out of
 `Array.from` for an error whose `name` was not a string, in the shipper's own path and in the tally of a stop. Every
 character outside letters, digits and the marks `_`, `$`, `.` and `-` is written as `_`, so the token never splits a
 line read by field and never carries a control character into a terminal.

 @param name - class name, of unknown type by construction

 @returns One token, never empty

 @example
 ```ts
 console.log(`INTERIM Cat class=${classTokenOf({ name: record.errorClass, },)}`,);
 ```
 */
export function classTokenOf(
  { name, }: { readonly name: unknown; },
): string {
  if ((typeof name) !== 'string')
    return 'Error';
  /**
   The class name with every character a field cannot hold written as an underscore, in one pass.
   */
  const characters = Array.from(
    name,
    function tokenCharacter(character,): string {
      return isTokenCharacter({ character, },) ? character : '_';
    },
  );
  return (characters.length === 0) ? 'Error' : characters.join('',);
}

/**
 Names a caught failure by its filesystem code where it carries one, and by
 its class otherwise.

 NAMES A CODE, NEVER A MESSAGE. A filesystem error's message quotes the path
 it failed on and a run directory path can name a person, which is why
 `errorName` exists at all. A code carries no path, and `EACCES` tells a
 reader what to do where a bare `Error` tells them nothing.

 LIFTED OUT OF `ledger-report.ts` on 2026-08-25, when `run-json-read.ts`
 needed the same distinction to report a run file it could not open.

 @param error - caught value, of unknown type by construction

 @returns Code or class name, whichever is there

 @example
 ```ts
 console.error(`could not read it (${failureName({ error, },)})`,);
 ```
 */
export function failureName(
  { error, }: { readonly error: unknown; },
): string {
  if (Error.isError(error,)
    && ('code' in error)
    && ((typeof error.code) === 'string'))
    return error.code;

  return errorName({ error, },);
}

//endregion Error name
