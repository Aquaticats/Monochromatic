import {
  type ArchiveDispute,
  disputedWordingsOf,
} from '@monochromatic-dev/translation-repair-archive/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { DocumentNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { DisputedWording, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type {
  ArtifactConsolidateSlice,
  ProjectedLanes,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type { ArtifactContestSlice, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  type ConsistencyEnglish,
  ConsistencyInvariantError,
  type PageEnglish,
  type PageLayout,
  pageOfSlices,
  type PageSlice,
  quoteReferencesIn,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { shippedWriterOf, } from './pass-consistency-shipped.ts';
import { wouldShipTextFor, } from '@monochromatic-dev/translation-repair-page-check/ts';

//region Pass consistency slices
// What the consistency pass reads of each slice: the original, the wording the
// slice would ship at this point, and the facts the floors need. Gathered once,
// so the pass itself works on data and never on the preparation. EVERY SLICE ALSO
// CARRIES THE PAGE THAT SHIPS (`page`), because the proof that a mark is an
// apostrophe looks for a quotation mark that could pair with it anywhere on the page.
// THE PAGE THAT SHIPS is the page the publishing writes for the wording the slices would ship before the pass,
// by the publishing's own code, not a model of it: the wording of each slice goes through typography (a wording
// the archive did not decide has its quotation marks curled), the page-assembly passes and their guard rewrite
// the composed page after it (`pass-consistency-shipped.ts`), and `placeSlices` (`splice-slices.ts`) writes the
// result over the span of each slice of the target text, so the text no slice holds (a sealed span the
// translators' note calls the English original, the blank lines between blocks, a block no slice claims) stands in
// it as the archive ships it, the line endings a span keeps at its edges are the span's, the separators around an
// inserted anchor are the publishing's, and a slice that ships no English contributes nothing. A slice's `current`
// is the wording before typography and the passes, which the pass addresses its replacements to and records; the
// page holds what that wording ships as where it differs (`page.shipped`). EVERY SLICE ALSO CARRIES WHAT THE PAGE
// IS WRITTEN FROM (`layout`), whose writer gives the page the publishing would write once the pass's replacements
// are in it, so the proof of an apostrophe on a page the pass would make reads the glyphs that ship.

/**
 What the pass knows of one slice.

 @example
 ```ts
 const facts: ConsistencySliceFacts = { sliceIndex: 0, sourceIndex: 0, sourceText: '小猫打盹。', sourceStart: 0, nodes: [], current: { kind: 'absent', }, lineStructured: false, disputedWordings: [], };
 ```
 */
export type ConsistencySliceFacts = {
  /**
   Index the comparison row and the artifact name the slice by.
   */
  readonly sliceIndex: number;

  /**
   Index the original's chunk carries, which the repeat groups name.
   */
  readonly sourceIndex: number;

  /**
   The slice's original.
   */
  readonly sourceText: string;

  /**
   Absolute offset in the whole original where the slice starts.
   */
  readonly sourceStart: number;

  /**
   Blocks of the slice's original, whose offsets are absolute.
   */
  readonly nodes: readonly DocumentNode[];

  /**
   The slice's English as it would ship now.
   */
  readonly current: ConsistencyEnglish;

  /**
   The English of the whole page this slice belongs to as it ships (every slice that carries English, as the slices would
   ship now, through typography and the page-assembly passes) and where the slice's English starts in it, which the proof of
   an apostrophe reads; `shipped` holds what the page carries for this slice where that is not `current`. Absent for a slice
   that carries no English, and for facts built without a page, where the slice's English alone is the page.
   */
  readonly page?: PageEnglish;

  /**
   What the published page is assembled from, shared by every slice that carries English: the proof of an apostrophe on a
   page the pass would make writes the wording of each slice into it with the code that publishes the page. Absent where
   `page` is.
   */
  readonly layout?: PageLayout;

  /**
   The character references that decode to a quotation-like character (`quoteReferencesIn`) in the English of every slice of the
   page, each once, in the order of the slices where each first stands, the same list beside every slice. It is read from the
   wording of each slice on its own, so a slice the page writes nothing for (which has no `page`, and is read alone) is in it
   too, and a reference there refuses every splice on the page as one in the page's text does. Absent for facts built by hand,
   whose references are those of `page` or of the slice's English.
   */
  readonly wholePageReferences?: readonly string[];

  /**
   Whether the line-structure rule governs the slice.
   */
  readonly lineStructured: boolean;

  /**
   Wordings the slice refuses because its archive rendering was disputed.
   */
  readonly disputedWordings: readonly DisputedWording[];
};

/**
 What the pass reads of one slice before the page they make is joined: its facts, and what the publishing writes for it when
 it ships no English.
 */
type SliceRead = {
  /**
   What the pass knows of the slice.
   */
  readonly facts: ConsistencySliceFacts;

  /**
   The replacement the publishing writes for a slice that ships no English: none at an anchor, an empty text over a span;
   none for a slice that ships English.
   */
  readonly silent: readonly SliceReplacement[];
};

/**
 Gathers what the pass reads of every slice.

 @param prepared - the preparation both lanes ran over

 @param projected - both lanes' rows and the comparison

 @param contestSlices - what the contest decided

 @param consolidateSlices - what the third rendering settled

 @param archiveDisputes - slices whose archive rendering was disputed

 @returns One entry per prepared slice, in order

 @throws {@link UnansweredContestSliceError} when the lanes differ at a slice
 the contest never answered, which the settled page would refuse as well

 @throws {@link Error} when a slice has no comparison row

 @example
 ```ts
 const slices = consistencySlicesOf({ prepared, projected, contestSlices, consolidateSlices, archiveDisputes, },);
 ```
 */
export function consistencySlicesOf(
  {
    prepared,
    projected,
    contestSlices,
    consolidateSlices,
    archiveDisputes,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly projected: ProjectedLanes;
    readonly contestSlices: readonly ArtifactContestSlice[];
    readonly consolidateSlices: readonly ArtifactConsolidateSlice[];
    readonly archiveDisputes: ReadonlyMap<number, ArchiveDispute>;
  },
): readonly ConsistencySliceFacts[] {
  /**
   What every slice would ship now, before the page they make is joined.
   */
  const read = prepared.slices
    .map(function factsOf(slice,): SliceRead {
      /**
       Comparison row of the slice.
       */
      const row = projected.comparison
        .find(function namesSlice(candidate,): boolean {
          return candidate.sliceIndex
            === slice.target
              .sliceIndex;
        },);
      if (row === undefined) {
        throw new ConsistencyInvariantError({ invariant:
          `slice ${String(slice.target
            .sliceIndex,)} of the preparation has no comparison row, `
            + 'and the comparison is derived from rows of that preparation', },);
      }
      /**
       What the slice would ship now.
       */
      const reading = wouldShipTextFor({
        artifact: {
          comparison: projected.comparison,
          laneSelection: {
            kind: 'contested',
            slices: contestSlices,
          },
          consolidation: {
            kind: 'settled',
            slices: consolidateSlices,
          },
        },
        row,
      },);
      /**
       Dispute over the slice's archive rendering, absent for most.
       */
      const dispute = archiveDisputes.get(slice.target
        .sliceIndex,);
      return {
        facts: {
          sliceIndex: slice.target
            .sliceIndex,
          sourceIndex: slice.source
            .sliceIndex,
          sourceText: slice.source
            .text,
          sourceStart: slice.source
            .startOffset,
          nodes: slice.source
            .nodes,
          current: (reading.kind === 'wording')
            ? {
              kind: 'present',
              text: reading.text,
            }
            : { kind: 'absent', },
          lineStructured: prepared.lineStructuredSliceIndices
            .has(slice.target
              .sliceIndex,),
          disputedWordings: (dispute === undefined)
            ? []
            : disputedWordingsOf({
              dispute,
              archiveText: row.incumbentText,
            },),
        },
        // What the publishing writes for a slice that ships no English (`shippableReplacements`): nothing at an anchor,
        // an empty text over a span.
        silent: ((reading.kind === 'wording') || (reading.incumbentKind === 'absent'))
          ? []
          : [
            {
              sliceIndex: slice.target
                .sliceIndex,
              replacementText: '',
            },
          ],
      };
    },);
  /**
   What every slice would ship now, before the page they make is joined.
   */
  const alone = read.map(function factsOnly({ facts, },): ConsistencySliceFacts {
    return facts;
  },);
  /**
   The wording each slice that carries English would ship before the pass, by the slice's index, mutated only inside this function.
   */
  const raw = new Map<number, string>();
  for (const {
    sliceIndex,
    current,
  } of alone) {
    if (current.kind === 'present') {
      raw.set(
        sliceIndex,
        current.text,
      );
    }
  }
  /**
   What the published page is assembled from: the target text, the slice pairs, what the slices that ship no English write,
   and the writer of what every slice ships for the English a reader gives, which is the publishing's own code.
   */
  const layout: PageLayout = {
    targetText: prepared.targetText,
    chunks: prepared.slices,
    silent: read.flatMap(function silentIn({ silent, },): readonly SliceReplacement[] {
      return silent;
    },),
    shipped: shippedWriterOf({
      prepared,
      comparison: projected.comparison,
      contestSlices,
      consolidateSlices,
      raw,
    },),
  };
  /**
   The published page with the wording the slices would ship before the pass written in, as the publishing ships it: through
   typography and the page-assembly passes, a sealed span and the separators of an anchor among it.
   */
  const page = pageOfSlices({
    slices: alone.flatMap(function presentIn({
      sliceIndex,
      current,
    },): readonly PageSlice[] {
      return (current.kind === 'present')
        ? [
          {
            sliceIndex,
            english: current.text,
          },
        ]
        : [];
    },),
    layout,
  },);
  /**
   The quotation-like character references in the English of every slice of the page, each once, in the order of the slices: the
   English of each slice on its own, so a slice read alone is in it too.
   */
  const wholePageReferences: readonly string[] = [
    ...new Set(alone.flatMap(function referencesIn({ current, },): readonly string[] {
      return (current.kind === 'present') ? quoteReferencesIn({ english: current.text, },) : [];
    },),),
  ];
  return alone.map(function withPage(entry,): ConsistencySliceFacts {
    /**
     What the pass knows of the slice, with the references of the whole page beside it.
     */
    const facts: ConsistencySliceFacts = {
      ...entry,
      wholePageReferences,
    };
    /**
     Where this slice's English starts in the page, none for a slice that carries none.
     */
    const offset = page.offsets
      .get(facts.sliceIndex,);
    /**
     What the page holds for the slice, which is the wording the slice carries now unless typography or a page-assembly pass
     rewrote it.
     */
    const shipped = page.shipped
      .get(facts.sliceIndex,);
    /**
     The wording the slice carries now.
     */
    const { current, } = facts;
    if (offset === undefined) {
      // A slice that carries English the page writes nothing for (the guard took its row back and the archive holds none
      // there) still belongs to the page: it keeps the layout, so the proof of the others is read on the page that ships,
      // and no page of its own, so it is read alone.
      return (current.kind === 'present')
        ? {
          ...facts,
          layout,
        }
        : facts;
    }
    if ((shipped === undefined) || (current.kind === 'absent')) {
      return {
        ...facts,
        layout,
        page: {
          text: page.text,
          offset,
        },
      };
    }
    return {
      ...facts,
      layout,
      page: {
        text: page.text,
        offset,
        ...(shipped === current.text) ? {} : { shipped, },
      },
    };
  },);
}

//endregion Pass consistency slices
