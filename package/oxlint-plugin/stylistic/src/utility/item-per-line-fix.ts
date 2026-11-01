import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  Fixer,
  Span,
} from '@oxlint/plugins';

import { leadingWhitespace, } from './indent.ts';
import type { PerLineBoundaryOffsets, } from './per-line-boundary.ts';
import {
  at,
  rangeOf,
} from './range.ts';

/**
 Bracket pair that delimits the list being reformatted.
 
 @example
 ```ts
 // Function arguments / parameters
 { open: '(', close: ')' }
 
 // Array / tuple
 { open: '[', close: ']' }
 
 // Object / destructure / type literal
 { open: '{', close: '}' }
 ```
 */
export type BracketPair = {
  /**
   Opening bracket character.
   */
  readonly open: '(' | '[' | '{';
  /**
   Closing bracket character.
   */
  readonly close: ')' | ']' | '}';
};

/**
 Configuration for generating a per-line autofix.
 */
export type PerLineFixConfig = {
  /**
   Fixer instance from the lint report callback.
   */
  readonly fixer: Fixer;
  /**
   Lint context for source text access.
   */
  readonly context: Context;
  /**
   Child items to place one per line.
   */
  readonly items: readonly Span[];
  /**
   Full file source text.
   */
  readonly sourceText: string;
  /**
   Bracket pair that wraps the items.
   
   Used to locate the opening and closing brackets by scanning
   from item positions rather than from the (potentially larger)
   container node span.
   */
  readonly bracketPair: BracketPair;
  /**
   Explicit delimiter offsets when the caller already located the list.
   */
  readonly boundary?: PerLineBoundaryOffsets;
  /**
   Delimiter to place after each item.
   
   Defaults to `','` for comma-separated constructs.
   Pass `';'` for TypeScript type/interface members.
   */
  readonly delimiter?: ',' | ';';
};

/**
 Builds a fixer result that reformats items one per line.
 
 Finds the enclosing brackets by scanning backward from the first item
 and forward from the last item, then replaces the bracket range with
 properly formatted content. This avoids the container-text approach
 which breaks when the container span includes extraneous content
 (type annotations, callee chains with nested brackets, generics).
 
 @param fixer - fixer instance from the lint report callback
 
 @param context - lint context for source text access
 
 @param items - child items to place one per line
 
 @param sourceText - full file source text
 
 @param bracketPair - opening and closing bracket characters to locate
 
 @param delimiter - character to separate items (`','` or `';'`, defaults to `','`)
 
 @returns fixer replacement result
 
 @example
 ```ts
 return buildPerLineFix({
   fixer, context, items, sourceText,
   bracketPair: { open: '(', close: ')' },
 });
 ```
 */
export function buildPerLineFix({
  fixer,
  context,
  items,
  sourceText,
  bracketPair,
  boundary,
  delimiter = ',',
}: ForeignBorrowed<PerLineFixConfig>,): ReturnType<Fixer['replaceText']> {
  /**
   Range of the first item; left edge used to locate the opening bracket.
   */
  const firstRange = rangeOf(at({
    arr: items,
    index: 0,
  },),);
  /**
   Range of the last item; right edge used to locate the closing bracket.
   */
  const lastRange = rangeOf(at({
    arr: items,
    index: items.length
      - 1,
  },),);

  /**
   Opening bracket position, either supplied by caller or scanned backward from first item.
   */
  const openPos = boundary === undefined
    ? sourceText.lastIndexOf(
      bracketPair.open,
      firstRange[0]
        - 1,
    )
    : boundary.openOffset;

  /**
   Scan forward from last item to find the closing bracket when no explicit boundary is supplied.
   */
  const [, lastRangeEnd,] = lastRange;
  /**
   Closing bracket position, either supplied by caller or scanned from the last item's end.
   */
  const closePos = boundary === undefined
    ? sourceText.indexOf(
      bracketPair.close,
      lastRangeEnd,
    )
    : boundary.closeOffset;

  if ((openPos === (-1)) || (closePos === (-1))) {
    return fixer.replaceTextRange(
      [
        firstRange[0],
        firstRange[0],
      ],
      '',
    );
  }

  /**
   Compute base indentation from the line containing the opening bracket.
   */
  const lineStart = sourceText.lastIndexOf(
    '\n',
    openPos - 1,
  )
    + 1;
  /**
   Substring of the open-bracket line up to the bracket itself; leading whitespace is the existing indent.
   */
  const linePrefix = sourceText.slice(
    lineStart,
    openPos,
  );
  /**
   Whitespace prefix extracted from the open-bracket line; defaults to empty when the bracket starts the line.
   */
  const baseIndent = leadingWhitespace(linePrefix,);
  /**
   Two-space continuation indent for items inside the brackets.
   */
  const childIndent = `${baseIndent}  `;

  /**
   Extract each item's source text, stripping any existing trailing delimiter.
   */
  const itemTexts = items.map(
    function getItemText(item: ForeignBorrowed<Span>,): string {
      /**
       Trimmed source text of the item; the trailing delimiter (if any) is stripped below to be re-added uniformly.
       */
      const raw = context.sourceCode
        .getText(item,)
        .trim();
      if (raw.endsWith(';',)
        || raw
          .endsWith(',',)) {
        return raw.slice(
          0,
          -1,
        );
      }
      return raw;
    },
  );

  /**
   Check whether the original source has a trailing delimiter between last item and close bracket.
   */
  const trailingRegion = new Set(sourceText.slice(
    lastRange[1],
    closePos,
  ),);
  /**
   Whether the original source had a trailing delimiter; preserved verbatim in the rewrite to avoid forcing a style change.
   */
  const hasTrailingDelimiter = trailingRegion.has(',',)
    || trailingRegion
      .has(';',);

  /**
   Items rendered one per line with `childIndent` prefix and the appropriate delimiter suffix.
   */
  const formattedItems = itemTexts
    .map(function formatItem(
      text,
      idx,
    ): string {
      /**
       Whether this is the last item; combined with `hasTrailingDelimiter` to decide the suffix.
       */
      const isLast = idx === (itemTexts.length
        - 1);
      /**
       Delimiter or empty string for the last item without a trailing delimiter in the original.
       */
      const suffix = (isLast && (!hasTrailingDelimiter)) ? '' : delimiter;
      return `${childIndent}${text}${suffix}`;
    },)
    .join('\n',);

  /**
   Replace from opening bracket to closing bracket inclusive.
   */
  const replacement =
    `${bracketPair.open}\n${formattedItems}\n${baseIndent}${bracketPair.close}`;
  return fixer.replaceTextRange(
    [
      openPos,
      closePos + 1,
    ],
    replacement,
  );
}
