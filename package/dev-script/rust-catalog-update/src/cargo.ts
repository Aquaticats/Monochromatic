/**
 Cargo invocations for one crate root.

 The update re-resolves each lockfile with
 `cargo -Zunstable-options generate-lockfile --publish-time <cutoff>`, which
 picks the newest versions published before the cutoff (pnpm's
 `minimumReleaseAge` semantics). Only `generate-lockfile` accepts
 `--publish-time` (re-probed 2026-10-29 on the pinned nightly), so a
 selective `cargo update` cannot carry the cutoff. Crates the catalog exempts
 from the release age then move to their newest release with `cargo update`.

 @module
 */

import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  isRegistryPackage,
  type LockedPackage,
  parseCargoLock,
} from '@monochromatic-dev/module-rust-catalog/ts';
import spawn, { SubprocessError, } from 'nano-spawn';

import { l, } from './logger.ts';
import {
  matchesPattern,
  type ParsedSelector,
  parseSelector,
} from './selectors.ts';

//region Runner

/**
 Outcome of one cargo run.

 @example
 ```ts
 const outcome: CargoOutcome = { ok: true, output: '', };
 ```
 */
export type CargoOutcome = {
  readonly ok: boolean;
  /**
   Interleaved stdout and stderr.
   */
  readonly output: string;
};

/**
 Runs cargo with arguments in a directory.

 @example
 ```ts
 const runCargo: RunCargo = async ({ cwd, args, }) => ({ ok: true, output: '', });
 ```
 */
export type RunCargo = (request: {
  readonly cwd: string;
  readonly args: readonly string[]
},) => Promise<CargoOutcome>;

/**
 Default {@link RunCargo} through `nano-spawn`.

 @param cwd - directory cargo runs in

 @param args - cargo arguments

 @returns whether cargo exited 0, with its output

 @throws Error when cargo cannot be started at all

 @example
 ```ts
 await spawnCargo({ cwd: 'package/cli/a', args: ['generate-lockfile',], });
 ```
 */
export async function spawnCargo({
  cwd,
  args,
}: {
  readonly cwd: string;
  readonly args: readonly string[]
},): Promise<CargoOutcome> {
  /**
   Logger for cargo runs.
   */
  const log = tagged({
    tag: spawnCargo.name,
    l,
  },);
  log.debug(`cargo ${args.join(' ',)} in ${cwd}`,);
  try {
    /**
     Finished run.
     */
    const result = await spawn(
      'cargo',
      [...args,],
      { cwd, },
    );
    return {
      ok: true,
      output: result.output,
    };
  } catch (error) {
    if ((error instanceof SubprocessError) && (error.exitCode !== undefined)) {
      log.debug(`cargo ${args.join(' ',)} in ${cwd} exited ${String(error.exitCode,)}`,);
      return {
        ok: false,
        output: error.output,
      };
    }
    throw error;
  }
}

//endregion Runner

//region Errors

/**
 Thrown when cargo cannot resolve a crate's dependencies.

 @example
 ```ts
 throw new CargoResolveError({ crateDir: 'package/cli/a', command: 'cargo generate-lockfile', output: '...', });
 ```
 */
export class CargoResolveError extends Error {
  /**
   Repo-relative crate directory.
   */
  readonly crateDir: string;

  /**
   Builds the error.

   @param crateDir - crate directory

   @param command - command that failed

   @param output - cargo output
   */
  constructor({
    crateDir,
    command,
    output,
  }: {
    readonly crateDir: string;
    readonly command: string;
    readonly output: string
  },) {
    super([
      `\`${command}\` failed in ${crateDir}; every lockfile was restored.`,
      'Cargo output:',
      output.trim(),
      'If no release old enough satisfies a requirement, either wait, add the crate to minimumReleaseAgeExclude in rust-catalog.toml,',
      'or set minimumReleaseAgeStrict = false to fall back to a younger release.',
    ].join('\n',),);
    this.name = 'CargoResolveError';
    this.crateDir = crateDir;
  }
}

//endregion Errors

//region Resolve

/**
 Milliseconds per minute.
 */
const MS_PER_MINUTE = 60_000;

/**
 Length of `yyyy-mm-ddThh:mm:ss` in an ISO timestamp.
 */
const ISO_SECONDS_LENGTH = 19;

/**
 Cutoff for `--publish-time`: `now` minus the release age, in cargo's `yyyy-mm-ddThh:mm:ssZ` form.

 @param now - current time in milliseconds

 @param minutes - minimum release age

 @returns cutoff timestamp

 @example
 ```ts
 publishTimeCutoff({ now: Date.parse('2026-10-29T12:00:00Z'), minutes: 1440, }); // '2026-10-28T12:00:00Z'
 ```
 */
export function publishTimeCutoff({
  now,
  minutes,
}: {
  readonly now: number;
  readonly minutes: number
},): string {
  return `${new Date(now - (minutes * MS_PER_MINUTE),).toISOString()
    .slice(
      0,
      ISO_SECONDS_LENGTH,
    )}Z`;
}

