/**
 Decides which host commands run `virsh` and `qemu-img`.

 Order, per tool:

 1.  The tool's environment variable, a JSON array holding the executable and
     its leading arguments.
 2.  The bare tool name, when `virsh` is an executable in a directory of `PATH`.
 3.  The tool inside the virt-manager Flatpak, when that Flatpak is installed.
 4.  The bare tool name, so the failure names what is missing.

 Steps 2 and 3 treat both tools as one installation: the `qemu-img` that
 creates a disk comes from the same place as the libvirt that runs it.

 @module
 */

import {
  access,
  constants,
} from 'node:fs/promises';
import {
  delimiter,
  join,
} from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { spawn, } from './spawn.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Names

/**
 Environment variable holding the command that runs `virsh`, as a JSON array.
 */
export const VIRSH_COMMAND_ENV = 'MVM_VIRSH_COMMAND';

/**
 Environment variable holding the command that runs `qemu-img`, as a JSON array.
 */
export const QEMU_IMG_COMMAND_ENV = 'MVM_QEMU_IMG_COMMAND';

/**
 Flatpak application whose sandbox ships libvirt and QEMU on hosts that have neither installed.
 */
export const VIRT_MANAGER_FLATPAK = 'org.virt_manager.virt-manager';

/**
 Where a tool command came from.

 @example
 ```ts
 const origin: ToolOrigin = 'flatpak';
 ```
 */
export type ToolOrigin = 'environment' | 'flatpak' | 'path';

/**
 Command that runs one tool: the executable, then arguments placed before the tool's own.

 @example
 ```ts
 const virsh: ToolCommand = {
   argv: ['flatpak', 'run', '--command=virsh', 'org.virt_manager.virt-manager'],
   origin: 'flatpak',
 };
 ```
 */
export type ToolCommand = {
  /**
   Executable followed by its leading arguments; never empty.
   */
  readonly argv: readonly [
    string,
    ...string[]
  ];
  /**
   Where the command came from, for diagnostics.
   */
  readonly origin: ToolOrigin;
};

/**
 Commands for the libvirt tools mvm runs.

 @example
 ```ts
 const tools: LibvirtTools = await libvirtTools();
 ```
 */
export type LibvirtTools = {
  readonly qemuImg: ToolCommand;
  readonly virsh: ToolCommand;
};

//endregion Names

//region Environment values

/**
 Example value shown when a tool variable cannot be read.
 */
const EXAMPLE_VALUE = `["flatpak","run","--command=virsh","${VIRT_MANAGER_FLATPAK}"]`;

/**
 A tool variable is set to something other than a JSON array of non-empty strings.

 @example
 ```ts
 try {
   parseToolCommand({ value: 'flatpak run virsh', variable: 'MVM_VIRSH_COMMAND' });
 }
 catch (error) {
   if (error instanceof ToolCommandConfigError) console.error(error.variable);
 }
 ```
 */
export class ToolCommandConfigError extends Error {
  /**
   Name of the environment variable holding the unusable value.
   */
  readonly variable: string;

  /**
   @param cause - Parse failure, when the value is not JSON at all

   @param value - Text the variable holds, quoted in the message

   @param variable - Name of the environment variable
   */
  constructor({
    cause,
    value,
    variable,
  }: {
    readonly cause?: unknown;
    readonly value: string;
    readonly variable: string;
  },) {
    super(
      [
        `${variable} must be a JSON array of non-empty strings: the executable, then its leading arguments.`,
        `It is set to: ${value}`,
        `Example: ${variable}='${EXAMPLE_VALUE}'`,
        'A JSON array is used so that an argument may contain spaces without any quoting rules.',
      ].join('\n',),
      { cause, },
    );
    this.name = 'ToolCommandConfigError';
    this.variable = variable;
  }
}

/**
 Checks that a parsed value is a non-empty list of non-empty strings.

 @param value - Parsed JSON value

 @returns Whether `value` can be a tool command

 @example
 ```ts
 isArgv(['virsh']); // true
 isArgv([]);        // false
 ```
 */
function isArgv(value: unknown,): value is readonly [
  string,
  ...string[]
] {
  return Array.isArray(value,)
    && (value.length > 0)
    && value.every(function isNonEmptyText(element: unknown,) {
      return ((typeof element) === 'string') && (element !== '');
    },);
}

/**
 Reads a tool command from the text of its environment variable.

 @param value - Text the variable holds

 @param variable - Name of the environment variable, named in the error

 @returns Executable followed by its leading arguments

 @throws {@link ToolCommandConfigError} when the text is not a JSON array of non-empty strings

 @example
 ```ts
 parseToolCommand({ value: '["virsh"]', variable: 'MVM_VIRSH_COMMAND' }); // => ['virsh']
 ```
 */
