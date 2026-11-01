import {
  classifyDisplacement,
  type DocumentDisplacement,
  type RelocationCandidate,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  type PreparedDocumentPair,
  sliceSizesOf,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { isMarkupOnly, } from '@monochromatic-dev/translation-repair-insertion/ts';
import { sharesMedia, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Displacement probe row
// One entry's reading of the displacement probe, flattened for printing: what
// the size screen made of the slicing the lanes saw.

/**
 One entry's reading, flattened for printing.

 @example
 ```ts
 const row: EntryDisplacement = {
   entryId: 'whiskers',
   sliceCount: 2,
   baseline: 2.5,
   baselineFrom: 'document',
   untranslated: [],
   targetOnly: [],
   relocationCandidates: [],
   transcriptionSuspects: [],
   markupDonors: [],
   otherImbalances: [],
 };
 ```
 */
export type EntryDisplacement = {
  /**
   Corpus entry this describes.
   */
  readonly entryId: string;

  /**
   Slices this document offered.
   */
  readonly sliceCount: number;

  /**
   Expansion the residuals were read against.
   */
  readonly baseline: number;

  /**
   Whether that expansion came from this document or from the corpus.
   */
  readonly baselineFrom: DocumentDisplacement['baselineFrom'];

  /**
   Slices whose original was left essentially unrendered.
   */
  readonly untranslated: readonly number[];

  /**
   Slices carrying translation the original does not account for.
   */
  readonly targetOnly: readonly number[];

  /**
   High slices whose neighbour gave up enough to account for them.
   */
  readonly relocationCandidates: readonly RelocationCandidate[];

  /**
   Relocation candidates whose high slice embeds the same media on both sides,
   so a transcription explains the surplus at least as well as a move does.
   */
  readonly transcriptionSuspects: readonly number[];

  /**
   Low slices whose ORIGINAL is markup rather than prose, so they sit below
   baseline for a reason unrelated to giving a passage up and cannot be a
   relocation donor. This class was named by hand; it is reported rather
   than suppressed so a reader knows what to subtract.
   */
  readonly markupDonors: readonly number[];

  /**
   High slices with no neighbour that gave anything up.
   */
  readonly otherImbalances: readonly number[];
};

/**
 Reads one slice a candidate names, which the classification drew from this
 same slicing.

 @param prepared - slicing the candidate's indices count over

 @param index - slice position the candidate names

 @param role - which end of the candidate the index is, for the message

 @returns The slice pair at that position

 @throws {@link Error} when the position is no slice of the slicing, which the
 classification, drawn over these very slices, never names

 @example
 ```ts
 const slice = sliceNamed({ prepared, index: candidate.high, role: 'high', },);
 ```
 */
function sliceNamed(
  {
    prepared,
    index,
    role,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly index: number;
    readonly role: 'high' | 'low';
  },
): PreparedDocumentPair['slices'][number] {
  /**
   Slice at that position.
   */
  const { slices, } = prepared;

  /**
   Slice at that position.
   */
  const slice = slices[index];
  if (slice === undefined)
    throw new Error(
      `unreachable: a relocation candidate names slice ${String(index,)} as its ${role} end, but the slicing it `
        + `was classified over has a slice count of ${String(slices.length,)}`,
    );
  return slice;
}

/**
 Reads one entry's slice sizes.

 @param entryId - corpus entry to read

 @param prepared - slicing the lanes saw, carved through the entry's settled
 recipe

 @returns What the screen made of it

 @throws {@link Error} when a relocation candidate names a slice the slicing
 does not hold

 @example
 ```ts
 const reading = readEntry({ entryId, prepared, },);
 ```
 */
export function readEntry(
  {
    entryId,
    prepared,
  }: {
    readonly entryId: string;
    readonly prepared: PreparedDocumentPair;
  },
): EntryDisplacement {
  /**
   Sizes of both sides per slice, classified.
   */
  const reading = classifyDisplacement({
    slices: sliceSizesOf({ slices: prepared.slices, },),
  },);
  return {
    entryId,
    sliceCount: reading.slices
      .length,
    baseline: reading.baseline,
    baselineFrom: reading.baselineFrom,
    untranslated: reading.untranslated,
    targetOnly: reading.targetOnly,
    relocationCandidates: reading.relocationCandidates,
    transcriptionSuspects: reading.relocationCandidates
      .filter(function embedsMedia(candidate,): boolean {
        /**
         Slice pair the surplus sits in.
         */
        const slice = sliceNamed({
          prepared,
          index: candidate.high,
          role: 'high',
        },);
        return sharesMedia({
          sourceText: slice.source
            .text,
          targetText: slice.target
            .text,
        },);
      },)
      .map(function toIndex(candidate,): number {
        return candidate.high;
      },),
    markupDonors: reading.relocationCandidates
      .filter(function donorIsMarkup(candidate,): boolean {
        /**
         Slice the passage would have had to come FROM, which is the low side.

         THE LOW SIDE, not the high one, and that is the whole point. A
         transcription suspect is recognised by what the HIGH slice embeds; a
         markup donor is recognised by what the LOW slice never had.
         */
        const slice = sliceNamed({
          prepared,
          index: candidate.low,
          role: 'low',
        },);
        return isMarkupOnly({
          sourceText: slice.source
            .text,
        },);
      },)
      .map(function toIndex(candidate,): number {
        return candidate.low;
      },),
    otherImbalances: reading.otherImbalances,
  };
}

//endregion Displacement probe row
