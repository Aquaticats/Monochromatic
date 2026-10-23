/**
 Tests for one slice of the editor calibration run through the whole repair
 lane over scripted clients, in which no model is ever called.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  editorLaneChairOf,
  runEditorLane,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_ONLY,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { benchSliceOf, } from './editor-calibrate-rounds.test-fixture.ts';
import { PINNED_EDITOR_BENCH_START, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/editor-bench-start.test-fixture.ts';
import { shippingLaneClient, } from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/editor-lane-script.test-fixture.ts';
import { scriptedClient, } from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/scripted-width-client.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: editorLaneChairOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES A CHAIR THAT SEATS ONE WIRE MODEL TWICE (R11), naming the bench and both cards: the run\'s '
            + 'list names Sonnet 5.5 and a Sonnet effort card, which the lane would otherwise seat together, one model '
            + 'heard twice counting as two voices toward a quorum',
          fn: async () => {
            expect(() =>
              editorLaneChairOf({
                bench: 'editor calibration editors',
                named: ['claude-sonnet-5-5', 'claude-sonnet-5-5-effort-high',],
              },)).toThrow(
              'The editor calibration editors bench seats claude-sonnet-5-5 more than once '
                + '(claude-sonnet-5-5, claude-sonnet-5-5-effort-high)',
              );
          },
        },),

        it({
          name: 'REFUSES THE SAME NAMED CARD TWICE, which is one voice counted twice',
          fn: async () => {
            expect(() =>
              editorLaneChairOf({
                bench: 'editor calibration refiners',
                named: ['claude-sonnet-5-5', 'claude-sonnet-5-5',],
              },)).toThrow('The editor calibration refiners bench seats claude-sonnet-5-5 more than once',);
          },
        },),

        it({
          name: 'SEATS THE WRITERS FIRST AND THEN THE NAMED CARDS (the control arm of R11): Sonnet beside the roster\'s '
            + 'writers is one wire model each and passes, and Haiku, which holds writer-unmeasured, takes no chair',
          fn: async () => {
            /**
             The chair as the lane builds it for the editors.
             */
            const chair = editorLaneChairOf({
              bench: 'editor calibration editors',
              named: ['claude-sonnet-5-5',],
            },);
            expect(chair.at(-1,),).toBe('claude-sonnet-5-5',);
            expect(chair,).not.toContain('claude-haiku-5-5',);
          },
        },),
      ],
    },),

    describe({
      name: runEditorLane.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS no round, no shipper and no refiner reach when the critics raise no claim',
          fn: async () => {
            /**
             What the lane produced for a slice nobody had a claim against.
             */
            const rounds = await runEditorLane({
              slice: benchSliceOf({ entryId: 'mittens', index: 0, },),
              client: scriptedClient({ script: { critic_report: { issues: [], }, }, },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(rounds,).toEqual({
              editor: [],
              refiner: [],
              refineAsked: false,
              editorShipped: [],
              refinerShipped: [],
              refinerHeard: [],
            },);
          },
        },),

        it({
          name: 'CREDITS every editor seat that wrote the shipped edit, as one composite candidate with no ballot, '
            + 'and reaches no rewriter on a one sentence slice',
          fn: async () => {
            /**
             What the lane produced when every stage agreed on one edit.
             */
            const rounds = await runEditorLane({
              slice: benchSliceOf({ entryId: 'mittens', index: 0, },),
              client: shippingLaneClient(),
              stageSettings: { ...SCRIPTED_STAGE_SETTINGS, editorBenchStart: PINNED_EDITOR_BENCH_START, },
            },);

            // THE SIX SEATS THAT SHIP ARE PINNED AGAIN (n166, Z3). The lane seats six editors from a bench the prompt rotates, so
            // which six of the seats that can hold an editor chair are seated moved whenever a sheet's wording did (n161: the
            // house tense rule), and the test had stopped naming them. The rotation's start is injected through the stage settings
            // instead, so no sheet text moves the seats and a lane that seated a worse subset of the bench fails. THE BENCH IS THE
            // ROSTER'S WRITERS AND THEN THE RUN'S NAMED EDITORS SINCE 2026-10-22 (`EDITOR_LANE_EDITORS`): Sonnet 5.5 joined on the
            // owner's Claude key and takes the last of the six, and Haiku 5.5, seated wide and not as an editor, holds none.
            /**
             Seats whose text shipped, in the order the lane seats them.
             */
            const shippers = [
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              SEAT_HYPER_ONLY,
              SEAT_HYPER_OPENROUTER_UNMEASURED,
              SEAT_BEDROCK_ONLY_TEXT,
              SEAT_OPENROUTER_ONLY,
              'claude-sonnet-5-5',
            ];

            expect(rounds,).toEqual({
              editor: [
                {
                  producers: [
                    {
                      kind: 'composite',
                      contributors: shippers,
                    },
                  ],
                  ballots: [],
                },
              ],
              refiner: [],
              refineAsked: false,
              editorShipped: shippers,
              refinerShipped: [],
              refinerHeard: [],
            },);
          },
        },),
        it({
          name: 'SEATS SONNET 5.5 AS AN EDITOR BY NAME AND HAIKU 5.5 AS NO EDITOR (C4): the run\'s editor list names Sonnet, '
            + 'which the roster-derived benches leave out (it holds judge-unmeasured), so the calibration must seat it by '
            + 'name to measure it; and the owner said "Haiku wide", so the wide-only card takes no editor chair and the '
            + 'calibration spends none of the Claude key on its edits',
          fn: async () => {
            /**
             What the lane produced when every stage agreed on one edit.
             */
            const rounds = await runEditorLane({
              slice: benchSliceOf({ entryId: 'mittens', index: 0, },),
              client: shippingLaneClient(),
              stageSettings: { ...SCRIPTED_STAGE_SETTINGS, editorBenchStart: PINNED_EDITOR_BENCH_START, },
            },);

            expect(rounds.editorShipped,).toContain('claude-sonnet-5-5',);
            expect(rounds.editorShipped,).not.toContain('claude-haiku-5-5',);
          },
        },),
      ],
    },),
  ],
},);
