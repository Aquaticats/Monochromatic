/**
 CLI argument parser definitions for the mvm command.
 Defines the result type and combines subcommand parsers
 into a single top-level parser.
 */

// TODO: deprecate Optique
import { or, } from '@optique/core/constructs';
// TODO: deprecate Optique
import type { Parser, } from '@optique/core/parser';

import {
  createCmd,
  destroyCmd,
  execCmd,
  listCmd,
  lsCmd,
  pullCmd,
  pushCmd,
  rmCmd,
  runCmd,
  shellCmd,
  updateCmd,
} from './index-parsers-cmds.ts';

//region Result types: discriminated union for subcommand dispatch

/**
 Discriminated union of all subcommand parse results
 */
export type MvmArgs =
  | {
    cmd: 'create';
    image?: string;
    name: string;
    from?: string;
    serverType?: string;
    location?: string;
  }
    | {
      cmd: 'shell';
      name: string;
    }
    | { cmd: 'list'; }
    | { cmd: 'update'; }
    | {
      cmd: 'destroy';
      name?: string;
      all: boolean;
    }
    | {
      cmd: 'exec';
      name: string;
      command: string;
    }
    | {
      cmd: 'run';
      command: string;
      from?: string;
    }
    | {
      cmd: 'push';
      name: string;
      hostPath: string;
      guestPath: string;
    }
    | {
      cmd: 'pull';
      name: string;
      guestPath: string;
      hostPath: string;
    };

//endregion Result types

/* oxlint-disable typescript-eslint/no-explicit-any -- Parser is invariant in TState; opaque nested state types can't use unknown */
/**
 TODO: deprecate Optique
 Combined top-level parser across all subcommands.
 
 @example
 ```ts
 const result = parser.parse(process.argv.slice(2,),);
 ```
 */
export const parser: Parser<'sync', MvmArgs, any> = or(
  createCmd,
  shellCmd,
  listCmd,
  lsCmd,
  updateCmd,
  destroyCmd,
  rmCmd,
  execCmd,
  runCmd,
  pushCmd,
  pullCmd,
);
/* oxlint-enable typescript-eslint/no-explicit-any */
