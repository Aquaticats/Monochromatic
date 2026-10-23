/**
 Tests for the seat readings a phase and a chunk take past a named hold.

 THE THIRTEENTH CLASS IN BOTH ITS FACES. Before a phase (the third hakureico
 pass of 2026-09-07, translate lane and consolidation on nobody under a
 538 s hold) and inside one (the fourth pass, a 923 s hold two minutes into
 consolidation, slice 5 stopping the entry). The phase reading waits when a
 bench it leans on cannot reach quorum; the chunk reading does the same and
 costs nothing while nothing is held.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  type BudgetView,
  EveryProviderDryError,
  type MeterState,
  type NeuralWattReturn,
} from '@monochromatic-dev/translation-repair-provider';
import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  mapOverlapped,
  rosterQuorumSize,
  RUN_SEATS,
} from '@monochromatic-dev/translation-repair-stage';
import {
  reachableSeats,
  readJudgeSeats,
  readPhaseSeats,
  RUN_TRANSLATORS,
  RUN_WIDE_SEATS,
  WritingBenchUnreachableError,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import {
  capturingLoggerPair,
  levelCapturingLogger,
  warningRecordingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { quotingFailure, } from '@monochromatic-dev/translation-repair-test-fixture/ts/quoting-failure.test-fixture.ts';
import {
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { withFreshRunSeats, } from '@monochromatic-dev/translation-repair-stage/ts/corpus-run/fresh-run-seats.test-fixture.ts';

//region Seat reading tests

/**
 The Hyper-slow judge.
 */
const QWEN = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 The seat only Hyper serves.
 */
const GLM = SEAT_HYPER_ONLY;

/**
 Every provider wet.
 */
const ALL_WET: BudgetView = {
  neuralwatt: false,
  synthetic: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  hyper: false,
  openrouter: false,
};

/**
 Every provider dry.
 */
const ALL_DRY: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: true,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  hyper: true,
  openrouter: true,
};

/**
 Synthetic alone dry.
 */
const SYNTHETIC_DRY: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  hyper: false,
  openrouter: false,
};

/**
 NeuralWatt and Bedrock wet, Synthetic and Hyper dry, OpenRouter wet: the
 view zheermao20 read after Synthetic ran dry at 02:37 UTC, under which
 `glm-5.3` (Hyper alone) and `hf:Qwen/Qwen3.8-27B` (Synthetic and Hyper) are
 seated and no wet provider serves either.
 */
const SYNTHETIC_AND_HYPER_DRY: BudgetView = {
  neuralwatt: false,
  synthetic: true,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  hyper: true,
  openrouter: false,
};

/**
 No provider held out.
 */
const NO_HOLDS = {
  neuralwatt: 0,
  synthetic: 0,
  bedrock: 0,
  deepseek: 0,
  xiaomi: 0,
  anthropic: 0,
  hyper: 0,
  openrouter: 0,
};

/**
 The third hakureico pass at 22:00 UTC: Synthetic's week spent, Hyper held
 out by its daily limit, OpenRouter at 0.01 USD, Bedrock alone wet.
 */
const BEDROCK_ALONE: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: false,
  deepseek: true,
  xiaomi: true,
  anthropic: true,
  hyper: true,
  openrouter: true,
};

/**
 Logger the readings write to.
 */
const l = tagged({ tag: 'run-seats-read-test', },);

/**
 What every meter said when each one answered wet.
 */
const ALL_METERS_WET: Readonly<Record<keyof BudgetView, MeterState>> = {
  neuralwatt: 'wet',
  synthetic: 'wet',
  bedrock: 'wet',
  deepseek: 'wet',
  xiaomi: 'wet',
  anthropic: 'wet',
  hyper: 'wet',
  openrouter: 'wet',
};

/**
 Builds a client whose dryness view, meter states and holds answer as scripted.

 @param providerDryness - scripted view

 @param providerMeterStates - scripted meter states, every meter wet when absent

 @param providerHolds - scripted holds, none when absent

 @param writingProviders - providers that serve a writing-bench seat, the run's roster when absent

 @returns Client with only the seat reader's surface

 @example
 ```ts
 const client = viewClient({ providerDryness: async () => ALL_WET, },);
 ```
 */
