/**
 Tests for the OpenRouter catalog: every row stands in for a roster seat,
 every roster seat has a row, and the routing preferences carry the owner's
 zero-data-retention decision.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  ANTHROPIC_SERVED_IDS,
  DECISION_ONLY_ROSTER_IDS,
  ROSTER_MODEL_IDS,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  OPENROUTER_DROPPED_SEATS,
  OPENROUTER_MODELS,
  OPENROUTER_PROVIDER_PREFERENCES,
  OPENROUTER_WITHHELD,
  openRouterIdFor,
  openRouterProviderPreferencesFor,
  reachOf,
  visionReachOf,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_UNMEASURED_OPENROUTER_ID,
  SEAT_HYPER_VISION,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { BEDROCK_ONLY_ROSTER_IDS, } from '@monochromatic-dev/translation-repair-shared-contract/ts/roster-buckets.test-fixture.ts';
import { openRouterServesLabel, } from './catalog-lookups.test-fixture.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'OPENROUTER_MODELS',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SERVES EVERY ROSTER SEAT BUT THE BEDROCK-ONLY SIZES AND THE TWO DROPPED ON 2026-09-09: '
            + 'eight rows, each standing in for a distinct roster id, seven widening reach and one, Mercury 2.5, '
            + 'the seat only this provider serves',
          fn: async () => {
            /**
             Roster seats the rows stand in for.
             */
            const seats = Object
              .values(OPENROUTER_MODELS,)
              .map(function seatOf(info,): string {
                return info.sharedWith;
              },);
            /**
             Roster ids this provider can stand in for: everything but the two
             sizes only Bedrock serves and the two seats dropped on evidence.
             */
            const reachable = ROSTER_MODEL_IDS.filter(function stillServed(modelId,): boolean {
              if (OPENROUTER_DROPPED_SEATS.has(modelId,))
                return false;
              // A decision-only seat is served by the decisions endpoint, which
              // is not this catalog.
              if (DECISION_ONLY_ROSTER_IDS.includes(modelId,))
                return false;
              // The Claude cards are served by the owner's own Claude key alone (`claude-reach.ts`).
              if (ANTHROPIC_SERVED_IDS.some(function isClaude(id,): boolean {
                return id === modelId;
              },))
                return false;
              return !BEDROCK_ONLY_ROSTER_IDS.some(function isBedrockOnly(id,): boolean {
                return id === modelId;
              },);
            },);
            expect(seats.toSorted(),).toEqual([...reachable,].toSorted(),);
            expect(new Set(seats,).size,).toBe(reachable.length,);
          },
        },),

        it({
          name: 'REACHES every roster model but the two dropped on 2026-09-09: Qwen3.8-27B keeps its '
            + 'Synthetic seat and GLM-5.3 keeps only its dry Hyper one, so a day with Synthetic and Hyper '
            + 'both dry seats the roster less those two',
          fn: async () => {
            // The two Gemma 4 sizes only Bedrock serves are reached there alone.
            for (const modelId of ROSTER_MODEL_IDS) {
              if (BEDROCK_ONLY_ROSTER_IDS.some(function isBedrockOnly(id,): boolean {
                return id === modelId;
              },))
                continue;
              if (DECISION_ONLY_ROSTER_IDS.includes(modelId,))
                continue;
              if (ANTHROPIC_SERVED_IDS.some(function isClaude(id,): boolean {
                return id === modelId;
              },))
                continue;
              expect(reachOf({ modelId, },).openrouter,).toBe(
                (!OPENROUTER_DROPPED_SEATS.has(modelId,)) && (!OPENROUTER_WITHHELD.has(modelId,)),
              );
            }
            // SERVED AND NOT BOUGHT: the catalog still lists Kimi-K3, so its
            // spelling and its prices are known, and the reach says OpenRouter
            // does not serve it, so the router never buys it there (the third
            // noname pass of 2026-09-09: 22 calls, 1.14 USD, after Synthetic
            // dried mid-phase).
            expect(openRouterServesLabel('moonshotai/kimi-k3',),).toBe(true,);
            expect(reachOf({ modelId: SEAT_SYNTHETIC_VISION_WITHHELD, },),).toMatchObject({
              synthetic: true,
              openrouter: false,
            },);
            expect(visionReachOf({ modelId: SEAT_SYNTHETIC_VISION_WITHHELD, },).openrouter,).toBe(false,);
            expect(reachOf({ modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },),).toMatchObject({
              synthetic: true,
              openrouter: false,
            },);
            expect(reachOf({ modelId: SEAT_HYPER_ONLY, },),).toMatchObject({
              hyper: true,
              openrouter: false,
            },);
          },
        },),

        it({
          name: 'CARRIES zero data retention and require_parameters on every request, the owner\'s decision '
            + 'of 2026-09-03, and price-sorted routing since 2026-09-09, when the balancer had sent the anchor '
            + 'judge to endpoints at twice the listing price',
          fn: async () => {
            expect(OPENROUTER_PROVIDER_PREFERENCES,).toEqual({
              zdr: true,
              require_parameters: true,
              sort: 'price',
            },);
          },
        },),

        it({
          name: 'IGNORES the measured endpoints and no others: Parasail and ModelRun for MiniMax M3 '
            + '(2026-09-03 and 2026-09-04 measurements beside the row), DeepInfra and Wafer for DeepSeek V4.1 '
            + 'Flash (XingZ607, 2026-09-18: 45 s and 26 s median against Morph\'s 11 s, 97 of 97 cut streams) '
            + 'joined by OpenInference (XingZ624 and XingZ625, 2026-09-23: 137 s and 116 s a stream against '
            + 'Morph\'s 11.5 s and 3.4 s, 450 cap-cut replies against 150; class ninety-one), '
            + 'then DekaLLM and Sail Research (XingZ626, 2026-09-23: 54 s and 40 s a stream against Morph\'s '
            + '6.2 s, 8,813 and 8,355 characters a reply against 514; class ninety-three), '
            + 'while every other row ignores no endpoint; Qwen3.8-27B and GLM-5.3 left the catalog on '
            + '2026-09-09, DeepSeek V4 Flash and V4 Pro on 2026-09-16',
          fn: async () => {
            expect(OPENROUTER_MODELS['minimax/minimax-m3'].ignoredEndpoints,).toEqual([
              'parasail',
              'modelrun',
            ],);
            expect(OPENROUTER_MODELS[SEAT_HYPER_OPENROUTER_UNMEASURED_OPENROUTER_ID].ignoredEndpoints,).toEqual([
              'deepinfra',
              'wafer',
              'open-inference',
              'dekallm',
              'sail-research',
            ],);
            /**
             Rows with a measured endpoint on them.
             */
            const measured: ReadonlySet<string> = new Set([
              'minimax/minimax-m3',
              SEAT_HYPER_OPENROUTER_UNMEASURED_OPENROUTER_ID,
            ],);
            /**
             Rows other than the one with a measured endpoint.
             */
            const others = Object
              .values(OPENROUTER_MODELS,)
              .filter(function unmeasured(info,): boolean {
                return !measured.has(info.id,);
              },);
            for (const info of others)
              expect(info.ignoredEndpoints,).toEqual([],);
          },
        },),

        it({
          name: 'NAMES THE MEASURED ENDPOINT AHEAD OF THE PRICE SORT where the cheapest endpoints reason at '
            + 'length by default (class ninety-three, XingZ626, 2026-09-23): Morph for DeepSeek V4.1 Flash '
            + '(847 streams at 6.2 s and 514 characters a reply against DekaLLM\'s 432 at 54 s and 8,813), '
            + 'Wafer for GLM-5.3-Flash (40 at 14.5 s and 651 characters against Together\'s 187 at 91 s and '
            + '23,666), and Together then CoreWeave for MiniMax M3, the two measured endpoints its schema '
            + 'requests reach (ledger H12: 7,601 streams at p90 5.5 s against 37,870 at 9.7 s since 2026-09-21, '
            + 'both FP4, completion alike), sent as `provider.order` with fallbacks allowed; every other row names none',
          fn: async () => {
            expect(OPENROUTER_MODELS[SEAT_HYPER_OPENROUTER_UNMEASURED_OPENROUTER_ID].preferredEndpoints,).toEqual(['morph',],);
            expect(OPENROUTER_MODELS['z-ai/glm-5.3-flash'].preferredEndpoints,).toEqual(['wafer',],);
            expect(OPENROUTER_MODELS['minimax/minimax-m3'].preferredEndpoints,).toEqual(['together', 'coreweave',],);
            /**
             Rows with a measured endpoint named on them.
             */
            const named: ReadonlySet<string> = new Set([
              SEAT_HYPER_OPENROUTER_UNMEASURED_OPENROUTER_ID,
              'z-ai/glm-5.3-flash',
              'minimax/minimax-m3',
            ],);
            for (const info of Object.values(OPENROUTER_MODELS,)) {
              if (!named.has(info.id,))
                expect(info.preferredEndpoints,).toEqual([],);
            }
          },
        },),

        it({
          name: 'SPELLS EVERY IGNORED OR NAMED SLUG AS THE GATEWAY LISTS IT, since a slug the gateway does not '
            + 'know is ignored silently: `openinference` kept OpenInference on the wire for a day because '
            + 'the listing spells it `open-inference`',
          fn: async () => {
            /**
             Provider slugs from `GET https://openrouter.ai/api/v1/providers`, read
             2026-09-04 (`~/temp/agent/providers-20260904.json`), for every
             upstream a run log of that day named for a roster model. Extend it
             from the same listing when a new slug is ignored or named: `wafer` read
             from the listing on 2026-09-18 for deepseek-v4.1-flash; `dekallm`,
             `morph` and `sail-research` on 2026-09-23 (class ninety-three).
             */
            const listed: ReadonlySet<string> = new Set([
              'akashml',
              'coreweave',
              'deepinfra',
              'dekallm',
              'io-net',
              'ionstream',
              'makora',
              'modal',
              'modelrun',
              'morph',
              'open-inference',
              'parasail',
              'phala',
              'reka',
              'sail-research',
              'together',
              'venice',
              'wafer',
            ],);
            for (const info of Object.values(OPENROUTER_MODELS,)) {
              for (const slug of [...info.ignoredEndpoints, ...info.preferredEndpoints,])
                expect(listed.has(slug,),).toBe(true,);
            }
          },
        },),
      ],
    },),

    describe({
      name: openRouterProviderPreferencesFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ADDS the row\'s ignore list to the shared preferences, as a copy the caller may not '
            + 'write back into the catalog, and the row\'s named endpoints as `order` only where it names '
            + 'one (class ninety-three)',
          fn: async () => {
            /**
             Preferences for the one row with an ignored endpoint.
             */
            const minimax = openRouterProviderPreferencesFor({ servedId: 'minimax/minimax-m3', },);
            expect(minimax,).toEqual({
              zdr: true,
              require_parameters: true,
              sort: 'price',
              ignore: [
                'parasail',
                'modelrun',
              ],
              order: ['together', 'coreweave',],
            },);
            expect(minimax.ignore,).not.toBe(OPENROUTER_MODELS['minimax/minimax-m3'].ignoredEndpoints,);
            expect(openRouterProviderPreferencesFor({ servedId: SEAT_HYPER_OPENROUTER_UNMEASURED_OPENROUTER_ID, },),).toEqual({
              zdr: true,
              require_parameters: true,
              sort: 'price',
              ignore: [
                'deepinfra',
                'wafer',
                'open-inference',
                'dekallm',
                'sail-research',
              ],
              order: ['morph',],
            },);
            expect(openRouterProviderPreferencesFor({ servedId: 'openai/gpt-oss-120b', },),).toEqual({
              zdr: true,
              require_parameters: true,
              sort: 'price',
              ignore: [],
            },);
          },
        },),
      ],
    },),

    describe({
      name: openRouterIdFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TRANSLATES each spelling the other providers use into this provider\'s slug',
          fn: async () => {
            expect(openRouterIdFor({ modelId: SEAT_SYNTHETIC_VISION_WITHHELD, },),).toEqual({
              served: true,
              id: 'moonshotai/kimi-k3',
            },);
            expect(openRouterIdFor({ modelId: SEAT_HYPER_VISION, },),).toEqual({
              served: true,
              id: 'minimax/minimax-m3',
            },);
            expect(openRouterIdFor({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },),).toEqual({
              served: true,
              id: 'z-ai/glm-5.3-flash',
            },);
          },
        },),
      ],
    },),

    describe({
      name: openRouterServesLabel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ANSWERS for a slug and not for a roster spelling, since the roster never names a model this way',
          fn: async () => {
            expect(openRouterServesLabel('moonshotai/kimi-k3',),).toBe(true,);
            expect(openRouterServesLabel(SEAT_SYNTHETIC_VISION_WITHHELD,),).toBe(false,);
            expect(openRouterServesLabel('kimi-k3',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
