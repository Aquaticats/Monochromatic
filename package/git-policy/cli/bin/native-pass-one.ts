/**
 Pass 1 of a two-pass native mutation campaign: the tests that can reach the mutated files.

 The selection comes from the crate's own sources (see `testScope` in the shared cargo-mutants package): the tests of
 the mutated file's module and of every module that imports it, directly or through other modules, plus the test
 targets that refer to one of those modules. The binary suite (`native_binary`) is left out because it is slow and
 exercises everything; pass 2 runs it, on the mutants the library tests missed.
 */
import { resolve } from 'node:path';

import {
  CargoMutantsToolingError,
  scopeFilterset,
  selectTestsArguments,
  testScope,
  type TestScope,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

/**
 Test targets that never take part in pass 1: the binary-level suite, which starts the built executable against real
 Git and is the slowest part of the package's tests.
 */
export const slowTestTargets: readonly string[] = ['native_binary'];

/**
 Characters that make a `--file` scope a glob rather than one file.
 */
const globCharacters = '*?[]{}';

/**
 Pass 1 of a campaign.
 */
export type PassOne = {
  /**
   The campaign command with the test selection added.
   */
  readonly command: readonly string[];
  readonly scope: TestScope;
  /**
   The nextest filterset the command carries.
   */
  readonly filterset: string;
};

/**
 Work out pass 1 for the files a campaign mutates.

 @param crateDirectory - directory of the crate under test, which holds `Cargo.toml`

 @param scopes - the campaign's `--file` scopes, as paths relative to the crate directory

 @param command - the campaign command, which pass 1 extends with its test selection

 @returns the pass-1 command, scope and filterset

 @throws CargoMutantsToolingError when there is no scope, a scope is a glob, or a file is not a module of the library

 @example
 ```ts
 await passOneFor({ crateDirectory, scopes: ['src/native/lock.rs'], command });
 ```
 */
export async function passOneFor({
  crateDirectory,
  scopes,
  command,
}: {
  readonly crateDirectory: string;
  readonly scopes: readonly string[];
  readonly command: readonly string[];
}): Promise<PassOne> {
  if (scopes.length === 0)
    throw new CargoMutantsToolingError('--passes 2 needs at least one --file scope, because the tests that reach a file are found from the file.');
  for (const scope of scopes) {
    if ([...scope].some(function isGlobCharacter(character) {
      return globCharacters.includes(character);
    }))
      throw new CargoMutantsToolingError(`--passes 2 needs literal --file paths, but ${scope} is a glob; name each file, or run one pass.`);
  }
  /**
   The tests that can reach the files.
   */
  const scope = await testScope({
    crateDirectory: resolve(crateDirectory),
    mutatedFiles: scopes,
    excludedTestTargets: slowTestTargets,
  });
  if (scope === undefined)
    throw new CargoMutantsToolingError(`--passes 2 found no library module for ${scopes.join(', ')}; the file is not declared by the library, so run one pass.`);
  /**
   The filterset that selects them.
   */
  const filterset = scopeFilterset(scope);
  if (filterset === undefined)
    throw new CargoMutantsToolingError(`--passes 2 found no tests that reach ${scopes.join(', ')}.`);
  return {
    command: selectTestsArguments({
      command,
      filterset,
    }),
    scope,
    filterset,
  };
}
