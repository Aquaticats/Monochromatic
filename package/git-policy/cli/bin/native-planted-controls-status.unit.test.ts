import {
  mkdir,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  controlsKeyOf,
  plantedControlsPassedRecord,
  plantedControlsRecordName,
} from './native-planted-controls-record.ts';
import {
  type PlantedStatus,
  plantedStatusName,
  readPlantedStatus,
  retirePlantedStatus,
  writePlantedStatus,
} from './native-planted-controls-status-file.ts';
import {
  type SettledDirectory,
  settlePlantedStatus,
} from './native-planted-controls-status.ts';
import {
  isInvocationRunning,
  type UnitState,
} from './native-planted-controls-unit.ts';
import { plantedControls } from './native-planted-controls.ts';

/**
 Image ID of the image under test.
 */
const image = 'a'.repeat(64);

/**
 Name prefix of campaign evidence directories.
 */
const prefix = 'native-mutation-';

/**
 Identity of the runner and judge the campaign deferred under.
 */
const harness = 'harness-a';

/**
 Container arguments of the deferring campaign and of the record.
 */
const containerArguments = [
  '--init',
  '--memory=2g',
  '--pids-limit=512',
];

/**
 A pending status, as a deferring campaign writes it.
 */
const pending: PlantedStatus = {
  status: 'pending',
  image,
  unit: 'unit-x',
  invocation: 'invocation-1',
  limits: {
    memory: '2g',
    pidsLimit: 512,
  },
  controls: controlsKeyOf(plantedControls),
  harness,
};

/**
 The controls unit as systemd reports it while the deferred invocation runs.
 */
const runningUnit: UnitState = {
  state: 'active',
  invocation: 'invocation-1',
};

/**
 The controls unit after it ended and was unloaded.
 */
const goneUnit: UnitState = {
  state: '',
  invocation: '',
};

/**
 Create an evidence root with one campaign directory.

 @returns the disposable root; declare it with `await using`

 @example
 ```ts
 await using root = await evidenceRoot();
 ```
 */
async function evidenceRoot(): Promise<AsyncDisposable & { readonly path: string }> {
  const parent = join(
    homedir(),
    'temp',
    'agent',
  );
  await mkdir(
    parent,
    { recursive: true },
  );
  const root = await mkdtempDisposable(join(
    parent,
    'planted-status-',
  ));
  await mkdir(join(
    root.path,
    `${prefix}campaign`,
  ));
  return root;
}

/**
 Leave a record, as a controls unit does, in a new campaign directory.

 @param root - evidence root

 @param overrides - fields that replace the record's own

 @returns the record's path

 @example
 ```ts
 await leaveRecord({ root: root.path });
 ```
 */
async function leaveRecord({
  root,
  overrides = {},
}: {
  readonly root: string;
  readonly overrides?: Readonly<Record<string, unknown>>;
}): Promise<string> {
  const directory = join(
    root,
    `${prefix}unit`,
  );
  await mkdir(
    directory,
    { recursive: true },
  );
  const file = join(
    directory,
    plantedControlsRecordName,
  );
  await writeFile(
    file,
    JSON.stringify({
      ...plantedControlsPassedRecord({
        image,
        limits: containerArguments,
        controls: plantedControls,
        harness,
      }),
      ...overrides,
    }),
  );
  return file;
}

/**
 Settle the campaign directory of a root.

 @param root - evidence root

 @param inspect - what systemd reports of the controls unit

 @returns the settled directories

 @example
 ```ts
 await settle({ root: root.path, inspect: async () => runningUnit });
 ```
 */
async function settle({
  root,
  inspect,
}: {
  readonly root: string;
  readonly inspect: (unit: string) => Promise<UnitState>;
}): Promise<readonly SettledDirectory[]> {
  return await settlePlantedStatus({
    evidence: join(
      root,
      `${prefix}campaign`,
    ),
    evidenceRoot: root,
    evidencePrefix: prefix,
    inspect,
  });
}

/**
 Write the pending status into the campaign directory of a root.

 @param root - evidence root

 @param status - the status to write

 @example
 ```ts
 await defer({ root: root.path });
 ```
 */
