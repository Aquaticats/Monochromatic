/**
 The rustup toolchain the repository pins in `toolchain.lock.json`.

 The update passes `-Zunstable-options`, which only nightly cargo accepts,
 and the repository builds everything with the pinned nightly. Outside a
 shell where mise exports `RUSTUP_TOOLCHAIN` (a systemd unit, a bare CI
 step), the toolchain is whatever the environment says: on the development
 host the systemd user manager exported `RUSTUP_TOOLCHAIN=1.98.1` (stable),
 which broke the first real run. So the repository's pin wins over an
 inherited `RUSTUP_TOOLCHAIN`; the inherited value applies only when the
 repository pins no Rust toolchain.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

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
 Rust toolchain the repository pins, or empty when there is no lock file or no Rust entry.

 @param root - repository root

 @returns pinned toolchain name

 @throws SyntaxError when `toolchain.lock.json` is not JSON

 @example
 ```ts
 await pinnedToolchain('/repo'); // 'nightly-2026-10-29'
 ```
 */
async function pinnedToolchain(root: string,): Promise<string> {
  /**
   Logger for a missing lock.
   */
  const log = tagged({
    tag: pinnedToolchain.name,
    l,
  },);
  try {
    return rustToolchainFromLock(await readFile(
      join(
        root,
        TOOLCHAIN_LOCK,
      ),
      'utf8',
    ),);
  } catch (error) {
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT')) {
      log.debug(`no ${TOOLCHAIN_LOCK} at ${root}`,);
      return '';
    }
    throw error;
  }
}

/**
 Toolchain cargo should run under: the repository's pin, else an inherited `RUSTUP_TOOLCHAIN`.

 @param root - repository root

 @param explicit - `RUSTUP_TOOLCHAIN` from the environment, empty when unset

 @returns toolchain name, or empty when neither source names one (rustup's default then applies)

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
  /**
   Repository pin.
   */
  const pinned = await pinnedToolchain(root,);
  if (pinned !== '') {
    if ((explicit !== '') && (explicit !== pinned))
      log.warn(`ignoring RUSTUP_TOOLCHAIN=${explicit} from the environment; ${TOOLCHAIN_LOCK} pins ${pinned}`,);
    return pinned;
  }
  if (explicit !== '')
    return explicit;
  log.warn(`${root} pins no Rust toolchain and RUSTUP_TOOLCHAIN is unset; cargo runs under rustup's default, which must be nightly`,);
  return '';
}

//endregion Toolchain
