/**
 Tests caught entry errors map to operational tally and scheduler state.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { EveryProviderDryError, } from '@monochromatic-dev/translation-repair-provider';
import { WritingBenchUnreachableError, } from '@monochromatic-dev/translation-repair-run';
import { modelDarkRefusal, } from '@monochromatic-dev/translation-repair-provider/ts/model-dark-refusal.test-fixture.ts';
import { TranslationRepairInterruptedError, } from '@monochromatic-dev/translation-repair-translate-stage';
import { PromptPayloadStoreError, } from '@monochromatic-dev/translation-repair-stage';
import {
  NaturalnessCompletenessError,
  NaturalnessRepairInterruptedError,
} from '@monochromatic-dev/translation-repair-adjudication';
import {
  DroppedDestinationError,
  FrontMatterCompletenessError,
  PublishedPageDisagreesError,
  UnansweredContestSliceError,
  UnparseablePageError,
} from '@monochromatic-dev/translation-repair-page-check';
import {
  CollapsedHeadingError,
  ContributorCompletenessError,
  SliceSpliceError,
} from '@monochromatic-dev/translation-repair-page-pass';
import {
  entryErrorOutcome,
  tallyCaughtEntry,
  VisualEvidenceInterruptedError,
} from '../../dist/final/node/index.mjs';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 Instant the injected clock reads, so no case waits on the real one.
 */
const PRINTED_AT = new Date('2026-10-22T08:10:11.123Z',);

/**
 Attempt the cases print the line for: the entry's second, the one that began under a fresh cap.
 */
const SECOND_ATTEMPT = 2;

/**
 The clock time and attempt every line a case prints carries, straight after the status.
 */
const STAMP = 'at=2026-10-22T08:10:11.123Z attempt=2';

