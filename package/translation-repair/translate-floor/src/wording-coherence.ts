import type { LaneSliceText, } from './lane-slice-text.ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Wording coherence
// The rule tying a lane's OUTCOME to the archive's own state at the same slice.
//
// `LaneSliceText` carries two independent axes on purpose, and independent is
// not unconstrained: three of their combinations describe a slice that cannot
// exist. `buildLaneSliceTexts` refuses all three while it is building, but it
// is not the only way a wording reaches a consumer. The type is exported, the
// delivery ledger and the lane comparison both take wordings from a caller, and
// an artifact reader will take them from disk. Each of those is a boundary
// where the producer's checks are somebody else's assumption.
//
// So the rule lives here rather than inside the builder, and every boundary
// asserts it for itself. The cost is one pass over the slices; what it buys is
// that no reader has to trust the writer.

/**
 Which contradiction between a slice's outcome and the archive's state a
 wording carries.

 @example
 ```ts
 const fault: WordingFault = 'fallback-without-archive';
 ```
 */
export type WordingFault =
  | 'fallback-without-archive'
    | 'missing-where-archive-holds'
    | 'nothing-to-do-where-archive-holds'
    | 'empty-decision-without-archive'
    | 'wording-beside-absent-archive';

/**
 What each fault says, after the slice it names.
 */
const WORDING_FAULT_SENTENCES: Readonly<Record<WordingFault, string>> = {
  'fallback-without-archive': 'reports the archive\'s wording standing by default, and the archive holds none',
  'missing-where-archive-holds': 'reports a missing passage, and the archive holds wording for it',
  'nothing-to-do-where-archive-holds': 'reports this lane having nothing to work on, and the archive holds wording '
    + 'there',
  'empty-decision-without-archive': 'reports a decision of empty wording where the archive holds none, so nothing '
    + 'was filled and nothing distinguishes it from a passage still missing',
  'wording-beside-absent-archive': 'says the archive holds no wording and carries some anyway, so the two sides of '
    + 'this row were built from different preparations',
};

/**
 Raised when one slice's outcome contradicts what the archive holds there.

 SEPARATE FROM COVERAGE. A coverage failure means the lane and the preparation
 disagree about which slices exist; this means one wording disagrees with
 itself, which no join or count could detect afterwards because every field is
 individually well formed.

 THE CLASS WRITES ITS SENTENCE, from a slice index and a closed fault kind,
 so its message names a slice and a fixed phrase and nothing a caller wrote;
 the artifact reader forwards it into its own refusal on that ground (ledger
 B34).

 @example
 ```ts
 throw new WordingCoherenceError({ sliceIndex: 4, fault: 'fallback-without-archive', },);
 ```
 */
export class WordingCoherenceError extends Error {
  /**
   Declares this message safe to forward: a slice index and a fixed phrase.
   */
  readonly messageNamesOnly: true = true;

  /**
   Slice whose wording contradicts itself.
   */
  readonly sliceIndex: number;

  /**
   Which contradiction it carries.
   */
  readonly fault: WordingFault;

  /**
   Builds the error with a message naming the slice and the contradiction.

   @param sliceIndex - slice whose wording contradicts itself

   @param fault - which contradiction it carries

   @example
   ```ts
   throw new WordingCoherenceError({ sliceIndex: 4, fault: 'fallback-without-archive', },);
   ```
   */
  constructor(
    {
      sliceIndex,
      fault,
    }: {
      readonly sliceIndex: number;
      readonly fault: WordingFault;
    },
  ) {
    super(`slice ${String(sliceIndex,)} ${WORDING_FAULT_SENTENCES[fault]}`,);
    this.name = 'WordingCoherenceError';
    this.sliceIndex = sliceIndex;
    this.fault = fault;
  }
}

/**
 Refuses a wording whose outcome and incumbent describe different slices.

 @param wording - one lane's record of one slice, naming both what the lane
 did and whether the archive holds anything there

 @throws {@link WordingCoherenceError} when the lane falls back on an
 incumbent that does not exist, reports a passage the archive translates as
 unfilled, or claims a decision that filled a gap with nothing a reader sees

 @example
 ```ts
 assertWordingCoherent({ wording, },);
 ```
 */
export function assertWordingCoherent(
  { wording, }: { readonly wording: LaneSliceText; },
): void {
  /**
   What the lane did here, whether the archive holds anything here, which are
   the two axes this rule relates, and the slice every refusal names so a
   failure points at a row.
   */
  const {
    outcome,
    incumbentKind,
    sliceIndex,
  } = wording;

  // NOTHING TO FALL BACK ON. `incumbent-fallback` says the archive's own
  // wording stands here because nobody produced one; at a place the archive
  // never translated, that reports a passage as covered by wording that does
  // not exist.
  if ((outcome.kind === 'incumbent-fallback') && (incumbentKind === 'absent')) {
    throw new WordingCoherenceError({
      sliceIndex,
      fault: 'fallback-without-archive',
    },);
  }

  // THE MIRROR. `unfilled` says the passage is missing and always was; at a
  // slice the archive does translate, it reports a passage the archive covers
  // as one it never did, and every count of missing passages inherits that.
  if ((outcome.kind === 'unfilled') && (incumbentKind === 'present')) {
    throw new WordingCoherenceError({
      sliceIndex,
      fault: 'missing-where-archive-holds',
    },);
  }

  // THE OTHER MIRROR. `not-applicable` says this lane had no input here, and
  // the only slice a lane can have no input at is one the archive never
  // translated: at a slice it did translate, the lane's work applies by
  // definition and declining it is a choice rather than a structural fact.
  if ((outcome.kind === 'not-applicable') && (incumbentKind === 'present')) {
    throw new WordingCoherenceError({
      sliceIndex,
      fault: 'nothing-to-do-where-archive-holds',
    },);
  }

  // A DECISION THAT DECIDED NOTHING, at a place with nothing already. The
  // wording is empty and so is the archive, so the document carries the gap it
  // had; calling that a decision credits the lane with filling a passage it
  // left exactly as it found it.
  //
  // EMPTY IS WHAT SHOWS A READER NOTHING, asked as the translate lane asks it
  // where it builds the record (`assertAbsentSliceFilled`, ledger B40), so
  // the two never disagree: asked here of an empty-string comparison, a
  // decision of one zero-width space or Hangul filler read from a caller or
  // from disk passed as a passage filled.
  if ((outcome.kind === 'decided')
    && (incumbentKind === 'absent')
    && rendersAsNothing({ text: outcome.acceptedText, },)) {
    throw new WordingCoherenceError({
      sliceIndex,
      fault: 'empty-decision-without-archive',
    },);
  }

  // NOT AN OUTCOME RULE AT ALL, and here because this is where the two axes
  // meet. An anchor names a boundary and carries no text by construction, so
  // wording recorded beside `absent` came from somewhere else: a preparation
  // that has since changed, or a row assembled from two.
  if ((incumbentKind === 'absent') && (wording.incumbentText !== '')) {
    throw new WordingCoherenceError({
      sliceIndex,
      fault: 'wording-beside-absent-archive',
    },);
  }
}

//endregion Wording coherence
