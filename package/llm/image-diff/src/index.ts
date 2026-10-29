/**
 Perceptual image difference comparison using multimodal embeddings
 from Voyage AI and Google Gemini.
 
 By default, all available providers are called concurrently with their
 latest models. Use the `provider` config field to select a specific backend.
 
 @example
 ```ts
 import { compareAll } from '@monochromatic-dev/llm-image-diff';
 
 // Compare using all providers (Voyage + Gemini)
 const results = await compareAll({
   imageA: { path: './before.png' },
   imageB: { path: './after.png' },
 });
 for (const { provider, result } of results) {
   console.log(`${provider}: similarity=${result.similarity}`);
 }
 ```
 
 @example
 ```ts
 import { compare } from '@monochromatic-dev/llm-image-diff';
 
 // Compare using a specific provider
 const result = await compare({
   imageA: { path: './before.png' },
   imageB: { path: './after.png' },
   config: { provider: 'gemini' },
 });
 ```
 
 @packageDocumentation
 */

export { compareAll, } from './client.multi.compare.ts';
export {
  embedAll,
  embedBatchAll,
} from './client.multi.ts';
export type {
  MultiProviderBatchEmbedEntry,
  MultiProviderComparisonEntry,
  MultiProviderEmbedEntry,
} from './client.multi.types.ts';
export {
  compare,
  embed,
  embedBatch,
} from './client.ts';
export { ABSENT, } from './describe.absent.ts';
export { describeImageDifference, } from './describe.ts';
export { geminiProvider, } from './gemini.ts';
export {
  cosineSimilarity,
  dotProduct,
} from './similarity.ts';
export type {
  BatchEmbeddingResult,
  ComparisonResult,
  EmbeddingModel,
  EmbeddingProvider,
  EmbeddingResult,
  GeminiModel,
  ImageBase64,
  ImageBuffer,
  ImageDiffConfig,
  ImageFormat,
  ImageInput,
  ImagePath,
  ImageUrl,
  Provider,
  VoyageModel,
} from './types.ts';
export { voyageProvider, } from './voyage.ts';
