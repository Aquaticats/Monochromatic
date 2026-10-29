/**
 CLI argument parser definitions for the vmsync command.
 Defines the result type and combines subcommand parsers
 into a single top-level parser.
 
 @module
 */

// TODO: deprecate Optique
import { or, } from '@optique/core/constructs';
// TODO: deprecate Optique
import type { Parser, } from '@optique/core/parser';

import {
  bootCmd,
  configCmd,
  importCmd,
  listCmd,
  statusCmd,
  syncCmd,
} from './index-parsers-cmds.ts';

//region Result types: discriminated union for subcommand dispatch

/**
 Discriminated union of all subcommand parse results.
 */
export type VmsyncArgs =
  | {
    cmd: 'import';
    imagePath: string;
    name?: string;
  }
    | {
      cmd: 'boot';
      name: string;
    }
    | {
      cmd: 'sync';
      name: string;
    }
    | {
      cmd: 'status';
      name: string;
    }
    | { cmd: 'list'; }
    | {
      cmd: 'config';
      name: string;
      memory?: string;
      cpus?: number;
    };

//endregion Result types

/* oxlint-disable typescript-eslint/no-explicit-any -- Parser is invariant in TState; opaque nested state types can't use unknown */
/**
 TODO: deprecate Optique
 Combined top-level parser across all subcommands: {@link importCmd}, {@link bootCmd},
 {@link syncCmd}, {@link statusCmd}, {@link listCmd}, and {@link configCmd}.
 
 @example
 ```ts
 const result = parser.parse(process.argv.slice(2,),);
 ```
 */
export const parser: Parser<'sync', VmsyncArgs, any> = or(
  importCmd,
  bootCmd,
  syncCmd,
  statusCmd,
  listCmd,
  configCmd,
);
/* oxlint-enable typescript-eslint/no-explicit-any */
