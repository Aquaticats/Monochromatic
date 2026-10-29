/**
 Tests for wrapping what the translate lane produced.

 WHAT THESE PIN is the same pair of properties `repair-wrap.unit.test.ts`
 pins, on the other lane: only wording this lane PRODUCED is wrapped, and the
 changed flag is re-derived from the wrapped text rather than carried forward.

 The retention case matters more here than on the repair side, because this
 lane stands on the archive by two separate routes: the judges preferring the
 incumbent, and no translator answering at all. Both carry the archive's own
 wording in `outputText`, and wrapping either would report a change nobody
 decided on and contradict `sliceRecordAgrees`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { wrapTranslateRecords, } from '../dist/final/node/index.mjs';
import { preparedPairAt, } from '@monochromatic-dev/translation-repair-slice-reading/ts/prepared-pair-at.test-fixture.ts';

/**
 Logger these hand to the lane, whose output is not what is under test.
 */
const l = tagged({ tag: 'translate-wrap-test', },);

/**
 Builds one settled translate record.

 MINIMAL BY DESIGN: the wrap reads three fields and carries the rest through
 untouched, so a fixture carrying the whole contract would test the spread
 rather than the decision.

 @param sliceIndex - slice index

 @param outputText - wording this lane settled on

 @param changed - whether it claims to differ from the archive

 @returns Record shaped as the lane settles one

 @example
 ```ts
 const record = recordOf({ sliceIndex: 0, outputText: 'It naps.', changed: true, },);
 ```
 */
function recordOf(
  {
    sliceIndex,
    outputText,
    changed,
  }: {
    readonly sliceIndex: number;
    readonly outputText: string;
    readonly changed: boolean;
  },
): Parameters<typeof wrapTranslateRecords>[0]['settled'][number] {
  return {
    kind: 'translate-slice',
    schemaVersion: 2,
    sliceIndex,
    outputText,
    changed,
    disposition: 'shipped',
  } as unknown as Parameters<typeof wrapTranslateRecords>[0]['settled'][number];
}

/**
 One passage a governed producer returned, carrying sentence boundaries the
 wrap would break at if it were allowed to run.
 */
