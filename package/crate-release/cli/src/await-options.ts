import { resolve } from 'node:path';

import {
  type CrateRegistry,
  findCrate,
} from './crate-registry.ts';
import { CrateReleaseError } from './error.ts';
import {
  requiredValue,
  rootOf,
  splitArguments,
} from './option-parsing.ts';
import { parseRunReference } from './run-results.ts';
import { parseCrateName } from './validate.ts';
import { assertReleaseVersion } from './version.ts';
import {
  parsePublishCheckSettings,
  type PublishCheckSettings,
  publishCheckOptionDefinitions,
} from './wait-options.ts';

/**
 Everything the release await needs, validated.
 */
export type AwaitOptions = PublishCheckSettings & {
  /**
   Absolute path of the repository root, where `gh` runs so it finds the repository.
   */
  readonly root: string;
  /**
   Name of the crate, as on crates.io.
   */
  readonly crateName: string;
  /**
   Numeric identifier of the release workflow run to wait for.
   */
  readonly releaseRunId: string;
  /**
   Version expected on crates.io and in the release.
   */
  readonly version: string;
};

/**
 Option definitions of the await command in the `node:util` `parseArgs` format.
 */
const awaitOptionDefinitions = {
  'root': { type: 'string' },
  'crate': { type: 'string' },
  ...publishCheckOptionDefinitions,
} as const;

/**
 Read and validate the command line of the await command.

 @param argv - arguments after the script path: options, then `<run> <version>`

 @param root - repository root used when `--root` is absent, or a function that finds it only when needed

 @param registry - the registry of releasable crates

 @returns the validated options

 @throws CrateReleaseError naming the argument and the valid form when anything is missing, malformed or unsafe

 @example
 ```ts
 parseAwaitArguments({ argv: ['--crate', 'c', '123', '1.0.0'], root: '/r', registry });
 ```
 */
export function parseAwaitArguments({
  argv,
  root,
  registry,
}: {
  readonly argv: readonly string[];
  readonly root: string | (() => string);
  readonly registry: () => CrateRegistry;
}): AwaitOptions {
  /**
   Options and positionals, split but not yet validated.
   */
  const raw = splitArguments({
    argv,
    options: awaitOptionDefinitions
  });
  /**
   The two positional arguments and anything beyond them.
   */
  const [reference, version, ...extra] = raw.positionals;
  if ((reference === undefined) || (version === undefined)
    || (extra.length > 0))
    throw new CrateReleaseError(
      `Expected exactly two arguments <run> <version> after the options, got ${String(raw.positionals
        .length)}.`,
    );
  /**
   The release run named by a run address or a bare identifier.
   */
  const releaseRun = parseRunReference(reference);
  if (!releaseRun.found)
    throw new CrateReleaseError(`<run>: ${releaseRun.reason}`);
  /**
   The crate, checked as a name and found in the registry.
   */
  const crateName = parseCrateName({
    option: '--crate',
    value: requiredValue({
      raw,
      name: 'crate'
    })
  });
  // The lookup refuses a crate that no release could have started for, instead of waiting on a version that never comes.
  findCrate({
    registry: registry(),
    name: crateName
  });
  return {
    root: resolve(rootOf({
      raw,
      root
    })),
    crateName,
    releaseRunId: releaseRun.value,
    version: assertReleaseVersion({
      option: '<version>',
      value: version
    }),
    ...parsePublishCheckSettings(raw),
  };
}
