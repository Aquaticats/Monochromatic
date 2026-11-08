import {
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  inspectionEnvironment,
  type LauncherOptions,
  parseLauncherOptions,
  planUnits,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';
import { unitIoWeight } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/constants.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  defaultContainerLimits,
  runLimitArguments,
} from './container-limits.ts';

import { nativeTestImageRepository } from './native-gate-image-tag.ts';
import {
  type ImageInspector,
  imageOptionOf,
  planControlsUnit,
  plantedTask,
  skipPlantedEntry,
  startPlantedControls,
  type UnitStarter,
} from './native-planted-parallel.ts';
import { ensurePlantedControls } from './native-planted-controls-ensure.ts';
import {
  plantedControlsPassedRecord,
  plantedControlsRecordName,
} from './native-planted-controls-record.ts';
import {
  plantedUnitFor,
  plantedUnitsEnvironmentVariable,
  plantedUnitsText,
} from './native-planted-controls-units.ts';
import { plantedHarnessId } from './native-planted-controls-harness.ts';
import type { UnitState } from './native-planted-controls-unit.ts';
import { plantedControls } from './native-planted-controls.ts';

/**
 Token of the launch the tests simulate, which makes the controls unit names unique to it.
 */
const launchToken = 'a1b2c3d4';

/**
 The controls unit as systemd reports it right after the launcher started it.
 */
const startedUnit: UnitState = {
  state: 'active',
  invocation: 'invocation-started',
};

/**
 The controls unit when it ended at once and was unloaded.
 */
const goneUnit: UnitState = {
  state: '',
  invocation: '',
};

/**
 Image ID the first tag names.
 */
const imageA = 'a'.repeat(64);

/**
 Image ID some other tag names.
 */
const imageB = 'b'.repeat(64);

/**
 Version of the host-side runner and judge that made the records of these tests.
 */
const harness = plantedHarnessId;

/**
 Container arguments a campaign at the defaults hands to the planted controls.
 */
