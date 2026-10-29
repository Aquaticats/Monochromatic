import { existsSync, } from 'node:fs';
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  stat,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { LogRecord, } from '@monochromatic-dev/logger-core';
import {
  _findNodeModulesUp as findNodeModulesUp,
  _NO_NODE_MODULES_FOUND as NO_NODE_MODULES_FOUND,
  createFileSink,
  DEFAULT_LOG_MAX_BYTES,
  DEFAULT_LOG_MAX_FILES,
  NO_LOG_RETENTION_SWEEP,
} from '@monochromatic-dev/logger-core/node';

/**
 Mock `stat` that always throws an ENOENT-like error, so `findNodeModulesUp`
 walks the whole tree and exhausts without matching.

 @returns Never; always throws.
 */
function statAlwaysMissing(): never {
  const error: NodeJS.ErrnoException = Object.assign(
    new Error('ENOENT',),
    { code: 'ENOENT', },
  );
  throw error;
}

/**
 Mock `stat` that always throws an unexpected permission error.

 @returns Never; always throws.
 */
function statAlwaysDenied(): never {
  const error: NodeJS.ErrnoException = Object.assign(
    new Error('EACCES',),
    { code: 'EACCES', },
  );
  throw error;
}

/**
 Builds a LogRecord for write-path tests.

 @param message - Message body.

 @returns Record at a fixed timestamp.
 */
function record({ message, }: { readonly message: string; },): LogRecord {
  return {
    level: 'info',
    message,
    timestamp: 0,
  };
}

/**
 Disposable sink destination under the agent scratch directory,
 so no in-process sink ever probes or sweeps shared log state.

 @returns Async-disposable destination root.
 */
