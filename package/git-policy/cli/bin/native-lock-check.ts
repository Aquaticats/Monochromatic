#!/usr/bin/env node
/**
 Ask cargo whether every manifest the wrapper's gate builds still resolves against its own lockfile, in one run.

 This is what `native:lock:check` runs. It promotes the coordinator's two scratch scripts: one reported which of the
 manifests fail `cargo metadata --locked --offline` in a checkout, and the other first re-resolved only the entries a raised
 floor made unsatisfiable and then re-checked them, which is what a version bump of a path crate or of a pinned crate costs
 before the gate can build. Printing one line per manifest is the point: a failing lock names its manifest and cargo's own
 first error, and nothing here writes a lockfile unless `--refresh` is asked for.
 */
import { resolve } from 'node:path';

import {
  gateOptions,
  textOption,
} from './native-gate-arguments.ts';
import { NativeGateError } from './native-gate-error.ts';
import {
  gateRefusalExitCode,
  printLine,
  runGateMain,
} from './native-gate-main.ts';
import { runGateCommand } from './native-gate-run.ts';

/**
 Milliseconds one `cargo metadata` run is given.

 A locked resolution of this package took 2.4 seconds on 2026-11-10 (`cargo metadata --locked --offline`), and an unlocked
 run that has to resolve a raised floor again takes longer; ten minutes covers either on a host under load and still ends a
 run that will not answer.
 */
const metadataDeadlineMilliseconds = 600_000;

/**
 The program every check runs, which resolves a workspace from its manifests and lockfiles.
 */
const cargoProgram = 'cargo';

/**
 The manifests the gate and its sidecars build, relative to a checkout, in the order the two scratch scripts listed them.

 A new path crate the gate compiles belongs here: a lock that fell behind a floor fails the gate's build long after the
 manifest changed, and this is the run that names it in seconds.
 */
export const recordedManifestDirectories: readonly string[] = [
  'package/git-policy/cli',
  'package/git-policy/cli.fuzz',
  'package/linter/monochromatic-lint',
  'package/linter/monochromatic-lint.fuzz',
  'package/forbidden-strings/cli',
  'package/forbidden-strings/core',
  'package/jsonc-edit/core-rust',
];

/**
 Characters of cargo's first error line a report keeps, which is enough for the crate and the requirement it could not meet.
 */
const detailCharacters = 110;

/**
 The text a cargo error line begins with.
 */
const errorPrefix = 'error';

/**
 Option that names the checkout whose manifests are read.
 */
const checkoutOption = '--checkout';

/**
 Option that re-resolves the locked entries before checking them.
 */
const refreshOption = '--refresh';

/**
 How the command is called, printed by every refusal that reads the command line.
 */
const usage = 'usage: native:lock:check [--checkout <path>] [--refresh] [<directory>...]';

/**
 Cargo's first error line of one run, cut to the width a report keeps.

 @param stderr - what the run printed on standard error

 @returns the line, or empty text when the run printed no error line

 @example
 ```ts
 firstErrorLine({ stderr: 'error: failed to select a version\\n' });
 ```
 */
export function firstErrorLine({ stderr }: { readonly stderr: string; }): string {
  return stderr.split('\n')
    .map(function trimmed(line) {
      return line.trim();
    })
    .find(function isError(line) {
      return line.startsWith(errorPrefix);
    })
    ?.slice(
    0,
    detailCharacters
    )
    ?? '';
}

/**
 The argument array of one `cargo metadata` run.

 @param locked - whether the run must answer from the lockfile alone

 @returns the arguments after the program name

 @example
 ```ts
 metadataArguments({ locked: true });
 ```
 */
export function metadataArguments({ locked }: { readonly locked: boolean; }): readonly string[] {
  return [
    'metadata',
    ...(locked ? ['--locked',] : []),
    '--offline',
    '--format-version',
    '1',
  ];
}

/**
 Ask cargo about one manifest directory and print one line.

 @param directory - the directory the run happens in

 @param locked - whether the run must answer from the lockfile alone

 @returns cargo's status and its first error line

 @example
 ```ts
 await cargoMetadata({ directory: '/repo/package/git-policy/cli', locked: true });
 ```
 */
