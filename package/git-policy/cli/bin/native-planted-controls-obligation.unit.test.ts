import {
  mkdir,
  mkdtempDisposable,
  readdir,
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
  obligationField,
  obligationIdentity,
  type PlantedObligation,
} from './native-planted-controls-manifest.ts';
import {
  assertNotUnmet,
  describeValidation,
  type Validation,
  validatePlanted,
} from './native-planted-controls-obligation.ts';
import {
  controlsKeyOf,
  plantedControlsPassedRecord,
  plantedControlsRecordName,
} from './native-planted-controls-record.ts';
import type { UnitState } from './native-planted-controls-unit.ts';
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
 Container arguments of the record.
 */
const containerArguments = [
  '--init',
  '--memory=2g',
  '--pids-limit=512',
];

/**
 The obligation a deferring campaign writes into its manifest.
 */
const obligation: PlantedObligation = {
  image,
  unit: 'unit-x',
  log: '/logs/unit-x.log.1',
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
    'planted-obligation-',
  ));
  await mkdir(join(
    root.path,
    `${prefix}campaign`,
  ));
  return root;
}

/**
 Write the campaign's manifest.

 @param root - evidence root

 @param manifest - the manifest's fields

 @param directory - the directory under the campaign that receives it; the campaign directory itself by default

 @example
 ```ts
 await writeManifest({ root: root.path, manifest: { [obligationField]: obligation } });
 ```
 */
