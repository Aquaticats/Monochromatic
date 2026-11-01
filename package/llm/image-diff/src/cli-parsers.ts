/**
 CLI argument parser definitions for the image-diff command.
 Defines the result type and combines subcommand parsers into a single
 top-level parser.
 
 @module
 */

// TODO: deprecate Optique
import { or, } from '@optique/core/constructs';
// TODO: deprecate Optique
import type { Parser, } from '@optique/core/parser';

import {
  compareCmd,
  embedCmd,
} from './cli-parsers-cmds.ts';
import type {
  EmbeddingModel,
  Provider,
} from './types.ts';

//region Result types: discriminated union for subcommand dispatch

/**
 Discriminated union of all subcommand parse results.
 */
export type ImageDiffArgs =
  | {
    cmd: 'compare';
    imageA: string;
    imageB: string;
    provider?: Provider;
    model?: EmbeddingModel;
  }
    | {
      cmd: 'embed';
      image: string;
      provider?: Provider;
      model?: EmbeddingModel;
    };

//endregion Result types

/* oxlint-disable typescript-eslint/no-explicit-any -- Parser is invariant in TState; opaque nested state types can't use unknown */
/**
 TODO: deprecate Optique
 Combined top-level parser across all subcommands.
 
 @example
 ```ts
 const result = runSync(parser, { programName: 'image-diff' });
 ```
 */
export const parser: Parser<'sync', ImageDiffArgs, any> = or(
  compareCmd,
  embedCmd,
);
/* oxlint-enable typescript-eslint/no-explicit-any */
