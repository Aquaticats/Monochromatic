import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  fakeCargoMutantsBytes,
  fakeCargoSource,
  fakePodmanSource,
} from './fake-programs.ts';

/**
 Permission bits that give the owner alone access.
 */
const ownerOnlyMode = 0o700;

/**
 Scratch parent of every fixture directory; never the system temporary directory, whose backing device is unknowable.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Fixture holding the stand-in programs and the working directory a script under test runs in.
 */
export type FakePodman = AsyncDisposable & {
  readonly directory: string;
  readonly workingDirectory: string;
  readonly environment: Readonly<Record<string, string>>;
};

/**
 Create the stand-in programs and a clean working directory.

 @returns the fixture; declare it with `await using`
 
 @example
 ```ts
 await using fake = await createFakePodman();
 ```
 */
export async function createFakePodman(): Promise<FakePodman> {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: ownerOnlyMode,
    },
  );
  /**
   Fresh directory for this fixture.
   */
  const directory = await mkdtemp(join(
    scratchParent,
    'cargo-mutants-podman-',
  ));
  /**
   Directory the script runs in, so evidence lands here and not in a package.
   */
  const workingDirectory = join(
    directory,
    'work',
  );
  /**
   Directory standing in for the cargo home.
   */
  const cargoHome = join(
    directory,
    'cargo',
  );
  /**
   Directory that goes first on `PATH` and holds the stand-in programs.
   */
  const programs = join(
    directory,
    'path',
  );
  await Promise.all([
    mkdir(
      join(
        cargoHome,
        'bin',
      ),
      { recursive: true },
    ),
    mkdir(
      workingDirectory,
      { recursive: true },
    ),
    mkdir(
      programs,
      { recursive: true },
    ),
  ]);
  await Promise.all([
    writeFile(
      join(
        programs,
        'podman',
      ),
      fakePodmanSource,
      { mode: ownerOnlyMode },
    ),
    writeFile(
      join(
        programs,
        'cargo',
      ),
      fakeCargoSource,
      { mode: ownerOnlyMode },
    ),
    writeFile(
      join(
        cargoHome,
        'bin',
        'cargo-mutants',
      ),
      fakeCargoMutantsBytes,
    ),
  ]);
  return {
    directory,
    workingDirectory,
    environment: {
      PATH: `${programs}:${process.env
        .PATH
        ?? ''}`,
      FAKE_DIR: directory,
      CARGO_HOME: cargoHome,
      HOME: directory,
    },
    async [Symbol.asyncDispose]() {
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}
