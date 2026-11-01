/**
 Subcommand parser definitions for the mvm CLI.
 
 Defines individual subcommand parsers that are combined into
 the top-level parser in `index-parsers.ts`.
 
 @module
 */

// TODO: deprecate Optique
import {
  object,
  or,
} from '@optique/core/constructs';
// TODO: deprecate Optique
import { message, } from '@optique/core/message';
// TODO: deprecate Optique
import {
  map,
  multiple,
  optional,
} from '@optique/core/modifiers';
// TODO: deprecate Optique
import type { Parser, } from '@optique/core/parser';
// TODO: deprecate Optique
import {
  argument,
  command,
  flag,
  option,
} from '@optique/core/primitives';
// TODO: deprecate Optique
import { string, } from '@optique/core/valueparser';

import type { MvmArgs, } from './index-parsers.ts';

/**
 TODO: deprecate Optique
 Subcommand parser producing {@link MvmArgs}.
 Uses `any` for TState because Parser is invariant in TState
 and the deeply-nested state types are opaque implementation details.
 */
// oxlint-disable-next-line typescript/no-explicit-any -- Parser is invariant in TState; opaque nested state types can't use unknown
type SubcommandParser = Parser<'sync', MvmArgs, any>;

//region Shared value parsers: reusable metavar-labeled string parsers

/**
 TODO: deprecate Optique
 Value parser for VM name arguments, displayed as NAME in help
 */
const name = string({ metavar: 'NAME', },);

/**
 TODO: deprecate Optique
 Shared option parser for `--from SOURCE` cloning flag
 */
const fromOption = optional(
  option(
    '--from',
    string({ metavar: 'SOURCE', },),
    {
      description: message`Clone from an existing VM instead of creating fresh`,
    },
  ),
);

/**
 TODO: deprecate Optique
 Option parser for `--image IMAGE` to select a distro
 */
const imageOption = optional(
  option(
    '--image',
    string({ metavar: 'IMAGE', },),
    {
      description:
        message`Image to use: ubuntu (default), fedora, alpine, windows, or a custom template name`,
    },
  ),
);

/**
 TODO: deprecate Optique
 Option parser for `--server-type TYPE`, used by cloud backends (Hetzner).
 */
const serverTypeOption = optional(
  option(
    '--server-type',
    string({ metavar: 'TYPE', },),
    {
      description:
        message`Hetzner server type (e.g. cx23); defaults to the cheapest non-deprecated type; ignored by the libvirt backend`,
    },
  ),
);

/**
 TODO: deprecate Optique
 Option parser for `--location LOCATION`, used by cloud backends (Hetzner).
 Accepts a single location or a comma-separated fallback series.
 */
const locationOption = optional(
  option(
    '--location',
    string({ metavar: 'LOCATION', },),
    {
      description:
        message`Hetzner location or comma-separated fallback series (e.g. fsn1,nbg1); ignored by the libvirt backend`,
    },
  ),
);

/**
 TODO: deprecate Optique
 Value parser for individual command tokens after `--`, displayed as COMMAND in help
 */
const commandToken = string({ metavar: 'COMMAND', },);

//endregion Shared value parsers

//region Subcommand parsers

/**
 TODO: deprecate Optique
 Parser for `create <name> [--from SOURCE] [--image IMAGE]`
 */
export const createCmd: SubcommandParser = command(
  'create',
  map(
    object({
      name: argument(name,),
      from: fromOption,
      image: imageOption,
      serverType: serverTypeOption,
      location: locationOption,
    },),
    function toCreateArgs(v,): MvmArgs {
      return {
        cmd: 'create',
        name: v.name,
        ...(v.from !== undefined ? { from: v.from, } : {}),
        ...(v.image !== undefined ? { image: v.image, } : {}),
        ...(v.serverType !== undefined ? { serverType: v.serverType, } : {}),
        ...(v.location !== undefined ? { location: v.location, } : {}),
      };
    },
  ),
  { brief: message`Create and start a new VM`, },
);

/**
 TODO: deprecate Optique
 Parser for `shell <name>`
 */
export const shellCmd: SubcommandParser = command(
  'shell',
  map(
    object({ name: argument(name,), },),
    function toShellArgs(v: { readonly name: string; },): MvmArgs {
      return {
        cmd: 'shell',
        ...v,
      };
    },
  ),
  { brief: message`Open a serial console to a running VM`, },
);

/**
 TODO: deprecate Optique
 Parser for `list` (alias `ls`)
 */
export const listCmd: SubcommandParser = command(
  'list',
  map(
    object({},),
    function toListArgs(): MvmArgs {
      return { cmd: 'list', };
    },
  ),
  { brief: message`Show all VMs and their state`, },
);

