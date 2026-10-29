#!/usr/bin/env node
/**
 Command line for rust-catalog-update.

 ```sh
 rust-catalog-update            # update every Cargo.lock, keep the result if every check passes
 rust-catalog-update --dry-run  # report what would change, then restore every Cargo.lock
 rust-catalog-update --verify   # check versions no lockfile held at HEAD against the trust policy
 rust-catalog-update --verify --base <rev>  # CI: the same against another revision
 ```

 Run it from the repository root (the `mise` task does).

 @module
 */

import { createSpawnCargo, } from './cargo.ts';
import {
  fetchGunzippedWithUserAgent,
  fetchTextWithUserAgent,
} from './http.ts';
import { createGitBaseLockReader, } from './git-base.ts';
import { createRegistry, } from './registry.ts';
import {
  type UpdateReport,
  updateRustLockfiles,
} from './update.ts';
import { cargoToolchain, } from './toolchain.ts';
import { verifyRustLockfiles, } from './verify.ts';

/**
 Usage text.
 */
const USAGE = 'Usage: rust-catalog-update [--dry-run | --verify [--base <rev>] | --help]';

/**
 Revision `--verify` compares with when `--base` is absent: the last commit, so uncommitted lockfile changes are checked.
 */
const DEFAULT_BASE = 'HEAD';

//region Errors

/**
 Thrown when the update or the verification finds policy problems.

 @example
 ```ts
 throw new RustCatalogPolicyError({ heading: 'Update refused', problems: ['x@1.0.0 ...',], });
 ```
 */
export class RustCatalogPolicyError extends Error {
  /**
   Every finding.
   */
  readonly problems: readonly string[];

  /**
   Builds the error.

   @param heading - what was refused

   @param problems - findings
   */
  constructor({
    heading,
    problems,
  }: {
    readonly heading: string;
    readonly problems: readonly string[]
  },) {
    super([
      `${heading}: ${String(problems.length,)} finding(s).`,
      ...problems.map(function bullet(problem: string,): string {
        return `- ${problem}`;
      },),
      'Settings live in rust-catalog.toml; see doc/decision/rust-dependency-catalog.md.',
    ].join('\n',),);
    this.name = 'RustCatalogPolicyError';
    this.problems = problems;
  }
}

/**
 Thrown for arguments the command does not accept.

 @example
 ```ts
 throw new UsageError('unknown argument --x');
 ```
 */
export class UsageError extends Error {
  /**
   Builds the error.

   @param problem - what was wrong with the arguments
   */
  constructor(problem: string,) {
    super(`${problem}\n${USAGE}`,);
    this.name = 'UsageError';
  }
}

//endregion Errors

//region Arguments

/**
 Mode the arguments select.
 */
type Mode = 'update' | 'dry-run' | 'verify' | 'help';

/**
 Mode for each accepted flag.
 */
const FLAG_MODES: ReadonlyMap<string, Mode> = new Map([
  [
    '--dry-run',
    'dry-run',
  ],
  [
    '--verify',
    'verify',
  ],
  [
    '--help',
    'help',
  ],
],);

/**
 What the arguments ask for.
 */
type Invocation = {
  readonly mode: Mode;

  /**
   Revision `--verify` compares with.
   */
  readonly base: string;
};

/**
 Base revision given after `--verify`.

 @param rest - arguments after `--verify`

 @returns the revision, or the default when none was given

 @throws UsageError for anything but one `--base <rev>` pair

 @example
 ```ts
 parseBase(['--base', 'HEAD~1',]); // 'HEAD~1'
 ```
 */
function parseBase(rest: readonly string[],): string {
  if (rest.length === 0)
    return DEFAULT_BASE;
  /**
   Revision after the flag.
   */
  const [flag, revision = '', ...extra] = rest;
  if ((flag !== '--base') || (revision === '')
    || revision.startsWith('-',)
    || (extra.length > 0))
    throw new UsageError(`--verify accepts only --base <rev>, got ${rest.join(' ',)}`,);
  return revision;
}

