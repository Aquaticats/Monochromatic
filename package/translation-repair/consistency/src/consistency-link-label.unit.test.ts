/**
 Tests for what the label of a link, a reference or an image shows a reader (`consistency-link-label.ts`, n168): nothing, marks and
 spaces that end in no closing mark and hold no letter or digit, or anything else. The nodes are written by hand, as the page
 grammar makes them; the stretches the whole reading finds in an English are tested in `consistency-markup-ranges.unit.test.ts`.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isLinkNode,
  linkLabelKindOf,
} from '../dist/final/node/index.mjs';

/**
 A node of the tree the page grammar makes.
 */
type TreeNode = Parameters<typeof linkLabelKindOf>[0]['node'];

/**
 A link whose label holds some nodes.

 @param children - nodes of the label

 @returns Link node

 @example
 ```ts
 const node = linkOf({ children: [{ type: 'text', value: 'x', },], },);
 ```
 */
function linkOf({ children, }: { readonly children: Extract<TreeNode, { readonly type: 'link'; }>['children']; },): TreeNode {
  return {
    type: 'link',
    url: 'u',
    children,
  };
}

/**
 A link whose label is one text.

 @param value - what the text says

 @returns Link node

 @example
 ```ts
 const node = linkTextOf({ value: '-', },);
 ```
 */
function linkTextOf({ value, }: { readonly value: string; },): TreeNode {
  return linkOf({
    children: [
      {
        type: 'text',
        value,
      },
    ],
  },);
}

