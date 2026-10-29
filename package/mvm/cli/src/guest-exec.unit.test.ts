/**
 Tests for running a command in a guest through the QEMU guest agent.

 The seam is the package's `exec` operation as built, over a stand-in `virsh`
 executable (`fixture.fake-virsh.ts`) that simulates the guest agent's process
 table. A disposable home directory keeps the tests away from real VM data.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import {
  delimiter,
  join,
} from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

//region Disposable home: the package derives its data directory from the home directory when it loads

/**
 Home directory for this test process; the package under test is loaded after it is set.
 */
const home = await mkdtemp(join(
  tmpdir(),
  'mvm-guest-exec-home-',
),);
process.env.HOME = home;

const {
  asciiJson,
  DEFAULT_GUEST_EXEC_LIMITS,
  exec,
  GuestExecAttributionError,
  GuestExecLaunchError,
  GuestExecOutputTooLargeError,
  GuestExecStatusUnavailableError,
  runGuestCommand,
} = await import('../dist/final/node/index.mjs');

/**
 Writes metadata for a Windows VM into the disposable home, so `exec` selects PowerShell.
 */
async function writeWindowsMeta(name: string,): Promise<void> {
  const vmDirectory = join(
    home,
    '.local',
    'share',
    'mvm',
    'vms',
    name,
  );
  await mkdir(
    vmDirectory,
    { recursive: true, },
  );
  await writeFile(
    join(
      vmDirectory,
      'meta.json',
    ),
    JSON.stringify({
      createdAt: '2026-10-05T00:00:00.000Z',
      defaultUser: 'Administrator',
      image: 'windows',
      osFamily: 'windows',
      shell: 'powershell.exe',
    },),
  );
}

/**
 Whether `character` is an ASCII character below DEL, the only kind `asciiJson` may emit.
 */
function isPlainAscii(character: string,): boolean {
  return (character.codePointAt(0,) ?? 0) <= 0x7E;
}

/**
 Runs `operation` and returns what it threw; fails when it did not throw.
 */
async function caught(operation: () => Promise<unknown>,): Promise<unknown> {
  try {
    await operation();
  }
  catch (error) {
    return error;
  }
  throw new Error('expected the operation to throw',);
}

//endregion Disposable home

//region Fake virsh

/**
 Text libvirt printed when one status poll outlasted the agent timeout,
 recorded in `doc/handover/scanner-native-verification.md`, section `Windows virtual machine and bridges`.
 */
const AGENT_TIMEOUT_STDERR =
  'error: guest agent command timed out: guest agent didn\'t respond to command within \'5\' seconds';

/**
 Handle on one installed fake `virsh`: reads back the calls it received and removes it.
 */
type FakeVirsh = AsyncDisposable & {
  readonly calls: () => Promise<readonly (readonly string[])[]>;
};

/**
 Puts a fake `virsh` first on `PATH`, driven by `scenario`.
 */
