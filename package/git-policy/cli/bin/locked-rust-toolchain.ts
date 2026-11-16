/**
 The Rust toolchain `toolchain.lock.json` records, which the container images that compile the wrapper install.

 The lock holds one Rust toolchain for every build of this repository: `.tools.rust.version`, a dated nightly the daily
 update moves forward. The container images take every tool they can from pacman (owner decisions of 2026-11-04,
 "Pacman for everything it has"), and a dated nightly is the one thing pacman does not have, so the images install
 exactly the name this module reads instead of a compiler typed into a Containerfile (owner decision of 2026-11-15:
 "Everything nightly. This doesn't break "pacman for everything it has" if you're implying pacman doesn't have nightly.
 And the lock's nightly is always the latest nightly anyway.").

 The base image's tag covers the value, so a lock that moves the date builds a new base rather than reusing one compiled
 by the night before (`testBaseTag`).
 */
import { readFile, } from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { NativeVerificationError, } from './native-verification-process.ts';

//region The lock and the row

/**
 File name of the repository lock, which stands at the repository root.
 */
export const LOCK_FILE_NAME = 'toolchain.lock.json';

/**
 Row of the lock that names the toolchain, as a `jq` path: the same row the workflows read with
 `.github/actions/rust-toolchain-setup`.
 */
export const RUST_TOOLCHAIN_ROW = '.tools.rust.version';

/**
 Repository root, four directories above this module's folder (`package/git-policy/cli/bin`).
 */
const repositoryRoot = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
  '..',
);

/**
 Prefix of a dated nightly, the form the lock holds.
 */
const DATED_NIGHTLY_PREFIX = 'nightly-';

/**
 Parts of a date (`2026-11-15`) and of a release (`1.99.0`).
 */
const VERSION_PART_COUNT = 3;

/**
 Digits of a year in a dated nightly.
 */
const YEAR_DIGITS = 4;

/**
 Digits of a month and of a day in a dated nightly.
 */
const MONTH_OR_DAY_DIGITS = 2;

/**
 Digits in each part of a date, year first.
 */
const DATE_PART_DIGITS = [
  YEAR_DIGITS,
  MONTH_OR_DAY_DIGITS,
  MONTH_OR_DAY_DIGITS,
] as const;

/**
 The value one read produced, so a gate that asks twice does not read the file twice.
 */
const reads = new Map<string, string>();

//endregion The lock and the row

//region Reading the name

/**
 Whether a text is one or more ASCII digits and nothing else.

 @param text - Text to read

 @returns whether the text is at least one digit and holds no other character

 @example
 ```ts
 isDigits('2026'); // true
 ```
 */
function isDigits(text: string,): boolean {
  if (text.length === 0)
    return false;
  for (const character of text) {
    if ((character < '0') || (character > '9'))
      return false;
  }
  return true;
}

/**
 Whether a text is a date written as the lock writes one, `2026-11-15`.

 @param text - Text to read

 @returns whether the text is three digit groups of the widths a date has

 @example
 ```ts
 isDate('2026-11-15'); // true
 ```
 */
function isDate(text: string,): boolean {
  /**
   Year, month and day.
   */
  const parts = text.split('-',);
  if (parts.length !== VERSION_PART_COUNT)
    return false;
  for (const [index, part,] of parts.entries()) {
    if ((part.length !== DATE_PART_DIGITS[index]) || (!isDigits(part,)))
      return false;
  }
  return true;
}

/**
 Whether a text is a release number, `1.99.0`.

 @param text - Text to read

 @returns whether the text is three digit groups

 @example
 ```ts
 isRelease('1.99.0'); // true
 ```
 */
function isRelease(text: string,): boolean {
  /**
   Major, minor and patch.
   */
  const parts = text.split('.',);
  return (parts.length === VERSION_PART_COUNT) && parts.every(isDigits,);
}

/**
 Whether `rustup toolchain install` accepts a text as a toolchain name.

 @param text - Text to read

 @returns whether the text names a dated nightly or a release

 @example
 ```ts
 isInstalableToolchain('nightly-2026-11-15'); // true
 isInstalableToolchain('nightly'); // false
 ```
 */
export function isInstalableToolchain(text: string,): boolean {
  if (text.startsWith(DATED_NIGHTLY_PREFIX,))
    return isDate(text.slice(DATED_NIGHTLY_PREFIX.length,),);
  return isRelease(text,);
}

