import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { RootContent, } from 'mdast';

//region Container unwrapping
// Corpus pages wrap material in disclosure elements: an `en` page collapses a
// whole trailing gallery into one `<details>`, while its `zh` counterpart
// carries the same blocks at top level. Because chunking and alignment walk
// TOP-LEVEL blocks only, that page counted 17 blocks against the original's 25,
// alignment degraded to proportional merging, and critics were handed
// non-corresponding material. The graded milestone-three sample scored the
// result as omissions of content that was present all along, merely nested.
//
// Flattening promotes a container's block children to peers so both sides
// expose comparable structure. This walk leaves offsets untouched: a promoted
// child keeps its own absolute span, and the container's opening and closing
// tags fall outside every promoted span.
//
// THOSE TAGS WOULD THEN BELONG TO NO BLOCK, which is why this walk also reports
// them, and why `container-extents.ts` hands each one to the block beside it
// before anything downstream reads a block offset. Leaving them as inter-block
// text was safe only while both tags sat on the same side of every slice
// boundary. `Zha_Ke` showed what happens otherwise: a slice range held the
// `<details>` opener and stopped eleven characters before its closer, so
// assembly replaced the opener away and copied the closer through, losing 3708
// characters of will and leaving markup that closes nothing. No invariant could
// see it, because all of them reason over blocks and a tag in no block is not
// one.
//
// SO THE SPANS REPORTED HERE ARE NOT DECORATION. They are the input that makes
// a tag reachable by the block beside it, and a caller that reads `blocks` and
// drops `containers` reintroduces the defect in full.
//
// Thirty of the pinned corpus's pages carry a disclosure element, so this is
// ordinary structure rather than one document's quirk.

/**
 mdast types that stand alone as document blocks. A container holding any of
 these is structural packaging around content, so its children are the real
 blocks. Anything else (a `<summary>` holding only phrasing content) is a
 block in its own right and stays whole.
 */
const BLOCK_TYPES: ReadonlySet<string> = new Set([
  'paragraph',
  'heading',
  'blockquote',
  'list',
  'code',
  'table',
  'thematicBreak',
  'footnoteDefinition',
  'mdxJsxFlowElement',
],);

/**
 mdast types that can hold block children worth promoting. Restricted to JSX
 flow elements because those are the only containers the pinned corpus uses
 for packaging; list items and block quotes carry meaning of their own and
 must never be dissolved.
 */
const CONTAINER_TYPE = 'mdxJsxFlowElement';

/**
 Where a dissolved container's two tags sit, in body-relative offsets.

 BOTH HALVES ARE REPORTED SEPARATELY rather than as one whole-element span,
 because the damage they permit is asymmetric: a range covering one tag and
 not the other destroys the element, while a range covering both or neither
 leaves it intact. Only the halves can express that.

 @example
 ```ts
 const span: ContainerSpan = {
   name: 'details',
   openerStartOffset: 325,
   openerEndOffset: 335,
   closerStartOffset: 3987,
   closerEndOffset: 3998,
 };
 ```
 */
export type ContainerSpan = {
  /**
   Element name as written, empty for a fragment, so a diagnostic can say
   which element would break.
   */
  readonly name: string;

  /**
   Body-relative start of opening tag, which is where the element starts.
   */
  readonly openerStartOffset: number;

  /**
   Body-relative exclusive end of opening tag, which is where its first
   promoted child starts.
   */
  readonly openerEndOffset: number;

  /**
   Body-relative start of closing tag, which is where its last promoted child
   ends.
   */
  readonly closerStartOffset: number;

  /**
   Body-relative exclusive end of closing tag, which is where the element
   ends.
   */
  readonly closerEndOffset: number;
};

/**
 Blocks a walk produced, beside the containers it dissolved to get them.

 @example
 ```ts
 const { blocks, containers, } = flattenContainers({ children: root.children, },);
 ```
 */
export type FlattenedBlocks = {
  /**
   Blocks in source order with containers replaced by their children.
   */
  readonly blocks: readonly ForeignBorrowed<RootContent>[];

  /**
   Every container dissolved along the way, in source order.
   */
  readonly containers: readonly ContainerSpan[];
};

