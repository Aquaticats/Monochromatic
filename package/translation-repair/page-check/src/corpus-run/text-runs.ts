import {
  codePointAt,
  codePointBefore,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Text runs
// Where a run of characters one test keeps ends, and where it starts reading
// back, for the page passes that scan prose a character at a time. `runEnd`
// lived in `canadian-date-parts.ts` and `runStart` in
// `canadian-spelling-context.ts`, and the pinyin tone pass kept its own copy
// of `runEnd` as `scanEnd` (audit area six, 2026-09-28); one home now serves
// them all.
//
// WHOLE CHARACTERS, AT UTF-16 OFFSETS (ledger B21). Both scans handed the test
// one UTF-16 unit, and neither half of a character beyond the first plane
// passes a test for a cased letter or for Han, so a Deseret letter ended a
// word and an Extension B ideograph ended a Han run. The test now reads each
// whole character, and the offsets returned stay UTF-16 offsets.

/**
 Where a run of characters one test keeps ends.

 @param text - text under scan

 @param from - where the run starts

 @param keeps - test each character of the run passes

 @returns Offset of the first character the test refuses, or the text's length

 @example
 ```ts
 runEnd({ text: '12 May', from: 0, keeps: isAsciiDigit, },); // 2
 ```
 */
export function runEnd(
  {
    text,
    from,
    keeps,
  }: {
    readonly text: string;
    readonly from: number;
    readonly keeps: (character: { readonly character: string; },) => boolean;
  },
): number {
  for (let at = from; at < text.length;) {
    /**
     Whole character at this offset.
     */
    const character = codePointAt({
      text,
      at,
    },);
    if (!keeps({ character, },))
      return at;
    at += character.length;
  }
  return text.length;
}

/**
 Where a run of characters one test keeps starts, reading back from one
 offset.

 @param text - text under scan

 @param from - offset just past the run

 @param keeps - test each character of the run passes

 @returns Offset of the run's first character

 @example
 ```ts
 runStart({ text: 'the cat', from: 7, keeps: isCasedLetter, },); // 4
 ```
 */
export function runStart(
  {
    text,
    from,
    keeps,
  }: {
    readonly text: string;
    readonly from: number;
    readonly keeps: (character: { readonly character: string; },) => boolean;
  },
): number {
  for (let at = from; at > 0;) {
    /**
     Whole character ending at this offset.
     */
    const character = codePointBefore({
      text,
      at,
    },);
    if (!keeps({ character, },))
      return at;
    at -= character.length;
  }
  return 0;
}

//endregion Text runs
