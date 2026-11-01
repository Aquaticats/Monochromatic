import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { TRIAL_ARMS, } from './window-trial-report.ts';

//region Window trial order
// Which call position each arm gets, decided per slice rather than fixed.
//
// A FIXED ORDER ALIASES THE TREATMENT ONTO THE CLOCK. The arms of one slice are
// bought back to back, and the wide arm used to be third every time. Anything
// that drifts across those three calls, provider rate limiting biting after two
// rounds of six, a queue filling, a deadline growing more likely to be missed,
// lands entirely on the wide arm. Worse, it lands in the direction the trial
// predicts, because a degraded round declines and a decline keeps the archive.
//
// THE TWO NARROW ARMS CANNOT DETECT IT. They sit at two positions, and the wide
// arm sits at a third that neither of them ever occupies, so their difference
// bounds noise between positions one and two while saying nothing about position
// three. No statistic recovers this afterwards: the treatment and the position
// are the same variable.
//
// SO THE POSITION IS ASSIGNED PER SLICE, deterministically from the slice's own
// identity. Across the draw the wide arm sits first, second and last in roughly
// equal numbers, position effects average out instead of accumulating on one
// arm, and the rows carry the position so the effect can be estimated rather
// than assumed away. Deterministic rather than random so a resumed run, or a
// rerun for verification, assigns the same order to the same slice.

/**
 Arms every slice buys, in canonical order.

 This is the SET the runner owes, not the sequence it buys them in.
 */
export const TRIAL_ARM_SET: readonly string[] = [
  TRIAL_ARMS.narrowFirst,
  TRIAL_ARMS.narrowSecond,
  TRIAL_ARMS.wide,
];

/**
 Hex characters of the digest read as the position.

 Eight is well inside the exact-integer range and far more entropy than three
 buckets need.
 */
const DIGEST_CHARS = 8;

/**
 Base the digest is written in.
 */
const HEX = 16;

/**
 Order one slice buys its arms in.

 WIDE FIRST, MIDDLE OR LAST depending on the slice, so no position belongs to
 one arm. The two narrow arms keep their relative order in whatever positions
 are left, which costs nothing: they are interchangeable by construction, since
 their whole purpose is to be the same treatment twice.

 @param protocol - digest this run buys under, so a protocol change reshuffles
 rather than repeating one assignment forever

 @param entryId - entry the slice belongs to

 @param sliceIndex - slice position within that entry

 @returns All three arms, once each, in buying order

 @example
 ```ts
 const order = armOrderFor({ protocol, entryId: 'Mittens', sliceIndex: 7, },);
 ```
 */
export function armOrderFor(
  {
    protocol,
    entryId,
    sliceIndex,
  }: {
    readonly protocol: string;
    readonly entryId: string;
    readonly sliceIndex: number;
  },
): readonly string[] {
  /**
   Position the wide arm takes for this slice, zero-based.
   */
  const widePosition = Number.parseInt(
    hashContent({
      content: JSON.stringify([
        protocol,
        entryId,
        sliceIndex,
      ],),
    },)
      .slice(
      0,
      DIGEST_CHARS,
      ),
    HEX,
  ) % TRIAL_ARM_SET.length;

  /**
   Narrow arms, which keep their relative order wherever they land.

   Costs nothing: they are interchangeable by construction, since their whole
   purpose is to be the same treatment twice.
   */
  const narrow = [
    TRIAL_ARMS.narrowFirst,
    TRIAL_ARMS.narrowSecond,
  ];

  // The wide arm is INSERTED at its position rather than swapped in, so the
  // result is all three arms exactly once whatever the digest said.
  return [
    ...narrow.slice(
      0,
      widePosition,
    ),
    TRIAL_ARMS.wide,
    ...narrow.slice(widePosition,),
  ];
}

//endregion Window trial order
