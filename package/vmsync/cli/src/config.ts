/**
 VM configuration persistence and platform detection.
 
 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

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
 Whether `c` is allowed in non-leading positions of a VM name.
 
 @param c - single-character string to inspect
 
 @returns whether `c` is alphanumeric, underscore, or hyphen
 
 @example
 ```ts
 isNameBodyChar('_'); // true
 isNameBodyChar('.'); // false
 ```
 */
function isNameBodyChar(c: string,): boolean {
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
    if (!isNameBodyChar(c,))
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
 Strips single-line (`//`) and multi-line (`/* *\/`) comments from a JSONC string,
 preserving string literals that contain comment-like sequences.
 
 @param text - JSONC source text
 
 @returns JSON-parseable string with comments removed
 
 @example
 ```ts
 stripJsoncComments('{ "a": 1 // comment\n}'); // '{ "a": 1 \n}'
 ```
 */
export function stripJsoncComments(text: string,): string {
  /**
   Result accumulator built character-by-character.
   */
  const result: string[] = [];
  /**
   Current position in the source text.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- parser cursor advances by 1 or 2 depending on which side-effecting branch fires
  let i = 0;
  /**
   Whether we are currently inside a double-quoted string.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- string-mode flag toggled across multiple branches of the parser state machine
  let inString = false;

  while (i < text
    .length) {
    /**
     Character at the current position.
     */
    const ch = nonNullishOrThrow(text[i],);

    if (inString) {
      if (ch === '\\') {
        /**
         Escaped character pair.
         */
        const next = nonNullishOrThrow(text[i + 1],);
        result.push(
          ch,
          next,
        );
        i += 2;
      }
      else {
        if (ch === '"')
          inString = false;
        result.push(ch,);
        i += 1;
      }
    }
    else if (ch === '"') {
      inString = true;
      result.push(ch,);
      i += 1;
    }
    else if ((ch === '/') && (text[i + 1]
      === '/')) {
      while ((i < text
        .length) && (text[i]
          !== '\n'))
        i += 1;
    }
    else {
      if ((ch === '/') && (text[i + 1]
        === '*')) {
        i += 2;
        while ((i < text
          .length) && (!((text[i - 1]
            === '*') && (text[i]
              === '/'))))
          i += 1;
      }
      else
        result.push(ch,);
      i += 1;
    }
  }

  return result.join('',);
}

//endregion JSONC parsing

//region Config read/write

/**
 Reads and parses the vmsync.jsonc config for a named VM.
 
 @param name - VM name
 
 @returns Parsed {@link VmsyncConfig}
 
 @throws Error when the config file does not exist or contains invalid JSON
 
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
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- trusted JSONC config we wrote
  const config = JSON.parse(stripJsoncComments(raw,),) as VmsyncConfig;
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