/**
 Clock that reads the same instant every time it is asked.

 @returns The instant the line is printed at

 @example
 ```ts
 tallyCaughtEntry({ entryId: 'Cat', error, durationMs: 5, aborted: false, attempt: SECOND_ATTEMPT, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

await describe({
  name: '',
  children: [
    describe({
      name: entryErrorOutcome.name,
      children: [
        ...([
          new ContributorCompletenessError({ entryId: 'Cat', droppedCount: 1, }),
          new DroppedDestinationError({
            entryId: 'Cat',
            droppedCount: 1,
            traces: [{
              sourceSlices: [2,],
              archiveSlices: [],
              shippedSlices: [],
            },],
          }),
          new FrontMatterCompletenessError({ entryId: 'Cat', reason: 'missing-slice', }),
          new NaturalnessRepairInterruptedError({ reason: 'contributor-structure', }),
          new NaturalnessCompletenessError({ sliceIndex: 1, }),
          new PromptPayloadStoreError({
            promptDigest: 'fixture-digest',
            operation: 'read',
            reason: 'the record is not JSON',
          },),
          new TranslationRepairInterruptedError({
            reason: 'provider-unavailable',
            findings: [],
          },),
          new VisualEvidenceInterruptedError({ unavailableCount: 1, }),
        // Ledger A7: deterministic page and artifact refusals, which a retry
        // resuming the same cached slices reproduces exactly.
          new CollapsedHeadingError({ entryId: 'Cat', sourceDistinct: 2, pageDistinct: 1, }),
          new UnparseablePageError({ entryId: 'Cat', refusal: 'at 3:12 (mdx-jsx)', }),
          new PublishedPageDisagreesError({
            entryId: 'Cat',
            disagreement: {
              kind: 'weight-off',
              actual: 10,
              expected: 12,
              exact: true,
            },
          },),
          new UnansweredContestSliceError({ sliceIndex: 3, },),
          new SliceSpliceError({ message: 'two replacements name one slice', },),
        // Every provider dry ends the run at the owner's instruction: a whole-entry retry meets the same dry router.
          new EveryProviderDryError({ measured: 'meters read every provider dry', },),
        ] as const).map(function stoppedError(error,) {
          return it({
            name: `MAPS ${error.name} to INCOMPLETE stopped work`,
            fn: async () => {
              expect(entryErrorOutcome({ error, },),).toEqual({
                status: 'INCOMPLETE',
                outcome: { kind: 'stopped', },
              },);
            },
          },);
        },),

        it({
          name: 'MAPS A MODEL-DARK REFUSAL to resumable ERROR like any operational failure, since a model whose own '
            + 'providers are dry costs one seat its voice and does not stop the run',
          fn: async () => {
            expect(entryErrorOutcome({ error: modelDarkRefusal({ modelId: 'glm-5.3', },), },),).toEqual({
              status: 'ERROR',
              outcome: { kind: 'resumable-failure', },
            },);
          },
        },),

        it({
          name: 'MAPS ORDINARY OPERATIONAL ERROR to resumable ERROR',
          fn: async () => {
            expect(entryErrorOutcome({ error: new Error('transport failed',), },),).toEqual({
              status: 'ERROR',
              outcome: { kind: 'resumable-failure', },
            },);
          },
        },),
      ],
    },),

    describe({
      name: tallyCaughtEntry.name,
      children: [
        it({
          name: 'TELLS THE RUN OF THE REFUSAL THAT ENDED THE ENTRY, once and before the entry is filed, prints the entry as '
            + 'INCOMPLETE, and tells it nothing for a failure that does not end the run',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
          /**
           What the run was told, in order.
           */
            const told: unknown[] = [];
          /**
           The refusal the router raised for the entry.
           */
            const dry = new EveryProviderDryError({ measured: 'every meter dry', },);
            expect(tallyCaughtEntry({
              entryId: 'Cat',
              attempt: SECOND_ATTEMPT,
              wallClock: fixedWallClock,
              error: dry,
              durationMs: 5,
              aborted: true,
              endRun: function endRun({ error, },): void {
                told.push(error,);
              },
            },),).toEqual({ kind: 'stopped', },);
            expect(told,).toEqual([dry,],);
            expect(printed.lines.some(function incomplete(line,): boolean {
              return line.startsWith('TALLY Cat status=INCOMPLETE',);
            },),).toBe(true,);

            expect(tallyCaughtEntry({
              entryId: 'Cat',
              attempt: SECOND_ATTEMPT,
              wallClock: fixedWallClock,
              error: new Error('transport failed',),
              durationMs: 5,
              aborted: false,
              endRun: function endRun({ error, },): void {
                told.push(error,);
              },
            },),).toEqual({ kind: 'resumable-failure', },);
            expect(told,).toEqual([dry,],);
          },
        },),

        it({
          name: 'DOES NOT TELL THE RUN OF A WRITING BENCH ERROR, whatever its clauses say of NeuralWatt: the entry is '
            + 'filed as a resumable failure and never ends the run, so a bench error that states NeuralWatt\'s day '
            + 'start in its clauses prints ERROR and tells the run nothing',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run was told, in order.
             */
            const told: unknown[] = [];
            expect(tallyCaughtEntry({
              entryId: 'Cat',
              attempt: SECOND_ATTEMPT,
              wallClock: fixedWallClock,
              error: new WritingBenchUnreachableError({
                phase: 'lanes',
                clauses: [
                  'editors 0 of 3 reachable, floor 2',
                  'NeuralWatt reads dry and its day starts again at 19:15:08 UTC (in 5 min 1 s)',
                ],
              },),
              durationMs: 5,
              aborted: false,
              endRun: function endRun({ error, },): void {
                told.push(error,);
              },
            },),).toEqual({ kind: 'resumable-failure', },);
            expect(told,).toEqual([],);
            expect(printed.lines.some(function failed(line,): boolean {
              return line.startsWith('TALLY Cat status=ERROR',) && line.includes('19:15:08 UTC',);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'PRINTS THE ALL-DRY REFUSAL AS INCOMPLETE whatever return it names, and never as PARKED: the pass '
            + 'stops, so the entry is filed as stopped work with the day start in its reason',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
            /**
             The router's refusal naming a return.
             */
            const dry = new EveryProviderDryError({
              measured: 'every meter dry',
              neuralwatt: { kind: 'at', msOfDay: 69_308_000, },
              neuralwattDryness: { kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', returnsInMs: 301_000, },
            },);
            expect(tallyCaughtEntry({
              entryId: 'Cat',
              attempt: SECOND_ATTEMPT,
              wallClock: fixedWallClock,
              error: dry,
              durationMs: 5,
              aborted: true,
              endRun: function endRun(): void {},
            },),).toEqual({ kind: 'stopped', },);
            expect(printed.lines.some(function incompleteLine(text,): boolean {
              return text.startsWith(`TALLY Cat status=INCOMPLETE ${STAMP} ms=5 `,);
            },),).toBe(true,);
            expect(printed.lines.some(function parkedLine(text,): boolean {
              return text.includes('PARKED',);
            },),).toBe(false,);
          },
        },),

        it({
          name: 'STAMPS THE ERROR TALLY OF A CUT ENTRY with the clock time and the attempt, before the capped error text, '
            + 'so a reader placing a fired hard cap in a time window finds which attempt it ended (2026-10-22, XingZ60: '
            + 'attempt 2 began under a fresh cap and nothing on the line said so)',
          fn: async (ctx,) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
            tallyCaughtEntry({
              entryId: 'XingZ60',
              error: new Error('Timeout: XingZ60 exceeded its 25200000ms deadline',),
              durationMs: 25_200_743,
              aborted: true,
              attempt: 1,
              wallClock: fixedWallClock,
            },);
            expect(printed.lines.length,).toBe(1,);
            expect(
              printed.lines[0]?.startsWith(
                'TALLY XingZ60 status=ERROR at=2026-10-22T08:10:11.123Z attempt=1 ms=25200743 aborted=true error=',
              ),
            ).toBe(true,);
          },
        },),

        it({
          name: 'STAMPS EACH INTERRUPTED LINE with the clock time and the attempt, after the entry and before the finding',
          fn: async (ctx,) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
            tallyCaughtEntry({
              entryId: 'Cat',
              error: new TranslationRepairInterruptedError({
                reason: 'carried-evidence-lost',
                findings: [
                  'slice 3 lost its carried reading',
                  'slice 5 lost its carried reading',
                ],
              },),
              durationMs: 5,
              aborted: false,
              attempt: SECOND_ATTEMPT,
              wallClock: fixedWallClock,
            },);
            expect(printed.lines.filter(function interrupted(text,): boolean {
              return text.startsWith('INTERRUPTED ',);
            },),).toEqual([
              `INTERRUPTED Cat ${STAMP}: slice 3 lost its carried reading`,
              `INTERRUPTED Cat ${STAMP}: slice 5 lost its carried reading`,
            ],);
          },
        },),

        it({
          name: 'FILES THE REFUSAL ALL THE SAME when the run gave no hook to tell',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
            expect(tallyCaughtEntry({
              entryId: 'Cat',
              attempt: SECOND_ATTEMPT,
              wallClock: fixedWallClock,
              error: new EveryProviderDryError({ measured: 'every meter dry', },),
              durationMs: 5,
              aborted: false,
            },),).toEqual({ kind: 'stopped', },);
            expect(printed.lines.length,).toBe(1,);
          },
        },),
      ],
    },),
  ],
},);