await describe({
  name: linkLabelKindOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'SAYS EMPTY for a link and a link reference with no label, and for an image and an image reference with no alt text',
      fn: async () => {
        expect([
          linkLabelKindOf({ node: linkOf({ children: [], },), },),
          linkLabelKindOf({
            node: {
              type: 'linkReference',
              identifier: 'r',
              referenceType: 'full',
              children: [],
            },
          },),
          linkLabelKindOf({
            node: {
              type: 'image',
              url: 'u',
              alt: '',
            },
          },),
          linkLabelKindOf({
            node: {
              type: 'image',
              url: 'u',
            },
          },),
          linkLabelKindOf({
            node: {
              type: 'imageReference',
              identifier: 'r',
              referenceType: 'full',
              alt: '',
            },
          },),
          linkLabelKindOf({ node: linkTextOf({ value: '', },), },),
        ],).toEqual(['empty', 'empty', 'empty', 'empty', 'empty', 'empty',],);
      },
    },),

    it({
      name: 'SAYS MARKS for a label of spaces, marks, code or a line break that holds no letter or digit and ends in no closing mark',
      fn: async () => {
        expect([
          linkLabelKindOf({ node: linkTextOf({ value: ' ', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: '-', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: '_', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: '[', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: '(', },), },),
          linkLabelKindOf({
            node: linkOf({
              children: [
                {
                  type: 'emphasis',
                  children: [
                    {
                      type: 'text',
                      value: '-',
                    },
                  ],
                },
              ],
            },),
          },),
          linkLabelKindOf({
            node: linkOf({
              children: [
                {
                  type: 'inlineCode',
                  value: '-',
                },
              ],
            },),
          },),
          linkLabelKindOf({ node: linkOf({ children: [{ type: 'break', },], },), },),
          linkLabelKindOf({
            node: {
              type: 'image',
              url: 'u',
              alt: ' ',
            },
          },),
          linkLabelKindOf({
            node: linkOf({
              children: [
                {
                  type: 'image',
                  url: 'a',
                  alt: ' ',
                },
              ],
            },),
          },),
        ],).toEqual(['marks', 'marks', 'marks', 'marks', 'marks', 'marks', 'marks', 'marks', 'marks', 'marks',],);
      },
    },),

    it({
      name: 'SAYS CLOSING for a label that shows closing round or square brackets and closing curly double quotation marks and nothing '
        + 'else, however many, and for an image whose alt text is such marks (n169)',
      fn: async () => {
        expect([
          linkLabelKindOf({ node: linkTextOf({ value: ']', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: ')', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: '\u{201D}', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: '))', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: ')]\u{201D}', },), },),
          linkLabelKindOf({
            node: {
              type: 'image',
              url: 'u',
              alt: ')',
            },
          },),
        ],).toEqual(['closing', 'closing', 'closing', 'closing', 'closing', 'closing',],);
      },
    },),

    it({
      name: 'SAYS OTHER for a label that holds a letter or a digit in any script the neighbour test reads, and for one that ends in a closing '
        + 'bracket after some other mark, a space or an opening bracket',
      fn: async () => {
        expect([
          linkLabelKindOf({ node: linkTextOf({ value: 'x', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: 'the cat', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: '7', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: 'é', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: '-x-', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: '-]', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: ' )', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: '()', },), },),
          linkLabelKindOf({ node: linkTextOf({ value: 'x]', },), },),
          linkLabelKindOf({
            node: {
              type: 'image',
              url: 'u',
              alt: 'x',
            },
          },),
          linkLabelKindOf({
            node: linkOf({
              children: [
                {
                  type: 'image',
                  url: 'a',
                  alt: 'x',
                },
              ],
            },),
          },),
        ],).toEqual(['other', 'other', 'other', 'other', 'other', 'other', 'other', 'other', 'other', 'other', 'other',],);
      },
    },),

    it({
      name: 'READS THE NODES OF A LABEL IN THE ORDER A READER SEES THEM, so the last character shown decides: code then a closing bracket '
        + 'ends in the bracket (other), a closing bracket then code ends in the code (marks)',
      fn: async () => {
        expect([
          linkLabelKindOf({
            node: linkOf({
              children: [
                {
                  type: 'inlineCode',
                  value: '-',
                },
                {
                  type: 'text',
                  value: ')',
                },
              ],
            },),
          },),
          linkLabelKindOf({
            node: linkOf({
              children: [
                {
                  type: 'text',
                  value: ')',
                },
                {
                  type: 'inlineCode',
                  value: '-',
                },
              ],
            },),
          },),
          linkLabelKindOf({
            node: linkOf({
              children: [
                {
                  type: 'emphasis',
                  children: [
                    {
                      type: 'text',
                      value: '-',
                    },
                    {
                      type: 'text',
                      value: ' ',
                    },
                  ],
                },
                {
                  type: 'text',
                  value: ')',
                },
              ],
            },),
          },),
          linkLabelKindOf({
            node: linkOf({
              children: [
                {
                  type: 'text',
                  value: ')',
                },
                {
                  type: 'emphasis',
                  children: [
                    {
                      type: 'text',
                      value: ' ',
                    },
                    {
                      type: 'text',
                      value: '-',
                    },
                  ],
                },
              ],
            },),
          },),
        ],).toEqual(['other', 'marks', 'other', 'marks',],);
      },
    },),

    it({
      name: 'READS THE CHILDREN OF A NESTED NODE IN THE ORDER A READER SEES THEM, so a closing bracket then a hyphen inside emphasis '
        + 'ends in the hyphen (marks), a hyphen then a closing bracket ends in the bracket (other)',
      fn: async () => {
        /**
         Emphasis holding two texts, in the order given.

         @param values - what the texts say

         @returns Emphasis node
         */
        function emphasisOf({ values, }: { readonly values: readonly string[]; },): Extract<TreeNode, { readonly type: 'emphasis'; }> {
          return {
            type: 'emphasis',
            children: values.map(function textOf(value,): { readonly type: 'text'; readonly value: string; } {
              return { type: 'text', value, };
            },),
          };
        }
        expect([
          linkLabelKindOf({ node: linkOf({ children: [emphasisOf({ values: [')', '-',], },),], },), },),
          linkLabelKindOf({ node: linkOf({ children: [emphasisOf({ values: ['-', ')',], },),], },), },),
          linkLabelKindOf({ node: linkOf({ children: [emphasisOf({ values: [')', ' ', '-',], },),], },), },),
        ],).toEqual(['marks', 'other', 'marks',],);
      },
    },),

    it({
      name: 'SAYS OTHER for a label that holds a node the tree does not say the text of, such as raw markup, even where nothing else is in it',
      fn: async () => {
        expect([
          linkLabelKindOf({
            node: linkOf({
              children: [
                {
                  type: 'html',
                  value: '<b>',
                },
              ],
            },),
          },),
          linkLabelKindOf({
            node: linkOf({
              children: [
                {
                  type: 'footnoteReference',
                  identifier: 'n',
                },
              ],
            },),
          },),
        ],).toEqual(['other', 'other',],);
      },
    },),
    it({
      name: `${isLinkNode.name} IS TRUE for a link, a link reference, an image and an image reference, and FALSE for any other node`,
      fn: async () => {
        expect([
          isLinkNode({ node: linkOf({ children: [], },), },),
          isLinkNode({
            node: {
              type: 'linkReference',
              identifier: 'r',
              referenceType: 'full',
              children: [],
            },
          },),
          isLinkNode({
            node: {
              type: 'image',
              url: 'u',
            },
          },),
          isLinkNode({
            node: {
              type: 'imageReference',
              identifier: 'r',
              referenceType: 'full',
            },
          },),
          isLinkNode({
            node: {
              type: 'text',
              value: 'x',
            },
          },),
          isLinkNode({
            node: {
              type: 'emphasis',
              children: [],
            },
          },),
        ],).toEqual([true, true, true, true, false, false,],);
      },
    },),
  ],
},);
