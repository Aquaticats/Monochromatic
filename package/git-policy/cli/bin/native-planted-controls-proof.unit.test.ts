import {
  mkdir,
  mkdtempDisposable,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ensurePlantedControls,
  passingPlantedControls,
  plantedControlsDigest,
  plantedControlsPassedRecord,
  plantedControlsRecordName,
  writePlantedControlsRecord,
} from './native-planted-controls-proof.ts';
import {
  type PlantedControl,
  plantedControls,
  type PlantedControlSetting,
} from './native-planted-controls.ts';

/**
 Image ID of the image a campaign is asked about.
 */
const image = 'a'.repeat(64);

/**
 Image ID of some other image.
 */
const otherImage = 'b'.repeat(64);

/**
 Name prefix of campaign evidence directories, as the campaign uses it.
 */
const prefix = 'native-mutation-';

/**
 Container arguments of a campaign at the defaults, as the campaign hands them to the planted controls.
 */
const containerArguments = [
  '--init',
  '--network=none',
  '--memory=2g',
  '--cpus=2',
  '--pids-limit=512',
];

/**
 Limits of a campaign as large as the one that made the record.
 */
const campaignLimits = {
  memory: '2g',
  pidsLimit: 512,
};

/**
 A table of two small controls, so that the tests do not depend on the real table's size.
 */
const controls: readonly PlantedControl[] = [
  {
    name: 'first guard',
    file: 'src/native/a.rs',
    from: 'if guard {',
    to: 'if false {',
    failing: 'a::tests::first',
  },
  {
    name: 'second guard',
    file: 'src/native/b.rs',
    from: 'if other {',
    to: 'if false {',
    failing: 'b::tests::second',
  },
];

/**
 Scratch parent of the test directories; never the system temporary directory, whose backing device is unknowable.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 A bound on a wait that ends, and cancels the wait, when the scope that holds it ends.

 @returns the signal to give the wait; declare it with `await using`

 @example
 ```ts
 await using bound = cancellableBound();
 ```
 */
function cancellableBound(): AsyncDisposable & { readonly signal: AbortSignal } {
  const abort = new AbortController();
  return {
    signal: abort.signal,
    async [Symbol.asyncDispose]() {
      abort.abort();
    },
  };
}

/**
 Fail after a long wait unless the signal ends it first, to bound a test that would otherwise hang.

 @param signal - aborted once the test no longer needs the bound

 @throws Error when the wait ends by itself
 */
async function failAfterWaiting(signal: AbortSignal): Promise<void> {
  try {
    await sleep(
      30_000,
      undefined,
      { signal },
    );
  }
  catch (error) {
    // The test finished first and cancelled the wait; the race ignores this result.
    if (signal.aborted)
      return;
    throw error;
  }
  throw new Error('A campaign waited for the other one.');
}

/**
 Create the evidence root a test fills.

 @returns the disposable root; declare it with `await using`

 @example
 ```ts
 await using root = await evidenceRoot();
 ```
 */
async function evidenceRoot(): Promise<AsyncDisposable & { readonly path: string }> {
  await mkdir(
    scratchParent,
    { recursive: true },
  );
  return await mkdtempDisposable(join(
    scratchParent,
    'planted-proof-',
  ));
}

/**
 Create a campaign evidence directory.

 @param root - evidence root

 @param name - directory name

 @returns the directory path

 @example
 ```ts
 const evidence = await campaignDirectory({ root: root.path, name: 'native-mutation-one' });
 ```
 */
async function campaignDirectory({
  root,
  name,
}: {
  readonly root: string;
  readonly name: string;
}): Promise<string> {
  const directory = join(
    root,
    name,
  );
  await mkdir(
    directory,
    { recursive: true },
  );
  return directory;
}

/**
 Leave a passed record in a campaign directory, the way a campaign that ran the controls does.

 @param evidence - the campaign's evidence directory

 @param overrides - fields that replace the record's own

 @example
 ```ts
 await leaveRecord({ evidence });
 ```
 */
