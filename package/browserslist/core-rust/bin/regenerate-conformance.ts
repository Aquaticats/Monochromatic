#!/usr/bin/env node
/**
 Regenerates the conformance fixtures from the data the lock pins.

 Steps: export the verified tarballs with the Rust binary, copy the four data files into `fixtures/data`,
 run npm browserslist 4.29.5 in a bounded container over `conformance/cases.json`, and write the answers to
 `conformance/expected.json`.
 Run it from the package directory after every lock change: `mise run //package/browserslist/core-rust:conformance:regenerate`.
 */
import { execFileSync, } from 'node:child_process';
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

/**
 Container image that runs the oracle; the same Node.js line the repository uses.
 */
const ORACLE_IMAGE = 'docker.io/library/node:26.12.0-trixie-slim';

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

const packageDirectory = process.cwd();
/**
 The lock lives at the repository root; this script runs from the package directory.
 */
const LOCK_PATH = join(
  packageDirectory,
  '..',
  '..',
  '..',
  'browserslist-data.lock.json',
);
const scratchRoot = join(
  homedir(),
  'temp',
  'agent',
  'browserslist-conformance',
);
mkdirSync(
  scratchRoot,
  {
    recursive: true,
    mode: 0o700,
  },
);
const work = mkdtempSync(join(
  scratchRoot,
  'run-',
),);

try {
  const tarballDirectory = join(
    work,
    'tgz',
  );
  mkdirSync(tarballDirectory,);
  execFileSync(
    'cargo',
    [
      'run',
      '--quiet',
      '--',
      'export',
      '--lock',
      LOCK_PATH,
      '--cache-dir',
      join(
        work,
        'cache',
      ),
      '--out',
      tarballDirectory,
    ],
    {
      cwd: packageDirectory,
      stdio: 'inherit',
    },
  );
  const tarballs = readdirSync(tarballDirectory,);
  for (const {
    tarballPrefix,
    member,
    fixture,
  } of DATA_FILES) {
    const tarball = tarballs.find((name,) => name.startsWith(`${tarballPrefix}-`,));
    if (tarball === undefined) {
      throw new Error(`export wrote no ${tarballPrefix} tarball`,);
    }
    const contents = execFileSync(
      'tar',
      [
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
      {
        maxBuffer: 64 * 1_024
          * 1_024,
      },
    );
    writeFileSync(
      join(
        packageDirectory,
        'fixtures',
        'data',
        fixture,
      ),
      contents,
    );
  }
  copyFileSync(
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
  execFileSync(
    'podman',
    [
      'run',
      '--rm',
      '--memory=2g',
      '--cpus=2',
      '--security-opt',
      'label=disable',
      '--volume',
      `${work}:/work`,
      '--volume',
      `${join(
      packageDirectory,
      'bin',
      'oracle-container.ts',
    )}:/oracle.ts:ro`,
      ORACLE_IMAGE,
      'node',
      '/oracle.ts',
    ],
    { stdio: 'inherit', },
  );
  const answers = readFileSync(
    join(
      work,
      'out.json',
    ),
    'utf8',
  );
  writeFileSync(
    join(
      packageDirectory,
      'conformance',
      'expected.json',
    ),
    answers,
  );
  console.log(`wrote conformance/expected.json and fixtures/data/* from ${tarballs.join(', ',)}`,);
} finally {
  rmSync(
    work,
    {
      recursive: true,
      force: true,
    },
  );
}
