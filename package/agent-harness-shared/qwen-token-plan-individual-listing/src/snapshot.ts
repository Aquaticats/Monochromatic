/**
 Checked-in Qwen Token Plan individual listing snapshot: the listing
 membership served while the bounded fetch cannot reach the endpoint. @module
 */

import type { QwenTokenPlanListingEntry, } from './listing.ts';

//region Public API

/**
 Listing membership snapshot fetched from GET
 `https://token-plan.maas.qwencloudapi.com/compatible-mode/v1/models` at
 2026-11-11T05:00:34-05:00: every id the endpoint returned then, in endpoint
 order, with the `created` stamp each entry carried. Facts come from
 `qwenTokenPlanIndividualFacts`, so this snapshot pins membership only.
 Regenerate it with the `refresh-snapshot` mise task, which prints the drift
 the rewrite would change.
 */
export const qwenTokenPlanIndividualListingSnapshot: {
  readonly fetchedAt: string;
  readonly entries: readonly QwenTokenPlanListingEntry[];
} = {
  fetchedAt: '2026-11-11T05:00:34-05:00',
  entries: [
    {
      id: 'qwen3.7-max',
      created: 1_779_367_398,
    },
    {
      id: 'qwen3.7-plus',
      created: 1_780_806_507,
    },
    {
      id: 'qwen3.6-flash',
      created: 1_779_173_747,
    },
    {
      id: 'glm-5.2',
      created: 1_781_605_418,
    },
    {
      id: 'deepseek-v4-pro',
      created: 1_779_173_747,
    },
    {
      id: 'wan2.7-image',
      created: 1_779_173_747,
    },
    {
      id: 'wan2.7-image-pro',
      created: 1_779_173_747,
    },
    {
      id: 'qwen-audio-3.0-tts-plus',
      created: 1_785_248_243,
    },
    {
      id: 'deepseek-v4-flash-0731',
      created: 1_785_513_696,
    },
    {
      id: 'qwen3.8-max',
      created: 1_785_722_409,
    },
    {
      id: 'qwen-audio-3.0-realtime-plus',
      created: 1_785_928_192,
    },
    {
      id: 'deepseek-v4-pro-0813',
      created: 1_786_615_063,
    },
    {
      id: 'qwen3.8-flash',
      created: 1_787_731_683,
    },
    {
      id: 'deepseek-v4.1-flash',
      created: 1_789_351_565,
    },
    {
      id: 'glm-5.3',
      created: 1_789_541_875,
    },
    {
      id: 'auto',
      created: 1_789_973_043,
    },
    {
      id: 'wan3.0-video',
      created: 1_791_856_789,
    },
    {
      id: 'deepseek-v4.1-flash-1017',
      created: 1_792_388_476,
    },
    {
      id: 'qwen-image-3.0-pro',
      created: 1_792_656_638,
    },
    {
      id: 'qwen-audio-3.0-asr-flash',
      created: 1_792_656_638,
    },
    {
      id: 'qwen4-max-preview',
      created: 1_794_362_561,
    },
  ],
};

//endregion Public API
