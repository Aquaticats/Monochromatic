/**
 Subcommand parser definitions for the vmsync CLI.
 
 Defines individual subcommand parsers that are combined into
 the top-level parser in `index-parsers.ts`.
 
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
  integer,
  string,
} from '@optique/core/valueparser';

import type { VmsyncArgs, } from './index-parsers.ts';

/**
 TODO: deprecate Optique
 Subcommand parser producing VmsyncArgs.
 Uses `any` for TState because Parser is invariant in TState
 and the deeply-nested state types are opaque implementation details.
 */
// oxlint-disable-next-line typescript/no-explicit-any -- Parser is invariant in TState; opaque nested state types can't use unknown
type SubcommandParser = Parser<'sync', VmsyncArgs, any>;

//region Shared value parsers

/**
 TODO: deprecate Optique
 Value parser for VM name arguments, displayed as NAME in help.
 */
const name = string({
  metavar: 'NAME',
},);

/**
 TODO: deprecate Optique
 Value parser for file path arguments, displayed as PATH in help.
 */
const path = string({
  metavar: 'PATH',
},);

/**
 TODO: deprecate Optique
 Value parser for memory strings, displayed as MEMORY in help.
 */
const memoryValue = string({
  metavar: 'MEMORY',
},);

/**
 TODO: deprecate Optique
 Value parser for CPU count, displayed as CPUS in help.
 */
const cpusValue = integer({
  metavar: 'CPUS',
  min: 1,
},);

//endregion Shared value parsers

//region Subcommand parsers

/**
 TODO: deprecate Optique
 Parser for `import <path> [--name NAME]`, producing an `import` {@link VmsyncArgs}.
 */
export const importCmd: SubcommandParser = command(
  'import',
  map(
    object({
      imagePath: argument(path,),
      name: optional(
        option(
          '--name',
          name,
          {
            description: message`Override the derived VM name`,
          },
        ),
      ),
    },),
    function toImportArgs(
      v: {
        readonly imagePath: string;
        // TODO: deprecate Optique
        // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors @optique/core optional() output, which yields `TValue | undefined` for an absent option; converted to an absent property below
        readonly name: string | undefined;
      },
    ): VmsyncArgs {
      if (v.name === undefined)
        return {
          cmd: 'import',
          imagePath: v.imagePath,
        };
      return {
        cmd: 'import',
        imagePath: v.imagePath,
        name: v.name,
      };
    },
  ),
  {
    brief: message`Import a disk image and convert to qcow2 + vhdx`,
  },
);

/**
 TODO: deprecate Optique
 Parser for `boot <name>`, producing a `boot` {@link VmsyncArgs}.
 */
export const bootCmd: SubcommandParser = command(
  'boot',
  map(
    object({
      name: argument(name,),
    },),
    function toBootArgs(v: { readonly name: string; },): VmsyncArgs {
      return {
        cmd: 'boot',
        name: v.name,
      };
    },
  ),
  {
    brief: message`Boot a VM using the platform's native hypervisor`,
  },
);

/**
 TODO: deprecate Optique
 Parser for `sync <name>`, producing a `sync` {@link VmsyncArgs}.
 */
export const syncCmd: SubcommandParser = command(
  'sync',
  map(
    object({
      name: argument(name,),
    },),
    function toSyncArgs(v: { readonly name: string; },): VmsyncArgs {
      return {
        cmd: 'sync',
        name: v.name,
      };
    },
  ),
  {
    brief: message`Sync changed blocks to the other disk format`,
  },
);

/**
 TODO: deprecate Optique
 Parser for `status <name>`, producing a `status` {@link VmsyncArgs}.
 */
export const statusCmd: SubcommandParser = command(
  'status',
  map(
    object({
      name: argument(name,),
    },),
    function toStatusArgs(v: { readonly name: string; },): VmsyncArgs {
      return {
        cmd: 'status',
        name: v.name,
      };
    },
  ),
  {
    brief: message`Show sync state and boot info for a VM`,
  },
);

/**
 TODO: deprecate Optique
 Parser for `list`, producing a `list` {@link VmsyncArgs}.
 */
export const listCmd: SubcommandParser = command(
  'list',
  map(
    object({},),
    function toListArgs(): VmsyncArgs {
      return {
        cmd: 'list',
      };
    },
  ),
  {
    brief: message`Show all managed VMs`,
  },
);

/**
 TODO: deprecate Optique
 Parser for `config <name> [--memory MEMORY] [--cpus CPUS]`, producing a `config` {@link VmsyncArgs}.
 */
export const configCmd: SubcommandParser = command(
  'config',
  map(
    object({
      name: argument(name,),
      memory: optional(
        option(
          '--memory',
          memoryValue,
          {
            description: message`Memory allocation (e.g. "8G", "4096M")`,
          },
        ),
      ),
      cpus: optional(
        option(
          '--cpus',
          cpusValue,
          {
            description: message`Number of virtual CPUs`,
          },
        ),
      ),
    },),
    function toConfigArgs(
      v: {
        readonly name: string;
        // TODO: deprecate Optique
        // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors @optique/core optional() output, which yields `TValue | undefined` for an absent option; converted to absent properties below
        readonly memory: string | undefined;
        // TODO: deprecate Optique
        // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors @optique/core optional() output, which yields `TValue | undefined` for an absent option; converted to absent properties below
        readonly cpus: number | undefined;
      },
    ): VmsyncArgs {
      if ((v.memory !== undefined) && (v.cpus !== undefined))
        return {
          cmd: 'config',
          name: v.name,
          memory: v.memory,
          cpus: v.cpus,
        };
      if (v.memory !== undefined)
        return {
          cmd: 'config',
          name: v.name,
          memory: v.memory,
        };
      if (v.cpus !== undefined)
        return {
          cmd: 'config',
          name: v.name,
          cpus: v.cpus,
        };
      return {
        cmd: 'config',
        name: v.name,
      };
    },
  ),
  {
    brief: message`Update boot settings for a VM`,
  },
);

//endregion Subcommand parsers