const GOVERNED_PRODUCED = 'The cat wakes. Sun is warm. She counts birds.';
await describe({
  name: wrapTranslateRecords.name,
  children: [
    it({
      name: 'WRAPS WORDING THE LANE PRODUCED, which is the reason this exists: this lane writes '
        + 'each slice fresh from its source and a model returns that as one line',
      fn: async () => {
        const wrapped = wrapTranslateRecords({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText: 'The cat sleeps on the sill.',
          },),],
          settled: [recordOf({
            sliceIndex: 0,
            outputText: 'The tabby naps on the sill. It wakes at dusk.',
            changed: true,
          },),],
          lineStructuredSlices: new Set(),
          l,
        },);

        expect(wrapped[0]?.outputText,).toBe('The tabby naps on the sill.\nIt wakes at dusk.',);
        expect(wrapped[0]?.changed,).toBe(true,);
      },
    },),

    it({
      name: 'LEAVES A RECORD STANDING ON THE ARCHIVE BYTE-IDENTICAL, whether the judges preferred '
        + 'the incumbent or no translator answered: both carry the archive’s own wording, and '
        + 'wrapping either would report a change nobody decided on',
      fn: async () => {
        /**
         Archive wording that the rule WOULD break, were it asked to.
         */
        const incumbentText = 'The cat sleeps on the sill. It wakes at dusk.';

        const wrapped = wrapTranslateRecords({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText,
          },),],
          settled: [recordOf({
            sliceIndex: 0,
            outputText: incumbentText,
            changed: false,
          },),],
          lineStructuredSlices: new Set(),
          l,
        },);

        expect(wrapped[0]?.outputText,).toBe(incumbentText,);
        expect(wrapped[0]?.changed,).toBe(false,);
      },
    },),

    it({
      name: 'DEMOTES TO A RETENTION when wrapping is all that separated the wording from the '
        + 'archive, since a record still claiming a change there contradicts sliceRecordAgrees',
      fn: async () => {
        /**
         Archive wording, already written as the rule would write it.
         */
        const incumbentText = 'It naps.\nIt wakes.';

        const wrapped = wrapTranslateRecords({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText,
          },),],
          settled: [recordOf({
            sliceIndex: 0,
            outputText: 'It naps. It wakes.',
            changed: true,
          },),],
          lineStructuredSlices: new Set(),
          l,
        },);

        expect(wrapped[0]?.outputText,).toBe(incumbentText,);
        expect(wrapped[0]?.changed,).toBe(false,);
      },
    },),

    it({
      name: 'DEMOTES A RECORD WHOSE WORDING IS THE ARCHIVE\'S WITH ITS SOFT LINE BREAKS ELSEWHERE, even when '
        + 'the wrap leaves it as it is, and ships the archive\'s own bytes: the site renders a soft break as a '
        + 'space, so the page would not change and the record would report a change nobody made (ledger B26)',
      fn: async () => {
        /**
         Archive wording with a soft break inside its paragraph.
         */
        const incumbentText = 'The cat naps on the mat\nall afternoon.';
        const wrapped = wrapTranslateRecords({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText,
          },),],
          settled: [recordOf({
            sliceIndex: 0,
            outputText: 'The cat naps on the mat all afternoon.',
            changed: true,
          },),],
          lineStructuredSlices: new Set(),
          l,
        },);

        expect(wrapped[0]?.outputText,).toBe(incumbentText,);
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
        const wrapped = wrapTranslateRecords({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText,
          },),],
          settled: [recordOf({
            sliceIndex: 0,
            outputText: '> The cat naps.\n> The dog waits.',
            changed: true,
          },),],
          lineStructuredSlices: new Set(),
          l,
        },);

        expect(wrapped[0]?.outputText,).toBe(incumbentText,);
        expect(wrapped[0]?.changed,).toBe(false,);
      },
    },),

    it({
      name: 'DEMOTES A LINE-STRUCTURED RECORD that differs from the archive only after its last character, '
        + 'and KEEPS one whose lines differ, since there the line breaks are the producer\'s work (ledger B26)',
      fn: async () => {
        /**
         Archive wording, one line per original line.
         */
        const incumbentText = 'The cat wakes.\nSun is warm.';
        const wrapped = wrapTranslateRecords({
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
          settled: [
            recordOf({
              sliceIndex: 0,
              outputText: `${incumbentText}\n`,
              changed: true,
            },),
            recordOf({
              sliceIndex: 1,
              outputText: 'The cat wakes. Sun is warm.',
              changed: true,
            },),
          ],
          lineStructuredSlices: new Set([
            0,
            1,
          ],),
          l,
        },);

        expect(wrapped[0]?.outputText,).toBe(incumbentText,);
        expect(wrapped[0]?.changed,).toBe(false,);
        expect(wrapped[1]?.outputText,).toBe('The cat wakes. Sun is warm.',);
        expect(wrapped[1]?.changed,).toBe(true,);
      },
    },),

    it({
      name: 'WRAPS EVERY CHANGED RECORD rather than the first, since a document settles many '
        + 'slices and a loop that stopped early would ship a mixture nobody could account for',
      fn: async () => {
        const wrapped = wrapTranslateRecords({
          slices: [
            preparedPairAt({
              sliceIndex: 0,
              incumbentText: 'One.',
            },),
            preparedPairAt({
              sliceIndex: 1,
              incumbentText: 'Two.',
            },),
          ],
          settled: [
            recordOf({
              sliceIndex: 0,
              outputText: 'A tabby naps. A tabby wakes.',
              changed: true,
            },),
            recordOf({
              sliceIndex: 1,
              outputText: 'A bowl fills. A bowl empties.',
              changed: true,
            },),
          ],
          lineStructuredSlices: new Set(),
          l,
        },);

        expect(wrapped[0]?.outputText,).toBe('A tabby naps.\nA tabby wakes.',);
        expect(wrapped[1]?.outputText,).toBe('A bowl fills.\nA bowl empties.',);
      },
    },),

    it({
      name: 'RETURNS AN EMPTY LEDGER as no records rather than failing, since a lane that settled '
        + 'nothing is an ordinary run',
      fn: async () => {
        expect(wrapTranslateRecords({
          slices: [],
          settled: [],
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
        const wrapped = wrapTranslateRecords({
          slices: [preparedPairAt({
            sliceIndex: 0,
            incumbentText: 'The cat sleeps on the sill.',
          },),],
          settled: [recordOf({
            sliceIndex: 0,
            outputText: GOVERNED_PRODUCED,
            changed: true,
          },),],
          lineStructuredSlices: new Set([0,],),
          l,
        },);

        expect(wrapped[0]?.outputText,).toBe(GOVERNED_PRODUCED,);
        expect(wrapped[0]?.changed,).toBe(true,);
      },
    },),
  ],
},);
