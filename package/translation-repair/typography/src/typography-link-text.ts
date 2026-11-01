import type { Nodes, } from 'mdast';

import {
  type DeepReadonlyData,
  parseSliceBody,
  requireMdxRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Link text
// WHICH LINK TEXT IS PROSE. The text of a link written between brackets is the label a reader reads, so the prose mask of the
// typography rules counts it as prose, with two exceptions: a label that is itself an address (a reader copies it as the address)
// and the text of a shortcut or collapsed reference (it is also the label its definition is looked up by). The prose mask
// (`typography-prose-mask.ts`) asks these three questions of each node it walks.

/**
 A node of the tree the slice grammar reads, which it hands over read-only.
 */
export type TreeNode = DeepReadonlyData<Nodes>;

/**
 Whether a link node is written between brackets, so its text is the label a reader reads and not the address itself.

 @param node - the link node

 @param masked - the text the parser read

 @returns Whether the node starts with an opening bracket; false for an autolink, a bare address and a node with no position

 @example
 ```ts
 const labelled = writesLabel({ node, masked: '[Peter](https://x.example)', },);
 ```
 */
function writesLabel(
  {
    node,
    masked,
  }: {
    readonly node: TreeNode;
    readonly masked: string;
  },
): boolean {
  /**
   Offset where the node starts, none for a node the autolink transform built.
   */
  const start = node.position
    ?.start
    .offset;
  if (start === undefined)
    return false;
  return masked.charAt(start,) === '[';
}

/**
 Whether a node is a shortcut or collapsed reference (`[dogs' bowl]`, `[dogs' bowl][]`), whose text is also the label its
 definition is looked up by. A mark curled in the text no longer matches the definition's label, so the link stops working; the text
 is no prose. The text of a full reference (`[the dogs][b]`) is not its label, and a footnote reference holds no text.

 @param node - a node of the tree

 @returns Whether the node is a reference that resolved to a definition and uses its own text as the label

 @example
 ```ts
 const named = namesItsDefinition({ node, },);
 ```
 */
function namesItsDefinition({ node, }: { readonly node: TreeNode; },): boolean {
  return (node.type === 'linkReference') && (node.referenceType !== 'full');
}

/**
 Whether the label of a link written between brackets is itself an address: one token, with no white space in it, in which the
 autolink-literal reading finds an address (a protocol address, a `www.` address or an email address). The text of such a link is
 the address a reader copies, so it is no prose.

 @param node - the link node, written between brackets

 @param masked - the text the parser read

 @returns Whether the label is a single token that holds an address; false for a label with words, a label the grammar refuses
 and a node with no positioned label

 @throws The error of any failure that is no refusal of the grammar

 @example
 ```ts
 const address = labelIsAddress({ node, masked: '[https://x.example](https://x.example)', },); // true
 ```
 */
function labelIsAddress(
  {
    node,
    masked,
  }: {
    readonly node: TreeNode;
    readonly masked: string;
  },
): boolean {
  if (!('children' in node))
    return false;
  /**
   The children of the link, which are its label.
   */
  const children = node.children as readonly TreeNode[];
  /**
   Where the label starts.
   */
  const start = children.at(0,)
    ?.position
    ?.start
    .offset;
  /**
   Where the label ends.
   */
  const end = children.at(-1,)
    ?.position
    ?.end
    .offset;
  if ((start === undefined) || (end === undefined))
    return false;
  /**
   The label as written.
   */
  const label = masked.slice(
    start,
    end,
  );
  // A label of words has white space in it; one without is a single token, and an address the reading finds in it makes the whole
  // token the address (`a'b@c.example`: the reading finds `b@c.example`, and the apostrophe belongs to the local part of the
  // address a reader copies).
  if (Array.from(label,)
    .some(function isSpace(character,): boolean {
      return character.trim() === '';
    },))
    return false;
  try {
    /**
     The parse of the label alone.
     */
    const { root, } = parseSliceBody({ text: label, },);
    /**
     The one block the label parses to, and what follows it.
     */
    const [block, ...others] = root.children;
    if ((block?.type !== 'paragraph') || (others.length > 0))
      return false;
    return block.children
      .some(function isLink(inline,): boolean {
        return inline.type === 'link';
      },);
  }
  catch (error) {
    // Only the grammar's own refusal says the label is no address; anything else keeps propagating.
    requireMdxRefusal({ error, },);
    return false;
  }
}

/**
 Whether the text nodes under a node are prose, given whether the text nodes over it are: the text of a link written between
 brackets is, unless the label is an address, and the text of a shortcut or collapsed reference is not.

 @param node - a node of the tree that holds children

 @param masked - the text the parser read

 @param prose - whether the text nodes over the node are prose

 @returns Whether the text nodes under the node are prose

 @throws The error of any failure that is no refusal of the grammar

 @example
 ```ts
 const under = childrenAreProse({ node, masked: '[Peter](https://x.example)', prose: true, },); // true
 ```
 */
export function childrenAreProse(
  {
    node,
    masked,
    prose,
  }: {
    readonly node: TreeNode;
    readonly masked: string;
    readonly prose: boolean;
  },
): boolean {
  if (!prose)
    return false;
  if (node.type !== 'link')
    return !namesItsDefinition({ node, },);
  return writesLabel({
    node,
    masked,
  },) && (!labelIsAddress({
    node,
    masked,
  },));
}

//endregion Link text
