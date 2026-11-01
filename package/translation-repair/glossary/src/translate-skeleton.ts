import type {
  Root,
  RootContent,
} from 'mdast';

import {
  type AtomKind,
  type DeepReadonlyData,
  type LoneContainerTag,
  parseSliceBody,
  type ProtectedAtom,
  requireMdxRefusal,
  withMdxExhaustionRefused,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { explicitBreakCounts, } from './source-only-breaks.ts';


//region Translate skeleton
// Reads one slice, source or candidate translation, into the shape a
// translation has to carry across languages: the sequence of blocks, and the
// references and code inside them.
//
// Shares `parseMdxBody` with document parsing rather than scanning Markdown by
// hand, so this cannot disagree with the parser about what a link is. That is
// the same reason `inspect-paragraph.ts` gives, and this is the whole-slice
// cousin of it.
//
// READ UNDER THE GRAMMAR THE DOCUMENT WAS READ IN. `parse-document.ts` masks
// HTML comments to same-length whitespace before its strict parse, because the
// MDX grammar refuses `<!--` outright and 17 of 92 pinned sources carry one.
// Until 2026-09-06 this reader did not, so every slice of yulianNyanner whose
// original carried one of its author's own comments was "an original that
// could not be read", the floor answered `unknown`, and both gates that consume
// the floor treat `unknown` as inadmissible: the entry stopped at consolidation
// with nothing to ship. Masking here makes a comment inter-node whitespace on
// both sides of the comparison, and the mask keeps every offset, so atoms after
// a comment still index the unmasked text. Whether a candidate carries such a
// comment rendered is wording, and the judges are the instrument for wording;
// the archive itself renders yulianNyanner's fourteen comment lines as twelve.
// A note between translators is another kind: a comment in an archive whose
// words hold Han and that its original lacks. `translators-note.ts` in the
// document-structure package cuts those from the lane contest, gate and
// selection sheets, and the page assembly carries the archive's own, so no
// judge weighs them.
//
// THE SAME FOR A CONTAINER TAG WITHOUT ITS PARTNER. `container-extents.ts`
// gives a container's opening tag to the first block inside it and its closing
// tag to the last, so when the blocks fall in different slices each slice
// owns one half, and the strict grammar refuses either half alone. The
// Huasheng pass of 2026-09-07 stopped at slices 9 and 12 that way. A lone tag
// is masked here (`mask-container-tags.ts`) and carried as a `container-tag`
// atom, so the rest of the slice reads as the document did and a candidate
// that drops the tag fails the floor rather than the page grammar.
//
// WHAT IS DELIBERATELY NOT HERE: prose atoms. `scanTextAtoms` protects numbers
// and foreign runs, which is right when a rewrite and its base are the same
// language and wrong across a translation, where 三只猫 becomes "three cats"
// and no digit survives on either side. Structure and references survive
// translation; wording does not, and the judges are the instrument for wording.

/**
 Atom kinds that must survive a translation.

 Every one of them is a machine-readable identity rather than prose: a URL, a
 reference label, a footnote marker, or code the author fenced precisely so it
 would not be rewritten.

 A DELIBERATE SUBSET of `AtomKind`. LEFT OUT are `number` and `foreign-run`,
 the prose atoms: across a translation 三只猫 becomes "three cats" and no digit
 survives on either side, so wording is judged by the judges rather than
 protected. An atom kind gained later is therefore not carried through a
 translation until it is added here.
 */
const TRANSLATABLE_ATOM_KINDS: ReadonlySet<AtomKind> = new Set<AtomKind>([
  'link-url',
  'image-url',
  'reference',
  'footnote',
  'inline-code',
  'container-tag',
],);

/**
 One top-level block's shape.

 @example
 ```ts
 const shape: BlockShape = { kind: 'heading', detail: 'level 2', };
 ```
 */
export type BlockShape = {
  /**
   mdast node type, kept as a plain string because remark plugins extend the
   vocabulary.
   */
  readonly kind: string;

  /**
   What distinguishes two blocks of the same kind, empty when nothing does.

   A heading carries its level and a list carries whether it is ordered,
   because a translation that turns a level-two heading into a level-three one
   or a bulleted list into a numbered one has changed the document while
   matching on kind alone.
   */
  readonly detail: string;
};

/**
 What one slice carries across a translation.

 @example
 ```ts
 const skeleton: SliceSkeleton = { blocks, atoms, explicitBreaks, quotedPassages, };
 ```
 */
export type SliceSkeleton = {
  /**
   Top-level blocks in document order.
   */
  readonly blocks: readonly BlockShape[];

  /**
   References and code in document order.
   */
  readonly atoms: readonly ProtectedAtom[];

  /**
   Explicit rendered breaks within each corresponding top-level block.
   */
  readonly explicitBreaks: readonly number[];

  /**
   Blockquotes at every depth: at the top level, inside a container tag, a
   list or a footnote, and inside another blockquote.

   READ OFF THE PARSE, as the blocks are (ledger B42). The quote guard once
   counted blank-line-separated chunks opening with `>`, so a quote opening
   on the line after a paragraph's, which the parser and the floor read as a
   blockquote, counted as none, and a replacement keeping it was refused.
   */
  readonly quotedPassages: number;
};

/**
 Outcome of reading one slice.

 @example
 ```ts
 const read: SkeletonRead = { kind: 'unparseable', detail: 'unexpected `{`', };
 ```
 */
export type SkeletonRead =
  | {
    readonly kind: 'read';

    /**
     What the slice carries.
     */
    readonly skeleton: SliceSkeleton;
  }
    | {
      readonly kind: 'unparseable';

    /**
     Parser's own account, for a finding a model can act on.
     */
      readonly detail: string;
    };

/**
 Recursively readonly mdast root, as this module BORROWS the parse result.

 @example
 ```ts
 const root: ReadonlyMdastRoot = parsed.root;
 ```
 */
type ReadonlyMdastRoot = DeepReadonlyData<Root>;

/**
 Recursively readonly mdast content node.

 @example
 ```ts
 const node: ReadonlyMdastContent = root.children[0];
 ```
 */
type ReadonlyMdastContent = DeepReadonlyData<RootContent>;

/**
 @internal

 Names what distinguishes this block from another of the same kind.

 @param node - top-level mdast block

 @returns Distinguishing detail, empty when the kind says everything

 @example
 ```ts
 const detail = blockDetail({ node, },);
 ```
 */
export function blockDetail({ node, }: { readonly node: ReadonlyMdastContent; },): string {
  if (node.type === 'heading')
    return `level ${String(node.depth,)}`;
  if (node.type === 'list')
    return (node.ordered === true) ? 'ordered' : 'bulleted';
  return '';
}

/**
 Reads one node into the atoms it contributes, if any.

 @param node - mdast node at any depth

 @returns Atoms this node contributes

 @example
 ```ts
 const atoms = atomsOfNode({ node, },);
 ```
 */
function atomsOfNode(
  { node, }: { readonly node: ReadonlyMdastContent; },
): readonly ProtectedAtom[] {
  if (node.type === 'inlineCode')
    return [{
      kind: 'inline-code',
      value: node.value,
    },];
  if (node.type === 'link')
    return [{
      kind: 'link-url',
      value: node.url,
    },];
  if (node.type === 'image')
    return [{
      kind: 'image-url',
      value: node.url,
    },];
  if ((node.type === 'linkReference') || (node.type === 'imageReference'))
    return [{
      kind: 'reference',
      value: node.identifier,
    },];
  if (node.type === 'footnoteReference')
    return [{
      kind: 'footnote',
      value: node.identifier,
    },];
  if (node.type === 'footnoteDefinition')
    return [{
      kind: 'footnote',
      value: node.identifier,
    },];
  return [];
}

/**
 @internal

 Walks a parsed slice into its ordered atoms.

 EXPORTED FOR THE UNWRAPPED-LINK FLOOR (ledger B23), which reads a rendering
 the strict grammar refuses under plain markdown and needs the same reading
 of a link destination this walk gives the strict tree.

 @param root - parsed slice, under either grammar

 @returns Atoms in document order

 @example
 ```ts
 const atoms = walkAtoms({ root, },);
 ```
 */
export function walkAtoms({ root, }: { readonly root: ReadonlyMdastRoot; },): readonly ProtectedAtom[] {
  /**
   Nodes still to visit, held as a stack so the walk stays iterative over a
   tree of unknown depth; children push reversed to keep document order.
   */
  const pending: ReadonlyMdastContent[] = [...root.children,].toReversed();

  /**
   Atoms in document order.
   */
  const atoms: ProtectedAtom[] = [];
  // Next node in document order, until the stack is empty.
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    atoms.push(
      ...atomsOfNode({ node, },)
        .filter(function survivesTranslation(atom,): boolean {
          return TRANSLATABLE_ATOM_KINDS.has(atom.kind,);
        },),
    );
    if ('children' in node)
      pending.push(...[...node.children,].toReversed(),);
  }
  return atoms;
}

