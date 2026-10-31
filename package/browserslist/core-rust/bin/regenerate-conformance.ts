#!/usr/bin/env node
/**
 Regenerates the conformance fixtures from the data the lock pins.

 Steps: export the verified tarballs with the Rust binary, copy the four data files into `fixtures/data`,
 run npm browserslist 4.29.5 in a bounded container over `conformance/cases.json`, and write the answers to
 `conformance/expected.json`.
 Run it from the package directory after every lock change: `mise run //package/browserslist/core-rust:conformance:regenerate`.
 */
import {
  copyFile,
  mkdir,
  mkdtempDisposable,
  readdir,
  readFile,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import { runCommand, } from './run-command.ts';

/**
 Container image that runs the oracle; the same Node.js line the repository uses.
 */
const ORACLE_IMAGE = 'docker.io/library/node:26.12.0-trixie-slim';

/**
 Repository root inside the container; the mounted `bin` directory and workspace package sit under it at the
 paths they have in the repository.
 */
const CONTAINER_REPOSITORY = '/repo';

/**
 Repository-relative directory of this package.
 */
const PACKAGE_PATH = 'package/browserslist/core-rust';

/**
 Repository-relative directory of the caught-value package the oracle imports.
 */
const CAUGHT_VALUE_PATH = 'package/caught-value/core';

/**
 Memory cap of the oracle container.
 */
const ORACLE_MEMORY = '2g';

/**
 CPU cap of the oracle container.
 */
const ORACLE_CPUS = '2';

/**
 Mode of the scratch root: private to the owner, as trust checks on the scratch directory require.
 */
const SCRATCH_MODE = 0o700;

/**
 Where each fixture data file comes from inside the packages.
 */
const DATA_FILES = [
  {
    tarballPrefix: 'caniuse-lite',
    member: 'package/data/agents.js',
    fixture: 'agents.js.txt',
  },
  {
    tarballPrefix: 'caniuse-lite',
    member: 'package/data/browsers.js',
    fixture: 'browsers.js.txt',
  },
  {
    tarballPrefix: 'caniuse-lite',
    member: 'package/data/browserVersions.js',
    fixture: 'browserVersions.js.txt',
  },
  {
    tarballPrefix: 'node-releases',
    member: 'package/data/processed/envs.json',
    fixture: 'envs.json.txt',
  },
] as const;

/**
 The directory of exported tarballs and the names in it.
 */
type ExportedTarballs = {
  readonly tarballDirectory: string;
  readonly tarballs: readonly string[];
};

/**
 Export the verified tarballs the lock names into a directory.

 @param packageDirectory - this package's directory, where the Rust binary runs

 @param work - scratch directory that receives the cargo cache and the `tgz` directory

 @returns the `tgz` directory and the tarball names in it

 @example
 ```ts
 const { tarballDirectory, tarballs } = await exportTarballs({ packageDirectory, work });
 ```
 */
async function exportTarballs({
  packageDirectory,
  work,
}: {
  readonly packageDirectory: string;
  readonly work: string;
},): Promise<ExportedTarballs> {
  /**
   The lock lives at the repository root; this script runs from the package directory.
   */
  const lockPath = join(
    packageDirectory,
    '..',
    '..',
    '..',
    'browserslist-data.lock.json',
  );
  /**
   Directory that receives the exported tarballs.
   */
  const tarballDirectory = join(
    work,
    'tgz',
  );
  await mkdir(tarballDirectory,);
  await runCommand({
    command: 'cargo',
    args: [
      'run',
      '--quiet',
      '--',
      'export',
      '--lock',
      lockPath,
      '--cache-dir',
      join(
        work,
        'cache',
      ),
      '--out',
      tarballDirectory,
    ],
    cwd: packageDirectory,
  },);
  return {
    tarballDirectory,
    tarballs: await readdir(tarballDirectory,),
  };
}

/**
 Copy the four data files out of the exported tarballs into `fixtures/data`.

 @param packageDirectory - this package's directory

 @param tarballDirectory - directory holding the exported tarballs

 @param tarballs - names of the exported tarballs

 @throws Error when the export wrote no tarball for a data file

 @example
 ```ts
 await copyFixtures({ packageDirectory, tarballDirectory, tarballs });
 ```
 */
async function copyFixtures({
  packageDirectory,
  tarballDirectory,
  tarballs,
}: {
  readonly packageDirectory: string;
  readonly tarballDirectory: string;
  readonly tarballs: readonly string[];
},): Promise<void> {
  await Promise.all(DATA_FILES.map(async function copyFixture({
    tarballPrefix,
    member,
    fixture,
  },) {
    /**
     Exported tarball that holds this data file.
     */
    const tarball = tarballs.find(function hasPrefix(name,) {
      return name.startsWith(`${tarballPrefix}-`,);
    },);
    if (tarball === undefined)
      throw new Error(`export wrote no ${tarballPrefix} tarball`,);
    await writeFile(
      join(
        packageDirectory,
        'fixtures',
        'data',
        fixture,
      ),
      await runCommand({
        command: 'tar',
        args: [
          '--extract',
          '--gzip',
          '--file',
          join(
            tarballDirectory,
            tarball,
          ),
          '--to-stdout',
          member,
        ],
        captureOutput: true,
      },),
    );
  },),);
}

/**
 Run the oracle container over the cases and return the answers it wrote.

 @param packageDirectory - this package's directory, whose `bin` the container mounts

 @param work - scratch directory mounted as the container's `/work`, holding `cases.json` and `tgz`

 @returns the text of `out.json`

 @example
 ```ts
 const answers = await runOracle({ packageDirectory, work });
 ```
 */
async function runOracle({
  packageDirectory,
  work,
}: {
  readonly packageDirectory: string;
  readonly work: string;
},): Promise<string> {
  /**
   The repository root on the host.
   */
  const repositoryRoot = join(
    packageDirectory,
    '..',
    '..',
    '..',
  );
  /**
   Directory mounted as the container's `/repo/node_modules`; it holds one dangling-on-the-host symlink that points
   at the mounted caught-value package.
   */
  const modules = join(
    work,
    'modules',
  );
  await mkdir(
    join(
      modules,
      '@monochromatic-dev',
    ),
    { recursive: true, },
  );
  await symlink(
    `${CONTAINER_REPOSITORY}/${CAUGHT_VALUE_PATH}`,
    join(
      modules,
      '@monochromatic-dev',
      'caught-value-core',
    ),
  );
  await copyFile(
    join(
      packageDirectory,
      'conformance',
      'cases.json',
    ),
    join(
      work,
      'cases.json',
    ),
  );
  await runCommand({
    command: 'podman',
    args: [
      'run',
      '--rm',
      `--memory=${ORACLE_MEMORY}`,
      `--cpus=${ORACLE_CPUS}`,
      '--security-opt',
      'label=disable',
      '--volume',
      `${work}:/work`,
      '--volume',
      `${modules}:${CONTAINER_REPOSITORY}/node_modules:ro`,
      '--volume',
      `${join(
        packageDirectory,
        'bin',
      )}:${CONTAINER_REPOSITORY}/${PACKAGE_PATH}/bin:ro`,
      '--volume',
      `${join(
        repositoryRoot,
        CAUGHT_VALUE_PATH,
      )}:${CONTAINER_REPOSITORY}/${CAUGHT_VALUE_PATH}:ro`,
      ORACLE_IMAGE,
      'node',
      `${CONTAINER_REPOSITORY}/${PACKAGE_PATH}/bin/oracle-container.ts`,
    ],
  },);
  return await readFile(
    join(
      work,
      'out.json',
    ),
    'utf8',
  );
}

/**
 Regenerate the fixtures and the expected answers.

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   This package's directory, which the script expects as its working directory.
   */
  const packageDirectory = process.cwd();
  /**
   Parent of every run's scratch directory.
   */
  const scratchRoot = join(
    homedir(),
    'temp',
    'agent',
    'browserslist-conformance',
  );
  await mkdir(
    scratchRoot,
    {
      recursive: true,
      mode: SCRATCH_MODE,
    },
  );
  /**
   Scratch directory of this run, removed when the function ends.
   */
  await using work = await mkdtempDisposable(join(
    scratchRoot,
    'run-',
  ),);
  /**
   Where the export wrote the tarballs, and their names.
   */
  const {
    tarballDirectory,
    tarballs,
  } = await exportTarballs({
    packageDirectory,
    work: work.path,
  },);
  await copyFixtures({
    packageDirectory,
    tarballDirectory,
    tarballs,
  },);
  await writeFile(
    join(
      packageDirectory,
      'conformance',
      'expected.json',
    ),
    await runOracle({
      packageDirectory,
      work: work.path,
    },),
  );
  console.log(`wrote conformance/expected.json and fixtures/data/* from ${tarballs.join(', ',)}`,);
}

await main();
