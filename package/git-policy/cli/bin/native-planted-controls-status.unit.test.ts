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
  plantedControlsPassedRecord,
  plantedControlsRecordName,
} from './native-planted-controls-record.ts';
import {
  type PlantedStatus,
  plantedStatusName,
  settlePlantedStatus,
  writePlantedStatus,
} from './native-planted-controls-status.ts';
import { plantedControls } from './native-planted-controls.ts';
import { plantedHarnessVersion } from './native-planted-controls-harness.ts';

/**
 Image ID of the image under test.
 */
const image = 'a'.repeat(64);

/**
 Name prefix of campaign evidence directories.
 */
const prefix = 'native-mutation-';

/**
 Container arguments of the deferring campaign.
 */
const limits = [
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
  limits,
};

/**
 Create an evidence root.

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
  return await mkdtempDisposable(join(
    parent,
    'planted-status-',
  ));
}

/**
 Leave a record, as a controls unit does, in a new campaign directory.

 @param root - evidence root

 @returns the record's path

 @example
 ```ts
 await leaveRecord(root.path);
 ```
 */
async function leaveRecord(root: string): Promise<string> {
  const directory = join(
    root,
    `${prefix}unit`,
  );
  await mkdir(directory);
  const file = join(
    directory,
    plantedControlsRecordName,
  );
  await writeFile(
    file,
    JSON.stringify(plantedControlsPassedRecord({
      image,
      limits,
      controls: plantedControls,
      harness: plantedHarnessVersion,
    })),
  );
  return file;
}

await describe({
  name: settlePlantedStatus.name,
  children: [
    it({
      name: 'settles a pending campaign to validated, naming the record, once a matching record exists',
      fn: async () => {
        await using root = await evidenceRoot();
        const evidence = join(root.path, `${prefix}campaign`);
        await mkdir(evidence);
        await writePlantedStatus({ evidence, status: pending });
        const record = await leaveRecord(root.path);
        const settled = await settlePlantedStatus({
          evidence,
          evidenceRoot: root.path,
          evidencePrefix: prefix,
          unitRunning: async () => true,
        });
        expect(settled).toEqual([{ ...pending, status: 'validated', record }]);
        expect(
          JSON.parse(await readFile(join(evidence, plantedStatusName), 'utf8')),
        ).toEqual(settled[0]);
      },
    }),
    it({
      name: 'keeps a campaign pending while its controls unit runs without a record',
      fn: async () => {
        await using root = await evidenceRoot();
        const evidence = join(root.path, `${prefix}campaign`);
        await mkdir(evidence);
        await writePlantedStatus({ evidence, status: pending });
        const settled = await settlePlantedStatus({
          evidence,
          evidenceRoot: root.path,
          evidencePrefix: prefix,
          unitRunning: async () => true,
        });
        expect(settled).toEqual([pending]);
      },
    }),
    it({
      name: 'marks a campaign unvalidated, and persists it, when its controls unit is gone without a record',
      fn: async () => {
        await using root = await evidenceRoot();
        const evidence = join(root.path, `${prefix}campaign`);
        await mkdir(evidence);
        await writePlantedStatus({ evidence, status: pending });
        const settled = await settlePlantedStatus({
          evidence,
          evidenceRoot: root.path,
          evidencePrefix: prefix,
          unitRunning: async () => false,
        });
        expect(settled).toEqual([{ ...pending, status: 'unvalidated' }]);
        expect(
          JSON.parse(await readFile(join(evidence, plantedStatusName), 'utf8')),
        ).toMatchObject({ status: 'unvalidated' });
        // A record that appears later (a retry of the controls) settles it again.
        await leaveRecord(root.path);
        const again = await settlePlantedStatus({
          evidence,
          evidenceRoot: root.path,
          evidencePrefix: prefix,
          unitRunning: async () => false,
        });
        expect(again[0]?.status).toBe('validated');
      },
    }),
    it({
      name: 'does not validate on a record made under smaller limits than the campaign\'s',
      fn: async () => {
        await using root = await evidenceRoot();
        const evidence = join(root.path, `${prefix}campaign`);
        await mkdir(evidence);
        await writePlantedStatus({
          evidence,
          status: { ...pending, limits: ['--init', '--memory=4g', '--pids-limit=512'] },
        });
        await leaveRecord(root.path);
        // The record was made at 2g and the campaign ran at 4g: more than the record had, so it counts.
        const settled = await settlePlantedStatus({
          evidence,
          evidenceRoot: root.path,
          evidencePrefix: prefix,
          unitRunning: async () => false,
        });
        expect(settled[0]?.status).toBe('validated');
      },
    }),
    it({
      name: 'settles the pass-1 directory of a two-pass campaign too, and finds nothing for a campaign that deferred nothing',
      fn: async () => {
        await using root = await evidenceRoot();
        const evidence = join(root.path, `${prefix}campaign`);
        await mkdir(join(evidence, 'pass-1'), { recursive: true });
        expect(await settlePlantedStatus({ evidence, evidenceRoot: root.path, evidencePrefix: prefix })).toEqual([]);
        await writePlantedStatus({ evidence: join(evidence, 'pass-1'), status: pending });
        const settled = await settlePlantedStatus({
          evidence,
          evidenceRoot: root.path,
          evidencePrefix: prefix,
          unitRunning: async () => false,
        });
        expect(settled.map((status) => status.status)).toEqual(['unvalidated']);
      },
    }),
    it({
      name: 'sets aside a torn or malformed status file without throwing',
      fn: async () => {
        await using root = await evidenceRoot();
        const evidence = join(root.path, `${prefix}campaign`);
        await mkdir(evidence);
        await writeFile(join(evidence, plantedStatusName), '{"status": "pend');
        expect(await settlePlantedStatus({ evidence, evidenceRoot: root.path, evidencePrefix: prefix })).toEqual([]);
        await writeFile(join(evidence, plantedStatusName), JSON.stringify({ status: 'bogus', image, unit: 'u', limits }));
        expect(await settlePlantedStatus({ evidence, evidenceRoot: root.path, evidencePrefix: prefix })).toEqual([]);
      },
    }),
  ],
});