export function parseToolCommand({
  value,
  variable,
}: {
  readonly value: string;
  readonly variable: string;
},): readonly [
  string,
  ...string[]
] {
  /**
   Logger scoped to the parse so an unreadable value is attributable.
   */
  const rl = tagged({
    tag: parseToolCommand.name,
    l,
  },);
  /**
   Parsed value, typed as unknown until the shape check narrows it.
   */
  const parsed = (function parse(): unknown {
    try {
      return JSON.parse(value,);
    }
    catch (error) {
      rl.debug(`${variable} is not JSON`,);
      throw new ToolCommandConfigError({
        cause: error,
        value,
        variable,
      },);
    }
  })();
  if (!isArgv(parsed,)) {
    throw new ToolCommandConfigError({
      value,
      variable,
    },);
  }
  return parsed;
}

//endregion Environment values

//region Choice

/**
 Chooses the tool commands from the facts about this host. Pure: reads nothing.

 @param flatpakInstalled - Whether the virt-manager Flatpak is installed; only consulted when `virsh` is not on `PATH`

 @param qemuImgValue - Text of the `qemu-img` variable, `''` when unset

 @param virshOnPath - Whether `virsh` is an executable in a directory of `PATH`

 @param virshValue - Text of the `virsh` variable, `''` when unset

 @returns Commands for both tools

 @throws {@link ToolCommandConfigError} when a set variable cannot be read

 @example
 ```ts
 chooseLibvirtTools({ flatpakInstalled: true, qemuImgValue: '', virshOnPath: false, virshValue: '' });
 // => both tools run through `flatpak run --command=<tool> org.virt_manager.virt-manager`
 ```
 */
export function chooseLibvirtTools({
  flatpakInstalled,
  qemuImgValue,
  virshOnPath,
  virshValue,
}: {
  readonly flatpakInstalled: boolean;
  readonly qemuImgValue: string;
  readonly virshOnPath: boolean;
  readonly virshValue: string;
},): LibvirtTools {
  /**
   Whether an unconfigured tool runs inside the Flatpak rather than by its bare name.
   */
  const viaFlatpak = (!virshOnPath) && flatpakInstalled;
  /**
   Chooses one tool's command.

   @param tool - Executable name of the tool

   @param value - Text of the tool's variable, `''` when unset

   @param variable - Name of the tool's variable

   @returns Command for the tool
   */
  function choose({
    tool,
    value,
    variable,
  }: {
    readonly tool: string;
    readonly value: string;
    readonly variable: string;
  },): ToolCommand {
    if (value !== '') {
      return {
        argv: parseToolCommand({
          value,
          variable,
        },),
        origin: 'environment',
      };
    }
    if (viaFlatpak) {
      return {
        argv: [
          'flatpak',
          'run',
          `--command=${tool}`,
          VIRT_MANAGER_FLATPAK,
        ],
        origin: 'flatpak',
      };
    }
    return {
      argv: [tool,],
      origin: 'path',
    };
  }
  return {
    qemuImg: choose({
      tool: 'qemu-img',
      value: qemuImgValue,
      variable: QEMU_IMG_COMMAND_ENV,
    },),
    virsh: choose({
      tool: 'virsh',
      value: virshValue,
      variable: VIRSH_COMMAND_ENV,
    },),
  };
}

//endregion Choice

//region Host facts

/**
 Checks whether a path is a file this user may execute.

 @param path - Candidate path

 @returns Whether the path exists and is executable

 @example
 ```ts
 await isExecutable('/usr/bin/virsh');
 ```
 */
async function isExecutable(path: string,): Promise<boolean> {
  try {
    await access(
      path,
      constants.X_OK,
    );
    return true;
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    return false;
  }
}

/**
 Checks whether a bare executable name resolves through `PATH`.

 @param name - Executable name without any directory

 @returns Whether some directory of `PATH` holds an executable of that name

 @example
 ```ts
 await isOnPath('virsh');
 ```
 */
async function isOnPath(name: string,): Promise<boolean> {
  /**
   Directories of `PATH`; an empty entry means the current directory, which is not searched here.
   */
  const directories = (process.env
    .PATH
    ?? '')
    .split(delimiter,)
    .filter(function isNamed(directory,) {
      return directory !== '';
    },);
  /**
   One answer per directory; the lookups do not depend on each other.
   */
  const found = await Promise.all(
    directories.map(function hasExecutable(directory,) {
      return isExecutable(join(
        directory,
        name,
      ),);
    },),
  );
  return found.includes(true,);
}

/**
 Checks whether the virt-manager Flatpak is installed.

 @returns Whether `flatpak info` knows the application

 @example
 ```ts
 await isFlatpakInstalled();
 ```
 */
async function isFlatpakInstalled(): Promise<boolean> {
  /**
   Logger scoped to the check so its outcome is attributable.
   */
  const rl = tagged({
    tag: isFlatpakInstalled.name,
    l,
  },);
  try {
    await spawn({
      args: [
        'info',
        '--show-ref',
        VIRT_MANAGER_FLATPAK,
      ],
      command: 'flatpak',
    },);
    return true;
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    rl.debug(`the ${VIRT_MANAGER_FLATPAK} Flatpak is not available: ${error.message}`,);
    return false;
  }
}

//endregion Host facts

//region Resolution

/**
 Sentinel for "nothing resolved yet".
 A unique symbol models the empty cache without a nullish union.
 */
const NOT_RESOLVED: unique symbol = Symbol('set before the tool commands are first resolved',);

