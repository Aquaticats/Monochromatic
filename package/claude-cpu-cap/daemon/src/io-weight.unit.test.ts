import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { NOT_MEASURED, } from './constants.ts';
import { parseDefaultWeight, reconcileIoWeight, } from './io-weight.ts';
import {
  makeHost,
  recordingLogger,
} from './test-fixture.ts';

await describe({
  name: 'io-weight',
  children: [
    describe({
      name: parseDefaultWeight.name,
      children: [
        it({
          name: 'reads the default line and ignores per-device lines',
          fn: async function readsDefault() {
            expect(parseDefaultWeight('default 100\n259:0 50\n',),).toBe(100,);
            expect(parseDefaultWeight('259:0 50\ndefault 7\n',),).toBe(7,);
          },
        },),
        it({
          name: 'yields NOT_MEASURED when there is no default line or it holds no number',
          fn: async function notMeasured() {
            // NaN is never strictly equal to itself, so the marker is compared through Number.isNaN.
            expect(Number.isNaN(NOT_MEASURED,),).toBe(true,);
            expect(Number.isNaN(parseDefaultWeight('259:0 50\n',),),).toBe(true,);
            expect(Number.isNaN(parseDefaultWeight('default abc\n',),),).toBe(true,);
            expect(Number.isNaN(parseDefaultWeight('',),),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: reconcileIoWeight.name,
      children: [
        it({
          name: 'writes the default weight and logs IO-WEIGHT-APPLIED when it differs',
          fn: async function appliesWeight() {
            /**
             Host whose cgroup starts at the kernel default weight of 100.
             */
            const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            expect(await reconcileIoWeight({ ioWeightFile: paths.ioWeightFile, weight: 10, l, },),).toBe(true,);
            expect(await readFile(paths.ioWeightFile, 'utf8',),).toBe('default 10',);
            expect(l.messages.info[0],).toContain('IO-WEIGHT-APPLIED',);
          },
        },),
        it({
          name: 'leaves io.weight alone when the default already matches, per-device lines included',
          fn: async function leavesEqual() {
            /**
             Host whose cgroup already holds the weight next to a per-device override.
             */
            const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            await writeFile(paths.ioWeightFile, 'default 10\n259:0 50\n',);
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            expect(await reconcileIoWeight({ ioWeightFile: paths.ioWeightFile, weight: 10, l, },),).toBe(false,);
            expect(await readFile(paths.ioWeightFile, 'utf8',),).toBe('default 10\n259:0 50\n',);
            expect(l.messages.info,).toEqual([],);
          },
        },),
        it({
          name: 'writes the default weight when the file lists only per-device weights',
          fn: async function writesWithoutDefault() {
            /**
             Host whose io.weight has no default line.
             */
            const { paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            await writeFile(paths.ioWeightFile, '259:0 50\n',);
            expect(await reconcileIoWeight({ ioWeightFile: paths.ioWeightFile, weight: 10, l: recordingLogger(), },),).toBe(true,);
            expect(await readFile(paths.ioWeightFile, 'utf8',),).toBe('default 10',);
          },
        },),
        it({
          name: 'does nothing when the cgroup does not exist yet',
          fn: async function absentCgroup() {
            /**
             Host with a cgroup path that does not exist.
             */
            const { root, } = await makeHost({ threads: '28', cpuPressure: 0, },);
            expect(await reconcileIoWeight({ ioWeightFile: `${root}/no-cgroup/io.weight`, weight: 10, l: recordingLogger(), },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
