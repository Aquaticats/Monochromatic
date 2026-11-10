/**
 VM configuration persistence and platform detection.
 
 JSONC parsing is owned by `@monochromatic-dev/jsonc-edit-core-ts`; this module
 only decides which file to read and writes the config back out.
 
 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  jsoncGetValue,
  parseJsoncEdit,
  type StringJsonc,
} from '@monochromatic-dev/jsonc-edit-core-ts/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import dedent from 'string-dedent';

import {
  CONFIG_FILENAME,
  type Hypervisor,
  type SyncState,
  type VmsyncConfig,
} from './types.ts';

/**
 Logger root for vmsync after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'vmsync', },);

//region Data directories

/**
 Root data directory for all vmsync-managed VMs.
 */
export const DATA_DIR: string = join(
  homedir(),
  '.local',
  'share',
  'vmsync',
);

/**
 Resolves the directory path for a named VM.
 
 @param name - VM name
 
 @returns Absolute path to the VM directory
 
 @example
 ```ts
 vmDir('alpine'); // "/home/user/.local/share/vmsync/alpine"
 ```
 */
export function vmDir(name: string,): string {
  return join(
    DATA_DIR,
    name,
  );
}

/**
 Resolves the config file path for a named VM.
 
 @param name - VM name
 
 @returns Absolute path to vmsync.jsonc
 
 @example
 ```ts
 vmConfigPath('alpine'); // "/home/user/.local/share/vmsync/alpine/vmsync.jsonc"
 ```
 */
export function vmConfigPath(name: string,): string {
  return join(
    vmDir(name,),
    CONFIG_FILENAME,
  );
}

//endregion Data directories

//region Name validation

/**
 Whether `c` is an ASCII alphanumeric character.
 
 @param c - single-character string to inspect
 
 @returns whether `c` is `[A-Za-z0-9]`
 
 @example
 ```ts
 isAlphaNum('a'); // true
 isAlphaNum('-'); // false
 ```
 */
function isAlphaNum(c: string,): boolean {
  return ((c >= 'a') && (c <= 'z'))
    || ((c >= 'A') && (c <= 'Z'))
    || ((c >= '0') && (c <= '9'));
}

/**
 Whether `c` is a character permitted in a VM name after its first character.
 The leading character has a stricter rule: it must be alphanumeric.
 
 @param c - single-character string to inspect
 
 @returns whether `c` is alphanumeric, underscore, or hyphen
 
 @example
 ```ts
 isVmNameChar('_'); // true
 isVmNameChar('.'); // false
 ```
 */
export function isVmNameChar(c: string,): boolean {
  return isAlphaNum(c,)
    || (c === '_')
    || (c === '-');
}

/**
 The naming rule a refusal states, written once so every refusal says the same
 thing.
 */
const NAME_RULE = 'A VM name starts with a letter or a digit and holds only letters, digits, "_" and "-" after that.';

/**
 First position of a VM name that the naming rule refuses.
 
 A name is refused for one of three reasons, and the refusal quotes the character
 that broke the rule, so the reader can see which character to change.
 */
export type VmNameDefect =
  | { readonly kind: 'empty' }
    | {
      readonly kind: 'first';
      readonly char: string;
    }
    | {
      readonly kind: 'later';
      readonly char: string;
    };

/**
 Where a VM name came from.
 
 A name vmsync derives from an image path is one the user never typed, so its
 refusal has to say where it came from and how to choose another. A name the user
 typed needs neither.
 */
export type VmNameOrigin =
  | { readonly kind: 'typed' }
    | {
      readonly kind: 'imagePath';
      readonly imagePath: string;
    };

/**
 Sentinel {@link findNameDefect} returns for a name that follows the rule, so
 acceptance is an explicit value rather than a nullish union.
 */
const NAME_FOLLOWS_RULE: unique symbol = Symbol('vmsync/name follows the rule',);

/**
 Finds the first character of `name` that the naming rule refuses, counting whole
 code points so a quoted character is never half of one.
 
 @param name - candidate VM name
 
 @returns the refused position, or {@link NAME_FOLLOWS_RULE} when the name follows the rule
 
 @example
 ```ts
 findNameDefect('my-vm');  // NAME_FOLLOWS_RULE
 findNameDefect('_my-vm'); // { kind: 'first', char: '_' }
 ```
 */
