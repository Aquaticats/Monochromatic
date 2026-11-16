/**
 The Rust toolchain `toolchain.lock.json` records, and the `rustup` command that installs it.

 The lock holds one Rust toolchain for every build of this repository: `.tools.rust.version`, a dated nightly the daily
 update moves forward (`doc/handover/cli-git-rust-implementation.md`, owner decision of 2026-11-15: "Everything nightly.
 This doesn't break "pacman for everything it has" if you're implying pacman doesn't have nightly. And the lock's nightly
 is always the latest nightly anyway."). A workflow that typed the date itself would drift away from it silently, and one
 that installed the distant `nightly` channel built with a compiler no run here tested.

 This module is the reading and the command line; `rust-toolchain-setup.ts` is the step that runs them.
 */

//region The row and the names it may hold

/**
 Where the toolchain's version is recorded, as a `jq` path into the lock: the same row
 `.github/workflows/music-player-release.yml` and the container gate read.
 */
export const RUST_TOOLCHAIN_ROW = '.tools.rust.version';

/**
 Prefix of a dated nightly, the form the lock holds.
 */
const DATED_NIGHTLY_PREFIX = 'nightly-';

/**
 Number of parts in a date, `2026-11-15`, and in a release, `1.99.0`.
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
 Digits in each part of a dated nightly's date, year first.
 */
const DATE_PART_DIGITS = [
  YEAR_DIGITS,
  MONTH_OR_DAY_DIGITS,
  MONTH_OR_DAY_DIGITS,
] as const;

/**
 Thrown when the lock holds no Rust toolchain version, or one no `rustup` install can name.
 */
export class RustToolchainPinError extends Error {
  /**
   Stable class name that tells this reading apart from an unrelated error in a step's log.
   */
  override name = 'RustToolchainPinError';
}

/**
 Whether a text is one or more ASCII digits and nothing else.

 @param text - Text to read

 @returns whether the text is at least one digit and holds no other character

 @example
 ```ts
 isDigits('2026'); // true
 isDigits('20a6'); // false
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
 Whether `rustup toolchain install` can be given a text as a toolchain name.

 A dated nightly is what the lock holds; a release number is the other form the field may take, which the workflows
 accepted before the pin existed and which an older lock still may.

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

//endregion The row and the names it may hold

//region Reading the lock

/**
 Reads the Rust toolchain version out of the parsed `toolchain.lock.json`.

 @param lock - The parsed lock, as any JSON value

 @returns the version the lock records

 @throws {@link RustToolchainPinError} When the lock is not an object, holds no Rust tool at `.tools.rust`, records no
 version there, or records one that is neither a dated nightly nor a release

 @example
 ```ts
 const version = lockedRustToolchainVersion({ lock: { tools: { rust: { version: 'nightly-2026-11-15' } } } });
 ```
 */
export function lockedRustToolchainVersion({ lock, }: { readonly lock: unknown },): string {
  if (((typeof lock) !== 'object') || (lock === null)
    || Array.isArray(lock,))
    throw new RustToolchainPinError(
      `The lock is not a JSON object, so it records no ${RUST_TOOLCHAIN_ROW}. Restore toolchain.lock.json from git, or re-resolve it with 'mise run toolchain:update'.`,
    );
  /**
   The Rust tool's row of the lock, which holds the version.
   */
  const rust = Reflect.get(
    Reflect.get(
      lock,
      'tools',
    ) ?? {},
    'rust',
  ) as unknown;
  if (((typeof rust) !== 'object') || (rust === null))
    throw new RustToolchainPinError(
      `The lock has no "rust" tool under "tools", so it records no ${RUST_TOOLCHAIN_ROW}. Re-resolve it with 'mise run toolchain:update'.`,
    );
  /**
   The recorded version, of any JSON type.
   */
  const version: unknown = Reflect.get(
    rust,
    'version',
  );
  if ((typeof version) !== 'string')
    throw new RustToolchainPinError(
      `The lock's Rust tool records no version string (${JSON.stringify(version,) ?? 'nothing'}), so ${RUST_TOOLCHAIN_ROW} is absent. Re-resolve it with 'mise run toolchain:update'.`,
    );
  if (!isInstalableToolchain(version,))
    throw new RustToolchainPinError(
      `The lock records the Rust toolchain "${version}" in ${RUST_TOOLCHAIN_ROW}, which is neither a dated nightly (nightly-2026-11-15) nor a release (1.99.0). Write the dated nightly the daily update resolves, or install it by hand and record that name; the floating "nightly" channel is a compiler no run of this repository tested.`,
    );
  return version;
}

//endregion Reading the lock

//region The commands the step runs

/**
 Arguments of the `rustup toolchain install` command that installs a toolchain, without the toolchain's own name.

 @returns the profile and self-update arguments, which every workflow of this repository installs with

 @example
 ```ts
 rustupInstallArguments({ version: 'nightly-2026-11-15' }); // ['toolchain', 'install', 'nightly-2026-11-15', ...]
 ```
 */
export function rustupInstallArguments({ version, }: { readonly version: string },): readonly string[] {
  return [
    'toolchain',
    'install',
    version,
    // The minimal profile carries cargo, rustc, rustdoc and the standard library; a job adds a component or a target
    // for itself, in a step of its own, after this one.
    '--profile',
    'minimal',
    '--no-self-update',
  ];
}

/**
 The line that makes every later step of a job compile with the toolchain, written to the runner's `GITHUB_ENV` file.

 `rustup` answers a shim on the runner's `PATH` with this toolchain, and applies it to `rustup target add` as well, so a
 job names the toolchain nowhere else.

 @param version - Toolchain the lock records

 @returns the assignment and its line break

 @example
 ```ts
 rustupToolchainEnvironmentLine({ version: 'nightly-2026-11-15' }); // 'RUSTUP_TOOLCHAIN=nightly-2026-11-15\n'
 ```
 */
export function rustupToolchainEnvironmentLine({ version, }: { readonly version: string },): string {
  return `RUSTUP_TOOLCHAIN=${version}\n`;
}

//endregion The commands the step runs
