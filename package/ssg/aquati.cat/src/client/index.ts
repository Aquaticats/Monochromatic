/**
 Client-side entry point for the SSG.
 
 Reads pre-computed syntax highlight ranges from `data-hl-<group>` attributes
 on `<code>` elements and registers them with the CSS Custom Highlight API
 for styling via `::highlight()` pseudo-elements.
 
 Lezer parsing happens at build time via the rehype-highlight plugin.
 This script only maps pre-computed character offsets to DOM Range objects.
 
 Degrades gracefully: if the CSS Custom Highlight API is unavailable
 or a code block has no pre-computed data, it remains unstyled plain text.
 
 @example
 ```html
 <script type="module" src="/client/index.js"></script>
 ```
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { HIGHLIGHT_GROUPS, } from './highlight-groups.ts';
// oxlint-disable-next-line no-unassigned-import -- side-effect import: registers DOM event listeners for the search widget on module load
import './search.ts';
// oxlint-disable-next-line no-unassigned-import -- side-effect import: shuffles <shuffle-children> direct children at load time when CSS random() is unsupported
import './shuffle-children.ts';

 // module boundary marker

//region Offset-to-Range mapping

/**
 Flattened text node with its character offset within the parent element.
 */
type TextEntry = {
  readonly node: Text;
  readonly start: number;
};

/**
 Collects all text nodes and their cumulative offsets within an element.
 
 @param element - container element to walk
 
 @mutates element through document.createTreeWalker root retention and traversal
 
 @returns array of text nodes with their start offsets
 */
function collectTextNodes(element: HTMLElement,): TextEntry[] {
  /**
   Sorted list of text nodes with cumulative offsets returned to the caller.
   */
  const entries: TextEntry[] = [];
  /**
   Tree walker pinned to text nodes so the SHOW_TEXT cast at the loop body is sound.
   */
  const walker = document.createTreeWalker(
    element,
    NodeFilter.SHOW_TEXT,
  );
  /**
   Running cumulative offset; mutated as nodes are visited.
   */
  let offset = 0;
  /**
   Loop cursor advanced through the tree walker.
   */
  let current = walker.nextNode();
  while (current !== null) {
    /* oxlint-disable no-unsafe-type-assertion -- TreeWalker with SHOW_TEXT guarantees Text nodes */
    /**
     Narrowed text node used for the length update and offset tracking.
     */
    const textNode = current as Text;
    /* oxlint-enable no-unsafe-type-assertion */
    entries.push({
      node: textNode,
      start: offset,
    },);
    offset += textNode.length;
    current = walker.nextNode();
  }
  return entries;
}

/**
 Finds the index of the first text entry whose end offset exceeds `from`
 using binary search over the sorted `textEntries` array.
 
 @param textEntries - sorted text nodes with cumulative offsets
 
 @param from - character offset to search for
 
 @returns index of the first potentially overlapping text entry
 */
function findFirstOverlap(
  {
    textEntries,
    from,
  }: {
    readonly textEntries: readonly TextEntry[];
    readonly from: number;
  },
): number {
  /**
   Inclusive lower binary-search bound.
   */
  let lo = 0;
  /**
   Exclusive upper binary-search bound.
   */
  let hi = textEntries.length;
  while (lo < hi) {
    /**
     Unsigned-shift midpoint avoids overflow on long entry lists.
     */
    const mid = (lo + hi) >>> 1;
    /**
     Entry inspected at the midpoint of the current search span.
     */
    const entry = nonNullishOrThrow(textEntries[mid],);
    if ((entry.start
      + entry
        .node
        .length) <= from)
      lo = mid + 1;
    else
      hi = mid;
  }
  return lo;
}

/**
 Creates DOM Range objects from encoded offset pairs, mapping character
 offsets to the appropriate text nodes within the code element.
 
 Uses binary search to find the first overlapping text node for each
 pair, then scans forward only through overlapping nodes.
 
 @param textEntries - pre-collected text nodes with cumulative offsets
 
 @param encoded - semicolon-separated `from-to` offset pairs (e.g. `"0-5;15-21"`)
 
 @returns array of DOM Range objects spanning the highlighted tokens
 */
