import { mkdir, rm } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  SCOPE_NOT_READ,
  checkScopeCgroupPath,
  readScopeAccounting,
} from './native-check-scope-accounting.ts';
import { createGateScratchDirectory } from './stand-in-gate-programs-for-tests.ts';
import {
  standInCgroupFigures,
  standInMemoryMaxBytes,
  standInMemoryPeakBytes,
  writeStandInCgroup,
} from './stand-in-cgroup-files-for-tests.ts';

await describe({
  name: 'the accounting of one check\'s scope',
  children: [
    it({
      name: 'names the control group a check\'s scope takes, under the slice the scope is started in',
      fn: async () => {
        expect(checkScopeCgroupPath({
          name: 'gpc-check-a-20261127T152200-0500',
          userId: 1_000,
        }),).toBe('/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/gpc-check-a-20261127T152200-0500.scope',);
      },
    }),
    it({
      name: 'reads the peaks, the bounds and the events a check\'s scope counted',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'check-scope-accounting-', });
        /**
         The fixture's control group.
         */
        const cgroupPath = join(
          scratch.path,
          'app.slice',
          'gpc-check-a.scope',
        );
        await writeStandInCgroup({
          cgroupPath,
          figures: standInCgroupFigures,
        });
        /**
         What the reader answered.
         */
        const read = await readScopeAccounting({ cgroupPath, });
        if (read === SCOPE_NOT_READ)
          throw new Error('the reader found no control group where the fixture wrote one');
        expect(read.pidsPeak,).toBe(434,);
        expect(read.pidsMax,).toBe(1_024,);
        expect(read.pidsEvents.max,).toBe(6,);
        expect(read.memoryPeakBytes,).toBe(standInMemoryPeakBytes,);
        expect(read.memoryMaxBytes,).toBe(standInMemoryMaxBytes,);
        expect(read.memoryEvents.oom_kill,).toBe(2,);
        expect(read.cpuQuotaMicroseconds,).toBe(800_000,);
        expect(read.cpuPeriodMicroseconds,).toBe(100_000,);
        expect(read.cpuStat.nr_periods,).toBe(160,);
        expect(read.cpuStat.nr_throttled,).toBe(68,);
        expect(read.cpuStat.usage_usec,).toBe(76_928_181,);
      },
    }),
    it({
      name: 'takes the counters of the scope\'s own bound, and the subtree counters only where the kernel writes no local ones',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'check-scope-accounting-', });
        /**
         A control group whose own bound refused six forks and killed two, beside a nested cgroup's nine and five.
         */
        const withLocal = join(
          scratch.path,
          'app.slice',
          'gpc-check-local.scope',
        );
        await writeStandInCgroup({
          cgroupPath: withLocal,
          figures: standInCgroupFigures,
        });
        /**
         A control group of a kernel that writes only the subtree counters.
         */
        const subtreeOnly = join(
          scratch.path,
          'app.slice',
          'gpc-check-subtree.scope',
        );
        await writeStandInCgroup({
          cgroupPath: subtreeOnly,
          figures: {
            'pids.events': 'max 9',
            'memory.events': 'oom_kill 5',
          },
        });
        /**
         What the reader answered for each of the two.
         */
        const local = await readScopeAccounting({ cgroupPath: withLocal, });
        /**
         What the reader answered where the kernel wrote no local counters.
         */
        const subtree = await readScopeAccounting({ cgroupPath: subtreeOnly, });
        if ((local === SCOPE_NOT_READ) || (subtree === SCOPE_NOT_READ))
          throw new Error('the reader found no control group where the fixture wrote one');
        // The nested cgroup's refusals and kills are not this scope's bound, so the local counters are what it reports.
        expect(local.pidsEvents.max,).toBe(6,);
        expect(local.memoryEvents.oom_kill,).toBe(2,);
        expect(subtree.pidsEvents.max,).toBe(9,);
        expect(subtree.memoryEvents.oom_kill,).toBe(5,);
      },
    }),
    it({
      name: 'answers that a scope was never read when its control group is there but holds no figures',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'check-scope-accounting-', });
        /**
         A folder where a control group is being removed, which answers no file of its own any more.
         */
        const cgroupPath = join(
          scratch.path,
          'gpc-check-removed.scope',
        );
        await mkdir(
          cgroupPath,
          { recursive: true, },
        );
        expect(await readScopeAccounting({ cgroupPath, }),).toBe(SCOPE_NOT_READ,);
      },
    }),
    it({
      name: 'answers that a scope was never read when its control group is not there',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'check-scope-accounting-', });
        expect(await readScopeAccounting({ cgroupPath: join(
          scratch.path,
          'gpc-check-gone.scope'
        ), }),).toBe(SCOPE_NOT_READ,);
      },
    }),
    it({
      name: 'answers that a scope was not read when its control group lost a part of the reading it would replace',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'check-scope-accounting-', });
        /**
         The fixture's control group.
         */
        const cgroupPath = join(
          scratch.path,
          'app.slice',
          'gpc-check-d.scope',
        );
        await writeStandInCgroup({
          cgroupPath,
          figures: standInCgroupFigures,
        });
        /**
         The whole reading, which the readings taken after it are measured against.
         */
        const whole = await readScopeAccounting({ cgroupPath, });
        expect(await readScopeAccounting({
          cgroupPath,
          replaces: whole,
        },),).not.toBe(SCOPE_NOT_READ,);
        // A kernel takes a control group's files away one at a time before the folder itself goes, so this is the first
        // stage of a group going and not a shape no kernel makes.
        await rm(join(
          cgroupPath,
          'cpu.stat',
        ),);
        expect(await readScopeAccounting({
          cgroupPath,
          replaces: whole,
        },),).toBe(SCOPE_NOT_READ,);
        // A reading with nothing to keep replaces nothing, so a group that is still being made is still read.
        expect(await readScopeAccounting({
          cgroupPath,
          replaces: SCOPE_NOT_READ,
        },),).not.toBe(SCOPE_NOT_READ,);
      },
    }),
  ],
});
