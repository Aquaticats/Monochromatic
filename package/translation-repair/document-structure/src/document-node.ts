import { createHash, } from 'node:crypto';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { DeepReadonlyData, } from './readonly-data.ts';
import type { RootContent, } from 'mdast';

//region Document node model
// Issues and patches anchor to these nodes: IDs are structural and immutable for a
// given base document, offsets are absolute within full source text, and hashes let
// later stages reject stale anchors (patch operations carry the hash they were built
// against).

/**
 Zone a block-level node belongs to.
 Front matter never appears here (preserved verbatim outside node list);
 footnote definitions separate from body so chunking can pair them with referencing
 blocks.

 @example
 ```ts
 const zone: DocumentZone = 'body';
 ```
 */
export type DocumentZone = 'body' | 'footnote-definition';

/**
 One block-level node of a parsed document,
 carrying everything later stages need to anchor and validate claims against it.

 @example
 ```ts
 const node: DocumentNode = {
   id: 'block/0',
   zone: 'body',
   kind: 'heading',
   text: '## 简介',
   startOffset: 16,
   endOffset: 23,
   contentHash: hashContent({ content: '## 简介', },),
 };
 ```
 */
export type DocumentNode = {
  /**
   Structural identifier `block/<index>` stable for a given base document;
   survives content edits elsewhere because identity comes from position in
   top-level block order, not content.
   */
  readonly id: string;

  /**
   Zone classification driving chunking and guard behavior.
   */
  readonly zone: DocumentZone;

  /**
   mdast node type (`paragraph`, `heading`, `blockquote`, `mdxJsxFlowElement`, ...);
   kept as plain string because remark plugins extend node vocabulary.
   */
  readonly kind: string;

  /**
   Exact source slice for this block; equality with offset-sliced source is
   a validated invariant, not an assumption.
   */
  readonly text: string;

  /**
   Absolute start offset within full document source (front matter included).
   */
  readonly startOffset: number;

  /**
   Absolute end offset (exclusive) within full document source.
   */
  readonly endOffset: number;

  /**
   SHA-256 of text; patch operations carry this to reject stale anchors.
   */
  readonly contentHash: string;
};

//endregion Document node model

//region Node construction

/**
 Hashes content for anchor-staleness detection.

 @param content - exact UTF-8 text to fingerprint

 @returns Lowercase hex SHA-256 digest

 @example
 ```ts
 hashContent({ content: 'paragraph text', },);
 ```
 */
export function hashContent({ content, }: { readonly content: string; },): string {
  return createHash('sha256',)
    .update(
    content,
    'utf8',
    )
    .digest('hex',);
}

/**
 Signals an mdast node missing position offsets;
 remark always emits positions when parsing source text,
 so absence means the tree was constructed rather than parsed and cannot anchor
 issues.

 @example
 ```ts
 throw new UnpositionedNodeError({ kind: 'paragraph', index: 3, },);
 ```
 */
export class UnpositionedNodeError extends Error {
  /**
   Declares this message safe to forward: it names a node type and its index in the tree.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming offending node.

   @param kind - mdast node type lacking positions

   @param index - top-level block index of offending node

   @example
   ```ts
   new UnpositionedNodeError({ kind: 'paragraph', index: 3, },);
   ```
   */
  public constructor(
    {
      kind,
      index,
    }: {
      readonly kind: string;
      readonly index: number;
    },
  ) {
    super(
      `mdast node ${kind} at top-level index ${String(index,)} lacks position offsets;`
        + ' parsed trees always carry positions, so this tree cannot anchor issues.',
    );
    this.name = 'UnpositionedNodeError';
  }
}

/**
 Readonly mdast fields this module reads while constructing document nodes.

 The FULL node view rather than a narrow projection of the fields read here.
 A narrow Pick is assignable from real mdast values but rejects object
 LITERALS through excess-property checking, which broke the footnote-graph
 test fixtures. This view exists to borrow without copying, so it has to
 accept what callers already hold.

 @example
 ```ts
 const child: DocumentNodeChild = root.children[0];
 ```
 */
type DocumentNodeChild = DeepReadonlyData<RootContent>;

/**
 Builds anchor-ready document nodes from top-level mdast children.

 @param children - top-level mdast blocks in source order

 @param bodyText - body source the children were parsed from

 @param bodyOffset - absolute offset of body start within full document source

 @returns Block nodes in source order with absolute offsets and content hashes

 @throws {@link UnpositionedNodeError} when any child lacks position offsets

 @example
 ```ts
 const nodes = buildDocumentNodes({ children: root.children, bodyText: body, bodyOffset, },);
 ```
 */
export function buildDocumentNodes(
  {
    children,
    bodyText,
    bodyOffset,
  }: {
    readonly children: ForeignBorrowed<readonly DocumentNodeChild[]>;
    readonly bodyText: string;
    readonly bodyOffset: number;
  },
): readonly DocumentNode[] {
  return children.map(function toDocumentNode(
    child,
    index,
  ): DocumentNode {
    if ((child.position
      ?.start
      .offset
      === undefined)
      || (child.position
        .end
        .offset
        === undefined))
      throw new UnpositionedNodeError({
        kind: child.type,
        index,
      },);

    /**
     Body-relative start offset, proven present by preceding guard.
     */
    const start = nonNullishOrThrow(child.position
      .start
      .offset,);

    /**
     Body-relative end offset, proven present by preceding guard.
     */
    const end = nonNullishOrThrow(child.position
      .end
      .offset,);

    /**
     Exact source slice backing text, offsets, and hash together.
     */
    const text = bodyText.slice(
      start,
      end,
    );

    return {
      id: `block/${String(index,)}`,
      zone: child.type === 'footnoteDefinition'
        ? 'footnote-definition'
        : 'body',
      kind: child.type,
      text,
      startOffset: bodyOffset + start,
      endOffset: bodyOffset + end,
      contentHash: hashContent({ content: text, },),
    };
  },);
}

//endregion Node construction
