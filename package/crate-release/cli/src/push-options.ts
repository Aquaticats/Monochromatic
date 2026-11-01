import { resolve } from 'node:path';

import { CrateReleaseError } from './error.ts';
import {
  requiredValue,
  rootOf,
  splitArguments,
} from './option-parsing.ts';
import {
  parseBranchName,
  parseRepository,
} from './validate.ts';

/**
 Everything the push of a release commit needs, validated.
 */
export type PushOptions = {
  /**
   Absolute path of the checkout that holds the release commit, where `git` and `gh` run.
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
   Absolute directory for the request file, outside the checkout so the file is never committed.
   */
  readonly scratchDirectory: string;
};

/**
 Option definitions of the push command in the `node:util` `parseArgs` format.
 */
const pushOptionDefinitions = {
  'root': { type: 'string' },
  'repo': { type: 'string' },
  'branch': { type: 'string' },
  'scratch-dir': { type: 'string' },
} as const;

/**
 Read and validate the command line of the push command.

 @param argv - arguments after the script path: options only

 @param root - repository root used when `--root` is absent, or a function that finds it only when needed

 @returns the validated options

 @throws CrateReleaseError naming the argument and the valid form when anything is missing, malformed or unsafe

 @example
 ```ts
 parsePushArguments({ argv: ['--repo', 'o/r', '--branch', 'main', '--scratch-dir', '/tmp'], root: '/r' });
 ```
 */
export function parsePushArguments({
  argv,
  root,
}: {
  readonly argv: readonly string[];
  readonly root: string | (() => string);
}): PushOptions {
  /**
   Options and positionals, split but not yet validated.
   */
  const raw = splitArguments({
    argv,
    options: pushOptionDefinitions
  });
  if (raw.positionals.length > 0)
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
  };
}
