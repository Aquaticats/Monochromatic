import type { TransportReply, } from '@monochromatic-dev/translation-repair-provider-stream';

//region NeuralWatt models listing test support
// The `GET /v1/models` listing as a case scripts it, with the names of 2026-10-20.

/**
 The backend model DeepSeek V4.1 Flash and the tracking alias `nw-flash` both list as `metadata.huggingface_id`.
 */
export const DEEPSEEK_LISTED_AS = 'deepseek-ai/DeepSeek-V4.1-Flash-1017';

/**
 The backend model the large tracking alias lists, which the free energy does not cover.
 */
export const KIMI_LISTED_AS = 'moonshotai/Kimi-K3';

/**
 One entry of the `GET /v1/models` listing, with the fields these cases read.

 @param id - the name a request uses

 @param listedAs - its `metadata.huggingface_id`

 @returns The entry

 @example
 ```ts
 const entry = modelEntryOf({ id: 'nw-flash', listedAs: DEEPSEEK_LISTED_AS, },);
 ```
 */
export function modelEntryOf(
  {
    id,
    listedAs,
  }: {
    readonly id: string;
    readonly listedAs: string
  },
): Readonly<Record<string, unknown>> {
  return {
    id,
    object: 'model',
    owned_by: 'neuralwatt',
    metadata: { huggingface_id: listedAs, },
  };
}

/**
 The listing as it stood on 2026-10-20 for the names these cases use: the two covered models, the flash and large
 aliases with their flex twins, and a model nothing here covers.
 */
export const LISTED_MODELS: readonly Readonly<Record<string, unknown>>[] = [
  modelEntryOf({
    id: 'deepseek-v4.1-flash',
    listedAs: DEEPSEEK_LISTED_AS,
  },),
  modelEntryOf({
    id: 'glm-5.3-flash',
    listedAs: 'zai-org/GLM-5.3-Flash',
  },),
  modelEntryOf({
    id: 'nw-flash',
    listedAs: DEEPSEEK_LISTED_AS,
  },),
  modelEntryOf({
    id: 'nw-flash-flex',
    listedAs: DEEPSEEK_LISTED_AS,
  },),
  modelEntryOf({
    id: 'nw-large',
    listedAs: KIMI_LISTED_AS,
  },),
  modelEntryOf({
    id: 'nw-large-flex',
    listedAs: KIMI_LISTED_AS,
  },),
  modelEntryOf({
    id: 'kimi-k3',
    listedAs: KIMI_LISTED_AS,
  },),
];

/**
 A `GET /v1/models` reply listing the given entries.

 @param entries - the listing

 @returns The reply

 @example
 ```ts
 const models = [modelsReply({ entries: LISTED_MODELS, },),];
 ```
 */
export function modelsReply(
  { entries, }: { readonly entries: readonly Readonly<Record<string, unknown>>[]; },
): TransportReply {
  return {
    status: 200,
    bodyText: JSON.stringify({
      object: 'list',
      data: entries,
    },),
  };
}

//endregion NeuralWatt models listing test support
