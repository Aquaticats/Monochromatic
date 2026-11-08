import { execFileSync, } from 'node:child_process';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CFS_PERIOD_MICROSECONDS,
  containerLimitArguments,
  DISK_WEIGHT_OFF,
  DISK_WEIGHT_SWITCH,
  diskWeightArguments,
  imageBuildLimitArguments,
  LOWEST_DISK_WEIGHT,
} from './index.ts';

/**
 Limits that differ in every field from any default, so a helper that ignored one would show it.
 */
const fixtureLimits = {
  memory: '512m',
  cpus: 1.5,
  pidsLimit: 64,
};

await describe({
  name: 'the shared container limits',
  children: [
    it({
      name: 'keeps the lowest weight Podman accepts for a started container',
      fn: async () => {
        expect(LOWEST_DISK_WEIGHT,).toBe(10,);
      },
    },),
    describe({
      name: diskWeightArguments.name,
      children: [
        it({
          name: 'sets both block-layer weight files from the one named weight',
          fn: async () => {
            expect(diskWeightArguments(),).toEqual([
              `--blkio-weight=${String(LOWEST_DISK_WEIGHT,)}`,
              `--cgroup-conf=io.weight=${String(LOWEST_DISK_WEIGHT,)}`,
            ],);
          },
        },),
        it({
          name: 'returns the same arguments on every call',
          fn: async () => {
            expect(diskWeightArguments(),).toEqual(diskWeightArguments(),);
          },
        },),
        it({
          name: 'returns no argument when the switch says off, for a runner that cannot set the weight',
          fn: async () => {
            expect(diskWeightArguments({ environment: { [DISK_WEIGHT_SWITCH]: DISK_WEIGHT_OFF, }, },),).toEqual([],);
          },
        },),
        it({
          name: 'keeps the weight for any other value of the switch, an empty one included',
          fn: async () => {
            for (const value of ['', 'on', 'OFF', '0',]) {
              expect(diskWeightArguments({ environment: { [DISK_WEIGHT_SWITCH]: value, }, },),).toEqual(diskWeightArguments({ environment: {}, },),);
            }
          },
        },),
        it({
          name: 'keeps the weight when the switch is absent',
          fn: async () => {
            expect(diskWeightArguments({ environment: {}, },),).toEqual([
              `--blkio-weight=${String(LOWEST_DISK_WEIGHT,)}`,
              `--cgroup-conf=io.weight=${String(LOWEST_DISK_WEIGHT,)}`,
            ],);
          },
        },),
      ],
    },),
    describe({
      name: containerLimitArguments.name,
      children: [
        it({
          name: 'carries memory, CPUs and processes and no weight option in a process whose switch says off',
          fn: async () => {
            /**
             The arguments a child process builds with the switch set.
             */
            const printed = execFileSync(
              process.execPath,
              [
                '--input-type=module-typescript',
                '--eval',
                `const { containerLimitArguments } = await import(${JSON.stringify(join(import.meta.dirname, 'index.ts',),)}); console.log(JSON.stringify(containerLimitArguments(${JSON.stringify(fixtureLimits,)})))`,
              ],
              {
                encoding: 'utf8',
                env: {
                  ...process.env,
                  [DISK_WEIGHT_SWITCH]: DISK_WEIGHT_OFF,
                },
              },
            );
            expect(JSON.parse(printed,),).toEqual([
              '--memory=512m',
              '--cpus=1.5',
              '--pids-limit=64',
            ],);
          },
        },),
        it({
          name: 'carries memory, CPUs, processes and both weight options',
          fn: async () => {
            expect(containerLimitArguments(fixtureLimits,),).toEqual([
              '--memory=512m',
              '--cpus=1.5',
              '--pids-limit=64',
              ...diskWeightArguments(),
            ],);
          },
        },),
      ],
    },),
    describe({
      name: imageBuildLimitArguments.name,
      children: [
        it({
          name: 'bounds memory and the CFS quota, and carries no option `podman build` lacks',
          fn: async () => {
            expect(imageBuildLimitArguments(fixtureLimits,),).toEqual([
              '--memory=512m',
              `--cpu-period=${String(CFS_PERIOD_MICROSECONDS,)}`,
              `--cpu-quota=${String(Math.round(1.5 * CFS_PERIOD_MICROSECONDS,),)}`,
            ],);
          },
        },),
        it({
          name: 'rounds a quota that floating point would turn into a fraction',
          fn: async () => {
            /**
             The quota option of a tenth of a CPU, which `0.1 * 100000` spells as `10000.000000000002`.
             */
            const quota = imageBuildLimitArguments({
              ...fixtureLimits,
              cpus: 0.1,
            },).find((argument,) => argument.startsWith('--cpu-quota=',),);
            expect(quota,).toBe(`--cpu-quota=${String(CFS_PERIOD_MICROSECONDS / 10,)}`,);
          },
        },),
      ],
    },),
  ],
},);
