//region Markdown link targets
// Where a Markdown text's links point, read without a Markdown parser: the
// living-docs fixture (`living-docs.test-fixture.ts`) follows the canonical
// handover's links to its snapshot and current planning docs with it. Kept
// beside that fixture rather than in it because that file sits at the
// package's line budget (moved 2026-10-22).

/**
 Every Markdown link target in a text: what follows each `](` up to its `)`,
 without a fragment.

 @param text - Markdown text

 @returns Targets in text order

 @example
 ```ts
 linkTargets({ text: 'see [the nap log](naps.md#noon)', },); // ['naps.md']
 ```
 */
export function linkTargets({ text, }: { readonly text: string; },): readonly string[] {
  return text
    .split('](',)
    .slice(1,)
    .flatMap(function target(after,): readonly string[] {
      /**
       Where the target closes.
       */
      const end = after.indexOf(')',);
      if (end === (-1))
        return [];
      /**
       Target with any fragment.
       */
      const whole = after.slice(
        0,
        end,
      );
      /**
       Where a fragment opens, or the end.
       */
      const fragment = whole.indexOf('#',);
      return [fragment === (-1) ? whole : whole.slice(
        0,
        fragment,
      ),];
    },);
}

//endregion Markdown link targets
