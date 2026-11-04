/**
 Guard: the executables the crate declares are exactly the ones the release job builds and packages, and the ones the
 release checks and the toolchain installs.

 cargo-binstall requires every `[[bin]]` of a crate to be in the archive, so a second `[[bin]]` that the release job does
 not package fails every install that does not compile: the `catalog-specifiers` workflow was red from 2026-11-01 until the
 commit service stopped being a second executable (`doc/planning/cli-git-resident-service.md`, "One executable").
 */
import { readFileSync, } from 'node:fs';
import { resolve, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Repository root, five levels above this script's directory.
 */
const repositoryRoot = resolve(
  import.meta.dirname,
  '../../../..',
);

/**
 The crate's name, as the release job, the release list and the toolchain lock name it.
 */
const crateName = 'monochromatic-git-policy-cli';

/**
 Marker line that starts the job which builds and packages the crate's archive, and the job that follows it.
 */
const buildJobStart = '  gpc-build-binary:';
const buildJobEnd = '  gpc-create-release:';

/**
 Names of the `[[bin]]` tables of a manifest, read line by line: a table header starts a table, and the first `name = "x"`
 line of a `[[bin]]` table names the executable.

 @param manifest - the text of a `Cargo.toml`

 @returns the executable names in manifest order
 */
function manifestBins(manifest: string): readonly string[] {
  const names: string[] = [];
  let inBin = false;
  for (const line of manifest.split('\n')) {
    const trimmed = line.trim();
    if (trimmed.startsWith('[')) {
      inBin = trimmed === '[[bin]]';
      continue;
    }
    if (inBin && trimmed.startsWith('name = "')) {
      names.push(trimmed.slice('name = "'.length, trimmed.lastIndexOf('"')));
      inBin = false;
    }
  }
  return names;
}

/**
 The lines of the release workflow's binary job for this crate.

 @param workflow - the text of `cargo-publish.yml`

 @returns the job's lines

 @throws Error when the job is not found, so a renamed job cannot make the guard pass by reading nothing
 */
function binaryJobLines(workflow: string): readonly string[] {
  const start = workflow.indexOf(buildJobStart);
  const end = workflow.indexOf(
    buildJobEnd,
    start,
  );
  if ((start === (-1)) || (end === (-1)))
    throw new Error(`cargo-publish.yml has no ${buildJobStart.trim()} job followed by ${buildJobEnd.trim()}`);
  return workflow.slice(
    start,
    end,
  ).split('\n');
}

/**
 The executables the binary job passes to `cargo build --bin`, and the one it copies into the archive (`bin='name'`).

 @param lines - the binary job's lines

 @returns what is built and what is packaged
 */
function builtAndPackaged(lines: readonly string[]): { readonly built: readonly string[]; readonly packaged: readonly string[]; } {
  const built: string[] = [];
  const packaged: string[] = [];
  for (const line of lines) {
    const words = line.trim().split(' ').filter((word) => word !== '');
    for (const [index, word,] of words.entries()) {
      const next = words[index + 1];
      if ((word === '--bin') && (next !== undefined))
        built.push(next);
    }
    const trimmed = line.trim();
    if (trimmed.startsWith('bin=\'') && trimmed.endsWith('\''))
      packaged.push(trimmed.slice('bin=\''.length, -1));
  }
  return {
    built,
    packaged,
  };
}

/**
 Every place that lists the crate's executables, and what each lists.

 @param sources - the texts to compare

 @returns one line per disagreement with the manifest, empty when all agree
 */
function disagreements({
  manifest,
  workflow,
  releasable,
  lock,
}: {
  readonly manifest: string;
  readonly workflow: string;
  readonly releasable: string;
  readonly lock: string;
}): readonly string[] {
  const declared = [...manifestBins(manifest),].toSorted();
  const { built, packaged, } = builtAndPackaged(binaryJobLines(workflow));
  const checkBins = (JSON.parse(releasable) as Record<string, { readonly checkBins: readonly string[]; }>)[crateName]?.checkBins;
  const installed = (JSON.parse(lock) as { readonly tools: Record<string, { readonly bins: readonly string[]; }>; }).tools[`cargo:${crateName}`]?.bins;
  const found: string[] = [];
  for (const [label, listed,] of [
    ['the release job builds (cargo build --bin)', built,],
    ['the release job packages (bin=)', packaged,],
    ['releasable-crates.json checkBins lists', checkBins,],
    ['toolchain.lock.json installs', installed,],
  ] as const) {
    const sorted = [...listed ?? [],].toSorted();
    if (JSON.stringify(sorted) !== JSON.stringify(declared))
      found.push(`${label} ${JSON.stringify(sorted)}, but Cargo.toml declares ${JSON.stringify(declared)}`);
  }
  return found;
}

/**
 The texts of the real files.
 */
function realSources(): {
  readonly manifest: string;
  readonly workflow: string;
  readonly releasable: string;
  readonly lock: string;
} {
  return {
    manifest: readFileSync(
      resolve(
        repositoryRoot,
        'package/git-policy/cli/Cargo.toml',
      ),
      'utf8',
    ),
    workflow: readFileSync(
      resolve(
        repositoryRoot,
        '.github/workflows/cargo-publish.yml',
      ),
      'utf8',
    ),
    releasable: readFileSync(
      resolve(
        repositoryRoot,
        'package/crate-release/cli/releasable-crates.json',
      ),
      'utf8',
    ),
    lock: readFileSync(
      resolve(
        repositoryRoot,
        'toolchain.lock.json',
      ),
      'utf8',
    ),
  };
}

await describe({
  name: 'the crate declares the executables the release ships',
  children: [
    it({
      name: 'Cargo.toml, the release job, the release check and the toolchain lock list one and the same executable set',
      fn: async () => {
        const sources = realSources();
        expect(manifestBins(sources.manifest).length).toBeGreaterThan(0);
        expect(disagreements(sources)).toEqual([]);
      },
    }),
    it({
      name: 'a second [[bin]] in the manifest is named by every list that lacks it',
      fn: async () => {
        const sources = realSources();
        const planted = `${sources.manifest}\n[[bin]]\nname = "cli-git-planted-second"\npath = "src/native/planted.rs"\n`;
        const found = disagreements({
          ...sources,
          manifest: planted,
        });
        expect(found.length).toBe(4);
        expect(found.every((line) => line.includes('cli-git-planted-second'))).toBe(true);
      },
    }),
    it({
      name: 'a release job that is not found is an error, not an empty list',
      fn: async () => {
        expect(() => binaryJobLines('jobs:\n  other:\n')).toThrow('gpc-build-binary');
      },
    }),
  ],
});
