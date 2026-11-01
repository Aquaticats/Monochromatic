#!/usr/bin/env node
/**
 CLI for perceptual image difference comparison using multimodal embeddings.
 Supports Voyage AI and Google Gemini backends.
 
 @example
 ```sh
 # Compare using all providers (default)
 image-diff compare before.png after.png
 
 # Compare with a specific provider
 image-diff compare --provider gemini before.png after.png
 
 # Embed with a specific model
 image-diff embed --provider voyage --model voyage-multimodal-3.5 photo.png
 
 # Embed using all providers
 image-diff embed photo.png
 ```
 
 @packageDocumentation
 */

// TODO: deprecate Optique
import { message, } from '@optique/core/message';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';

import { parser, } from './cli-parsers.ts';
import { handleCompare, } from './cli.compare.ts';
import { handleEmbed, } from './cli.embed.ts';

/**
 TODO: deprecate Optique
 Parsed CLI result from process.argv.
 */
const args = runSync(
  parser,
  {
    programName: 'image-diff',
    help: 'option',
    aboveError: 'help',
    brief: message`image-diff - perceptual image comparison via multimodal embeddings`,
    footer:
      message`Set IMAGE_DIFF_VOYAGE_API_KEY, IMAGE_DIFF_GEMINI_API_KEY, or IMAGE_DIFF_OPENROUTER_API_KEY to enable each backend.`,
  },
);

/**
 Provider/model overrides shared by both subcommands; spread in only when present.
 */
const {
  provider,
  model,
} = args;

await (args.cmd
  === 'compare'
  ? handleCompare({
    imageA: args.imageA,
    imageB: args.imageB,
    ...(provider !== undefined ? { provider, } : {}),
    ...(model !== undefined ? { model, } : {}),
  },)
  : handleEmbed({
    image: args.image,
    ...(provider !== undefined ? { provider, } : {}),
    ...(model !== undefined ? { model, } : {}),
  },));
