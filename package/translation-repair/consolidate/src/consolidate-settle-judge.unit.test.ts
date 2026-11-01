/**
 Tests for how the consolidation's slate judging names an absence it cannot
 reach.

 WHAT THIS FILE PINS: a decision passes through; an absence over a withheld
 standing, which no judging reaches since a withheld slate ships its
 preference (ledger B51), becomes the fault `WithheldSlateAbsenceError`
 names, carrying the absence; an absence over an eligible standing, and any
 other failure, passes through unchanged.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { TranslateAbsenceError, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  requireWithheldSlateDecided,
  WithheldSlateAbsenceError,
} from '../dist/final/node/index.mjs';

/**
 An absence the judging could raise over a slate nobody was heard on.
 */
const ABSENCE = new TranslateAbsenceError({
  reason: 'no-voice-heard',
  findings: ['translate-no-voice-heard',],
},);

/**
 Runs the narrowing over a judging that fails, and returns what it raised.

 @param failure - what the judging raises

 @param standingEligible - whether the standing passed the deterministic gate

 @returns What the narrowing raised
 */
async function raisedBy(
  {
    failure,
    standingEligible,
  }: {
    readonly failure: Error;
    readonly standingEligible: boolean;
  },
): Promise<unknown> {
  try {
    await requireWithheldSlateDecided({
      judged: Promise.reject(failure,),
      standingEligible,
    },);
    return 'decided';
  }
  catch (error) {
    return error;
  }
}

await describe({
  name: requireWithheldSlateDecided.name,
  children: [
    it({
      name: 'RETURNS THE DECISION the judging reached, over a withheld standing and an eligible one',
      fn: async () => {
        /**
         A decision the judges reached.
         */
        const decided = { decision: 'judged', text: 'The cat naps by the window.', } as const;
        expect([
          await requireWithheldSlateDecided({ judged: Promise.resolve(decided,), standingEligible: false, },),
          await requireWithheldSlateDecided({ judged: Promise.resolve(decided,), standingEligible: true, },),
        ],).toEqual([decided, decided,],);
      },
    },),
    it({
      name: 'NAMES AN ABSENCE OVER A WITHHELD STANDING AS THE FAULT IT IS, carrying the absence and its reason '
        + '(ledger B51: the settlement used to keep the archive on it, an exit no judging reached)',
      fn: async () => {
        /**
         What the narrowing raised over a withheld standing.
         */
        const raised = await raisedBy({
          failure: ABSENCE,
          standingEligible: false,
        },);
        if (!(raised instanceof WithheldSlateAbsenceError))
          throw new Error('expected WithheldSlateAbsenceError',);
        expect({
          reason: raised.reason,
          cause: raised.cause,
        },).toEqual({
          reason: ABSENCE.reason,
          cause: ABSENCE,
        },);
      },
    },),
    it({
      name: 'PASSES AN ABSENCE OVER AN ELIGIBLE STANDING THROUGH UNCHANGED, and any other failure over either',
      fn: async () => {
        /**
         A failure that is no absence.
         */
        const other = new Error('the cat unplugged the router',);
        expect([
          await raisedBy({ failure: ABSENCE, standingEligible: true, },),
          await raisedBy({ failure: other, standingEligible: false, },),
          await raisedBy({ failure: other, standingEligible: true, },),
        ],).toEqual([ABSENCE, other, other,],);
      },
    },),
  ],
},);
