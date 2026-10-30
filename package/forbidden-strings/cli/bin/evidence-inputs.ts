/**
 The compiled input set of the scanner, hashed from the working tree.
 */
import { createHash } from 'node:crypto';
import {
  readdir,
  readFile,
  stat,
} from 'node:fs/promises';
import { join } from 'node:path';

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

/**
 Repository-relative directory of the scanner.
 */
const scanner = 'package/forbidden-strings/cli';

/**
 Repository-relative directory of the engine.
 */
const engine = 'package/forbidden-strings/core';

/**
 Repository files the scan-scope tests read.
 */
const repositoryInputs: readonly string[] = [
  'clippy.toml',
  'forbidden-strings.append.txt',
  '.github/workflows/forbidden-strings.yml',
];

/**
 Entries of the scanner that enter Cargo's artifact or its tests.
 */
const scannerInputs: readonly string[] = [
  'Cargo.toml',
  'Cargo.lock',
  'build.rs',
  'src',
  'tests',
  'data',
];

/**
 Entries of the engine that enter Cargo's artifact.
 */
const engineInputs: readonly string[] = [
  'Cargo.toml',
  'Cargo.lock',
  'src',
];

/**
 Entries of the fuzz sidecar that enter its artifact.
 */
const fuzzInputs: readonly string[] = [
  'Cargo.toml',
  'Cargo.lock',
  'src',
  'fuzz_targets',
  'seed',
  'dictionary',
];

/**
 The digest of one current input.
 */
export type InputDigest = {
  readonly path: string;
  readonly sha256: string;
};

/**
 List the paths to hash, relative to the repository.

 @param fuzzing - true to include the fuzz sidecar

 @returns the starting paths; directories are expanded by the walk

 @example
 ```ts
 startingPaths(false);
 ```
 */
function startingPaths(fuzzing: boolean): readonly string[] {
  return [
    ...repositoryInputs,
    ...scannerInputs.map(function inScanner(file) {
      return `${scanner}/${file}`;
    }),
    ...engineInputs.map(function inEngine(file) {
      return `${engine}/${file}`;
    }),
    ...(fuzzing ? fuzzInputs.map(function inFuzz(file) {
      return `${scanner}.fuzz/${file}`;
    }) : []),
  ];
}

/**
 Digest of every file below one path, directories expanded.

 @param repository - root of the repository

 @param path - repository-relative path of a file or a directory

 @returns the digest of each file below the path, last entry of a directory first, which is the order a work stack
 visits them in

 @example
 ```ts
 const digests = await digestsBelow({ repository, path: 'clippy.toml' });
 ```
 */
async function digestsBelow({
  repository,
  path,
}: {
  readonly repository: string;
  readonly path: string;
}): Promise<readonly InputDigest[]> {
  /**
   Absolute location of the path.
   */
  const absolute = join(
    repository,
    path,
  );
  if (!(await stat(absolute)).isDirectory()) return [
    {
      path,
      sha256: createHash('sha256')
        .update(await readFile(absolute))
        .digest('hex'),
    },
  ];
  /**
   Names inside the directory, last first.
   */
  const names = (await readdir(absolute)).toReversed();
  /**
   Digests below each name.
   */
  const below = await mapInSequence({
    items: names,
    run: function digestsOf(name) {
      return digestsBelow({
        repository,
        path: `${path}/${name}`,
      });
    },
  });
  return below.flat();
}

/**
 Enumerate both sides of the copied source/build/test input set, including newly added current files.

 @param repository - root of the repository

 @param fuzzing - true to include the fuzz sidecar's inputs

 @returns the digest of every current input, in the order a work stack visits them

 @example
 ```ts
 const inputs = await currentInputs({ repository, fuzzing: false });
 ```
 */
export async function currentInputs({
  repository,
  fuzzing,
}: {
  readonly repository: string;
  readonly fuzzing: boolean;
}): Promise<readonly InputDigest[]> {
  /**
   Digests below every starting path, the last starting path first.
   */
  const digests = await mapInSequence({
    items: startingPaths(fuzzing)
      .toReversed(),
    run: function digestsOf(path) {
      return digestsBelow({
        repository,
        path,
      });
    },
  });
  return digests.flat();
}
