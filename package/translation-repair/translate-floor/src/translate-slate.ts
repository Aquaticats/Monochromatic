import type {
  Candidate,
  CandidateProducer,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type {
  TranslateCandidateValue,
  TranslateOrigin,
} from './translate-candidates.ts';

//region Translate slate
// The candidate list AS THE JUDGES SAW IT, kept so a ballot can be read later.
//
// Ballots name a one-based position, the slate is rotated by a hash of the
// slice before judges see it, and nothing recorded the rotated order. A stored
// ballot saying "candidate 2" therefore could not be joined to any text or to
// any model: the number was meaningless the moment the round ended.
//
// This is the join key. Producer names are attached HERE, after the round,
// never in the judge messages: the judges still see anonymous positions.

/**
 Position meaning "this text was not on the ballot at all".

 Zero rather than a nullish value, matching `CANDIDATE_NONE` on the ballot
 side, so every recorded position is one number a reader compares the same
 way.
 */
export const NOT_ON_SLATE = 0;

/**
 One position on the ballot, with everything needed to read a vote for it.

 @example
 ```ts
 const entry: TranslateSlateEntry = { index: 1, text, hash, origin: 'incumbent', producer, };
 ```
 */
export type TranslateSlateEntry = {
  /**
   One-based position, exactly as the judges were shown it.
   */
  readonly index: number;

  /**
   Candidate text at that position.
   */
  readonly text: string;

  /**
   Digest of that text, so an artifact can be checked against a rebuilt slice
   without carrying every candidate twice.
   */
  readonly hash: string;

  /**
   Whether this position was the translation already in the archive.
   */
  readonly origin: TranslateOrigin;

  /**
   Who produced it, including every model that reproduced it exactly.
   */
  readonly producer: CandidateProducer;
};

/**
 Records the rotated slate.

 @param candidates - candidates in the order the judges were shown them

 @returns One entry per position

 @example
 ```ts
 const slate = describeSlate({ candidates: rotated, },);
 ```
 */
export function describeSlate(
  {
    candidates,
  }: {
    readonly candidates: readonly Candidate<TranslateCandidateValue>[];
  },
): readonly TranslateSlateEntry[] {
  return candidates.map(function toEntry(
    candidate,
    position,
  ): TranslateSlateEntry {
    return {
      index: position + 1,
      text: candidate.value
        .text,
      hash: hashContent({ content: candidate.value
        .text, },),
      origin: candidate.value
        .origin,
      producer: candidate.producer,
    };
  },);
}

/**
 Position the shipped text occupies on the slate.

 @param slate - rotated slate

 @param text - text that shipped

 @returns One-based position, or {@link NOT_ON_SLATE} when the shipped text
 was never a candidate, which is what a blank incumbent looks like

 @example
 ```ts
 const shippedIndex = positionOf({ slate, text, },);
 ```
 */
export function positionOf(
  {
    slate,
    text,
  }: {
    readonly slate: readonly TranslateSlateEntry[];
    readonly text: string;
  },
): number {
  return slate.find(function carriesText(entry,): boolean {
    return entry.text === text;
  },)
    ?.index
    ?? NOT_ON_SLATE;
}

/**
 Hex digits of the slice hash that fix candidate order.
 */
const ROTATION_HEX_DIGITS = 8;

/**
 Radix of that hash prefix.
 */
const HEX_RADIX = 16;

/**
 Rotates the candidate slate by a hash of the slice, so the incumbent does not
 sit in the same ballot position on every slice.

 Judges receive one caller-fixed order, and the incumbent win rate is the
 measurement this whole lane exists to produce. Pinning the incumbent to
 position one would confound that rate with whatever position preference the
 judges have, and the confound would be invisible: every slice would carry it
 equally.

 Rotation rather than shuffling, and keyed on the SOURCE rather than on a
 random draw, because a slice's candidate order has to be identical between a
 fresh run and a resumed one. A cached slice replayed under a different order
 would be a different question asked of the judges.

 @param candidates - slate in assembly order

 @param sourceText - slice original, the rotation key

 @returns Same candidates, rotated

 @example
 ```ts
 const ordered = rotateCandidates({ candidates, sourceText, },);
 ```
 */
export function rotateCandidates<ValueT,>(
  {
    candidates,
    sourceText,
  }: {
    readonly candidates: readonly ValueT[];
    readonly sourceText: string;
  },
): readonly ValueT[] {
  if (candidates.length === 0)
    return candidates;

  /**
   Positions to rotate left by, derived from the slice itself.
   */
  const offset = Number.parseInt(
    hashContent({ content: sourceText, },)
      .slice(
      0,
      ROTATION_HEX_DIGITS,
      ),
    HEX_RADIX,
  ) % candidates.length;

  return [
    ...candidates.slice(offset,),
    ...candidates.slice(
      0,
      offset,
    ),
  ];
}

//endregion Translate slate
