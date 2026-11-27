import { open } from 'node:fs/promises';
import { join, } from 'node:path';

import { wait as waitUntil } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { SCOPE_NOT_READ } from './native-check-scope-accounting.ts';
import { watchScopeAccounting } from './native-check-scope-watch-accounting.ts';
import { createGateScratchDirectory } from './stand-in-gate-programs-for-tests.ts';
import {
  makeStandInNamedPipe,
  removeStandInCgroup,
  standInCgroupFigures,
  writeStandInCgroup,
} from './stand-in-cgroup-files-for-tests.ts';
import { until } from './until-a-test-condition-holds.ts';

/**
 Milliseconds a fixture's watch reads with, which is short so a test ends quickly.
 */
const fixturePollMilliseconds = 20;

await describe({
  name: 'the accounting watch of one check\'s scope',
  children: [
    it({
      name: 'keeps the last figures a scope gave after the scope and its control group are gone',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'check-scope-accounting-', });
        /**
         The fixture's control group.
         */
        const cgroupPath = join(
          scratch.path,
          'app.slice',
          'gpc-check-b.scope',
        );
        await writeStandInCgroup({
          cgroupPath,
          figures: standInCgroupFigures,
        });
        await using watch = watchScopeAccounting({
          cgroupPath,
          intervalMilliseconds: fixturePollMilliseconds,
        });
        await until({
          condition: function read() {
            return watch.lastRead() !== SCOPE_NOT_READ;
          },
          description: 'the watch to read the fixture\'s control group',
          pollMilliseconds: fixturePollMilliseconds,
        },);
        // The scope ends when its work does, which takes its control group with it; what was read must survive that.
        await removeStandInCgroup({ cgroupPath, });
        await waitUntil(fixturePollMilliseconds * 2,);
        /**
         What the watch kept once the control group was gone.
         */
        const kept = watch.lastRead();
        if (kept === SCOPE_NOT_READ)
          throw new Error('the watch never read the control group the fixture wrote');
        expect(kept.pidsPeak,).toBe(434,);
        expect(kept.cpuStat.nr_throttled,).toBe(68,);
        await watch.stop();
      },
    }),
    it({
      name: 'never lets a reading that lost its control group part way replace the whole reading before it',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'check-scope-accounting-', });
        /**
         The fixture's control group.
         */
        const cgroupPath = join(
          scratch.path,
          'app.slice',
          'gpc-check-c.scope',
        );
        await writeStandInCgroup({
          cgroupPath,
          // The pipe below goes where `pids.events.local` would be, so this group is one whose kernel writes only the
          // counters of the subtree, and the reading that saw the whole group answers nine refusals from it.
          figures: Object.fromEntries(Object.entries(standInCgroupFigures,)
            .filter(function notTheLocalPidsEvents([name]) {
              return name !== 'pids.events.local';
            },),),
        });
        await using watch = watchScopeAccounting({
          cgroupPath,
          intervalMilliseconds: fixturePollMilliseconds,
        });
        await until({
          condition: function whole() {
            /**
             What the watch has read so far.
             */
            const read = watch.lastRead();
            return (read !== SCOPE_NOT_READ) && (read.cpuStat.nr_throttled === 68);
          },
          description: 'the watch to read the whole control group the fixture wrote',
          pollMilliseconds: fixturePollMilliseconds,
        },);
        /**
         The counter file whose read this test holds open, so the control group goes at a chosen point of a reading.
         */
        const held = join(
          cgroupPath,
          'pids.events.local',
        );
        await makeStandInNamedPipe({ path: held, });
        // The watch's next reading opens the named pipe and waits there, so this writer answers only once that reading
        // has reached this point of its own: no sleep decides where the group goes.
        /**
         The writing end of the held read, held until the group has gone.
         */
        const writer = await open(held, 'w',);
        await removeStandInCgroup({ cgroupPath, });
        await writer.close();
        // A watch is stopped by waiting for its reading loop, so the reading that lost its group has certainly ended.
        await watch.stop();
        /**
         What the watch kept once a reading had lost part of its group.
         */
        const kept = watch.lastRead();
        if (kept === SCOPE_NOT_READ)
          throw new Error('the watch never read the whole control group the fixture wrote');
        expect(kept.pidsPeak,).toBe(434,);
        expect(kept.pidsEvents.max,).toBe(9,);
        expect(kept.memoryEvents.oom_kill,).toBe(2,);
        expect(kept.cpuStat.nr_throttled,).toBe(68,);
      },
    }),
  ],
});