async function scratchDestination() {
  /**
   Agent scratch root, made private before fixtures are created inside it.
   */
  const scratch = join(
    homedir(),
    'temp',
    'agent',
  );
  await mkdir(
    scratch,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  await chmod(scratch, 0o700,);
  const path = await mkdtemp(join(
    scratch,
    'logger-file-',),
  );
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Every entry name in a destination, one and two levels deep,
 so stamps, probes, and day-directory logs all stay visible.

 @param directory - Destination root to list.

 @returns Sorted `name` and `day/name` entry names.
 */
async function entryNames(directory: string,): Promise<string[]> {
  /**
   Root entries of the destination, expanded one level into directories.
   */
  const entries = await readdir(
    directory,
    { withFileTypes: true, },
  );
  const names = await Promise.all(entries.map(async function listEntry(entry,) {
    if (!entry.isDirectory())
      return [entry.name,];
    /**
     Names directly inside one root directory.
     */
    const inner = await readdir(join(
      directory,
      entry.name,
    ),);
    return [
      entry.name,
      ...inner.map(function withParent(name: string,) {
        return `${entry.name}/${name}`;
      },),
    ];
  },),);
  return names.flat()
    .toSorted();
}

/**
 Paths of the sink log files under a destination,
 descending one level and skipping stamps and probe residue.

 @param directory - Destination root to search.

 @returns Absolute paths of every `*.log.jsonl` file.
 */
async function logFiles(directory: string,): Promise<string[]> {
  return (await entryNames(directory,))
    .filter(function isLog(name: string,) {
      return name.endsWith('.log.jsonl',);
    },)
    .map(function toPath(name: string,) {
      return join(
        directory,
        name,
      );
    },);
}

await describe({
  name: 'file sink',
  children: [
    it({
      name: 'exposes callable verify and write methods',
      fn: async () => {
        // The sentinel keeps even an accidental verify away from shared state.
        const sink = createFileSink({ retention: NO_LOG_RETENTION_SWEEP, },);
        expect(typeof sink.verify,).toBe('function',);
        expect(typeof sink.write,).toBe('function',);
        expect(typeof sink.flush,).toBe('function',);
      },
    },),

    it({
      name: 'verify resolves a boolean',
      fn: async () => {
        await using destination = await scratchDestination();
        const sink = createFileSink({
          directory: destination.path,
          retention: NO_LOG_RETENTION_SWEEP,
        },);
        expect(typeof (await sink.verify()),).toBe('boolean',);
      },
    },),

    it({
      name: 'verify reports availability when running in a package with ancestor node_modules',
      fn: async () => {
        // Tests run from within the monorepo, so an ancestor node_modules
        // always exists. `environment: {}` keeps the lookup on the real
        // ancestor path even when the harness redirects
        // MONOCHROMATIC_LOG_DIRECTORY. NO_LOG_RETENTION_SWEEP keeps that
        // shared destination unswept, and a zero-record sink leaves zero
        // files because the writability probe is always removed.
        expect(await createFileSink({
          environment: {},
          retention: NO_LOG_RETENTION_SWEEP,
        },).verify(),).toBe(true,);
      },
    },),

    it({
      name: 'concurrent verify calls on one sink share a single in-flight promise',
      fn: async () => {
        // Regression guard for the race that used to return false to late
        // callers. The memo now lives in the instance closure, not a module
        // global, so a fresh sink exercises it cleanly.
        await using destination = await scratchDestination();
        const sink = createFileSink({
          directory: destination.path,
          retention: NO_LOG_RETENTION_SWEEP,
        },);
        const [
          a,
          b,
          c,
        ] = await Promise.all([
          sink.verify(),
          sink.verify(),
          sink.verify(),
        ],);
        expect(a,).toBe(b,);
        expect(b,).toBe(c,);
      },
    },),

    it({
      name: 'findNodeModulesUp finds the nearest ancestor node_modules',
      fn: async () => {
        const result = await findNodeModulesUp({
          cwd: import.meta.dirname,
          stat,
          dirname,
          join,
        },);
        expect(typeof result,).toBe('string',);
        if ((typeof result) === 'string')
          expect(result.endsWith('node_modules',),).toBe(true,);
      },
    },),

    it({
      name: 'findNodeModulesUp keeps expected missing candidates silent',
      fn: async () => {
        /** Unexpected faults reported during expected missing-path walk. */
        const reports: unknown[] = [];
        /** Exhausted ancestor result. */
        const result = await findNodeModulesUp({
          cwd: import.meta.dirname,
          stat: statAlwaysMissing as unknown as typeof stat,
          dirname,
          join,
          reportError: function collectUnexpectedReport(report,) {
            reports.push(report,);
          },
        },);
        expect(result,).toBe(NO_NODE_MODULES_FOUND,);
        expect(reports,).toEqual([],);
      },
    },),

    it({
      name: 'findNodeModulesUp reports unexpected stat failures',
      fn: async () => {
        /** Unexpected faults reported during denied-path walk. */
        const reports: unknown[] = [];
        await findNodeModulesUp({
          cwd: import.meta.dirname,
          stat: statAlwaysDenied as unknown as typeof stat,
          dirname,
          join,
          reportError: function collectUnexpectedReport(report,) {
            reports.push(report,);
          },
        },);
        expect(reports.length > 0,).toBe(true,);
      },
    },),

    it({
      name: 'write before verify resolves without touching the filesystem',
      fn: async () => {
        // Verification resolves the log path; without it the path stays
        // unset, so write takes the unset-guard early return, resolving as a
        // silent no-op rather than throwing or writing.
        await using destination = await scratchDestination();
        const sink = createFileSink({
          directory: destination.path,
          retention: NO_LOG_RETENTION_SWEEP,
        },);
        const beforeVerify = record({ message: 'before verify', },);
        expect(await sink.write(beforeVerify,),).toBeUndefined();
        expect(await entryNames(destination.path,),).toEqual([],);
      },
    },),

    it({
      name: 'a verified sink accepts records across levels and message shapes in one log file',
      fn: async () => {
        await using destination = await scratchDestination();
        const sink = createFileSink({
          directory: destination.path,
          retention: NO_LOG_RETENTION_SWEEP,
        },);
        await sink.verify();

        const messages = [
          'plain message',
          'Hello 世界 🌍',
          '',
          '{"key": "value", "nested": {"a": 1}}',
          'line1\nline2\nline3',
        ];
        for (const message of messages)
          // oxlint-disable-next-line no-await-in-loop -- sequential appends keep file order deterministic
          await sink.write(record({ message, },),);

        // One process keeps one log file under its day directory.
        const entries = await entryNames(destination.path,);
        expect(entries,).toHaveLength(2,);
        const [
          dayName,
          logEntry,
        ] = entries;
        expect(dayName ?? '',).toHaveLength(10,);
        expect((logEntry ?? '').startsWith(`${dayName ?? ''}/${dayName ?? ''}T`,),).toBe(true,);
        expect((logEntry ?? '').endsWith('.log.jsonl',),).toBe(true,);
        const files = await logFiles(destination.path,);
        expect(files,).toHaveLength(1,);
        const [logPath,] = files;
        if (logPath === undefined)
          throw new Error('Verified file sink did not create its log file',);
        const rows = (await readFile(
          logPath,
          'utf8',
        )).trimEnd()
          .split('\n',);
        expect(rows.map(function parse(row: string,): unknown {
          return JSON.parse(row,);
        },),).toEqual(messages.map(function toRecord(message: string,) {
          return record({ message, },);
        },),);
      },
    },),

    it({
      name: 'creates no files at verification and one day-directory log with the first record',
      fn: async () => {
        await using destination = await scratchDestination();
        const sink = createFileSink({
          directory: destination.path,
          retention: NO_LOG_RETENTION_SWEEP,
        },);
        expect(await sink.verify(),).toBe(true,);
        // Verification proves writability through a probe round-trip that is
        // always removed, so a zero-record process leaves zero files.
        expect(await entryNames(destination.path,),).toEqual([],);
        await sink.write(record({ message: 'first record', },),);
        expect(await logFiles(destination.path,),).toHaveLength(1,);
        expect((await entryNames(destination.path,)).filter(function probeResidue(name: string,) {
          return name.endsWith('.probe.tmp',);
        },),).toEqual([],);
      },
    },),

    it({
      name: 'sweeps at verification and leaves only the retention stamp when no record is written',
      fn: async () => {
        await using destination = await scratchDestination();
        const sink = createFileSink({
          directory: destination.path,
          retention: {
            maxAgeDays: 14,
            maxBytes: DEFAULT_LOG_MAX_BYTES,
            maxFiles: DEFAULT_LOG_MAX_FILES,
          },
        },);
        expect(await sink.verify(),).toBe(true,);
        await sink.flush?.();
        expect(await entryNames(destination.path,),).toEqual(['.logger-retention-sweep',],);
      },
    },),

    it({
      name: 'leaves the destination untouched with NO_LOG_RETENTION_SWEEP',
      fn: async () => {
        await using destination = await scratchDestination();
        const sink = createFileSink({
          directory: destination.path,
          retention: NO_LOG_RETENTION_SWEEP,
        },);
        expect(await sink.verify(),).toBe(true,);
        await sink.flush?.();
        expect(await entryNames(destination.path,),).toEqual([],);
      },
    },),

    it({
      name: 'MONOCHROMATIC_LOG_FILE=off disables verify and writes even with an explicit directory',
      fn: async ({ sinon, },) => {
        await using destination = await scratchDestination();
        const directory = join(
          destination.path,
          'explicit',
        );
        const sink = createFileSink({
          environment: { MONOCHROMATIC_LOG_FILE: 'off', },
          directory,
        },);
        /** A record that cannot serialize proves writes stop before JSON. */
        const cyclic: LogRecord & { self?: unknown; } = {
          level: 'info',
          timestamp: 0,
          message: 'dropped',
        };
        cyclic.self = cyclic;
        const warnings = sinon.stub(console, 'warn',);
        expect(await sink.verify(),).toBe(false,);
        expect(await sink.write(cyclic,),).toBeUndefined();
        expect(warnings.called,).toBe(false,);
        // The disabled sink never touches the filesystem, so the explicit
        // destination is never even created.
        expect((await entryNames(destination.path,)).includes('explicit',),).toBe(false,);
      },
    },),

    it({
      name: 'reads the kill switch from the injectable environment object',
      fn: async () => {
        await using destination = await scratchDestination();
        const sink = createFileSink({
          environment: {
            MONOCHROMATIC_LOG_FILE: 'off',
            MONOCHROMATIC_LOG_DIRECTORY: destination.path,
          },
        },);
        expect(await sink.verify(),).toBe(false,);
        expect(await entryNames(destination.path,),).toEqual([],);
      },
    },),

    it({
      name: 'prefers the explicit directory parameter over the environment destination',
      fn: async () => {
        await using destination = await scratchDestination();
        const explicit = join(
          destination.path,
          'explicit',
        );
        const configured = join(
          destination.path,
          'configured',
        );
        const sink = createFileSink({
          environment: { MONOCHROMATIC_LOG_DIRECTORY: configured, },
          directory: explicit,
          retention: NO_LOG_RETENTION_SWEEP,
        },);
        expect(await sink.verify(),).toBe(true,);
        await sink.write(record({ message: 'explicit wins', },),);
        expect(await logFiles(explicit,),).toHaveLength(1,);
        /** The environment destination is never even created. */
        expect(existsSync(configured,),).toBe(false,);
      },
    },),

    it({
      name: 'uses MONOCHROMATIC_LOG_DIRECTORY when no directory parameter is given',
      fn: async () => {
        await using destination = await scratchDestination();
        const configured = join(
          destination.path,
          'configured',
        );
        const sink = createFileSink({
          environment: { MONOCHROMATIC_LOG_DIRECTORY: configured, },
          retention: NO_LOG_RETENTION_SWEEP,
        },);
        expect(await sink.verify(),).toBe(true,);
        await sink.write(record({ message: 'environment destination', },),);
        expect(await logFiles(configured,),).toHaveLength(1,);
      },
    },),
  ],
},);
