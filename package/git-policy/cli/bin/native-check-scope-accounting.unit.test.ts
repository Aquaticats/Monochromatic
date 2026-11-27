import { spawnSync, } from 'node:child_process';
import {
  mkdir,
  open,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait as waitUntil } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  SCOPE_NOT_READ,
  checkScopeCgroupPath,
  readScopeAccounting,
  watchScopeAccounting,
} from './native-check-scope-accounting.ts';
import { createGateScratchDirectory } from './stand-in-gate-programs-for-tests.ts';
import { until } from './until-a-test-condition-holds.ts';

/**
 Milliseconds a fixture's watch reads with, which is short so a test ends quickly.
 */
const fixturePollMilliseconds = 20;

/**
 Milliseconds `mkfifo` is given to make one named pipe, which is far more than the call needs.
 */
const namedPipeDeadlineMilliseconds = 5_000;

/**
 Bytes a fixture's `memory.peak` holds, which is the peak a check at its largest was measured at.
 */
const fixtureMemoryPeakBytes = 1_953_531_904;

/**
 Bytes a fixture's `memory.max` holds, which is the 4 GiB bound every recorded check ran under.
 */
const fixtureMemoryMaxBytes = 4_294_967_296;

/**
 Writes one control group's files into a fixture tree, as the kernel would.

 @param cgroupPath - the fixture's control group

 @param figures - further files to write, by name
 */
async function writeCgroup({
  cgroupPath,
  figures,
}: {
  readonly cgroupPath: string;
  readonly figures: Readonly<Record<string, string>>;
}): Promise<void> {
  // The kernel makes a control group with its files already there, so the fixture does too: a watch that read a half
  // written fixture would read a control group no kernel ever made.
  /**
   The folder this fixture writes before its control group appears.
   */
  const written = `${cgroupPath}.written`;
  await mkdir(
    written,
    { recursive: true, },
  );
  await Promise.all(Object.entries(figures,)
    .map(async function writeOne([name, text]) {
      await writeFile(
        join(
          written,
          name
        ),
        `${text}\n`
      );
    },),);
  await rename(written, cgroupPath,);
}

/**
 The figures a control group holds after a loaded check.

 These are a stand-in: the shapes of the files are the kernel's (`bin/native-gate-container-cgroup-memory.ts` reads the
 same names), and the numbers are the ones this round measured for `//package/git-policy/cli:test:unit` at 800% on this
 host on 2026-11-27, with the refused forks added because no unbound run of the suite met its task bound.
 */
const fixtureFigures: Readonly<Record<string, string>> = {
  'pids.current': '30',
  'pids.peak': '434',
  'pids.max': '1024',
  'pids.events': 'max 9',
  'pids.events.local': 'max 6',
  'memory.current': '893263872',
  'memory.peak': String(fixtureMemoryPeakBytes,),
  'memory.max': String(fixtureMemoryMaxBytes,),
  'memory.events': 'low 0\nhigh 0\nmax 1\noom 5\noom_kill 5\noom_group_kill 0',
  'memory.events.local': 'low 0\nhigh 0\nmax 0\noom 2\noom_kill 2\noom_group_kill 0',
  'cpu.max': '800000 100000',
  'cpu.stat': 'usage_usec 76928181\nuser_usec 54543767\nsystem_usec 22384414\nnr_periods 160\nnr_throttled 68\nthrottled_usec 97294323',
};

/**
 Makes a named pipe where one of a control group's files is, which holds one read of a watch open at a chosen point.

 A named pipe is a stand-in: no kernel writes one in a control group, and nothing in this package reads one. It is the
 seam a test needs to know where a reading stood when the group went, rather than hoping a loaded host picks that point.
 The reader of a named pipe waits inside its open until something opens the other end, and then reads until that end
 closes.

 @param path - where the pipe goes

 @throws Error when `mkfifo` did not make the pipe
 */
function makeNamedPipe({ path }: { readonly path: string; }): void {
  /**
   What `mkfifo` answered.
   */
  const made = spawnSync('mkfifo', [path,], { timeout: namedPipeDeadlineMilliseconds, },);
  if (made.status !== 0)
    throw new Error(`mkfifo could not make ${path}: ${made.error?.message ?? made.stderr.toString()}`);
}

/**
 Ends one fixture control group, which is what a scope that ends does to its own.

 A scope's control group goes as one path, so the fixture takes its folder away under another name before it deletes it:
 a reader then sees a scope that is gone rather than a control group being deleted file by file, which is a shape no
 kernel makes.

 @param cgroupPath - the fixture's control group
 */
async function removeCgroup({ cgroupPath }: { readonly cgroupPath: string; }): Promise<void> {
  await rename(cgroupPath, `${cgroupPath}.gone`,);
  await rm(
    `${cgroupPath}.gone`,
    {
      recursive: true,
      force: true,
    },
  );
}

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
        await writeCgroup({
          cgroupPath,
          figures: fixtureFigures,
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
        expect(read.memoryPeakBytes,).toBe(fixtureMemoryPeakBytes,);
        expect(read.memoryMaxBytes,).toBe(fixtureMemoryMaxBytes,);
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
        await writeCgroup({
          cgroupPath: withLocal,
          figures: fixtureFigures,
        });
        /**
         A control group of a kernel that writes only the subtree counters.
         */
        const subtreeOnly = join(
          scratch.path,
          'app.slice',
          'gpc-check-subtree.scope',
        );
        await writeCgroup({
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
        await writeCgroup({
          cgroupPath,
          figures: fixtureFigures,
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
        await removeCgroup({ cgroupPath, });
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
        await writeCgroup({
          cgroupPath,
          // The pipe below goes where `pids.events.local` would be, so this group is one whose kernel writes only the
          // counters of the subtree, and the reading that saw the whole group answers nine refusals from it.
          figures: Object.fromEntries(Object.entries(fixtureFigures,)
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
        makeNamedPipe({ path: held, });
        // The watch's next reading opens the named pipe and waits there, so this writer answers only once that reading
        // has reached this point of its own: no sleep decides where the group goes.
        /**
         The writing end of the held read, held until the group has gone.
         */
        const writer = await open(held, 'w',);
        await removeCgroup({ cgroupPath, });
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