/**
 Mode and options selected by the arguments.

 @param args - arguments after the script path

 @returns selected mode and the verification base

 @throws UsageError for unknown or combined flags

 @example
 ```ts
 parseMode(['--verify', '--base', 'abc123',]); // { mode: 'verify', base: 'abc123' }
 ```
 */
export function parseMode(args: readonly string[],): Invocation {
  if (args.length === 0) {
    return {
      mode: 'update',
      base: DEFAULT_BASE,
    };
  }
  /**
   Mode of the first flag.
   */
  const mode = FLAG_MODES.get(args[0] ?? '',);
  if (mode === undefined)
    throw new UsageError(`unknown argument ${JSON.stringify(args[0],)}`,);
  if (mode === 'verify') {
    return {
      mode,
      base: parseBase(args.slice(1,),),
    };
  }
  if (args.length > 1)
    throw new UsageError(`expected at most one flag, got ${args.join(' ',)}`,);
  return {
    mode,
    base: DEFAULT_BASE,
  };
}

//endregion Arguments

//region Output

/**
 Prints an update report.

 @param report - update outcome

 @param dryRun - whether the run was a dry run

 @example
 ```ts
 printReport({ report, dryRun: true, });
 ```
 */
function printReport({
  report,
  dryRun,
}: {
  readonly report: UpdateReport;
  readonly dryRun: boolean
},): void {
  console.info(`${String(report.crateDirs
    .length,)} lockfiles, ${String(report.added
      .length,)} newly locked versions:`,);
  for (const added of report.added)
    console.info(`  ${added.name} ${added.version}`,);
  for (const crateDir of report.fellBack)
    console.info(`Resolved ${crateDir} without the release-age cutoff (minimumReleaseAgeStrict = false).`,);
  for (const warning of report.warnings)
    console.info(`Warning: ${warning}`,);
  for (const selector of report.pruned)
    console.info(`Pruned ${selector} from rust-catalog.toml.`,);
  if (dryRun)
    console.info('Dry run: every Cargo.lock was restored.',);
  else if (report.written)
    console.info('Kept the new lockfiles. Next: `mise run catalog:tighten`, then `mise run sync:files`.',);
}

//endregion Output

//region Main

/**
 Runs the command.

 @param args - arguments after the script path

 @throws UsageError for bad arguments

 @throws RustCatalogPolicyError when the update is refused or verification fails

 @example
 ```ts
 await main(['--verify',]);
 ```
 */
export async function main(args: readonly string[],): Promise<void> {
  /**
   Selected mode and verification base.
   */
  const {
    mode,
    base,
  } = parseMode(args,);
  if (mode === 'help') {
    console.info(USAGE,);
    return;
  }
  /**
   Repository root.
   */
  const root = process.cwd();
  /**
   Registry with the history cache under the root.
   */
  const registry = createRegistry({
    root,
    fetchText: fetchTextWithUserAgent,
    fetchDump: fetchGunzippedWithUserAgent,
  },);
  if (mode === 'verify') {
    /**
     Trust findings.
     */
    const problems = await verifyRustLockfiles({
      root,
      now: Date.now(),
      registry,
      readBaseLock: await createGitBaseLockReader({
        root,
        base,
      },),
    },);
    if (problems.length > 0)
      throw new RustCatalogPolicyError({
        heading: 'Trust verification failed',
        problems,
      },);
    console.info(`Every registry version locked since ${base} passes the trust policy.`,);
    return;
  }
  /**
   Update outcome.
   */
  const report = await updateRustLockfiles({
    root,
    dryRun: mode === 'dry-run',
    now: Date.now(),
    runCargo: createSpawnCargo({
      toolchain: await cargoToolchain({
        root,
        explicit: process.env
          .RUSTUP_TOOLCHAIN
          ?? '',
      },),
    },),
    registry,
  },);
  printReport({
    report,
    dryRun: mode === 'dry-run',
  },);
  if (report.problems
    .length
    > 0)
    throw new RustCatalogPolicyError({
      heading: 'Update refused; every Cargo.lock was restored',
      problems: report.problems,
    },);
}

if (import.meta.main)
  await main(process.argv
    .slice(2,),);

//endregion Main
