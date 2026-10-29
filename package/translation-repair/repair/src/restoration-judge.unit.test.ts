/**
 Tests for the bilingual restoration judge wire and ensemble stage.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  isRestorationJudgeWire,
  resolveRestorationJudgment,
  runRestorationJudge,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the stage under test.
 */
const l = tagged({ tag: 'restoration-judge-test', },);

/**
 References for a two-seed entry.
 */
const REFERENCES = [
  {
    seedId: 'seed/omission-0',
    deletedText: 'The cat also chases crimson butterflies across the meadow.',
  },
  {
    seedId: 'seed/omission-1',
    deletedText: 'The bowl by the stove stays full through the night.',
  },
] as const;

/**
 Client scripted per model with a fixed verdict list; a model in the
 silent set loses its voice so quorum paths can be exercised.

 @param verdictsByModel - verdicts each model casts, seed order

 @param silent - models whose calls fail
 */
function judgingClient(
  {
    verdictsByModel,
    silent = new Set<string>(),
  }: {
    readonly verdictsByModel: Readonly<Record<string, readonly string[]>>;
    readonly silent?: ReadonlySet<string>;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (silent.has(request.modelId,)) {
        return {
          kind: 'schema-mismatch',
          rawText: '',
          detail: 'scripted silence',
        };
      }

      /**
       Verdicts this model casts, defaulting to all restored.
       */
      const verdicts = verdictsByModel[request.modelId] ?? ['restored', 'restored',];

      /**
       Scripted wire report over the reference numbering.
       */
      const scripted: unknown = {
        judgments: verdicts.map(function toJudgment(verdict, index,) {
          return {
            reference: index + 1,
            verdict,
          };
        },),
      };
      if (!request.validate(scripted,))
        throw new Error('scripted payload failed the guard',);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
  };
}

/**
 Three-judge roster for the stage tests.
 */
const JUDGES: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isRestorationJudgeWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'accepts well-formed reports and rejects malformed ones',
          fn: async () => {
            expect(isRestorationJudgeWire({
              judgments: [{ reference: 1, verdict: 'restored', },],
            },),).toBe(true,);
            expect(isRestorationJudgeWire({ judgments: [], },),).toBe(true,);
            expect(isRestorationJudgeWire({},),).toBe(false,);
            expect(isRestorationJudgeWire({ judgments: [{ reference: 1.5, verdict: 'restored', },], },),)
              .toBe(false,);
            expect(isRestorationJudgeWire({ judgments: [{ reference: 1, },], },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: resolveRestorationJudgment.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'resolves through the index map and records irregularities',
          fn: async () => {
            /** Report with one good, one out-of-range, one unknown verdict. */
            const resolution = resolveRestorationJudgment({
              wire: {
                judgments: [
                  { reference: 1, verdict: 'restored', },
                  { reference: 9, verdict: 'restored', },
                  { reference: 2, verdict: 'flawless', },
                ],
              },
              seedIds: ['seed/omission-0', 'seed/omission-1',],
            },);
            expect(resolution.verdicts['seed/omission-0'],).toBe('restored',);
            expect(resolution.verdicts['seed/omission-1'],).toBe(undefined,);
            expect(resolution.findings,).toContain('judge-reference-out-of-range (9)',);
            expect(resolution.findings,).toContain('unknown-restoration-verdict (flawless)',);
            expect(resolution.findings,).toContain('missing-judgment (2)',);
          },
        },),
      ],
    },),

    describe({
      name: runRestorationJudge.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'takes the conservative lower median across judges',
          fn: async () => {
            /** Split judgments: restored/partial/absent on seed 0, all restored on seed 1. */
            const judgments = await runRestorationJudge({
              client: judgingClient({
                verdictsByModel: {
                  [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: ['restored', 'restored',],
                  [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: ['partial', 'restored',],
                  [SEAT_SYNTHETIC_VISION_WITHHELD]: ['absent', 'restored',],
                },
              },),
              judgeModelIds: JUDGES,
              sourceText: '猫猫追蝴蝶。碗是满的。',
              repairedText: 'The cat chases butterflies. The bowl is full.',
              references: REFERENCES,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            // restored/partial/absent sorts to absent<partial<restored; lower median is partial.
            expect(judgments['seed/omission-0']?.verdict,).toBe('partial',);
            expect(judgments['seed/omission-0']?.judged,).toBe(true,);
            expect(judgments['seed/omission-1']?.verdict,).toBe('restored',);
          },
        },),

        it({
          name: 'rounds an even split toward the less-credited verdict',
          fn: async () => {
            /** Two judges heard, one restored one absent, on seed 0. */
            const judgments = await runRestorationJudge({
              client: judgingClient({
                verdictsByModel: {
                  [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: ['restored', 'restored',],
                  [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: ['absent', 'restored',],
                },
                silent: new Set([SEAT_SYNTHETIC_VISION_WITHHELD,],),
              },),
              judgeModelIds: JUDGES,
              sourceText: '猫猫追蝴蝶。碗是满的。',
              repairedText: 'The cat chases butterflies. The bowl is full.',
              references: REFERENCES,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            // Two heard is still quorum over three; lower median of [absent, restored] is absent.
            expect(judgments['seed/omission-0']?.judged,).toBe(true,);
            expect(judgments['seed/omission-0']?.verdict,).toBe('absent',);
          },
        },),

        it({
          name: 'marks seeds unjudged when the judge roster loses quorum',
          fn: async () => {
            /** Only one of three judges answers: quorum unmet. */
            const judgments = await runRestorationJudge({
              client: judgingClient({
                verdictsByModel: { [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: ['restored', 'restored',], },
                silent: new Set([SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],),
              },),
              judgeModelIds: JUDGES,
              sourceText: '猫猫追蝴蝶。碗是满的。',
              repairedText: 'The cat chases butterflies. The bowl is full.',
              references: REFERENCES,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(judgments['seed/omission-0']?.judged,).toBe(false,);
            expect(judgments['seed/omission-1']?.judged,).toBe(false,);
          },
        },),

        it({
          name: 'returns nothing for an entry with no references',
          fn: async () => {
            /** Empty reference set short-circuits before any call. */
            const judgments = await runRestorationJudge({
              client: judgingClient({ verdictsByModel: {}, },),
              judgeModelIds: JUDGES,
              sourceText: '猫',
              repairedText: 'cat',
              references: [],
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(Object.keys(judgments,),).toHaveLength(0,);
          },
        },),

        it({
          name: 'COUNTS A BALLOT THAT NAMED ONE SEED ONLY on that seed, since the other seeds keep the '
            + 'verdicts they did get',
          fn: async () => {
            /**
             One judge names both seeds, one names only the first, one names
             both: the second seed is judged from the two ballots it has.
             */
            const judgments = await runRestorationJudge({
              client: judgingClient({
                verdictsByModel: {
                  [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: ['restored', 'restored',],
                  [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: ['restored',],
                  [SEAT_SYNTHETIC_VISION_WITHHELD]: ['restored', 'restored',],
                },
              },),
              judgeModelIds: JUDGES,
              sourceText: '猫猫追蝴蝶。碗是满的。',
              repairedText: 'The cat chases butterflies. The bowl is full.',
              references: REFERENCES,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(judgments,).toEqual({
              'seed/omission-0': { verdict: 'restored', judged: true, votes: 3, },
              'seed/omission-1': { verdict: 'restored', judged: true, votes: 2, },
            },);
          },
        },),
      ],
    },),
  ],
},);
