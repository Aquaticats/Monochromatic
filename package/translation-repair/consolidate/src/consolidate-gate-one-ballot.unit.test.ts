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
      name: 'KEEPS THE STANDING when the gate heard one answering judge over an ineligible standing, so the proposal '
        + 'the slate chose does not ship on a single ballot',
      fn: async () => {
        const messages: string[] = [];
        const gated = shipPastForfeitStanding({
          outcome: neitherOver({ usable: 1, },),
          standingEligible: false,
          l: capturingLogger({ messages, },),
        },);
        expect(gated.ships,).toBe('standing',);
      },
    },),

    it({
      name: 'KEEPS THE STANDING when the gate heard one answering judge over a standing the contest majority called flawed',
      fn: async () => {
        const messages: string[] = [];
        const gated = shipPastForfeitStanding({
          outcome: neitherOver({ usable: 1, },),
          standingEligible: true,
          standingFlawedByMajority: true,
          l: capturingLogger({ messages, },),
        },);
        expect(gated.ships,).toBe('standing',);
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
          l: capturingLogger({ messages, },),
        },);
        expect(gated.ships,).toBe('consolidated',);
      },
    },),
  ],
},);
