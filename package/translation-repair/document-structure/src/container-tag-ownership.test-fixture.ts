import type {
  ContainerSpan,
  DocumentNode,
} from '../dist/final/node/index.mjs';

//region Container tag ownership
// WHETHER A PARSED NODE'S OFFSETS FULLY COVER ONE OF A DISSOLVED CONTAINER'S
// TWO TAGS, the opener or the closer, which is how a case confirms every
// container tag landed inside some promoted block rather than in the gap
// between two of them.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The container-extents and unwrap-container
// tests kept their own copy of these checks; all now import them from here.

/**
 Whether a node's offsets fully cover a container's opening tag.

 @param node - parsed node to check

 @param container - container whose opener is sought

 @returns Whether the node's span contains the whole opener

 @example
 ```ts
 const owns = nodeOwnsOpener({ node, container, },);
 ```
 */
export function nodeOwnsOpener(
  {
    node,
    container,
  }: {
    readonly node: DocumentNode;
    readonly container: ContainerSpan;
  },
): boolean {
  return (node.startOffset <= container.openerStartOffset)
    && (node.endOffset >= container.openerEndOffset);
}

/**
 Whether a node's offsets fully cover a container's closing tag.

 @param node - parsed node to check

 @param container - container whose closer is sought

 @returns Whether the node's span contains the whole closer

 @example
 ```ts
 const owns = nodeOwnsCloser({ node, container, },);
 ```
 */
export function nodeOwnsCloser(
  {
    node,
    container,
  }: {
    readonly node: DocumentNode;
    readonly container: ContainerSpan;
  },
): boolean {
  return (node.startOffset <= container.closerStartOffset)
    && (node.endOffset >= container.closerEndOffset);
}

//endregion Container tag ownership
