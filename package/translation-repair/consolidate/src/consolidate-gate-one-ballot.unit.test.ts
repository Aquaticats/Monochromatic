/**
 Tests that a gate which heard one answering judge never ships a consolidation on that ballot.

 THE OWNER'S RULINGS OF 2026-10-25, quoted where the rule is stated
 (`doc/design-commitments-votes-repairs-and-run-offs.md`): "floor of 3 - don't set a floor.", "Two ballots decide
 (Recommended)" and "Every weight floor". A claim or contest is decided with two or more answering judges, and one
 judge alone never decides. In XingZ6021 five consolidations (slices 13, 14, 33, 44 and 84) shipped over a standing the
 deterministic rule had refused, after a gate that heard ONE usable ballot settled on neither rendering and the rule
 "an undecided gate ships the slate's choice" then shipped the proposal (log line
 `settled on neither over an ineligible standing, so the proposal the slate chose ships (1 usable ballot)`).
 A gate that heard fewer than two judges has no verdict, undecided or otherwise, to ship a proposal past.

 THE SLATE'S OWN PICK IS A DECISION OF ITS OWN (the owner's answer of 2026-10-25, "Ship the slate's wording"): when
 two or more slate ballots chose the proposal, the want of a gate verdict does not undo that decision, and the
 archive the deterministic rule refused cannot ship, so over such an archive the proposal ships. A proposal the slate
 chose on one ballot, or a standing the contest majority called flawed (which passes the rule), keeps the standing.

 Fixtures are cat-themed invention mirroring corpus structure only. No call leaves the process.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  CONSOLIDATE_GATE_QUORUM,
  type ConsolidateGateOutcome,
  type GateBallot,
  type GateChoice,
  shipPastForfeitStanding,
  slatePickBallotsOf,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

/**
 One usable ballot naming a rendering.

 @param choice - rendering the judge would publish

 @returns Ballot as the gate records it

 @example
 ```ts
 const ballot = ballotNaming({ choice: 'neither', },);
 ```
 */
function ballotNaming({ choice, }: { readonly choice: GateChoice; },): GateBallot {
  return {
    choice,
    unsupported: [],
    unsupportedRaw: [],
    dropped: [],
    droppedRaw: [],
    reason: 'fixture',
  };
}

/**
 A gate outcome that settled on neither rendering over some number of usable ballots.

 @param usable - answering judges the gate heard

 @returns Outcome the gate returns before the forfeit rule reads it

 @example
 ```ts
 const outcome = neitherOver({ usable: 1, },);
 ```
 */
function neitherOver({ usable, }: { readonly usable: number; },): ConsolidateGateOutcome {
  return {
    choice: 'neither',
    ships: 'standing',
    ballots: Array.from({ length: usable, },).map(function toBallot(): GateBallot {
      return ballotNaming({ choice: 'neither', },);
    },),
    usable,
    findings: [],
  };
}

await describe({
  name: shipPastForfeitStanding.name,
  children: [
    it({
      name: 'KEEPS THE STANDING when the gate heard one answering judge over an ineligible standing and the slate\'s '
        + 'pick drew one ballot, so the proposal does not ship on a single ballot anywhere',
      fn: async () => {
        const messages: string[] = [];
        const gated = shipPastForfeitStanding({
          outcome: neitherOver({ usable: 1, },),
          standingEligible: false,
          slatePickBallots: 1,
          l: capturingLogger({ messages, },),
        },);
        expect(gated.ships,).toBe('standing',);
      },
    },),

    it({
      name: 'KEEPS THE STANDING when the gate heard one answering judge over a standing the contest majority called '
        + 'flawed, even where the slate\'s pick drew two ballots: only the rule-failing archive gives way to the '
        + 'slate\'s pick (the owner\'s answer of 2026-10-25, "Ship the slate\'s wording")',
      fn: async () => {
        const messages: string[] = [];
        const gated = shipPastForfeitStanding({
          outcome: neitherOver({ usable: 1, },),
          standingEligible: true,
          standingFlawedByMajority: true,
          slatePickBallots: 2,
          l: capturingLogger({ messages, },),
        },);
        expect(gated.ships,).toBe('standing',);
      },
    },),

    it({
      name: 'SHIPS THE SLATE\'S PICK when the gate heard one answering judge over an ineligible standing and two '
        + 'slate ballots chose the pick, since the want of a gate verdict does not undo the slate\'s two-ballot '
        + 'decision and the archive the rule refused cannot ship (the owner\'s answer of 2026-10-25, "Ship the '
        + 'slate\'s wording")',
      fn: async () => {
        const messages: string[] = [];
        const gated = shipPastForfeitStanding({
          outcome: neitherOver({ usable: 1, },),
          standingEligible: false,
          slatePickBallots: 2,
          l: capturingLogger({ messages, },),
        },);
        expect({
          ships: gated.ships,
          findings: gated.findings.map(function head(finding,): string {
            return finding.split(':',)[0] ?? '';
          },),
        },).toEqual({
          ships: 'consolidated',
          findings: ['unheard-gate-ships-slate-pick',],
        },);
      },
    },),

    it({
      name: 'SHIPS THE SLATE\'S PICK when the gate heard no answering judge at all over an ineligible standing and '
        + 'two slate ballots chose the pick',
      fn: async () => {
        const messages: string[] = [];
        const gated = shipPastForfeitStanding({
          outcome: neitherOver({ usable: 0, },),
          standingEligible: false,
          slatePickBallots: 3,
          l: capturingLogger({ messages, },),
        },);
        expect(gated.ships,).toBe('consolidated',);
      },
    },),

    it({
      name: 'STILL SHIPS THE PROPOSAL when two judges answered and settled on neither over an ineligible standing, '
        + 'the positive control: the gate heard enough judges to call the standing no default to keep',
      fn: async () => {
        const messages: string[] = [];
        expect(CONSOLIDATE_GATE_QUORUM,).toBe(2,);
        const gated = shipPastForfeitStanding({
          outcome: neitherOver({ usable: 2, },),
          standingEligible: false,
          slatePickBallots: 1,
          l: capturingLogger({ messages, },),
        },);
        expect(gated.ships,).toBe('consolidated',);
      },
    },),
  ],
},);

await describe({
  name: slatePickBallotsOf.name,
  children: [
    it({
      name: 'READS THE BALLOTS OF THE ROW AT THE SHIPPED POSITION, self-votes included, so a pick two judges named '
        + 'counts two whatever its weight',
      fn: async () => {
        expect(slatePickBallotsOf({
          decided: {
            shippedIndex: 2,
            perCandidate: [
              {
                index: 1,
                ballots: 1,
                fullVotes: 1,
                selfVotes: 0,
                weight: 1,
              },
              {
                index: 2,
                ballots: 2,
                fullVotes: 1,
                selfVotes: 1,
                weight: 1.5,
              },
            ],
          },
        },),).toBe(2,);
      },
    },),

    it({
      name: 'READS ZERO where the slate records no row for the shipped position, so a pick no ballot named never '
        + 'ships past an unheard gate',
      fn: async () => {
        expect(slatePickBallotsOf({
          decided: {
            shippedIndex: 3,
            perCandidate: [],
          },
        },),).toBe(0,);
      },
    },),
  ],
},);
