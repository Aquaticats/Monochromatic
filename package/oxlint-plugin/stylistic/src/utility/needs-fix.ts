import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { Span, } from '@oxlint/plugins';

import type { PerLineBoundaryOffsets, } from './per-line-boundary.ts';
import { lineAt, } from './line-at.ts';
import {
  at,
  rangeOf,
} from './range.ts';

/**
 Parameters for {@link needsPerLineFix}.
 */
export type NeedsPerLineFixParams = {
  /**
   Full file source text.
   */
  readonly sourceText: string;
  /**
   Container AST node.
   */
  readonly container: Span;
  /**
   Explicit delimiter offsets when the container span is wider than the list.
   */
  readonly boundary?: PerLineBoundaryOffsets;
  /**
   Child items to check (must be non-empty).
   */
  readonly items: readonly Span[];
};

/**
 Checks whether any items share a source line with each other
 or with the container delimiters.
 
 @returns whether a fix is needed
 
 @example
 ```ts
 if (needsPerLineFix({ sourceText, container, items })) { /* report *\/ }
 ```
 */
export function needsPerLineFix({
  sourceText,
  container,
  boundary,
  items,
}: ForeignBorrowed<NeedsPerLineFixParams>,): boolean {
  /**
   Container span boundaries; fallback delimiter offsets when no explicit boundary is supplied.
   */
  const containerRange = rangeOf(container,);
  /**
   Delimiter offsets compared against item lines to detect inline first/last items.
   */
  const boundaryOffsets = boundary ?? {
    openOffset: containerRange[0],
    closeOffset: containerRange[1],
  };
  /**
   First item's range; used to test whether it shares a line with the opening delimiter.
   */
  const firstRange = rangeOf(at({
    arr: items,
    index: 0,
  },),);

  if (lineAt({
    sourceText,
    offset: boundaryOffsets.openOffset,
  },)
    === lineAt({
      sourceText,
      offset: firstRange[0],
    },)) {
    return true;
  }

  /**
   Last item's range; used to test whether it shares a line with the closing delimiter.
   */
  const lastRange = rangeOf(at({
    arr: items,
    index: items.length
      - 1,
  },),);
  if (lineAt({
    sourceText,
    offset: lastRange[1],
  },)
    === lineAt({
      sourceText,
      offset: boundaryOffsets.closeOffset,
    },)) {
    return true;
  }

  for (let loopIndex = 1; loopIndex < items
    .length; loopIndex++) {
    /**
     Previous item's range; paired with `currRange` to detect items sharing a line.
     */
    const prevRange = rangeOf(at({
      arr: items,
      index: loopIndex - 1,
    },),);
    /**
     Current item's range; paired with `prevRange` to detect items sharing a line.
     */
    const currRange = rangeOf(at({
      arr: items,
      index: loopIndex,
    },),);
    if (lineAt({
      sourceText,
      offset: prevRange[1],
    },)
      === lineAt({
        sourceText,
        offset: currRange[0],
      },)) {
      return true;
    }
  }

  return false;
}
