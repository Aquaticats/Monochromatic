import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { parseImageArg, } from './cli.image.ts';
import { embedAll, } from './client.multi.ts';
import { embed, } from './client.ts';
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
 Handle the `embed` subcommand.
 
 @param image - image argument (path or URL)
 
 @param provider - provider name; omit to use all providers
 
 @param model - model override; omit for the provider's default
 
 @example
 ```ts
 await handleEmbed({
   image: 'photo.png',
   provider: 'voyage',
 });
 ```
 */
export async function handleEmbed({
  image,
  provider,
  model,
}: {
  readonly image: string;
  readonly provider?: Provider;
  readonly model?: EmbeddingModel;
},): Promise<void> {
  /**
   Logger pre-tagged with this function's name so call-site context is preserved across debug lines.
   */
  const rl = tagged({
    tag: handleEmbed.name,
    l,
  },);
  /**
   Parsed image input; converts the CLI path-or-URL string into the structured {@link ImageInput}.
   */
  const input = parseImageArg(image,);

  if (provider !== undefined) {
    rl.debug(`embedding via ${provider}`,);
    /**
     Single-provider config; only includes `model` when the caller passed an override.
     */
    const config = {
      provider,
      ...(model !== undefined ? { model, } : {}),
    };
    /**
     Single-provider embedding result; printed as JSON below.
     */
    const result = await embed({
      input,
      config,
    },);

    console.log(JSON.stringify(
      {
        provider,
        dimensions: result.embedding
          .length,
        usage: result.usage,
        embedding: result.embedding,
      },
      null,
      2,
    ),);
  }
  else {
    rl.debug('embedding via all providers',);
    /**
     Per-provider embedding entries from the multi-provider dispatch; printed as JSON below.
     */
    const results = await embedAll(input,);

    console.log(JSON.stringify(
      results.map(function formatEntry(entry,) {
        return {
          provider: entry.provider,
          dimensions: entry.result
            .embedding
            .length,
          usage: entry.result
            .usage,
          embedding: entry.result
            .embedding,
        };
      },),
      null,
      2,
    ),);
  }
}
