import type {
  Nodes,
  Root,
} from 'mdast';

import {
  type BlockRange,
  footnoteDefinitionRangesOf,
  pageFootnoteDefinitionRangesOf,
} from './gfm-block-ranges.ts';
import { parseMarkdownMathBody, } from './parse-markdown-math.ts';

//region Container footnote ranges
// WHERE THE TWO READINGS OF A FOOTNOTE DISAGREE INSIDE A BLOCK QUOTE OR A LIST ITEM. The owner's two-readings ruling takes
// the source's ranges at the top of a text and the page's inside a container and after a footnote's last block
// (`gfm-block-ranges.ts`). A reader of plain Markdown gives only the source's reading, so a comparison of what the page
// shows with what the source shows cannot say which is right where the ruling takes the page's. This says whether the
// two readings differ at a place a container holds, and where each such place starts, so that a caller can tell which
// block of the text it is in.

/**
 Stretches of a text that a block quote or a list holds.

 @param root - tree of the plain reading

 @returns Offsets of every block quote and list, nested ones included

 @example
 ```ts
 const spans = containerSpansOf({ root: parseMarkdownMathBody({ body: '> a\n', },), },);
 ```
 */
function containerSpansOf({ root, }: { readonly root: Root; },): readonly BlockRange[] {
  /**
   Nodes still to read, a work stack so that a deep tree is no deep recursion.
   */
  const pending: Nodes[] = [root,];
  /**
   Containers found.
   */
  const spans: BlockRange[] = [];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if (((node.type === 'blockquote') || (node.type === 'list')) && (node.position !== undefined)) {
      /**
       Where the container starts and ends.
       */
      const {
        start,
        end,
      } = node.position;
      spans.push({
        start: start.offset ?? 0,
        end: end.offset ?? 0,
      },);
    }
    if ('children' in node)
      pending.push(...node.children,);
  }
  return spans;
}

/**
 The ranges of one list that the other lacks.

 @param ranges - ranges to filter

 @param others - ranges to compare with, by both ends

 @returns Ranges of the first list with no range of the second at the same start and end

 @example
 ```ts
 const only = rangesLacking({ ranges: [{ start: 0, end: 5, },], others: [], },);
 ```
 */
function rangesLacking(
  {
    ranges,
    others,
  }: {
    readonly ranges: readonly BlockRange[];
    readonly others: readonly BlockRange[];
  },
): readonly BlockRange[] {
  return ranges.filter(function lacked(range,): boolean {
    return !others.some(function same(other,): boolean {
      return (other.start === range.start) && (other.end === range.end);
    },);
  },);
}

/**
 Where the source's and the page's footnote ranges differ at a place a block quote or a list holds.

 @param body - text of a run, as the artifact records it

 @returns Offsets, in text order and each once, of the start of each range one reading has and the other lacks that starts
 inside a block quote or a list; none where the two readings agree there

 @example
 ```ts
 const starts = containerFootnoteDisagreementsOf({ body: '> [^1]: x\n> <q>\n> text\n> ===\n', },);
 ```
 */
export function containerFootnoteDisagreementsOf({ body, }: { readonly body: string; },): readonly number[] {
  /**
   The source's ranges.
   */
  const source = footnoteDefinitionRangesOf({ body, },);
  /**
   The page's ranges.
   */
  const page = pageFootnoteDefinitionRangesOf({ body, },);
  /**
   Ranges only one reading has.
   */
  const differing = [
    ...rangesLacking({
      ranges: source,
      others: page,
    },),
    ...rangesLacking({
      ranges: page,
      others: source,
    },),
  ];
  if (differing.length === 0)
    return [];
  /**
   Where the containers of the text stand.
   */
  const containers = containerSpansOf({ root: parseMarkdownMathBody({ body, },), },);
  /**
   Where the differing ranges that a container holds start, a start both readings have counted once.
   */
  const starts = new Set(
    differing
      .filter(function inContainer(range,): boolean {
        return containers.some(function holds(container,): boolean {
          return (container.start <= range.start) && (range.start < container.end);
        },);
      },)
      .map(function startOf({ start, },): number {
        return start;
      },),
  );
  return [...starts,].toSorted(function ascending(
    left,
    right,
  ): number {
    return left - right;
  },);
}

/**
 Whether the source's and the page's footnote ranges differ at a place a block quote or a list holds.

 @param body - text of a run, as the artifact records it

 @returns Whether a range one reading has and the other lacks starts inside a block quote or a list

 @example
 ```ts
 const undecided = footnoteRangesDifferInContainer({ body: '> [^1]: x\n> <q>\n> text\n> ===\n', },);
 ```
 */
export function footnoteRangesDifferInContainer({ body, }: { readonly body: string; },): boolean {
  /**
   Where the two readings disagree inside a container.
   */
  const starts = containerFootnoteDisagreementsOf({ body, },);
  return starts.length > 0;
}

//endregion Container footnote ranges