/**
 Latest resolution and the inputs it was made from, so a long-lived process
 such as the MCP server probes the host once, yet a changed environment is honored.
 */
const latest: {
  inputs: string;
  tools: Promise<LibvirtTools> | typeof NOT_RESOLVED;
} = {
  inputs: '',
  tools: NOT_RESOLVED,
};

/**
 Gathers the host facts and chooses the tool commands.

 @param qemuImgValue - Text of the `qemu-img` variable, `''` when unset

 @param virshValue - Text of the `virsh` variable, `''` when unset

 @returns Commands for both tools

 @example
 ```ts
 const tools = await resolve({ qemuImgValue: '', virshValue: '' });
 ```
 */
async function resolve({
  qemuImgValue,
  virshValue,
}: {
  readonly qemuImgValue: string;
  readonly virshValue: string;
},): Promise<LibvirtTools> {
  /**
   Logger scoped to the resolution so the chosen commands are attributable.
   */
  const rl = tagged({
    tag: resolve.name,
    l,
  },);
  /**
   Whether any tool still needs the host to be looked at.
   */
  const needsHostFacts = (virshValue === '') || (qemuImgValue === '');
  /**
   Whether bare `virsh` works; not looked up when both tools are configured.
   */
  const virshOnPath = needsHostFacts && (await isOnPath('virsh',));
  /**
   Whether the Flatpak can stand in; only asked when bare `virsh` is missing.
   */
  const flatpakInstalled = needsHostFacts && (!virshOnPath)
    && (await isFlatpakInstalled());
  /**
   Chosen commands.
   */
  const tools = chooseLibvirtTools({
    flatpakInstalled,
    qemuImgValue,
    virshOnPath,
    virshValue,
  },);
  rl.debug(
    `virsh runs as ${JSON.stringify(tools.virsh
      .argv,)} (${tools.virsh
        .origin}), qemu-img as ${
      JSON.stringify(tools.qemuImg
        .argv,)
    } (${tools.qemuImg
      .origin})`,
  );
  return tools;
}

/**
 Returns the commands that run `virsh` and `qemu-img` on this host.
 The host is probed once per process for each distinct set of inputs
 (the two variables and `PATH`).

 @returns Commands for both tools

 @throws {@link ToolCommandConfigError} when a set variable cannot be read

 @example
 ```ts
 const { virsh, } = await libvirtTools();
 ```
 */
export function libvirtTools(): Promise<LibvirtTools> {
  /**
   Text of the `virsh` variable, `''` when unset.
   */
  const virshValue = process.env[VIRSH_COMMAND_ENV] ?? '';
  /**
   Text of the `qemu-img` variable, `''` when unset.
   */
  const qemuImgValue = process.env[QEMU_IMG_COMMAND_ENV] ?? '';
  /**
   Everything the resolution depends on, as one comparable text.
   */
  const inputs = JSON.stringify([
    virshValue,
    qemuImgValue,
    process.env
      .PATH
      ?? '',
  ],);
  if ((latest.tools !== NOT_RESOLVED) && (latest.inputs === inputs)) {
    return latest.tools;
  }
  /**
   Resolution in progress; cached before it settles so concurrent callers share one probe.
   */
  const tools = resolve({
    qemuImgValue,
    virshValue,
  },);
  latest.inputs = inputs;
  latest.tools = tools;
  return tools;
}

//endregion Resolution

//region Advice for a missing tool

/**
 Explains what to do when a tool's executable does not exist.

 @param command - Command mvm tried to run the tool with

 @param tool - Executable name of the tool, `virsh` or `qemu-img`

 @param variable - Environment variable that configures the tool's command

 @returns Advice text of several lines

 @example
 ```ts
 toolNotFoundRemedy({ command: { argv: ['virsh'], origin: 'path' }, tool: 'virsh', variable: 'MVM_VIRSH_COMMAND' });
 ```
 */
export function toolNotFoundRemedy({
  command,
  tool,
  variable,
}: {
  readonly command: ToolCommand;
  readonly tool: string;
  readonly variable: string;
},): string {
  /**
   Why mvm ran this command, by where the command came from.
   */
  const reason = {
    environment: `${variable} names it`,
    flatpak: `${variable} is not set, virsh is not on PATH, and the ${VIRT_MANAGER_FLATPAK} Flatpak is installed`,
    path: `${variable} is not set, so the bare name is looked up on PATH`,
  }[command.origin];
  return [
    `mvm runs ${tool} as ${JSON.stringify(command.argv,)}, because ${reason}.`,
    'Any one of these makes it available:',
    `- install the host package that provides \`${tool}\`, so that it is on PATH;`,
    `- install the ${VIRT_MANAGER_FLATPAK} Flatpak, whose virsh and qemu-img mvm then uses;`,
    `- set ${variable} to a JSON array holding the executable and its leading arguments, for example ${variable}='["flatpak","run","--command=${tool}","${VIRT_MANAGER_FLATPAK}"]'.`,
    `The MCP server reads ${variable} from its own environment, which is set where the server is registered, not in a shell profile.`,
  ].join('\n',);
}

//endregion Advice for a missing tool