async function writeManifest({
  root,
  manifest,
  directory = '',
}: {
  readonly root: string;
  readonly manifest: Readonly<Record<string, unknown>>;
  readonly directory?: string;
}): Promise<void> {
  await writeFile(
    join(
      root,
      `${prefix}campaign`,
      directory,
      'manifest.json',
    ),
    JSON.stringify(manifest),
  );
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
 Validate the campaign directory of a root.

 @param root - evidence root

 @param inspect - what systemd reports of the controls unit

 @returns the validations

 @example
 ```ts
 await validate({ root: root.path, inspect: async () => runningUnit });
 ```
 */
async function validate({
  root,
  inspect,
}: {
  readonly root: string;
  readonly inspect: (unit: string) => Promise<UnitState>;
}): Promise<readonly Validation[]> {
  return await validatePlanted({
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
 The states of validations.

 @param validations - the validations

 @returns their states

 @example
 ```ts
 statesOf(validations);
 ```
 */
function statesOf(validations: readonly Validation[]): readonly string[] {
  return validations.map(function stateOf(validation) {
    return validation.state;
  });
}

await describe({
  name: validatePlanted.name,
  children: [
    it({
      name: 'finds nothing for a campaign whose manifest defers nothing, or that has no manifest yet, and writes nothing',
      fn: async () => {
        await using root = await evidenceRoot();
        expect(await validate({ root: root.path, inspect: async () => runningUnit })).toEqual([]);
        await writeManifest({ root: root.path, manifest: { baseImage: image } });
        expect(await validate({ root: root.path, inspect: async () => runningUnit })).toEqual([]);
        expect(
          await readdir(join(root.path, `${prefix}campaign`)),
        ).toEqual(['manifest.json']);
      },
    }),
    it({
      name: 'validates a deferred campaign, naming the record, once a matching record exists',
      fn: async () => {
        await using root = await evidenceRoot();
        await writeManifest({ root: root.path, manifest: { [obligationField]: obligation } });
        const record = await leaveRecord({ root: root.path });
        const validations = await validate({ root: root.path, inspect: async () => runningUnit });
        expect(statesOf(validations)).toEqual(['validated']);
        expect((validations[0] !== undefined) && (validations[0].state === 'validated') ? validations[0].record : '').toBe(record);
        expect(describeValidation(validations[0] as Validation).includes(record)).toBe(true);
      },
    }),
    it({
      name: 'is pending while the deferred invocation runs without a record, and says where the unit and its log are',
      fn: async () => {
        await using root = await evidenceRoot();
        await writeManifest({ root: root.path, manifest: { [obligationField]: obligation } });
        const validations = await validate({ root: root.path, inspect: async () => runningUnit });
        expect(statesOf(validations)).toEqual(['pending']);
        const line = describeValidation(validations[0] as Validation);
        expect(['no record yet', 'unit-x', 'invocation-1', '/logs/unit-x.log.1'].every((part) => line.includes(part))).toBe(true);
      },
    }),
    it({
      name: 'fails, as unvalidated, when that invocation ended without a matching record, and validates when a record appears later',
      fn: async () => {
        await using root = await evidenceRoot();
        await writeManifest({ root: root.path, manifest: { [obligationField]: obligation } });
        const ended = await validate({ root: root.path, inspect: async () => goneUnit });
        expect(statesOf(ended)).toEqual(['unvalidated']);
        expect(describeValidation(ended[0] as Validation).includes('ended without a matching record')).toBe(true);
        await leaveRecord({ root: root.path });
        expect(
          statesOf(await validate({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['validated']);
      },
    }),
    it({
      name: 'does not let another invocation under the same unit name keep a failed dependency pending',
      fn: async () => {
        await using root = await evidenceRoot();
        await writeManifest({ root: root.path, manifest: { [obligationField]: obligation } });
        const reused: UnitState = { state: 'active', invocation: 'invocation-2' };
        expect(
          statesOf(await validate({ root: root.path, inspect: async () => reused })),
        ).toEqual(['unvalidated']);
        const failed: UnitState = { state: 'failed', invocation: 'invocation-1' };
        expect(
          statesOf(await validate({ root: root.path, inspect: async () => failed })),
        ).toEqual(['unvalidated']);
      },
    }),
    it({
      name: 'looks for the record once more after the unit is seen to have ended, so a record published in between validates',
      fn: async () => {
        await using root = await evidenceRoot();
        await writeManifest({ root: root.path, manifest: { [obligationField]: obligation } });
        // The unit publishes its record just as validation asks whether it still runs.
        const publishing = async (): Promise<UnitState> => {
          await leaveRecord({ root: root.path });
          return goneUnit;
        };
        expect(
          statesOf(await validate({ root: root.path, inspect: publishing })),
        ).toEqual(['validated']);
      },
    }),
    it({
      name: 'matches records against the controls, harness and limits it deferred under, not the asking process\'s own',
      fn: async () => {
        await using root = await evidenceRoot();
        const [first] = plantedControls;
        await leaveRecord({ root: root.path });
        // Deferred under a table of one control: a record for the whole table does not prove it.
        await writeManifest({
          root: root.path,
          manifest: { [obligationField]: { ...obligation, controls: controlsKeyOf([first as (typeof plantedControls)[number]]) } },
        });
        expect(
          statesOf(await validate({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['unvalidated']);
        await writeManifest({ root: root.path, manifest: { [obligationField]: { ...obligation, harness: 'harness-b' } } });
        expect(
          statesOf(await validate({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['unvalidated']);
        // A campaign that ran with more than the record had is proved by it; one that ran with less is not.
        await writeManifest({
          root: root.path,
          manifest: { [obligationField]: { ...obligation, limits: { memory: '4g', pidsLimit: 1_024 } } },
        });
        expect(
          statesOf(await validate({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['validated']);
        await writeManifest({
          root: root.path,
          manifest: { [obligationField]: { ...obligation, limits: { memory: '1g', pidsLimit: 64 } } },
        });
        expect(
          statesOf(await validate({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['unvalidated']);
      },
    }),
    it({
      name: 'fails closed, as invalid, for a manifest that cannot be read or whose obligation is incomplete or malformed',
      fn: async () => {
        await using root = await evidenceRoot();
        const file = join(root.path, `${prefix}campaign`, 'manifest.json');
        await writeFile(file, '{"baseImage": "x"');
        expect(
          statesOf(await validate({ root: root.path, inspect: async () => runningUnit })),
        ).toEqual(['invalid']);
        const malformed = [
          null,
          'unit-x',
          [obligation],
          { ...obligation, limits: ['--memory=2g'] },
          { ...obligation, limits: { memory: '2g', pidsLimit: -1 } },
          { ...obligation, limits: { memory: '2g', pidsLimit: 0 } },
          { ...obligation, limits: { memory: '2g', pidsLimit: 1.5 } },
          { ...obligation, limits: { memory: '0', pidsLimit: 512 } },
          { ...obligation, controls: { sha256: 'x', names: ['a', 1] } },
          { ...obligation, invocation: undefined },
          { ...obligation, log: 7 },
        ];
        const states = await Promise.all(malformed.map(async function stateOfMalformed(field, index) {
          await using other = await evidenceRoot();
          await writeManifest({ root: other.path, manifest: { [obligationField]: field } });
          const validations = await validate({ root: other.path, inspect: async () => runningUnit });
          return `${String(index)}:${statesOf(validations).join(",")}`;
        }));
        expect(states).toEqual(malformed.map((_field, index) => `${String(index)}:invalid`));
      },
    }),
    it({
      name: 'ends an earlier obligation by construction: an attempt that did not defer writes a manifest without the field',
      fn: async () => {
        await using root = await evidenceRoot();
        await writeManifest({ root: root.path, manifest: { [obligationField]: obligation } });
        expect(
          statesOf(await validate({ root: root.path, inspect: async () => goneUnit })),
        ).toEqual(['unvalidated']);
        await writeManifest({ root: root.path, manifest: { baseImage: image } });
        expect(await validate({ root: root.path, inspect: async () => goneUnit })).toEqual([]);
      },
    }),
    it({
      name: 'fails finalization for an unvalidated or invalid obligation, and not for a validated or pending one',
      fn: async () => {
        await using root = await evidenceRoot();
        await writeManifest({ root: root.path, manifest: { [obligationField]: obligation } });
        const pendingOnes = await validate({ root: root.path, inspect: async () => runningUnit });
        const unvalidated = await validate({ root: root.path, inspect: async () => goneUnit });
        expect(() => assertNotUnmet({ validations: pendingOnes })).not.toThrow();
        expect(() => assertNotUnmet({ validations: unvalidated })).toThrow('ended without a matching record');
        expect(() => assertNotUnmet({ validations: [{ directory: '/d', state: 'invalid', reason: 'torn' }] })).toThrow('torn');
        await leaveRecord({ root: root.path });
        expect(() => assertNotUnmet({ validations: [] })).not.toThrow();
        expect(() => assertNotUnmet({ validations: [{ directory: '/d', state: 'validated', obligation, record: '/r' }] })).not.toThrow();
      },
    }),
    it({
      name: 'validates the pass-1 directory of a two-pass campaign too',
      fn: async () => {
        await using root = await evidenceRoot();
        await mkdir(join(root.path, `${prefix}campaign`, 'pass-1'));
        await writeManifest({ root: root.path, directory: 'pass-1', manifest: { [obligationField]: obligation } });
        const validations = await validate({ root: root.path, inspect: async () => goneUnit });
        expect(validations.map((validation) => [validation.directory, validation.state])).toEqual([
          [join(root.path, `${prefix}campaign`, 'pass-1'), 'unvalidated'],
        ]);
      },
    }),
    it({
      name: 'treats a campaign that finished (exit.json) without a manifest as invalid, and one that has not run yet, or a controls-only directory, as nothing',
      fn: async () => {
        await using root = await evidenceRoot();
        expect(await validate({ root: root.path, inspect: async () => runningUnit })).toEqual([]);
        await writeFile(join(root.path, `${prefix}campaign`, 'exit.json'), '{}');
        expect(
          statesOf(await validate({ root: root.path, inspect: async () => runningUnit })),
        ).toEqual(['invalid']);
      },
    }),
    it({
      name: 'fails finalization when an obligation this process expects is no longer held by the manifests',
      fn: async () => {
        await using root = await evidenceRoot();
        await writeManifest({ root: root.path, manifest: { baseImage: image } });
        const none = await validate({ root: root.path, inspect: async () => runningUnit });
        expect(() => assertNotUnmet({ validations: none, expected: [obligation] })).toThrow('no longer holds the obligation on unit unit-x');
        await writeManifest({ root: root.path, manifest: { [obligationField]: obligation } });
        const held = await validate({ root: root.path, inspect: async () => runningUnit });
        expect(() => assertNotUnmet({ validations: held, expected: [obligation] })).not.toThrow();
        // The same obligation built in another field order is the same obligation.
        const reordered: PlantedObligation = { harness, controls: structuredClone(obligation.controls), limits: structuredClone(obligation.limits), invocation: obligation.invocation, log: obligation.log, unit: obligation.unit, image };
        expect(obligationIdentity(reordered)).toBe(obligationIdentity(obligation));
      },
    }),
    it({
      name: 'validates the campaign directory and the pass-1 directory of a two-pass campaign each by its own manifest',
      fn: async () => {
        await using root = await evidenceRoot();
        await mkdir(join(root.path, `${prefix}campaign`, 'pass-1'));
        await writeManifest({ root: root.path, directory: 'pass-1', manifest: { [obligationField]: obligation } });
        await writeManifest({ root: root.path, manifest: { [obligationField]: { ...obligation, unit: 'unit-y' } } });
        const validations = await validate({ root: root.path, inspect: async () => goneUnit });
        expect(validations.map((validation) => [validation.directory, validation.state])).toEqual([
          [join(root.path, `${prefix}campaign`), 'unvalidated'],
          [join(root.path, `${prefix}campaign`, 'pass-1'), 'unvalidated'],
        ]);
        // A pass that proved or ran the controls writes a manifest without the field and leaves the other pass's alone.
        await writeManifest({ root: root.path, manifest: { baseImage: image } });
        expect((await validate({ root: root.path, inspect: async () => goneUnit })).map((validation) => validation.directory))
          .toEqual([join(root.path, `${prefix}campaign`, 'pass-1')]);
      },
    }),
  ],
});
