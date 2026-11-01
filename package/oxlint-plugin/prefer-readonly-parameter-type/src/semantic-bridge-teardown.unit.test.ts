import {
  spawn,
  spawnSync,
} from 'node:child_process';
import {
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import {
  fileURLToPath,
  pathToFileURL,
} from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isTerminalSemanticChannelFailure, } from '../dist/final/node/index.mjs';

/**
 Filesystem path of the built bridge entry the fixture children import.
 */
const DIST_PATH = fileURLToPath(new URL(
  '../dist/final/node/index.mjs',
  import.meta.url,
),);
/** Built bridge entry imported by every fixture child. */
const DIST_HREF = pathToFileURL(DIST_PATH,).href;

/** Source fixture resolved through configured package project. */
const FIXTURE_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/valid/typescript-sync-adapter.ts',
  import.meta.url,
),);

/** Child handshake marker printed after the semantic snapshot exists. */
const READY_MARKER = 'READY';

/** Upper bound for a fixture child to finish after the channel dies. */
const CHILD_TIMEOUT_MS = 30_000;

/** Bounded wait for a SIGKILLed native child pid to disappear before the fixture continues. */
const KILLED_PID_GONE_WAIT_MS = 5_000;

/**
 Disposable directory holding one fixture child script.
 */
type ScriptDirectory = {
  readonly path: string;
  [Symbol.dispose]: () => void;
};

/**
 Creates disposable directory for fixture child scripts.

 @returns disposable script directory removed when the enclosing scope ends.
 */