async function leaveRecord({
  evidence,
  overrides = {},
  table = controls,
}: {
  readonly evidence: string;
  readonly overrides?: Readonly<Record<string, unknown>>;
  readonly table?: readonly PlantedControl[];
}): Promise<void> {
  await writeFile(
    join(
      evidence,
      plantedControlsRecordName,
    ),
    JSON.stringify({
      ...plantedControlsPassedRecord({
        image,
        limits: containerArguments,
        controls: table,
      }),
      ...overrides,
    }),
  );
}

/**
 Ask whether a campaign finds proof.

 @param root - evidence root

 @param limits - the campaign's limits

 @param table - the controls the campaign would plant

 @returns the proving records

 @example
 ```ts
 await find({ root: root.path });
 ```
 */
async function find({
  root,
  limits = campaignLimits,
  table = controls,
  asked = image,
}: {
  readonly root: string;
  readonly limits?: { readonly memory: string; readonly pidsLimit: number };
  readonly table?: readonly PlantedControl[];
  readonly asked?: string;
}): Promise<readonly string[]> {
  return await passingPlantedControls({
    evidenceRoot: root,
    evidencePrefix: prefix,
    image: asked,
    limits,
    controls: table,
  });
}

/**
 Inputs of one campaign's planted-controls step.

 @param evidence - the campaign's evidence directory

 @param limits - the container arguments of the campaign

 @returns the setting the controls would receive

 @example
 ```ts
 settingIn(evidence);
 ```
 */
function settingIn(evidence: string, limits: readonly string[] = containerArguments): PlantedControlSetting {
  return {
    base: image,
    limits,
    context: evidence,
    evidence,
  };
}

/**
 Run the decision step the way a campaign does, counting how often the controls ran.

 @param root - evidence root

 @param evidence - the campaign's evidence directory

 @returns how often the controls ran

 @example
 ```ts
 await decide({ root: root.path, evidence });
 ```
 */
async function decide({
  root,
  evidence,
  scoredMutants = 0,
  skipRequested = false,
  limits = containerArguments,
  run,
}: {
  readonly root: string;
  readonly evidence: string;
  readonly scoredMutants?: number;
  readonly skipRequested?: boolean;
  readonly limits?: readonly string[];
  readonly run?: (setting: PlantedControlSetting) => Promise<void>;
}): Promise<number> {
  let ran = 0;
  await ensurePlantedControls({
    setting: settingIn(
      evidence,
      limits,
    ),
    evidenceRoot: root,
    evidencePrefix: prefix,
    scoredMutants,
    skipRequested,
    controls,
    run: run ?? async function countedRun() {
      ran += 1;
    },
  });
  return ran;
}

