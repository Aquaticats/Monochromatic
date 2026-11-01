//region Line starts
// Where each line of a text starts, for the readers that turn an offset into a
// line: the Han residue floor (`translate-han-residue.ts`) and the coverage
// census's source-line mapping (`corpus-run/coverage-lines.ts`). ONE BODY,
// kept here after the census wrote a second copy and the duplicate-body guard
// named the pair (ledger T8, 2026-09-29).

/**
 Offsets each line of a text starts at.

 @param text - text to read

 @returns Start offsets, the first always zero, one more after every newline

 @example
 ```ts
 lineStartsOf({ text: 'a\nbc', },); // [0, 2]
 ```
 */
export function lineStartsOf({ text, }: { readonly text: string; },): readonly number[] {
  /**
   Starts found so far.
   */
  const starts = [0,];
  for (
    let at = text.indexOf('\n',);
    at !== (-1);
    at = text.indexOf(
      '\n',
      at + 1,
    )
  )
    starts.push(at + 1,);
  return starts;
}

//endregion Line starts