async function defer({
  root,
  status = pending,
}: {
  readonly root: string;
  readonly status?: PlantedStatus;
}): Promise<void> {
  await writePlantedStatus({
    evidence: join(
      root,
      `${prefix}campaign`,
    ),
    status,
  });
}

/**
 Write a campaign manifest that names the unit it deferred to.

 @param root - evidence root

 @param manifest - the manifest's fields

 @example
 ```ts
 await writeManifest({ root: root.path, manifest: { plantedControlsDeferredTo: 'unit-x' } });
 ```
 */
async function writeManifest({
  root,
  manifest,
}: {
  readonly root: string;
  readonly manifest: Readonly<Record<string, unknown>>;
}): Promise<void> {
  await writeFile(
    join(
      root,
      `${prefix}campaign`,
      'manifest.json',
    ),
    JSON.stringify(manifest),
  );
}

/**
 The state of the only settled directory, or what went wrong.

 @param settled - the settled directories

 @returns their states

 @example
 ```ts
 statesOf(settled);
 ```
 */
function statesOf(settled: readonly SettledDirectory[]): readonly string[] {
  return settled.map(function stateOf(outcome) {
    return outcome.state;
  });
}

await describe({
  name: settlePlantedStatus.name,
  children: [
    it({
      name: 'settles a pending campaign to validated, naming the record, once a matching record exists, and keeps the record on the status',
      fn: async () => {
        await using root = await evidenceRoot();
        await defer({ root: root.path });
        const record = await leaveRecord({ root: root.path });
        const settled = await settle({ root: root.path, inspect: async () => runningUnit });
        expect(statesOf(settled)).toEqual(['validated']);
        const status: unknown = JSON.parse(await readFile(join(root.path, `${prefix}campaign`, plantedStatusName), 'utf8'));
        expect(status).toMatchObject({ status: 'validated', record });
      },
    }),
    it({
      name: 'keeps a campaign pending while its controls invocation runs without a record',
      fn: async () => {
        await using root = await evidenceRoot();
        await defer({ root: root.path });
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => runningUnit })),
        ).toEqual(['pending']);
      },
    }),
    it({
      name: 'marks a campaign unvalidated, and persists it, when its controls unit is gone without a record, and validates it when a record appears later',
      fn: async () => {
        await using root = await evidenceRoot();
        await defer({ root: root.path });
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['unvalidated']);
        const read = await readPlantedStatus(join(root.path, `${prefix}campaign`));
        expect(read.kind === 'valid' ? read.status.status : read.kind).toBe('unvalidated');
        await leaveRecord({ root: root.path });
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['validated']);
      },
    }),
    it({
      name: 'does not let another invocation under the same unit name keep a failed dependency pending',
      fn: async () => {
        await using root = await evidenceRoot();
        await defer({ root: root.path });
        const reused: UnitState = { state: 'active', invocation: 'invocation-2' };
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => reused })),
        ).toEqual(['unvalidated']);
        expect(isInvocationRunning({ state: reused, invocation: 'invocation-1' })).toBe(false);
        expect(isInvocationRunning({ state: runningUnit, invocation: 'invocation-1' })).toBe(true);
        expect(isInvocationRunning({ state: { state: 'failed', invocation: 'invocation-1' }, invocation: 'invocation-1' })).toBe(false);
        expect(isInvocationRunning({ state: goneUnit, invocation: '' })).toBe(false);
      },
    }),
    it({
      name: 'looks for the record once more after the unit is seen to have ended, so a record published in between validates',
      fn: async () => {
        await using root = await evidenceRoot();
        await defer({ root: root.path });
        // The unit publishes its record just as settlement asks whether it still runs.
        const publishing = async (): Promise<UnitState> => {
          await leaveRecord({ root: root.path });
          return goneUnit;
        };
        expect(
          statesOf(await settle({ root: root.path, inspect: publishing })),
        ).toEqual(['validated']);
      },
    }),
    it({
      name: 'matches records against the controls, harness and limits it deferred under, not the settling process\'s own',
      fn: async () => {
        await using root = await evidenceRoot();
        const [first] = plantedControls;
        // The campaign deferred under a table of one control; a record for the whole table does not prove it, and a
        // record for the saved key does.
        await defer({
          root: root.path,
          status: { ...pending, controls: controlsKeyOf([first as (typeof plantedControls)[number]]) },
        });
        await leaveRecord({ root: root.path });
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['unvalidated']);
        await defer({ root: root.path, status: { ...pending, harness: 'harness-b' } });
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['unvalidated']);
        await defer({ root: root.path, status: { ...pending, limits: { memory: '4g', pidsLimit: 1_024 } } });
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['validated']);
      },
    }),
    it({
      name: 'checks the saved record again instead of trusting the word validated',
      fn: async () => {
        await using root = await evidenceRoot();
        await defer({ root: root.path });
        const record = await leaveRecord({ root: root.path });
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['validated']);
        // The record is replaced by one of another runner and judge: the saved status no longer proves anything.
        await writeFile(record, JSON.stringify({ ...JSON.parse(await readFile(record, 'utf8')), harnessId: 'harness-b' }));
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['unvalidated']);
      },
    }),
    it({
      name: 'fails closed for a status that cannot be read, is incomplete, has malformed limits, or disagrees with the manifest',
      fn: async () => {
        await using root = await evidenceRoot();
        const file = join(root.path, `${prefix}campaign`, plantedStatusName);
        await writeFile(file, '{"status": "pend');
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => runningUnit })),
        ).toEqual(['invalid']);
        await writeFile(file, JSON.stringify({ ...pending, limits: ['--memory=2g'] }));
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => runningUnit })),
        ).toEqual(['invalid']);
        await writeFile(file, JSON.stringify({ ...pending, controls: { sha256: 'x', names: ['a', 1] } }));
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => runningUnit })),
        ).toEqual(['invalid']);
        await writeFile(file, JSON.stringify({ ...pending, status: 'bogus' }));
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => runningUnit })),
        ).toEqual(['invalid']);
        await defer({ root: root.path });
        await writeManifest({ root: root.path, manifest: { plantedControlsDeferredTo: 'another-unit' } });
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => runningUnit })),
        ).toEqual(['invalid']);
      },
    }),
    it({
      name: 'fails closed when the manifest records a deferral but the status is absent or retired',
      fn: async () => {
        await using root = await evidenceRoot();
        await writeManifest({ root: root.path, manifest: { plantedControlsDeferredTo: 'unit-x' } });
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => runningUnit })),
        ).toEqual(['invalid']);
        await retirePlantedStatus({ evidence: join(root.path, `${prefix}campaign`), by: 'ran' });
        // Nothing to retire: no status existed, so nothing was written and the manifest's claim still has no status.
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => runningUnit })),
        ).toEqual(['invalid']);
        await writePlantedStatus({ evidence: join(root.path, `${prefix}campaign`), status: { status: 'retired', retiredBy: 'ran' } });
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => runningUnit })),
        ).toEqual(['invalid']);
      },
    }),
    it({
      name: 'finds nothing for a campaign that deferred nothing, and settles the pass-1 directory too',
      fn: async () => {
        await using root = await evidenceRoot();
        expect(await settle({ root: root.path, inspect: async () => runningUnit })).toEqual([]);
        const passOne = join(root.path, `${prefix}campaign`, 'pass-1');
        await mkdir(passOne);
        await writePlantedStatus({ evidence: passOne, status: pending });
        const settled = await settle({ root: root.path, inspect: async () => goneUnit });
        expect(settled.map((outcome) => [outcome.directory, outcome.state])).toEqual([[passOne, 'unvalidated']]);
      },
    }),
    it({
      name: 'treats a retired status as settled when the manifest agrees, and retires a deferred or unreadable one',
      fn: async () => {
        await using root = await evidenceRoot();
        const evidence = join(root.path, `${prefix}campaign`);
        await defer({ root: root.path });
        await retirePlantedStatus({ evidence, by: 'proved' });
        expect(
          statesOf(await settle({ root: root.path, inspect: async () => runningUnit })),
        ).toEqual(['retired']);
        await writeFile(join(evidence, plantedStatusName), '{torn');
        await retirePlantedStatus({ evidence, by: 'ran' });
        expect(
          JSON.parse(await readFile(join(evidence, plantedStatusName), 'utf8')),
        ).toEqual({ status: 'retired', retiredBy: 'ran' });
      },
    }),
  ],
});
