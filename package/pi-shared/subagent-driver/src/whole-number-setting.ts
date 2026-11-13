/**
 Reads the whole number a caller gave for one count, which is the only shape a limit or a count may be set to, and says why a text
 cannot be read.

 A setting that is no whole number is refused rather than read as zero or truncated: a limit of `4.5` read as `4` would start a
 run the owner did not allow, and one read as `0` would stop every start on the machine with no line saying why.

 A whole number too large for a JavaScript number is refused for the same reason. `Number` reads a digit string past its own
 range as `Infinity`, and a cap of `Infinity` refuses nothing, while a volume given `Infinity` bytes is never under its floor: the
 protection the setting exists for would be silently absent, which is worse than a refusal.

 @module
 */

/**
 The largest whole number a JavaScript number holds exactly: 2^53, one more than `Number.MAX_SAFE_INTEGER`.

 The next whole number up, 9007199254740993, reads as 9007199254740992, and one beyond a number's range reads as `Infinity`. A
 setting above this is refused rather than read.
 */
export const LARGEST_EXACTLY_HELD_WHOLE_NUMBER: number = Number.MAX_SAFE_INTEGER + 1;

/**
 Why the text of a whole-number setting cannot be read.

 - `not-a-whole-number`: a character that is no digit, or nothing at all.
 - `too-large-to-hold-exactly`: a whole number a JavaScript number cannot hold, so it would read as another value or as
   `Infinity`.
 */
export type WholeNumberProblem = 'not-a-whole-number' | 'too-large-to-hold-exactly';

/**
 What reading one setting gave: the number, or the reason there is none.
 */
export type WholeNumberReading =
  | {
    /**
     Whether the text was read, which is true here.
     */
    readonly read: true;

    /**
     The number the text was read as, which every caller acts on.
     */
    readonly value: number;
  }
    | {
      /**
       Whether the text was read, which is false here.
       */
      readonly read: false;

      /**
       What stopped the text from being read.
       */
      readonly problem: WholeNumberProblem;
    };

/**
 Reads whether text is one or more digits and nothing else.

 @param text - the text to read

 @returns whether it is a whole number in shape

 @example
 ```ts
 const whole = isWholeNumber('20',);
 ```
 */
function isWholeNumber(text: string,): boolean {
  if (text === '') {
    return false;
  }
  for (const character of text) {
    if ((character < '0') || (character > '9')) {
      return false;
    }
  }
  return true;
}

/**
 Reads a digit string without the zeros a caller wrote in front of its value.

 One pass over the text and no stack, so a setting of hundreds of digits costs its own length and nothing more.

 @param digits - the digits, with no sign and no spaces

 @returns the digits from the first that is no zero, which is empty when every digit is a zero

 @example
 ```ts
 const significant = withoutTheZerosInFront('0009007199254740992',);
 ```
 */
function withoutTheZerosInFront(digits: string,): string {
  for (let at = 0; at < digits.length; at += 1) {
    if (digits[at] !== '0') {
      return digits.slice(at,);
    }
  }
  return '';
}

/**
 Says whether a digit string is greater than the largest whole number a number holds.

 The comparison is made on the text, never by building a number from it, so the reading itself cannot overflow into the value it
 is refusing. Dropping the zeros a caller wrote in front of the value is one pass over the text, so a setting of any length is
 read in the time its length takes and with no stack at all.

 @param digits - the digits, with no sign and no spaces

 @returns whether the value is above {@link LARGEST_EXACTLY_HELD_WHOLE_NUMBER}

 @example
 ```ts
 const tooLarge = isAboveTheLargestExactlyHeld('9007199254740993',);
 ```
 */
function isAboveTheLargestExactlyHeld(digits: string,): boolean {
  /**
   The value with the zeros in front of it dropped, which is no digit at all when the value is zero.
   */
  const significant = withoutTheZerosInFront(digits,);
  /**
   The largest value this reads, in the digits the comparison is made on.
   */
  const largest = String(LARGEST_EXACTLY_HELD_WHOLE_NUMBER,);
  if (significant.length !== largest.length) {
    return significant.length > largest.length;
  }
  return significant > largest;
}

/**
 Reads the whole number the text of one setting gives, or says why it gives none.

 @param text - the setting's text, already trimmed of the spaces around it

 @returns the number, or the problem that stopped it from being read

 @example
 ```ts
 const reading = wholeNumberIn('20',);
 ```
 */
export function wholeNumberIn(text: string,): WholeNumberReading {
  if (!isWholeNumber(text,)) {
    return {
      read: false,
      problem: 'not-a-whole-number',
    };
  }
  if (isAboveTheLargestExactlyHeld(text,)) {
    return {
      read: false,
      problem: 'too-large-to-hold-exactly',
    };
  }
  return {
    read: true,
    value: Number(text,),
  };
}
