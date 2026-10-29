/**
 The rustup toolchain the repository pins in `toolchain.lock.json`.

 The update passes `-Zunstable-options`, which only nightly cargo accepts,
 and the repository builds everything with the pinned nightly. Outside a
 shell where mise exports `RUSTUP_TOOLCHAIN` (a systemd unit, a bare CI
 step), rustup falls back to its default toolchain, which may be stable, so
 the update pins the toolchain itself. An explicit `RUSTUP_TOOLCHAIN` in the
 environment still wins.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { l, } from './logger.ts';

//region Toolchain

/**
 Lock file name at the repository root.
 */
const TOOLCHAIN_LOCK = 'toolchain.lock.json';

/**
 Whether `value` is a plain object.

 @param value - parsed JSON

 @returns whether it is a non-array object

 @example
 ```ts
 isObject({}); // true
 ```
 */
function isObject(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Rust toolchain from `toolchain.lock.json` text.

 @param text - lock file body

 @returns `.tools.rust.version`, or empty when absent

 @throws SyntaxError when the text is not JSON

 @example
 ```ts
 rustToolchainFromLock('{"tools":{"rust":{"version":"nightly-2026-10-29"}}}'); // 'nightly-2026-10-29'
 ```
 */
export function rustToolchainFromLock(text: string,): string {
  /**
   Parsed lock.
   */
  const lock: unknown = JSON.parse(text,);
  /**
   Tool table.
   */
  const tools = isObject(lock,) && isObject(lock.tools,) ? lock.tools : {};
  /**
   Rust entry.
   */
  const rust = isObject(tools.rust,) ? tools.rust : {};
  return (typeof rust.version) === 'string' ? rust.version : '';
}

/**
 Toolchain cargo should run under: `RUSTUP_TOOLCHAIN` when set, else the repository's pin.

 @param root - repository root

 @param explicit - `RUSTUP_TOOLCHAIN` from the environment, empty when unset

 @returns toolchain name, or empty when neither source names one

 @throws SyntaxError when `toolchain.lock.json` is not JSON

 @example
 ```ts
 await cargoToolchain({ root: process.cwd(), explicit: process.env.RUSTUP_TOOLCHAIN ?? '', });
 ```
 */
export async function cargoToolchain(
  {
    root,
    explicit,
  }: {
    readonly root: string;
    readonly explicit: string;
  },
): Promise<string> {
  /**
   Logger for the choice.
   */
  const log = tagged({
    tag: cargoToolchain.name,
    l,
  },);
  if (explicit !== '') {
    log.debug(`using RUSTUP_TOOLCHAIN=${explicit} from the environment`,);
    return explicit;
  }
  try {
    /**
     Pinned toolchain.
     */
    const pinned = rustToolchainFromLock(await readFile(
      join(
        root,
        TOOLCHAIN_LOCK,
      ),
      'utf8',
    ),);
    log.debug(`using ${pinned === '' ? 'rustup\'s default toolchain' : pinned} from ${TOOLCHAIN_LOCK}`,);
    return pinned;
  } catch (error) {
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT')) {
      log.warn(`no ${TOOLCHAIN_LOCK} at ${root}; cargo runs under rustup's default toolchain, which must be nightly`,);
      return '';
    }
    throw error;
  }
}

//endregion Toolchain
