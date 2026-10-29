// oxlint-disable require-await -- Promise.allSettled values require type assertions; async functions return provider promises directly
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { ABSENT, } from './describe.absent.ts';
import { describeImageDifference, } from './describe.ts';
import { geminiProvider, } from './gemini.ts';
import { dotProduct, } from './similarity.ts';
import type {
  BatchEmbeddingResult,
  ComparisonResult,
  EmbeddingProvider,
  EmbeddingResult,
  ImageDiffConfig,
  ImageInput,
  Provider,
} from './types.ts';
import { voyageProvider, } from './voyage.ts';

/**
 Logger root for image-diff after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'image-diff', },);

/**
 Registry mapping provider names to their implementations.
 */
const PROVIDERS: Record<Provider, EmbeddingProvider> = {
  voyage: voyageProvider,
  gemini: geminiProvider,
};

/**
 Resolve a single provider from config.
 
 @param provider - provider name
 
 @returns provider implementation
 
 @example
 ```ts
 const p = getProvider('voyage');
 ```
 */
function getProvider(provider: Provider,): EmbeddingProvider {
  return PROVIDERS[provider];
}

/**
 Compute a multimodal embedding for a single image via the specified provider.
 When no provider is specified, defaults to Voyage.
 
 @param input - image to embed, in any supported format
 
 @param config - optional client configuration (provider, API key, model)
 
 @returns embedding vector and usage metadata
 
 @example
 ```ts
 const { embedding } = await embed({ input: { path: './photo.png' } });
 const geminiResult = await embed({
   input: { path: './photo.png' },
   config: { provider: 'gemini' },
 });
 ```
 */
export async function embed({
  input,
  config = {},
}: {
  readonly input: ImageInput;
  readonly config?: ImageDiffConfig;
},): Promise<EmbeddingResult> {
  /**
   Selected provider name; defaults to Voyage when unspecified by the caller.
   */
  const provider = config.provider
    ?? 'voyage';
  return getProvider(provider,)
    .embed({
      input,
      config,
    },);
}

/**
 Compute multimodal embeddings for multiple images in a single batch API call.
 When no provider is specified, defaults to Voyage.
 
 @param inputs - array of images to embed
 
 @param config - optional client configuration (provider, API key, model)
 
 @returns embedding vectors (in input order) and aggregate usage metadata
 
 @example
 ```ts
 const { embeddings } = await embedBatch({
   inputs: [
     { path: './before.png' },
     { path: './after.png' },
   ],
 });
 ```
 */
export async function embedBatch({
  inputs,
  config = {},
}: {
  readonly inputs: readonly ImageInput[];
  readonly config?: ImageDiffConfig;
},): Promise<BatchEmbeddingResult> {
  /**
   Selected provider name; defaults to Voyage when unspecified by the caller.
   */
  const provider = config.provider
    ?? 'voyage';
  return getProvider(provider,)
    .embedBatch({
      inputs,
      config,
    },);
}

/**
 Embedding-only comparison without the natural-language description.
 Used internally by {@link compare} and by the multi-provider comparison
 to avoid duplicate description calls.
 
 @param imageA - first image
 
 @param imageB - second image
 
 @param config - client configuration (provider, API key, model)
 
 @returns similarity, distance, and both embedding vectors (no description)
 
 @example
 ```ts
 const result = await compareEmbeddings({
   imageA: { path: 'a.png' },
   imageB: { path: 'b.png' },
   config: { provider: 'voyage' },
 });
 // result.similarity, result.distance, result.embeddings
 ```
 */
export async function compareEmbeddings({
  imageA,
  imageB,
  config = {},
}: {
  readonly imageA: ImageInput;
  readonly imageB: ImageInput;
  readonly config?: ImageDiffConfig;
},): Promise<Omit<ComparisonResult, 'description'>> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: compareEmbeddings.name,
    l,
  },);
  /**
   Selected provider name; defaults to Voyage when unspecified by the caller.
   */
  const provider = config.provider
    ?? 'voyage';
  rl.debug(`comparing embeddings via ${provider}`,);

  /**
   Both image embeddings produced in a single batch call so the API charges and round-trips just once.
   */
  const { embeddings, } = await getProvider(provider,)
    .embedBatch({
      inputs: [
        imageA,
        imageB,
      ],
      config,
    },);
  /**
   Pair of embeddings destructured for the dot-product call; guarded against a malformed batch result.
   */
  const [embeddingA, embeddingB,] = embeddings;
  if ((embeddingA === undefined) || (embeddingB === undefined))
    throw new Error('Expected exactly 2 embeddings from batch call',);

  /**
   Cosine-equivalent similarity (embeddings are unit vectors) for the two images.
   */
  const similarity = dotProduct({
    a: embeddingA,
    b: embeddingB,
  },);
  /**
   Perceptual distance derived from similarity; `0` when identical, `1` when orthogonal.
   */
  const distance = 1 - similarity;

  rl.debug(
    `embedding comparison complete (${provider}): similarity=${
      String(similarity,)
    }, distance=${String(distance,)}`,
  );

  return {
    similarity,
    distance,
    embeddingA,
    embeddingB,
  };
}

/**
 Compare two images using a single provider by computing their multimodal embeddings
 and returning cosine similarity, perceptual distance, and a natural-language
 description of the visual differences via Gemini 3.1 Pro Preview.
 When no provider is specified, defaults to Voyage.
 
 @param imageA - first image
 
 @param imageB - second image
 
 @param config - optional client configuration (provider, API key, model)
 
 @returns similarity score, distance, both embedding vectors, and description
 
 @throws when the description call resolves to {@link ABSENT} (no OpenRouter API key configured)
 
 @example
 ```ts
 const result = await compare({
   imageA: { path: './before.png' },
   imageB: { path: './after.png' },
 });
 console.log(result.description);
 ```
 */
export async function compare({
  imageA,
  imageB,
  config = {},
}: {
  readonly imageA: ImageInput;
  readonly imageB: ImageInput;
  readonly config?: ImageDiffConfig;
},): Promise<ComparisonResult> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: compare.name,
    l,
  },);
  rl.debug('running embedding comparison and description concurrently',);

  /**
   Tuple of [embedding comparison, textual description] resolved concurrently to halve wall time.
   */
  const results = await Promise.all([
    compareEmbeddings({
      imageA,
      imageB,
      config,
    },),
    describeImageDifference({
      imageA,
      imageB,
    },),
  ],);
  /**
   Tuple destructured for separate handling; description-absence triggers an explicit missing-key error.
   */
  const [embeddingResult, description,] = results;
  if (description === ABSENT) {
    throw new Error(
      'OpenRouter API key is required for image comparison. Set IMAGE_DIFF_OPENROUTER_API_KEY (or OPENROUTER_API_KEY) environment variable.',
    );
  }

  rl.debug('comparison with description complete',);

  return {
    ...embeddingResult,
    description,
  };
}
