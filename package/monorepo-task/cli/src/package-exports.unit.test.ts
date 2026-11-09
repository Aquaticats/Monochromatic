/**
 The package root is the library, not the command line.

 A consumer importing `@monochromatic-dev/monorepo-task-cli` reaches the loader
 surface, every declared subpath of the exports map resolves at runtime, and
 merely importing the package name dispatches no task.

 @module
 */

import { execFile, } from 'node:child_process';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';
import { promisify, } from 'node:util';

import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';

import { loadPackageTasks, } from '@monochromatic-dev/monorepo-task-cli';

/** Package root: the directory holding the manifest the exports map belongs to. */
const PACKAGE_ROOT = fileURLToPath(new URL('..', import.meta.url,),);

/** Ceiling for the child process that imports the package name and nothing else. */
const IMPORTER_TIMEOUT_MS = 60_000;

/** Runtime bindings the library entry exports, each proven callable below. */
const LIBRARY_EXPORTS = [
  'TaskDependencyCycleError',
  'TaskInvocationClosedError',
  'createTaskInvocation',
  'loadPackageTasks',
  'loadRootTasks',
  'resolveDefinition',
  'runTaskCli',
] as const;

/** Task keys this package's own `task.ts` declares, as the loader reports them. */
const OWN_TASK_KEYS = [
  'build',
  'build:js',
  'build:js:node',
  'format:oxlint',
  'lint:oxlint',
  'lint:types',
  'test:unit',
] as const;

/** Subprocess adapter that keeps Node's declared `execFile` promise contract. */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/** Runs one child process to completion. */
const execFileAsync = promisifyExecFile(execFile,);

await describe({
  name: 'package exports',
  children: [
    it({
      name: 'root entry exports the library surface instead of the command line',
      fn: async () => {
        const root = await import('@monochromatic-dev/monorepo-task-cli',);
        expect(Object.keys(root,).toSorted(),).toEqual([...LIBRARY_EXPORTS,]);
        for (const name of LIBRARY_EXPORTS) {
          expect(typeof root[name],).toBe('function',);
        }
      },
    },),
    it({
      name: 'importing the package name dispatches no task and prints nothing',
      fn: async () => {
        const { stdout, stderr, } = await execFileAsync(
          process.execPath,
          ['--input-type=module', '--eval', "await import('@monochromatic-dev/monorepo-task-cli');",],
          { cwd: PACKAGE_ROOT, timeout: IMPORTER_TIMEOUT_MS, },
        );
        expect(stdout,).toBe('',);
        expect(stderr,).toBe('',);
      },
    },),
    it({
      name: 'root entry loads task declarations through the exported loader',
      fn: async () => {
        const tasks = await loadPackageTasks(join(PACKAGE_ROOT, 'task.ts',),);
        expect(Object.keys(tasks,).toSorted(),).toEqual([...OWN_TASK_KEYS,]);
      },
    },),
    it({
      name: 'declared subpaths resolve to the source entry and its modules',
      fn: async () => {
        const entryUrl = import.meta.resolve('@monochromatic-dev/monorepo-task-cli/ts',);
        expect(entryUrl.endsWith('/src/index.ts',),).toBe(true,);
        const entry = await import('@monochromatic-dev/monorepo-task-cli/ts',);
        expect(Object.keys(entry,).toSorted(),).toEqual([...LIBRARY_EXPORTS,]);
        expect(typeof entry.runTaskCli,).toBe('function',);

        const moduleUrl = import.meta.resolve('@monochromatic-dev/monorepo-task-cli/ts/load.ts',);
        expect(moduleUrl.endsWith('/src/load.ts',),).toBe(true,);
        const module = await import('@monochromatic-dev/monorepo-task-cli/ts/load.ts',);
        expect(typeof module.loadPackageTasks,).toBe('function',);
        expect(typeof module.runTaskCli,).toBe('function',);
      },
    },),
  ],
},);
