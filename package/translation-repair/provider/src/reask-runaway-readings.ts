import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Reasoning runaway readings
// THE ORDER A REASONING RUNAWAY IS RE-ASKED IN (ledger B1075, the owner's answer of 2026-10-30, "Lowest runaway
// share (Recommended)"): the other models by the share of their calls that ran away, pooled over providers, lowest
// first.
//
// WHAT "RAN AWAY" MEANS HERE IS WHAT THE CAP CENSUS COUNTS: a completed call that reached the cap its card sends
// (`completionCap`) and streamed no content (`N with none` in `cap-census` output, "since the caps"). A call that
// reached the cap with content is a cut answer and not a runaway; a call the census could not pair with its stream
// line (`unpaired`, `ambiguous`) is counted among the calls and among neither the ones with content nor the ones with
// none (none unpaired and 4 ambiguous in this reading: GLM-5.3-Flash on Synthetic 2, Qwen3.8-27B on Synthetic 1,
// minimax-m3 on Hyper 1).
//
// THE READING IS THE CENSUS OF 00:18 UTC ON 2026-10-30, and its limits are these:
//
// - Command: `mise run //package/translation-repair/run:cap-census -- ~/temp/agent`, with the pairing of ledger B1070
//   and the seats of B1071 (the run of the w382 fixer, kept as `census-after-2.out`). It read 113,645 logs, 447 of
//   them pass runs, and 582,710 completed calls.
// - THE SAME COMMAND RE-RUN AT 01:41 UTC ON 2026-10-31 READ 106,377 logs, 40 pass-run logs and 0 completed calls: the
//   `corpus-pass-<name>.log` files of the runs from `XingZ6016` to `XingZ6022` are no longer under the agent root
//   (their run directories are). The numbers below therefore cannot be reproduced from the logs now on disk, and the
//   next census over pass logs that survive is the one that should replace them (ledger B1078).
// - The census seats a call by the model id the provider served it under, with the retired DeepSeek ids of ledger
//   B1071 counted under the DeepSeek seat; 53,706 calls on ids no card names are in no row.
// - Claude cards are not here (the Claude route is never a re-ask target), nor is the decision-only seat.
//
// A MODEL WITH FEWER THAN `MIN_CALLS_TO_MEASURE` CALLS, OR NO ROW, IS PLACED AFTER EVERY MEASURED MODEL, in roster
// order. The figure is the completion-cap rule's own floor (`completion-cap.ts`: a provider needs 100 calls before
// its percentile is read). A share read from fewer calls says less than any measured share, so it is not allowed to
// outrank one; and no measured model is made to wait behind one whose runaways are not known. A model enters the
// table when a census gives it 100 calls since the caps.

/**
 Calls a model needs since the caps before its share of runaways is read.

 @example
 ```ts
 const measured = reading.calls >= MIN_CALLS_TO_MEASURE;
 ```
 */
export const MIN_CALLS_TO_MEASURE = 100;

/**
 One model's calls since the caps, pooled over providers, and how many of them ran to the cap with no content.

 @example
 ```ts
 const reading: RunawayReading = { modelId: 'minimax-m3', calls: 73_956, runaways: 253, };
 ```
 */
export type RunawayReading = {
  /**
   Roster model the calls were made under.
   */
  readonly modelId: RosterModelId;

  /**
   Completed calls since the caps went on the wire, over every provider that served the model.
   */
  readonly calls: number;

  /**
   Those calls that reached the cap and streamed no content, the census's `with none`.
   */
  readonly runaways: number;
};

/**
 Every chat model's reading in the census of 00:18 UTC on 2026-10-30, in roster order.

 Per provider (calls since the caps, at the cap with none): GLM-5.3-Flash Hyper 2,095 and 10, NeuralWatt 1,758 and 19,
 OpenRouter 21,498 and 225, Synthetic 2,062 and 95; Qwen3.8-27B Hyper 917 and 0, Synthetic 15,423 and 194; Kimi-K3 Hyper
 1,990 and 0, OpenRouter 20 and 1, Synthetic 13,192 and 2; gpt-oss-120b Bedrock 35,021 and 22, Synthetic 5,804 and 8;
 minimax-m3 Hyper 7,297 and 252, OpenRouter 66,659 and 1; gemma-4-26b-a4b-it Bedrock 68,147 and 0, Hyper 686 and 0,
 OpenRouter 2,014 and 0; glm-5.3 Hyper 3,202 and 23; deepseek-v4.1-flash direct 63 and 7, Hyper 7,120 and 137,
 NeuralWatt 5,310 and 60, OpenRouter 65,607 and 234; gemma-4-e2b Bedrock 67,481 and 0; gemma-4-31b Bedrock 214 and 0;
 mercury-2.5 OpenRouter 59,027 and 0; mimo-v2.6-flash OpenRouter 1,362 and 0, Xiaomi 467 and 1.

 @example
 ```ts
 const deepseek = RUNAWAY_READINGS.find(function isDeepseek(reading,) { return reading.modelId === 'deepseek-v4.1-flash'; },);
 ```
 */
export const RUNAWAY_READINGS: readonly RunawayReading[] = [
  {
    modelId: 'hf:zai-org/GLM-5.3-Flash',
    calls: 27_413,
    runaways: 349,
  },
  {
    modelId: 'hf:Qwen/Qwen3.8-27B',
    calls: 16_340,
    runaways: 194,
  },
  {
    modelId: 'hf:moonshotai/Kimi-K3',
    calls: 15_202,
    runaways: 3,
  },
  {
    modelId: 'hf:openai/gpt-oss-120b',
    calls: 40_825,
    runaways: 30,
  },
  {
    modelId: 'minimax-m3',
    calls: 73_956,
    runaways: 253,
  },
  {
    modelId: 'gemma-4-26b-a4b-it',
    calls: 70_847,
    runaways: 0,
  },
  {
    modelId: 'glm-5.3',
    calls: 3_202,
    runaways: 23,
  },
  {
    modelId: 'deepseek-v4.1-flash',
    calls: 78_100,
    runaways: 438,
  },
  {
    modelId: 'google.gemma-4-e2b',
    calls: 67_481,
    runaways: 0,
  },
  {
    modelId: 'google.gemma-4-31b',
    calls: 214,
    runaways: 0,
  },
  {
    modelId: 'inception/mercury-2.5',
    calls: 59_027,
    runaways: 0,
  },
  {
    modelId: 'xiaomi/mimo-v2.6-flash',
    calls: 1_829,
    runaways: 1,
  },
];

/**
 The share of a model's calls that ran away, where enough calls were read to say.

 @param modelId - model to look up

 @param readings - table to read, the census's by default

 @returns Runaways over calls as a fraction, or the words `unmeasured` where the model has no row or fewer calls than
 can be measured

 @example
 ```ts
 const share = runawayShareOf({ modelId: 'minimax-m3', },);
 ```
 */
export function runawayShareOf(
  {
    modelId,
    readings = RUNAWAY_READINGS,
  }: {
    readonly modelId: RosterModelId;
    readonly readings?: readonly RunawayReading[];
  },
): number | 'unmeasured' {
  /**
   The row for this model, if the table has one.
   */
  const reading = readings.find(function isModel(row,): boolean {
    return row.modelId === modelId;
  },);
  if ((reading === undefined) || (reading.calls < MIN_CALLS_TO_MEASURE))
    return 'unmeasured';
  return reading.runaways / reading.calls;
}

//endregion Reasoning runaway readings
