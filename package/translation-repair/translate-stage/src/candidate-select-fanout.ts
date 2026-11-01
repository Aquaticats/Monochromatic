import { MIN_SELECTION_BALLOTS, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  type FanOutMode,
  firstRoundWindow,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Selection fan-out
// A WINDOW THAT COULD NOT HOLD TWO ANSWERING JUDGES IS NO WINDOW. A winner needs
// `MIN_SELECTION_BALLOTS` ballots and no summed weight (the owner's rulings of
// 2026-10-25, "Every weight floor"), so a window of quorum plus one seats enough
// judges to decide wherever it holds two. It used to be sized by weight: the
// select stage seats producers as judges at half weight (`SELF_VOTE_WEIGHT`) and
// shipped nothing under a weight of two, a candidate every seated model wrote
// (the archive-block review, where the reviewers are the judges and
// byte-identical revisions merge into one candidate) was carried by self-votes
// alone, and the four-seat bench's window of three halves fell short, so the
// whole bench was asked. The window is a saving, not a verdict: where it could
// not hold two judges the whole bench is asked.

/**
 Chooses how many judges a selection round asks.

 @param judgeCount - seats on the judge bench

 @param requested - what the caller asked for; absent for the production
 default

 @returns Whole bench where the window holds fewer judges than a winner needs
 ballots; the request otherwise

 @example
 ```ts
 const mode = selectionFanOut({ judgeCount: 4, },);
 ```
 */
export function selectionFanOut(
  {
    judgeCount,
    requested = 'window',
  }: {
    readonly judgeCount: number;
    readonly requested?: FanOutMode;
  },
): FanOutMode {
  if (requested === 'whole-bench')
    return 'whole-bench';
  return (firstRoundWindow({ benchSize: judgeCount, },) < MIN_SELECTION_BALLOTS) ? 'whole-bench' : 'window';
}

//endregion Selection fan-out
