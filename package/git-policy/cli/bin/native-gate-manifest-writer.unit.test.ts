import {
  mkdir,
  mkdtempDisposable,
  readFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { GateManifest } from './native-gate-manifest.ts';
import { gateManifestWriterOf, } from './native-gate-manifest-writer.ts';

/**
 Private scratch parent outside the repository, whose backing device is known.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 The run id every file these checks write carries.
 */
const runId = 'native-HZPg8O';

/**
 The label every container of that run carries.
 */
const runLabel = `dev.monochromatic.git-policy.gate-run=${runId}`;

/**
 An ID in the shape podman writes, so the file's own text says which container it names.
 */
const suiteId = '3f2a9c1e'.repeat(8,);

/**
 The suite's record as the run writes it once its client reported the ID.
 */
const suiteRecord = {
  role: 'suite',
  name: `gpc-gate-${runId}-suite`,
  label: runLabel,
  cidFile: `target/verification/${runId}/suite.cid`,
  id: suiteId,
} as const;

/**
 One manifest of a run, as the runner builds it from the fields it knows at that moment.

 @param evidenceDirectory - directory the manifest is written into

 @param containers - the container records the run holds now

 @returns every field the manifest records, the directory among them
 */
function manifestOf({
  evidenceDirectory,
  containers,
}: {
  readonly evidenceDirectory: string;
  readonly containers: readonly Readonly<Record<string, string>>[];
},): GateManifest {
  return {
    evidenceDirectory,
    base: 'a'.repeat(64),
    baseTag: 'localhost/git-policy-native-test-base:abc-def',
    cachyosImage: 'docker.io/cachyos/cachyos-v4:latest',
    cachyosImageId: 'b'.repeat(64),
    image: 'c'.repeat(64),
    fixtureRoot: {
      path: '/work/package/git-policy/cli/target/gate-fixtures',
      mount: 'rw,size=768m,mode=1777',
    },
    limits: [
      '--rm',
      '--memory=2g',
    ],
    run: {
      id: runId,
      label: runLabel,
      containers,
    },
    user: '1000:1000',
  };
}

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
    'gate-manifest-writer-test-',
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
  name: gateManifestWriterOf.name,
  children: [
    it({
      name: 'writes one manifest at a time, so two containers reporting their IDs at once never interleave',
      fn: async () => {
        await using tree = await evidenceTree();
        /**
         Every start and end of a write, in the order they ran.
         */
        const events: string[] = [];
        /**
         A write slow enough that an unserialized second one would start before the first ended.
         */
        async function slowWrite(): Promise<void> {
          events.push('start',);
          await wait(10,);
          events.push('end',);
        }
        /**
         The writer whose writes are observed.
         */
        const writeManifest = gateManifestWriterOf({
          manifestOf: function emptyManifest(): GateManifest {
            return manifestOf({
              evidenceDirectory: tree.path,
              containers: [],
            },);
          },
          write: slowWrite,
        },);
        await Promise.all([
          writeManifest(),
          writeManifest(),
        ],);
        expect(events,).toEqual([
          'start',
          'end',
          'start',
          'end',
        ],);
      },
      timeout: 20_000,
    },),
    it({
      name: 'writes the IDs the run holds when each write is asked for, so a manifest read mid-run names the containers',
      fn: async () => {
        await using tree = await evidenceTree();
        /**
         The records the run holds, which the suite's client adds its ID to while it runs.
         */
        let containers: readonly Readonly<Record<string, string>>[] = [];
        /**
         This run's writer, which builds each manifest as the run stands at that write.
         */
        const writeManifest = gateManifestWriterOf({
          manifestOf: function currentManifest(): GateManifest {
            return manifestOf({
              evidenceDirectory: tree.path,
              containers,
            },);
          },
        },);
        await writeManifest();
        expect(await containersIn(tree.path,),).toEqual([],);
        containers = [suiteRecord,];
        await writeManifest();
        expect(await containersIn(tree.path,),).toEqual([suiteRecord,],);
      },
      timeout: 20_000,
    },),
  ],
},);
