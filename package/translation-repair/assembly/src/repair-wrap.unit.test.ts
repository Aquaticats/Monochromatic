/**
 Tests for wrapping what the repair lane produced.

 WHAT THESE PIN is which outcomes are touched. `assembleRepair` builds the
 replacements AND the lane wordings out of one outcome list, and the delivery
 invariant splices the ledger's rows over the archive and demands the result
 equal the document the lane returned, byte for byte. Wrapping one consumer
 and not the other breaks that, so the list is wrapped once before either
 reads it.

 The second thing they pin is the demotion. A passage differing from the
 archive only in its wrapping becomes the archive once wrapped, and an outcome
 still claiming a change there fails `assertReplacementsChange` and the
 coherence rule that a replacement's wording may not be the archive's own. No
 slice in the pool settled 2026-08-18 does this, so the case is constructed
 here rather than observed.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { wrapRepairOutcomes, } from '../dist/final/node/index.mjs';
import { preparedPairAt, } from '@monochromatic-dev/translation-repair-slice-reading/ts/prepared-pair-at.test-fixture.ts';

/**
 Logger these hand to the lane, whose output is not what is under test.
 */
const l = tagged({ tag: 'repair-wrap-test', },);

/**
 Builds one settled repair outcome.

 MINIMAL BY DESIGN: the wrap reads three fields and carries the rest through
 untouched, so a fixture carrying the whole contract would test the spread
 rather than the decision.

 @param sliceIndex - slice index

 @param repairedText - wording this lane produced

 @param changed - whether it claims to differ from the archive

 @returns Outcome shaped as the lane settles one

 @example
 ```ts
 const outcome = outcomeOf({ sliceIndex: 0, repairedText: 'It naps.', changed: true, },);
 ```
 */
function outcomeOf(
  {
    sliceIndex,
    repairedText,
    changed,
  }: {
    readonly sliceIndex: number;
    readonly repairedText: string;
    readonly changed: boolean;
  },
): Parameters<typeof wrapRepairOutcomes>[0]['outcomes'][number] {
  return {
    sliceIndex,
    repairedText,
    changed,
    issues: [],
    resolvedIssueIds: [],
    claimAttributions: [],
  } as unknown as Parameters<typeof wrapRepairOutcomes>[0]['outcomes'][number];
}

/**
 One passage a governed producer returned, carrying sentence boundaries the
 wrap would break at if it were allowed to run.
 */