/**
 Reads the toolchain's version out of the parsed lock.

 @param lock - The parsed lock, as any JSON value

 @returns the dated nightly or release the lock records

 @throws {@link NativeVerificationError} When the lock is not an object, holds no Rust tool, records no version there, or
 records one that is neither a dated nightly nor a release

 @example
 ```ts
 const version = rustToolchainOfLock({ lock: { tools: { rust: { version: 'nightly-2026-11-15' } } } });
 ```
 */
export function rustToolchainOfLock({ lock, }: {
  readonly lock: unknown;
},): string {
  if (((typeof lock) !== 'object') || (lock === null)
    || Array.isArray(lock,))
    throw new NativeVerificationError(
      `${LOCK_FILE_NAME} is not a JSON object, so it records no ${RUST_TOOLCHAIN_ROW}. Restore the file from git, or re-resolve it with 'mise run toolchain:update'.`,
    );
  /**
   The Rust tool's row, which holds the version.
   */
  const rust: unknown = Reflect.get(
    Reflect.get(
      lock,
      'tools',
    ) ?? {},
    'rust',
  );
  if (((typeof rust) !== 'object') || (rust === null))
    throw new NativeVerificationError(
      `${LOCK_FILE_NAME} has no "rust" tool under "tools", so it records no ${RUST_TOOLCHAIN_ROW}. Re-resolve it with 'mise run toolchain:update'.`,
    );
  /**
   The recorded version, of any JSON type.
   */
  const version: unknown = Reflect.get(
    rust,
    'version',
  );
  if ((typeof version) !== 'string')
    throw new NativeVerificationError(
      `${LOCK_FILE_NAME} records no Rust version string in ${RUST_TOOLCHAIN_ROW} (${JSON.stringify(version,) ?? 'nothing'}). Re-resolve it with 'mise run toolchain:update'.`,
    );
  if (!isInstalableToolchain(version,))
    throw new NativeVerificationError(
      `${LOCK_FILE_NAME} records the Rust toolchain "${version}" in ${RUST_TOOLCHAIN_ROW}, which is neither a dated nightly (nightly-2026-11-15) nor a release (1.99.0). Write the dated nightly the daily update resolves; a floating channel is a compiler no script here can pin an image to.`,
    );
  return version;
}

/**
 Reads the lock's text, naming the file and the fix when it cannot be read.

 @param path - Path of the lock

 @returns the file's text

 @throws {@link NativeVerificationError} When the file cannot be read

 @example
 ```ts
 const text = await lockText('/repo/toolchain.lock.json');
 ```
 */
async function lockText(path: string,): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    throw new NativeVerificationError(
      `${path} could not be read (${String(error,)}). A gate runs from a checkout of this repository; restore the file from git.`,
    );
  }
}

/**
 Parses the lock's text, naming the file and the fix when it is not JSON.

 @param path - Path of the lock, for the message

 @param text - The file's text

 @returns the parsed lock

 @throws {@link NativeVerificationError} When the text is not JSON

 @example
 ```ts
 const lock = parseLock({ path: '/repo/toolchain.lock.json', text: '{}' });
 ```
 */
function parseLock({
  path,
  text,
}: {
  readonly path: string;
  readonly text: string;
},): unknown {
  try {
    return JSON.parse(text,) as unknown;
  }
  catch (error) {
    throw new NativeVerificationError(
      `${path} is not valid JSON (${String(error,)}). Restore the file from git, or re-resolve it with 'mise run toolchain:update'.`,
    );
  }
}

/**
 Reads the Rust toolchain version the repository's lock records, once per process.

 @returns the dated nightly or release the lock records

 @throws {@link NativeVerificationError} When the lock cannot be read, is not JSON, or records no usable version

 @example
 ```ts
 const toolchain = await lockedRustToolchain();
 ```
 */
export async function lockedRustToolchain(): Promise<string> {
  /**
   Path of the lock at the repository root, wherever the caller's working directory is.
   */
  const path = join(
    repositoryRoot,
    LOCK_FILE_NAME,
  );
  /**
   A version an earlier call in this process read.
   */
  const remembered = reads.get(path,);
  if (remembered !== undefined)
    return remembered;
  /**
   The parsed lock.
   */
  const lock = parseLock({
    path,
    text: await lockText(path,),
  },);
  /**
   The version the lock records.
   */
  const version = rustToolchainOfLock({ lock, },);
  reads.set(
    path,
    version,
  );
  return version;
}

//endregion Reading the name
