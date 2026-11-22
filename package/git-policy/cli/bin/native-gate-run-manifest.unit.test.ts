import {
  mkdir,
  mkdtempDisposable,
  readFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { gateRunContainersOf, } from './native-gate-run-containers.ts';
import { gateRunManifestOf, } from './native-gate-run-manifest.ts';

/**
 Private scratch parent outside the repository, whose backing device is known.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 An ID in the shape podman writes, so the file's own text says which container it names.
 */
const clippyId = '9c1e3f2a'.repeat(8,);

/**
 An evidence directory of one run, removed when the caller's scope ends.

 @returns the directory, under the private scratch parent
 */
async function evidenceTree(): Promise<{ readonly path: string; readonly [Symbol.asyncDispose]: () => Promise<void>; }> {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  return await mkdtempDisposable(join(
    scratchParent,
    'gate-run-manifest-test-',
  ),);
}

/**
 The container records the manifest in one evidence directory holds.

 @param evidenceDirectory - directory the manifest was written into

 @returns every record under `run.containers`
 */
async function containersIn(evidenceDirectory: string,): Promise<readonly Readonly<Record<string, string>>[]> {
  /**
   The manifest as its readers parse it.
   */
  const written = JSON.parse(await readFile(
    join(
      evidenceDirectory,
      'manifest.json',
    ),
    'utf8',
  )) as {
    readonly run: { readonly containers: readonly Readonly<Record<string, string>>[] };
  };
  return written.run.containers;
}

await describe({
  name: gateRunManifestOf.name,
  children: [
    it({
      name: 'records each container\'s ID under its own role and writes the manifest before it answers',
      fn: async () => {
        await using tree = await evidenceTree();
        /**
         The run whose evidence directory names its containers.
         */
        const run = gateRunContainersOf({ evidenceDirectory: tree.path, },);
        /**
         This run's manifest, which starts with no ID read.
         */
        const evidenceManifest = gateRunManifestOf({
          evidenceDirectory: tree.path,
          base: 'a'.repeat(64),
          baseTag: 'localhost/git-policy-native-test-base:abc-def',
          cachyosImage: 'docker.io/cachyos/cachyos-v4:latest',
          cachyosImageId: 'b'.repeat(64),
          image: 'c'.repeat(64),
          fixtureRoot: {
            path: '/work/package/git-policy/cli/target/gate-fixtures',
            mount: 'rw,size=768m,mode=1777',
          },
          limits: ['--rm',],
          run,
          user: '1000:1000',
        },);
        await evidenceManifest.writeManifest();
        expect(await containersIn(tree.path,),).toEqual([
          {
            role: 'suite',
            name: run.suite.identity.name,
            label: run.label,
            cidFile: run.suite.cidFile,
          },
          {
            role: 'clippy',
            name: run.clippy.identity.name,
            label: run.label,
            cidFile: run.clippy.cidFile,
          },
        ],);
        await evidenceManifest.recorded('clippy',)(clippyId,);
        expect(await containersIn(tree.path,),).toEqual([
          {
            role: 'suite',
            name: run.suite.identity.name,
            label: run.label,
            cidFile: run.suite.cidFile,
          },
          {
            role: 'clippy',
            name: run.clippy.identity.name,
            label: run.label,
            cidFile: run.clippy.cidFile,
            id: clippyId,
          },
        ],);
      },
      timeout: 20_000,
    },),
  ],
},);