function viewClient(
  {
    providerDryness,
    providerMeterStates = async () => ALL_METERS_WET,
    providerHolds = () => NO_HOLDS,
    neuralwattReturn,
    writingProviders,
  }: {
    readonly providerDryness: () => Promise<BudgetView>;
    readonly providerMeterStates?: () => Promise<Readonly<Record<keyof BudgetView, MeterState>>>;
    readonly providerHolds?: () => typeof NO_HOLDS;
    readonly neuralwattReturn?: () => NeuralWattReturn;
    readonly writingProviders?: Readonly<Record<keyof BudgetView, boolean>>;
  },
) {
  return {
    providerDryness,
    providerMeterStates,
    providerHolds,
    // Conditional spread keeps the record absent on a client that carries none, which reads the roster's.
    ...((writingProviders === undefined) ? {} : { writingProviders, }),
    // Conditional spread keeps the accessor absent on a client that carries none.
    ...((neuralwattReturn === undefined) ? {} : { neuralwattReturn, }),
  };
}

/**
 Holds that end once they have been waited out: the scripted holds on the first ask, none on any later one.

 @param first - holds the first ask reads

 @returns Accessor shaped as the run client's holds

 @example
 ```ts
 const providerHolds = holdsThatEnd({ first: { ...NO_HOLDS, hyper: 40, }, },);
 ```
 */
function holdsThatEnd(
  { first, }: { readonly first: typeof NO_HOLDS; },
): () => typeof NO_HOLDS {
  /**
   Asks made so far.
   */
  const asks = { count: 0, };
  return function holdsNow(): typeof NO_HOLDS {
    asks.count += 1;
    return (asks.count === 1) ? first : NO_HOLDS;
  };
}

/**
 Dryness views handed out in order, the last one repeated, counting reads.

 @param views - views in the order they are read

 @returns Reader and the count of reads so far

 @example
 ```ts
 const script = scriptedViews({ views: [BEDROCK_ALONE, ALL_WET,], },);
 ```
 */
