import { resolve } from 'node:path';

import { defaultPushAttempts } from './constants.ts';
import {
  type CrateRegistry,
  findCrate,
} from './crate-registry.ts';
import { CrateReleaseError } from './error.ts';
import {
  requiredValue,
  rootOf,
  splitArguments,
  valueOr,
} from './option-parsing.ts';
import {
  parseBranchName,
  parseCrateName,
  parseRepository,
  parseWholeNumber,
} from './validate.ts';

/**
 Everything the push of a release commit needs, validated.
 */
export type PushOptions = {
  /**
   Absolute path of the checkout that holds the release commit, where `git`, `cargo` and `gh` run.
   */
  readonly root: string;
  /**
   `owner/name` of the repository to push to.
   */
  readonly repo: string;
  /**
   Branch to append the commit to.
   */
  readonly branch: string;
  /**
   Absolute directory for the request files, outside the checkout so no file is ever committed.
   */
  readonly scratchDirectory: string;
  /**
   Name of the crate that was released, as in the registry.
   */
  readonly crateName: string;
  /**
   Repository-relative directory of the crate, whose files the release depends on.
   */
  readonly crateDirectory: string;
  /**
   Repository-relative directories whose lockfile the release refreshed, whose files it also depends on.
   */
  readonly lockDirectories: readonly string[];
  /**
   Most times the commit is sent, the first send included.
   */
  readonly maxAttempts: number;
};

/**
 Option definitions of the push command in the `node:util` `parseArgs` format.
 */
const pushOptionDefinitions = {
  'root': { type: 'string' },
  'crate': { type: 'string' },
  'repo': { type: 'string' },
  'branch': { type: 'string' },
  'scratch-dir': { type: 'string' },
  'max-attempts': { type: 'string' },
} as const;

/**
 Read and validate the command line of the push command.

 @param argv - arguments after the script path: options only

 @param root - repository root used when `--root` is absent, or a function that finds it only when needed

 @param registry - the registry of releasable crates

 @returns the validated options

 @throws CrateReleaseError naming the argument and the valid form when anything is missing, malformed or unsafe

 @example
 ```ts
 parsePushArguments({ argv: ['--crate', 'c', '--repo', 'o/r', '--branch', 'main', '--scratch-dir', '/t'], root: '/r', registry });
 ```
 */
export function parsePushArguments({
  argv,
  root,
  registry,
}: {
  readonly argv: readonly string[];
  readonly root: string | (() => string);
  readonly registry: () => CrateRegistry;
}): PushOptions {
  /**
   Options and positionals, split but not yet validated.
   */
  const raw = splitArguments({
    argv,
    options: pushOptionDefinitions
  });
  if (raw.positionals
    .length
    > 0)
    throw new CrateReleaseError(
      `The push command takes options only, got ${JSON.stringify(raw.positionals)}.`,
    );
  /**
   The scratch directory as given.
   */
  const scratchDirectory = requiredValue({
    raw,
    name: 'scratch-dir'
  });
  if (!scratchDirectory.startsWith('/'))
    throw new CrateReleaseError(`--scratch-dir must be an absolute path, got ${JSON.stringify(scratchDirectory)}.`);
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
  /**
   The crate's arguments in the registry.
   */
  const entry = findCrate({
    registry: registry(),
    name: crateName
  });
  return {
    root: resolve(rootOf({
      raw,
      root
    })),
    repo: parseRepository({
      option: '--repo',
      value: requiredValue({
        raw,
        name: 'repo'
      })
    }),
    branch: parseBranchName({
      option: '--branch',
      value: requiredValue({
        raw,
        name: 'branch'
      })
    }),
    scratchDirectory,
    crateName,
    crateDirectory: entry.crateDirectory,
    lockDirectories: entry.lockDirectories,
    maxAttempts: parseWholeNumber({
      option: '--max-attempts',
      value: valueOr({
        raw,
        name: 'max-attempts',
        fallback: String(defaultPushAttempts)
      })
    }),
  };
}
