import {
  access,
  chmod,
  mkdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  DEFAULT_IO_WEIGHT,
  IO_BFQ_WEIGHT_MAX,
  IO_WEIGHT_MAX,
  NOT_MEASURED,
} from './constants.ts';
import { pathExists, } from './files.ts';
import { IoWeightError, } from './io-weight-error.ts';
import {
  clampWeight,
  parseDefaultWeight,
  reconcileIoWeight,
  reconcileIoWeightOrLog,
} from './io-weight.ts';
import {
  makeHost,
  recordingLogger,
} from './test-fixture.ts';
import type { Paths, } from './types.ts';

/**
 Whether the tests run as root, which ignores file modes and so cannot provoke a refused write with `chmod`.
 */
const RUNS_AS_ROOT = process.getuid?.() === 0;

/**
 Runs a call that is expected to reject and hands back what it rejected with.

 @param run - the call

 @returns the rejection, or undefined when the call resolved
 */
async function rejectionOf(run: () => Promise<unknown>,): Promise<unknown> {
  try {
    await run();
  } catch (error) {
    return error;
  }
  return undefined;
}

/**
 Runs a call that is expected to reject with an {@link IoWeightError} and hands that error back.

 @param run - the call

 @returns the error the call rejected with

 @throws when the call resolved or rejected with anything else
 */
async function ioWeightErrorOf(run: () => Promise<unknown>,): Promise<IoWeightError> {
  /**
   What the call rejected with, undefined when it resolved.
   */
  const caught = await rejectionOf(run,);
  if (caught instanceof IoWeightError) return caught;
  throw new Error(
    `expected an IoWeightError but the call ended with ${String(caught,)}`,
    { cause: caught, },
  );
}

/**
 Reads both weight files of a fixture host.

 @param paths - the fixture paths

 @returns the text of `io.bfq.weight` and of `io.weight`, in that order
 */