function scriptedViews(
  { views, }: { readonly views: readonly BudgetView[]; },
) {
  const counter = { reads: 0, };
  return {
    counter,
    read: async (): Promise<BudgetView> => {
      const view = views[Math.min(counter.reads, views.length - 1,)];
      counter.reads += 1;
      if (view === undefined)
        throw new Error('scripted views must name at least one view',);
      return view;
    },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readJudgeSeats.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the dryness view off the run client, and SEATS the full bench when the view cannot be '
            + 'read, since an unreadable view is not evidence of dryness',
          fn: async () => {
            const dry = await readJudgeSeats({
              client: viewClient({ providerDryness: async () => SYNTHETIC_DRY, },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l,
            },);
            expect(dry.dry,).toEqual(SYNTHETIC_DRY,);
            expect(dry.wideSeats.includes(QWEN,),).toBe(false,);

            const wet = await readJudgeSeats({
              client: viewClient({ providerDryness: async () => ALL_WET, },),
              phase: 'lane contest',
              signal: new AbortController().signal,
              l,
            },);
            expect(wet.dry,).toEqual(ALL_WET,);
            expect(wet.wideSeats.includes(QWEN,),).toBe(true,);

            const unread = await readJudgeSeats({
              client: viewClient({
                providerDryness: async () => {
                  throw new Error('meters offline',);
                },
              },),
              phase: 'consolidation',
              signal: new AbortController().signal,
              l,
            },);
            expect(unread.dry,).toEqual(ALL_WET,);
            expect(unread.wideSeats.includes(QWEN,),).toBe(true,);
          },
        },),
        it({
          name: 'NAMES IN THE JUDGE SEATS LINE each seat the run has withheld for throwing on every ask, with its '
            + 'counts, and names none while a seat that threw some asks and answered others stays seated',
          fn: async () => {
            using _fresh = withFreshRunSeats();
            for (let ask = 0; ask < 4; ask += 1)
              RUN_SEATS.record({ modelId: QWEN, outcome: 'threw', },);
            /** Lines the reading writes at the info level. */
            const lines: string[] = [];
            await readJudgeSeats({
              client: viewClient({ providerDryness: async () => ALL_WET, },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines, },),
            },);
            expect(lines,).toEqual([
              'info JUDGE SEATS phase=lanes neuralwatt=wet synthetic=wet bedrock=wet deepseek=wet xiaomi=wet anthropic=wet '
                + 'hyper=wet openrouter=wet wide=8 select=9 '
                + 'late=7 slate=7 checkers=3 translators=8 readers=6 seeing=6 writers=9 roster=10 withheld=none unserved=none '
                + 'threw-withheld=hf:Qwen/Qwen3.8-27B (threw 4 of 4 asks) waited=0ms',
            ],);

            /** Lines of a second reading, for a seat that answered on some asks. */
            const mixedLines: string[] = [];
            RUN_SEATS.reset();
            for (let ask = 0; ask < 3; ask += 1)
              RUN_SEATS.record({ modelId: QWEN, outcome: 'threw', },);
            RUN_SEATS.record({ modelId: QWEN, outcome: 'usable', },);
            await readJudgeSeats({
              client: viewClient({ providerDryness: async () => ALL_WET, },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines: mixedLines, },),
            },);
            expect(mixedLines,).toEqual([
              'info JUDGE SEATS phase=lanes neuralwatt=wet synthetic=wet bedrock=wet deepseek=wet xiaomi=wet anthropic=wet '
                + 'hyper=wet openrouter=wet wide=8 select=9 '
                + 'late=7 slate=7 checkers=3 translators=8 readers=6 seeing=6 writers=9 roster=10 withheld=none unserved=none '
                + 'threw-withheld=none waited=0ms',
            ],);
          },
        },),
        it({
          name: 'NAMES IN THE JUDGE SEATS LINE a meter that could not be read as unreadable and treated as '
            + 'spendable, never as wet, while a meter that answered wet still reads wet and a provider routed '
            + 'dry reads dry whatever its meter said',
          fn: async () => {
            /** Lines the reading writes at the info level. */
            const lines: string[] = [];
            await readJudgeSeats({
              client: viewClient({
                providerDryness: async () => ({
                  neuralwatt: true,
                  synthetic: false,
                  bedrock: true,
                  deepseek: false,
                  xiaomi: false,
                  anthropic: false,
                  hyper: false,
                  openrouter: false,
                }),
                providerMeterStates: async () => ({
                  neuralwatt: 'dry',
                  synthetic: 'unreadable',
                  bedrock: 'unreadable',
                  deepseek: 'wet',
                  xiaomi: 'wet',
                  anthropic: 'wet',
                  hyper: 'wet',
                  openrouter: 'wet',
                }),
              },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines, },),
            },);
            expect(lines.length,).toBe(1,);
            expect(lines[0],).toContain(
              'info JUDGE SEATS phase=lanes neuralwatt=dry synthetic=unreadable-treated-as-spendable bedrock=dry '
                + 'deepseek=wet xiaomi=wet anthropic=wet hyper=wet openrouter=wet ',
            );
          },
        },),
        it({
          name: 'NAMES EVERY METER AS UNREADABLE when the states cannot be read either, since a view that could '
            + 'not be read says nothing of any budget',
          fn: async () => {
            /** Lines the reading writes at the info level. */
            const lines: string[] = [];
            await readJudgeSeats({
              client: viewClient({
                providerDryness: async () => ALL_WET,
                providerMeterStates: async () => {
                  throw new Error('meters offline',);
                },
              },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines, },),
            },);
            expect(lines[0],).toBe(
              'warn judge seats: the meter states could not be read (refused by Error); naming every meter unreadable',
            );
            expect(lines.length,).toBe(2,);
            expect(lines[1],).toContain(
              'info JUDGE SEATS phase=lanes neuralwatt=unreadable-treated-as-spendable '
                + 'synthetic=unreadable-treated-as-spendable '
                + 'bedrock=unreadable-treated-as-spendable deepseek=unreadable-treated-as-spendable '
                + 'xiaomi=unreadable-treated-as-spendable anthropic=unreadable-treated-as-spendable hyper=unreadable-treated-as-spendable '
                + 'openrouter=unreadable-treated-as-spendable ',
            );
          },
        },),
        it({
          name: 'WARNS OF AN UNREADABLE VIEW BY THE FAILURE\'S CLASS and not by its message, which a meter '
            + 'request with an unsendable header quotes',
          fn: async () => {
            const warnings: string[] = [];
            await readJudgeSeats({
              client: viewClient({
                providerDryness: async () => {
                  throw quotingFailure();
                },
              },),
              phase: 'consolidation',
              signal: new AbortController().signal,
              l: warningRecordingLogger({ base: l, warnings, },),
            },);
            expect(warnings,).toEqual([
              'judge seats: the budget view could not be read (refused by TypeError); seating the full bench',
            ],);
          },
        },),
        it({
          name: 'WARNS OF AN UNREADABLE VIEW BY CLASS AND STATUS, then the provider\'s own words labelled as its own',
          fn: async () => {
            const warnings: string[] = [];
            await readJudgeSeats({
              client: viewClient({
                providerDryness: async () => {
                  throw new ProviderHttpError({ provider: 'synthetic', status: 502, bodyText: 'the cat meter is asleep', },);
                },
              },),
              phase: 'consolidation',
              signal: new AbortController().signal,
              l: warningRecordingLogger({ base: l, warnings, },),
            },);
            expect(warnings,).toEqual([
              'judge seats: the budget view could not be read (refused by ProviderHttpError from synthetic with HTTP 502 '
                + '(the provider said: "the cat meter is asleep")); seating the full bench',
            ],);
          },
        },),
        it({
          name: 'WAITS OUT THE SHORTEST HOLD ONCE and reads again when a bench this phase leans on cannot '
            + 'reach quorum among the seats a wet provider serves and a provider has named its return, '
            + 'the thirteenth class: Bedrock alone wet at the translate lane, Hyper held for its daily limit',
          fn: async () => {
            const script = scriptedViews({
              views: [
                BEDROCK_ALONE,
                ALL_WET,
              ],
            },);
            const seats = await readJudgeSeats({
              client: viewClient({
                providerDryness: script.read,
                providerHolds: () => ({
                  ...NO_HOLDS,
                  hyper: 40,
                }),
              },),
              phase: 'translate lane',
              signal: new AbortController().signal,
              l,
              pollMs: 5,
            },);
            expect(script.counter.reads,).toBe(2,);
            expect(seats.dry,).toEqual(ALL_WET,);
            expect(seats.translators,).toEqual(RUN_TRANSLATORS,);
          },
        },),
        it({
          name: 'DOES NOT WAIT when no provider has named a return the pass can wait for, seating what it read, nor '
            + 'when every bench can reach quorum however long a provider is held',
          fn: async () => {
            const noHold = scriptedViews({
              views: [
                BEDROCK_ALONE,
                ALL_WET,
              ],
            },);
            const short = await readJudgeSeats({
              client: viewClient({ providerDryness: noHold.read, },),
              phase: 'consolidation',
              signal: new AbortController().signal,
              l,
              pollMs: 5,
            },);
            expect(noHold.counter.reads,).toBe(1,);
            expect(short.dry,).toEqual(BEDROCK_ALONE,);

            const wet = scriptedViews({ views: [ALL_WET,], },);
            const full = await readJudgeSeats({
              client: viewClient({
                providerDryness: wet.read,
                providerHolds: () => ({
                  ...NO_HOLDS,
                  openrouter: 60_000,
                }),
              },),
              phase: 'consolidation',
              signal: new AbortController().signal,
              l,
              pollMs: 5,
            },);
            expect(wet.counter.reads,).toBe(1,);
            expect(full.dry,).toEqual(ALL_WET,);
          },
        },),
        it({
          name: 'SAYS SO when a bench the phase leans on is short of quorum and no provider has named its '
            + 'return, seating what it read without a wait: the seventh hakureico launch, Bedrock alone, whose '
            + 'seats lines read roster=10 withheld=none with three seats reachable (two since the owner culled '
            + 'gpt-oss-120b on 2026-09-24)',
          fn: async () => {
            const script = scriptedViews({ views: [BEDROCK_ALONE,], },);
            const { logger, lines, } = capturingLoggerPair();
            const seats = await readJudgeSeats({
              client: viewClient({ providerDryness: script.read, },),
              phase: 'preparation',
              signal: new AbortController().signal,
              l: logger,
              pollMs: 5,
            },);
            expect(script.counter.reads,).toBe(1,);
            expect(seats.wideSeats.length,).toBe(RUN_WIDE_SEATS.length,);
            /**
             The line that names the shortfall, if the reading said so.
             */
            const said = lines.find(function namesShortfall(line: string,): boolean {
              return line.includes(`JUDGE SEATS phase=preparation short of quorum: wide ${
            String(reachableSeats({ seats: RUN_WIDE_SEATS, dry: BEDROCK_ALONE, },).length,)
          } of ${
            String(RUN_WIDE_SEATS.length,)
          } reachable, quorum ${String(rosterQuorumSize({ rosterSize: RUN_WIDE_SEATS.length, },),)}`,);
            },);
            expect(said === undefined,).toBe(false,);
            expect(said?.includes('no provider has named a return the pass can wait for',),).toBe(true,);

            const whole = scriptedViews({ views: [ALL_WET,], },);
            const quiet = capturingLoggerPair();
            await readJudgeSeats({
              client: viewClient({ providerDryness: whole.read, },),
              phase: 'pictures',
              signal: new AbortController().signal,
              l: quiet.logger,
              pollMs: 5,
            },);
            expect(quiet.lines.some(function namesShortfall(line: string,): boolean {
              return line.includes('short of quorum',);
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: `${readJudgeSeats.name} beside the run-wide seat tally`,
      // THE CASES SHARE THE PROCESS-WIDE TALLY, so they run one at a time.
      concurrency: 1,
      children: [
        it({
          name: 'NAMES IN THE JUDGE SEATS LINE EVERY SEATED SEAT NO WET PROVIDER SERVES as unserved, and none when every '
            + 'seated seat is served, and does not take a seat the router only refused for withheld (zheermao20: '
            + '"glm-5.3 threw 4 of 4 asks" for four refusals and no request)',
          fn: async () => {
            using _fresh = withFreshRunSeats();
            for (let refusal = 0; refusal < 4; refusal += 1)
              RUN_SEATS.record({ modelId: GLM, outcome: 'unreachable', },);
            /** Lines the reading writes under the dry view. */
            const dryLines: string[] = [];
            await readJudgeSeats({
              client: viewClient({ providerDryness: async () => SYNTHETIC_AND_HYPER_DRY, },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines: dryLines, },),
            },);
            /** Lines the reading writes with every provider wet. */
            const wetLines: string[] = [];
            await readJudgeSeats({
              client: viewClient({ providerDryness: async () => ALL_WET, },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines: wetLines, },),
            },);

            expect(dryLines.length,).toBe(1,);
            expect(dryLines[0],).toContain(
              ' unserved=hf:Qwen/Qwen3.8-27B,glm-5.3 threw-withheld=none waited=0ms',
            );
            expect(wetLines.length,).toBe(1,);
            expect(wetLines[0],).toContain(' unserved=none threw-withheld=none waited=0ms',);
          },
        },),
      ],
    },),

    describe({
      name: `${readJudgeSeats.name} on the writing-bench floor`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STOPS THE ENTRY when a writing bench the phase leans on is below the pair a slate needs and no '
            + 'provider has named its return, the fifteenth class: the eighth hakureico pass at the lanes on '
            + 'Bedrock alone, no editor, no refiner (and, since google.gemma-4-e2b writes, a pair of '
            + 'translators at the floor)',
          fn: async () => {
            const script = scriptedViews({ views: [BEDROCK_ALONE,], },);
            const { logger, lines, } = capturingLoggerPair();
            await expect(readJudgeSeats({
              client: viewClient({ providerDryness: script.read, },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l: logger,
              pollMs: 5,
            },),).rejects.toThrow(WritingBenchUnreachableError,);
            expect(script.counter.reads,).toBe(1,);
            expect(lines.some(function namesStop(line: string,): boolean {
              return line.includes('JUDGE SEATS phase=lanes writing bench unreachable: editors 0 of',)
                && line.includes('stopping the entry',);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'WAITS OUT THE HOLD ONCE and STOPS when a bench is still below the floor after it (the editors '
            + 'at the lanes, since Bedrock alone reaches a pair of translators from 2026-09-08), and SEATS when '
            + 'the wait brought the bench back',
          fn: async () => {
            const stays = scriptedViews({
              views: [
                BEDROCK_ALONE,
                BEDROCK_ALONE,
              ],
            },);
            await expect(readJudgeSeats({
              client: viewClient({
                providerDryness: stays.read,
                providerHolds: holdsThatEnd({
                  first: {
                    ...NO_HOLDS,
                    hyper: 40,
                  },
                },),
              },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l,
              pollMs: 5,
            },),).rejects.toThrow('writing bench unreachable at lanes: editors',);
            expect(stays.counter.reads,).toBe(2,);

            const returns = scriptedViews({
              views: [
                BEDROCK_ALONE,
                ALL_WET,
              ],
            },);
            const seats = await readJudgeSeats({
              client: viewClient({
                providerDryness: returns.read,
                providerHolds: () => ({
                  ...NO_HOLDS,
                  hyper: 40,
                }),
              },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l,
              pollMs: 5,
            },);
            expect(returns.counter.reads,).toBe(2,);
            expect(seats.translators,).toEqual(RUN_TRANSLATORS,);
          },
        },),
        it({
          name: 'WRITES ONE LINE FOR EACH OVERLAPPED SLICE WHOSE SEAT READING STOPS THE ENTRY, its own, whether or '
            + 'not the caller has stopped, and the overlapped map adds none for the stop it does not throw',
          fn: async () => {
            /**
             Runs two slices side by side, each reading its seats before it
             starts as the per-slice hook does, on Bedrock alone.

             @param signal - caller's stop, aborted or not

             @returns Every line written, and what the slices refused with
             */
            async function overlappedReadings(
              { signal, }: { readonly signal: AbortSignal; },
            ): Promise<{
              readonly lines: readonly string[];
              readonly refusal: unknown;
            }> {
              const { logger, lines, } = capturingLoggerPair();
              try {
                await mapOverlapped({
                  items: [
                    'the windowsill',
                    'the sofa',
                  ],
                  overlap: 2,
                  signal,
                  l: logger,
                  nameOf: function sliceName({ item, },): string {
                    return item;
                  },
                  oneItem: async function readSeatsFirst(): Promise<string> {
                    await readJudgeSeats({
                      client: viewClient({ providerDryness: scriptedViews({ views: [BEDROCK_ALONE,], },).read, },),
                      phase: 'lanes',
                      signal,
                      l: logger,
                      pollMs: 5,
                    },);
                    throw new Error('unreachable: a reading on Bedrock alone seated the lanes',);
                  },
                },);
                throw new Error('unreachable: two slices whose readings stop the entry both finished',);
              }
              catch (refusal) {
                return {
                  lines,
                  refusal,
                };
              }
            }
            /**
             Reading under a caller that stopped before the slices started.
             */
            const stopping = new AbortController();
            stopping.abort(new Error('the owner called the cats in',),);
            /**
             Each slice's own line, the same for both since both read one view.
             */
            const stopLine = 'JUDGE SEATS phase=lanes writing bench unreachable: editors 0 of 4 reachable, floor 2; '
              + 'refiners 0 of 3 reachable, floor 2; stopping the entry';
            for (const signal of [
              stopping.signal,
              new AbortController().signal,
            ]) {
              // oxlint-disable-next-line no-await-in-loop -- each caller's run is checked in turn
              const { lines, refusal, } = await overlappedReadings({ signal, },);
              expect(refusal,).toBeInstanceOf(WritingBenchUnreachableError,);
              expect(lines,).toEqual([
                stopLine,
                stopLine,
              ],);
            }
          },
        },),
      ],
    },),
    describe({
      name: `${readPhaseSeats.name} for a phase that goes on without its writers`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GOES ON WHEN THE WRITING BENCH IS BELOW ITS FLOOR AND NO PROVIDER HAS NAMED ITS RETURN: one error line '
            + 'names the bench and says the phase goes on, none says the entry stops, and the benches below the floor are '
            + 'handed back',
          fn: async () => {
            const script = scriptedViews({ views: [ALL_DRY,], },);
            const lines: string[] = [];
            const { seats, unreachableWriters, } = await readPhaseSeats({
              client: viewClient({ providerDryness: script.read, },),
              phase: 'page consistency',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines, },),
              pollMs: 5,
            },);
            expect({
              reads: script.counter.reads,
              unreachableWriters,
              translators: seats.translators,
              errors: lines.filter(function isError(line,): boolean {
                return line.startsWith('error ',);
              },),
              stops: lines.filter(function saysStop(line,): boolean {
                return line.includes('stopping the entry',);
              },),
              warned: lines.filter(function isWarning(line,): boolean {
                return line.startsWith('warn ',);
              },),
            },).toEqual({
              reads: 1,
              unreachableWriters: ['translators 0 of 8 reachable, floor 2',],
              translators: RUN_TRANSLATORS,
              errors: [
                'error JUDGE SEATS phase=page consistency writing bench unreachable: translators 0 of 8 reachable, floor 2; '
                  + 'no provider has named a return the pass can wait for, so the phase goes on without its writers',
              ],
              stops: [],
              warned: [
                'warn JUDGE SEATS phase=page consistency short of quorum: wide 0 of 8 reachable, quorum 4 '
                  + '(reached again with the return of hyper, openrouter); '
                  + 'no provider has named a return the pass can wait for, so the phase runs on what is reachable',
              ],
            },);
          },
        },),
        it({
          name: 'SEATS THE WRITERS AND SAYS NOTHING OF THEM WHEN THE BENCH IS AT ITS FLOOR OR ABOVE, handing back no bench',
          fn: async () => {
            const lines: string[] = [];
            const { unreachableWriters, seats, } = await readPhaseSeats({
              client: viewClient({ providerDryness: scriptedViews({ views: [ALL_WET,], },).read, },),
              phase: 'page consistency',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines, },),
              pollMs: 5,
            },);
            expect({
              unreachableWriters,
              translators: seats.translators,
              unwanted: lines.filter(function isNotTheSeatsLine(line,): boolean {
                return !line.startsWith('info JUDGE SEATS phase=page consistency ',);
              },),
            },).toEqual({ unreachableWriters: [], translators: RUN_TRANSLATORS, unwanted: [], },);
          },
        },),
        it({
          name: 'STILL WAITS OUT A NAMED HOLD ONCE, then goes on without the writers when the bench is still below its floor, '
            + 'saying so after the wait in one error line that does not say the entry stops',
          fn: async () => {
            const stays = scriptedViews({ views: [ALL_DRY, ALL_DRY,], },);
            const lines: string[] = [];
            const { unreachableWriters, } = await readPhaseSeats({
              client: viewClient({
                providerDryness: stays.read,
                providerHolds: holdsThatEnd({ first: { ...NO_HOLDS, hyper: 40, }, },),
              },),
              phase: 'page consistency',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines, },),
              pollMs: 5,
            },);
            expect({
              reads: stays.counter.reads,
              unreachableWriters,
              errors: lines.filter(function isError(line,): boolean {
                return line.startsWith('error ',);
              },),
              waits: lines.filter(function isWait(line,): boolean {
                return line.startsWith('warn ',) && line.includes('waiting 40ms for the shortest hold to end',);
              },).length,
            },).toEqual({
              reads: 2,
              unreachableWriters: ['translators 0 of 8 reachable, floor 2',],
              errors: [
                'error JUDGE SEATS phase=page consistency writing bench unreachable after waiting 40ms: '
                  + 'translators 0 of 8 reachable, floor 2; the phase goes on without its writers',
              ],
              waits: 1,
            },);
          },
        },),
        it({
          name: 'SEATS THE WRITERS WHEN THE WAIT BROUGHT THE BENCH BACK, handing back no bench and writing no error line',
          fn: async () => {
            const returns = scriptedViews({ views: [ALL_DRY, ALL_WET,], },);
            const lines: string[] = [];
            const { unreachableWriters, seats, } = await readPhaseSeats({
              client: viewClient({
                providerDryness: returns.read,
                providerHolds: () => ({ ...NO_HOLDS, hyper: 40, }),
              },),
              phase: 'page consistency',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines, },),
              pollMs: 5,
            },);
            expect({
              reads: returns.counter.reads,
              unreachableWriters,
              translators: seats.translators,
              errors: lines.filter(function isError(line,): boolean {
                return line.startsWith('error ',);
              },),
            },).toEqual({ reads: 2, unreachableWriters: [], translators: RUN_TRANSLATORS, errors: [], },);
          },
        },),
        it({
          name: 'KEEPS WAITING WHILE A HOLD STILL RUNS: a rate-limit hold that ends first leaves the editors bench '
            + 'short, and the lanes wait goes on for the hold behind it (the NeuralWatt return) instead of stopping '
            + 'the entry short of it',
          fn: async () => {
            const script = scriptedViews({
              views: [
                BEDROCK_ALONE,
                BEDROCK_ALONE,
                ALL_WET,
              ],
            },);
            /**
             Holds asked so far.
             */
            const asks = { count: 0, };
            /**
             The holds each ask reads: the rate-limit hold and the return, then the return alone, then none.
             */
            const holdsByAsk = [
              {
                ...NO_HOLDS,
                synthetic: 10,
                neuralwatt: 30,
              },
              {
                ...NO_HOLDS,
                neuralwatt: 20,
              },
              NO_HOLDS,
            ];
            const lines: string[] = [];
            const seats = await readJudgeSeats({
              client: viewClient({
                providerDryness: script.read,
                providerHolds: () => {
                  const asked = holdsByAsk[Math.min(
                    asks.count,
                    holdsByAsk.length - 1,
                  )] ?? NO_HOLDS;
                  asks.count += 1;
                  return asked;
                },
              },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines, },),
              pollMs: 5,
            },);
            expect(script.counter.reads,).toBe(3,);
            expect(seats.translators,).toEqual(RUN_TRANSLATORS,);
            expect(lines.filter(function isError(line,): boolean {
              return line.startsWith('error ',);
            },),).toEqual([],);
          },
        },),

        it({
          name: 'STOPS A PHASE THAT DOES NOT GO ON WITHOUT ITS WRITERS exactly as before when the cause is not every '
            + 'provider dry, so the lanes on Bedrock alone still refuse with the bench error and still say the entry stops',
          fn: async () => {
            const lines: string[] = [];
            await expect(readPhaseSeats({
              client: viewClient({ providerDryness: scriptedViews({ views: [BEDROCK_ALONE,], },).read, },),
              phase: 'lanes',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines, },),
              pollMs: 5,
            },),).rejects.toThrow(WritingBenchUnreachableError,);
            expect(lines.filter(function isError(line,): boolean {
              return line.startsWith('error ',);
            },).length,).toBe(1,);
          },
        },),
        it({
          name: 'ENDS THE RUN WITH THE EVERY-PROVIDER-DRY REFUSAL, still saying the entry stops, when every provider reads '
            + 'dry with no hold to wait out before any chat call has been made, so the pass can stop instead of '
            + 'filing each pending entry as an error',
          fn: async () => {
            const lines: string[] = [];
            /**
             What the seat reading stopped with.
             */
            const refusal = await rejectionOf({
              promise: readPhaseSeats({
                client: viewClient({
                  providerDryness: scriptedViews({ views: [ALL_DRY,], },).read,
                  neuralwattReturn: () => ({ kind: 'at', msOfDay: 69_308_000, }),
                },),
                phase: 'lanes',
                signal: new AbortController().signal,
                l: levelCapturingLogger({ lines, },),
                pollMs: 5,
              },),
            },);
            expect(refusal,).toBeInstanceOf(EveryProviderDryError,);
            expect(caughtValueText(refusal,),).toContain('19:15:08 UTC',);
            expect(lines.filter(function isError(line,): boolean {
              return line.startsWith('error ',);
            },),).toEqual([
              'error JUDGE SEATS phase=lanes writing bench unreachable: editors 0 of 4 reachable, floor 2; '
                + 'refiners 0 of 3 reachable, floor 2; translators 0 of 8 reachable, floor 2; stopping the entry',
            ],);
          },
        },),
        it({
          name: 'ENDS THE RUN WITH THE EVERY-PROVIDER-DRY REFUSAL when every provider reads dry but Xiaomi, whose wet '
            + 'meter serves only the checker seat pinned here and no seat that writes, since a wet provider counts '
            + 'only if a writing bench could still reach its floor from it; with a writer seat on Xiaomi the cause '
            + 'stays the bench error and the entry stops',
          fn: async () => {
            /**
             Every provider dry but Xiaomi, whose meter can never read dry by itself.
             */
            const xiaomiWet: BudgetView = { ...ALL_DRY, xiaomi: false, };
            // THE ROSTER PIN: the seats a Xiaomi-only fleet reaches, among the roster and the checkers. The case
            // means "a fleet with one wet provider that serves one checker seat", and this line says which seat that
            // is today; the refusal that follows reads the roster's writing benches and not this seat.
            const { seats, } = await readPhaseSeats({
              client: viewClient({ providerDryness: scriptedViews({ views: [xiaomiWet,], },).read, },),
              phase: 'page consistency',
              signal: new AbortController().signal,
              l: levelCapturingLogger({ lines: [], },),
              pollMs: 5,
            },);
            expect([...seats.roster, ...seats.checkers,].filter(function isServed(modelId,): boolean {
              return !seats.unserved.includes(modelId,);
            },),).toEqual([SEAT_OPENROUTER_ONLY_CHECKER,],);
            /**
             What the seat reading stopped with on the roster's own writing providers.
             */
            const refusal = await rejectionOf({
              promise: readPhaseSeats({
                client: viewClient({ providerDryness: scriptedViews({ views: [xiaomiWet,], },).read, },),
                phase: 'lanes',
                signal: new AbortController().signal,
                l: levelCapturingLogger({ lines: [], },),
                pollMs: 5,
              },),
            },);
            expect(refusal,).toBeInstanceOf(EveryProviderDryError,);
            expect(caughtValueText(refusal,),).toContain('xiaomi wet (serves no seat that writes)',);
            /**
             What it stopped with when a writer seat sits on Xiaomi.
             */
            const withWriter = await rejectionOf({
              promise: readPhaseSeats({
                client: viewClient({
                  providerDryness: scriptedViews({ views: [xiaomiWet,], },).read,
                  writingProviders: { ...ALL_DRY, },
                },),
                phase: 'lanes',
                signal: new AbortController().signal,
                l: levelCapturingLogger({ lines: [], },),
                pollMs: 5,
              },),
            },);
            expect(withWriter,).toBeInstanceOf(WritingBenchUnreachableError,);
            expect(withWriter,).not.toBeInstanceOf(EveryProviderDryError,);
          },
        },),

        it({
          name: 'ENDS THE RUN WITH THE EVERY-PROVIDER-DRY REFUSAL when every provider still reads dry after the one wait '
            + 'for the shortest hold, as it does when none was held',
          fn: async () => {
            /**
             What the seat reading stopped with.
             */
            const refusal = await rejectionOf({
              promise: readPhaseSeats({
                client: viewClient({
                  providerDryness: scriptedViews({ views: [ALL_DRY, ALL_DRY,], },).read,
                  providerHolds: holdsThatEnd({ first: { ...NO_HOLDS, hyper: 40, }, },),
                },),
                phase: 'lanes',
                signal: new AbortController().signal,
                l: levelCapturingLogger({ lines: [], },),
                pollMs: 5,
              },),
            },);
            expect(refusal,).toBeInstanceOf(EveryProviderDryError,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Seat reading tests
