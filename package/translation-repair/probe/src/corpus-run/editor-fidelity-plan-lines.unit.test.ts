/**
 Tests for the lines an editor fidelity run says about what it priced and what it kept: the preflight line a cap of
 zero ends on, with the passes over the pool it may price, and the line that says how many rows were kept. Fixtures are
 cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  editorKeptLine,
  editorPreflightLine,
} from '../../dist/final/node/index.mjs';
import {
  HIGH_CARD,
  LOW_CARD,
} from '../editor-fidelity.test-fixture.ts';

await describe({
  name: 'editor fidelity plan lines',
  children: [
    describe({
      name: editorPreflightLine.name,
      children: [
        it({
          name: 'SAYS WHAT A CAP OF ZERO BOUGHT: cards named and the references selected, that the corpus was read and '
            + 'no model was called, and that the ceiling is priced over every trial',
          fn: async () => {
            expect(editorPreflightLine({
              cardIds: [LOW_CARD,],
              selected: 3,
              planned: 88,
              total: 88,
            },),).toBe(
              `preflight only: 3 pinned reference specifications selected for ${LOW_CARD}; the corpus was read and no model was called; the ceiling is priced over every trial of the pool`,
            );
            expect(editorPreflightLine({
              cardIds: [LOW_CARD, HIGH_CARD,],
              selected: 1,
              planned: 2,
              total: 2,
            },),).toBe(
              `preflight only: 1 pinned reference specification selected for ${LOW_CARD}, ${HIGH_CARD}; the corpus was read and no model was called; the ceiling is priced over every trial of the pool`,
            );
          },
        },),

        it({
          name: 'SAYS THE CEILING IS PRICED OVER THE FIRST TRIALS when the arm was asked for fewer than the pool holds, '
            + 'so a preflight of a later arm does not claim to price the whole pool',
          fn: async () => {
            expect(editorPreflightLine({
              cardIds: [LOW_CARD,],
              selected: 53,
              planned: 41,
              total: 88,
            },),).toBe(
              `preflight only: 53 pinned reference specifications selected for ${LOW_CARD}; the corpus was read and no model was called; the ceiling is priced over the first 41 of 88 trials`,
            );
          },
        },),

        it({
          name: 'SAYS THE CEILING IS PRICED OVER EVERY ASK OF THE PASSES, or over the first asks of them, when the arm '
            + 'walks the pool more than once',
          fn: async () => {
            expect(editorPreflightLine({
              cardIds: [LOW_CARD,],
              selected: 53,
              planned: 264,
              total: 264,
              passes: 3,
            },),).toBe(
              `preflight only: 53 pinned reference specifications selected for ${LOW_CARD}; the corpus was read and no model was called; the ceiling is priced over every ask of the 3 passes over the pool`,
            );
            expect(editorPreflightLine({
              cardIds: [LOW_CARD,],
              selected: 53,
              planned: 200,
              total: 264,
              passes: 3,
            },),).toBe(
              `preflight only: 53 pinned reference specifications selected for ${LOW_CARD}; the corpus was read and no model was called; the ceiling is priced over the first 200 of 264 asks`,
            );
          },
        },),
      ],
    },),

    describe({
      name: editorKeptLine.name,
      children: [
        it({
          name: 'SAYS HOW MANY ROWS WERE KEPT AND WHERE',
          fn: async () => {
            expect(editorKeptLine({
              count: 1,
              keptAt: '/runs/x.json',
            },),).toBe('kept 1 editor row at /runs/x.json',);
            expect(editorKeptLine({
              count: 4,
              keptAt: '/runs/x.json',
            },),).toBe('kept 4 editor rows at /runs/x.json',);
          },
        },),
      ],
    },),
  ],
},);