/**
 Re-resolves one crate's lockfile under the release-age cutoff.

 @param root - repository root

 @param crateDir - repo-relative crate directory

 @param cutoff - `--publish-time` value; empty to resolve without one

 @param strict - fail instead of falling back to an unconstrained resolution

 @param runCargo - cargo runner

 @returns whether the resolution fell back to ignoring the cutoff

 @throws CargoResolveError when cargo cannot resolve

 @example
 ```ts
 await resolveCrate({ root, crateDir: 'package/cli/a', cutoff: '2026-10-28T12:00:00Z', strict: true, runCargo: spawnCargo, });
 ```
 */
export async function resolveCrate(
  {
    root,
    crateDir,
    cutoff,
    strict,
    runCargo,
  }: {
    readonly root: string;
    readonly crateDir: string;
    readonly cutoff: string;
    readonly strict: boolean;
    readonly runCargo: RunCargo;
  },
): Promise<{ readonly fellBack: boolean; }> {
  /**
   Logger for this crate.
   */
  const log = tagged({
    tag: resolveCrate.name,
    l,
  },);
  /**
   Absolute crate directory.
   */
  const cwd = join(
    root,
    crateDir,
  );
  /**
   Constrained resolution arguments.
   */
  const constrained = cutoff === '' ? ['generate-lockfile',] : [
    '-Zunstable-options',
    'generate-lockfile',
    '--publish-time',
    cutoff,
  ];
  /**
   First attempt.
   */
  const first = await runCargo({
    cwd,
    args: constrained,
  },);
  if (first.ok)
    return { fellBack: false, };
  if (strict || (cutoff === ''))
    throw new CargoResolveError({
      crateDir,
      command: `cargo ${constrained.join(' ',)}`,
      output: first.output,
    },);
  log.warn(`${crateDir}: no resolution satisfies the release-age cutoff ${cutoff}; minimumReleaseAgeStrict is false, so resolving without it`,);
  /**
   Unconstrained fallback.
   */
  const fallback = await runCargo({
    cwd,
    args: ['generate-lockfile',],
  },);
  if (!fallback.ok)
    throw new CargoResolveError({
      crateDir,
      command: 'cargo generate-lockfile',
      output: fallback.output,
    },);
  return { fellBack: true, };
}

/**
 `cargo update` specs for locked crates exempt from the release age by a name or pattern selector.

 Selectors that name exact versions only exempt those versions from the check;
 they do not move a crate, because the cutoff already decided its version.

 @param lock - `Cargo.lock` text after the constrained resolution

 @param selectors - `minimumReleaseAgeExclude`

 @returns `name@version` specs, sorted

 @throws TomlEditError when the lockfile is not valid TOML

 @example
 ```ts
 excludedUpdateSpecs({ lock, selectors: ['ring',], }); // ['ring@0.17.8']
 ```
 */
export function excludedUpdateSpecs({
  lock,
  selectors,
}: {
  readonly lock: string;
  readonly selectors: readonly string[]
},): readonly string[] {
  /**
   Name patterns that exempt every version.
   */
  const patterns = selectors.map(function parse(selector: string,): ParsedSelector {
    return parseSelector(selector,);
  },)
    .filter(function isNameOnly(selector: ParsedSelector,): boolean {
      return selector.versions
        .length
        === 0;
    },)
    .map(function toPattern(selector: ParsedSelector,): string {
      return selector.pattern;
    },);
  if (patterns.length === 0)
    return [];
  return [...new Set(parseCargoLock(lock,)
    .filter(function isRegistry(locked: LockedPackage,): boolean {
      return isRegistryPackage(locked,);
    },)
    .filter(function isExempt(locked: LockedPackage,): boolean {
      return patterns.some(function matches(pattern: string,): boolean {
        return matchesPattern({
          pattern,
          name: locked.name,
        },);
      },);
    },)
    .map(function toSpec(locked: LockedPackage,): string {
      return `${locked.name}@${locked.version}`;
    },),),].toSorted();
}

/**
 Moves exempt crates to their newest compatible release.

 @param root - repository root

 @param crateDir - repo-relative crate directory

 @param specs - `name@version` specs from {@link excludedUpdateSpecs}

 @param runCargo - cargo runner

 @throws CargoResolveError when cargo cannot update them

 @example
 ```ts
 await updateExempt({ root, crateDir, specs: ['ring@0.17.8',], runCargo: spawnCargo, });
 ```
 */
export async function updateExempt(
  {
    root,
    crateDir,
    specs,
    runCargo,
  }: {
    readonly root: string;
    readonly crateDir: string;
    readonly specs: readonly string[];
    readonly runCargo: RunCargo;
  },
): Promise<void> {
  if (specs.length === 0)
    return;
  /**
   Update outcome.
   */
  const outcome = await runCargo({
    cwd: join(
      root,
      crateDir,
    ),
    args: [
      'update',
      ...specs,
    ],
  },);
  if (!outcome.ok)
    throw new CargoResolveError({
      crateDir,
      command: `cargo update ${specs.join(' ',)}`,
      output: outcome.output,
    },);
}

//endregion Resolve
