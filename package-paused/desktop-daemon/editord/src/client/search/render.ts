/**
 * Search overlay result rendering and highlighting.
 *
 * Creates result DOM elements and applies CSS Custom Highlight API
 * marks for query matches. Separated from the main overlay component
 * to keep each file under the effective line limit.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { SearchResult, } from '../../../protocol.ts';
import { middleOut, } from '../middle-out.ts';

/**
 * Renders search results into DOM elements.
 *
 * @param results - search results to render
 *
 * @param query - search query for middle-out path truncation
 *
 * @param rootPrefix - root directory prefix for relativizing paths
 *
 * @param budget - character budget for middle-out truncation
 *
 * @param onSelect - callback invoked when a result is clicked
 *
 * @returns array of result DOM elements
 *
 * @example
 * ```ts
 * const result = renderResultElements({ results: [{ path: "src/app.ts", line: 5, text: "const app = ..." }], query: 'searchTerm', rootPrefix: '/home/user/project', budget: 50, onSelect: function handleSelect(event) { l.info(event); }, });
 * ```
 */
export function renderResultElements({
  results,
  query,
  rootPrefix,
  budget,
  onSelect,
}: {
  readonly results: readonly SearchResult[];
  readonly query: string;
  readonly rootPrefix: string;
  readonly budget: number;
  readonly onSelect: (index: number,) => void;
},): HTMLElement[] {
  return results.map(function createResultElement(
    result,
    index,
  ) {
    /**
     * Computes a display-friendly relative path from the root directory.
     *
     * @param absolutePath - absolute file path
     *
     * @returns path relative to rootDir
     */
    function relativePath(absolutePath: string,): string {
      return absolutePath.startsWith(rootPrefix,)
        ? absolutePath.slice(rootPrefix.length,)
        : absolutePath;
    }

    /**
     * Path truncated around the query match so the result fits in the line budget.
     */
    const displayPath = middleOut({
      text: relativePath(result.path,),
      query,
      budget,
    },);

    /**
     * DOM children built up incrementally; content results append line and text spans below.
     */
    const children: (Node | string)[] = [
      h({
        tag: 'span',
        class: 'result-path',
        text: displayPath,
      },),
    ];

    if (result.kind
      === 'content') {
      children.push(
        h({
          tag: 'span',
          class: 'result-line',
          text: `:${String(result.line,)}`,
        },),
        h({
          tag: 'span',
          class: 'result-text',
          text: result.text,
        },),
      );
    }

    return h({
      tag: 'div',
      class: 'result',
      attrs: index === 0 ? { 'data-selected': '', } : {},
      children,
      on: {
        click: function handleClick() {
          onSelect(index,);
        },
      },
    },);
  },);
}

/**
 * Highlights all occurrences of the query in rendered result text nodes
 * using the CSS Custom Highlight API. Case-insensitive matching.
 *
 * @param query - search query to highlight
 *
 * @param container - results container element whose text nodes to scan
 *
 * @example
 * ```ts
 * highlightMatches({ query: 'searchTerm', container: resultElement, });
 * ```
 */
export function highlightMatches(
  {
    query,
    container,
  }: {
    readonly query: string;
    readonly container: HTMLDivElement;
  },
): void {
  if (query === '') {
    CSS.highlights
      .delete('hl-search-match',);
    return;
  }

  /**
   * Lowercase query reused for every comparison to keep matching case-insensitive.
   */
  const lowerQuery = query.toLowerCase();
  /**
   * Cached query length so each match advance avoids re-reading `query.length`.
   */
  const queryLength = query.length;
  /**
   * Accumulated DOM Ranges submitted to the Custom Highlight API in one call.
   */
  const ranges: Range[] = [];

  /**
   * Text-only walker over the results container; skips element nodes that cannot host highlights.
   */
  const walker = document.createTreeWalker(
    container,
    NodeFilter.SHOW_TEXT,
  );

  /**
   * Current text node being scanned for matches; null terminates the walk.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- tree-walker cursor: `node` advances via `walker.nextNode()`
  let node = walker.nextNode();
  while (node !== null) {
    /**
     * Raw text of the current node; empty string falls back when textContent is null.
     */
    const text = node.textContent
      ?? '';
    /**
     * Lowercase version of `text` for case-insensitive `indexOf` comparisons.
     */
    const lowerText = text.toLowerCase();
    /**
     * Cursor advanced past each match so overlapping matches are not re-reported.
     */
    let searchFrom = 0;

    // oxlint-disable-next-line -- indexOf returns -1 when not found; loop terminates correctly
    for (;;) {
      const index = lowerText.indexOf(
        lowerQuery,
        searchFrom,
      );
      if (index === (-1))
        break;

      /**
       * DOM Range covering this match so the Custom Highlight API can style it.
       */
      const range = new Range();
      range.setStart(
        node,
        index,
      );
      range.setEnd(
        node,
        index + queryLength,
      );
      ranges.push(range,);
      searchFrom = index + queryLength;
    }

    node = walker.nextNode();
  }

  if (ranges.length
    > 0) {
    CSS.highlights
      .set(
      'hl-search-match',
      new Highlight(...ranges,),
    );
  }
  else {
    CSS.highlights
      .delete('hl-search-match',);
  }
}
