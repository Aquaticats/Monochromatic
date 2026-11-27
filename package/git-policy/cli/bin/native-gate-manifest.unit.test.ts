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

import {
  type GateManifest,
  writeGateManifest,
} from './native-gate-manifest.ts';

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
const suiteId = '3f2a9c1e'.repeat(8,);

/**
 The run id every file these checks write carries.
 */
const runId = 'native-HZPg8O';

/**
 The label every container of that run carries.
 */
const runLabel = `dev.monochromatic.git-policy.gate-run=${runId}`;

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
    clippyLimits: [
      '--rm',
      '--memory=8g',
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
    'gate-manifest-test-',
  ),);
}

/**
 The manifest one evidence directory holds, as its readers parse it.

 @param evidenceDirectory - directory the manifest was written into

 @returns every field the file records
 */
async function manifestIn(evidenceDirectory: string,): Promise<unknown> {
  return JSON.parse(await readFile(
    join(
      evidenceDirectory,
      'manifest.json',
    ),
    'utf8',
  ));
}

await describe({
  name: writeGateManifest.name,
  children: [
    it({
      name: 'writes the run and every container into the evidence directory, and no field for the directory itself',
      fn: async () => {
        await using tree = await evidenceTree();
        await writeGateManifest(manifestOf({
          evidenceDirectory: tree.path,
          containers: [suiteRecord,],
        },),);
        expect(await manifestIn(tree.path,),).toEqual({
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
          clippyLimits: [
            '--rm',
            '--memory=8g',
          ],
          run: {
            id: runId,
            label: runLabel,
            containers: [suiteRecord,],
          },
          user: '1000:1000',
        },);
      },
      timeout: 20_000,
    },),
  ],
},);
