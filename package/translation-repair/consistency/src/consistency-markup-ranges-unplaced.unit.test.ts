/**
 Tests for what the markup ranges do with a tree whose nodes the parser did not place, or placed against each other: the
 stretches of a node cannot be read, and the resource of a link is lost if it is skipped, so each case is a named
 invariant error and not an empty answer. The parser places every node it reads, so these trees are hand-built.

 Fixtures are cat-themed invention.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { markupRangesOf, } from '../dist/final/node/index.mjs';

/**
 A node of the tree the page grammar makes.
 */
type TreeNode = Parameters<typeof markupRangesOf>[0]['nodes'][number];

/**
 Where a node stands in the English, as the parser writes it.

 @param start - offset where the node starts, none to leave the start unplaced

 @param end - offset where the node ends, none to leave the end unplaced

 @returns Position, for the `position` of a node

 @example
 ```ts
 const position = placed({ start: 0, end: 8, },);
 ```
 */
function placed(
  { start, end, }: { readonly start?: number; readonly end?: number; },
): NonNullable<TreeNode['position']> {
  return {
    start: {
      line: 1,
      column: 1,
      ...((start === undefined) ? {} : { offset: start, }),
    },
    end: {
      line: 1,
      column: 1,
      ...((end === undefined) ? {} : { offset: end, }),
    },
  };
}

/**
 The text of a label, `kit`, which shows a reader a word.

 @param position - where the text stands, none to leave it unplaced

 @returns Text node

 @example
 ```ts
 const text = kitOf({ position: placed({ start: 1, end: 4, },), },);
 ```
 */
function kitOf({ position, }: { readonly position?: TreeNode['position']; },): TreeNode {
  return {
    type: 'text',
    value: 'kit',
    ...((position === undefined) ? {} : { position, }),
  };
}

/**
 The link `[kit](u)` with the parts that stand where the caller says.

 @param position - where the link stands, none to leave it unplaced

 @param label - text of the label

 @returns Link node

 @example
 ```ts
 const link = linkOf({ position: placed({ start: 0, end: 8, },), label: kitOf({},), },);
 ```
 */
function linkOf(
  { position, label, }: { readonly position?: TreeNode['position']; readonly label: TreeNode; },
): TreeNode {
  return {
    type: 'link',
    url: 'u',
    children: [label as Extract<TreeNode, { readonly type: 'text'; }>,],
    ...((position === undefined) ? {} : { position, }),
  };
}

/**
 What reading the markup ranges of some nodes of the English `[kit](u)` throws.

 @param nodes - nodes of the tree

 @returns Error as text

 @example
 ```ts
 const refusal = readingOf({ nodes: [], },); // fails the test, since nothing is thrown
 ```
 */
function readingOf({ nodes, }: { readonly nodes: readonly TreeNode[]; },): string {
  return String(caught(function reading(): unknown {
    return markupRangesOf({ english: '[kit](u)', nodes, },);
  },),);
}

await describe({
  name: 'the markup ranges of a tree with a node the parser did not place',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'READS THE RESOURCE OF A LINK WHOSE LABEL ENDS AT THE LABEL, as the control the refusing tests differ from',
      fn: async () => {
        expect(markupRangesOf({
          english: '[kit](u)',
          nodes: [
            linkOf({
              position: placed({ start: 0, end: 8, },),
              label: kitOf({ position: placed({ start: 1, end: 4, },), },),
            },),
          ],
        },).linkResources,).toEqual([{ start: 4, end: 8, },],);
      },
    },),
    it({
      name: 'REFUSES A NODE OF A KIND OF STRETCH THAT HAS NO START, and one that has no end, instead of leaving its stretch out',
      fn: async () => {
        expect([
          readingOf({ nodes: [{ type: 'tableRow', children: [], },], },),
          readingOf({ nodes: [{ type: 'tableRow', children: [], position: placed({ start: 0, },), },], },),
          readingOf({ nodes: [{ type: 'inlineCode', value: 'kit', position: placed({ end: 8, },), },], },),
        ],).toEqual([
          'ConsistencyInvariantError: unreachable: the tableRow node has no start or no end offset, and the parser places every node it reads',
          'ConsistencyInvariantError: unreachable: the tableRow node has no start or no end offset, and the parser places every node it reads',
          'ConsistencyInvariantError: unreachable: the inlineCode node has no start or no end offset, and the parser places every node it reads',
        ],);
      },
    },),
    it({
      name: 'REFUSES A LINK WITH A LABEL THAT HAS NO START AND ONE THAT HAS NO END, instead of losing its resource',
      fn: async () => {
        expect([
          readingOf({ nodes: [linkOf({ label: kitOf({ position: placed({ start: 1, end: 4, },), },), },),], },),
          readingOf({
            nodes: [
              linkOf({
                position: placed({ start: 0, },),
                label: kitOf({ position: placed({ start: 1, end: 4, },), },),
              },),
            ],
          },),
          readingOf({
            nodes: [
              linkOf({
                position: placed({ end: 8, },),
                label: kitOf({ position: placed({ start: 1, end: 4, },), },),
              },),
            ],
          },),
        ],).toEqual([
          'ConsistencyInvariantError: unreachable: the link node has no start or no end offset, and the parser places every node it reads',
          'ConsistencyInvariantError: unreachable: the link node has no start or no end offset, and the parser places every node it reads',
          'ConsistencyInvariantError: unreachable: the link node has no start or no end offset, and the parser places every node it reads',
        ],);
      },
    },),
    it({
      name: 'REFUSES A LINK WHOSE LABEL ENDS IN A CHILD THE PARSER DID NOT PLACE, since where the label ends is then unknown',
      fn: async () => {
        expect([
          readingOf({
            nodes: [linkOf({ position: placed({ start: 0, end: 8, },), label: kitOf({},), },),],
          },),
          readingOf({
            nodes: [
              linkOf({
                position: placed({ start: 0, end: 8, },),
                label: kitOf({ position: placed({ start: 1, },), },),
              },),
            ],
          },),
        ],).toEqual([
          'ConsistencyInvariantError: unreachable: the label of a link node ends in a child with no end offset, and the parser places every child of a label',
          'ConsistencyInvariantError: unreachable: the label of a link node ends in a child with no end offset, and the parser places every child of a label',
        ],);
      },
    },),
    it({
      name: 'REFUSES A LINK WHOSE LABEL ENDS AFTER THE LINK DOES, naming both offsets',
      fn: async () => {
        expect(readingOf({
          nodes: [
            linkOf({
              position: placed({ start: 0, end: 8, },),
              label: kitOf({ position: placed({ start: 1, end: 12, },), },),
            },),
          ],
        },),).toBe(
          'ConsistencyInvariantError: unreachable: the label of a link node ends at 12, after the link ends at 8, and a label ends inside its link',
        );
      },
    },),
  ],
},);
