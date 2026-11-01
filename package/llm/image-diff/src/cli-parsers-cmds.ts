/**
 Subcommand parser definitions for the image-diff CLI.
 
 Defines individual subcommand parsers (`compare`, `embed`) that are
 combined into the top-level parser in `cli-parsers.ts`.
 
 @module
 */

// TODO: deprecate Optique
import { object, } from '@optique/core/constructs';
// TODO: deprecate Optique
import { message, } from '@optique/core/message';
// TODO: deprecate Optique
import {
  map,
  optional,
} from '@optique/core/modifiers';
// TODO: deprecate Optique
import type { Parser, } from '@optique/core/parser';
// TODO: deprecate Optique
import {
  argument,
  command,
  option,
} from '@optique/core/primitives';
// TODO: deprecate Optique
import {
  choice,
  string,
} from '@optique/core/valueparser';

import type { ImageDiffArgs, } from './cli-parsers.ts';

/**
 TODO: deprecate Optique
 Subcommand parser producing ImageDiffArgs.
 Uses `any` for TState because Parser is invariant in TState
 and the deeply-nested state types are opaque implementation details.
 */
// oxlint-disable-next-line typescript/no-explicit-any -- Parser is invariant in TState; opaque nested state types can't use unknown
type SubcommandParser = Parser<'sync', ImageDiffArgs, any>;

//region Shared value parsers

/**
 TODO: deprecate Optique
 Recognized provider names.
 */
const providerValue = choice(
  [
    'voyage',
    'gemini',
  ] as const,
  { metavar: 'PROVIDER', },
);

/**
 TODO: deprecate Optique
 Recognized embedding model names across providers.
 */
const modelValue = choice(
  [
    'voyage-multimodal-3',
    'voyage-multimodal-3.5',
    'gemini-embedding-2-preview',
  ] as const,
  { metavar: 'MODEL', },
);

/**
 TODO: deprecate Optique
 Value parser for image arguments (file path or URL); URL vs path resolved at runtime.
 */
const imageValue = string({ metavar: 'IMAGE', },);

//endregion Shared value parsers

//region Subcommand parsers

/**
 TODO: deprecate Optique
 Parser for `compare <imageA> <imageB> [--provider PROVIDER] [--model MODEL]`.
 */
export const compareCmd: SubcommandParser = command(
  'compare',
  map(
    object({
      imageA: argument(imageValue,),
      imageB: argument(imageValue,),
      provider: optional(option(
        '--provider',
        providerValue,
        { description: message`Embedding provider (omit to use all providers)`, },
      ),),
      model: optional(option(
        '--model',
        modelValue,
        { description: message`Model override for the selected provider`, },
      ),),
    },),
    function toCompareArgs(v,): ImageDiffArgs {
      /**
       Parsed fields destructured so optional spreads read plain identifiers.
       */
      const {
        imageA,
        imageB,
        provider,
        model,
      } = v;
      return {
        cmd: 'compare',
        imageA,
        imageB,
        ...(provider !== undefined ? { provider, } : {}),
        ...(model !== undefined ? { model, } : {}),
      };
    },
  ),
  {
    brief: message`Compare two images and report similarity, distance, and a description`,
  },
);

/**
 TODO: deprecate Optique
 Parser for `embed <image> [--provider PROVIDER] [--model MODEL]`.
 */
export const embedCmd: SubcommandParser = command(
  'embed',
  map(
    object({
      image: argument(imageValue,),
      provider: optional(option(
        '--provider',
        providerValue,
        { description: message`Embedding provider (omit to use all providers)`, },
      ),),
      model: optional(option(
        '--model',
        modelValue,
        { description: message`Model override for the selected provider`, },
      ),),
    },),
    function toEmbedArgs(v,): ImageDiffArgs {
      /**
       Parsed fields destructured so optional spreads read plain identifiers.
       */
      const {
        image,
        provider,
        model,
      } = v;
      return {
        cmd: 'embed',
        image,
        ...(provider !== undefined ? { provider, } : {}),
        ...(model !== undefined ? { model, } : {}),
      };
    },
  ),
  {
    brief: message`Compute multimodal embedding(s) for an image`,
  },
);

//endregion Subcommand parsers
