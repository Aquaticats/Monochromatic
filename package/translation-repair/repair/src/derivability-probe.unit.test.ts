/**
 Tests for the derivability probe wire and ensemble stage.
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
  isDerivabilityVerdict,
  resolveDerivabilityJudgment,
  runDerivabilityProbe,
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
const l = tagged({ tag: 'derivability-probe-test', },);

/**
 References for a two-seed entry.
 */
const REFERENCES = [
  {
    seedId: 'seed/omission-0',
    deletedText: 'The cat also chases crimson butterflies across the meadow at dawn.',
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
function probingClient(
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
       Verdicts this model casts, defaulting to all derivable.
       */
      const verdicts = verdictsByModel[request.modelId] ?? ['derivable', 'derivable',];

      /**
       Scripted wire report over the candidate numbering.
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
      name: isDerivabilityVerdict.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'accepts the closed vocabulary and rejects everything else',
          fn: async () => {
            expect(isDerivabilityVerdict('derivable',),).toBe(true,);
            expect(isDerivabilityVerdict('partially-derivable',),).toBe(true,);
            expect(isDerivabilityVerdict('not-derivable',),).toBe(true,);
            expect(isDerivabilityVerdict('restored',),).toBe(false,);
            expect(isDerivabilityVerdict(1,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: resolveDerivabilityJudgment.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'resolves through the index map and records irregularities',
          fn: async () => {
            /** Report with one good, one out-of-range, one unknown verdict. */
            const resolution = resolveDerivabilityJudgment({
              wire: {
                judgments: [
                  { reference: 1, verdict: 'derivable', },
                  { reference: 9, verdict: 'derivable', },
                  { reference: 2, verdict: 'restored', },
                ],
              },
              seedIds: ['seed/omission-0', 'seed/omission-1',],
            },);
            expect(resolution.verdicts['seed/omission-0'],).toBe('derivable',);
            expect(resolution.verdicts['seed/omission-1'],).toBe(undefined,);
            expect(resolution.findings,).toContain('derivability-reference-out-of-range (9)',);
            expect(resolution.findings,).toContain('unknown-derivability-verdict (restored)',);
            expect(resolution.findings,).toContain('missing-derivability-judgment (2)',);
          },
        },),
        it({
          name: 'KEEPS THE FIRST of two judgments naming one candidate and records the second as a duplicate',
          fn: async () => {
            expect(resolveDerivabilityJudgment({
              wire: {
                judgments: [
                  { reference: 1, verdict: 'not-derivable', },
                  { reference: 1, verdict: 'derivable', },
                  { reference: 2, verdict: 'derivable', },
                ],
              },
              seedIds: ['seed/omission-0', 'seed/omission-1',],
            },),).toEqual({
              verdicts: {
                'seed/omission-0': 'not-derivable',
                'seed/omission-1': 'derivable',
              },
              findings: ['duplicate-derivability-judgment (1)',],
            },);
          },
        },),
      ],
    },),

    describe({
      name: runDerivabilityProbe.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'takes the upper median across judges',
          fn: async () => {
            /** Split judgments: derivable/partially/not on seed 0, all derivable on seed 1. */
            const derivability = await runDerivabilityProbe({
              client: probingClient({
                verdictsByModel: {
                  [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: ['derivable', 'derivable',],
                  [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: ['partially-derivable', 'derivable',],
                  [SEAT_SYNTHETIC_VISION_WITHHELD]: ['not-derivable', 'derivable',],
                },
              },),
              judgeModelIds: JUDGES,
              sourceText: '猫猫黎明追蝴蝶。碗是满的。',
              references: REFERENCES,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            // Odd count: the median of not<partially<derivable is partially-derivable.
            expect(derivability['seed/omission-0']?.verdict,).toBe('partially-derivable',);
            expect(derivability['seed/omission-0']?.judged,).toBe(true,);
            expect(derivability['seed/omission-1']?.verdict,).toBe('derivable',);
          },
        },),

        it({
          name: 'rounds an even split toward the more-derivable verdict',
          fn: async () => {
            /** Two judges heard, one derivable one not-derivable, on seed 0. */
            const derivability = await runDerivabilityProbe({
              client: probingClient({
                verdictsByModel: {
                  [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: ['derivable', 'derivable',],
                  [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: ['not-derivable', 'derivable',],
                },
                silent: new Set([SEAT_SYNTHETIC_VISION_WITHHELD,],),
              },),
              judgeModelIds: JUDGES,
              sourceText: '猫猫黎明追蝴蝶。碗是满的。',
              references: REFERENCES,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            // Upper median of [not-derivable, derivable] is derivable: the
            // excuse, not the pipeline, carries the burden of proof.
            expect(derivability['seed/omission-0']?.judged,).toBe(true,);
            expect(derivability['seed/omission-0']?.verdict,).toBe('derivable',);
          },
        },),

        it({
          name: 'marks seeds unjudged and unexcused when quorum is lost',
          fn: async () => {
            /** Only one of three judges answers: quorum unmet. */
            const derivability = await runDerivabilityProbe({
              client: probingClient({
                verdictsByModel: { [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: ['not-derivable', 'not-derivable',], },
                silent: new Set([SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],),
              },),
              judgeModelIds: JUDGES,
              sourceText: '猫猫黎明追蝴蝶。碗是满的。',
              references: REFERENCES,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            // Unjudged defaults to derivable so a lost probe never excuses.
            expect(derivability['seed/omission-0']?.judged,).toBe(false,);
            expect(derivability['seed/omission-0']?.verdict,).toBe('derivable',);
          },
        },),

        it({
          name: 'COUNTS ONLY THE JUDGES WHO CAST A VERDICT on a seed, so a judge that left one candidate out is no '
            + 'vote on it',
          fn: async () => {
            /** One judge answers only the first candidate, the other two answer both. */
            const derivability = await runDerivabilityProbe({
              client: probingClient({
                verdictsByModel: {
                  [SEAT_HYPER_OPENROUTER_VISION_EDITOR]: ['not-derivable',],
                  [SEAT_SYNTHETIC_VISION_NO_OPENROUTER]: ['derivable', 'not-derivable',],
                  [SEAT_SYNTHETIC_VISION_WITHHELD]: ['partially-derivable', 'not-derivable',],
                },
              },),
              judgeModelIds: JUDGES,
              sourceText: '猫猫黎明追蝴蝶。碗是满的。',
              references: REFERENCES,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(derivability,).toEqual({
              'seed/omission-0': {
                verdict: 'partially-derivable',
                judged: true,
                votes: 3,
              },
              'seed/omission-1': {
                verdict: 'not-derivable',
                judged: true,
                votes: 2,
              },
            },);
          },
        },),

        it({
          name: 'returns nothing for an entry with no references',
          fn: async () => {
            /** Empty reference set short-circuits before any call. */
            const derivability = await runDerivabilityProbe({
              client: probingClient({ verdictsByModel: {}, },),
              judgeModelIds: JUDGES,
              sourceText: '猫',
              references: [],
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(Object.keys(derivability,),).toHaveLength(0,);
          },
        },),
      ],
    },),
  ],
},);
