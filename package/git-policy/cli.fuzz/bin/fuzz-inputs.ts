/**
 The package inputs a fuzz build context holds, and the PATH-selected tools it copies in.

 Only package inputs are copied, never caches, dependencies, TypeScript sources or machine-local files.
 */
// The bin scripts sit outside the package tsconfig's include list, so type-aware lint checks them in an inferred
// program that loads no ambient Node types; this reference loads them, so `process` and `node:` imports stay typed.
/// <reference types="node" />
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

/** Error codes that mean a PATH entry does not hold a usable candidate. */
const unusableCandidateCodes = new Set([
  'ENOENT',
  'ENOTDIR',
  'EACCES',
]);

/**
 Whether a failure only says that a candidate path is not an executable file.

 @param {unknown} error - the value an access check threw
 @returns {boolean} true for a missing, non-directory or unreadable path
 */
function isUnusableCandidate(error) {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error)
    && unusableCandidateCodes.has(String(error.code));
}

/**
 Whether a path is an executable regular file.

 @param {string} candidate - absolute path to test
 @returns {Promise<boolean>} false when it is missing, unreadable or not a file
 */
async function isExecutableFile(candidate) {
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

 @param {string} name - executable name
 @returns {Promise<string>} absolute path of the first PATH entry holding it
 @throws {NativeVerificationError} when no PATH entry holds it
 */
export async function executable(name) {
  const candidates = (process.env
    .PATH
    ?? '').split(delimiter)
    .map(function candidateIn(directory) {
      return resolve(
        directory,
        name,
      );
    });
  const usable = await Promise.all(candidates.map(isExecutableFile));
  const found = candidates[usable.indexOf(true)];
  if (found === undefined)
    throw new NativeVerificationError(`Executable ${name} was not found on PATH.`);
  return found;
}

/**
 Inputs of one package to copy into the build context.

 @typedef {{ from: string, to: string, names: readonly string[] }} InputSet
 */

/**
 Copy the named entries of each input set.

 @param {{ source: string, context: string }} request - this package's directory and the build context
 */
export async function copyInputs({
  source,
  context,
}) {
  /** @type {readonly InputSet[]} */
  const sets = [
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