async function installFakeVirsh(scenario: Readonly<Record<string, unknown>>,): Promise<FakeVirsh> {
  const directory = await mkdtemp(join(
    tmpdir(),
    'mvm-fake-virsh-',
  ),);
  await writeFile(
    join(
      directory,
      'scenario.json',
    ),
    JSON.stringify(scenario,),
  );
  await writeFile(
    join(
      directory,
      'virsh',
    ),
    [
      '#!/usr/bin/env node',
      `process.env.MVM_FAKE_VIRSH_DIR = ${JSON.stringify(directory,)};`,
      `import(${
        JSON.stringify(pathToFileURL(join(
          import.meta.dirname,
          'fixture.fake-virsh.ts',
        ),).href,)
      });`,
      '',
    ].join('\n',),
    { mode: 0o700, },
  );
  const priorPath = process.env.PATH;
  process.env.PATH = [
    directory,
    ...(priorPath === undefined ? [] : [priorPath,]),
  ].join(delimiter,);
  return {
    async calls() {
      const log = await readFile(
        join(
          directory,
          'calls.jsonl',
        ),
        'utf8',
      );
      return log
        .split('\n',)
        .filter((line,) => line !== '')
        .map((line,) => JSON.parse(line,) as readonly string[]);
    },
    async [Symbol.asyncDispose]() {
      if (priorPath === undefined)
        Reflect.deleteProperty(process.env, 'PATH',);
      else
        process.env.PATH = priorPath;
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

//endregion Fake virsh

await describe({
  name: exec.name,
  // Sequential: every test replaces the process-wide PATH.
  concurrency: 1,
  children: [
    //region Slow status poll

    it({
      name: 'returns the command result when one status poll times out while the command still runs',
      fn: async () => {
        await using _fake = await installFakeVirsh({
          runningPolls: 1,
          statusFailures: [AGENT_TIMEOUT_STDERR,],
        },);
        const result = await exec({
          command: 'printf \'still here\'',
          name: 'slow-poll',
        },);
        expect(result,).toEqual({
          exitCode: 0,
          stderr: '',
          stdout: 'still here',
        },);
      },
    },),

    //endregion Slow status poll

    //region Reused process ID

    it({
      name: 'returns the started command\'s own result when the agent still holds an older result for the same process ID',
      fn: async () => {
        await using _fake = await installFakeVirsh({
          pids: [2_440,],
          staleEntries: [
            {
              exitcode: 0,
              pid: 2_440,
              runningPolls: 0,
              stderr: '',
              stdout: 'output of an earlier command\n',
            },
          ],
        },);
        const result = await exec({
          command: 'printf \'fresh output\'; printf \'fresh error\' >&2; exit 3',
          name: 'reused-pid',
        },);
        expect(result,).toEqual({
          exitCode: 3,
          stderr: 'fresh error',
          stdout: 'fresh output',
        },);
      },
    },),

    it({
      name: 'refuses to return a result when the agent forgets the process ID after reporting only an older result',
      fn: async () => {
        await using fake = await installFakeVirsh({
          loseLaunchedEntry: true,
          pids: [2_440,],
          staleEntries: [
            {
              exitcode: 0,
              pid: 2_440,
              runningPolls: 0,
              stderr: '',
              stdout: 'output of an earlier command\n',
            },
          ],
        },);
        const error = await caught(() =>
          exec({
            command: 'printf \'fresh output\'',
            name: 'unattributable',
          },)
        );
        expect(error,).toBeInstanceOf(GuestExecAttributionError,);
        const attribution = error as InstanceType<typeof GuestExecAttributionError>;
        expect(attribution.pid,).toBe(2_440,);
        expect(attribution.discarded,).toEqual([
          {
            exitCode: 0,
            stderr: '',
            stdout: 'output of an earlier command\n',
          },
        ],);
        expect(attribution.message,).toContain('guest process 2440',);
        expect(attribution.message,).toContain('output of an earlier command',);
        // One launch, the status read that returned the older result, and the read that found nothing.
        expect((await fake.calls()).length,).toBe(3,);
      },
    },),

    it({
      name: 'reports a lost result when the agent knows no result for the started process',
      fn: async () => {
        await using _fake = await installFakeVirsh({ loseLaunchedEntry: true, },);
        const error = await caught(() =>
          exec({
            command: 'true',
            name: 'lost-result',
          },)
        );
        expect(error,).toBeInstanceOf(GuestExecAttributionError,);
        expect((error as InstanceType<typeof GuestExecAttributionError>).discarded,).toEqual([],);
        expect((error as Error).message,).toContain('reported no finished result',);
      },
    },),

    //endregion Reused process ID

    //region Marker and shell behavior

    it({
      name: 'ties a command with a shell syntax error to its own failing result',
      fn: async () => {
        await using _fake = await installFakeVirsh({},);
        const result = await exec({
          command: 'if then',
          name: 'syntax-error',
        },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stdout,).toBe('',);
        expect(result.stderr,).toContain('syntax error',);
      },
    },),

    it({
      name: 'reports the shell convention exit status for a command a signal ended',
      fn: async () => {
        await using _fake = await installFakeVirsh({},);
        const result = await exec({
          command: 'kill -KILL $$',
          name: 'signalled',
        },);
        expect(result.exitCode,).toBe(137,);
      },
    },),

    it({
      name: 'passes a command with non-ASCII text through an ASCII-only argument and returns its UTF-8 output',
      fn: async () => {
        await using fake = await installFakeVirsh({},);
        const result = await exec({
          command: 'printf \'%s\' \'h\u00E9llo \u6F22\u5B57 \u{1F600}\'',
          name: 'unicode',
        },);
        expect(result.stdout,).toBe('h\u00E9llo \u6F22\u5B57 \u{1F600}',);
        const launch = (await fake.calls())[0]?.at(-1,) ?? '';
        expect(launch,).toContain('guest-exec',);
        expect(Array.from(launch,).every(isPlainAscii,),).toBe(true,);
      },
    },),

    it({
      name: 'runs a Windows command through PowerShell and strips the marker line with its CRLF',
      fn: async () => {
        await writeWindowsMeta('win',);
        await using fake = await installFakeVirsh({},);
        const result = await exec({
          command: '\'hello from windows\'',
          name: 'win',
        },);
        expect(result,).toEqual({
          exitCode: 0,
          stderr: '',
          stdout: 'hello from windows\r\n',
        },);
        const launch = JSON.parse((await fake.calls())[0]?.at(-1,) ?? '{}',) as {
          arguments: { path: string; arg: string[]; };
        };
        expect(launch.arguments.path,).toBe('powershell.exe',);
        expect(launch.arguments.arg.slice(
          0,
          3,
        ),).toEqual([
          '-NoProfile',
          '-NonInteractive',
          '-Command',
        ],);
      },
    },),

    it({
      name: 'shows the shell error when a PowerShell text does not parse and so prints no marker',
      fn: async () => {
        await writeWindowsMeta('win-parse',);
        await using _fake = await installFakeVirsh({},);
        const error = await caught(() =>
          exec({
            command: 'Get-Thing | <<does-not-parse>>',
            name: 'win-parse',
          },)
        );
        expect(error,).toBeInstanceOf(GuestExecAttributionError,);
        expect((error as Error).message,).toContain('ParserError',);
      },
    },),

    //endregion Marker and shell behavior

    //region Status no longer readable

    it({
      name: 'stops at the first unanswered status request when libvirt no longer knows the domain',
      fn: async () => {
        await using fake = await installFakeVirsh({
          domstate: { failure: 'error: failed to get domain \'mvm-vanished\'', },
          statusFailures: ['error: failed to get domain \'mvm-vanished\'',],
        },);
        const error = await caught(() =>
          exec({
            command: 'true',
            name: 'vanished',
          },)
        );
        expect(error,).toBeInstanceOf(GuestExecStatusUnavailableError,);
        expect((error as Error).message,).toContain('unknown to libvirt',);
        const statusRequests = (await fake.calls()).filter((call,) =>
          (call.at(-1,) ?? '').includes('guest-exec-status',)
        );
        expect(statusRequests.length,).toBe(1,);
      },
    },),

    it({
      name: 'stops when the domain shut off while the command ran',
      fn: async () => {
        await using _fake = await installFakeVirsh({
          domstate: 'shut off',
          statusFailures: ['error: Requested operation is not valid: domain is not running',],
        },);
        const error = await caught(() =>
          exec({
            command: 'true',
            name: 'powered-off',
          },)
        );
        expect(error,).toBeInstanceOf(GuestExecStatusUnavailableError,);
        expect((error as Error).message,).toContain('the domain is now shut off',);
      },
    },),

    it({
      name: 'gives up with the process ID once status requests stay unanswered past the limit',
      fn: async () => {
        await using _fake = await installFakeVirsh({
          pids: [77,],
          statusFailures: [AGENT_TIMEOUT_STDERR,],
        },);
        const error = await caught(() =>
          runGuestCommand({
            command: 'true',
            domain: 'mvm-silent',
            limits: {
              ...DEFAULT_GUEST_EXEC_LIMITS,
              unresponsiveLimitMs: 0,
            },
            osFamily: 'linux',
            shell: '/bin/sh',
          },)
        );
        expect(error,).toBeInstanceOf(GuestExecStatusUnavailableError,);
        expect((error as InstanceType<typeof GuestExecStatusUnavailableError>).pid,).toBe(77,);
        expect((error as Error).message,).toContain('has not answered a status request',);
      },
    },),

    it({
      name: 'says the output is too large for libvirt when the finished result cannot be handed over, without asking again',
      fn: async () => {
        await using fake = await installFakeVirsh({
          pids: [91,],
          statusFailures: ['error: Unable to encode message payload',],
        },);
        const error = await caught(() =>
          runGuestCommand({
            command: 'cat huge.log',
            domain: 'mvm-verbose',
            osFamily: 'linux',
            shell: '/bin/sh',
          },)
        );
        expect(error,).toBeInstanceOf(GuestExecOutputTooLargeError,);
        expect((error as InstanceType<typeof GuestExecOutputTooLargeError>).pid,).toBe(91,);
        expect((error as Error).message,).toContain('its output is too large for libvirt to hand over',);
        expect((error as Error).message,).toContain('fetch the file with mvm pull',);
        const statusRequests = (await fake.calls()).filter((call,) =>
          (call.at(-1,) ?? '').includes('guest-exec-status',)
        );
        expect(statusRequests.length,).toBe(1,);
      },
    },),

    //endregion Status no longer readable

    //region Launch

    it({
      name: 'says nothing ran when the agent refuses the launch',
      fn: async () => {
        await using _fake = await installFakeVirsh({
          launchFailure:
            'error: guest agent command failed: unable to execute QEMU agent command \'guest-exec\': Guest agent command failed, error was \'Failed to execute child process\'',
        },);
        const error = await caught(() =>
          exec({
            command: 'true',
            name: 'refused',
          },)
        );
        expect(error,).toBeInstanceOf(GuestExecLaunchError,);
        expect((error as InstanceType<typeof GuestExecLaunchError>).mayBeRunning,).toBe(false,);
        expect((error as Error).message,).toContain('nothing ran',);
      },
    },),

    it({
      name: 'says the command may be running when the launch gets no answer',
      fn: async () => {
        await using _fake = await installFakeVirsh({ launchFailure: AGENT_TIMEOUT_STDERR, },);
        const error = await caught(() =>
          exec({
            command: 'true',
            name: 'unconfirmed',
          },)
        );
        expect(error,).toBeInstanceOf(GuestExecLaunchError,);
        expect((error as InstanceType<typeof GuestExecLaunchError>).mayBeRunning,).toBe(true,);
        expect((error as Error).message,).toContain('may or may not be running',);
      },
    },),

    //endregion Launch

    //region asciiJson

    it({
      name: 'asciiJson escapes every non-ASCII character and stays valid JSON',
      fn: async () => {
        const value = { path: 'C:\\d\u00E9p\u00F4t\\\u{1F600} \u007F.txt', };
        const json = asciiJson(value,);
        expect(Array.from(json,).every(isPlainAscii,),).toBe(true,);
        expect(JSON.parse(json,),).toEqual(value,);
        expect(asciiJson({ plain: 'ascii only', },),).toBe('{"plain":"ascii only"}',);
      },
    },),

    //endregion asciiJson
  ],
},);

await rm(
  home,
  {
    recursive: true,
    force: true,
  },
);
