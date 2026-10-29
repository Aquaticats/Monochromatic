/**
 Tests for the selection bench reading: a round's findings say whether it was decided on a bench short of quorum, and
 no weight is compared (the owner's rulings of 2026-10-25, "floor of 3 - don't set a floor." and "Every weight floor").

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rosterQuorumSize, } from '@monochromatic-dev/translation-repair-stage';
import {
  MIN_SELECTION_BALLOTS,
  selectionBench,
  shortBenchFinding,
} from '../dist/final/node/index.mjs';

/**
 Seated wide bench the production run carries, as a size.
 */
const SEATED_BENCH = 11;

/**
 Seats Bedrock alone reaches on that bench.
 */
const BEDROCK_REACHABLE = 3;

await describe({
  name: 'selection bench',
  children: [
    describe({
      name: selectionBench.name,
      children: [
        it({
          name: 'READS A BENCH AT OR ABOVE QUORUM as not short, unreachable seats or not',
          fn: async () => {
            /** Quorum over the seated bench. */
            const quorum = rosterQuorumSize({ rosterSize: SEATED_BENCH, },);
            /** A full bench. */
            const full = selectionBench({ benchSize: SEATED_BENCH, unreachable: 0, },);
            expect(full.short,).toBe(false,);
            expect(full.quorum,).toBe(quorum,);
            expect(full.reachable,).toBe(SEATED_BENCH,);
            /** A bench that lost seats down to exactly quorum. */
            const atQuorum = selectionBench({ benchSize: SEATED_BENCH, unreachable: SEATED_BENCH - quorum, },);
            expect(atQuorum.short,).toBe(false,);
            /** The archive-block review's four seats, every one reachable. */
            const four = selectionBench({ benchSize: 4, unreachable: 0, },);
            expect(four.short,).toBe(false,);
          },
        },),

        it({
          name: 'READS A BENCH BELOW QUORUM as short and names it in the finding, which no longer sizes any threshold: '
            + 'two ballots are the floor on a short bench as on a full one',
          fn: async () => {
            /** Quorum over the seated bench. */
            const quorum = rosterQuorumSize({ rosterSize: SEATED_BENCH, },);
            /** Bedrock alone. */
            const bench = selectionBench({
              benchSize: SEATED_BENCH,
              unreachable: SEATED_BENCH - BEDROCK_REACHABLE,
            },);
            expect(bench.short,).toBe(true,);
            expect(bench.reachable,).toBe(BEDROCK_REACHABLE,);
            expect(bench.quorum,).toBe(quorum,);
            expect(MIN_SELECTION_BALLOTS,).toBe(2,);
            expect(shortBenchFinding({ bench, benchSize: SEATED_BENCH, },),).toBe(
              `select-short-bench (reachable ${String(BEDROCK_REACHABLE,)} of ${String(SEATED_BENCH,)}, quorum ${
                String(quorum,)
              })`,
            );
          },
        },),
      ],
    },),
  ],
},);
