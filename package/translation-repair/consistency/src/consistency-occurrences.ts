//region Consistency occurrences
// WHERE A TEXT STANDS INSIDE ANOTHER, by plain equality of characters: the
// counter (`consistency-count.ts`) reads a judge's quote through the fold of
// `consistency-fold.ts`, and this is the reading for a text that is already the
// words a place holds, such as a recorded span read back against its own words
// (`corpus-run/artifact-two-lane-read-consistency-located.ts`).

/**
 Where a text stands inside another.

 @example
 ```ts
 const found: QuoteOccurrences = { kind: 'once', at: 4, };
 ```
 */
export type QuoteOccurrences =
  | {
    /**
     The text stands nowhere in the other.
     */
    readonly kind: 'none';
  }
    | {
    /**
     The text stands exactly once.
     */
      readonly kind: 'once';

    /**
     Offset of that one occurrence.
     */
      readonly at: number;
    }
    | {
    /**
     The text stands more than once, overlaps included.
     */
      readonly kind: 'many';
    };

/**
 Where a text stands inside another: nowhere, once, or more than once,
 overlaps included.

 @param haystack - text searched

 @param needle - text looked for, never empty

 @returns That it is absent, the offset of its one occurrence, or that it
 occurs more than once

 @example
 ```ts
 const found = occurrencesOf({ haystack: 'aaa', needle: 'aa', },); // { kind: 'many' }
 ```
 */
export function occurrencesOf(
  {
    haystack,
    needle,
  }: {
    readonly haystack: string;
    readonly needle: string;
  },
): QuoteOccurrences {
  if (!haystack.includes(needle,))
    return { kind: 'none', };
  /**
   Offset of the first occurrence.
   */
  const at = haystack.indexOf(needle,);
  if (haystack.includes(
    needle,
    at + 1,
  ))
    return { kind: 'many', };
  return {
    kind: 'once',
    at,
  };
}

//endregion Consistency occurrences