/**
 Raised when a container due to be dissolved carries no offsets.

 Parsed trees always carry positions, so this means a constructed tree reached
 a walk that reports spans. Reporting nothing for it would be worse than
 refusing: the tags would go back to belonging to no block and being guarded
 by nothing, which is the defect this walk exists to close.

 @example
 ```ts
 throw new UnpositionedContainerError({ name: 'details', },);
 ```
 */
export class UnpositionedContainerError extends Error {
  /**
   Declares this message safe to forward: it names a container tag.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the failure naming which element could not be located.

   @param name - element name as written, empty for a fragment

   @example
   ```ts
   throw new UnpositionedContainerError({ name: 'BlurBlock', },);
   ```
   */
  public constructor({ name, }: { readonly name: string; },) {
    super(
      `container ${name === '' ? '<>' : name} carries block children and no position offsets, `
        + 'so its opening and closing tags cannot be located and nothing downstream could keep '
        + 'a slice boundary from falling between them',
    );
    this.name = 'UnpositionedContainerError';
  }
}

/**
 Whether a node carries the position offsets every later stage anchors
 against. Parsed trees always do; a constructed one does not, and promoting
 an unpositioned child would produce a block that cannot anchor an issue.

 @param node - candidate mdast node

 @returns Whether both start and end offsets are present

 @example
 ```ts
 if (isPositioned(child,)) { }
 ```
 */
function isPositioned(node: ForeignBorrowed<RootContent>,): boolean {
  return ((node.position
    ?.start
    .offset)
    !== undefined)
    && ((node.position
      .end
      .offset)
      !== undefined);
}
/**
 A JSX flow element carrying block children. mdast's own `RootContent` union
 has no member for JSX elements (they arrive from the MDX extension), so the
 shape is named structurally, which is also all the walk needs.

 @example
 ```ts
 const container = node as BlockContainer;
 ```
 */
type BlockContainer = {
  /**
   Element name as written.

   DECLARED OPTIONAL RATHER THAN NULLABLE, though the MDX extension does write
   null here for a fragment. The property is foreign, so its absence is read
   at the boundary with a type check rather than modelled as a union this repo
   does not use.
   */
  readonly name?: string;

  /**
   Blocks the container packages, in source order.
   */
  readonly children: readonly ForeignBorrowed<RootContent>[];
};

/**
 Whether a value read off a foreign node is a list of mdast nodes.

 A PREDICATE RATHER THAN AN ASSERTION, which is the whole point of it. The
 `in` check that proves `children` exists types it `unknown`, and naming the
 element type with `as` claims two things at once: that the value is an array,
 which nothing has checked, and that its elements are borrowed-foreign, which
 the marker exists to carry. Splitting the array check out proves the first at
 runtime and lets the predicate carry the second, so neither
 `no-unsafe-type-assertion` nor `prefer-readonly-parameter-types` has anything
 to object to.

 @param value - property value read from a foreign node

 @returns Whether it is an array this walk can read as nodes

 @example
 ```ts
 if (isNodeList(children,)) { }
 ```
 */
function isNodeList(value: unknown,): value is BlockContainer['children'] {
  return Array.isArray(value,);
}

/**
 Whether a node is a container whose children should be promoted: a JSX flow
 element holding at least one real block, every child positioned. A
 self-closing component such as a photo scroll has no children and stays a
 block, which is correct: it is content, not packaging.

 @param node - candidate mdast node

 @returns Whether the node packages blocks rather than being one

 @throws {@link Error} when a JSX flow element has no `children` property,
 which the parser writes on every element

 @example
 ```ts
 if (isUnwrappableContainer(node,)) { }
 ```
 */
function isUnwrappableContainer(
  node: ForeignBorrowed<RootContent>,
): node is ForeignBorrowed<RootContent> & BlockContainer {
  if (node.type !== CONTAINER_TYPE)
    return false;
  // THE PARSER WRITES `children` ON EVERY JSX FLOW ELEMENT, an empty list for
  // a self-closing one, so a node of this type without the property is not a
  // component that stays a block but a node nothing here can read.
  if (!('children' in node))
    throw new Error('unreachable: an MDX flow element node carries no children property, though the parser writes one on every element',);

  /**
   Child blocks the container holds, typed `unknown` by the `in` check that
   proved the property exists. mdast has no member for JSX elements at all,
   which is why nothing about this shape arrives already named.
   */
  const { children, } = node;
  if (!isNodeList(children,))
    return false;

  return (children.length > 0)
    && children.every(isPositioned,)
    && children.some(function isBlock(child: BlockContainer['children'][number],) {
      return BLOCK_TYPES.has(child.type,);
    },);
}