function createScriptDirectory(): ScriptDirectory {
  /** Unique directory path under the host temp directory. */
  const path = mkdtempSync(join(scratchRoot(), 'semantic-bridge-teardown-',),);
  return {
    path,
    [Symbol.dispose]: function removeScriptDirectory(): void {
      rmSync(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Sentinel for a fixture child terminated by a signal instead of an exit code.
 */
const CHILD_KILLED_BY_SIGNAL: unique symbol = Symbol('fixture child terminated without an exit code',);

/**
 Result of one fixture child run.
 */
type ChildRun = {
  readonly exitCode: number | typeof CHILD_KILLED_BY_SIGNAL;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Output and handshake state of one fixture child, accumulated by its stream handlers.

 `handshakeError` holds the failure from the post-ready action, surfaced instead of a timeout.
 */
type ChildOutputCollection = {
  stdout: string;
  stderr: string;
  readySeen: boolean;
  handshakeError?: Error;
};

/**
 Runs one fixture script to completion inside a child Node process.

 When the handshake option is set, the ready marker triggers the incident shape:
 every native TypeScript child is killed and only then may the script continue.

 @param scriptPath - Absolute path of the fixture script.

 @param killNativeChildOnReady - Whether to kill the native children at the ready marker.

 @returns captured exit code and output streams.

 @throws when no native child exists to kill, or the child exceeds the completion deadline.
 */
async function runScriptChild({
  scriptPath,
  killNativeChildOnReady = false,
}: {
  readonly scriptPath: string;
  readonly killNativeChildOnReady?: boolean;
},): Promise<ChildRun> {
  /**
   Child running the fixture script with the fixture source path in its environment.
   */
  const child = spawn(process.execPath, [scriptPath,], {
    cwd: fileURLToPath(new URL('../../..', import.meta.url),),
    env: {
      ...process.env,
      TEARDOWN_FIXTURE_PATH: FIXTURE_PATH,
    },
    stdio: ['pipe', 'pipe', 'pipe',],
  },);
  /**
   Output and handshake state accumulated while the child runs.
   */
  const collected: ChildOutputCollection = {
    stdout: '',
    stderr: '',
    readySeen: false,
  };
  /**
   Kills the native TypeScript children and then lets the script continue.
   */
  function killNativeChildAndContinue(): void {
    /**
     Child processes currently spawned under the fixture process.
     */
    const probe = spawnSync('pgrep', ['--parent', String(child.pid,),], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe',],
    },);
    if ((probe.status !== 0) && (probe.status !== 1))
      throw new Error(`pgrep --parent failed with status ${String(probe.status,)}: ${probe.stderr}`,);
    /**
     Native child pids to kill, excluding the probe's own process.
     */
    const pids = probe.stdout
      .trim()
      .split('\n',)
      .filter((pid,) => (pid.length > 0) && (Number(pid,) !== probe.pid),);
    if (pids.length === 0)
      throw new Error(`no native child under fixture pid ${String(child.pid,)} to kill`,);
    pids.forEach(function killNativePid(pid,): void {
      process.kill(Number(pid,), 'SIGKILL',);
    },);
    // Waits, bounded, until each killed pid is gone so a dying native child cannot answer
    // one more request before the fixture continues (the `tail --pid` wait form).
    pids.forEach(function awaitKilledPidGone(pid,): void {
      spawnSync('tail', [`--pid=${pid}`, '--follow', '/dev/null',], {
        encoding: 'utf8',
        timeout: KILLED_PID_GONE_WAIT_MS,
      },);
    },);
    child.stdin.end('\n',);
  }
  /**
   Runs the post-ready action exactly once.
   */
  function fireReady(): void {
    if (collected.readySeen)
      return;
    collected.readySeen = true;
    if (!killNativeChildOnReady)
      return;
    try {
      killNativeChildAndContinue();
    }
    catch (error) {
      collected.handshakeError = Error.isError(error,)
        ? error
        : new Error(String(error,),);
      child.kill('SIGKILL',);
    }
  }
  child.stdout.on('data', function collectStdout(chunk: Buffer,): void {
    collected.stdout += String(chunk,);
    if (collected.stdout.includes(READY_MARKER,))
      fireReady();
  },);
  child.stderr.on('data', function collectStderr(chunk: Buffer,): void {
    collected.stderr += String(chunk,);
  },);
  /**
   Exit code observed when the child terminates.
   */
  const exitPromise = new Promise<ChildRun>(function awaitExit(resolve, reject,): void {
    child.on('close', function childClosed(): void {
      if (collected.handshakeError !== undefined) {
        reject(collected.handshakeError,);
        return;
      }
      /**
       Exit code Node reports, replaced by the signal sentinel when the child never exited.
       */
      const rawExitCode = child.exitCode;
      resolve({
        exitCode: rawExitCode ?? CHILD_KILLED_BY_SIGNAL,
        stdout: collected.stdout,
        stderr: collected.stderr,
      },);
    },);
  },);
  /**
   Timeout that fails the test instead of hanging on a wedged child.
   */
  const timeoutPromise = new Promise<ChildRun>(function awaitTimeout(_resolve, reject,): void {
    setTimeout(function childTimedOut(): void {
      child.kill('SIGKILL',);
      reject(new Error(`fixture child exceeded ${CHILD_TIMEOUT_MS}ms: stdout=${collected.stdout} stderr=${collected.stderr}`,),);
    }, CHILD_TIMEOUT_MS,)
      .unref();
  },);
  return Promise.race([exitPromise, timeoutPromise,],);
}

await describe({
  name: 'semantic bridge teardown when the native channel dies',
  children: [
    it({
      name: 'classifies terminal channel failures and leaves ordinary errors alone',
      fn: async () => {
        expect(isTerminalSemanticChannelFailure({
          error: {
            message: 'write EPIPE',
            code: 'EPIPE',
          },
        },),).toBe(true,);
        expect(isTerminalSemanticChannelFailure({
          error: {
            message: 'system',
            code: 'EBADF',
          },
        },),).toBe(true,);
        expect(isTerminalSemanticChannelFailure({
          error: {
            message: 'system',
            code: 'EIO',
          },
        },),).toBe(true,);
        expect(isTerminalSemanticChannelFailure({
          error: {
            message: 'destroyed',
            code: 'ERR_STREAM_DESTROYED',
          },
        },),).toBe(true,);
        expect(isTerminalSemanticChannelFailure({
          error: new Error('SyncRpcChannel is closed',),
        },),).toBe(true,);
        expect(isTerminalSemanticChannelFailure({
          error: new Error('Unexpected EOF while reading from child process (killed by signal SIGKILL)',),
        },),).toBe(true,);
        expect(isTerminalSemanticChannelFailure({
          error: new Error('wrapper', {
            cause: {
              message: 'inner',
              code: 'EBADF',
            },
          },),
        },),).toBe(true,);
        expect(isTerminalSemanticChannelFailure({ error: 'EBADF', },),).toBe(true,);
        expect(isTerminalSemanticChannelFailure({ error: new Error('project config missing',), },),).toBe(false,);
        expect(isTerminalSemanticChannelFailure({ error: undefined, },),).toBe(false,);
        /**
         Error whose cause chain loops back to itself, so the walk must stop on the seen set.
         */
        const cyclicError = new Error('cyclic cause chain',);
        cyclicError.cause = cyclicError;
        expect(isTerminalSemanticChannelFailure({ error: cyclicError, },),).toBe(false,);
      },
    },),
    it({
      name: 'exits cleanly from teardown after the native child is killed',
      fn: async () => {
        using scriptDirectory = createScriptDirectory();
        /** Fixture child driving the real bridge until its channel dies. */
        const scriptPath = join(scriptDirectory.path, 'teardown-child.ts',);
        writeFileSync(scriptPath, [
          String.raw`import { closeSemanticBridge, initializeSemanticBridge, openSemanticFile } from '${DIST_HREF}';`,
          String.raw`import { readFileSync } from 'node:fs';`,
          String.raw`const fixturePath = process.env['TEARDOWN_FIXTURE_PATH'];`,
          String.raw`if (fixturePath === undefined)`,
          String.raw`  throw new Error('TEARDOWN_FIXTURE_PATH missing');`,
          String.raw`initializeSemanticBridge();`,
          String.raw`openSemanticFile({ fileName: fixturePath, sourceText: readFileSync(fixturePath, 'utf8'), hasBOM: false },);`,
          String.raw`console.log('${READY_MARKER}');`,
          String.raw`process.stdin.once('data', function proceed(): void {`,
          String.raw`  try {`,
          String.raw`    openSemanticFile({ fileName: fixturePath, sourceText: readFileSync(fixturePath, 'utf8') + '\n// touched by teardown fixture\n', hasBOM: false },);`,
          String.raw`    console.log('SECOND-OPEN-OK');`,
          String.raw`  }`,
          String.raw`  catch (error) {`,
          String.raw`    console.log('SECOND-OPEN-FAILED: ' + String(error).split('\n')[0]);`,
          String.raw`  }`,
          String.raw`  try {`,
          String.raw`    initializeSemanticBridge();`,
          String.raw`    console.log('RESTART-ALLOWED');`,
          String.raw`  }`,
          String.raw`  catch (error) {`,
          String.raw`    const reason = ((typeof error === 'object') && (error !== null) && ('reason' in error))`,
          String.raw`      ? String(error.reason)`,
          String.raw`      : 'no-reason';`,
          String.raw`    console.log('RESTART-BLOCKED: ' + reason);`,
          String.raw`  }`,
          String.raw`  closeSemanticBridge();`,
          String.raw`  console.log('CLOSE-OK');`,
          String.raw`  process.stdin.destroy();`,
          String.raw`});`,
        ].join('\n',),);
        /** Result of the fixture child that lost its native channel mid-request. */
        const run = await runScriptChild({
          scriptPath,
          killNativeChildOnReady: true,
        },);
        expect(run.exitCode,).toBe(0,);
        expect(run.stdout,).toContain('SECOND-OPEN-FAILED: SemanticBridgeError',);
        expect(run.stdout,).toContain('RESTART-BLOCKED: api-unavailable',);
        expect(run.stdout,).not.toContain('RESTART-ALLOWED',);
        expect(run.stdout,).toContain('CLOSE-OK',);
        // Tagged retirement warning is the expected lifecycle output on stderr.
        expect(run.stderr,).toContain('retiring semantic bridge after terminal channel failure',);
        // Crash footer and stack frames from an uncaught teardown error must stay absent.
        expect(run.stderr,).not.toContain('Node.js v',);
        expect(run.stderr,).not.toContain('    at ',);
      },
    },),
    it({
      name: 'fails closed and stays idempotent once the bridge is retired',
      fn: async () => {
        using scriptDirectory = createScriptDirectory();
        /** Fixture child exercising retirement semantics in isolation. */
        const scriptPath = join(scriptDirectory.path, 'retirement-child.ts',);
        writeFileSync(scriptPath, [
          String.raw`import { closeSemanticBridge, initializeSemanticBridge, retireSemanticBridgeAfterChannelFailure } from '${DIST_HREF}';`,
          String.raw`retireSemanticBridgeAfterChannelFailure({ error: new Error('write EPIPE'), },);`,
          String.raw`try {`,
          String.raw`  initializeSemanticBridge();`,
          String.raw`  console.log('RESTART-ALLOWED');`,
          String.raw`}`,
          String.raw`catch (error) {`,
          String.raw`  const reason = ((typeof error === 'object') && (error !== null) && ('reason' in error))`,
          String.raw`    ? String(error.reason)`,
          String.raw`    : 'no-reason';`,
          String.raw`  console.log('RESTART-BLOCKED: ' + reason);`,
          String.raw`}`,
          String.raw`retireSemanticBridgeAfterChannelFailure({ error: new Error('second failure'), },);`,
          String.raw`closeSemanticBridge();`,
          String.raw`console.log('RETIRE-IDEMPOTENT');`,
          String.raw`console.log('CLOSE-OK');`,
        ].join('\n',),);
        /** Result of the fixture child exercising retirement semantics in isolation. */
        const run = await runScriptChild({ scriptPath, },);
        expect(run.exitCode,).toBe(0,);
        expect(run.stdout,).toContain('RESTART-BLOCKED: api-unavailable',);
        expect(run.stdout,).not.toContain('RESTART-ALLOWED',);
        expect(run.stdout,).toContain('RETIRE-IDEMPOTENT',);
        expect(run.stdout,).toContain('CLOSE-OK',);
        // Tagged retirement warning is the expected lifecycle output on stderr.
        expect(run.stderr,).toContain('retiring semantic bridge after terminal channel failure',);
        // Crash footer and stack frames from an uncaught teardown error must stay absent.
        expect(run.stderr,).not.toContain('Node.js v',);
        expect(run.stderr,).not.toContain('    at ',);
      },
    },),
  ],
},);
