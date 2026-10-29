/**
 Tests for the line that records what one call cost.

 TESTED THROUGH ITS RETURN VALUE rather than by capturing a logger's side
 effect, matching `stream-cut.unit.test.ts`: `reportSpend` hands back the exact
 line it logs, so an assertion here reads as a statement about the LINE a
 reader will parse rather than about whatever the logging subsystem did.

 THE CASE THIS MODULE EXISTS FOR is the one where the provider reported no
 usage at all. A run whose provider stayed quiet and a run that spent nothing
 total the same, and the only thing that tells them apart is that the line is
 printed anyway, carrying a named absence. Omitting the line would let a reader
 report a cheap run when what happened was an unreported one.

 ZERO IS NOT ABSENCE, and it gets its own case beside it. A provider that
 reports zero completion tokens has said something; one that reports nothing
 has not. Collapsing them is the single mistake this shape is built to refuse.

 Model identifiers come from the catalog, since a spend line names a seat that
 has to be findable in the roster. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  reportSpend,
  SPEND_MARKER,
} from '../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Model served by the metered provider, from the Charm Hyper catalog.
 */
const HYPER_MODEL = 'qwen3.8-max';

/**
 Model served by the subscription provider, from the Synthetic roster.
 */
const SYNTHETIC_MODEL = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 Completion carrying the usage block a provider fills in when it reports.

 @param promptTokens - tokens the request consumed

 @param completionTokens - tokens the answer produced, thinking included

 @returns Completion shaped as the extractor hands one back

 @example
 ```ts
 const extracted = reported({ promptTokens: 8, completionTokens: 2, },);
 ```
 */
function reported(
  {
    promptTokens,
    completionTokens,
  }: {
    readonly promptTokens: number;
    readonly completionTokens: number;
  },
): { readonly text: string; readonly usage: { readonly prompt_tokens: number; readonly completion_tokens: number; }; } {
  return {
    text: 'The cat approved this rendering.',
    usage: {
      prompt_tokens: promptTokens,
      completion_tokens: completionTokens,
    },
  };
}

/**
 Completion from a provider that returned no usage block at all.

 @returns Completion with text and nothing said about what it cost

 @example
 ```ts
 const extracted = silent();
 ```
 */
