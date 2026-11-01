# @monochromatic-dev/llm-image-diff

Perceptual image difference comparison using multimodal embeddings
from [Voyage AI][voyage] and [Google Gemini][gemini].

## How it works

Images are converted to high-dimensional embedding vectors via multimodal embedding APIs.
Perceptual similarity is measured using cosine similarity (dot product for unit-normalized vectors).
A similarity score near 1 means the images are perceptually identical;
a score near 0 means they share little visual content.

By default,
 all available providers are called concurrently with their latest models,
giving you results from multiple perspectives.

## Setup

Set API keys in `.env.local`:

```text
IMAGE_DIFF_VOYAGE_API_KEY=pa-...
IMAGE_DIFF_GEMINI_API_KEY=AIza...
```

Or pass them directly via config:

```ts
import { compare, } from '@monochromatic-dev/llm-image-diff';

const result = await compare({
  imageA: { path: './before.png', },
  imageB: { path: './after.png', },
  config: { provider: 'gemini', apiKey: 'AIza...', },
},);
```

## Usage

### Compare using all providers (default)

```ts
import { compareAll, } from '@monochromatic-dev/llm-image-diff';

const results = await compareAll({
  imageA: { path: './before.png', },
  imageB: { path: './after.png', },
},);
for (const { provider, result, } of results)
  console.log(`${provider}: similarity=${result.similarity}`,);
```

### Compare using a specific provider

```ts
import { compare, } from '@monochromatic-dev/llm-image-diff';

const result = await compare({
  imageA: { path: './before.png', },
  imageB: { path: './after.png', },
  config: { provider: 'gemini', },
},);
console.log(`Similarity: ${result.similarity}`,);
```

### Embed a single image (all providers)

```ts
import { embedAll, } from '@monochromatic-dev/llm-image-diff';

const results = await embedAll({ path: './photo.png', },);
for (const { provider, result, } of results)
  console.log(`${provider}: ${result.embedding.length} dimensions`,);
```

### Batch embed multiple images

```ts
import { embedBatch, } from '@monochromatic-dev/llm-image-diff';

const { embeddings, } = await embedBatch({
  inputs: [{ path: './a.png', }, { path: './b.png', },],
  config: { provider: 'voyage', },
},);
```

### Supported input formats

- **File path**:
   `{ path: './photo.png' }`:
   reads from disk,
   infers format from extension
- **URL**:
   `{ url: 'https://...' }`:
   passed directly to Voyage;
   fetched and base64-encoded for Gemini
- **Base64**:
   `{ base64: 'data:image/png;base64,...' }`:
   pre-encoded data URI
- **Buffer**:
   `{ buffer: arrayBuffer, format: 'png' }`:
   raw bytes with explicit format

### Similarity utilities

```ts
import {
  cosineSimilarity,
  dotProduct,
} from '@monochromatic-dev/llm-image-diff';

// For unit-normalized embeddings (Voyage, Gemini at 3072 dims), dotProduct === cosineSimilarity
const sim = dotProduct({ a: embeddingA, b: embeddingB, },);

// For arbitrary vectors, use cosineSimilarity which normalizes first
const sim2 = cosineSimilarity({ a: vectorA, b: vectorB, },);
```

## CLI

```sh
# Compare using all providers (default)
image-diff compare before.png after.png

# Compare with a specific provider
image-diff compare --provider gemini before.png after.png

# Embed a single image (all providers)
image-diff embed photo.png

# Embed with a specific provider and model
image-diff embed --provider voyage --model voyage-multimodal-3.5 photo.png
```

## Providers and models

### Voyage AI

- `voyage-multimodal-3.5` (default):
   latest,
   highest quality
- `voyage-multimodal-3`:
   previous generation

### Google Gemini

- `gemini-embedding-2-preview` (default):
   multimodal embedding with 3072 dimensions

[voyage]: https://docs.voyageai.com/docs/multimodal-embeddings
[gemini]: https://ai.google.dev/gemini-api/docs/embeddings
