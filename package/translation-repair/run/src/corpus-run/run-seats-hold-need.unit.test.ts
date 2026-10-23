/**
 Tests that the seat reader waits only for a hold that could bring a short bench back.

 A hold on a provider that serves no seat of a bench the phase leans on, and that stands short, cannot change that
 bench's reach however long it is waited for, so the reader seats what is reachable at once; a hold on a provider that
 does serve such a seat is waited for. The providers a bench draws on come from the roster through `reachOf`,
 never from a list of names.

 Fixtures are cat-themed invention; every view and hold is scripted and no HTTP is made.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  type BudgetView,
  reachOf,
} from '@monochromatic-dev/translation-repair-provider';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  levelCapturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  phaseBenches,
  readJudgeSeats,
  RUN_TRANSLATORS,
} from '../../dist/final/node/index.mjs';
import { metersAllWet, } from '@monochromatic-dev/translation-repair-provider/ts/meter-states.test-fixture.ts';

//region Hold need tests

/**
 Every provider wet.
 */
const ALL_WET: BudgetView = {
  neuralwatt: false,
  synthetic: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  hyper: false,
  openrouter: false,
};

/**
 Bedrock alone wet: the translators stand short of their quorum and every other provider is dry or held.
 */
const BEDROCK_ALONE: BudgetView = {
  neuralwatt: true,
  synthetic: true,
  bedrock: false,
  deepseek: true,
  xiaomi: true,
  hyper: true,
  openrouter: true,
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
  hyper: 0,
  openrouter: 0,
};

/**
 Whether some seat of the translators bench is served by a provider.

 @param provider - provider asked about

 @returns Whether the roster routes a translator seat to it

 @example
 ```ts
 const serves = servesTranslators({ provider: 'hyper', },);
 ```
 */
function servesTranslators({ provider, }: { readonly provider: ProviderName; },): boolean {
  return RUN_TRANSLATORS.some(function isServed(modelId,): boolean {
    return reachOf({ modelId, },)[provider];
  },);
}

/**
 Reads the seats of the translate lane over scripted views and holds, counting reads.

 @param holdsByAsk - holds each ask reads, the last repeated

 @param lines - receives what the reading logs

 @returns Dryness reads made

 @example
 ```ts
 const reads = await readTranslateLane({ holdsByAsk: [NO_HOLDS,], lines: [], },);
 ```
 */
async function readTranslateLane(
  {
    holdsByAsk,
    lines,
  }: {
    readonly holdsByAsk: readonly typeof NO_HOLDS[];
    readonly lines: string[];
  },
): Promise<number> {
  /**
   Reads and asks made so far.
   */
  const counted = { reads: 0, asks: 0, };
  await readJudgeSeats({
    client: {
      providerDryness: async () => {
        counted.reads += 1;
        return (counted.reads === 1) ? BEDROCK_ALONE : ALL_WET;
      },
      providerMeterStates: metersAllWet,
      providerHolds: () => {
        const asked = holdsByAsk[Math.min(counted.asks, holdsByAsk.length - 1,)] ?? NO_HOLDS;
        counted.asks += 1;
        return asked;
      },
    },
    phase: 'translate lane',
    signal: new AbortController().signal,
    l: levelCapturingLogger({ lines, },),
    pollMs: 5,
  },);
  return counted.reads;
}

await describe({
  name: 'the seat reader waits only for a hold that serves a short bench',
  concurrency: 1,
  children: [
    it({
      name: 'THE CASE\'S PREMISE HOLDS: the translate lane leans on the translators, and no seat of them is served by '
        + 'Xiaomi while Hyper serves several',
      fn: async () => {
        expect(phaseBenches({ phase: 'translate lane', },),).toContain('translators',);
        expect(servesTranslators({ provider: 'xiaomi', },),).toBe(false,);
        expect(servesTranslators({ provider: 'hyper', },),).toBe(true,);
      },
    },),

    it({
      name: 'DOES NOT WAIT FOR A HOLD ON A PROVIDER THAT SERVES NO SEAT OF A SHORT BENCH: Bedrock alone wet, the '
        + 'translators short, and only Xiaomi held (it serves none of them), so the reading seats what it read at once',
      fn: async () => {
        const lines: string[] = [];
        const reads = await readTranslateLane({ holdsByAsk: [{ ...NO_HOLDS, xiaomi: 40, },], lines, },);
        expect(reads,).toBe(1,);
        expect(lines.some(function waits(line,): boolean {
          return line.includes('waiting ',);
        },),).toBe(false,);
      },
    },),

    it({
      name: 'WAITS FOR A HOLD ON A PROVIDER THAT SERVES A SEAT OF A SHORT BENCH, and for that one: Hyper held 40 ms '
        + 'and Xiaomi 10 ms, the wait is the 40 ms of the provider that serves the translators and not the 10 ms of '
        + 'the one that does not',
      fn: async () => {
        const lines: string[] = [];
        const reads = await readTranslateLane({
          holdsByAsk: [
            { ...NO_HOLDS, xiaomi: 10, hyper: 40, },
            NO_HOLDS,
          ],
          lines,
        },);
        expect(reads,).toBe(2,);
        expect(lines.some(function waitsForHyper(line,): boolean {
          return line.startsWith('warn JUDGE SEATS phase=translate lane short of quorum',)
            && line.includes('waiting 40ms for',);
        },),).toBe(true,);
        expect(lines.some(function waitsForXiaomi(line,): boolean {
          return line.includes('waiting 10ms for',);
        },),).toBe(false,);
      },
    },),
  ],
},);

//endregion Hold need tests