/**
 @internal

 Counts the blockquotes of a parsed slice at every depth.

 A QUOTE INSIDE A CONTAINER COUNTS, which is the reach the top-level block
 floor lacks: 20 of the 8,295 distinct texts of the stored translate records
 carry a quote below the top level (ledger B42). A quote inside a
 quote counts too, so turning a quoted reply into the letter's own words
 is a quoted passage lost, as turning a top-level quote into a paragraph is
 for the floor.

 EXPORTED FOR THE PAGE READING, which counts the plain-markdown tree the
 same way.

 @param root - parsed slice, under either grammar

 @returns Blockquote count

 @example
 ```ts
 const quotedPassages = quotedPassageCount({ root, },);
 ```
 */
export function quotedPassageCount({ root, }: { readonly root: ReadonlyMdastRoot; },): number {
  /**
   Nodes still to visit, held as a stack so the walk stays iterative over a
   tree of unknown depth; order does not matter to a count.
   */
  const pending: ReadonlyMdastContent[] = [...root.children,];

  /**
   Blockquotes met so far.
   */
  const quotes: ReadonlyMdastContent[] = [];
  // Next node, until the stack is empty.
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if (node.type === 'blockquote')
      quotes.push(node,);
    if ('children' in node)
      pending.push(...node.children,);
  }
  return quotes.length;
}