const containerArguments = [
  '--init',
  '--network=none',
  ...runLimitArguments({
    limits: defaultContainerLimits,
    withPidsLimit: true,
  }),
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
 Create the working tree a launch starts its units in.

 @returns the disposable root; declare it with `await using`

 @example
 ```ts
 await using root = await workingTree();
 ```
 */
async function workingTree(): Promise<AsyncDisposable & { readonly path: string }> {
  await mkdir(
    scratchParent,
    { recursive: true },
  );
  return await mkdtempDisposable(join(
    scratchParent,
    'planted-parallel-',
  ));
}

/**
 Parse launcher options through the real parser.

 @param root - the working tree

 @param argv - arguments after the required ones

 @returns the options

 @example
 ```ts
 launch({ root: root.path, argv: ['w5', 'a.rs', 'b.rs'] });
 ```
 */
function launch({
  root,
  argv,
}: {
  readonly root: string;
  readonly argv: readonly string[];
}): LauncherOptions {
  return parseLauncherOptions({
    argv: [
      '--task',
      '//package/git-policy/cli:native:mutation:scoped',
      '--tag-env',
      'GIT_POLICY_NATIVE_IMAGE_TAG',
      '--image-repository',
      nativeTestImageRepository,
      '--unit-prefix',
      'cli-git-mutpar',
      '--root',
      root,
      ...argv,
    ],
    environment: { PATH: '/usr/bin:/bin' },
    home: '/h',
    packageDirectory: '/repo/package/mutation-test/cli-cargo-mutants',
  });
}

/**
 Answer image inspections from a table.

 @param images - tag to image ID; a tag not in the table fails to resolve

 @returns the inspector, with the tags it was asked about in `asked`

 @example
 ```ts
 inspecting({ w50: imageA });
 ```
 */
function inspecting(images: Readonly<Record<string, string>>): ImageInspector & { readonly asked: string[] } {
  const asked: string[] = [];
  async function inspect(tag: string): ReturnType<ImageInspector> {
    asked.push(tag);
    const image = images[tag];
    return image === undefined ? { failure: `no such image ${tag}` } : { image };
  }
  return Object.assign(
    inspect,
    { asked },
  );
}

/**
 Record every unit start and answer as told.

 @param started - whether `systemd-run` accepts a unit

 @returns the starter, with the calls in `calls`

 @example
 ```ts
 starting(true);
 ```
 */
function starting(started: boolean): UnitStarter & { readonly calls: { unit: string; systemdRunArguments: readonly string[] }[] } {
  const calls: { unit: string; systemdRunArguments: readonly string[] }[] = [];
  async function start(call: Parameters<UnitStarter>[0]): ReturnType<UnitStarter> {
    calls.push({ ...call });
    return {
      started,
      line: started ? `${call.unit}: started` : `${call.unit}: Failed to start transient service unit`,
    };
  }
  return Object.assign(
    start,
    { calls },
  );
}

/**
 Leave a record that proves the controls passed on an image, the way a controls unit does.

 @param root - the working tree

 @param image - the image the record names

 @param limits - container arguments the controls ran with

 @returns the record's path

 @example
 ```ts
 await leaveRecord({ root: root.path, image: imageA });
 ```
 */
async function leaveRecord({
  root,
  image,
  limits = containerArguments,
}: {
  readonly root: string;
  readonly image: string;
  readonly limits?: readonly string[];
}): Promise<string> {
  const directory = join(
    root,
    'package',
    'git-policy',
    'cli',
    'target',
    'verification',
    'native-mutation-planted-abc123',
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
    JSON.stringify(plantedControlsPassedRecord({
      image,
      limits,
      controls: plantedControls,
      harness,
    })),
  );
  return file;
}

/**
 Run the launch decision.

 @param root - the working tree

 @param argv - launcher arguments

 @param images - tag to image ID

 @param accept - whether `systemd-run` accepts the controls unit

 @returns what was decided, the starts and the inspections

 @example
 ```ts
 await decide({ root: root.path, argv: ['w5', 'a.rs'], images: { w50: imageA }, accept: true });
 ```
 */
async function decide({
  root,
  argv,
  images,
  accept,
  alive = true,
  token = true,
}: {
  readonly root: string;
  readonly argv: readonly string[];
  readonly images: Readonly<Record<string, string>>;
  readonly accept: boolean;
  readonly alive?: boolean;
  readonly token?: boolean;
}) {
  const options = launch({
    root,
    argv,
  });
  const start = starting(accept);
  const inspect = inspecting(images);
  const launched = await startPlantedControls({
    options,
    plans: planUnits(options),
    inspectImage: inspect,
    start,
    inspectUnit: async function inspectStarted() {
      return alive ? startedUnit : goneUnit;
    },
    ...(token ? { launchToken } : {}),
  });
  return {
    launched,
    start,
    inspect,
    options,
  };
}

/**
 Text that contains every one of the parts, for a log line.

 @param text - the log text

 @param parts - parts it must contain

 @returns true when each part occurs

 @example
 ```ts
 containsAll('a b', ['a', 'b']);
 ```
 */
function containsAll({
  text,
  parts,
}: {
  readonly text: string;
  readonly parts: readonly string[];
}): boolean {
  return parts.every(function occurs(part) {
    return text.includes(part);
  });
}

/**
 Collect the lines the decision prints.

 @param lines - receives each line

 @returns the printer to pass as `log`

 @example
 ```ts
 const lines: string[] = [];
 collectInto(lines);
 ```
 */
function collectInto(lines: string[]): (message: string) => void {
  return function collect(message: string): void {
    lines.push(message);
  };
}

await describe({
  name: 'the planted controls of a parallel launch',
  concurrency: 1,
  children: [
    describe({
      name: startPlantedControls.name,
      children: [
        it({
          name: 'starts no controls unit when a record proves the image, and says which record',
          fn: async () => {
            await using root = await workingTree();
            const record = await leaveRecord({ root: root.path, image: imageA });
            const { launched, start } = await decide({
              root: root.path,
              argv: ['w5', 'a.rs', 'b.rs'],
              images: { w50: imageA, w51: imageA },
              accept: true,
            });
            expect(start.calls).toEqual([]);
            expect(launched.units).toEqual([]);
            expect(launched.environment).toEqual([]);
            expect(launched.lines.length).toBe(1);
            expect(containsAll({
              text: launched.lines.join('\n'),
              parts: [imageA, `proved by ${record}`, 'no controls unit started'],
            })).toBe(true);
          },
        }),
        it({
          name: 'starts one controls unit beside the campaign units when no record proves the image, and names it',
          fn: async () => {
            await using root = await workingTree();
            const { launched, start, options } = await decide({
              root: root.path,
              argv: ['--memory', '3g', '--cpus', '4', 'w5', 'a.rs', 'b.rs'],
              images: { w50: imageA, w51: imageA },
              accept: true,
            });
            const unit = `cli-git-mutpar-w5-planted-${imageA.slice(0, 12)}-${launchToken}`;
            expect(start.calls.map((call) => call.unit)).toEqual([unit]);
            expect(launched.units).toEqual([unit]);
            expect(launched.environment).toEqual([`${plantedUnitsEnvironmentVariable}=${plantedUnitsText([{ image: imageA, unit, log: `${options.logDirectory}/${options.logPrefix}-w5-planted-${imageA.slice(0, 12)}-${launchToken}.log.1`, invocation: 'invocation-started' }])}`]);
            expect(launched.lines.length).toBe(1);
            expect(containsAll({
              text: launched.lines.join('\n'),
              parts: [
                imageA,
                `unit ${unit} (invocation invocation-started) runs them beside the campaign units`,
                `${options.logDirectory}/${options.logPrefix}-w5-planted-${imageA.slice(0, 12)}-${launchToken}.log.1`,
                'skip them without proof',
                'hold only once that unit ends with a planted-controls-passed.json',
                'the control that was not noticed',
              ],
            })).toBe(true);
            const [call] = start.calls;
            const body = call?.systemdRunArguments.at(-1) ?? '';
            expect(containsAll({
              text: body,
              parts: [plantedTask, '--memory', '3g', '--cpus', '4', 'gate-slot.ts'],
            })).toBe(true);
            expect(call?.systemdRunArguments).toContain('--setenv=GIT_POLICY_NATIVE_IMAGE_TAG=w50');
          },
        }),
        it({
          name: 'does not wait: the decision returns while the controls unit has only been started',
          fn: async () => {
            await using root = await workingTree();
            const { start, launched } = await decide({
              root: root.path,
              argv: ['w5', 'a.rs'],
              images: { w50: imageA },
              accept: true,
            });
            // The starter answers at once, so a launcher that waited for a record would not return a result.
            expect(start.calls.length).toBe(1);
            expect(launched.units.length).toBe(1);
          },
        }),
        it({
          name: 'leaves the campaign units without a skip, and says so, when the controls unit does not start',
          fn: async () => {
            await using root = await workingTree();
            const { launched, start } = await decide({
              root: root.path,
              argv: ['w5', 'a.rs'],
              images: { w50: imageA },
              accept: false,
            });
            expect(start.calls.length).toBe(1);
            expect(launched.units).toEqual([]);
            expect(launched.environment).toEqual([]);
            expect(containsAll({
              text: launched.lines.join('\n'),
              parts: [imageA, 'did not start', 'Failed to start transient service unit', 'plant the controls themselves'],
            })).toBe(true);
          },
        }),
        it({
          name: 'gives the controls unit a name unique to the launch, so another launch\'s unit of the same image is never taken for it',
          fn: async () => {
            await using root = await workingTree();
            const options = launch({ root: root.path, argv: ['w5', 'a.rs'] });
            const names = ['aaaa1111', 'bbbb2222'].map(function nameFor(token) {
              return planControlsUnit({ options, image: imageA, tag: 'w50', taken: [], launchToken: token }).unit;
            });
            expect(new Set(names).size).toBe(2);
            expect(names.every((name) => name.includes(imageA.slice(0, 12)))).toBe(true);
            // Without a given token, each launch draws its own.
            const first = await decide({ root: root.path, argv: ['w5', 'a.rs'], images: { w50: imageA }, accept: true, token: false });
            const second = await decide({ root: root.path, argv: ['w5', 'a.rs'], images: { w50: imageA }, accept: true, token: false });
            expect(first.launched.units[0]).not.toBe(second.launched.units[0]);
            // 128 bits: the token is the last segment, 32 hexadecimal digits, so a repeat is not a realistic event.
            expect(first.launched.units[0]?.split('-').at(-1)?.length).toBe(32);
          },
        }),
        it({
          name: 'does not offer the campaign units a controls unit that ended at once, and says so with the log',
          fn: async () => {
            await using root = await workingTree();
            const { launched, start } = await decide({
              root: root.path,
              argv: ['w5', 'a.rs'],
              images: { w50: imageA },
              accept: true,
              alive: false,
            });
            expect(start.calls.length).toBe(1);
            expect(launched.units).toEqual([]);
            expect(launched.environment).toEqual([]);
            expect(containsAll({
              text: launched.lines.join('\n'),
              parts: [imageA, 'already gone', 'plant the controls themselves', '.log.1'],
            })).toBe(true);
          },
        }),
        it({
          name: 'starts one controls unit per distinct image and names each image in the campaign environment',
          fn: async () => {
            await using root = await workingTree();
            await leaveRecord({ root: root.path, image: imageA });
            const { launched, start } = await decide({
              root: root.path,
              argv: ['w5', 'a.rs', 'b.rs', 'c.rs'],
              images: { w50: imageA, w51: imageB, w52: imageB },
              accept: true,
            });
            const unit = `cli-git-mutpar-w5-planted-${imageB.slice(0, 12)}-${launchToken}`;
            expect(start.calls.map((call) => call.unit)).toEqual([unit]);
            expect(launched.environment[0]?.startsWith(`${plantedUnitsEnvironmentVariable}=`)).toBe(true);
            expect(plantedUnitFor({ image: imageB, text: launched.environment[0]?.slice(plantedUnitsEnvironmentVariable.length + 1) ?? '' })[0]?.unit).toBe(unit);
            expect(launched.lines.length).toBe(2);
          },
        }),
        it({
          name: 'treats a record made at larger limits than the campaign as no proof',
          fn: async () => {
            await using root = await workingTree();
            await leaveRecord({
              root: root.path,
              image: imageA,
              limits: containerArguments.map((argument) => (argument === '--memory=2g' ? '--memory=4g' : argument)),
            });
            const { start } = await decide({
              root: root.path,
              argv: ['w5', 'a.rs'],
              images: { w50: imageA },
              accept: true,
            });
            expect(start.calls.length).toBe(1);
          },
        }),
        it({
          name: 'leaves a unit whose image did not resolve to plant the controls itself, and says which tag',
          fn: async () => {
            await using root = await workingTree();
            const { launched, start } = await decide({
              root: root.path,
              argv: ['w5', 'a.rs', 'b.rs'],
              images: { w50: imageA },
              accept: true,
            });
            expect(start.calls.length).toBe(1);
            expect(containsAll({
              text: launched.lines.join('\n'),
              parts: ['image of tag w51 did not resolve', 'no such image w51', 'plants the controls itself'],
            })).toBe(true);
          },
        }),
        it({
          name: 'starts nothing and says so when the person passed the skip entry',
          fn: async () => {
            await using root = await workingTree();
            const { launched, start, inspect } = await decide({
              root: root.path,
              argv: ['--setenv', skipPlantedEntry, 'w5', 'a.rs'],
              images: { w50: imageA },
              accept: true,
            });
            expect(start.calls).toEqual([]);
            expect(inspect.asked).toEqual([]);
            expect(launched.environment).toEqual([]);
            expect(containsAll({
              text: launched.lines.join('\n'),
              parts: [skipPlantedEntry, 'without proof'],
            })).toBe(true);
          },
        }),
      ],
    }),
    describe({
      name: planControlsUnit.name,
      children: [
        it({
          name: 'refuses a controls unit name that a campaign unit already has',
          fn: async () => {
            await using root = await workingTree();
            const options = launch({ root: root.path, argv: ['w5', 'a.rs'] });
            expect(() => planControlsUnit({
              options,
              image: imageA,
              tag: 'w50',
              taken: [`cli-git-mutpar-w5-planted-${imageA.slice(0, 12)}-${launchToken}`],
              launchToken,
            })).toThrow('equals a campaign unit');
          },
        }),
      ],
    }),
    describe({
      name: `${planControlsUnit.name} command and environment`,
      children: [
        it({
          name: 'runs the inspected image ID, never the tag, and passes the launcher\'s extra environment but not the units\' coordination entry',
          fn: async () => {
            await using root = await workingTree();
            const options = launch({
              root: root.path,
              argv: ['--setenv', 'CONTAINERS_STORAGE_CONF=/s.conf', 'w5', 'a.rs'],
            });
            const { systemdRunArguments } = planControlsUnit({ options, image: imageA, tag: 'w50', taken: [], launchToken });
            expect(systemdRunArguments.at(-1)?.includes(`'--image' '${imageA}'`)).toBe(true);
            expect(systemdRunArguments).toContain('--setenv=CONTAINERS_STORAGE_CONF=/s.conf');
            expect(systemdRunArguments).toContain('--setenv=GIT_POLICY_NATIVE_IMAGE_TAG=w50');
            expect(systemdRunArguments.some((argument) => argument.includes(plantedUnitsEnvironmentVariable))).toBe(false);
          },
        }),
        it({
          name: 'caps the controls unit with the launcher\'s task limit and the unit disk weight',
          fn: async () => {
            await using root = await workingTree();
            const options = launch({ root: root.path, argv: ['--unit-tasks-max', '96', 'w5', 'a.rs'] });
            const { systemdRunArguments } = planControlsUnit({ options, image: imageA, tag: 'w50', taken: [], launchToken });
            expect(systemdRunArguments).toContain('--property=TasksMax=96');
            expect(systemdRunArguments).toContain(`--property=IOWeight=${String(unitIoWeight)}`);
          },
        }),
      ],
    }),
    describe({
      name: 'the environment the launcher inspects with',
      children: [
        it({
          name: 'is the unit\'s own assignments, final TMPDIR, PATH and HOME included, with nothing else inherited but the runtime directory',
          fn: async () => {
            await using root = await workingTree();
            const options = launch({
              root: root.path,
              argv: ['--setenv', 'CONTAINERS_STORAGE_CONF=/alternate/storage.conf', '--setenv', 'HOME=/alternate-home', 'w5', 'a.rs'],
            });
            // The launcher's HOME wins over a --setenv of HOME, as it does for the unit.
            expect(inspectionEnvironment({ options, tag: 'w50', runtimeDirectory: '/run/user/1000' })).toEqual({
              XDG_RUNTIME_DIR: '/run/user/1000',
              GIT_POLICY_NATIVE_IMAGE_TAG: 'w50',
              CONTAINERS_STORAGE_CONF: '/alternate/storage.conf',
              TMPDIR: options.temporaryDirectory,
              PATH: '/usr/bin:/bin',
              HOME: '/h',
            });
            expect(Object.hasOwn(inspectionEnvironment({ options, tag: 'w50' }), 'XDG_RUNTIME_DIR')).toBe(false);
          },
        }),
        it({
          name: 'refuses a --setenv of the image tag variable or the coordination variable, before anything starts',
          fn: async () => {
            await using root = await workingTree();
            await Promise.all(['GIT_POLICY_NATIVE_IMAGE_TAG=other', `${plantedUnitsEnvironmentVariable}=x@y`].map(async function refuse(entry) {
              const options = launch({ root: root.path, argv: ['--setenv', entry, 'w5', 'a.rs'] });
              const start = starting(true);
              await expect(startPlantedControls({
                options,
                plans: planUnits(options),
                inspectImage: inspecting({ w50: imageA }),
                start,
                inspectUnit: async () => startedUnit,
              })).rejects.toThrow('--setenv cannot set');
              expect(start.calls).toEqual([]);
            }));
          },
        }),
      ],
    }),
    describe({
      name: imageOptionOf.name,
      children: [
        it({
          name: 'reads a full image ID, nothing when absent, and refuses a short ID, a missing value or other arguments',
          fn: async () => {
            expect(imageOptionOf(['--image', imageA])).toEqual([imageA]);
            expect(imageOptionOf([])).toEqual([]);
            expect(() => imageOptionOf(['--image', 'abc'])).toThrow('--image');
            expect(() => imageOptionOf(['--image'])).toThrow('--image');
            expect(() => imageOptionOf(['--timeout', '9'])).toThrow('--image');
            expect(() => imageOptionOf(['--image', imageA, '--x'])).toThrow('--image');
          },
        }),
      ],
    }),
    describe({
      name: plantedUnitFor.name,
      children: [
        it({
          name: 'reads back the unit written for an image, and nothing for another image, junk or empty text',
          fn: async () => {
            const text = plantedUnitsText([
              { image: imageA, unit: 'unit-a', log: '/l/a.log.1', invocation: 'i-a' },
              { image: imageB, unit: 'unit-b', log: '/l/b.log.1', invocation: 'i-b' },
            ]);
            expect(plantedUnitFor({ image: imageA, text })).toEqual([{ image: imageA, unit: 'unit-a', log: '/l/a.log.1', invocation: 'i-a' }]);
            expect(plantedUnitFor({ image: imageB, text })[0]?.unit).toBe('unit-b');
            expect(plantedUnitFor({ image: 'c'.repeat(64), text })).toEqual([]);
            expect(plantedUnitFor({ image: imageA, text: '' })).toEqual([]);
            // Junk is set aside, not trusted: not JSON, not a list, and entries missing a field.
            expect(plantedUnitFor({ image: imageA, text: '{torn' })).toEqual([]);
            expect(plantedUnitFor({ image: imageA, text: '{"image":"x"}' })).toEqual([]);
            expect(plantedUnitFor({ image: imageA, text: JSON.stringify([{ image: imageA, unit: 'u', log: 'l' }]) })).toEqual([]);
          },
        }),
      ],
    }),
    describe({
      name: `${ensurePlantedControls.name} beside a controls unit`,
      children: [
        it({
          name: 'skips without proof, runs nothing and writes no record, and the log names the unit and the image',
          fn: async () => {
            await using root = await workingTree();
            const evidence = join(root.path, 'native-mutation-one');
            await mkdir(evidence, { recursive: true });
            const lines: string[] = [];
            let ran = 0;
            await ensurePlantedControls({
              setting: { base: imageA, limits: containerArguments, context: evidence, evidence },
              evidenceRoot: root.path,
              evidencePrefix: 'native-mutation-',
              skipRequested: false,
              beside: { image: imageA, unit: 'cli-git-mutpar-w5-planted-aaaaaaaaaaaa', log: '/l/x.log.1', invocation: 'inv-1' },
              inspect: async () => ({ state: 'active', invocation: 'inv-1' }),
              run: async () => { ran += 1; },
              log: collectInto(lines),
            });
            expect(ran).toBe(0);
            expect(containsAll({
              text: lines.join('\n'),
              parts: ['skipped here, without proof', 'cli-git-mutpar-w5-planted-aaaaaaaaaaaa', imageA, 'hold only once'],
            })).toBe(true);
          },
        }),
        it({
          name: 'prefers a proving record to the controls unit, and says which record',
          fn: async () => {
            await using root = await workingTree();
            const record = await leaveRecord({ root: root.path, image: imageA });
            const lines: string[] = [];
            await ensurePlantedControls({
              setting: { base: imageA, limits: containerArguments, context: root.path, evidence: root.path },
              evidenceRoot: join(root.path, 'package', 'git-policy', 'cli', 'target', 'verification'),
              evidencePrefix: 'native-mutation-',
              skipRequested: false,
              beside: { image: imageA, unit: 'cli-git-mutpar-w5-planted-aaaaaaaaaaaa', log: '/l/x.log.1', invocation: 'inv-1' },
              inspect: async () => ({ state: 'active', invocation: 'inv-1' }),
              run: async () => {},
              log: collectInto(lines),
            });
            expect(containsAll({ text: lines.join('\n'), parts: [record, 'proves they already passed'] })).toBe(true);
            expect(lines.join('\n').includes('without proof')).toBe(false);
          },
        }),
      ],
    }),
  ],
});
