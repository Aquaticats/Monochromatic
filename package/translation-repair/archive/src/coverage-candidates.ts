import {
  alignBlocks,
  alignHeadingsForced,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type {
  ContentChunk,
  RepairDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  alignDocumentSections,
  chunkByHeadings,
  chunkLabel,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';

//region Coverage candidates
// Every passage an aligner reports as unpaired, which is every passage anything
// might insert.
//
// ONE LIST FROM BOTH SCALES, because they are the same question. A source
// section with no target section and a source block with no target block are
// both "the aligner could not pair this", and it was measured that neither is
// evidence the translation lacks the passage. What decides that is the coverage
// stage, and this is what it has to ask about.
//
// THE SECTION SCALE IS RECOMPUTED HERE rather than read off `alignDocumentSections`,
// which reports its refusals only as prose findings: an `attachedTo` place plus a
// detail string. Recomputing means mirroring its branching, including the equal-shape
// short circuit that 85 of 92 corpus entries take, and the mirror has to be kept
// in step by hand until refusals are reported structurally.

/**
 One passage no pairing covers.

 @example
 ```ts
 const candidate: CoverageCandidate = { scale: 'section', sourceIndex: 12, sourceText, };
 ```
 */
export type CoverageCandidate = {
  /**
   A block inside a section both sides carry.
   */
  readonly scale: 'block';

  /**
   Which section pair it sits in.
   */
  readonly pairIndex: number;

  /**
   Which block of that section's original side.
   */
  readonly sourceIndex: number;

  /**
   Text of the passage, which is what the coverage question is about.
   */
  readonly sourceText: string;
} | {
  /**
   A whole section the matcher paired with nothing.
   */
  readonly scale: 'section';

  /**
   Which section of the original.
   */
  readonly sourceIndex: number;

  /**
   Text of the whole section, heading included.
   */
  readonly sourceText: string;
};

/**
 Whether two sides pair by index without consulting the matcher.

 MIRRORS `alignDocumentSections`, whose fast path this has to reproduce to
 report the same refusals it would.

 @param sourceChunks - original-side sections

 @param targetChunks - translation-side sections

 @returns Whether the counts and leading kinds agree

 @example
 ```ts
 const fast = pairsByIndex({ sourceChunks, targetChunks, },);
 ```
 */
function pairsByIndex(
  {
    sourceChunks,
    targetChunks,
  }: {
    readonly sourceChunks: readonly ContentChunk[];
    readonly targetChunks: readonly ContentChunk[];
  },
): boolean {
  return (sourceChunks.length === targetChunks.length)
    && sourceChunks.every(function leadingKindMatches(
      chunk,
      index,
    ): boolean {
      return chunk.nodes[0]
        ?.kind
        === targetChunks[index]
          ?.nodes[0]
          ?.kind;
    },);
}

/**
 Lists the sections the matcher paired with nothing.

 @param source - original document

 @param target - translation

 @returns One candidate per unpaired source section

 @example
 ```ts
 const sections = unpairedSections({ source, target, },);
 ```
 */
function unpairedSections(
  {
    source,
    target,
  }: {
    readonly source: RepairDocument;
    readonly target: RepairDocument;
  },
): readonly CoverageCandidate[] {
  /**
   Sections of the original.
   */
  const sourceChunks = chunkByHeadings({ document: source, },);

  /**
   Sections of the translation.
   */
  const targetChunks = chunkByHeadings({ document: target, },);
  if ((sourceChunks.length === 0) || (targetChunks.length === 0))
    return [];
  if (pairsByIndex({
    sourceChunks,
    targetChunks,
  },))
    return [];

  return alignHeadingsForced({
    sourceHeadings: sourceChunks.map(function toLabel(chunk,): string {
      return chunkLabel(chunk,);
    },),
    targetHeadings: targetChunks.map(function toLabel(chunk,): string {
      return chunkLabel(chunk,);
    },),
  },)
    .flatMap(function toCandidate(step,): readonly CoverageCandidate[] {
      if (step.kind !== 'source-only')
        return [];

      /**
       Section the matcher refused, present by the aligner's own contract.
       */
      const chunk = sourceChunks[step.sourceIndex];
      if (chunk === undefined)
        throw new Error('unreachable: the matcher named a section outside its own input',);

      return [{
        scale: 'section',
        sourceIndex: step.sourceIndex,
        sourceText: chunk.text,
      },];
    },);
}

/**
 Lists the blocks the aligner paired with nothing, inside sections that paired.

 @param source - original document

 @param target - translation

 @returns One candidate per unpaired source block

 @example
 ```ts
 const blocks = unpairedBlocks({ source, target, },);
 ```
 */
function unpairedBlocks(
  {
    source,
    target,
  }: {
    readonly source: RepairDocument;
    readonly target: RepairDocument;
  },
): readonly CoverageCandidate[] {
  return alignDocumentSections({
    source,
    target,
  },)
    .pairs
    .flatMap(function toCandidates(
      pair,
      pairIndex,
    ): readonly CoverageCandidate[] {
      /**
       Blocks of this section's original side.
       */
      const sourceNodes = pair.source
        .nodes;

      /**
       Blocks of its translation side.
       */
      const targetNodes = pair.target
        .nodes;
      if ((sourceNodes.length === 0) || (targetNodes.length === 0))
        return [];

      return alignBlocks({
        sourceNodes,
        targetNodes,
      },)
        .flatMap(function toCandidate(step,): readonly CoverageCandidate[] {
          if (step.kind !== 'source-only')
            return [];

          /**
           Block the aligner refused, present by its own contract.
           */
          const node = sourceNodes[step.sourceIndex];
          if (node === undefined)
            throw new Error('unreachable: the aligner named a block outside its own input',);

          return [{
            scale: 'block',
            pairIndex,
            sourceIndex: step.sourceIndex,
            sourceText: node.text,
          },];
        },);
    },);
}

/**
 @internal

 Lists every passage an aligner reports as unpaired, at both scales.

 @param source - original document

 @param target - translation

 @returns Section candidates first, then block candidates

 @example
 ```ts
 const candidates = listCoverageCandidates({ source, target, },);
 ```
 */
export function listCoverageCandidates(
  {
    source,
    target,
  }: {
    readonly source: RepairDocument;
    readonly target: RepairDocument;
  },
): readonly CoverageCandidate[] {
  return [
    ...unpairedSections({
      source,
      target,
    },),
    ...unpairedBlocks({
      source,
      target,
    },),
  ];
}

//endregion Coverage candidates
