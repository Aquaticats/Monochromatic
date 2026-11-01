/**
 Tests for the derivation of a writing bench from a reading: the static seats a wet provider serves, then the next models
 of the owner's role order, then the static seats nobody serves, cut at the static width.

 THE OWNER'S ANSWER OF 2026-11-01, "Backfill, and add sonnet", asked what a run does when a writing bench (editors or
 refiners) has fewer than two reachable seats: fill the dark seats with the next reachable models of the role order,
 and seat Claude Sonnet 5.5 on the refiner bench as it is on the editor bench. The cases here drive the derivation with
 scripted answers to "does a wet provider serve it" and "does the run seat it", so each rule is shown to move the
 bench: the order, the width, a withheld candidate, and a candidate that also checks under a contract that does and does
 not let a checker confirm text it wrote. NOTHING HERE REACHES A PROVIDER.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isDecisionSeat,
  MEASURED_CHECKER_ORDER,
  MODEL_CARDS,
  reaskPoolOf,
  rolePoolOf,
  type RosterModelId,
  standingOrderOf,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { reachOf, } from '@monochromatic-dev/translation-repair-provider';
import {
  backfillOrderOf,
  backfillWordsOf,
  EDITOR_BACKFILL_ORDER,
  REFINER_BACKFILL_ORDER,
  RUN_MODELS,
  RUN_WRITING_PROVIDERS,
  writingBenchFor,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Writing bench backfill derivation tests

/**
 The refiner bench of the static list: where Sonnet 5.5 sits second since the owner's answer of 2026-11-01.
 */
const STATIC_REFINERS: readonly RosterModelId[] = RUN_MODELS.refinerModelIds;

/**
 Builds a predicate from the models a case says are in.

 @param modelIds - models the predicate answers true for

 @returns Predicate over a model

 @example
 ```ts
 const served = isOneOf({ modelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,], },);
 ```
 */
function isOneOf(
  { modelIds, }: { readonly modelIds: readonly RosterModelId[]; },
): (modelId: RosterModelId,) => boolean {
  return function isIn(modelId: RosterModelId,): boolean {
    return modelIds.includes(modelId,);
  };
}

/**
 The Claude model the owner named for both writing benches on 2026-11-01 ("Backfill, and add sonnet").
 */
const SONNET = 'claude-sonnet-5-5' satisfies RosterModelId;

/**
 Models of the role order a pool-ordered run asks first, which every case leans on.
 */
const [firstRefiner, secondRefiner,] = REFINER_BACKFILL_ORDER;

