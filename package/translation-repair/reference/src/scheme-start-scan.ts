//region Scheme start scan
// Where the next web address begins in a text. Two readers scan prose for the
// same two schemes, each with its own stop characters and its own trimming:
// the dropped-destination check (`corpus-run/dropped-destinations.ts`) and the
// cited-reference scan (`cited-reference-scan.ts`). Both ask this module where
// the next address starts, so the search is written once and kept linear once.
//
// THE STEM IS SEARCHED, NOT EACH SCHEME. The dropped-destination scan searched
// for each scheme apart with `indexOf` and took the earlier answer, so a scheme
// the text lacks was read for to the text's end again at every address found.
// Measured on 2026-10-05 (UTC) over a text repeating one `https://` address:
// 2,000 addresses took 34 ms and 16,000 took 2,168 ms, four times the time at
// every doubling. Every scheme opens with the stem, so one forward search for
// the stem meets every place a scheme can start, and each is tested where it
// stands.

/**
 Schemes a web address starts with.
 */
const SCHEMES = [
  'https://',
  'http://',
] as const;

/**
 Shortest prefix every scheme shares, the one text searched for.
 */
const SCHEME_STEM = 'http';

/**
 What `indexOf` answers when nothing is found.
 */
const NOT_FOUND = -1;

/**
 Whether a whole scheme starts at one position.

 @param text - text scanned

 @param at - offset to test

 @returns Whether `https://` or `http://` begins here

 @example
 ```ts
 schemeAt({ text: 'see https://a.example', at: 4, },);
 // => true
 ```
 */
function schemeAt(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  return SCHEMES
    .some(function startsHere(scheme,): boolean {
      return text.startsWith(
        scheme,
        at,
      );
    },);
}

/**
 Offset of the first whole scheme at or after a position, or the text's
 length when none starts there.

 THE TEXT'S LENGTH STANDS FOR "NONE", since no scheme starts where the text
 ends: a caller that reads a run from the answer without testing it reads an
 empty one rather than one cut from an offset no text has.

 LINEAR OVER A WHOLE SCAN. Each call searches forward from `from` for the
 stem and resumes one past a stem that opens no scheme, so a call reads only
 the stretch between `from` and its answer. A caller that resumes at or past
 the end of the run it read never has text behind its cursor searched again,
 and its whole scan costs time in proportion to the text.

 @param text - text scanned

 @param from - offset the search starts at

 @returns Offset a scheme starts at, or `text.length` when no scheme starts
 at or after `from`

 @example
 ```ts
 nextSchemeStart({ text: 'the httpd log, then http://a.example', from: 0, },);
 // => 20
 ```
 */
export function nextSchemeStart(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  for (
    let at = text.indexOf(
      SCHEME_STEM,
      from,
    );
    at !== NOT_FOUND;
    at = text.indexOf(
      SCHEME_STEM,
      at + 1,
    )
  ) {
    if (
      schemeAt({
        text,
        at,
      },)
    )
      return at;
  }
  return text.length;
}

//endregion Scheme start scan
