import {
  type FootnoteGraphFinding,
  scanGfmReferenceLiterals,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';

//region Orphan definition trim
// AN ORPHAN DEFINITION TAKES ONLY ITS OWN BLOCK. The nineteenth `hakureico`
// pass of 2026-09-09 rendered the source's two footnote definitions as one
// insertion behind the sealed letter; the seal keeps the archive's letter,
// which never carried the second marker, so the second definition was an
// orphan, and the assembly guard withdrew the whole insertion, the first
// definition with it, leaving the body's `[^1]` with no note. A replacement
// that is nothing but definition blocks can lose the orphan's block alone and
// keep the rest, which is what the page needed. The twenty-first class.
//
// A DEFINITION LINE OPENS A BLOCK, blank line or not. The twenty-first pass
// rendered the two notes as `[^1]: …\n[^2]: …`, one paragraph by the blank-line
// reading, and the trim saw one block labelled `1` and left it; the guard then
// withdrew both notes and the reference. GFM ends a definition where the next
// `[^id]:` line begins, so the reading here does too, and the gaps between
// blocks are kept as written so a cut changes nothing beside the cut.
//
// ONLY DEFINITIONS-ONLY REPLACEMENTS ARE TRIMMED. Prose that carries a
// definition among its paragraphs is a judged rendering, and cutting a block
// out of it would ship a text nobody judged; that replacement is withdrawn
// whole, as before.

/**
 Replacements after the trim, and what the trim did.

 @example
 ```ts
 const trimmed: TrimmedReplacements = trimOrphanDefinitions({ findings, replacements, incumbentBySlice, },);
 ```
 */
export type TrimmedReplacements = {
  /**
   Replacements with the orphan blocks cut out of definitions-only texts.
   */
  readonly replacements: readonly SliceReplacement[];

  /**
   Slices whose trimmed text repeats their incumbent, withdrawn instead of
   shipped as a change the document does not carry.
   */
  readonly withdrawn: readonly number[];

  /**
   One finding per trimmed block, and one per withdrawn slice.
   */
  readonly findings: readonly string[];

  /**
   Whether any replacement changed.
   */
  readonly trimmed: boolean;
};

/**
 What follows a marker that opens a definition.
 */
const DEFINITION_SEPARATOR = ':';

/**
 Convention whose definitions open a block with `[^id]:`.
 */
const GFM = 'gfm';

/**
 Characters of the marker frame around an identifier.
 */
const MARKER_FRAME_LENGTH = '[^]'.length;

/**
 Line break between lines of a text.
 */
const LINE_BREAK = '\n';

/**
 Label a line opens a definition with, empty when the line is anything else
 (prose, a heading, a definition's continuation).

 @param line - one line of a replacement

 @returns The label alone, or the empty string

 @example
 ```ts
 definitionLabelOfLine({ line: '[^2]: the note', },);
 // => '2'
 ```
 */
function definitionLabelOfLine({ line, }: { readonly line: string; },): string {
  /**
   The first marker literal, the opener when it sits at offset zero.
   */
  const [first,] = scanGfmReferenceLiterals({ slice: line, },);
  if ((first === undefined) || (first.localOffset !== 0))
    return '';
  /**
   Characters of the identifier between the brackets.
   */
  const identifierLength = first.identifier
    .length;
  /**
   Offset just past the closing bracket, since the marker opens the line.
   */
  const after = MARKER_FRAME_LENGTH + identifierLength;
  if (line.slice(
    after,
    after + DEFINITION_SEPARATOR.length,
  ) !== DEFINITION_SEPARATOR)
    return '';
  return first.identifier;
}

/**
 One block of a replacement beside the gap that preceded it and the
 definition label it opens with.

 @example
 ```ts
 const labelled: LabelledBlock = { gapBefore: '\n\n', block: '[^2]: the note', label: '2', };
 ```
 */
type LabelledBlock = {
  /**
   Text between the previous block and this one, empty for the first.
   */
  readonly gapBefore: string;

  /**
   The block's text, without its trailing line break.
   */
  readonly block: string;

  /**
   Label the block defines, empty when it defines none.
   */
  readonly label: string;
};

/**
 Blocks of a text with the text after the last block.

 @example
 ```ts
 const layout: BlockLayout = { blocks, tail: '\n', };
 ```
 */
type BlockLayout = {
  /**
   Blocks in order.
   */
  readonly blocks: readonly LabelledBlock[];

  /**
   Text after the last block, which is its trailing line breaks.
   */
  readonly tail: string;
};

/**
 Where the line walk stands: the open block's span, if any, and the end of
 the last closed one.

 @example
 ```ts
 const walk: BlockWalk = { openStart: -1, openEnd: -1, lastEnd: 0, offset: 0, };
 ```
 */
type BlockWalk = {
  /**
   Offset the open block began at, negative when none is open.
   */
  readonly openStart: number;

  /**
   Offset the open block's last line ends at.
   */
  readonly openEnd: number;

  /**
   Offset the last closed block ended at.
   */
  readonly lastEnd: number;

  /**
   Offset the next line begins at.
   */
  readonly offset: number;
};

/**
 Closes the open block into the list, if one is open.

 @param text - whole text

 @param walk - where the walk stands

 @param blocks - list the block joins

 @returns The walk with no block open

 @example
 ```ts
 const closed = closeBlock({ text, walk, blocks, },);
 ```
 */
function closeBlock(
  {
    text,
    walk,
    blocks,
  }: {
    readonly text: string;
    readonly walk: BlockWalk;
    readonly blocks: LabelledBlock[];
  },
): BlockWalk {
  if (walk.openStart < 0)
    return walk;
  /**
   The block's text.
   */
  const block = text.slice(
    walk.openStart,
    walk.openEnd,
  );
  blocks.push({
    gapBefore: text.slice(
      walk.lastEnd,
      walk.openStart,
    ),
    block,
    label: definitionLabelOfLine({ line: block, },),
  },);
  return {
    openStart: -1,
    openEnd: -1,
    lastEnd: walk.openEnd,
    offset: walk.offset,
  };
}

/**
 Blocks of a text: a block ends at a blank line or where a definition line
 begins, and every gap between blocks is kept as written.

 @param text - replacement text

 @returns Blocks in order with the trailing text

 @example
 ```ts
 labelledBlocksOf({ text: '[^1]: one\n[^2]: two\n', },);
 // => two blocks labelled 1 and 2, the second's gap '\n', the tail '\n'
 ```
 */
function labelledBlocksOf({ text, }: { readonly text: string; },): BlockLayout {
  /**
   Blocks closed so far.
   */
  const blocks: LabelledBlock[] = [];
  /**
   The walk after the last line.
   */
  const ended = text
    .split(LINE_BREAK,)
    .reduce(
    function walkLine(
      walk: BlockWalk,
      line,
    ): BlockWalk {
        /**
         Offset just past this line's text.
         */
      const lineEnd = walk.offset + line.length;
        /**
         Offset the next line begins at.
         */
      const next = lineEnd + LINE_BREAK.length;
      if (line.trim() === '') {
        return {
          ...closeBlock({
            text,
            walk,
            blocks,
          },),
          offset: next,
        };
      }
        /**
         The walk once a definition line has closed what was open.
         */
      const closed = ((walk.openStart >= 0) && (definitionLabelOfLine({ line, },) !== ''))
        ? closeBlock({
          text,
          walk,
          blocks,
        },)
        : walk;
      return {
        openStart: (closed.openStart < 0) ? walk.offset : closed.openStart,
        openEnd: lineEnd,
        lastEnd: closed.lastEnd,
        offset: next,
      };
    },
    {
      openStart: -1,
      openEnd: -1,
      lastEnd: 0,
      offset: 0,
    },
    );
  /**
   The walk with the last block closed.
   */
  const final = closeBlock({
    text,
    walk: ended,
    blocks,
  },);
  return {
    blocks,
    tail: text.slice(final.lastEnd,),
  };
}

/**
 Kept text and the gap owed before the next kept block.

 @example
 ```ts
 const state: KeptText = { text: '[^1]: one', pendingGap: '\n\n', };
 ```
 */
type KeptText = {
  /**
   Text kept so far.
   */
  readonly text: string;

  /**
   Gap that stood before the first block of the cut run just walked, empty
   when the last block walked was kept.
   */
  readonly pendingGap: string;
};

/**
 Cuts every definition block carrying one of the labels out of a text,
 keeping the gaps between the blocks that stay and the trailing text.

 A kept block that follows cut blocks takes the gap that stood before the
 first of them, so a paragraph break survives the cut of what followed it.

 @param text - replacement text

 @param labels - definition labels whose blocks go

 @returns Text without those blocks

 @example
 ```ts
 cutDefinitionBlocks({ text: '[^1]: one\n\n[^2]: two\n', labels: new Set(['2',],), },);
 // => '[^1]: one\n'
 ```
 */
export function cutDefinitionBlocks(
  {
    text,
    labels,
  }: {
    readonly text: string;
    readonly labels: ReadonlySet<string>;
  },
): string {
  /**
   The text's blocks and trailing text.
   */
  const layout = labelledBlocksOf({ text, },);
  /**
   What stays, joined.
   */
  const joined = layout.blocks
    .reduce(
    function keep(
      state: KeptText,
      entry,
    ): KeptText {
      if ((entry.label !== '') && labels.has(entry.label,)) {
        return {
          text: state.text,
          pendingGap: (state.pendingGap === '') ? entry.gapBefore : state.pendingGap,
        };
      }
        /**
         Gap owed by the cut run just walked, else this block's own.
         */
      const owed = (state.pendingGap === '') ? entry.gapBefore : state.pendingGap;
        /**
         Gap written before this block, none at the start of the text.
         */
      const gap = (state.text === '') ? '' : owed;
      return {
        text: `${state.text}${gap}${entry.block}`,
        pendingGap: '',
      };
    },
    {
      text: '',
      pendingGap: '',
    },
    );
  return `${joined.text}${layout.tail}`;
}

/**
 Whether a carried text is a decided text with definition blocks cut and
 nothing else changed, which is the one difference the assembly guard's trim
 makes between what a lane decided and what its document carries.

 @param decided - text the lane decided

 @param carried - text the document carries

 @returns Whether the carried text is the decided text under a definition trim

 @example
 ```ts
 isDefinitionTrim({ decided: 'a[^1].\n\n[^1]: one\n\n[^2]: two', carried: 'a[^1].\n\n[^1]: one', },);
 // => true
 ```
 */
export function isDefinitionTrim(
  {
    decided,
    carried,
  }: {
    readonly decided: string;
    readonly carried: string;
  },
): boolean {
  /**
   Blocks the carried text holds.
   */
  const carriedBlocks = new Set(
    labelledBlocksOf({ text: carried, },)
      .blocks
      .map(function toText(entry,): string {
        return entry.block;
      },),
  );
  if (carriedBlocks.size === 0)
    return false;
  /**
   Labels of the decided text's definition blocks the carried text lacks.
   */
  const cut = new Set(
    labelledBlocksOf({ text: decided, },)
      .blocks
      .filter(function isGone(entry,): boolean {
        if (entry.label === '')
          return false;
        return !carriedBlocks.has(entry.block,);
      },)
      .map(function toLabel(entry,): string {
        return entry.label;
      },),
  );
  if (cut.size === 0)
    return false;
  return cutDefinitionBlocks({
    text: decided,
    labels: cut,
  },) === carried;
}

/**
 Count of definition blocks across every replacement, the most trims the
 assembly guard can make.

 @param replacements - replacements the guard walks

 @returns Definition blocks in all of them

 @example
 ```ts
 definitionBlockCount({ replacements, },);
 ```
 */
export function definitionBlockCount(
  { replacements, }: { readonly replacements: readonly SliceReplacement[]; },
): number {
  return replacements.reduce(
    function add(
      count,
      replacement,
    ): number {
      /**
       Definition blocks in this replacement.
       */
      const definitions = labelledBlocksOf({ text: replacement.replacementText, },)
        .blocks
        .filter(function isDefinition(entry,): boolean {
          return entry.label !== '';
        },);
      return count + definitions.length;
    },
    0,
  );
}

/**
 Cuts the orphan definitions' blocks out of every replacement that is
 nothing but definition blocks, leaving any other replacement for the
 guard to withdraw whole.

 @param findings - footnote findings the assembly introduced

 @param replacements - replacements standing this round

 @param incumbentBySlice - archive text of every slice, by chunk index as a
 string, so a trim that lands on the incumbent is withdrawn rather than shipped

 @returns Replacements after the trim

 @example
 ```ts
 const trimmed = trimOrphanDefinitions({ findings: introduced, replacements: standing, incumbentBySlice, },);
 if (trimmed.trimmed) surviving = trimmed.replacements;
 ```
 */
export function trimOrphanDefinitions(
  {
    findings,
    replacements,
    incumbentBySlice,
  }: {
    readonly findings: readonly FootnoteGraphFinding[];
    readonly replacements: readonly SliceReplacement[];
    readonly incumbentBySlice: ReadonlyMap<string, string>;
  },
): TrimmedReplacements {
  /**
   Labels of the orphaned GFM definitions.
   */
  const orphans = new Set(
    findings
      .filter(function isOrphan(finding,): boolean {
        return (finding.kind === 'orphan-definition') && (finding.convention === GFM);
      },)
      .map(function toIdentifier(finding,): string {
        return finding.identifier;
      },),
  );
  if (orphans.size === 0)
    return {
      replacements,
      withdrawn: [],
      findings: [],
      trimmed: false,
    };
  /**
   Findings for each block cut.
   */
  const cuts: string[] = [];
  /**
   Replacements with the orphan blocks cut.
   */
  const trimmed = replacements.map(function trim(replacement,): SliceReplacement {
    /**
     The replacement's blocks, each beside its label.
     */
    const { blocks, } = labelledBlocksOf({ text: replacement.replacementText, },);
    if (blocks.some(function isProse(entry,): boolean {
      return entry.label === '';
    },))
      return replacement;
    /**
     Blocks whose label is not an orphan.
     */
    const kept = blocks.filter(function keeps(entry,): boolean {
      return !orphans.has(entry.label,);
    },);
    if ((kept.length === blocks.length) || (kept.length === 0))
      return replacement;
    for (const entry of blocks) {
      if (!kept.includes(entry,))
        cuts.push(
          `assembly-footnote-trimmed orphan-definition ${GFM} ${entry.label} (slice ${
            String(replacement.sliceIndex,)
          })`,
        );
    }
    return {
      sliceIndex: replacement.sliceIndex,
      replacementText: cutDefinitionBlocks({
        text: replacement.replacementText,
        labels: orphans,
      },),
    };
  },);
  /**
   Trimmed replacements that now repeat their incumbent.
   */
  const emptied = trimmed.filter(function repeatsIncumbent(replacement,): boolean {
    return replacement.replacementText === incumbentBySlice.get(String(replacement.sliceIndex,),);
  },);
  for (const replacement of emptied) {
    cuts.push(
      `assembly-footnote-trimmed-to-incumbent (slice ${String(replacement.sliceIndex,)})`,
    );
  }
  return {
    replacements: trimmed.filter(function stands(replacement,): boolean {
      return !emptied.includes(replacement,);
    },),
    withdrawn: emptied.map(function toIndex(replacement,): number {
      return replacement.sliceIndex;
    },),
    findings: cuts,
    trimmed: cuts.length > 0,
  };
}

//endregion Orphan definition trim
