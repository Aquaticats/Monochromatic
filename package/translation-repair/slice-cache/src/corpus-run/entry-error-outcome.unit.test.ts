/**
 Tests caught entry errors map to operational tally and scheduler state.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

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
  ReturnTimeNotUtcError,
  tallyCaughtEntry,
  VisualEvidenceInterruptedError,
} from '../../dist/final/node/index.mjs';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

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
          name: 'TELLS THE RUN OF A WRITING BENCH ERROR THAT CARRIES NEURALWATT\'S RETURN, for the pass to park on, '
            + 'and files the entry as it does any bench error; a bench error with no return tells it nothing',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the run was told, in order.
             */
            const told: unknown[] = [];
            /**
             The bench error with the return that would lift it.
             */
            const withReturn = new WritingBenchUnreachableError({
              phase: 'lanes',
              clauses: ['editors 0 of 3 reachable, floor 2',],
              neuralwattReturnsInMs: 301_000,
              neuralwattReturnsAt: '19:15:08 UTC',
            },);
            expect(tallyCaughtEntry({
              entryId: 'Cat',
              error: withReturn,
              durationMs: 5,
              aborted: false,
              endRun: function endRun({ error, },): void {
                told.push(error,);
              },
            },),).toEqual({ kind: 'resumable-failure', },);
            expect(told,).toEqual([withReturn,],);
            expect(tallyCaughtEntry({
              entryId: 'Cat',
              error: new WritingBenchUnreachableError({
                phase: 'lanes',
                clauses: ['editors 0 of 3 reachable, floor 2',],
              },),
              durationMs: 5,
              aborted: false,
              endRun: function endRun({ error, },): void {
                told.push(error,);
              },
            },),).toEqual({ kind: 'resumable-failure', },);
            expect(told,).toEqual([withReturn,],);
            expect(printed.lines.some(function failed(line,): boolean {
              return line.startsWith('TALLY Cat status=ERROR',);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'PRINTS AN ENTRY THE GATE PARKED FOR AS PARKED, never ERROR or INCOMPLETE, with the reason: the hook '
            + 'is told first, the gate is then asked whether it parked, and the entry is still filed as it was',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Whether the gate has parked, which the hook sets as the gate does.
             */
            const gate = { parked: false, };
            /**
             The bench error with the return that would lift it.
             */
            const withReturn = new WritingBenchUnreachableError({
              phase: 'lanes',
              clauses: ['editors 0 of 3 reachable, floor 2',],
              neuralwattReturnsInMs: 301_000,
              neuralwattReturnsAt: '19:15:08 UTC',
            },);
            expect(tallyCaughtEntry({
              entryId: 'Cat',
              error: withReturn,
              durationMs: 5,
              aborted: true,
              endRun: function endRun(): void {
                gate.parked = true;
              },
              parked: function parked(): boolean {
                return gate.parked;
              },
            },),).toEqual({ kind: 'resumable-failure', },);
            /**
             The one tally line printed.
             */
            const [line,] = printed.lines.filter(function tally(text,): boolean {
              return text.startsWith('TALLY ',);
            },);
            expect(line,).toContain('TALLY Cat status=PARKED ',);
            expect(line,).toContain('NeuralWatt\'s energy returns at 19:15:08 UTC',);
            expect(line,).not.toContain('status=ERROR',);

            // A gate that left the error alone (the return is past the pass) leaves the entry an ERROR.
            gate.parked = false;
            expect(tallyCaughtEntry({
              entryId: 'Dog',
              error: withReturn,
              durationMs: 5,
              aborted: false,
              endRun: function endRun(): void {},
              parked: function parked(): boolean {
                return gate.parked;
              },
            },),).toEqual({ kind: 'resumable-failure', },);
            expect(printed.lines.some(function failed(text,): boolean {
              return text.startsWith('TALLY Dog status=ERROR',);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'KEEPS THE RETURN ON THE PARKED LINE WHATEVER THE REASON COSTS: the lanes with editors, refiners and '
            + 'translators all below their floor make a reason longer than the cap, and the status, the distance and '
            + 'the time of day (one token, no space) come before it so the cap trims only the reason',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
            /**
             The bench error of the real clause shapes, three benches below their floor and the spent day stated.
             */
            const lanes = new WritingBenchUnreachableError({
              phase: 'lanes',
              clauses: [
                'editors 0 of 3 reachable, floor 2',
                'refiners 0 of 3 reachable, floor 2',
                'translators 0 of 8 reachable, floor 2',
                'NeuralWatt reads dry and its day starts again at 19:15:08 UTC',
              ],
              neuralwattReturnsInMs: 301_000,
              neuralwattReturnsAt: '19:15:08 UTC',
            },);
            tallyCaughtEntry({
              entryId: 'Cat',
              error: lanes,
              durationMs: 5,
              aborted: true,
              endRun: function endRun(): void {},
              parked: function parked(): boolean {
                return true;
              },
            },);
            /**
             The one tally line printed.
             */
            const [line,] = printed.lines.filter(function tally(text,): boolean {
              return text.startsWith('TALLY ',);
            },);
            expect(line,).toContain('TALLY Cat status=PARKED returnsInMs=301000 returnsAt=19:15:08Z ms=5 ',);
            expect(line?.indexOf('returnsAt=',) ?? (-1),).toBeLessThan(line?.indexOf(' error=',) ?? (-1),);
            expect(line?.indexOf('returnsAt=',) ?? (-1),).toBeGreaterThan(-1,);
          },
        },),

        it({
          name: 'THROWS ON A RETURN TIME THAT IS NOT IN UTC, naming the text it got and printing no line, since the '
            + 'only writer of the time gives it with the UTC zone and any other text is a break of that rule',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
            /**
             The bench error whose time carries another zone than the one its writer gives.
             */
            const lanes = new WritingBenchUnreachableError({
              phase: 'lanes',
              clauses: ['editors 0 of 3 reachable, floor 2',],
              neuralwattReturnsInMs: 301_000,
              neuralwattReturnsAt: '19:15:08 CEST',
            },);
            /**
             Tallies the entry as the pass does once it has parked for the failure.
             */
            function tallyParked(): void {
              tallyCaughtEntry({
                entryId: 'Cat',
                error: lanes,
                durationMs: 5,
                aborted: true,
                endRun: function endRun(): void {},
                parked: function parked(): boolean {
                  return true;
                },
              },);
            }
            expect(tallyParked,).toThrow(ReturnTimeNotUtcError,);
            expect(tallyParked,).toThrow('"19:15:08 CEST"',);
            expect(printed.lines.filter(function tally(text,): boolean {
              return text.startsWith('TALLY ',);
            },),).toEqual([],);
          },
        },),

        it({
          name: 'PRINTS THE ALL-DRY REFUSAL THE GATE PARKED FOR AS PARKED too, with the same return fields and not '
            + 'INCOMPLETE, and prints a refusal the gate did not park for as INCOMPLETE',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
            /**
             The router's refusal naming a return.
             */
            const dry = new EveryProviderDryError({
              measured: 'every meter dry',
              neuralwatt: { kind: 'at', msOfDay: 69_308_000, },
              neuralwattDryness: { kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', returnsInMs: 301_000, },
              neuralwattReturnsInMs: 301_000,
            },);
            expect(tallyCaughtEntry({
              entryId: 'Cat',
              error: dry,
              durationMs: 5,
              aborted: true,
              endRun: function endRun(): void {},
              parked: function parked(): boolean {
                return true;
              },
            },),).toEqual({ kind: 'stopped', },);
            expect(tallyCaughtEntry({
              entryId: 'Dog',
              error: dry,
              durationMs: 5,
              aborted: true,
              endRun: function endRun(): void {},
              parked: function parked(): boolean {
                return false;
              },
            },),).toEqual({ kind: 'stopped', },);
            expect(printed.lines.some(function parkedLine(text,): boolean {
              return text.startsWith('TALLY Cat status=PARKED returnsInMs=301000 returnsAt=19:15:08Z ms=5 ',);
            },),).toBe(true,);
            expect(printed.lines.some(function incompleteLine(text,): boolean {
              return text.startsWith('TALLY Dog status=INCOMPLETE ms=5 ',);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'ASKS THE GATE WHETHER IT PARKED FOR THIS FAILURE, naming the failure, so a failure the gate did not '
            + 'park for is never printed as PARKED',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
            /**
             What the gate was asked about.
             */
            const asked: unknown[] = [];
            /**
             The failure that has nothing to do with the park.
             */
            const reset = new Error('connection reset',);
            tallyCaughtEntry({
              entryId: 'Cat',
              error: reset,
              durationMs: 5,
              aborted: false,
              endRun: function endRun(): void {},
              parked: function parked({ error, },): boolean {
                asked.push(error,);
                return error !== reset;
              },
            },);
            expect(asked,).toEqual([reset,],);
            expect(printed.lines.some(function failed(text,): boolean {
              return text.startsWith('TALLY Cat status=ERROR ',);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'FILES THE REFUSAL ALL THE SAME when the run gave no hook to tell',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);
            expect(tallyCaughtEntry({
              entryId: 'Cat',
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
