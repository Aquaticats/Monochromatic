/**
 The package inputs a fuzz build context holds, and the PATH-selected tools it copies in.

 Only package inputs are copied, never caches, dependencies, TypeScript sources or machine-local files.
 */
import { constants } from 'node:fs';
import {
  access,
  cp,
  stat,
} from 'node:fs/promises';
import {
  delimiter,
  join,
  resolve,
} from 'node:path';

import { NativeVerificationError } from '../../cli/bin/native-verification-process.ts';

/**
 Error codes that mean a PATH entry does not hold a usable candidate.
 */
const unusableCandidateCodes = new Set([
  'ENOENT',
  'ENOTDIR',
  'EACCES',
]);

/**
 Whether a failure only says that a candidate path is not an executable file.

 @param error - value an access check threw
 
 @returns true for a missing, non-directory or unreadable path
 */
function isUnusableCandidate(error: unknown): boolean {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error)
    && unusableCandidateCodes.has(String(error.code));
}

/**
 Whether a path is an executable regular file.

 @param candidate - absolute path to test
 
 @returns false when it is missing, unreadable or not a file
 */
async function isExecutableFile(candidate: string): Promise<boolean> {
  try {
    await access(
      candidate,
      constants.X_OK,
    );
    return (await stat(candidate)).isFile();
  }
  catch (error) {
    if (isUnusableCandidate(error))
      return false;
    throw error;
  }
}

/**
 Resolve the PATH-selected tool rather than an older unrelated Cargo-home installation.

 @param name - executable name
 
 @returns absolute path of the first PATH entry holding it

 @throws NativeVerificationError when no PATH entry holds it

 @example
 ```ts
 const cargoFuzz = await executable('cargo-fuzz');
 ```
 */
export async function executable(name: string): Promise<string> {
  /**
   The path the executable would have in each PATH entry, in PATH order.
   */
  const candidates = (process.env
    .PATH
    ?? '').split(delimiter)
    .map(function candidateIn(directory) {
      return resolve(
        directory,
        name,
      );
    });
  /**
   Whether each candidate is an executable file.
   */
  const usable = await Promise.all(candidates.map(isExecutableFile));
  /**
   The first usable candidate, absent when no PATH entry holds the tool.
   */
  const found = candidates[usable.indexOf(true)];
  if (found === undefined)
    throw new NativeVerificationError(`Executable ${name} was not found on PATH.`);
  return found;
}

/**
 Inputs of one package to copy into the build context.
 */
export type InputSet = {
  readonly from: string;
  readonly to: string;
  readonly names: readonly string[];
};

/**
 Copy the named entries of each input set.

 @param source - this package's directory

 @param context - build context

 @example
 ```ts
 await copyInputs({ source: process.cwd(), context });
 ```
 */
export async function copyInputs({
  source,
  context,
}: {
  readonly source: string;
  readonly context: string;
}): Promise<void> {
  /**
   Every package whose inputs the build reads, with the entries of each.
   */
  const sets: readonly InputSet[] = [
    {
      from: '.',
      to: 'package/git-policy/cli.fuzz',
      names: [
        'Cargo.toml',
        'Cargo.lock',
        'src',
        'fuzz_targets',
      ],
    },
    {
      from: '../cli',
      to: 'package/git-policy/cli',
      names: [
        'Cargo.toml',
        'Cargo.lock',
        'README.md',
        'src/native',
      ],
    },
    {
      from: '../../jsonc-edit/core-rust',
      to: 'package/jsonc-edit/core-rust',
      names: [
        'Cargo.toml',
        'Cargo.lock',
        'src',
        'fixtures',
      ],
    },
    // The subject links the scanner; `build.rs` and `data` hold the embedded baseline its library compiles in.
    {
      from: '../../forbidden-strings/cli',
      to: 'package/forbidden-strings/cli',
      names: [
        'Cargo.toml',
        'Cargo.lock',
        'build.rs',
        'src',
        'data',
      ],
    },
    {
      from: '../../forbidden-strings/core',
      to: 'package/forbidden-strings/core',
      names: [
        'Cargo.toml',
        'Cargo.lock',
        'src',
      ],
    },
  ];
  await Promise.all(sets.flatMap(function copiesOf({
    from,
    to,
    names,
  }) {
    return names.map(function copyOne(name) {
      return cp(
        resolve(
          source,
          from,
          name,
        ),
        join(
          context,
          to,
          name,
        ),
        { recursive: true },
      );
    });
  }));
}
