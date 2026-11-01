import { isAsciiDigit, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Fidelity alteration
// The fixture that asks whether the judges read THE ORIGINAL, rather than
// whether they read well.
//
// WHAT THE OTHER TWO CANNOT SEPARATE. A deletion leaves a gap in the argument
// and an inserted sentence is a non-sequitur, so a judge that never looks at the
// Chinese, and simply prefers whichever English reads better, scores both in
// full. Passing them rules out habits; it does not show the source was consulted.
//
// THIS ONE CHANGES A NUMBER THE ORIGINAL ALSO STATES. Digits survive
// translation: a birth year written `2004年` in the Chinese is `2004` in the
// English, so the same run of characters sits on both sides and can be checked
// mechanically. Changing it in the English alone produces a candidate that is
// the SAME LENGTH, equally fluent, equally in register, and wrong about a fact
// the reader can only catch by looking at the original. No amount of reading the
// English decides it.
//
// GROUND TRUTH IS VERIFIED RATHER THAN ASSUMED, which is what makes this fixture
// stronger than a dropped qualifier. The number is only used when it occurs
// exactly once in the English and appears in the Chinese, and the replacement is
// only used when it appears in NEITHER, so the clean text is supported by the
// original and the damaged text is not.
//
// WHAT IT STILL DOES NOT PROVE: that a judge reads the source on ORDINARY
// slices. It shows whether the roster can, on a slice where nothing else can
// decide.

/**
 Shortest digit run worth altering.

 Two, because a lone digit collides constantly: it appears inside longer
 numbers, in list markers and in dates on both sides, so neither the
 occurrence check nor the support check would mean anything.
 */
const MIN_DIGITS = 2;

/**
 How many replacement values are tried before giving up on one number.
 */
const REPLACEMENT_TRIES = 9;

/**
 Every maximal run of digits in a passage, in the order they appear.

 @param text - passage to scan

 @returns Digit runs, duplicates included

 @example
 ```ts
 const runs = digitRuns({ text: cleanText, },);
 ```
 */
export function digitRuns({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Runs collected so far, and where the run in progress began.
   */
  const collected: string[] = [];

  /**
   Start of the run in progress, or minus one between runs.
   */
  let runStart = -1;
  for (
    let cursor = 0;
    cursor <= text.length;
    cursor += 1
  ) {
    /**
     Character at the cursor, empty once past the end so the last run closes.
     */
    const character = text.charAt(cursor,);

    /**
     Whether the scan is inside a run.
     */
    const inRun = isAsciiDigit({ character, },);
    if (inRun && (runStart === (-1)))
      runStart = cursor;

    /**
     Whether a run just ended at this cursor.
     */
    const runEnded = (!inRun) && (runStart !== (-1));
    if (runEnded) {
      collected.push(text.slice(
        runStart,
        cursor,
      ),);
      runStart = -1;
    }
  }
  return collected;
}

/**
 Whether a passage carries a string exactly once.

 @param text - passage to search

 @param needle - string to count, drawn from the passage itself

 @returns Whether exactly one occurrence exists

 @throws Error when the passage does not carry the string at all, which no
 caller's needle can do

 @example
 ```ts
 const unique = occursOnce({ text: cleanText, needle: '2004', },);
 ```
 */
function occursOnce(
  {
    text,
    needle,
  }: {
    readonly text: string;
    readonly needle: string;
  },
): boolean {
  /**
   Where it first appears. The caller draws the needle from this very text
   (`digitRuns` over it), so the absence case has no input.
   */
  const first = text.indexOf(needle,);

  // LOUD RATHER THAN ANSWERED. Without this check a missing needle searched
  // again from the passage's start, found nothing, and read as occurring
  // once (ledger M113).
  if (first === (-1))
    throw new Error(
      'unreachable: a number missing from the passage it was read off, though sharedNumber draws every '
        + 'needle from that passage with digitRuns',
    );
  return !text.includes(
    needle,
    first + 1,
  );
}

/**
 Number the original states and the translation renders exactly once.

 @param cleanText - archive English for this slice

 @param sourceText - Chinese original for the same slice

 @returns Longest such number, empty when the slice carries none

 @example
 ```ts
 const shared = sharedNumber({ cleanText, sourceText, },);
 ```
 */
export function sharedNumber(
  {
    cleanText,
    sourceText,
  }: {
    readonly cleanText: string;
    readonly sourceText: string;
  },
): string {
  /**
   Numbers the ORIGINAL states in its own right.

   WHOLE RUNS RATHER THAN SUBSTRINGS, which decides whether the fixture has
   ground truth at all. `sourceText.includes('2004')` is true of a Chinese
   carrying `120045`, a QQ number or a phone number, and this corpus carries
   plenty; altering a year on that evidence damages a claim the original never
   made, so neither candidate would be source-supported and a judge that
   refused to choose would be scored wrong for being right.
   */
  const statedNumbers = digitRuns({ text: sourceText, },);

  /**
   Numbers in the English that the Chinese also carries and that appear once.
   */
  const shared = digitRuns({ text: cleanText, },)
    .filter(function longEnough(run,) {
      return run.length >= MIN_DIGITS;
    },)
    .filter(function statedInSource(run,) {
      return statedNumbers.includes(run,);
    },)
    .filter(function unique(run,) {
      return occursOnce({
        text: cleanText,
        needle: run,
      },);
    },);

  /**
   Longest of them, since a longer number is likelier to be a year or a count
   than an accident of formatting.
   */
  const longest = shared.toSorted(function byLengthDescending(
    a,
    b,
  ) {
    return b.length - a.length;
  },)
    .at(0,);
  return longest ?? '';
}

/**
 A different number of the same shape that neither side supports.

 @param original - number as both sides state it

 @param cleanText - archive English, which must not already carry the result

 @param sourceText - Chinese original, which must not state it either

 @returns Replacement, empty when every candidate collides

 @example
 ```ts
 const wrong = unsupportedVariant({ original: '2004', cleanText, sourceText, },);
 ```
 */
export function unsupportedVariant(
  {
    original,
    cleanText,
    sourceText,
  }: {
    readonly original: string;
    readonly cleanText: string;
    readonly sourceText: string;
  },
): string {
  /**
   Last digit as a number, which is what the tries walk.
   */
  const lastDigit = Number(original.slice(-1,),);

  /**
   Every same-length variant differing in the final digit, nearest first.
   */
  const variants = Array.from(
    { length: REPLACEMENT_TRIES, },
    function toVariant(
      _unused,
      step,
    ): string {
      /**
       How far past the original digit this step reaches, from one.
       */
      const distance = step + 1;

      /**
       Digit this step lands on, wrapping through ten.
       */
      const digit = (lastDigit + distance) % 10;
      return original.slice(
        0,
        -1,
      ) + String(digit,);
    },
  );

  /**
   First variant neither side carries, so the damaged text states something no
   reading of the original supports.
   */
  const unsupported = variants.find(function isUnsupported(variant,) {
    if (sourceText.includes(variant,))
      return false;
    return !cleanText.includes(variant,);
  },);
  return unsupported ?? '';
}

//endregion Fidelity alteration
