import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  answersEveryPartOf,
  isRemoved,
  partsOf,
} from './cgroup-whole-readings.ts';

/**
 The sentinel one reader of a control group answers with when a reading is not there to keep.
 */
const NOT_READ: unique symbol = Symbol('a reading of a control group that is not there to keep',);

await describe({
  name: '',
  children: [
    describe({
      name: partsOf.name,
      children: [
        it({
          name: 'names each figure a reading answered and each counter of each of its counter files',
          fn: async () => {
            expect(partsOf({
              sample: {
                cgroupPath: '/sys/fs/cgroup/a.scope',
                readAt: '2026-11-27T13:00:00.000Z',
                pidsPeak: 434,
                pidsEvents: { max: 9, },
                cpuStat: {
                  nr_periods: 160,
                  nr_throttled: 68,
                },
              },
            },),).toEqual(new Set([
              'pidsPeak',
              'pidsEvents.max',
              'cpuStat.nr_periods',
              'cpuStat.nr_throttled',
            ],),);
          },
        }),
        it({
          name: 'names no part for a figure a kernel wrote nothing for and for the text a reading carries',
          fn: async () => {
            expect(partsOf({
              sample: {
                cgroupPath: '/sys/fs/cgroup/b.scope',
                readAt: '2026-11-27T13:00:00.000Z',
                peakBytes: undefined,
                events: {},
              },
            },),).toEqual(new Set(),);
          },
        }),
      ],
    }),
    describe({
      name: answersEveryPartOf.name,
      children: [
        it({
          name: 'keeps a reading that answers every part of the one before it, and every part it answers more',
          fn: async () => {
            expect(answersEveryPartOf({
              read: {
                pidsPeak: 434,
                events: {
                  max: 1,
                  oom_kill: 1,
                },
              },
              kept: {
                pidsPeak: 434,
                events: { max: 1, },
              },
            },),).toBe(true,);
          },
        }),
        it({
          name: 'refuses a reading that answered less than the one it would replace, as a group going leaves one',
          fn: async () => {
            expect(answersEveryPartOf({
              read: {
                pidsPeak: 434,
                events: {},
              },
              kept: {
                pidsPeak: 434,
                events: {
                  max: 1,
                  oom_kill: 1,
                },
              },
            },),).toBe(false,);
          },
        }),
        it({
          name: 'keeps the first reading of a group there is, and refuses a reading that is not there after one',
          fn: async () => {
            // A reading with nothing before it replaces nothing, so a group that is still being made is still read.
            expect(answersEveryPartOf({
              read: { pidsPeak: 434, },
              kept: NOT_READ,
            },),).toBe(true,);
            expect(answersEveryPartOf({
              read: NOT_READ,
              kept: { pidsPeak: 434, },
            },),).toBe(false,);
            expect(answersEveryPartOf({
              read: NOT_READ,
              kept: NOT_READ,
            },),).toBe(true,);
          },
        }),
      ],
    }),
    describe({
      name: isRemoved.name,
      children: [
        it({
          name: 'takes the kernel removing a file of a control group as its own answer, and no other error',
          fn: async () => {
            // The error Node builds for this kernel's answer, captured on this host on 2026-11-27 from a control group
            // the account made and removed while a reader read it (`kernfs_seq_start` answering `ENODEV` for a node the
            // kernel had deactivated): `doc/troubleshooting/a-cgroup-gives-its-files-up-one-at-a-time-before-its-directory-goes.md`.
            /**
             One answer of a file whose control group is going.
             */
            const removed = Object.assign(
              new Error('ENODEV: no such device, read \'/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/gpc-probe-6610-d6157fac49704927860679adb5c5b0b6-110/pids.max\''),
              {
                code: 'ENODEV',
                errno: -19,
                syscall: 'read',
                path: '/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/gpc-probe-6610-d6157fac49704927860679adb5c5b0b6-110/pids.max',
              },
            );
            expect(isRemoved(removed,),).toBe(true,);
            // A file that is not there, and words that only look like the code, are not an answer of the kernel's.
            /**
             The answer of a file a directory does not hold.
             */
            const missing = Object.assign(
              new Error('ENOENT: no such file or directory, open \'/sys/fs/cgroup/a.scope/pids.max\''),
              {
                code: 'ENOENT',
                errno: -2,
                syscall: 'open',
                path: '/sys/fs/cgroup/a.scope/pids.max',
              },
            );
            expect(isRemoved(missing,),).toBe(false,);
            /**
             Words that name the code without carrying it.
             */
            const words = new Error('ENODEV',);
            expect(isRemoved(words,),).toBe(false,);
          },
        }),
      ],
    }),
  ],
});