async function cargoMetadata({
  directory,
  locked,
}: {
  readonly directory: string;
  readonly locked: boolean;
}): Promise<{
  readonly status: number;
  readonly detail: string
}> {
  /**
   How the run ended.
   */
  const run = await runGateCommand({
    program: cargoProgram,
    args: metadataArguments({ locked, }),
    deadlineMilliseconds: metadataDeadlineMilliseconds,
    workingDirectory: directory,
  });
  return {
    status: run.timedOut ? (-1) : run.status,
    detail: run.timedOut ? 'the run was still going at its deadline' : firstErrorLine({ stderr: run.stderr, }),
  };
}

/**
 Run the one or two cargo resolutions of one manifest directory and answer the line it prints.

 @param checkout - the checkout the directory belongs to

 @param directory - the directory, relative to the checkout

 @param refresh - whether to re-resolve the locked entries before the locked check

 @returns the line the run prints and whether the locked check failed

 @example
 ```ts
 await checkOne({ checkout: '/repo', directory: 'package/git-policy/cli', refresh: false });
 ```
 */
async function checkOne({
  checkout,
  directory,
  refresh,
}: {
  readonly checkout: string;
  readonly directory: string;
  readonly refresh: boolean;
}): Promise<{
  readonly line: string;
  readonly failed: boolean
}> {
  /**
   What the unlocked resolution answered, when the caller asked for one.
   */
  const refreshRun = refresh ? await cargoMetadata({
    directory: resolve(
      checkout,
      directory
    ),
    locked: false,
  }) : undefined;
  /**
   What the locked resolution answered, which is the answer this command reports.
   */
  const lockedRun = await cargoMetadata({
    directory: resolve(
      checkout,
      directory
    ),
    locked: true,
  });
  return {
    line: refreshRun === undefined
      ? `${String(lockedRun.status)} ${directory} ${lockedRun.detail}`
      : `refresh=${String(refreshRun.status)} locked=${String(lockedRun.status)} ${directory} ${lockedRun.detail}`,
    failed: lockedRun.status !== 0,
  };
}

/**
 Check the manifest directories of one checkout and answer whether all of them resolve from their lockfiles.

 @returns zero when every locked check passed, {@link gateRefusalExitCode} otherwise

 @throws NativeGateError when the command line is one the command cannot read

 @example
 ```ts
 process.exitCode = await runGateMain({ main });
 ```
 */
async function main(): Promise<number> {
  /**
   The command line, whose positionals narrow the run to the directories a person names.
   */
  const options = gateOptions({
    argv: process.argv
      .slice(2),
    knownOptions: [checkoutOption,],
    flagOptions: [refreshOption,],
  });
  /**
   The checkout whose manifests are read.
   */
  const checkout = resolve(textOption({
    options,
    name: checkoutOption,
    fallback: process.cwd(),
  }));
  if ((options.positionals
    .length
    === 0) && (recordedManifestDirectories.length === 0))
    throw new NativeGateError(`native:lock:check knows no manifest directory to check. ${usage}`);
  /**
   Whether the locked entries are re-resolved first, which writes a lockfile.
   */
  const refresh = options.values
    .has(refreshOption);
  /**
   The directories to check, which are the caller's own or the ones the gate builds.
   */
  const directories = options.positionals
    .length
    === 0 ? recordedManifestDirectories : options.positionals;
  /**
   Every directory's answer, in the order the directories were listed.
   */
  const results: {
    readonly line: string;
    readonly failed: boolean
  }[] = [];
  for (const directory of directories) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- the two cargo runs of one directory must finish before the next directory starts, because a refresh writes a lockfile
    results.push(await checkOne({
      checkout,
      directory,
      refresh,
    }));
  }
  for (const result of results)
    printLine(result.line);
  /**
   How many locked checks failed.
   */
  const failures = results.filter(function failed(result) {
    return result.failed;
  });
  return failures.length === 0 ? 0 : gateRefusalExitCode;
}

if (import.meta.main)
  process.exitCode = await runGateMain({ main, });
