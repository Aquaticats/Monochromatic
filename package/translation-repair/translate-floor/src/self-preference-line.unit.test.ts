/**
 Tests for how a self-preference result is stated.

 WHAT THESE PIN is that the three outcomes read differently. The measurement
 distinguishes "no favouritism" from "nobody was asked" from "nobody was left
 to answer"; a wording that printed a number for all three would collapse them
 again at the only point a human reads.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { describeSelfPreference, } from '../dist/final/node/index.mjs';

await describe({
  name: describeSelfPreference.name,
  children: [
    it({
      name: 'states the excess FIRST and both rates behind it, so a reader sees what was concluded '
        + 'and what it was concluded from without having to subtract',
      fn: async () => {
        /**
         Producers backed themselves twice as often as anyone else did.
         */
        const line = describeSelfPreference({ preference: {
          kind: 'measured',
          opportunities: 4,
          ownVotes: 2,
          otherBallots: 8,
          otherVotes: 2,
          ownRate: 0.5,
          disinterestedRate: 0.25,
          excess: 0.25,
        }, },);
        expect(line.includes('self-preference 0.25',),).toBe(true,);
        expect(line.includes('own 0.50 of 4',),).toBe(true,);
        expect(line.includes('others 0.25 of 8',),).toBe(true,);
      },
    },),
    it({
      name: 'says the question was NOT PUT when no producer judged its own candidate, rather than '
        + 'printing a zero: a zero there would read as evidence of no favouritism, which is the '
        + 'opposite of no evidence',
      fn: async () => {
        /**
         Nobody with a stake ever cast a ballot.
         */
        const line = describeSelfPreference({ preference: {
          kind: 'no-stakeholder-ballots',
          opportunities: 0,
          ownVotes: 0,
          otherBallots: 6,
          otherVotes: 3,
        }, },);
        expect(line.includes('not put',),).toBe(true,);
        expect(line.includes('0.00',),).toBe(false,);
      },
    },),
    it({
      name: 'says UNANSWERABLE when every judge held a stake, and names how many ballots that was, '
        + 'since a roster of all producers is a shape someone chose rather than data going missing',
      fn: async () => {
        const line = describeSelfPreference({ preference: {
          kind: 'no-disinterested-ballots',
          opportunities: 5,
          ownVotes: 4,
          otherBallots: 0,
          otherVotes: 0,
        }, },);
        expect(line.includes('unanswerable',),).toBe(true,);
        expect(line.includes('5',),).toBe(true,);
        expect(line.includes('0.00',),).toBe(false,);
      },
    },),
    it({
      name: 'renders a NEGATIVE excess with its sign, so a roster harder on itself than on its '
        + 'neighbours is not read as favouritism of the same size',
      fn: async () => {
        const line = describeSelfPreference({ preference: {
          kind: 'measured',
          opportunities: 4,
          ownVotes: 0,
          otherBallots: 8,
          otherVotes: 4,
          ownRate: 0,
          disinterestedRate: 0.5,
          excess: -0.5,
        }, },);
        expect(line.includes('self-preference -0.50',),).toBe(true,);
      },
    },),
    it({
      name: 'gives the three outcomes three DISTINCT openings, which is what lets a reader scanning '
        + 'a report tell them apart without reading the numbers',
      fn: async () => {
        /**
         One line per outcome, from the same counts where possible.
         */
        const lines = [
          describeSelfPreference({ preference: {
            kind: 'measured',
            opportunities: 4,
            ownVotes: 2,
            otherBallots: 8,
            otherVotes: 2,
            ownRate: 0.5,
            disinterestedRate: 0.25,
            excess: 0.25,
          }, },),
          describeSelfPreference({ preference: {
            kind: 'no-stakeholder-ballots',
            opportunities: 0,
            ownVotes: 0,
            otherBallots: 8,
            otherVotes: 2,
          }, },),
          describeSelfPreference({ preference: {
            kind: 'no-disinterested-ballots',
            opportunities: 4,
            ownVotes: 2,
            otherBallots: 0,
            otherVotes: 0,
          }, },),
        ];
        expect(new Set(lines,).size,).toBe(lines.length,);
      },
    },),
  ],
},);
