import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  COMPLETION_CAP,
  MIN_PROVIDER_CALLS,
  MODEL_CARDS,
  POOLED_P90,
  ROSTER_MODEL_IDS,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  CAPS_ON_WIRE_AT,
  type CapSample,
} from './cap-census-read.ts';

//region Cap census rule
// The completion cap rule applied to samples `cap-census-read.ts` collected,
// per roster seat and per provider; see that module for why there are two
// readings.

/**
 Percentile the rule reads, as a fraction.
 */
const P99 = 0.99;

/**
 Share of a provider's capped calls past which the census flags the cap, the
 2026-09-09 table's own "under one percent".
 */
const CUT_SHARE_FLAG = 0.01;

/**
 What a seat's row asks a reader to look at: a card still on the pooled 99th
 placeholder though the rule now reads its calls, a rule reading off the
 card's cap, or a provider cutting more than the table's share.

 @example
 ```ts
 const flag: CapFlag = 'rule-off-card';
 ```
 */
export type CapFlag = 'placeholder-with-calls' | 'rule-off-card' | 'cuts-over-share';

/**
 What one provider's calls of one seat say.

 @example
 ```ts
 const reading: ProviderCapReading = { provider: 'hyper', calls: 4_062, p99: 13_082, sinceCaps: 4_062, atCap: 75, atCapWithContent: 1, atCapNoContent: 74, atCapUnpaired: 0, };
 ```
 */
export type ProviderCapReading = {
  /**
   Provider that served them.
   */
  readonly provider: CapSample['provider'];

  /**
   Completed calls, all time.
   */
  readonly calls: number;

  /**
   Their 99th percentile of completion tokens.
   */
  readonly p99: number;

  /**
   Completed calls since the caps went on the wire.
   */
  readonly sinceCaps: number;

  /**
   Of those, calls that ran to the card's cap or past it.
   */
  readonly atCap: number;

  /**
   Calls at the cap whose stream delivered content: answers the cap cut.
   */
  readonly atCapWithContent: number;

  /**
   Calls at the cap whose stream delivered none: reasoning runaways.
   */
  readonly atCapNoContent: number;

  /**
   Calls at the cap no stream line paired with.
   */
  readonly atCapUnpaired: number;
};

/**
 What the census says about one roster seat.

 @example
 ```ts
 const flagged = rows.filter(function placeheld(row,) { return row.placeholder && (row.ruleCap !== 'too-few-calls'); },);
 ```
 */
export type CapCensusRow = {
  /**
   Seat these calls went to.
   */
  readonly modelId: RosterModelId;

  /**
   Cap the card carries today.
   */
  readonly cardCap: number;

  /**
   Whether the card names the pooled 99th percentile, the placeholder for a
   model with no calls of its own; the pooled 90th is a measured floor, not a
   placeholder.
   */
  readonly placeholder: boolean;

  /**
   The rule's reading over these calls, or that no provider held enough.
   */
  readonly ruleCap: number | 'too-few-calls';

  /**
   One reading per provider that served the seat.
   */
  readonly providers: readonly ProviderCapReading[];
};

/**
 Everything the census read.

 @example
 ```ts
 const { rows, offRoster, } = capCensus({ samples, },);
 ```
 */
export type CapCensus = {
  /**
   One row per roster seat with at least one call, in roster order.
   */
  readonly rows: readonly CapCensusRow[];

  /**
   Calls on served ids no card names: retired models and renamed ids.
   */
  readonly offRoster: number;
};

/**
 Roster seat each provider's served id belongs to, keyed `provider id`.
 */
const SEAT_OF_SERVED: ReadonlyMap<string, RosterModelId> = new Map(
  ROSTER_MODEL_IDS.flatMap(function servedIdsOf(modelId,): readonly (readonly [
    string,
    RosterModelId
  ])[] {
    /**
     The seat's card.
     */
    const card = MODEL_CARDS[modelId];
    return [
      ...((card.synthetic === undefined) ? [] : [`synthetic ${card.synthetic
        .id}`,]),
      ...((card.hyper === undefined) ? [] : [`hyper ${card.hyper
        .id}`,]),
      ...((card.openrouter === undefined) ? [] : [`openrouter ${card.openrouter
        .id}`,]),
      ...((card.bedrock === undefined) ? [] : [`bedrock ${card.bedrock
        .id}`,]),
      ...((card.deepseek === undefined) ? [] : [`deepseek ${card.deepseek
        .id}`,]),
      ...((card.xiaomi === undefined) ? [] : [`xiaomi ${card.xiaomi
        .id}`,]),
      ...((card.anthropic === undefined) ? [] : [`anthropic ${card.anthropic
        .id}`,]),
    ].map(function keyed(key,): readonly [
      string,
      RosterModelId
    ] {
      return [
        key,
        modelId,
      ];
    },);
  },),
);

/**
 Reads a 99th percentile off counts.

 @param values - completion counts, any order

 @returns Value at the 99th percentile, the nearest-rank way the scratch
 measurements of 2026-09-28 read it

 @example
 ```ts
 const p99 = p99Of({ values: [1, 2, 3,], },);
 ```
 */
function p99Of({ values, }: { readonly values: readonly number[]; },): number {
  /**
   Values in ascending order.
   */
  const sorted = values.toSorted(function ascending(
    left,
    right,
  ): number {
    return left - right;
  },);
  return nonNullishOrThrow(sorted[Math.min(
    sorted.length - 1,
    Math.floor(P99 * sorted.length,),
  )],);
}

