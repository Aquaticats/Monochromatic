import {
  mkdir,
  mkdtempDisposable,
  readdir,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ensurePlantedControls } from './native-planted-controls-ensure.ts';
import {
  controlsKeyOf,
  plantedControlsRecordName,
} from './native-planted-controls-record.ts';
import type { PlantedUnit } from './native-planted-controls-units.ts';
import type { UnitState } from './native-planted-controls-unit.ts';
import type {
  PlantedControl,
  PlantedControlSetting,
} from './native-planted-controls.ts';

/**
 Image ID of the image under test.
 */
const image = 'a'.repeat(64);

/**
 Name prefix of campaign evidence directories, as the campaign uses it.
 */
const prefix = 'native-mutation-';

/**
 Container arguments a campaign hands to the planted controls.
 */
const containerArguments = [
  '--init',
  '--network=none',
  '--memory=2g',
  '--cpus=2',
  '--pids-limit=512',
];

/**
 A table of one small control.
 */
const controls: readonly PlantedControl[] = [
  {
    name: 'first guard',
    file: 'src/native/a.rs',
    from: 'if guard {',
    to: 'if false {',
    failing: 'a::tests::first',
  },
];

/**
 The controls unit a launcher started beside the campaign.
 */
const beside: PlantedUnit = {
  image,
  unit: 'unit-x',
  log: '/logs/unit-x.log.1',
  invocation: 'invocation-1',
};

/**
 The controls unit as systemd reports it while it runs.
 */
const runningUnit: UnitState = {
  state: 'active',
  invocation: 'invocation-1',
};

/**
 The controls unit as systemd reports it after it ended and was unloaded.
 */
const goneUnit: UnitState = {
  state: '',
  invocation: '',
};

/**
 Create an evidence root with one campaign directory.

 @returns the disposable root and the campaign directory; declare the root with `await using`

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
    'planted-ensure-',
  ));
  await mkdir(join(
    root.path,
    `${prefix}one`,
  ));
  return root;
}

/**
 The fields of the call a test may replace.
 */
type Overrides = {
  readonly setting?: PlantedControlSetting;
  readonly skipRequested?: boolean;
  readonly beside?: PlantedUnit;
  readonly inspect?: (unit: string) => Promise<UnitState>;
  readonly requirePublished?: boolean;
  readonly run?: (setting: PlantedControlSetting) => Promise<void>;
};

/**
 Run the decision with a counting stand-in for the controls.

 @param root - evidence root

 @param overrides - fields of the call that replace the defaults

 @returns the decision, how often the controls ran, and the printed lines

 @example
 ```ts
 await decide({ root: root.path, overrides: {} });
 ```
 */
async function decide({
  root,
  overrides = {},
}: {
  readonly root: string;
  readonly overrides?: Overrides;
}) {
  const evidence = join(
    root,
    `${prefix}one`,
  );
  const lines: string[] = [];
  const counter = { ran: 0, };
  const setting: PlantedControlSetting = {
    base: image,
    limits: containerArguments,
    context: evidence,
    evidence,
  };
  const decision = await ensurePlantedControls({
    setting,
    evidenceRoot: root,
    evidencePrefix: prefix,
    skipRequested: false,
    controls,
    harness: 'harness-a',
    log: function collect(message: string): void {
      lines.push(message);
    },
    run: async function countedRun() {
      counter.ran += 1;
    },
    ...overrides,
  });
  return {
    decision,
    ran: () => counter.ran,
    lines,
    evidence,
  };
}

