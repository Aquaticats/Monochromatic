import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { makeHost, } from './test-fixture.ts';

/**
 Path of the watcher entry point.
 */
const WATCHER = join(import.meta.dirname, 'cpu-watch.ts',);

/**
 How long the smoke test lets the watcher run before stopping it, in milliseconds; two passes plus slack.
 */
const RUN_MS = 5_500;

await describe({
  name: 'cpu-watch',
  children: [
    it({
      name: 'starts on fixture paths, applies the cap, samples, and stops cleanly on SIGTERM',
      fn: async function runsAndStops() {
        /**
         Disposable host the watcher is pointed at.
         */
        const { root, paths, } = await makeHost({ threads: '28', cpuPressure: 0, },);
        /**
         The watcher process with every path overridden to the fixture.
         */
        const child = spawn(process.execPath, [WATCHER,], {
          env: {
            ...process.env,
            CPU_WATCH_CAP_FILE: paths.capFile,
            CPU_WATCH_CGROUP_FILE: paths.cgroupFile,
            CPU_WATCH_HWMON_ROOT: paths.hwmonRoot,
            CPU_WATCH_PROC_ROOT: paths.procRoot,
            CPU_WATCH_SAMPLE_DIR: paths.sampleDir,
          },
          stdio: ['ignore', 'pipe', 'pipe',],
        },);
        /**
         Everything the watcher printed, in arrival order.
         */
        const output: string[] = [];
        child.stdout.on('data', function collectOut(chunk: Buffer,): void {
          output.push(chunk.toString(),);
        },);
        child.stderr.on('data', function collectErr(chunk: Buffer,): void {
          output.push(chunk.toString(),);
        },);
        await wait(RUN_MS,);
        child.kill('SIGTERM',);
        const exit: readonly unknown[] = await once(child, 'exit',);
        expect(exit[0],).toBe(0,);
        /**
         All output joined for substring checks.
         */
        const text = output.join('',);
        expect(text,).toContain('WATCHER-STARTED',);
        expect(text,).toContain('CAP-APPLIED',);
        expect(text,).toContain('WATCHER-STOPPED',);
        expect(await readFile(join(root, 'cgroup', 'cpu.max',), 'utf8',),).toBe('2800000 100000',);
      },
    },),
  ],
},);