function findNameDefect(name: string,): VmNameDefect | typeof NAME_FOLLOWS_RULE {
  /**
   Characters of the candidate name, one entry per code point.
   */
  const chars = Array.from(name,);
  if (chars.length
    === 0)
    return { kind: 'empty', };
  for (const [index, char,] of chars.entries()) {
    if (index
      === 0) {
      if (!isAlphaNum(char,))
        return {
          kind: 'first',
          char,
        };
      continue;
    }
    if (!isVmNameChar(char,))
      return {
        kind: 'later',
        char,
      };
  }
  return NAME_FOLLOWS_RULE;
}

/**
 Renders one character for a refusal message. Whitespace is named as well as
 quoted, because a quoted space otherwise reads as empty quotes.
 
 @param char - single character to render
 
 @returns the character, quoted, with a name when it is whitespace
 
 @example
 ```ts
 describeChar(' '); // '" " (a space)'
 describeChar('_'); // '"_"'
 ```
 */
function describeChar(char: string,): string {
  if (char
    === ' ')
    return '" " (a space)';
  if (char
    === '\t')
    return String.raw`"\t" (a tab)`;
  if (char
    === '\n')
    return String.raw`"\n" (a newline)`;
  if (char
    === '\r')
    return String.raw`"\r" (a carriage return)`;
  return JSON.stringify(char,);
}

/**
 Writes the explanation a refusal carries: what was refused, the rule it broke,
 and every way to proceed.
 
 @param vmName - the refused name, exactly as vmsync received it
 
 @param defect - the position that broke the rule
 
 @param origin - whether the user typed the name or vmsync derived it
 
 @returns the message the caller throws
 
 @example
 ```ts
 describeRefusal({ vmName: '_alpine', defect: { kind: 'first', char: '_' }, origin: { kind: 'typed' } });
 ```
 */
function describeRefusal(
  {
    vmName,
    defect,
    origin,
  }: {
    readonly vmName: string;
    readonly defect: VmNameDefect;
    readonly origin: VmNameOrigin;
  },
): string {
  /**
   Sentence naming the character that broke the rule.
   */
  const found = (defect.kind
    === 'empty')
    ? 'This name is empty.'
    : (defect.kind
      === 'first')
      ? `This name starts with ${describeChar(defect.char,)}, which may not start a name.`
      : `This name holds ${describeChar(defect.char,)}, which a name may not hold.`;
  /**
   Sentence saying what to do next, which differs for a name nobody typed.
   */
  const remedy = (origin.kind
    === 'typed')
    ? 'Choose a different name.'
    : `vmsync took this name from the image file "${origin.imagePath}"; pass --name to choose the name yourself.`;
  return dedent`
    vmsync refused the VM name ${JSON.stringify(vmName,)}.
    ${NAME_RULE}
    ${found}
    ${remedy}
  `;
}

/**
 Error thrown when a VM name breaks the naming rule.
 
 The refusal carries the whole explanation in its message, so a caller that prints
 the message tells the user everything vmsync knows about the refusal.
 
 @example
 ```ts
 try {
   validateName({ name: '_alpine', origin: { kind: 'typed' } });
 }
 catch (error) {
   if (error instanceof InvalidVmNameError) console.error(error.message);
 }
 ```
 */
export class InvalidVmNameError extends Error {
  /**
   The refused name, exactly as vmsync received it.
   */
  readonly vmName: string;

  /**
   @param vmName - the refused name, exactly as vmsync received it
   
   @param defect - the position that broke the rule
   
   @param origin - whether the user typed the name or vmsync derived it
   */
  constructor({
    vmName,
    defect,
    origin,
  }: {
    readonly vmName: string;
    readonly defect: VmNameDefect;
    readonly origin: VmNameOrigin;
  },) {
    super(describeRefusal({
      vmName,
      defect,
      origin,
    },),);
    this.name = 'InvalidVmNameError';
    this.vmName = vmName;
  }
}

/**
 Validates a VM name against the naming rule.
 
 @param name - VM name to validate
 
 @param origin - whether the user typed the name or vmsync derived it, which the refusal states
 
 @throws InvalidVmNameError when the name breaks the naming rule
 
 @example
 ```ts
 validateName({ name: 'my-vm-01', origin: { kind: 'typed' } }); // OK
 validateName({ name: '../evil', origin: { kind: 'typed' } });  // throws
 ```
 */
export function validateName(
  {
    name,
    origin,
  }: {
    readonly name: string;
    readonly origin: VmNameOrigin;
  },
): void {
  /**
   Position that broke the rule, if the name breaks it.
   */
  const defect = findNameDefect(name,);
  if (defect === NAME_FOLLOWS_RULE)
    return;
  throw new InvalidVmNameError({
    vmName: name,
    defect,
    origin,
  },);
}

