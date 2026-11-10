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
 Checks whether `name` matches the original regex
 `/^[A-Za-z0-9][A-Za-z0-9_-]*$/`.
 
 Empty input fails the leading-alphanumeric requirement; all subsequent
 characters must be alphanumeric, underscore, or hyphen.
 
 @param name - candidate VM name
 
 @returns whether name is a valid VM identifier
 
 @example
 ```ts
 isValidVmName('my-vm-01'); // true
 isValidVmName('-leading'); // false
 isValidVmName('');         // false
 ```
 */
function isValidVmName(name: string,): boolean {
  if (name.length
    === 0)
    return false;
  if (!isAlphaNum(name.charAt(0,),))
    return false;
  for (const c of name.slice(1,)) {
    if (!isVmNameChar(c,))
      return false;
  }
  return true;
}

/**
 Validates a VM name contains only safe characters.
 
 @param name - VM name to validate
 
 @throws Error when name contains invalid characters
 
 @example
 ```ts
 validateName('my-vm-01'); // OK
 validateName('../evil');  // throws
 ```
 */
export function validateName(name: string,): void {
  if (!isValidVmName(name,)) {
    throw new Error(
      `invalid VM name: ${name} (must start with alphanumeric, contain only alphanumerics, hyphens, underscores)`,
    );
  }
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
