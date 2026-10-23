/**
 Tests for the plain-markdown reading with the site's math extension, which a caller reads structure from.

 The reading is CommonMark and remark-math, with no MDX extensions and no GFM, so it never refuses a text for its
 syntax, and every node it builds carries a position (the GFM autolink-literal transform builds nodes without one).
 A caller that copies or rewrites a text by node positions needs both.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import type {
  Nodes,
  Root,
} from 'mdast';

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  MarkdownParseError,
  parseMarkdownMathBody,
} from '../dist/final/node/index.mjs';

/**
 Every node of a tree, the root first, in document order.

 @param node - subtree to list

 @returns The node and its descendants

 @example
 ```ts
 const nodes = nodesOf({ node: root, },);
 ```
 */
function nodesOf({ node, }: { readonly node: Nodes; },): readonly Nodes[] {
  return [
    node,
    ...('children' in node)
      ? node.children.flatMap(function underNode(child,): readonly Nodes[] {
        return nodesOf({ node: child, },);
      },)
      : [],
  ];
}

/**
 Nodes of a tree with the given type.

 @param root - tree to search

 @param type - node type

 @returns Matching nodes in document order

 @example
 ```ts
 const [math,] = ofType({ root, type: 'inlineMath', },);
 ```
 */
function ofType({ root, type, }: { readonly root: Root; readonly type: string; },): readonly Nodes[] {
  return nodesOf({ node: root, },).filter(function isType(node,): boolean {
    return node.type === type;
  },);
}

await describe({
  name: parseMarkdownMathBody.name,
  children: [
    it({
      name: 'READS TEXT THE MDX GRAMMAR REFUSES WITHOUT REFUSING IT, a bare brace, an unclosed tag and an angle '
        + 'autolink among it, as text, html and a link',
      fn: async () => {
        /**
         Nodes of a body holding a brace, a tag and an autolink.
         */
        const root = parseMarkdownMathBody({ body: '她说 { 然后 <details>\n\n<https://example.net/a>\n', },);
        expect(nodesOf({ node: root, },).map(function typeOf(node,): string {
          return node.type;
        },),).toEqual(['root', 'paragraph', 'text', 'html', 'paragraph', 'link', 'text',],);
      },
    },),

    it({
      name: 'READS MATH THE WAY THE SITE DOES, so a less-than sign and a brace between dollar signs are math and '
        + 'not text, inline and as a block',
      fn: async () => {
        /**
         Body with an inline formula and a block formula, each holding a less-than sign and a brace.
         */
        const root = parseMarkdownMathBody({ body: '猫 $a < b {c}$ 走\n\n$$\nx < {y}\n$$\n', },);
        expect(ofType({ root, type: 'inlineMath', },).length,).toBe(1,);
        expect(ofType({ root, type: 'math', },).length,).toBe(1,);
        expect(ofType({ root, type: 'html', },).length,).toBe(0,);
      },
    },),

    it({
      name: 'GIVES EVERY NODE A POSITION, a trailing `www.` and a bare address holding a brace among the texts that '
        + 'make the GFM reading drop them, counted in UTF-16 units after an astral character',
      fn: async () => {
        /**
         Bodies where the GFM autolink-literal transform leaves nodes without a position.
         */
        const bodies = ['x {a} www.', '猫 https://example.net/b{c} 走', '\u{1F408} {d}',];
        for (const body of bodies) {
          for (const node of nodesOf({ node: parseMarkdownMathBody({ body, },), },))
            expect(node.position,).not.toBeUndefined();
        }
        /**
         The text node after an astral character, whose start is two units in.
         */
        const [text,] = ofType({ root: parseMarkdownMathBody({ body: '\u{1F408} {d}', },), type: 'text', },);
        expect(text?.position?.start.offset,).toBe(0,);
        expect(text?.position?.end.offset,).toBe('\u{1F408} {d}'.length,);
      },
    },),

    it({
      name: 'READS NO GFM: a footnote definition is a paragraph and a bare address stays text',
      fn: async () => {
        /**
         Body with a footnote definition and a bare address.
         */
        const root = parseMarkdownMathBody({ body: '[^1]: 注\n\nhttps://example.net/a\n', },);
        expect(ofType({ root, type: 'footnoteDefinition', },).length,).toBe(0,);
        expect(ofType({ root, type: 'link', },).length,).toBe(0,);
      },
    },),

    it({
      name: 'REFUSES A BODY NESTED PAST THE BOUND as the plain grammar does, with the same error class',
      fn: async () => {
        expect(caught(function parseTooDeep(): void {
          parseMarkdownMathBody({ body: `${'>'.repeat(257,)} cat`, },);
        },),).toBeInstanceOf(MarkdownParseError,);
        expect(parseMarkdownMathBody({ body: `${'>'.repeat(256,)} cat`, },).children.length,).toBe(1,);
      },
    },),
  ],
},);