await describe({
  name: 'the decision about the planted controls',
  children: [
    describe({
      name: ensurePlantedControls.name,
      children: [
        it({
          name: 'defers to a controls unit that still runs, carries the whole obligation in the decision, writes nothing and runs nothing',
          fn: async () => {
            await using root = await evidenceRoot();
            const { decision, ran, lines, evidence } = await decide({
              root: root.path,
              overrides: { beside, inspect: async () => runningUnit },
            });
            expect(decision).toEqual({
              kind: 'deferred',
              obligation: {
                image,
                unit: 'unit-x',
                log: '/logs/unit-x.log.1',
                invocation: 'invocation-1',
                limits: { memory: '2g', pidsLimit: 512 },
                controls: controlsKeyOf(controls),
                harness: 'harness-a',
              },
            });
            expect(ran()).toBe(0);
            expect(await readdir(evidence)).toEqual([]);
            expect(lines.join('\n').includes('without proof')).toBe(true);
            expect(lines.join('\n').includes('/logs/unit-x.log.1')).toBe(true);
            expect(lines.join('\n').includes('native:mutation:reconcile')).toBe(true);
          },
        }),
        it({
          name: 'does not wait for a controls unit that is gone without a record: it runs the controls itself and records them',
          fn: async () => {
            await using root = await evidenceRoot();
            const { decision, ran, lines, evidence } = await decide({
              root: root.path,
              overrides: { beside, inspect: async () => goneUnit },
            });
            expect(ran()).toBe(1);
            expect(decision).toEqual({ kind: 'ran', record: join(evidence, plantedControlsRecordName) });
            expect(lines.join('\n').includes('no longer running')).toBe(true);
            expect(await readdir(evidence)).toEqual([plantedControlsRecordName]);
          },
        }),
        it({
          name: 'reports a proving record as proved, ahead of a controls unit',
          fn: async () => {
            await using root = await evidenceRoot();
            const first = await decide({ root: root.path });
            const second = await decide({
              root: root.path,
              overrides: { beside, inspect: async () => runningUnit },
            });
            expect(first.decision.kind).toBe('ran');
            expect(second.decision).toEqual({ kind: 'proved', record: join(first.evidence, plantedControlsRecordName) });
            expect(second.ran()).toBe(0);
          },
        }),
        it({
          name: 'does not defer to a unit that reuses the name under another invocation than the launcher started: it runs the controls itself',
          fn: async () => {
            await using root = await evidenceRoot();
            const { decision, ran, lines } = await decide({
              root: root.path,
              overrides: {
                beside,
                inspect: async () => ({ state: 'active', invocation: 'invocation-7' }),
              },
            });
            expect(decision.kind).toBe('ran');
            expect(ran()).toBe(1);
            expect(lines.join('\n').includes('no longer running')).toBe(true);
          },
        }),
        it({
          name: 'reports the skip a person asked for',
          fn: async () => {
            await using root = await evidenceRoot();
            const { decision, ran } = await decide({ root: root.path, overrides: { skipRequested: true } });
            expect(decision).toEqual({ kind: 'requested' });
            expect(ran()).toBe(0);
          },
        }),
        it({
          name: 'in producer mode throws when the record cannot be written, and keeps a campaign\'s write non-fatal',
          fn: async () => {
            await using root = await evidenceRoot();
            const missing = join(root.path, 'missing');
            const unwritable = {
              base: image,
              limits: containerArguments,
              context: missing,
              evidence: missing,
            };
            await expect(decide({
              root: root.path,
              overrides: { setting: unwritable, requirePublished: true },
            })).rejects.toThrow('record could not be written');
            const lenient = await decide({ root: root.path, overrides: { setting: unwritable } });
            expect(lenient.decision).toEqual({ kind: 'ran' });
          },
        }),
        it({
          name: 'requirePublished fails the decision loudly, naming the reason, after the controls ran locally but the record could not be written',
          fn: async () => {
            await using root = await evidenceRoot();
            const blocked = join(root.path, `${prefix}one`);
            // The record's path is a directory, so the write fails while the evidence directory itself is fine.
            await mkdir(join(blocked, plantedControlsRecordName));
            const counter = { ran: 0, };
            await expect(decide({
              root: root.path,
              overrides: {
                requirePublished: true,
                run: async function countedRun() {
                  counter.ran += 1;
                },
              },
            })).rejects.toThrow('record could not be written');
            expect(counter.ran).toBe(1);
          },
        }),
        it({
          name: 'in producer mode throws when the container arguments hold no readable limits',
          fn: async () => {
            await using root = await evidenceRoot();
            const evidence = join(root.path, `${prefix}one`);
            const unreadable = { base: image, limits: ['--init'], context: evidence, evidence };
            await expect(decide({
              root: root.path,
              overrides: { setting: unreadable, requirePublished: true },
            })).rejects.toThrow('no readable --memory');
          },
        }),
      ],
    }),
  ],
});
