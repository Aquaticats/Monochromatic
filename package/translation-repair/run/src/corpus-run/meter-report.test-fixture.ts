import type { MeterSample, } from '../../dist/final/node/index.mjs';

//region Meter report fixture
// Log lines a cat-themed run wrote when it read its provider meters, as the
// meter report reads them: the logger's own prefix, then the record the budget
// layer writes, a state for each provider and then the numbers behind them.

/**
 What a reading says of each provider and its numbers.

 @example
 ```ts
 const reading: MeterReading = { synthetic: 'wet', hyper: 'dry', levels: ['hyperBalance=0',], };
 ```
 */
type MeterReading = {
  /**
   What the first provider's meter said.
   */
  readonly synthetic: string;

  /**
   What the second provider's meter said.
   */
  readonly hyper: string;

  /**
   What the fifth provider's meter said, left off a line older than it.
   */
  readonly neuralwatt?: string;

  /**
   What the third provider's meter said, left off a line older than it.
   */
  readonly openrouter?: string;

  /**
   What the fourth provider's meter said, left off a line older than it.
   */
  readonly bedrock?: string;

  /**
   Numbers beside the states, in the order written.
   */
  readonly levels: readonly string[];
};

/**
 Writes a line carrying a meter record.

 @param stamp - instant the logger stamped the line with

 @param reading - what the record says

 @returns The whole log line

 @example
 ```ts
 const line = meterLine({ stamp: '2026-08-25T10:00:00.000Z', reading: { synthetic: 'wet', hyper: 'wet', levels: [], }, },);
 ```
 */
export function meterLine(
  {
    stamp,
    reading,
  }: {
    readonly stamp: string;
    readonly reading: MeterReading;
  },
): string {
  /**
   The state fields, in the order the budget layer writes them, leaving off
   a provider the record predates.
   */
  const states = [
    ...((reading.neuralwatt === undefined) ? [] : [`neuralwatt=${reading.neuralwatt}`,]),
    `synthetic=${reading.synthetic}`,
    `hyper=${reading.hyper}`,
    ...((reading.openrouter === undefined) ? [] : [`openrouter=${reading.openrouter}`,]),
    ...((reading.bedrock === undefined) ? [] : [`bedrock=${reading.bedrock}`,]),
  ];

  return `[info] [${stamp}] [translation-repair] [takeReading] METERS ${
    [
      ...states,
      ...reading.levels,
    ].join(' ',)
  }`;
}

/**
 A reading when every meter found budget.
 */
export const EVERYTHING_WET: string = meterLine({
  stamp: '2026-08-25T10:00:00.000Z',
  reading: {
    synthetic: 'wet',
    hyper: 'wet',
    levels: ['hyperBalance=2497',],
  },
},);

/**
 A reading half an hour later when the first provider's meter found none.
 */
export const SYNTHETIC_DRY: string = meterLine({
  stamp: '2026-08-25T10:30:00.000Z',
  reading: {
    synthetic: 'dry',
    hyper: 'wet',
    levels: ['hyperBalance=2000',],
  },
},);

/**
 A reading an hour after the first when both found none.
 */
export const BOTH_DRY: string = meterLine({
  stamp: '2026-08-25T11:00:00.000Z',
  reading: {
    synthetic: 'dry',
    hyper: 'dry',
    levels: ['hyperBalance=0',],
  },
},);

/**
 A reading two hours after the first when both found budget again.
 */
export const BOTH_WET_AGAIN: string = meterLine({
  stamp: '2026-08-25T12:00:00.000Z',
  reading: {
    synthetic: 'wet',
    hyper: 'wet',
    levels: ['hyperBalance=500',],
  },
},);

/**
 What the report says once, under its window, about when readings are taken.
 */
export const ASKING_NOTE: string = '  READINGS HAPPEN WHEN A RUN ASKS FOR ONE, so this window is dense while work ran and '
  + 'empty otherwise. Every figure in this report is availability WHEN WE WERE ASKING.';

/**
 What the report says of a provider whose readings carry no numbers.
 */
export const LEVEL_NOT_RECORDED: string = '  level: NOT RECORDED. These readings predate the meter numbers being written '
  + 'down, so a dry one here cannot be told from a threshold that was wrong about a budget that was fine';

/**
 What the report says when its logs carry no reading.
 */
export const NOTHING_RECORDED_LINE: string = '  NOTHING RECORDED. These logs carry no availability reading. Runs written '
  + 'before the reading was promoted out of debug level have none, so read a log from a pass or a '
  + '`budget-sample` taken after that landed.';

/**
 The fields of a reading a case changes, each optional because a case names
 only the ones it is about.
 */
type SampleOverrides = {
  /**
   Epoch milliseconds the reading was taken at.
   */
  readonly at?: MeterSample['at'];

  /**
   What the first provider's meter said.
   */
  readonly synthetic?: MeterSample['synthetic'];

  /**
   What the second provider's meter said.
   */
  readonly hyper?: MeterSample['hyper'];

  /**
   What the third provider's meter said.
   */
  readonly openrouter?: MeterSample['openrouter'];

  /**
   What the fourth provider's meter said.
   */
  readonly bedrock?: MeterSample['bedrock'];

  /**
   What the DeepSeek meter said.
   */
  readonly deepseek?: MeterSample['deepseek'];

  /**
   What the Xiaomi meter said.
   */
  readonly xiaomi?: MeterSample['xiaomi'];

  /**
   What the Claude key's meter said.
   */
  readonly anthropic?: MeterSample['anthropic'];

  /**
   Numbers beside the states.
   */
  readonly levels?: MeterSample['levels'];
};

/**
 Instant every sample of a case is measured from, as epoch milliseconds: 2026-08-25T10:00:00.000Z, the stamp of the first fixture line.
 */
export const BASE_AT = 1_787_652_000_000;

/**
 Builds one reading of the meters, every provider wet and no numbers, with the
 fields a case names changed.

 @param overrides - fields that differ from that reading

 @returns The sample

 @example
 ```ts
 const sample = sampleOf({ overrides: { hyper: 'dry', }, },);
 ```
 */
export function sampleOf({ overrides, }: { readonly overrides: SampleOverrides; },): MeterSample {
  return {
    at: BASE_AT,
    neuralwatt: 'absent',
    synthetic: 'wet',
    hyper: 'wet',
    openrouter: 'wet',
    bedrock: 'wet',
    deepseek: 'absent',
    xiaomi: 'absent',
    anthropic: 'absent',
    levels: [],
    ...overrides,
  };
}

//endregion Meter report fixture