//endregion Name validation

//region JSONC parsing

/**
 Parses the contents of a `vmsync.jsonc` file into a {@link VmsyncConfig}.
 
 Comments and trailing commas are accepted, as the README documents, because the
 shared parser reads them as data instead of stripping them first. Text that is
 not a JSONC object is refused by the parser with a `JsoncParseError` naming the
 byte offset of the fault.
 
 @param source - File contents read verbatim, comments included
 
 @returns Parsed {@link VmsyncConfig}
 
 @throws JsoncParseError when the text is not a JSONC object
 
 @example
 ```ts
 const cfg = parseVmsyncConfig({ source: '{ // boot\n "name": "alpine", }' });
 ```
 */
export function parseVmsyncConfig(
  { source, }: {
    readonly source: string;
  },
): VmsyncConfig {
  /**
   Plain JSON value of the whole document; the empty path addresses the root.
   */
  const value = jsoncGetValue({
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the JSONC brand is a compile-time marker on an ordinary string, and this is the file boundary where an unbranded string arrives
    state: parseJsoncEdit({ source: source as StringJsonc, },),
    path: [],
  },);
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted JSONC config vmsync wrote
  return value as VmsyncConfig;
}

//endregion JSONC parsing

//region Config read/write

/**
 Reads and parses the vmsync.jsonc config for a named VM.
 
 @param name - VM name
 
 @returns Parsed {@link VmsyncConfig}
 
 @throws Error when the config file does not exist
 
 @throws JsoncParseError when the file is not a JSONC object
 
 @example
 ```ts
 const cfg = await readConfig('alpine');
 console.log(cfg.boot.memory);
 ```
 */
export async function readConfig(name: string,): Promise<VmsyncConfig> {
  /**
   Tagged logger so read-config entries are scoped to `readConfig` in the output.
   */
  const rl = tagged({
    tag: readConfig.name,
    l,
  },);
  /**
   Raw JSONC content from disk.
   */
  const raw = await readFile(
    vmConfigPath(name,),
    'utf8',
  );
  /**
   Parsed configuration object.
   */
  const config = parseVmsyncConfig({ source: raw, },);
  rl.info(`loaded config for "${name}"`,);
  return config;
}

/**
 Serializes and writes the vmsync.jsonc config for a named VM.
 Creates the VM directory if it does not exist.
 
 @param name - VM name
 
 @param config - {@link VmsyncConfig} to persist
 
 @mutates config - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 await writeConfig({ name: 'alpine', config });
 ```
 */
export async function writeConfig(
  {
    name,
    config,
  }: {
    readonly name: string;
    config: Omit<VmsyncConfig, 'state'> & {
      readonly state: Readonly<SyncState>;
    };
  },
): Promise<void> {
  /**
   Tagged logger so write-config entries are scoped to `writeConfig` in the output.
   */
  const rl = tagged({
    tag: writeConfig.name,
    l,
  },);
  /**
   Target directory for this VM.
   */
  const dir = vmDir(name,);
  await mkdir(
    dir,
    { recursive: true, },
  );
  /**
   Formatted JSON with 2-space indent.
   */
  const serialized = JSON.stringify(
    config,
    undefined,
    2,
  );
  await writeFile(
    vmConfigPath(name,),
    serialized,
    'utf8',
  );
  rl.info(`wrote config for "${name}"`,);
}

//endregion Config read/write

//region Platform detection

/**
 Detects the available hypervisor on the current platform.
 
 @returns {@link Hypervisor}: `'kvm'` on Linux, `'hyperv'` on Windows
 
 @throws Error when running on an unsupported platform
 
 @example
 ```ts
 const hv = detectHypervisor();
 // 'kvm' on Linux, 'hyperv' on Windows
 ```
 */
export function detectHypervisor(): Hypervisor {
  if (process.platform
    === 'linux')
    return 'kvm';
  if (process.platform
    === 'win32')
    return 'hyperv';
  throw new Error(
    `unsupported platform: ${process.platform} (vmsync requires Linux with KVM or Windows with Hyper-V)`,
  );
}

//endregion Platform detection

//region Default boot config

/**
 Default memory allocation for new VMs.
 */
export const DEFAULT_MEMORY = '4G';

/**
 Default number of virtual CPUs for new VMs.
 */
export const DEFAULT_CPUS = 4;

//endregion Default boot config
