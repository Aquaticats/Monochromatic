/**
 Type definitions for multi-provider image diff operations.
 Separated from {@link ./client.multi.ts} to stay within the max-lines limit.
 
 @module
 */
import type {
  BatchEmbeddingResult,
  ComparisonResult,
  EmbeddingResult,
  Provider,
} from './types.ts';

/**
 Result from a single provider in a multi-provider comparison.
 
 @example
 ```ts
 for (const entry of results) {
   console.log(`${entry.provider}: similarity=${entry.result.similarity}`);
 }
 ```
 */
export type MultiProviderComparisonEntry = {
  /**
   Provider that produced this result.
   */
  readonly provider: Provider;
  /**
   Comparison result from this provider.
   */
  readonly result: ComparisonResult;
};

/**
 Result from a single provider in a multi-provider embed call.
 
 @example
 ```ts
 for (const entry of results) {
   console.log(`${entry.provider}: ${entry.result.embedding.length} dims`);
 }
 ```
 */
export type MultiProviderEmbedEntry = {
  /**
   Provider that produced this result.
   */
  readonly provider: Provider;
  /**
   Embedding result from this provider.
   */
  readonly result: EmbeddingResult;
};

/**
 Result from a single provider in a multi-provider batch embed call.
 
 @example
 ```ts
 for (const entry of results) {
   console.log(`${entry.provider}: ${entry.result.embeddings.length} embeddings`);
 }
 ```
 */
export type MultiProviderBatchEmbedEntry = {
  /**
   Provider that produced this result.
   */
  readonly provider: Provider;
  /**
   Batch embedding result from this provider.
   */
  readonly result: BatchEmbeddingResult;
};
