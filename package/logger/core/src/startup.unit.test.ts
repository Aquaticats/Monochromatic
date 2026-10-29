import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { text, } from 'node:stream/consumers';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/** Built package entry used by the subprocess probe. */
const BUILT_INDEX_PATH = join(
  import.meta.dirname,
  '..',
  'dist',
  'final',
  'node',
  'index.mjs',
);

/** Message logged before the probe awaits anything from the logger package. */
const STARTUP_MESSAGE = 'startup before consumer init await';

/** JSONL fragment proving the startup message reached the file sink. */
const STARTUP_MESSAGE_FRAGMENT = `"message":${JSON.stringify(STARTUP_MESSAGE,)}`;

/** Debug message used by the process-stream startup probe. */
const DEBUG_MESSAGE = 'debug before consumer init await';

/** Async-disposable temporary project for file-sink startup probes. */
type TempProject = {
  readonly path: string;
  readonly scriptPath: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/** Result captured from the probe subprocess. */
type ProbeResult = {
  readonly exitCode: number;
  readonly stderr: string;
  readonly stdout: string;
};

/**
 Builds a throwaway project root with `node_modules` so the file sink chooses
 an isolated `node_modules/.monochromatic` log directory.
 
 @param logLine - Logger call that the generated probe should execute.
 
 @returns Temporary project handle removed by `await using`.
 */
async function createTempProject(
  { logLine, }: { readonly logLine: string; },
): Promise<TempProject> {
  const scratch = join(homedir(), 'temp/agent',);
  await mkdir(scratch, { recursive: true, mode: 0o700, },);
  const path = await mkdtemp(join(scratch, 'logger-startup-',),);
  await mkdir(
    join(
      path,
      'node_modules',
    ),
    { recursive: true, },
  );

  /** Probe script that never imports or awaits `initPromise`. */
  const scriptPath = join(
    path,
    'probe.ts',
  );
  /** Probe source assembled as separate lines so generated syntax stays readable. */
  const script = [
    `import { logger, } from ${JSON.stringify(BUILT_INDEX_PATH,)};`,
    '',
    logLine,
    'await logger.flush();',
    '',
  ].join('\n',);
  await writeFile(
    scriptPath,
    script,
  );

  return {
    path,
    scriptPath,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Runs a probe script in its temporary project root.
 
 @param cwd - Project root used as `process.cwd()` by the file sink.
 @param env - Environment overrides applied to the probe process.
 @param scriptPath - Absolute path to the probe script.
 
 @returns Captured stdout, stderr, and exit code.
 */
async function runProbe(
  {
    cwd,
    env = {},
    scriptPath,
  }: {
    readonly cwd: string;
    readonly env?: Readonly<Record<string, string>>;
    readonly scriptPath: string;
  },
): Promise<ProbeResult> {
  // Default stdio is 'pipe', which yields a ChildProcessWithoutNullStreams so
  // stdout/stderr are non-null Readables. The probe is a `.ts` file, so it runs
  // through the same Node runtime used by mise test tasks.
  const subprocess = spawn(
    'node',
    [scriptPath,],
    {
      cwd,
      env: {
        ...process.env,
        HOME: cwd,
        MONOCHROMATIC_LOG_DIRECTORY: '',
        ...env,
      },
    },
  );

  const [
    stdout,
    stderr,
  ] = await Promise.all([
    text(subprocess.stdout,),
    text(subprocess.stderr,),
    once(subprocess, 'close',),
  ],);

  return {
    // After 'close' the exit code is populated; null means killed by signal.
    exitCode: subprocess.exitCode ?? (-1),
    stderr,
    stdout,
  };
}

/**
 Reads the single JSONL file created by the probe's file sink.
 
 @param projectPath - Temporary project root containing `node_modules`.
 
 @returns Log file contents.
 */
async function readOnlyLogContent({ projectPath, }: { readonly projectPath: string; },): Promise<string> {
  const logDir = join(
    projectPath,
    'node_modules',
    '.monochromatic',
  );
  /** The root holds one day directory plus the retention stamp; logs sit one level down. */
  const entries = await readdir(
    logDir,
    { withFileTypes: true, },
  );
  const nested = await Promise.all(entries.map(async function collect(entry,) {
    if (!entry.isDirectory())
      return entry.name.endsWith('.log.jsonl',) ? [entry.name,] : [];
    const inner = await readdir(join(
      logDir,
      entry.name,
    ),);
    return inner.filter(function isLog(name: string,) {
      return name.endsWith('.log.jsonl',);
    },)
      .map(function withDay(name: string,) {
        return join(
          entry.name,
          name,
        );
      },);
  },),);
  /** Sink log files collected one level below the destination root. */
  const logFiles = nested.flat();
  expect(logFiles.length,)
    .toBe(1,);

  /** Single log file emitted by one probe process. */
  const [logFile,] = logFiles;
  if (logFile === undefined)
    throw new Error('Expected logger to create a log file.',);

  return await readFile(
    join(
      logDir,
      logFile,
    ),
    'utf8',
  );
}

await describe({
  name: 'logger startup initialization',
  children: [
    it({
      name: 'delivers startup records to file sink without awaiting initPromise',
      fn: async () => {
        await using project = await createTempProject({
          logLine: `logger.info(${JSON.stringify(STARTUP_MESSAGE,)},);`,
        },);

        const result = await runProbe({
          cwd: project.path,
          scriptPath: project.scriptPath,
        },);
        expect(result.exitCode,)
          .toBe(0,);
        expect(result.stderr,)
          .toBe('',);
        expect(result.stdout,)
          .toContain(STARTUP_MESSAGE,);

        const content = await readOnlyLogContent({ projectPath: project.path, },);
        expect(content,)
          .toContain(STARTUP_MESSAGE_FRAGMENT,);
      },
    },),

    it({
      name: 'writes debug startup records to stderr when process stderr is available',
      fn: async () => {
        await using project = await createTempProject({
          logLine: `logger.debug(${JSON.stringify(DEBUG_MESSAGE,)},);`,
        },);

        const result = await runProbe({
          cwd: project.path,
          env: { MONOCHROMATIC_VERBOSE: 'true', },
          scriptPath: project.scriptPath,
        },);
        expect(result.exitCode,)
          .toBe(0,);
        expect(result.stdout,)
          .toBe('',);
        expect(result.stderr,)
          .toContain(DEBUG_MESSAGE,);
      },
    },),
  ],
},);
