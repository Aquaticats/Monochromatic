import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { parseImageArg, } from './cli.image.ts';
import { compareAll, } from './client.multi.compare.ts';
import { compare, } from './client.ts';
import type {
  EmbeddingModel,
  Provider,
} from './types.ts';

/**
 Logger root for image-diff after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'image-diff', },);

/**
 Handle the `compare` subcommand.
 
 @param imageA - first image argument (path or URL)
 
 @param imageB - second image argument (path or URL)
 
 @param provider - provider name; omit to use all providers
 
 @param model - model override; omit for the provider's default
 
 @example
 ```ts
 await handleCompare({
   imageA: 'a.png',
   imageB: 'b.png',
   provider: 'voyage',
 });
 ```
 */
export async function handleCompare({
  imageA,
  imageB,
  provider,
  model,
}: {
  readonly imageA: string;
  readonly imageB: string;
  readonly provider?: Provider;
  readonly model?: EmbeddingModel;
},): Promise<void> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: handleCompare.name,
    l,
  },);
  /**
   Parsed first-image input; converts path or URL string into the structured {@link ImageInput}.
   */
  const inputA = parseImageArg(imageA,);
  /**
   Parsed second-image input; converts path or URL string into the structured {@link ImageInput}.
   */
  const inputB = parseImageArg(imageB,);

  if (provider !== undefined) {
    rl.debug(`comparing via ${provider}`,);
    /**
     Single-provider config; only includes `model` when the caller passed an override.
     */
    const config = {
      provider,
      ...(model !== undefined ? { model, } : {}),
    };
    /**
     Comparison result for the single requested provider; printed as JSON below.
     */
    const result = await compare({
      imageA: inputA,
      imageB: inputB,
      config,
    },);

    console.log(JSON.stringify(
      {
        provider,
        similarity: result.similarity,
        distance: result.distance,
        embeddingDimensions: result.embeddingA
          .length,
        description: result.description,
      },
      null,
      2,
    ),);
  }
  else {
    rl.debug('comparing via all providers',);
    /**
     Per-provider comparison entries from the multi-provider dispatch; printed as JSON below.
     */
    const results = await compareAll({
      imageA: inputA,
      imageB: inputB,
    },);

    console.log(JSON.stringify(
      results.map(function formatEntry(entry,) {
        return {
          provider: entry.provider,
          similarity: entry.result
            .similarity,
          distance: entry.result
            .distance,
          embeddingDimensions: entry.result
            .embeddingA
            .length,
          description: entry.result
            .description,
        };
      },),
      null,
      2,
    ),);
  }
}
