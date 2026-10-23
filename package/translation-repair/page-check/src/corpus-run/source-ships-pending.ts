import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Source ships pending
// THE STATE THE SPANS READER KEEPS while it walks the nodes of a parse: a node still to read with where it stands, and,
// for each container, whether the block read last in it is part of a footnote definition.

/**
 A node still to read, and where it stands.
 */
export type Pending = {
  /**
   The node.
   */
  readonly node: TreeNode;

  /**
   Whether its parent is the root of the text being read.
   */
  readonly isRootChild: boolean;

  /**
   Whether it stands inside the text of a link, where a link cannot be written.
   */
  readonly insideLink: boolean;

  /**
   Type of its parent, which says whether it stands among blocks or among inline content.
   */
  readonly parentType: string;

  /**
   Its parent, which keeps whether the block read last in it is part of a footnote definition.
   */
  readonly parent: TreeNode;
};

/**
 Whether the block read last in a container is part of a footnote definition, which a line right after it continues.
 */
export type FootnoteState = {
  /**
   Whether the block is the first line of a footnote definition, or a block indented under one.
   */
  open: boolean;

  /**
   Line the block ends on.
   */
  endLine: number;

  /**
   Whether the block ends in a raw HTML block or indented code, after which a definition line is a lazy line too.
   */
  endsInLiteral: boolean;
};

/**
 What the block read last in a container was, made when the container is first looked at.

 @param footnotes - state of each container met so far, which this adds the container to when it has none yet

 @param parent - the container

 @returns The container's state, the one every later call for the container gives again

 @example
 ```ts
 const state = footnoteStateOf({ footnotes: new Map(), parent: root, },);
 ```
 */
export function footnoteStateOf(
  {
    footnotes,
    parent,
  }: {
    readonly footnotes: Map<TreeNode, FootnoteState>;
    readonly parent: TreeNode;
  },
): FootnoteState {
  /**
   State kept from an earlier look at the container.
   */
  const known = footnotes.get(parent,);
  if (known !== undefined)
    return known;
  /**
   State of a container nothing has been read in.
   */
  const fresh: FootnoteState = {
    open: false,
    endLine: 0,
    endsInLiteral: false,
  };
  footnotes.set(
    parent,
    fresh,
  );
  return fresh;
}

//endregion Source ships pending