function silent(): { readonly text: string; } {
  return { text: 'The cat approved this rendering.', };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: reportSpend.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES the metered provider, the seat, and both counts, which is '
            + 'the control the rest of these cases depart from one field at a time',
          fn: async () => {
            expect(
              reportSpend({
                provider: 'hyper',
                label: HYPER_MODEL,
                extracted: reported({
                  promptTokens: 5_120,
                  completionTokens: 3_072,
                },),
              },),
            )
              .toBe('SPEND provider=hyper model=qwen3.8-max prompt=5120 completion=3072',);
          },
        },),

        it({
          name: 'NAMES the subscription provider on its own calls, so a reader '
            + 'totalling credits can drop the half of a run that is not priced per '
            + 'token instead of adding it',
          fn: async () => {
            expect(
              reportSpend({
                provider: 'synthetic',
                label: SYNTHETIC_MODEL,
                extracted: reported({
                  promptTokens: 12,
                  completionTokens: 34,
                },),
              },),
            )
              .toBe('SPEND provider=synthetic model=hf:zai-org/GLM-5.3-Flash prompt=12 completion=34',);
          },
        },),

        it({
          name: 'KEEPS the line and marks both counts absent when the provider '
            + 'reported no usage, so an unreported run cannot be read as a cheap one',
          fn: async () => {
            expect(
              reportSpend({
                provider: 'hyper',
                label: HYPER_MODEL,
                extracted: silent(),
              },),
            )
              .toBe('SPEND provider=hyper model=qwen3.8-max prompt=unreported completion=unreported',);
          },
        },),

        it({
          name: 'REPORTS a reported zero as zero rather than as absent, which is '
            + 'the distinction the named absence exists to protect: a provider that '
            + 'said nothing and one that said nothing was spent are not the same run',
          fn: async () => {
            expect(
              reportSpend({
                provider: 'hyper',
                label: HYPER_MODEL,
                extracted: reported({
                  promptTokens: 0,
                  completionTokens: 0,
                },),
              },),
            )
              .toBe('SPEND provider=hyper model=qwen3.8-max prompt=0 completion=0',);
          },
        },),

        it({
          name: 'APPENDS the USD cost as a trailing field only when the caller has one, so the provider '
            + 'billing in USD says what a call cost and every other line keeps its shape',
          fn: async () => {
            expect(
              reportSpend({
                provider: 'openrouter',
                label: 'deepseek/deepseek-v4.1-flash',
                extracted: reported({
                  promptTokens: 342,
                  completionTokens: 400,
                },),
                costUsd: 0.00015646,
              },),
            )
              .toBe('SPEND provider=openrouter model=deepseek/deepseek-v4.1-flash prompt=342 completion=400 cost=0.00015646',);
          },
        },),

        it({
          name: 'APPENDS the serving upstream as a trailing field only when the caller names one, '
            + 'percent-encoded so a display name with a space stays one field of a line that splits '
            + 'on spaces',
          fn: async () => {
            expect(
              reportSpend({
                provider: 'openrouter',
                label: 'minimax/minimax-m3',
                extracted: reported({
                  promptTokens: 2_263,
                  completionTokens: 117,
                },),
                costUsd: 0.00032778,
                endpoint: 'Parasail',
              },),
            )
              .toBe('SPEND provider=openrouter model=minimax/minimax-m3 prompt=2263 completion=117 cost=0.00032778 endpoint=Parasail',);
            expect(
              reportSpend({
                provider: 'openrouter',
                label: 'google/gemma-4-26b-a4b-it',
                extracted: reported({
                  promptTokens: 1,
                  completionTokens: 2,
                },),
                endpoint: 'Google AI Studio',
              },),
            )
              .toBe('SPEND provider=openrouter model=google/gemma-4-26b-a4b-it prompt=1 completion=2 endpoint=Google%20AI%20Studio',);
          },
        },),

        it({
          name: 'OPENS the returned line with the marker the reader finds it by, so '
            + 'a line this module wrote round-trips through `readSpendLine` rather '
            + 'than reading as prose that happens to mention it',
          fn: async () => {
            expect(
              reportSpend({
                provider: 'hyper',
                label: HYPER_MODEL,
                extracted: silent(),
              },)
                .startsWith(SPEND_MARKER,),
            )
              .toBe(true,);
          },
        },),

        it({
          name: 'writes every field as one name and one value, so a reader splits '
            + 'the line rather than matching it and no field can hide a space',
          fn: async () => {
            expect(
              reportSpend({
                provider: 'synthetic',
                label: SYNTHETIC_MODEL,
                extracted: reported({
                  promptTokens: 1,
                  completionTokens: 2,
                },),
              },)
                .split(' ',)
                .slice(1,)
                .map(function nameOf(field,): string {
                  return field.split('=',)[0] ?? '';
                },),
            )
              .toEqual([
                'provider',
                'model',
                'prompt',
                'completion',
              ],);
          },
        },),
      ],
    },),

    describe({
      name: `${reportSpend.name} trailing marks`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'APPENDS estimated=abandoned and cached=N as trailing fields only when given, after the cost '
            + 'and the endpoint, so the 2026-09-09 reckoning of abandoned streams and the cache hits of '
            + 'price-sorted routing both read off the same grammar',
          fn: async () => {
            expect(
              reportSpend({
                provider: 'openrouter',
                label: 'deepseek/deepseek-v4.1-flash',
                extracted: reported({
                  promptTokens: 1_000,
                  completionTokens: 10,
                },),
                costUsd: 0.0006,
                estimated: 'abandoned',
              },),
            )
              .toBe('SPEND provider=openrouter model=deepseek/deepseek-v4.1-flash prompt=1000 completion=10 cost=0.0006 estimated=abandoned',);
            expect(
              reportSpend({
                provider: 'openrouter',
                label: 'deepseek/deepseek-v4.1-flash',
                extracted: reported({
                  promptTokens: 4_000,
                  completionTokens: 12,
                },),
                costUsd: 0.0004,
                endpoint: 'Baidu',
                cachedTokens: 3_072,
              },),
            )
              .toBe('SPEND provider=openrouter model=deepseek/deepseek-v4.1-flash prompt=4000 completion=12 cost=0.0004 endpoint=Baidu cached=3072',);
          },
        },),
      ],
    },),
  ],
},);