async function readWeights(paths: Paths,): Promise<readonly [string, string]> {
  return [
    await readFile(paths.ioBfqWeightFile, 'utf8',),
    await readFile(paths.ioWeightFile, 'utf8',),
  ];
}

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
            /**
             Weight read from a file with per-device lines only.
             */
            const perDeviceOnly = parseDefaultWeight('259:0 50\n',);
            /**
             Weight read from a default line without a number.
             */
            const noNumber = parseDefaultWeight('default abc\n',);
            /**
             Weight read from an empty file.
             */
            const empty = parseDefaultWeight('',);
            expect(Number.isNaN(perDeviceOnly,),).toBe(true,);
            expect(Number.isNaN(noNumber,),).toBe(true,);
            expect(Number.isNaN(empty,),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: clampWeight.name,
      children: [
        it({
          name: 'keeps a weight inside the range and moves one outside it to the nearest bound',
          fn: async function clamps() {
            expect(clampWeight({ weight: 10, max: IO_BFQ_WEIGHT_MAX, },),).toBe(10,);
            expect(clampWeight({ weight: 1, max: IO_BFQ_WEIGHT_MAX, },),).toBe(1,);
            expect(clampWeight({ weight: IO_BFQ_WEIGHT_MAX, max: IO_BFQ_WEIGHT_MAX, },),).toBe(IO_BFQ_WEIGHT_MAX,);
            expect(clampWeight({ weight: 0, max: IO_BFQ_WEIGHT_MAX, },),).toBe(1,);
            expect(clampWeight({ weight: -5, max: IO_WEIGHT_MAX, },),).toBe(1,);
            expect(clampWeight({ weight: 1_001, max: IO_BFQ_WEIGHT_MAX, },),).toBe(IO_BFQ_WEIGHT_MAX,);
            expect(clampWeight({ weight: 1_001, max: IO_WEIGHT_MAX, },),).toBe(1_001,);
            expect(clampWeight({ weight: 10_001, max: IO_WEIGHT_MAX, },),).toBe(IO_WEIGHT_MAX,);
          },
        },),
      ],
    },),
    describe({
      name: reconcileIoWeight.name,
      children: [
        it({
          name: 'both files: writes the default weight into io.bfq.weight and io.weight and logs IO-WEIGHT-APPLIED for each',
          fn: async function bothFiles() {
            /**
             Host whose cgroup offers both weight files at the kernel default of 100.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { paths, } = host;
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            expect(await reconcileIoWeight({ ...paths, weight: 10, l, },),).toBe(true,);
            expect(await readWeights(paths,),).toEqual([`default ${DEFAULT_IO_WEIGHT}`, `default ${DEFAULT_IO_WEIGHT}`,],);
            expect(l.messages.info,).toHaveLength(2,);
            expect(l.messages.info.some(function namesBfq(line,) {
              return line.includes('IO-WEIGHT-APPLIED',) && line.includes(`io.bfq.weight default 100 -> ${DEFAULT_IO_WEIGHT}`);
            },),).toBe(true,);
            expect(l.messages.info.some(function namesIoWeight(line,) {
              return line.includes('IO-WEIGHT-APPLIED',) && line.includes(`io.weight default 100 -> ${DEFAULT_IO_WEIGHT}`);
            },),).toBe(true,);
            expect(l.messages.warn,).toEqual([],);
          },
        },),
        it({
          name: 'only io.weight (iocost kernel): writes io.weight and does not create io.bfq.weight',
          fn: async function onlyIoWeight() {
            /**
             Host whose kernel offers no BFQ weight file.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { paths, } = host;
            await rm(paths.ioBfqWeightFile,);
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            expect(await reconcileIoWeight({ ...paths, weight: 10, l, },),).toBe(true,);
            expect(await readFile(paths.ioWeightFile, 'utf8',),).toBe(`default ${DEFAULT_IO_WEIGHT}`,);
            expect(await pathExists(paths.ioBfqWeightFile,),).toBe(false,);
            expect(l.messages.info,).toHaveLength(1,);
            expect(l.messages.info[0],).toContain(`io.weight default 100 -> ${DEFAULT_IO_WEIGHT}`,);
            expect(l.messages.warn,).toEqual([],);
            expect(l.messages.error,).toEqual([],);
          },
        },),
        it({
          name: 'only io.bfq.weight (BFQ kernel without iocost): writes io.bfq.weight and does not create io.weight',
          fn: async function onlyBfqWeight() {
            /**
             Host whose kernel offers no iocost weight file.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { paths, } = host;
            await rm(paths.ioWeightFile,);
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            expect(await reconcileIoWeight({ ...paths, weight: 10, l, },),).toBe(true,);
            expect(await readFile(paths.ioBfqWeightFile, 'utf8',),).toBe(`default ${DEFAULT_IO_WEIGHT}`,);
            expect(await pathExists(paths.ioWeightFile,),).toBe(false,);
            expect(l.messages.info,).toHaveLength(1,);
            expect(l.messages.info[0],).toContain(`io.bfq.weight default 100 -> ${DEFAULT_IO_WEIGHT}`,);
            expect(l.messages.warn,).toEqual([],);
            expect(l.messages.error,).toEqual([],);
          },
        },),
        it({
          name: 'out-of-range weight: clamps each file to its own range, warns once per write, and settles',
          fn: async function clampsPerFile() {
            /**
             Host with both weight files.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { paths, } = host;
            /**
             Recorder for the first call.
             */
            const high = recordingLogger();
            // 5000 fits io.weight (1 to 10000) but not io.bfq.weight (1 to 1000).
            expect(await reconcileIoWeight({ ...paths, weight: 5_000, l: high, },),).toBe(true,);
            expect(await readWeights(paths,),).toEqual(['default 1000', 'default 5000',],);
            expect(high.messages.warn,).toHaveLength(1,);
            expect(high.messages.warn[0],).toContain('IO-WEIGHT-CLAMPED',);
            expect(high.messages.warn[0],).toContain(paths.ioBfqWeightFile,);
            expect(high.messages.warn[0],).toContain('1 to 1000',);
            // The clamped value is already in place, so the next pass writes and warns about nothing.
            /**
             Recorder for the repeat call.
             */
            const repeat = recordingLogger();
            expect(await reconcileIoWeight({ ...paths, weight: 5_000, l: repeat, },),).toBe(false,);
            expect(repeat.messages.warn,).toEqual([],);
            expect(repeat.messages.info,).toEqual([],);
            /**
             Recorder for the call with a weight larger than both maxima.
             */
            const tooHigh = recordingLogger();
            expect(await reconcileIoWeight({ ...paths, weight: 20_000, l: tooHigh, },),).toBe(true,);
            expect(await readWeights(paths,),).toEqual(['default 1000', 'default 10000',],);
            expect(tooHigh.messages.warn,).toHaveLength(1,);
            expect(tooHigh.messages.warn[0],).toContain(paths.ioWeightFile,);
            /**
             Recorder for the call with a weight smaller than both minima.
             */
            const tooLow = recordingLogger();
            expect(await reconcileIoWeight({ ...paths, weight: 0, l: tooLow, },),).toBe(true,);
            expect(await readWeights(paths,),).toEqual(['default 1', 'default 1',],);
            expect(tooLow.messages.warn,).toHaveLength(2,);
          },
        },),
        it({
          name: 'leaves both files alone when the default already matches, per-device lines included',
          fn: async function leavesEqual() {
            /**
             Host whose cgroup already holds the weight next to a per-device override in both files.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { paths, } = host;
            await writeFile(paths.ioWeightFile, 'default 10\n259:0 50\n',);
            await writeFile(paths.ioBfqWeightFile, 'default 10\n259:0 50\n',);
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            expect(await reconcileIoWeight({ ...paths, weight: 10, l, },),).toBe(false,);
            expect(await readWeights(paths,),).toEqual(['default 10\n259:0 50\n', 'default 10\n259:0 50\n',],);
            expect(l.messages.info,).toEqual([],);
          },
        },),
        it({
          name: 'writes the default weight when the files list only per-device weights',
          fn: async function writesWithoutDefault() {
            /**
             Host whose weight files have no default line.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { paths, } = host;
            await writeFile(paths.ioWeightFile, '259:0 50\n',);
            await writeFile(paths.ioBfqWeightFile, '259:0 50\n',);
            expect(await reconcileIoWeight({ ...paths, weight: 10, l: recordingLogger(), },),).toBe(true,);
            expect(await readWeights(paths,),).toEqual([`default ${DEFAULT_IO_WEIGHT}`, `default ${DEFAULT_IO_WEIGHT}`,],);
          },
        },),
        it({
          name: 'does nothing, and does not throw, when the cgroup does not exist yet',
          fn: async function absentCgroup() {
            /**
             Host with a cgroup path that does not exist.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { root, } = host;
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            expect(await reconcileIoWeight({
              ioWeightFile: `${root}/no-cgroup/io.weight`,
              ioBfqWeightFile: `${root}/no-cgroup/io.bfq.weight`,
              weight: 10,
              l,
            },),).toBe(false,);
            expect(l.messages.warn,).toEqual([],);
            expect(l.messages.error,).toEqual([],);
          },
        },),
        it({
          name: 'neither file in an existing cgroup: throws no-weight-file naming both files',
          fn: async function neitherFile() {
            /**
             Host whose cgroup directory exists without either weight file (io controller off).
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { paths, } = host;
            await rm(paths.ioWeightFile,);
            await rm(paths.ioBfqWeightFile,);
            /**
             What the call rejected with.
             */
            const caught = await ioWeightErrorOf(async function reconcile() {
              return reconcileIoWeight({ ...paths, weight: 10, l: recordingLogger(), },);
            },);
            expect(caught.reason,).toBe('no-weight-file',);
            expect(caught.files,).toEqual([paths.ioBfqWeightFile, paths.ioWeightFile,],);
            expect(caught.message,).toContain(paths.ioBfqWeightFile,);
            expect(caught.message,).toContain('cgroup.subtree_control',);
          },
        },),
        it({
          name: 'failed write: still writes the other file, then throws file-access-failed naming the refused file',
          skip: RUNS_AS_ROOT ? 'root ignores file modes, so chmod cannot make the write fail' : false,
          fn: async function failedWrite() {
            /**
             Host whose io.bfq.weight is read-only.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { paths, } = host;
            await chmod(paths.ioBfqWeightFile, 0o444,);
            // Positive control: the file really refuses writes, so the rejection of the reconcile call comes from the write.
            expect(await rejectionOf(async function probeWrite() {
              return access(paths.ioBfqWeightFile, 2,);
            },),).toBeDefined();
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            /**
             What the call rejected with.
             */
            const caught = await ioWeightErrorOf(async function reconcile() {
              return reconcileIoWeight({ ...paths, weight: 10, l, },);
            },);
            expect(caught.reason,).toBe('file-access-failed',);
            expect(caught.files,).toEqual([paths.ioBfqWeightFile,],);
            expect(caught.message,).toContain(paths.ioBfqWeightFile,);
            expect(caught.message,).toContain('EACCES',);
            expect(caught.message,).toContain('The other weight file was still tried',);
            expect(caught.cause,).toBeInstanceOf(AggregateError,);
            expect(await readWeights(paths,),).toEqual(['default 100\n', `default ${DEFAULT_IO_WEIGHT}`,],);
            expect(l.messages.info,).toHaveLength(1,);
          },
        },),
        it({
          name: 'failed read: a weight path that is a directory throws file-access-failed and the other file is still written',
          fn: async function failedRead() {
            /**
             Host whose io.bfq.weight is a directory, so reading it fails with EISDIR.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { paths, } = host;
            await rm(paths.ioBfqWeightFile,);
            await mkdir(paths.ioBfqWeightFile,);
            /**
             What the call rejected with.
             */
            const caught = await ioWeightErrorOf(async function reconcile() {
              return reconcileIoWeight({ ...paths, weight: 10, l: recordingLogger(), },);
            },);
            expect(caught.reason,).toBe('file-access-failed',);
            expect(caught.files,).toEqual([paths.ioBfqWeightFile,],);
            expect(caught.message,).toContain('EISDIR',);
            expect(await readFile(paths.ioWeightFile, 'utf8',),).toBe(`default ${DEFAULT_IO_WEIGHT}`,);
          },
        },),
      ],
    },),
    describe({
      name: reconcileIoWeightOrLog.name,
      children: [
        it({
          name: 'returns whether a file was rewritten and logs nothing at warn or error on success',
          fn: async function succeeds() {
            /**
             Host with both weight files.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { paths, } = host;
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            expect(await reconcileIoWeightOrLog({ ...paths, weight: 10, l, },),).toBe(true,);
            expect(await reconcileIoWeightOrLog({ ...paths, weight: 10, l, },),).toBe(false,);
            expect(l.messages.warn,).toEqual([],);
            expect(l.messages.error,).toEqual([],);
          },
        },),
        it({
          name: 'logs IO-WEIGHT-UNAVAILABLE at warn and returns false when the cgroup has neither file',
          fn: async function logsUnavailable() {
            /**
             Host whose cgroup directory exists without either weight file.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { paths, } = host;
            await rm(paths.ioWeightFile,);
            await rm(paths.ioBfqWeightFile,);
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            expect(await reconcileIoWeightOrLog({ ...paths, weight: 10, l, },),).toBe(false,);
            expect(l.messages.warn,).toHaveLength(1,);
            expect(l.messages.warn[0],).toContain('IO-WEIGHT-UNAVAILABLE',);
            expect(l.messages.warn[0],).toContain(paths.ioBfqWeightFile,);
            expect(l.messages.warn[0],).toContain(paths.ioWeightFile,);
            expect(l.messages.error,).toEqual([],);
          },
        },),
        it({
          name: 'logs IO-WEIGHT-FAILED at error and returns false when a weight file cannot be accessed',
          fn: async function logsFailed() {
            /**
             Host whose io.weight is a directory, so reading it fails with EISDIR.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { paths, } = host;
            await rm(paths.ioWeightFile,);
            await mkdir(paths.ioWeightFile,);
            /**
             Recorder for the logged lines.
             */
            const l = recordingLogger();
            expect(await reconcileIoWeightOrLog({ ...paths, weight: 10, l, },),).toBe(false,);
            expect(l.messages.error,).toHaveLength(1,);
            expect(l.messages.error[0],).toContain('IO-WEIGHT-FAILED',);
            expect(l.messages.error[0],).toContain(paths.ioWeightFile,);
            expect(l.messages.error[0],).toContain('EISDIR',);
            expect(await readFile(paths.ioBfqWeightFile, 'utf8',),).toBe(`default ${DEFAULT_IO_WEIGHT}`,);
          },
        },),
        it({
          name: 'rethrows a failure that is not an IoWeightError',
          fn: async function rethrowsForeign() {
            /**
             Host whose cgroup does not exist, so the absent-cgroup debug line is the only logging.
             */
            await using host = await makeHost({ threads: '28', cpuPressure: 0, },);
            const { root, } = host;
            /**
             Logger whose debug method fails, standing in for any unexpected error inside the call.
             */
            const broken = {
              ...recordingLogger(),
              debug: function failingDebug(): void {
                throw new Error('logger broke',);
              },
            };
            /**
             What the call rejected with.
             */
            const caught = await rejectionOf(async function reconcile() {
              return reconcileIoWeightOrLog({
                ioWeightFile: `${root}/no-cgroup/io.weight`,
                ioBfqWeightFile: `${root}/no-cgroup/io.bfq.weight`,
                weight: 10,
                l: broken,
              },);
            },);
            expect(Error.isError(caught,) && caught.message,).toBe('logger broke',);
          },
        },),
      ],
    },),
  ],
},);