function createRangesFromPairs({
  textEntries,
  encoded,
}: {
  readonly textEntries: readonly TextEntry[];
  readonly encoded: string;
},): Range[] {
  /**
   Accumulated DOM Range list returned to the caller.
   */
  const ranges: Range[] = [];

  for (const pair of encoded.split(';',)) {
    /**
     Position of the hyphen splitting `from-to`; `-1` means malformed.
     */
    const dashIndex = pair.indexOf('-',);
    if (dashIndex === (-1))
      continue;

    /**
     Substring before the hyphen used as the start offset.
     */
    const fromStr = pair.slice(
      0,
      dashIndex,
    );
    /**
     Substring after the hyphen used as the end offset.
     */
    const toStr = pair.slice(dashIndex + 1,);
    /**
     Numeric start offset measured in code units.
     */
    const from = Number(fromStr,);
    /**
     Numeric end offset measured in code units.
     */
    const to = Number(toStr,);

    /**
     First text-entry index that may overlap `from`, located via binary search.
     */
    const startIdx = findFirstOverlap({
      textEntries,
      from,
    },);

    for (let loopIndex = startIdx; loopIndex < textEntries
      .length; loopIndex++) {
      /**
       Text entry inspected at the current scan index.
       */
      const entry = nonNullishOrThrow(textEntries[loopIndex],);
      if (entry.start
        >= to)
        break;

      /**
       Range-local start within `entry.node` clamped to zero.
       */
      const rangeStart = Math.max(
        0,
        from - entry
          .start,
      );
      /**
       Range-local end within `entry.node` clamped to the node length.
       */
      const rangeEnd = Math.min(
        entry.node
          .length,
        to - entry
          .start,
      );

      /**
       Fresh DOM Range whose endpoints are set to the computed local offsets.
       */
      const range = new Range();
      range.setStart(
        entry.node,
        rangeStart,
      );
      range.setEnd(
        entry.node,
        rangeEnd,
      );
      ranges.push(range,);
    }
  }

  return ranges;
}

//endregion Offset-to-Range mapping

//region Highlight registration

/**
 Registers pre-computed syntax highlights for all code blocks on the page.
 
 Finds `<pre><code>` elements with `data-hl-*` attributes for every group in
 {@link HIGHLIGHT_GROUPS}, maps the encoded offset pairs to DOM Range
 objects, and registers them with the CSS Custom Highlight API.
 
 Ranges from all code blocks are merged into shared per-group highlights
 so a single `::highlight(hl-keyword)` rule styles all keywords site-wide.
 */
function highlightAllCodeBlocks(): void {
  if (((typeof CSS) === 'undefined') || (!('highlights' in CSS)))
    return;

  /**
   Code block elements with pre-computed offset data emitted by the SSG.
   */
  const codeBlocks = document.querySelectorAll<HTMLElement>('pre > code',);

  /**
   Accumulated ranges across all code blocks, keyed by highlight group.
   */
  const allRanges = new Map<string, Range[]>();

  for (const codeElement of codeBlocks) {
    /**
     Cached text node entries for this code block, reused across every highlight group.
     */
    const textEntries = collectTextNodes(codeElement,);

    for (const group of HIGHLIGHT_GROUPS) {
      /**
       Serialised offset pairs for the current highlight group; null when absent.
       */
      const encoded = codeElement.getAttribute(`data-hl-${group}`,);
      if ((encoded === null) || (encoded.length
        === 0))
        continue;

      /**
       Decoded DOM ranges for this group's encoded pairs.
       */
      const ranges = createRangesFromPairs({
        textEntries,
        encoded,
      },);
      if (ranges.length
        === 0)
        continue;

      /**
       Existing per-group bucket initialised lazily on first encounter.
       */
      let existing = allRanges.get(group,);
      if (existing === undefined) {
        existing = [];
        allRanges.set(
          group,
          existing,
        );
      }
      existing.push(...ranges,);
    }
  }

  for (const group of HIGHLIGHT_GROUPS) {
    /**
     Highlight registry name following the `hl-<group>` convention.
     */
    const name = `hl-${group}`;
    /**
     Merged range list for the current group, or undefined when no matches were found.
     */
    const ranges = allRanges.get(group,);
    if ((ranges !== undefined) && (ranges.length
      > 0)) {
      CSS.highlights
        .set(
        name,
        new Highlight(...ranges,),
        );
    }
  }
}

//endregion Highlight registration

highlightAllCodeBlocks();