await describe({
  name: 'the proof that the planted controls passed on an image',
  children: [
    describe({
      name: plantedControlsDigest.name,
      children: [
        it({
          name: 'does not depend on the order of the table',
          fn: async () => {
            expect(
              plantedControlsDigest(controls.toReversed()),
            )
              .toBe(plantedControlsDigest(controls));
          },
        }),
        it({
          name: 'changes with every part of a control and with a control added or removed',
          fn: async () => {
            const [first, second] = controls;
            const base = plantedControlsDigest(controls);
            const variants = [
              { ...first, name: 'renamed' },
              { ...first, file: 'src/native/other.rs' },
              { ...first, from: 'if changed {' },
              { ...first, to: 'if true {' },
              { ...first, failing: 'a::tests::other' },
            ].map(function digestWith(changed) {
              return plantedControlsDigest([changed as PlantedControl, second as PlantedControl]);
            });
            expect([
              ...variants,
              plantedControlsDigest([first as PlantedControl]),
              plantedControlsDigest([...controls, { ...first, name: 'third' } as PlantedControl]),
            ].every(function differs(digest) {
              return digest !== base;
            })).toBe(true);
          },
        }),
      ],
    }),
    describe({
      name: plantedControlsPassedRecord.name,
      children: [
        it({
          name: 'names the image, the container arguments, the control names and their digest, so the record stands alone',
          fn: async () => {
            expect(plantedControlsPassedRecord({
              image,
              limits: containerArguments,
              controls,
            })).toEqual({
              plantedControls: true,
              image,
              limits: containerArguments,
              controls: controls.map(function nameOf(control) {
                return control.name;
              }),
              controlsSha256: plantedControlsDigest(controls),
            });
          },
        }),
      ],
    }),
    describe({
      name: writePlantedControlsRecord.name,
      children: [
        it({
          name: 'leaves the record and no temporary file, and replaces an earlier record',
          fn: async () => {
            await using root = await evidenceRoot();
            const evidence = await campaignDirectory({ root: root.path, name: `${prefix}one` });
            const record = plantedControlsPassedRecord({ image, limits: containerArguments, controls });
            const path = await writePlantedControlsRecord({ evidence, record: { stale: true } });
            await writePlantedControlsRecord({ evidence, record });
            expect(path).toBe(join(evidence, plantedControlsRecordName));
            expect(
              JSON.parse(await readFile(path, 'utf8')),
            ).toEqual(record);
            expect(await readdir(evidence)).toEqual([plantedControlsRecordName]);
          },
        }),
        it({
          name: 'throws and leaves no temporary file when the evidence directory is gone',
          fn: async () => {
            await using root = await evidenceRoot();
            await expect(writePlantedControlsRecord({
              evidence: join(root.path, 'missing'),
              record: {},
            })).rejects.toThrow();
            expect(await readdir(root.path)).toEqual([]);
          },
        }),
      ],
    }),
    describe({
      name: passingPlantedControls.name,
      children: [
        it({
          name: 'names the record of a campaign whose controls passed on the asked image',
          fn: async () => {
            await using root = await evidenceRoot();
            const evidence = await campaignDirectory({ root: root.path, name: `${prefix}one` });
            await leaveRecord({ evidence });
            expect(await find({ root: root.path }))
              .toEqual([join(evidence, plantedControlsRecordName)]);
          },
        }),
        it({
          name: 'finds the record of a two-pass campaign in its pass-1 directory',
          fn: async () => {
            await using root = await evidenceRoot();
            const evidence = await campaignDirectory({ root: root.path, name: `${prefix}one/pass-1` });
            await leaveRecord({ evidence });
            expect(await find({ root: root.path }))
              .toEqual([join(evidence, plantedControlsRecordName)]);
          },
        }),
        it({
          name: 'accepts a campaign with more memory and more processes than the record',
          fn: async () => {
            await using root = await evidenceRoot();
            await leaveRecord({ evidence: await campaignDirectory({ root: root.path, name: `${prefix}one` }) });
            expect((await find({
              root: root.path,
              limits: { memory: '3g', pidsLimit: 1_024 },
            })).length).toBe(1);
          },
        }),
        it({
          name: 'refuses a campaign with less memory than the record',
          fn: async () => {
            await using root = await evidenceRoot();
            await leaveRecord({ evidence: await campaignDirectory({ root: root.path, name: `${prefix}one` }) });
            expect(await find({
              root: root.path,
              limits: { memory: '1g', pidsLimit: 512 },
            })).toEqual([]);
          },
        }),
        it({
          name: 'refuses a campaign with fewer processes than the record',
          fn: async () => {
            await using root = await evidenceRoot();
            await leaveRecord({ evidence: await campaignDirectory({ root: root.path, name: `${prefix}one` }) });
            expect(await find({
              root: root.path,
              limits: { memory: '2g', pidsLimit: 128 },
            })).toEqual([]);
          },
        }),
        it({
          name: 'refuses a record of another image',
          fn: async () => {
            await using root = await evidenceRoot();
            await leaveRecord({ evidence: await campaignDirectory({ root: root.path, name: `${prefix}one` }) });
            expect(await find({ root: root.path, asked: otherImage })).toEqual([]);
          },
        }),
        it({
          name: 'refuses a record made for a table with a control removed, added or edited',
          fn: async () => {
            await using root = await evidenceRoot();
            const [first, second] = controls;
            await leaveRecord({
              evidence: await campaignDirectory({ root: root.path, name: `${prefix}one` }),
              table: [first as PlantedControl],
            });
            expect(await find({ root: root.path })).toEqual([]);
            expect((await find({
              root: root.path,
              table: [first as PlantedControl],
            })).length).toBe(1);
            expect(await find({
              root: root.path,
              table: [first as PlantedControl, { ...second, from: 'if edited {' } as PlantedControl],
            })).toEqual([]);
          },
        }),
        it({
          name: 'refuses a record whose digest matches but whose listed names do not',
          fn: async () => {
            await using root = await evidenceRoot();
            await leaveRecord({
              evidence: await campaignDirectory({ root: root.path, name: `${prefix}one` }),
              overrides: { controls: ['first guard'] },
            });
            expect(await find({ root: root.path })).toEqual([]);
          },
        }),
        it({
          name: 'refuses a record without the passed flag, with unreadable limits, torn, or not an object',
          fn: async () => {
            await using root = await evidenceRoot();
            await leaveRecord({
              evidence: await campaignDirectory({ root: root.path, name: `${prefix}a-unflagged` }),
              overrides: { plantedControls: false },
            });
            await leaveRecord({
              evidence: await campaignDirectory({ root: root.path, name: `${prefix}b-limits` }),
              overrides: { limits: ['--init'] },
            });
            await writeFile(
              join(await campaignDirectory({ root: root.path, name: `${prefix}c-torn` }), plantedControlsRecordName),
              '{"plantedControls": tr',
            );
            await writeFile(
              join(await campaignDirectory({ root: root.path, name: `${prefix}d-array` }), plantedControlsRecordName),
              '[]',
            );
            expect(await find({ root: root.path })).toEqual([]);
          },
        }),
        it({
          name: 'ignores a record outside the campaigns directories, such as a gate directory',
          fn: async () => {
            await using root = await evidenceRoot();
            await leaveRecord({ evidence: await campaignDirectory({ root: root.path, name: 'native-gate' }) });
            expect(await find({ root: root.path })).toEqual([]);
          },
        }),
        it({
          name: 'finds the passing record among campaigns without one',
          fn: async () => {
            await using root = await evidenceRoot();
            await campaignDirectory({ root: root.path, name: `${prefix}a-empty` });
            const evidence = await campaignDirectory({ root: root.path, name: `${prefix}b-passed` });
            await leaveRecord({ evidence });
            expect(await find({ root: root.path }))
              .toEqual([join(evidence, plantedControlsRecordName)]);
          },
        }),
        it({
          name: 'finds nothing when the evidence root does not exist',
          fn: async () => {
            await using root = await evidenceRoot();
            expect(await find({ root: join(root.path, 'missing') })).toEqual([]);
          },
        }),
      ],
    }),
    describe({
      name: ensurePlantedControls.name,
      children: [
        it({
          name: 'runs the controls and records the pass when nothing proves them',
          fn: async () => {
            await using root = await evidenceRoot();
            const evidence = await campaignDirectory({ root: root.path, name: `${prefix}one` });
            expect(await decide({ root: root.path, evidence })).toBe(1);
            expect(await find({ root: root.path }))
              .toEqual([join(evidence, plantedControlsRecordName)]);
          },
        }),
        it({
          name: 'lets a later campaign over the same image skip the controls, and one with less memory run them',
          fn: async () => {
            await using root = await evidenceRoot();
            await decide({
              root: root.path,
              evidence: await campaignDirectory({ root: root.path, name: `${prefix}one` }),
            });
            expect(await decide({
              root: root.path,
              evidence: await campaignDirectory({ root: root.path, name: `${prefix}two` }),
            })).toBe(0);
            expect(await decide({
              root: root.path,
              evidence: await campaignDirectory({ root: root.path, name: `${prefix}three` }),
              limits: containerArguments.map(function lessMemory(argument) {
                return argument === '--memory=2g' ? '--memory=1g' : argument;
              }),
            })).toBe(1);
          },
        }),
        it({
          name: 'writes no record when a control was not noticed, so the failure is not skipped later',
          fn: async () => {
            await using root = await evidenceRoot();
            const evidence = await campaignDirectory({ root: root.path, name: `${prefix}one` });
            await expect(decide({
              root: root.path,
              evidence,
              run: async function unnoticed() {
                throw new Error('Planted guard removals were not noticed');
              },
            })).rejects.toThrow('not noticed');
            expect(await readdir(evidence)).toEqual([]);
            expect(await decide({
              root: root.path,
              evidence: await campaignDirectory({ root: root.path, name: `${prefix}two` }),
            })).toBe(1);
          },
        }),
        it({
          name: 'skips for a resumed campaign and for NATIVE_MUTATION_SKIP_PLANTED=1 without a record, and writes none',
          fn: async () => {
            await using root = await evidenceRoot();
            const evidence = await campaignDirectory({ root: root.path, name: `${prefix}one` });
            expect(await decide({ root: root.path, evidence, scoredMutants: 3 })).toBe(0);
            expect(await decide({ root: root.path, evidence, skipRequested: true })).toBe(0);
            expect(await readdir(evidence)).toEqual([]);
          },
        }),
        it({
          name: 'runs the controls but writes no record when the container arguments hold no readable limits',
          fn: async () => {
            await using root = await evidenceRoot();
            const evidence = await campaignDirectory({ root: root.path, name: `${prefix}one` });
            expect(await decide({ root: root.path, evidence, limits: ['--init'] })).toBe(1);
            expect(await readdir(evidence)).toEqual([]);
          },
        }),
        it({
          name: 'keeps the pass when the record cannot be written',
          fn: async () => {
            await using root = await evidenceRoot();
            expect(await decide({
              root: root.path,
              evidence: join(root.path, 'missing'),
            })).toBe(1);
          },
        }),
        it({
          name: 'lets two campaigns started together over one image both run the controls, neither waiting for the other',
          fn: async () => {
            await using root = await evidenceRoot();
            const bothStarted = Promise.withResolvers<void>();
            let started = 0;
            /**
             Counts a run in and finishes only once both campaigns are inside it, so a campaign that waited for the other
             would never get here.
             */
            async function meeting(): Promise<void> {
              started += 1;
              if (started === 2)
                bothStarted.resolve();
              await using bound = cancellableBound();
              await Promise.race([
                bothStarted.promise,
                failAfterWaiting(bound.signal),
              ]);
            }
            const one = await campaignDirectory({ root: root.path, name: `${prefix}one` });
            const two = await campaignDirectory({ root: root.path, name: `${prefix}two` });
            await Promise.all([
              decide({ root: root.path, evidence: one, run: meeting }),
              decide({ root: root.path, evidence: two, run: meeting }),
            ]);
            expect(started).toBe(2);
            // Each campaign recorded its own pass; the stand-in run above is not the real controls, so only the files count.
            expect(await readdir(one)).toEqual([plantedControlsRecordName]);
            expect(await readdir(two)).toEqual([plantedControlsRecordName]);
          },
        }),
        it({
          name: 'proves the real control table the way it proves a small one',
          fn: async () => {
            await using root = await evidenceRoot();
            await leaveRecord({
              evidence: await campaignDirectory({ root: root.path, name: `${prefix}one` }),
              table: plantedControls,
            });
            expect((await find({ root: root.path, table: plantedControls })).length).toBe(1);
            expect(await find({ root: root.path })).toEqual([]);
          },
        }),
      ],
    }),
  ],
});
