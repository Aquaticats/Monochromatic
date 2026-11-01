/**
 Guards class one hundred nine (zheermao9, 2026-09-24): the class one hundred
 three re-seat lands at a chunk's start, but with eight chunks in flight the
 chunks already running when Synthetic's weekly allowance ran out kept the
 bench read at their start, so seven of eight checker rounds asked the two
 Synthetic-only seats a minute after the dry-out and heard one voice, short
 of quorum, while five re-seat lines had already named a bench with two
 reachable seats. The checker stage now reads the seating again at the stage
 itself. Also guards ledger B29: the bench a chunk starts with and the bench
 read at the stage are both refused where a checker edits or refines.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CheckerIndependenceError,
  CheckerQuorumError,
  type RepairModels,
  type RepairSliceSeating,
} from '@monochromatic-dev/translation-repair-assembly';
import {
  assertCheckerBench,
  checkerBenchAtStage,
  standingSeating,
} from '../dist/final/node/index.mjs';
import {
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Logger the stage writes to.
 */
const l = tagged({ tag: 'repair-checker-reseat-test', },);

/**
 Roster a chunk was seated with before the provider ran dry.
 */
const SEATED: RepairModels = {
  criticModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
  panelModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
  editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  judgeModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR, SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
  checkerModelIds: [
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  ],
};

/**
 Bench the reading seats once the withheld seat's provider is dry.
 */
const FRESH_CHECKERS = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_OPENROUTER_ONLY_CHECKER,
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: checkerBenchAtStage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS THE CHUNK\'S BENCH where the hook says the given roster still stands',
          fn: async () => {
            expect(await checkerBenchAtStage({
              models: SEATED,
              reseat: standingSeating,
              l,
            },),).toEqual(SEATED.checkerModelIds,);
            expect(await checkerBenchAtStage({
              models: SEATED,
              reseat: async (): Promise<RepairSliceSeating> => ({}),
              l,
            },),).toEqual(SEATED.checkerModelIds,);
          },
        },),
        it({
          name: 'ASKS THE BENCH READ AT THE STAGE where the hook hands a roster whose checkers differ',
          fn: async () => {
            expect(await checkerBenchAtStage({
              models: SEATED,
              reseat: async (): Promise<RepairSliceSeating> => ({
                repairModels: { ...SEATED, checkerModelIds: FRESH_CHECKERS, },
              }),
              l,
            },),).toEqual(FRESH_CHECKERS,);
          },
        },),
        it({
          name: 'REFUSES a re-seated bench below the checker floor rather than running a stage the contract refuses',
          fn: async () => {
            await expect(checkerBenchAtStage({
              models: SEATED,
              reseat: async (): Promise<RepairSliceSeating> => ({
                repairModels: { ...SEATED, checkerModelIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,], },
              }),
              l,
            },),).rejects
              .toThrow(CheckerQuorumError,);
          },
        },),
        // LEDGER B29: the re-seated bench was read for quorum alone, so a bench
        // naming the model that edited this chunk would have graded its own
        // rewrite. The chunk's editors wrote the text the stage checks, whatever
        // the fresh roster names as its editors.
        it({
          name: 'REFUSES a re-seated bench naming the editor that wrote this chunk\'s text',
          fn: async () => {
            await expect(checkerBenchAtStage({
              models: SEATED,
              reseat: async (): Promise<RepairSliceSeating> => ({
                repairModels: {
                  ...SEATED,
                  editorModelIds: [SEAT_OPENROUTER_ONLY_CHECKER,],
                  checkerModelIds: [
                    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                    SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                  ],
                },
              }),
              l,
            },),).rejects
              .toThrow(CheckerIndependenceError,);
          },
        },),
        it({
          name: 'REFUSES a re-seated bench naming the chunk\'s refiner',
          fn: async () => {
            await expect(checkerBenchAtStage({
              models: { ...SEATED, refinerModelIds: [SEAT_OPENROUTER_ONLY_CHECKER,], },
              reseat: async (): Promise<RepairSliceSeating> => ({
                repairModels: { ...SEATED, checkerModelIds: FRESH_CHECKERS, },
              }),
              l,
            },),).rejects
              .toThrow(CheckerIndependenceError,);
          },
        },),
        it({
          name: 'ASKS a re-seated bench naming the chunk\'s editor where the chunk permits self-certification',
          fn: async () => {
            /**
             Bench naming the chunk's editor.
             */
            const overlapping = [
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            ];
            expect(await checkerBenchAtStage({
              models: { ...SEATED, checkerSelfCertificationPermitted: true, },
              reseat: async (): Promise<RepairSliceSeating> => ({
                repairModels: { ...SEATED, checkerModelIds: overlapping, },
              }),
              l,
            },),).toEqual(overlapping,);
          },
        },),
      ],
    },),

    describe({
      name: assertCheckerBench.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a chunk roster whose checkers neither edit nor refine',
          fn: async () => {
            expect(() => {
              assertCheckerBench({
                models: { ...SEATED, refinerModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,], },
              },);
            },).not
              .toThrow();
          },
        },),
        it({
          name: 'REFUSES a chunk roster whose checker edits',
          fn: async () => {
            expect(() => {
              assertCheckerBench({
                models: { ...SEATED, editorModelIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,], },
              },);
            },).toThrow(CheckerIndependenceError,);
          },
        },),
        // LEDGER B29: the chunk check left the refiners out while its own TSDoc
        // said it refused a checker who refines, and every other caller of the
        // independence check passes them.
        it({
          name: 'REFUSES a chunk roster whose checker refines',
          fn: async () => {
            expect(() => {
              assertCheckerBench({
                models: { ...SEATED, refinerModelIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,], },
              },);
            },).toThrow(CheckerIndependenceError,);
          },
        },),
      ],
    },),
  ],
},);
