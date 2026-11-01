/**
 * Tier state-machine tests. The full `attachComposer` boot path is
 * DOM-bound and out of scope for a Bun-only unit test; here we cover
 * the pure transition predicate (`decideTierTransition`) that drives
 * every promotion. See verification 16/16a in the plan for end-to-end
 * coverage of the DOM wiring.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  decideTierTransition,
  TIER_2_THRESHOLD,
  TIER_3_THRESHOLD,
} from './composer.ts';

await describe({
  name: '',
  children: [
    describe({
      name: decideTierTransition.name,
      children: [
        it({
          name: 'tier 1 below the tier-2 threshold stays put',
          fn: async () => {
            expect(decideTierTransition({
              tier: 1,
              length: TIER_2_THRESHOLD - 1,
              tier3Active: false,
              inEditMode: false,
            },)
              .kind,)
              .toBe('none',);
          },
        },),

        it({
          name: 'tier 1 at the tier-2 threshold promotes to tier 2',
          fn: async () => {
            expect(decideTierTransition({
              tier: 1,
              length: TIER_2_THRESHOLD,
              tier3Active: false,
              inEditMode: false,
            },)
              .kind,)
              .toBe('to-tier-2',);
          },
        },),

        it({
          name:
            'tier 1 jumping past the tier-3 threshold returns to-tier-2 (one step at a time)',
          fn: async () => {
            // The state machine takes one step per check; the next check
            // (after the textarea fires another input event) decides the
            // tier-2 -> tier-3 transition.
            expect(decideTierTransition({
              tier: 1,
              length: TIER_3_THRESHOLD + 100,
              tier3Active: false,
              inEditMode: false,
            },)
              .kind,)
              .toBe('to-tier-2',);
          },
        },),

        it({
          name: 'tier 2 below the tier-3 threshold stays put',
          fn: async () => {
            expect(decideTierTransition({
              tier: 2,
              length: TIER_3_THRESHOLD - 1,
              tier3Active: false,
              inEditMode: false,
            },)
              .kind,)
              .toBe('none',);
          },
        },),

        it({
          name: 'tier 2 at the tier-3 threshold promotes to tier 3',
          fn: async () => {
            expect(decideTierTransition({
              tier: 2,
              length: TIER_3_THRESHOLD,
              tier3Active: false,
              inEditMode: false,
            },)
              .kind,)
              .toBe('to-tier-3',);
          },
        },),

        it({
          name: 'tier 2 in edit mode does not promote to tier 3 even past the threshold',
          fn: async () => {
            expect(decideTierTransition({
              tier: 2,
              length: TIER_3_THRESHOLD + 100,
              tier3Active: false,
              inEditMode: true,
            },)
              .kind,)
              .toBe('none',);
          },
        },),

        it({
          name: 'tier 2 with tier3 already active is a no-op',
          fn: async () => {
            expect(decideTierTransition({
              tier: 2,
              length: TIER_3_THRESHOLD + 100,
              tier3Active: true,
              inEditMode: false,
            },)
              .kind,)
              .toBe('none',);
          },
        },),

        it({
          name: 'tier 3 stays in tier 3 regardless of body length (one-way)',
          fn: async () => {
            expect(decideTierTransition({
              tier: 3,
              length: 0,
              tier3Active: true,
              inEditMode: false,
            },)
              .kind,)
              .toBe('none',);
            expect(decideTierTransition({
              tier: 3,
              length: TIER_3_THRESHOLD * 2,
              tier3Active: true,
              inEditMode: false,
            },)
              .kind,)
              .toBe('none',);
          },
        },),

        it({
          name: 'thresholds are sensible (TIER_2 < TIER_3, both positive integers)',
          fn: async () => {
            expect(TIER_2_THRESHOLD,).toBeGreaterThan(0,);
            expect(TIER_3_THRESHOLD,).toBeGreaterThan(TIER_2_THRESHOLD,);
            expect(Number.isInteger(TIER_2_THRESHOLD,),).toBe(true,);
            expect(Number.isInteger(TIER_3_THRESHOLD,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
