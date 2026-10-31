/**
 The source files a snapshot copies into its build context.
 */
import { cp } from 'node:fs/promises';
import { join } from 'node:path';

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  repository,
  scanner,
  scannerDirectory,
} from './snapshot-constants.ts';

/**
 A directory copied into the build context.
 */
type SourcePackage = {
  readonly source: string;
  readonly destination: string;
  readonly files: readonly string[];
};

/**
 One copy: where a file comes from and where the context receives it.
 */
type Copy = {
  readonly from: string;
  readonly to: string;
};

/**
 The scanner crate with the files the tests and the build read.
 */
const scannerPackage: SourcePackage = {
  source: scannerDirectory,
  destination: scanner,
  files: [
    'Cargo.toml',
    'Cargo.lock',
    'build.rs',
    'src',
    'tests',
    'data',
    'bin',
    'mise.toml',
  ],
};

/**
 The engine crate the scanner embeds.
 */
const enginePackage: SourcePackage = {
  source: join(
    scannerDirectory,
    '../../forbidden-strings/core',
  ),
  destination: 'package/forbidden-strings/core',
  files: [
    'Cargo.toml',
    'Cargo.lock',
    'src',
  ],
};

/**
 The fuzz sidecar, copied only for fuzzing snapshots.
 */
const fuzzPackage: SourcePackage = {
  source: join(
    scannerDirectory,
    '../cli.fuzz',
  ),
  destination: `${scanner}.fuzz`,
  files: [
    'Cargo.toml',
    'Cargo.lock',
    'src',
    'fuzz_targets',
    'seed',
    'dictionary',
    'bin',
    'mise.toml',
  ],
};

/**
 Repository files the CI scan-scope tests read from the repository root, so the snapshot carries them.
 */
const repositoryFiles: readonly string[] = [
  'forbidden-strings.append.txt',
  'clippy.toml',
  '.github/workflows/forbidden-strings.yml',
];

/**
 List the copies one source package needs.

 @param item - the package whose files are copied

 @param context - the build context

 @returns one copy per listed file

 @example
 ```ts
 copiesOf({ item: enginePackage, context });
 ```
 */
function copiesOf({
  item,
  context,
}: {
  readonly item: SourcePackage;
  readonly context: string;
}): readonly Copy[] {
  return item.files
    .map(function copyOf(file) {
      return {
        from: join(
          item.source,
          file,
        ),
        to: join(
          context,
          item.destination,
          file,
        ),
      };
    });
}

/**
 Copy one file or directory into the context.

 @param from - the source path

 @param to - the destination path

 @example
 ```ts
 await copyInto({ from: 'a', to: 'b' });
 ```
 */
async function copyInto({
  from,
  to,
}: Copy): Promise<void> {
  await cp(
    from,
    to,
    { recursive: true },
  );
}

/**
 Copy the scanner, the engine, the repository files the tests read and, for fuzzing, the fuzz sidecar into the context.

 @param context - the build context

 @param fuzzing - true to copy the fuzz sidecar too

 @example
 ```ts
 await copySources({ context, fuzzing: false });
 ```
 */
export async function copySources({
  context,
  fuzzing,
}: {
  readonly context: string;
  readonly fuzzing: boolean;
}): Promise<void> {
  await mapInSequence({
    items: [
      scannerPackage,
      enginePackage,
      ...(fuzzing ? [fuzzPackage] : []),
    ].flatMap(function copiesFor(item) {
      return copiesOf({
        item,
        context,
      });
    }),
    run: copyInto,
  });
  await mapInSequence({
    items: repositoryFiles.map(function repositoryCopy(file) {
      return {
        from: join(
          repository,
          file,
        ),
        to: join(
          context,
          file,
        ),
      };
    }),
    run: copyInto,
  });
}