/**
 TODO: deprecate Optique
 Parser for `ls` (hidden alias of `list`)
 */
export const lsCmd: SubcommandParser = command(
  'ls',
  map(
    object({},),
    function toLsArgs(): MvmArgs {
      return { cmd: 'list', };
    },
  ),
  { hidden: true, },
);

/**
 TODO: deprecate Optique
 Parser for `destroy --all` -- destroys every managed VM
 */
const destroyAllParser = map(
  object({ all: flag(
    '--all',
    { description: message`Destroy every managed VM`, },
  ), },),
  function toDestroyAllArgs(): MvmArgs {
    return {
      cmd: 'destroy',
      all: true,
    };
  },
);

/**
 TODO: deprecate Optique
 Parser for `destroy <name>` -- destroys a single VM by name
 */
const destroyNameParser = map(
  object({ name: argument(name,), },),
  function toDestroyNameArgs(v: { readonly name: string; },): MvmArgs {
    return {
      cmd: 'destroy',
      name: v.name,
      all: false,
    };
  },
);

/**
 TODO: deprecate Optique
 Parser for `destroy`
 */
export const destroyCmd: SubcommandParser = command(
  'destroy',
  or(
    destroyAllParser,
    destroyNameParser,
  ),
  { brief: message`Stop and delete a VM`, },
);

/**
 TODO: deprecate Optique
 Parser for `rm` (hidden alias of `destroy`)
 */
export const rmCmd: SubcommandParser = command(
  'rm',
  or(
    destroyAllParser,
    destroyNameParser,
  ),
  { hidden: true, },
);

/**
 TODO: deprecate Optique
 Parser for `exec <name> -- <command...>` -- run a command in an existing VM
 */
export const execCmd: SubcommandParser = command(
  'exec',
  map(
    object({
      name: argument(name,),
      args: multiple(argument(commandToken,),),
    },),
    function toExecArgs(v: {
      readonly name: string;
      readonly args: readonly string[];
    },): MvmArgs {
      return {
        cmd: 'exec',
        name: v.name,
        command: v.args
          .join(' ',),
      };
    },
  ),
  { brief: message`Run a command inside a named VM via guest agent`, },
);

/**
 TODO: deprecate Optique
 Parser for `run [--from SOURCE] -- <command...>` -- ephemeral VM
 */
export const runCmd: SubcommandParser = command(
  'run',
  map(
    object({
      from: fromOption,
      args: multiple(argument(commandToken,),),
    },),
    function toRunArgs(v,): MvmArgs {
      return {
        cmd: 'run',
        command: v.args
          .join(' ',),
        ...(v.from !== undefined ? { from: v.from, } : {}),
      };
    },
  ),
  { brief: message`Create an ephemeral VM, run a command, then destroy it`, },
);

/**
 TODO: deprecate Optique
 Parser for `update` -- re-downloads and rebuilds all template images
 */
export const updateCmd: SubcommandParser = command(
  'update',
  map(
    object({},),
    function toUpdateArgs(): MvmArgs {
      return { cmd: 'update', };
    },
  ),
  { brief: message`Re-download and rebuild all template images`, },
);

/**
 TODO: deprecate Optique
 Value parser for file path arguments, displayed as PATH in help.
 */
const path = string({ metavar: 'PATH', },);

/**
 TODO: deprecate Optique
 Parser for `push <name> <hostPath> <guestPath>` -- copy file from host into VM.
 */
export const pushCmd: SubcommandParser = command(
  'push',
  map(
    object({
      name: argument(name,),
      hostPath: argument(path,),
      guestPath: argument(path,),
    },),
    function toPushArgs(
      v: {
        readonly name: string;
        readonly hostPath: string;
        readonly guestPath: string;
      },
    ): MvmArgs {
      return {
        cmd: 'push',
        name: v.name,
        hostPath: v.hostPath,
        guestPath: v.guestPath,
      };
    },
  ),
  { brief: message`Copy a file from the host into a running VM`, },
);

/**
 TODO: deprecate Optique
 Parser for `pull <name> <guestPath> <hostPath>` -- copy file from VM to host.
 */
export const pullCmd: SubcommandParser = command(
  'pull',
  map(
    object({
      name: argument(name,),
      guestPath: argument(path,),
      hostPath: argument(path,),
    },),
    function toPullArgs(
      v: {
        readonly name: string;
        readonly guestPath: string;
        readonly hostPath: string;
      },
    ): MvmArgs {
      return {
        cmd: 'pull',
        name: v.name,
        guestPath: v.guestPath,
        hostPath: v.hostPath,
      };
    },
  ),
  { brief: message`Copy a file from a running VM to the host`, },
);

//endregion Subcommand parsers
