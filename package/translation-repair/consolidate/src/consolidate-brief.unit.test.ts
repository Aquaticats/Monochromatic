/**
 Tests for the brief a consolidating producer is shown.

 WHAT THIS FILE EXISTS TO STOP. A judge that answers `unsupported: ["repair"]`
 has put the substance of its finding in its reason, so pooling the findings
 into one list and the reasons into another would hand a producer a bare
 candidate name with nothing attached. Every judge's reading stays one block.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { LaneContestBallot, } from '@monochromatic-dev/translation-repair-lane-contest';
import { renderConsolidationBrief, } from '../dist/final/node/index.mjs';

/**
 One ballot with everything filled in.
 */
const full: LaneContestBallot = {
  choice: 'repair',
  unsupported: [ 'translate', ],
  unsupportedRaw: [ 'translate (invents an afternoon nap)', ],
  dropped: [ 'translate', ],
  droppedRaw: [ 'translate omits the second bowl', ],
  reason: 'the repair candidate keeps the window ledge',
};

await describe({
  name: renderConsolidationBrief.name,
  children: [
    it({
      name: 'ACCEPTS an empty roster by rendering nothing',
      fn: async () => {
        // A CONTEST THAT NEVER REACHED QUORUM still leaves two candidates worth
        // consolidating. An empty heading would read as judges having looked.
        expect(renderConsolidationBrief({ ballots: [], },),).toBe('',);
      },
    },),
    it({
      name: 'keeps one judge\'s choice, findings and reason together',
      fn: async () => {
        const brief = renderConsolidationBrief({ ballots: [ full, ], },);
        expect(brief,).toContain('Judge 1 would publish: repair',);
        expect(brief,).toContain('translate (invents an afternoon nap)',);
        expect(brief,).toContain('translate omits the second bowl',);
        expect(brief,).toContain('the repair candidate keeps the window ledge',);
      },
    },),
    it({
      name: 'names each finding list by what it claims',
      fn: async () => {
        const brief = renderConsolidationBrief({ ballots: [ full, ], },);
        expect(brief,).toContain('Says what the original does not',);
        expect(brief,).toContain('Omits what the original says',);
      },
    },),
    it({
      name: 'numbers judges from one, in ballot order',
      fn: async () => {
        const brief = renderConsolidationBrief({
          ballots: [
            full,
            {
              ...full,
              choice: 'neither',
              reason: 'both candidates lose the tail',
            },
          ],
        },);
        expect(brief,).toContain('Judge 1 would publish: repair',);
        expect(brief,).toContain('Judge 2 would publish: neither',);
      },
    },),
    it({
      name: 'REFUSES to name a model, since who said it is not evidence',
      fn: async () => {
        const brief = renderConsolidationBrief({ ballots: [ full, ], },);
        expect(brief.includes('hf:',),).toBe(false,);
      },
    },),
    it({
      name: 'drops findings that say nothing rather than listing blanks',
      fn: async () => {
        const brief = renderConsolidationBrief({
          ballots: [
            {
              ...full,
              // A zero-width space with a Hangul filler shows nothing to a
              // reader either, though trim() keeps it (ledger B40).
              unsupportedRaw: [ '', '   ', '\u{200B}\u{3164}', ],
              droppedRaw: [],
            },
          ],
        },);
        expect(brief.includes('Says what the original does not',),).toBe(false,);
        expect(brief.includes('Omits what the original says',),).toBe(false,);
        expect(brief,).toContain('Judge 1 would publish: repair',);
      },
    },),
    it({
      name: 'omits the reason line when a judge gave none, or gave only invisible characters (ledger B40)',
      fn: async () => {
        const brief = renderConsolidationBrief({
          ballots: [
            {
              ...full,
              reason: '   ',
            },
          ],
        },);
        expect(brief.includes('Why:',),).toBe(false,);
        expect(renderConsolidationBrief({
          ballots: [
            {
              ...full,
              reason: '\u{2060}',
            },
          ],
        },).includes('Why:',),).toBe(false,);
      },
    },),
    it({
      name: 'trims surrounding space off a finding',
      fn: async () => {
        const brief = renderConsolidationBrief({
          ballots: [
            {
              ...full,
              unsupportedRaw: [ '  repair adds a sunbeam  ', ],
            },
          ],
        },);
        expect(brief,).toContain('    - repair adds a sunbeam',);
      },
    },),
  ],
},);