/**
 Reads where a container's two tags sit, from its own span and its children's.

 The tags are never nodes of their own, so they can only be named as what is
 left of the element once its promoted children are taken out of it: the head
 before the first child, and the tail after the last.

 @param container - container about to be dissolved, children positioned

 @returns Both tag spans, body-relative

 @throws {@link UnpositionedContainerError} when the element carries no
 offsets of its own

 @example
 ```ts
 const span = containerSpanOf({ container, },);
 ```
 */
function containerSpanOf(
  {
    container,
  }: {
    readonly container: ForeignBorrowed<RootContent> & BlockContainer;
  },
): ContainerSpan {
  /**
   Element name as written, normalised so a fragment reads as empty rather
   than as absent or as the null the MDX extension writes for one.
   */
  const name = ((typeof container.name) === 'string')
    ? container.name
    : '';
  if (!isPositioned(container,))
    throw new UnpositionedContainerError({ name, },);

  /**
   First promoted child, present because an unwrappable container has one.
   */
  const first = nonNullishOrThrow(container.children
    .at(0,),);

  /**
   Last promoted child, which is the first one again on a single-child
   container.
   */
  const last = nonNullishOrThrow(container.children
    .at(-1,),);
  return {
    name,
    openerStartOffset: nonNullishOrThrow(container.position
      ?.start
      .offset,),
    openerEndOffset: nonNullishOrThrow(first.position
      ?.start
      .offset,),
    closerStartOffset: nonNullishOrThrow(last.position
      ?.end
      .offset,),
    closerEndOffset: nonNullishOrThrow(container.position
      ?.end
      .offset,),
  };
}

/**
 Promotes container children to top-level blocks, repeatedly, so a
 disclosure nested inside a disclosure also flattens. Walks with an explicit
 work stack rather than recursion, and preserves document order: a
 container's children take its place exactly where it stood.

 REPORTS WHAT IT DISSOLVED, because after this walk nothing else can. The tags
 belong to no promoted block, so a later reader handed only the blocks cannot
 tell a page carrying containers from one that never had any.

 @param children - top-level mdast blocks in source order

 @returns Blocks in source order beside every container dissolved to get them

 @throws {@link UnpositionedContainerError} when a dissolved container carries
 no offsets of its own

 @example
 ```ts
 const { blocks, containers, } = flattenContainers({ children: root.children, },);
 ```
 */
export function flattenContainers(
  { children, }: { readonly children: ForeignBorrowed<readonly RootContent[]>; },
): FlattenedBlocks {
  /**
   Blocks still to inspect, in reverse so popping yields document order.
   */
  const pending: ForeignBorrowed<RootContent>[] = [...children,].toReversed();

  /**
   Flattened blocks in document order.
   */
  const flattened: ForeignBorrowed<RootContent>[] = [];

  /**
   Containers dissolved along the way, in the order they were met.
   */
  const dissolved: ContainerSpan[] = [];
  while (pending.length > 0) {
    /**
     Next block in document order, present by the loop condition (ledger T8:
     the guard that broke the loop on none could not run, and is gone).
     */
    const node = nonNullishOrThrow(pending.pop(),);

    if (isUnwrappableContainer(node,)) {
      dissolved.push(containerSpanOf({ container: node, },),);
      // Children replace the container in place, and go back through the
      // stack so a nested container flattens on a later turn.
      pending.push(...node.children
        .toReversed(),);
      continue;
    }
    flattened.push(node,);
  }
  return {
    blocks: flattened,
    // Sorted, because a nested container is met before its parent's later
    // siblings and would otherwise report out of document order.
    containers: dissolved.toSorted(function byOpener(
      left,
      right,
    ) {
      return left.openerStartOffset - right.openerStartOffset;
    },),
  };
}

//endregion Container unwrapping
