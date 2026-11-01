/**
 Tests for pair agreement across every voice.

 Both pairing stages once took candidates from the first usable reply and
 only counted the others, so a pair two later voices named vanished when the
 first omitted it. These cases pin agreement per pair, the vote rule for a
 contested source, and the strictly increasing result the step builders need.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { agreePairs, } from '../dist/final/node/index.mjs';

/**
 Voices a pair needs, as both stages set it.
 */
const NEEDED = 2;

await describe({
  name: agreePairs.name,
  children: [
    it({
      name: 'KEEPS a pair two later voices named though the first voice omitted it',
      fn: async () => {
        expect(agreePairs({
          pairings: [
            [
              {
                source: 0,
                target: 0,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 1,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 1,
              },
            ],
          ],
          needed: NEEDED,
        },),).toEqual({
          pairs: [
            {
              source: 0,
              target: 0,
            },
            {
              source: 1,
              target: 1,
            },
          ],
          findings: [],
        },);
      },
    },),
    it({
      name: 'DROPS a pair only one voice named, whichever voice that was',
      fn: async () => {
        expect(agreePairs({
          pairings: [
            [
              {
                source: 0,
                target: 0,
              },
              {
                source: 2,
                target: 2,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 1,
              },
            ],
          ],
          needed: NEEDED,
        },).pairs,).toEqual([
          {
            source: 0,
            target: 0,
          },
        ],);
      },
    },),
    it({
      name: 'PREFERS the better-voted target where voices name one source against two, and says'
        + ' nothing when nothing agreed was lost',
      fn: async () => {
        expect(agreePairs({
          pairings: [
            [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 1,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 1,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 2,
              },
            ],
          ],
          needed: NEEDED,
        },),).toEqual({
          pairs: [
            {
              source: 0,
              target: 0,
            },
            {
              source: 1,
              target: 1,
            },
          ],
          findings: [],
        },);
      },
    },),
    it({
      name: 'DROPS a contested source whose targets tie, and names it, since a pairing the voices'
        + ' split on evenly is nobody\'s agreement',
      fn: async () => {
        expect(agreePairs({
          pairings: [
            [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 1,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 1,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 2,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
              {
                source: 1,
                target: 2,
              },
            ],
          ],
          needed: NEEDED,
        },),).toEqual({
          pairs: [
            {
              source: 0,
              target: 0,
            },
          ],
          findings: ['contested (source 1 named against 2 targets)'],
        },);
      },
    },),
    it({
      name: 'KEEPS corroborated paragraph splits and merges in many-to-many mode, because block wire explicitly represents repeated positions on either side',
      fn: async () => {
        expect(agreePairs({
          pairings: [
            [
              { source: 0, target: 0, },
              { source: 0, target: 1, },
              { source: 1, target: 1, },
            ],
            [
              { source: 0, target: 0, },
              { source: 0, target: 1, },
              { source: 1, target: 1, },
            ],
          ],
          needed: NEEDED,
          pairingShape: 'many-to-many',
        },),).toEqual({
          pairs: [
            { source: 0, target: 0, },
            { source: 0, target: 1, },
            { source: 1, target: 1, },
          ],
          findings: [],
        },);
      },
    },),
    it({
      name: 'ORDERS CORROBORATED SPLIT CANONICALLY when higher target was named first by an earlier voice',
      fn: async () => {
        expect(agreePairs({
          pairings: [
            [{ source: 0, target: 1, },],
            [
              { source: 0, target: 0, },
              { source: 0, target: 1, },
            ],
            [
              { source: 0, target: 0, },
              { source: 0, target: 1, },
            ],
          ],
          needed: NEEDED,
          pairingShape: 'many-to-many',
        },).pairs,).toEqual([
          { source: 0, target: 0, },
          { source: 0, target: 1, },
        ],);
      },
    },),
    it({
      name: 'DOES NOT INVENT A SPLIT from tied alternatives no voice named together',
      fn: async () => {
        expect(agreePairs({
          pairings: [
            [{ source: 0, target: 0, },],
            [{ source: 0, target: 0, },],
            [{ source: 0, target: 1, },],
            [{ source: 0, target: 1, },],
          ],
          needed: NEEDED,
          pairingShape: 'many-to-many',
        },),).toEqual({
          pairs: [],
          findings: ['contested (source 0 named against 2 targets)'],
        },);
      },
    },),
    it({
      name: 'KEEPS the result strictly increasing on both sides, skipping and naming an agreed'
        + ' pair that would run backwards',
      fn: async () => {
        expect(agreePairs({
          pairings: [
            [
              {
                source: 0,
                target: 2,
              },
              {
                source: 1,
                target: 1,
              },
            ],
            [
              {
                source: 0,
                target: 2,
              },
              {
                source: 1,
                target: 1,
              },
            ],
          ],
          needed: NEEDED,
        },),).toEqual({
          pairs: [
            {
              source: 0,
              target: 2,
            },
          ],
          findings: ['non-monotone (1,1 runs back behind 2)'],
        },);
      },
    },),
    it({
      name: 'DECIDES A CONTESTED TARGET BY VOTES: a later source that outvotes the earlier claim on'
        + ' the same target takes it and the earlier source is left unpaired, the shape of the first'
        + ' noname pass where the heading took the paragraph\'s partner by source order alone',
      fn: async () => {
        expect(agreePairs({
          pairings: [
            [
              {
                source: 1,
                target: 0,
              },
            ],
            [
              {
                source: 1,
                target: 0,
              },
            ],
            [
              {
                source: 1,
                target: 0,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
            ],
          ],
          needed: NEEDED,
          pairingShape: 'many-to-many',
        },),).toEqual({
          pairs: [
            {
              source: 1,
              target: 0,
            },
          ],
          findings: ['contested target (target 0: source 1 outvotes source 0, 3 to 2)'],
        },);
      },
    },),
    it({
      name: 'KEEPS THE EARLIER CLAIM on a contested target when the later source has fewer votes,'
        + ' and names the loss rather than calling it non-monotone',
      fn: async () => {
        expect(agreePairs({
          pairings: [
            [
              {
                source: 1,
                target: 0,
              },
            ],
            [
              {
                source: 1,
                target: 0,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
            ],
          ],
          needed: NEEDED,
        },),).toEqual({
          pairs: [
            {
              source: 0,
              target: 0,
            },
          ],
          findings: ['contested target (target 0: source 1 loses to source 0, 2 to 3)'],
        },);
      },
    },),
    it({
      name: 'DROPS BOTH CLAIMS on a contested target that ties, since a target the voices split on'
        + ' evenly is nobody\'s agreement, as a contested source already is',
      fn: async () => {
        expect(agreePairs({
          pairings: [
            [
              {
                source: 0,
                target: 0,
              },
            ],
            [
              {
                source: 0,
                target: 0,
              },
            ],
            [
              {
                source: 1,
                target: 0,
              },
            ],
            [
              {
                source: 1,
                target: 0,
              },
            ],
          ],
          needed: NEEDED,
          pairingShape: 'many-to-many',
        },),).toEqual({
          pairs: [],
          findings: ['contested target (target 0: sources 0 and 1 tie at 2 votes)'],
        },);
      },
    },),
    it({
      name: 'AGREES on nothing from a single voice, because a pairing one model invented is not'
        + ' agreement',
      fn: async () => {
        expect(agreePairs({
          pairings: [
            [
              {
                source: 0,
                target: 0,
              },
            ],
          ],
          needed: NEEDED,
        },).pairs,).toEqual([
        ],);
      },
    },),
  ],
},);
