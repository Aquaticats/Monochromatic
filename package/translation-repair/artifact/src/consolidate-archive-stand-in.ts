import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  type ArchiveDispute,
  archiveDisputeNote,
  describeArchiveDispute,
  disputedWordingsOf,
} from '@monochromatic-dev/translation-repair-archive/ts';
import type { DisputedWording, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { LaneChoice, } from '@monochromatic-dev/translation-repair-lane-contest/ts';

//region Consolidation archive stand-in
// The consolidation's side of class one hundred seven: on a disputed slice
// the repair lane's text is the archive wording every sheet, key and
// standing reads, and where the contest chose neither lane the kept standing
// is that stand-in, which the page must then be written with rather than
// left as the contest found it.
//
// ONLY WHERE THE STAND-IN MAY STAND (owner answer 2026-09-27, "No eligible
// standing"). Where the checkers confirmed no repair of the disputed reading,
// the archive's own wording stays the incumbent every sheet shows as
// evidence, and it and the repair lane's text are disputed wordings the
// standing verdict, the lane offer and the producers' floor refuse.

/**
 Archive wording as one consolidation settlement takes it.

 @param archiveDisputes - disputed slices, absent where none were read

 @param sliceIndex - slice being settled

 @param incumbentText - archive's own wording of the slice

 @param choice - what the lane contest settled

 @param l - driver logger

 @returns Wording standing as the archive here, whether a kept standing is
 the stand-in the page must carry, the sheet note on a disputed slice (class
 one hundred eight), and the wordings the slice refuses, none on an
 undisputed one

 @example
 ```ts
 const { incumbentText, standInShips, } = archiveStandInFor({ archiveDisputes, sliceIndex, incumbentText, choice, l, },);
 ```
 */
export function archiveStandInFor(
  {
    archiveDisputes,
    sliceIndex,
    incumbentText,
    choice,
    l,
  }: {
    readonly archiveDisputes?: ReadonlyMap<number, ArchiveDispute>;
    readonly sliceIndex: number;
    readonly incumbentText: string;
    readonly choice: LaneChoice;
    readonly l: Logger;
  },
): {
  readonly incumbentText: string;
  readonly standInShips: boolean;
  readonly disputeNote?: string;
  readonly disputedWordings: readonly DisputedWording[];
} {
  /**
   Dispute over this slice's archive rendering, absent for most slices.
   */
  const dispute = archiveDisputes?.get(sliceIndex,);
  if (dispute === undefined) {
    return {
      incumbentText,
      standInShips: false,
      disputedWordings: [],
    };
  }
  /**
   Wordings this slice refuses.
   */
  const disputedWordings = disputedWordingsOf({
    dispute,
    archiveText: incumbentText,
  },);
  if (!dispute.standInEligible) {
    l.info(
      `${describeArchiveDispute({ dispute, },)}; the archive stays the consolidation's incumbent, refused with the `
        + 'repair lane\'s text as a standing',
    );
    return {
      incumbentText,
      standInShips: false,
      disputeNote: archiveDisputeNote({ dispute, },),
      disputedWordings,
    };
  }
  l.info(`${describeArchiveDispute({ dispute, },)}; the stand-in is the consolidation's incumbent`,);
  return {
    incumbentText: dispute.standIn,
    standInShips: choice === 'neither',
    disputeNote: archiveDisputeNote({ dispute, },),
    disputedWordings,
  };
}

//endregion Consolidation archive stand-in