const GOVERNED_PRODUCED = 'The cat wakes. Sun is warm. She counts birds.';
await describe({
  name: wrapRepairOutcomes.name,
  children: [
    it({
      name: 'WRAPS WORDING THE LANE PRODUCED, which is the reason this exists: a model returns a '
        + 'passage as one line and the archive it replaces was wrapped',
      fn: async () => {
        /**
         One changed outcome, flat as a model wrote it.
         */
        const wrapped = wrapRepairOutcomes({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText: 'The cat sleeps on the sill.',
          },),],
          outcomes: [outcomeOf({
            sliceIndex: 0,
            repairedText: 'The tabby naps on the sill. It wakes at dusk.',
            changed: true,
          },),],
          lineStructuredSlices: new Set(),
          l,
        },);

        expect(wrapped[0]?.repairedText,).toBe('The tabby naps on the sill.\nIt wakes at dusk.',);
        expect(wrapped[0]?.changed,).toBe(true,);
      },
    },),

    it({
      name: 'LEAVES AN UNCHANGED OUTCOME BYTE-IDENTICAL, because it carries the archive’s own '
        + 'wording: wrapping a retention would report a change nobody decided on, and both the '
        + 'assembly assertion and the delivery coherence rule refuse exactly that',
      fn: async () => {
        /**
         Archive wording that the rule WOULD break, were it asked to.
         */
        const incumbentText = 'The cat sleeps on the sill. It wakes at dusk.';

        const wrapped = wrapRepairOutcomes({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText,
          },),],
          outcomes: [outcomeOf({
            sliceIndex: 0,
            repairedText: incumbentText,
            changed: false,
          },),],
          lineStructuredSlices: new Set(),
          l,
        },);

        expect(wrapped[0]?.repairedText,).toBe(incumbentText,);
        expect(wrapped[0]?.changed,).toBe(false,);
      },
    },),

    it({
      name: 'DEMOTES TO A RETENTION when wrapping is all that separated the wording from the '
        + 'archive. An outcome still claiming a change there fails the assembly assertion, so the '
        + 'flag is re-derived from the wrapped text rather than carried forward',
      fn: async () => {
        /**
         Archive wording, already written as the rule would write it.
         */
        const incumbentText = 'It naps.\nIt wakes.';

        const wrapped = wrapRepairOutcomes({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText,
          },),],
          outcomes: [outcomeOf({
            sliceIndex: 0,
            repairedText: 'It naps. It wakes.',
            changed: true,
          },),],
          lineStructuredSlices: new Set(),
          l,
        },);

        expect(wrapped[0]?.repairedText,).toBe(incumbentText,);
        expect(wrapped[0]?.changed,).toBe(false,);
      },
    },),

    it({
      name: 'DEMOTES AN OUTCOME WHOSE WORDING IS THE ARCHIVE\'S WITH ITS SOFT LINE BREAKS ELSEWHERE, even when '
        + 'the wrap leaves it as it is, and ships the archive\'s own bytes: the site renders a soft break as a '
        + 'space, so the page would not change and the outcome would report a change nobody made (ledger B26)',
      fn: async () => {
        /**
         Archive wording with a soft break inside its paragraph.
         */
        const incumbentText = 'The cat naps on the mat\nall afternoon.';
        const wrapped = wrapRepairOutcomes({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText,
          },),],
          outcomes: [outcomeOf({
            sliceIndex: 0,
            repairedText: 'The cat naps on the mat all afternoon.',
            changed: true,
          },),],
          lineStructuredSlices: new Set(),
          l,
        },);

        expect(wrapped[0]?.repairedText,).toBe(incumbentText,);
        expect(wrapped[0]?.changed,).toBe(false,);
      },
    },),

    it({
      name: 'DEMOTES A BLOCKQUOTE THAT IS THE ARCHIVE\'S REWRAPPED, which the soft-break fold alone cannot '
        + 'see, since it folds top-level paragraphs only (ledger B26)',
      fn: async () => {
        /**
         Archive quotation on one line.
         */
        const incumbentText = '> The cat naps. The dog waits.';
        const wrapped = wrapRepairOutcomes({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText,
          },),],
          outcomes: [outcomeOf({
            sliceIndex: 0,
            repairedText: '> The cat naps.\n> The dog waits.',
            changed: true,
          },),],
          lineStructuredSlices: new Set(),
          l,
        },);

        expect(wrapped[0]?.repairedText,).toBe(incumbentText,);
        expect(wrapped[0]?.changed,).toBe(false,);
      },
    },),

    it({
      name: 'DEMOTES A LINE-STRUCTURED OUTCOME that differs from the archive only after its last character, '
        + 'and KEEPS one whose lines differ, since there the line breaks are the producer\'s work (ledger B26)',
      fn: async () => {
        /**
         Archive wording, one line per original line.
         */
        const incumbentText = 'The cat wakes.\nSun is warm.';
        const wrapped = wrapRepairOutcomes({
          slices: [
            preparedPairAt({
              sliceIndex: 0,
              incumbentText,
            },),
            preparedPairAt({
              sliceIndex: 1,
              incumbentText,
            },),
          ],
          outcomes: [
            outcomeOf({
              sliceIndex: 0,
              repairedText: `${incumbentText}\n`,
              changed: true,
            },),
            outcomeOf({
              sliceIndex: 1,
              repairedText: 'The cat wakes. Sun is warm.',
              changed: true,
            },),
          ],
          lineStructuredSlices: new Set([
            0,
            1,
          ],),
          l,
        },);

        expect(wrapped[0]?.repairedText,).toBe(incumbentText,);
        expect(wrapped[0]?.changed,).toBe(false,);
        expect(wrapped[1]?.repairedText,).toBe('The cat wakes. Sun is warm.',);
        expect(wrapped[1]?.changed,).toBe(true,);
      },
    },),

    it({
      name: 'KEEPS A SLICE THE ARCHIVE NEVER TRANSLATED as a change, since filling an empty '
        + 'passage differs from it however the filling is wrapped',
      fn: async () => {
        const wrapped = wrapRepairOutcomes({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText: '',
          },),],
          outcomes: [outcomeOf({
            sliceIndex: 0,
            repairedText: 'The tabby naps. It wakes.',
            changed: true,
          },),],
          lineStructuredSlices: new Set(),
          l,
        },);

        expect(wrapped[0]?.changed,).toBe(true,);
        expect(wrapped[0]?.repairedText,).toBe('The tabby naps.\nIt wakes.',);
      },
    },),

    it({
      name: 'RETURNS AN EMPTY LEDGER as no outcomes rather than failing, since a lane that '
        + 'produced nothing is an ordinary run',
      fn: async () => {
        expect(wrapRepairOutcomes({
          slices: [],
          outcomes: [],
          lineStructuredSlices: new Set(),
          l,
        },).length,).toBe(0,);
      },
    },),
    it({
      name:
        'LEAVES A LINE-STRUCTURED SLICE EXACTLY AS PRODUCED, though the wrap would otherwise '
          + 'change it. The line rule was handed to this producer and it obeyed; the wrap only ever '
          + 'adds breaks, so running it here would break work every decider had already approved',
      fn: async () => {
        const wrapped = wrapRepairOutcomes({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText: 'The cat sleeps on the sill.',
          },),],
          outcomes: [outcomeOf({
            sliceIndex: 0,
            repairedText: GOVERNED_PRODUCED,
            changed: true,
          },),],
          lineStructuredSlices: new Set([0,],),
          l,
        },);

        expect(wrapped[0]?.repairedText,).toBe(GOVERNED_PRODUCED,);
        expect(wrapped[0]?.changed,).toBe(true,);
      },
    },),
  ],
},);