await describe({
  name: 'writing bench backfill',
  concurrency: 1,
  children: [
    describe({
      name: backfillOrderOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'IS THE OWNER\'S ROLE ORDER OF THE ROLE\'S POOL LESS THE STATIC BENCH: the models the calibration '
            + 'measured in the role come first by standing, best first, and the pool is the role\'s, never a Claude '
            + 'card or a decision-only seat',
          fn: async () => {
            for (const { role, staticSeats, order, } of [
              {
                role: 'editor',
                staticSeats: RUN_MODELS.editorModelIds,
                order: EDITOR_BACKFILL_ORDER,
              },
              {
                role: 'refiner',
                staticSeats: RUN_MODELS.refinerModelIds,
                order: REFINER_BACKFILL_ORDER,
              },
            ] as const) {
              expect(order,).toEqual(backfillOrderOf({ role, staticSeats, },),);
              /**
               Models of the role's pool the static bench does not seat.
               */
              const candidates = rolePoolOf({ role, },).filter(function notSeated(modelId,): boolean {
                return !staticSeats.includes(modelId,);
              },);
              expect(order.length,).toBeGreaterThan(0,);
              for (const modelId of order) {
                expect(candidates,).toContain(modelId,);
                expect(isDecisionSeat({ modelId, },),).toBe(false,);
                expect(MODEL_CARDS[modelId].anthropic,).toBeUndefined();
              }
              // THE MEASURED MODELS LEAD, in the order of their standing, and the ones nobody measured follow.
              /**
               The candidates the calibration measured in the role.
               */
              const measured = standingOrderOf({
                role,
                modelIds: candidates,
              },);
              expect(order.slice(0, measured.length,),).toEqual(measured,);
            }
          },
        },),
        it({
          name: 'KEEPS QWEN3.8-27B AS A CANDIDATE, first for an editor and in the order of the owner for a refiner, '
            + 'although the run withholds it while Hyper serves it: the reading decides whether the bench seats it '
            + '(ledger B1330), where a re-ask that knows no reading leaves it out',
          fn: async () => {
            expect(EDITOR_BACKFILL_ORDER[0],).toBe(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,);
            expect(REFINER_BACKFILL_ORDER,).toContain(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,);
            expect(reaskPoolOf({ role: 'editor', },),).not.toContain(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,);
            expect(reaskPoolOf({ role: 'refiner', },),).not.toContain(SEAT_SYNTHETIC_VISION_NO_OPENROUTER,);
          },
        },),
        it({
          name: 'ENDS BOTH ORDERS WITH GEMMA 4 E2B BEFORE MERCURY, the owner\'s order of 2026-10-31 for the models '
            + 'never measured in the role, which the share of runaways and then the roster fall into',
          fn: async () => {
            for (const order of [EDITOR_BACKFILL_ORDER, REFINER_BACKFILL_ORDER,]) {
              expect(order.slice(-2,),).toEqual(['google.gemma-4-e2b', 'inception/mercury-2.5',],);
            }
          },
        },),
        it({
          name: 'LEAVES OUT EVERY STATIC SEAT, so a seat is never both on the bench and a stand-in for itself',
          fn: async () => {
            for (const modelId of RUN_MODELS.editorModelIds)
              expect(EDITOR_BACKFILL_ORDER,).not.toContain(modelId,);
            for (const modelId of RUN_MODELS.refinerModelIds)
              expect(REFINER_BACKFILL_ORDER,).not.toContain(modelId,);
          },
        },),
      ],
    },),
    describe({
      name: writingBenchFor.name,
      concurrency: 1,
      children: [
        it({
          name: 'KEEPS THE STATIC BENCH WHEN EVERY STATIC SEAT IS SERVED, and takes nothing from the role order',
          fn: async () => {
            const { bench, backfilled, } = writingBenchFor({
              staticSeats: STATIC_REFINERS,
              backfillOrder: REFINER_BACKFILL_ORDER,
              served: () => true,
              seatable: () => true,
              checkers: [],
              selfCertificationPermitted: true,
            },);
            expect(bench,).toEqual(STATIC_REFINERS,);
            expect(backfilled,).toEqual([],);
          },
        },),
        it({
          name: 'FILLS EACH PLACE A DARK STATIC SEAT LEAVES FROM THE ROLE ORDER, served static seats first, and cuts '
            + 'at the static width, so the dark seats the backfill replaced are gone',
          fn: async () => {
            /**
             The one static seat a wet provider serves here: the last, so that an unmoved order would show.
             */
            const keptSeat = STATIC_REFINERS.at(-1,);
            const { bench, backfilled, } = writingBenchFor({
              staticSeats: STATIC_REFINERS,
              backfillOrder: REFINER_BACKFILL_ORDER,
              served: isOneOf({ modelIds: [...STATIC_REFINERS.slice(-1,), ...REFINER_BACKFILL_ORDER,], },),
              seatable: () => true,
              checkers: MEASURED_CHECKER_ORDER,
              selfCertificationPermitted: true,
            },);
            expect(bench.length,).toBe(STATIC_REFINERS.length,);
            expect(bench[0],).toBe(keptSeat,);
            expect(backfilled,).toEqual(REFINER_BACKFILL_ORDER.slice(0, STATIC_REFINERS.length - 1,),);
            expect(bench.slice(1,),).toEqual(backfilled,);
          },
        },),
        it({
          name: 'DROPS A STATIC SEAT THE READING WITHHOLDS AND FILLS ITS PLACE FROM THE ROLE ORDER (ledger B1330): a '
            + 'hold is honoured for a hand-picked seat as for a candidate, so a served static seat that is not '
            + 'seatable is not on the bench and is not put last either',
          fn: async () => {
            const [withheldSeat,] = STATIC_REFINERS;
            expect(withheldSeat,).toBeDefined();
            const { bench, backfilled, } = writingBenchFor({
              staticSeats: STATIC_REFINERS,
              backfillOrder: REFINER_BACKFILL_ORDER,
              served: () => true,
              seatable: function notTheFirstStatic(modelId: RosterModelId,): boolean {
                return modelId !== withheldSeat;
              },
              checkers: MEASURED_CHECKER_ORDER,
              selfCertificationPermitted: true,
            },);
            expect(bench,).not.toContain(withheldSeat,);
            expect(bench.length,).toBe(STATIC_REFINERS.length,);
            expect(backfilled,).toEqual(REFINER_BACKFILL_ORDER.slice(0, 1,),);
          },
        },),
        it({
          name: 'KEEPS THE DARK STATIC SEATS LAST WHEN THE ROLE ORDER CANNOT FILL THEIR PLACES, so the bench still '
            + 'counts them as seats nobody serves and the floor reads the shortfall',
          fn: async () => {
            /**
             The second static seat is the only one served, and no candidate is.
             */
            const [, keptSeat,] = STATIC_REFINERS;
            const { bench, backfilled, } = writingBenchFor({
              staticSeats: STATIC_REFINERS,
              backfillOrder: REFINER_BACKFILL_ORDER,
              served: isOneOf({ modelIds: STATIC_REFINERS.slice(1, 2,), },),
              seatable: () => true,
              checkers: [],
              selfCertificationPermitted: true,
            },);
            expect(backfilled,).toEqual([],);
            expect(bench[0],).toBe(keptSeat,);
            expect([...bench,].toSorted(),).toEqual([...STATIC_REFINERS,].toSorted(),);
          },
        },),
        it({
          name: 'NEVER SEATS A CANDIDATE THE RUN WITHHOLDS, and the next of the order takes the place: the first '
            + 'candidate is served and not seatable here, so it is skipped and not replaced by nothing',
          fn: async () => {
            expect(firstRefiner,).toBeDefined();
            expect(secondRefiner,).toBeDefined();
            const { backfilled, } = writingBenchFor({
              staticSeats: STATIC_REFINERS,
              backfillOrder: REFINER_BACKFILL_ORDER,
              served: function servedAll(modelId: RosterModelId,): boolean {
                return !STATIC_REFINERS.includes(modelId,);
              },
              seatable: function notTheFirst(modelId: RosterModelId,): boolean {
                return modelId !== firstRefiner;
              },
              checkers: MEASURED_CHECKER_ORDER,
              selfCertificationPermitted: true,
            },);
            expect(backfilled,).not.toContain(firstRefiner,);
            expect(backfilled[0],).toBe(secondRefiner,);
            expect(backfilled.length,).toBe(Math.min(STATIC_REFINERS.length, REFINER_BACKFILL_ORDER.length - 1,),);
          },
        },),
        it({
          name: 'SEATS A CANDIDATE OF THE MEASURED CHECKER ORDER WHEREVER THE CHECKER BENCH IS, and skips it only '
            + 'where it sits on that bench and the contract does not let a checker confirm text it wrote (ledger '
            + 'B1333): the answerer names the voice, so a candidate off the bench never casts a ballot on text it '
            + 'wrote at a weight the discount does not halve',
          fn: async () => {
            const checkerCandidate = REFINER_BACKFILL_ORDER.find(function isChecker(modelId,): boolean {
              return MEASURED_CHECKER_ORDER.includes(modelId,);
            },);
            expect(checkerCandidate,).toBeDefined();
            if (checkerCandidate === undefined)
              return;
            /**
             Runs the derivation with every candidate served and seatable, the checker bench and the contract as given.
             */
            const backfilledUnder = (
              { checkers, selfCertificationPermitted, }: {
                readonly checkers: readonly RosterModelId[];
                readonly selfCertificationPermitted: boolean;
              },
            ): readonly RosterModelId[] =>
              writingBenchFor({
                staticSeats: STATIC_REFINERS,
                backfillOrder: REFINER_BACKFILL_ORDER,
                served: function servedAll(modelId: RosterModelId,): boolean {
                  return !STATIC_REFINERS.includes(modelId,);
                },
                seatable: () => true,
                checkers,
                selfCertificationPermitted,
              },)
                .backfilled;
            expect(backfilledUnder({ checkers: [checkerCandidate,], selfCertificationPermitted: true, },),)
              .toContain(checkerCandidate,);
            expect(backfilledUnder({ checkers: [], selfCertificationPermitted: true, },),)
              .toContain(checkerCandidate,);
            expect(backfilledUnder({ checkers: [], selfCertificationPermitted: false, },),)
              .toContain(checkerCandidate,);
            expect(backfilledUnder({ checkers: [checkerCandidate,], selfCertificationPermitted: false, },),)
              .not.toContain(checkerCandidate,);
            // THE SKIP IS NOT A LOST PLACE: the next candidate of the order takes it.
            expect(backfilledUnder({ checkers: [checkerCandidate,], selfCertificationPermitted: false, },).length,)
              .toBe(Math.min(STATIC_REFINERS.length, REFINER_BACKFILL_ORDER.length - 1,),);
          },
        },),
      ],
    },),
    describe({
      name: backfillWordsOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'SAYS NONE WHEN NOTHING WAS TAKEN, and names each bench that took seats, in bench order, with the '
            + 'seats in the order they were taken',
          fn: async () => {
            expect(backfillWordsOf({ backfilled: { editors: [], refiners: [], }, },),).toBe('none',);
            expect(backfillWordsOf({
              backfilled: {
                editors: [],
                refiners: [SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
              },
            },),).toBe(`refiners:${SEAT_SYNTHETIC_VISION_WITHHELD},${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}`,);
            expect(backfillWordsOf({
              backfilled: {
                editors: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
                refiners: [SEAT_SYNTHETIC_VISION_WITHHELD,],
              },
            },),).toBe(`editors:${SEAT_SYNTHETIC_VISION_NO_OPENROUTER}|refiners:${SEAT_SYNTHETIC_VISION_WITHHELD}`,);
          },
        },),
      ],
    },),
    describe({
      name: 'the static refiner bench and the writing providers',
      concurrency: 1,
      children: [
        it({
          name: 'SEATS CLAUDE SONNET 5.5 ON THE REFINER BENCH, as it is on the editor bench, on the owner\'s word of '
            + '2026-11-01, named and not found by its position',
          fn: async () => {
            expect(RUN_MODELS.refinerModelIds,).toContain(SONNET,);
            expect(RUN_MODELS.editorModelIds,).toContain(SONNET,);
            expect(MODEL_CARDS[SONNET].anthropic,).toBeDefined();
          },
        },),
        it({
          name: 'COUNTS THE PROVIDER OF EVERY MODEL THE ROLE ORDER MAY SEAT as a writing provider, not only the static seats\'',
          fn: async () => {
            for (const role of ['editor', 'refiner',] as const) {
              for (const modelId of [...rolePoolOf({ role, },), ...RUN_MODELS[`${role}ModelIds`],]) {
                /**
                 Providers that serve the model.
                 */
                const serving = Object.entries(reachOf({ modelId, },),)
                  .filter(function isServing([, isServed,],): boolean {
                    return isServed;
                  },)
                  .map(function nameOf([provider,],): string {
                    return provider;
                  },);
                for (const provider of serving)
                  expect(RUN_WRITING_PROVIDERS[provider as keyof typeof RUN_WRITING_PROVIDERS],).toBe(true,);
              }
            }
          },
        },),
      ],
    },),
  ],
},);

//endregion Writing bench backfill derivation tests
