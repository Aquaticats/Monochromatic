import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { RetainedArchiveFootnoteLabel, } from './footnote-label-rewrite.ts';

//region Fresh archive-label allocation
// Decimal candidates are generated, never parsed from potentially large or nonnumeric existing labels.

/**
 Assigns deterministic unused labels to colliding unmatched archive notes.
 Every result stays outside both input namespaces; no original correspondence is implied.

 @param labels - distinct colliding archive spellings in original encounter order

 @param reserved - normalized identifiers from both complete input namespaces

 @returns One distinct fresh label per displaced archive identifier

 @example
 ```ts
 const retained = retainedFootnoteLabels({ labels: ['1'], reserved: new Set(['1', '2', '3']) });
 ```
 */
export function retainedFootnoteLabels(
  {
    labels,
    reserved,
  }: {
    readonly labels: readonly string[];
    readonly reserved: ReadonlySet<string>;
  },
): readonly RetainedArchiveFootnoteLabel[] {
  /**
   Available decimal identifiers in increasing order.
   */
  const free: string[] = [];
  /**
   At most every reserved identifier can occupy one decimal candidate.
   */
  const bound = reserved.size + labels.length;
  for (let candidate = 1; (candidate <= bound) && (free.length < labels.length); candidate += 1) {
    /**
     ASCII decimal labels normalize to themselves.
     */
    const label = String(candidate,);
    if (!reserved.has(label,))
      free.push(label,);
  }
  return labels.map(function retainedAs(
    from,
    index,
  ): RetainedArchiveFootnoteLabel {
    return {
      from,
      retainedAs: nonNullishOrThrow(free[index],),
    };
  },);
}

//endregion Fresh archive-label allocation