/**
 Lone container tags of one kind as atoms, in document order.

 @param tags - lone tags the mask reported

 @param kind - which half of a container to keep

 @returns Those tags as `container-tag` atoms

 @example
 ```ts
 const openers = tagAtomsOf({ tags, kind: 'open', },);
 ```
 */
function tagAtomsOf(
  {
    tags,
    kind,
  }: {
    readonly tags: readonly LoneContainerTag[];
    readonly kind: 'open' | 'close';
  },
): readonly ProtectedAtom[] {
  return tags
    .filter(function ofKind(tag,): boolean {
      return tag.kind === kind;
    },)
    .map(function toAtom(tag,): ProtectedAtom {
      return {
        kind: 'container-tag',
        value: tag.text,
      };
    },);
}

/**
 Reads one slice into the shape a translation of it has to match.

 @param text - exact slice source, original or candidate

 @returns Blocks and atoms, or the parser's refusal

 @example
 ```ts
 const read = readSliceSkeleton({ text: candidate, },);
 ```
 */
export function readSliceSkeleton(
  { text, }: { readonly text: string; },
): SkeletonRead {
  try {
    /**
     Shared offset-preserving slice grammar also used by source presentation.
     */
    const {
      root,
      tags,
    } = withMdxExhaustionRefused({
      parse: parseSliceBody,
      input: { text, },
    },);

    return {
      kind: 'read',
      skeleton: {
        explicitBreaks: explicitBreakCounts({ root, },),
        quotedPassages: quotedPassageCount({ root, },),
        blocks: root.children
          .map(function toShape(node,): BlockShape {
            return {
              kind: node.type,
              detail: blockDetail({ node, },),
            };
          },),
        // Openers before the content and closers after it, which is where
        // their extents put them.
        atoms: [
          ...tagAtomsOf({
            tags,
            kind: 'open',
          },),
          ...walkAtoms({ root, },),
          ...tagAtomsOf({
            tags,
            kind: 'close',
          },),
        ],
      },
    };
  }
  catch (error) {
    // Only the grammar's own rejection becomes a refusal; anything else is an
    // unexpected state that must keep propagating.
    return {
      kind: 'unparseable',
      detail: String(requireMdxRefusal({ error, },),),
    };
  }
}

//endregion Translate skeleton