/**
 Reads one provider's calls of one seat.

 @param provider - provider that served them

 @param samples - its calls

 @param cardCap - cap the seat's card carries

 @returns The provider's reading

 @example
 ```ts
 const reading = providerReading({ provider: 'hyper', samples, cardCap: 13_082, },);
 ```
 */
function providerReading(
  {
    provider,
    samples,
    cardCap,
  }: {
    readonly provider: CapSample['provider'];
    readonly samples: readonly CapSample[];
    readonly cardCap: number;
  },
): ProviderCapReading {
  /**
   Calls since the caps went on the wire.
   */
  const since = samples.filter(function capped(sample,): boolean {
    return sample.at >= CAPS_ON_WIRE_AT;
  },);

  /**
   Of those, the calls that ran to the cap.
   */
  const atCap = since.filter(function ranToCap(sample,): boolean {
    return sample.completion >= cardCap;
  },);
  return {
    provider,
    calls: samples.length,
    p99: p99Of({
      values: samples.map(function completionOf(sample,): number {
        return sample.completion;
      },),
    },),
    sinceCaps: since.length,
    atCap: atCap.length,
    atCapWithContent: atCap.filter(function answered(sample,): boolean {
      return (sample.content !== 'unpaired') && (sample.content > 0);
    },)
      .length,
    atCapNoContent: atCap.filter(function ranAway(sample,): boolean {
      return sample.content === 0;
    },)
      .length,
    atCapUnpaired: atCap.filter(function unpaired(sample,): boolean {
      return sample.content === 'unpaired';
    },)
      .length,
  };
}

/**
 Applies the cap rule to every roster seat the samples reach.

 @param samples - completed calls from pass-run logs

 @returns A row per seat, and how many calls went to ids no card names

 @example
 ```ts
 const census = capCensus({ samples, },);
 ```
 */
export function capCensus({ samples, }: { readonly samples: readonly CapSample[]; },): CapCensus {
  /**
   Seat of each sample, or that no card names its id.
   */
  const seated = samples.map(function seatOf(sample,): {
    readonly sample: CapSample;
    readonly modelId: RosterModelId | 'off-roster';
  } {
    return {
      sample,
      modelId: SEAT_OF_SERVED.get(`${sample.provider} ${sample.model}`,) ?? 'off-roster',
    };
  },);
  return {
    rows: ROSTER_MODEL_IDS.flatMap(function rowOf(modelId,): readonly CapCensusRow[] {
      /**
       This seat's calls.
       */
      const own = seated
        .filter(function isOwn(entry,): boolean {
          return entry.modelId === modelId;
        },)
        .map(function sampleOf(entry,): CapSample {
          return entry.sample;
        },);
      if (own.length === 0)
        return [];

      /**
       Cap the card carries.
       */
      const cardCap = COMPLETION_CAP[modelId];

      /**
       One reading per provider that served it.
       */
      const providers = textsInCodePointOrder({ texts: [...new Set(own.map(function providerOf(sample,) {
        return sample.provider;
      },),),], },)
        .map(function readingOf(provider,): ProviderCapReading {
          return providerReading({
            provider,
            samples: own.filter(function servedBy(sample,): boolean {
              return sample.provider === provider;
            },),
            cardCap,
          },);
        },);

      /**
       The 99th percentiles of providers holding enough calls.
       */
      const eligible = providers
        .filter(function enough(reading,): boolean {
          return reading.calls >= MIN_PROVIDER_CALLS;
        },)
        .map(function p99(reading,): number {
          return reading.p99;
        },);
      return [{
        modelId,
        cardCap,
        // THE POOLED 99TH IS THE PLACEHOLDER, for a model with no calls of
        // its own; the pooled 90th is a measured result, a model whose own
        // 99th fell under the floor (`completion-cap.ts`).
        placeholder: MODEL_CARDS[modelId]
          .completionCap
          === 'pooled-p99',
        ruleCap: (eligible.length === 0) ? 'too-few-calls' : Math.max(
          POOLED_P90,
          ...eligible,
        ),
        providers,
      },];
    },),
    offRoster: seated.filter(function isOff(entry,): boolean {
      return entry.modelId === 'off-roster';
    },)
      .length,
  };
}

/**
 Names what a seat's row asks a reader to look at.

 @param row - the seat's census row

 @returns Flags in report order, empty where the card and its calls agree

 @example
 ```ts
 const flags = capFlagsOf({ row, },);
 ```
 */
export function capFlagsOf({ row, }: { readonly row: CapCensusRow; },): readonly CapFlag[] {
  /**
   Whether any provider ran to the cap on more than the table's share.
   */
  const cutsOver = row.providers
    .some(function overShare(reading,): boolean {
    // ENOUGH CALLS TO READ A SHARE FROM, by the rule's own threshold: one cut
    // in twenty calls is not a cap cutting five percent.
      return (reading.sinceCaps >= MIN_PROVIDER_CALLS) && ((reading.atCap / reading.sinceCaps) > CUT_SHARE_FLAG);
    },);
  return [
    ...((row.placeholder && (row.ruleCap !== 'too-few-calls')) ? ['placeholder-with-calls' as const,] : []),
    ...(((row.ruleCap !== 'too-few-calls') && (row.ruleCap !== row.cardCap)) ? ['rule-off-card' as const,] : []),
    ...(cutsOver ? ['cuts-over-share' as const,] : []),
  ];
}

//endregion Cap census rule
