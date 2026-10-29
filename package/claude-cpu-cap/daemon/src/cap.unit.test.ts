import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { desiredThreads, reconcileCap, } from './cap.ts';
import { DEFAULT_THREADS, } from './constants.ts';
import {
  makeHost,
  recordingLogger,
} from './test-fixture.ts';

await describe({
  name: 'cap',
  children: [
    describe({
      name: desiredThreads.name,
      children: [
        it({
          name: 'reads an integer inside the allowed range',
          fn: async function readsInteger() {
            /**
             Host whose cap file holds 28.
             */
            const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            expect(await desiredThreads({ capFile: paths.capFile, totalThreads: 32, l: recordingLogger(), },),).toBe(28,);
          },
        },),
        it({
          name: 'uses the default when the file is absent',
          fn: async function defaultsWhenAbsent() {
            /**
             Host with an unrelated cap file path.
             */
            const { root, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            expect(await desiredThreads({ capFile: `${root}/missing`, totalThreads: 32, l: recordingLogger(), },),).toBe(DEFAULT_THREADS,);
          },
        },),
        it({
          name: 'uses the default and warns on zero, too many, or non-integer values',
          fn: async function warnsOnInvalid() {
            await Promise.all(['0', '33', 'abc', '2.5',].map(async function checkBad(bad,) {
              /**
               Host whose cap file holds the bad value.
               */
              const { paths, } = await makeHost({ threads: bad, cpuPressure: 0, },);
              /**
               Recorder for this case.
               */
              const l = recordingLogger();
              expect(await desiredThreads({ capFile: paths.capFile, totalThreads: 32, l, },),).toBe(DEFAULT_THREADS,);
              expect(l.messages.warn[0],).toContain('CAP-FILE-INVALID',);
            },),);
          },
        },),
      ],
    },),
    describe({
      name: reconcileCap.name,
      children: [
        it({
          name: 'writes the quota into cpu.max and logs CAP-APPLIED when it differs',
          fn: async function appliesQuota() {
            /**
             Host whose cgroup starts unlimited.
             */
            const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            expect(await reconcileCap({ capFile: paths.capFile, cgroupFile: paths.cgroupFile, totalThreads: 32, l, },),)
              .toEqual({ threads: 28, applied: true, },);
            expect(await readFile(paths.cgroupFile, 'utf8',),).toBe('2800000 100000',);
            expect(l.messages.info[0],).toContain('CAP-APPLIED',);
          },
        },),
        it({
          name: 'leaves cpu.max alone when it already holds the wanted quota',
          fn: async function leavesEqual() {
            /**
             Host whose cgroup already holds the quota.
             */
            const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            await writeFile(paths.cgroupFile, '2800000 100000\n',);
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            expect(await reconcileCap({ capFile: paths.capFile, cgroupFile: paths.cgroupFile, totalThreads: 32, l, },),)
              .toEqual({ threads: 28, applied: false, },);
            expect(l.messages.info,).toEqual([],);
          },
        },),
        it({
          name: 'does nothing when the cgroup does not exist yet',
          fn: async function absentCgroup() {
            /**
             Host with a cgroup path that does not exist.
             */
            const { root, paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            expect(await reconcileCap({ capFile: paths.capFile, cgroupFile: `${root}/no-cgroup/cpu.max`, totalThreads: 32, l: recordingLogger(), },),)
              .toEqual({ threads: 28, applied: false, },);
          },
        },),
      ],
    },),
  ],
},);
