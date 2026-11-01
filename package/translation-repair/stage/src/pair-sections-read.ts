import {
  type SectionPair,
  SectionPairingError,
  type SectionPairingWire,
} from './pair-sections-wire.ts';
import { isIndexPairingWire, } from './index-pair-list.ts';

//region Section pairing reader
// REFUSES RATHER THAN REPAIRS, for the reason the block reader gives: a pairing
// that runs backwards, names a section that does not exist, or claims one
// section twice is not a near-miss to be tidied up but evidence the model did
// not do the task, and using part of it would put mismatched passages in front
// of every later stage exactly as the aligner used to.
//
// STRICTER THAN THE BLOCK READER ON ONE POINT, and only that one. The block
// reader permits a repeat on either side, because a translation splitting or
// merging paragraphs is a correspondence the slice machinery can carry. A
// `ChunkPair` carries ONE section on each side, so a repeat here has nowhere to
// go: it would silently drop whichever section lost the race. Strictly
// increasing on both sides is therefore not conservatism, it is the shape the
// downstream type can hold.

/**
 Whether a parsed value has the shape of a section pairing.

 SHAPE ONLY. Whether the pairing is usable is {@link readSectionPairing}'s
 question, because that needs the section counts.

 @param value - parsed model reply

 @returns Whether it is a {@link SectionPairingWire}

 @example
 ```ts
 const ok = isSectionPairingWire({ pairs: [], },);
 ```
 */
export function isSectionPairingWire(value: unknown,): value is SectionPairingWire {
  return isIndexPairingWire(value,);
}

/**
 Refuses any pair naming a section neither document has.

 @param pairs - correspondences as the model gave them

 @param sourceCount - original sections the sheet numbered

 @param targetCount - translation sections the sheet numbered

 @throws SectionPairingError when an index falls outside its document

 @example
 ```ts
 assertIndicesExist({ pairs, sourceCount: 8, targetCount: 9, },);
 ```
 */
function assertIndicesExist(
  {
    pairs,
    sourceCount,
    targetCount,
  }: {
    readonly pairs: readonly SectionPair[];
    readonly sourceCount: number;
    readonly targetCount: number;
  },
): void {
  for (const pair of pairs) {
    if ((pair.source < 0) || (pair.source >= sourceCount))
      throw new SectionPairingError({
        message: `pairing names original section ${String(pair.source,)}, and there are ${
          String(sourceCount,)
        }`,
      },);
    if ((pair.target < 0) || (pair.target >= targetCount))
      throw new SectionPairingError({
        message: `pairing names translation section ${String(pair.target,)}, and there are ${
          String(targetCount,)
        }`,
      },);
  }
}

/**
 Refuses any pairing that is not strictly increasing on both sides.

 Both documents say things in the same order, so a backwards step is a reply
 that did not read them as documents. A step that stands still on either side
 claims one section renders two, which {@link SectionPair}'s one-to-one
 downstream cannot carry.

 @param pairs - correspondences as the model gave them

 @throws SectionPairingError when the pairing repeats or reverses

 @example
 ```ts
 assertStrictlyIncreasing({ pairs, },);
 ```
 */
function assertStrictlyIncreasing(
  { pairs, }: { readonly pairs: readonly SectionPair[]; },
): void {
  for (const [at, pair,] of pairs.entries()) {
    /**
     Pair before this one, absent at the first position.
     */
    const previous = pairs[at - 1];
    if (previous === undefined)
      continue;
    if (pair.source <= previous.source)
      throw new SectionPairingError({
        message: `pairing does not advance on the original side at position ${String(at,)}`,
      },);
    if (pair.target <= previous.target)
      throw new SectionPairingError({
        message: `pairing does not advance on the translation side at position ${String(at,)}`,
      },);
  }
}

/**
 Reads a model's section pairing, refusing anything that cannot be used as one.

 @param value - parsed model reply

 @param sourceCount - original sections the sheet numbered

 @param targetCount - translation sections the sheet numbered

 @returns Pairs in document order, strictly increasing on both sides

 @throws SectionPairingError when the reply is not a usable pairing

 @example
 ```ts
 const pairs = readSectionPairing({ value, sourceCount: 8, targetCount: 9, },);
 ```
 */
export function readSectionPairing(
  {
    value,
    sourceCount,
    targetCount,
  }: {
    readonly value: unknown;
    readonly sourceCount: number;
    readonly targetCount: number;
  },
): readonly SectionPair[] {
  if (!isSectionPairingWire(value,))
    throw new SectionPairingError({
      message: 'reply is not a section pairing: expected {"pairs":[{"source":n,"target":n}]}',
    },);

  /**
   Pairs in the order the model gave them.
   */
  const { pairs, } = value;
  assertIndicesExist({
    pairs,
    sourceCount,
    targetCount,
  },);
  assertStrictlyIncreasing({ pairs, },);
  return pairs;
}

//endregion Section pairing reader
