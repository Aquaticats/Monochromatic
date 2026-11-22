/**
 Tests of the built `mvm` executable against a stand-in `virsh` and a stand-in
 `systemd-run`: whether it starts libvirt's session daemon in a user unit of its
 own before its first `virsh` call, what it does while a unit of the same name is
 still loaded, what it falls back to when it has no user manager, and what it
 says when that fails.

 Each test has its own sandbox, so the tests run side by side.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  MEMINFO_PATH,
  SESSION_DAEMON_IDLE_TIMEOUT_SECONDS,
  SESSION_DAEMON_UNIT,
} from '../dist/final/node/index.mjs';
import {
  createSandbox,
  FAKE_VIRSH_PATH,
  fakeVirshEnv,
  installFakeFlatpak,
  installFakeSystemdRun,
  runCli,
  systemdRunCalls,
  writeExecutable,
} from './fixture.cli-harness.ts';

//region Fixtures

/**
 Domains the fake virsh lists: one managed by mvm and one that is not.
 */
const DOMAINS = [
  {
    name: 'mvm-dev',
    state: 'running',
  },
  {
    name: 'someone-elses-vm',
    state: 'shut off',
  },
];

/**
 Path of the troubleshooting document the fallback warning must name.
 */
const TROUBLESHOOTING_DOC =
  'doc/troubleshooting/libvirt-session-daemon-runs-in-the-capped-scope-of-whichever-command-starts-it.md';

/**
 Variable that shortens the wait for a started daemon, so a test can watch it give up.
 */
const START_TIMEOUT_ENV = 'MVM_SESSION_DAEMON_START_TIMEOUT_MS';

/**
 Milliseconds the tests let mvm wait for a daemon that never answers.
 */
const SHORT_START_TIMEOUT_MS = 800;

/**
 Text the stand-in `systemd-run` writes when it stands for a user manager the
 caller cannot reach, as recorded here on 2026-11-21 from a caller with a private
 `XDG_RUNTIME_DIR`.
 */
const UNREACHABLE_BUS_TEXT =
  'Failed to connect to user scope bus via local transport: No such file or directory';

/**
 Derives the cap mvm must give the unit from this machine's own total memory,
 the test's own way, so the cap is checked against the machine rather than
 against mvm's reader.

 @returns Cap in the form systemd's `MemoryMax=` reads

 @example
 ```ts
 await hostHalfMemoryMax(); // => '31576M' on a machine with 61.7 GiB
 ```
 */
async function hostHalfMemoryMax(): Promise<string> {
  /**
   Content of this machine's memory file.
   */
  const text = await readFile(
    MEMINFO_PATH,
    'utf8',
  );
  /**
   The `MemTotal` line of that file, read without any mvm code.
   */
  const totalLine = text
    .split('\n',)
    .find(function isTotal(line,) {
      return line.startsWith('MemTotal:',);
    },);
  /**
   Its number of kibibytes, from the field after the key.
   */
  const kib = Number(totalLine
    ?.slice('MemTotal:'.length,)
    .trim()
    .split(' ',)[0],);
  return `${String(Math.floor(kib / 2 / 1_024,),)}M`;
}

/**
 Half of this machine's total memory, which the unit's cap must equal.
 */
const MEMORY_MAX = await hostHalfMemoryMax();

/**
 Arguments the stand-in `systemd-run` must receive before the daemon command.

 @param daemon - Command that runs `virtqemud`, then its own arguments

 @returns Every argument after the executable name

 @example
 ```ts
 daemonStartArgs({ daemon: ['virtqemud'] }); // => ['--user', '--unit=mvm-libvirt-session', ..., 'virtqemud', '--timeout=120']
 ```
 */
function daemonStartArgs({ daemon, }: { readonly daemon: readonly string[]; },): readonly string[] {
  return [
    '--user',
    `--unit=${SESSION_DAEMON_UNIT}`,
    '--collect',
    `--property=MemoryMax=${MEMORY_MAX}`,
    '--property=OOMPolicy=continue',
    '--property=ExitType=cgroup',
    ...daemon,
    `--timeout=${String(SESSION_DAEMON_IDLE_TIMEOUT_SECONDS,)}`,
  ];
}

/**
 Command the fake virsh's own path becomes when mvm turns it into the daemon command.
 */
const DERIVED_DAEMON = [
  process.execPath,
  join(
    import.meta.dirname,
    'fixture.fake-virtqemud.ts',
  ),
];

//endregion Fixtures

await describe({
  name: 'mvm and libvirt\'s session daemon',
  children: [
    //region Starting the daemon

    it({
      name: 'starts the daemon in the user unit mvm-libvirt-session when none answers',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'absent',
        },);
        await installFakeSystemdRun({ sandbox, },);
        const result = await runCli({
          args: ['list',],
          env: fakeVirshEnv(sandbox,),
          sandbox,
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toContain('dev',);
        const starts = await systemdRunCalls(sandbox,);
        expect(starts,).toHaveLength(1,);
        expect(starts[0]?.accepted,).toBe(true,);
        expect(starts[0]?.args,).toEqual(daemonStartArgs({ daemon: DERIVED_DAEMON, },),);
      },
    },),

    it({
      name: 'caps the unit at half of this machine\'s total memory',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'absent',
        },);
        await installFakeSystemdRun({ sandbox, },);
        await runCli({
          args: ['list',],
          env: fakeVirshEnv(sandbox,),
          sandbox,
        },);
        const starts = await systemdRunCalls(sandbox,);
        expect(starts[0]?.args,).toContain(`--property=MemoryMax=${MEMORY_MAX}`,);
      },
    },),

    it({
      name: 'starts nothing when a session daemon already answers',
      fn: async () => {
        await using sandbox = await createSandbox({ domains: DOMAINS, },);
        await installFakeSystemdRun({ sandbox, },);
        const result = await runCli({
          args: ['list',],
          env: fakeVirshEnv(sandbox,),
          sandbox,
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toContain('dev',);
        expect(await systemdRunCalls(sandbox,),).toHaveLength(0,);
        // The first call asks whether a daemon answers; the second is the listing. Both ask the same question of libvirt.
        expect(await sandbox.calls(),).toEqual([
          [
            '--connect',
            'qemu:///session',
            'list',
            '--all',
          ],
          [
            '--connect',
            'qemu:///session',
            'list',
            '--all',
          ],
        ],);
      },
    },),

    it({
      name: 'starts one unit when two processes find no daemon at the same time',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'absent',
        },);
        await installFakeSystemdRun({ sandbox, },);
        const [first, second,] = await Promise.all([
          runCli({
            args: [
              '--verbose',
              'list',
            ],
            env: fakeVirshEnv(sandbox,),
            sandbox,
          },),
          runCli({
            args: [
              '--verbose',
              'list',
            ],
            env: fakeVirshEnv(sandbox,),
            sandbox,
          },),
        ],);
        expect(first.exitCode,).toBe(0,);
        expect(second.exitCode,).toBe(0,);
        // The one that lost the unit name says so and waits, rather than exiting or starting a second unit.
        const losers = [
          first,
          second,
        ].filter(function waited(result,) {
          return result.stderr
            .includes('is still loaded; waiting for it to go inactive',);
        },);
        expect(losers,).toHaveLength(1,);
      },
    },),

    it({
      name: 'runs the Flatpak\'s virtqemud in the unit when virsh runs inside the Flatpak',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'absent',
        },);
        await installFakeFlatpak(sandbox,);
        await installFakeSystemdRun({ sandbox, },);
        const result = await runCli({
          args: ['list',],
          env: { MVM_FAKE_VIRSH_DIR: sandbox.state, },
          sandbox,
        },);
        expect(result.exitCode,).toBe(0,);
        const starts = await systemdRunCalls(sandbox,);
        expect(starts[0]?.args,).toEqual(daemonStartArgs({
          daemon: [
            'flatpak',
            'run',
            '--command=virtqemud',
            'org.virt_manager.virt-manager',
          ],
        },),);
      },
    },),

    it({
      name: 'takes the daemon command from MVM_VIRTQEMUD_COMMAND when it is set',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'absent',
        },);
        await installFakeSystemdRun({ sandbox, },);
        const result = await runCli({
          args: ['list',],
          env: {
            ...fakeVirshEnv(sandbox,),
            MVM_VIRTQEMUD_COMMAND: JSON.stringify([
              '/opt/libvirt/sbin/virtqemud-wrapper',
              '--verbose',
            ],),
          },
          sandbox,
        },);
        expect(result.exitCode,).toBe(0,);
        const starts = await systemdRunCalls(sandbox,);
        expect(starts[0]?.args,).toEqual(daemonStartArgs({
          daemon: [
            '/opt/libvirt/sbin/virtqemud-wrapper',
            '--verbose',
          ],
        },),);
      },
    },),

    //endregion Starting the daemon

    //region Starting it again

    it({
      name: 'starts the unit again when the daemon exits between two virsh calls',
      fn: async () => {
        await using sandbox = await createSandbox({
          daemonStopsAfterDomainCall: true,
          sessionDaemon: 'absent',
        },);
        await installFakeSystemdRun({ sandbox, },);
        const result = await runCli({
          args: [
            'destroy',
            'dev',
          ],
          env: fakeVirshEnv(sandbox,),
          sandbox,
        },);
        expect(result.exitCode,).toBe(0,);
        const starts = await systemdRunCalls(sandbox,);
        expect(
          starts.filter(function claimed(call,) {
            return call.accepted;
          },),
        ).toHaveLength(2,);
        // The failed call is repeated once the daemon is back, not handed to virsh to start on demand.
        expect(
          (await sandbox.calls()).filter(function isUndefine(call,) {
            return call.includes('undefine',);
          },),
        ).toHaveLength(2,);
      },
    },),

    it({
      name: 'waits for a unit name that is still loaded and starts it again',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'absent',
        },);
        // The first start finds the name of a unit that is still going away; the second finds it free.
        await installFakeSystemdRun({
          outcomes: [
            'refused',
            'accepted',
          ],
          sandbox,
        },);
        const result = await runCli({
          args: ['list',],
          env: fakeVirshEnv(sandbox,),
          sandbox,
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toContain('dev',);
        const starts = await systemdRunCalls(sandbox,);
        expect(starts,).toHaveLength(2,);
        expect(starts[0]?.accepted,).toBe(false,);
        expect(starts[1]?.accepted,).toBe(true,);
      },
    },),

    it({
      name: 'says the unit name stayed loaded when no start ever claimed it',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'absent',
        },);
        // Every attempt inside the shortened deadline finds the name of a unit still loaded.
        await installFakeSystemdRun({
          outcomes: [
            'refused',
            'refused',
            'refused',
            'refused',
            'refused',
            'refused',
            'refused',
            'refused',
          ],
          sandbox,
        },);
        const result = await runCli({
          args: ['list',],
          env: {
            ...fakeVirshEnv(sandbox,),
            [START_TIMEOUT_ENV]: String(SHORT_START_TIMEOUT_MS,),
          },
          sandbox,
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain('still loaded every time mvm tried to start it',);
        expect(result.stderr,).toContain(`systemctl --user reset-failed ${SESSION_DAEMON_UNIT}.service`,);
      },
    },),

    //endregion Starting it again

    //region What it does without a user manager

    it({
      name: 'lets virsh start the daemon and warns when there is no systemd-run',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'on-demand',
        },);
        const result = await runCli({
          args: ['list',],
          env: fakeVirshEnv(sandbox,),
          sandbox,
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stderr,).toContain('mvm found no systemd-run on PATH',);
        expect(result.stderr,).toContain(TROUBLESHOOTING_DOC,);
      },
    },),

    it({
      name: 'lets virsh start the daemon and warns when the user manager cannot be reached',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'on-demand',
        },);
        await writeExecutable({
          body: [
            `process.stderr.write(${JSON.stringify(`${UNREACHABLE_BUS_TEXT}\n`,)});`,
            'process.exitCode = 1;',
          ].join('\n',),
          name: 'systemd-run',
          sandbox,
        },);
        const result = await runCli({
          args: ['list',],
          env: fakeVirshEnv(sandbox,),
          sandbox,
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stderr,).toContain(`mvm could not reach systemd's user manager`,);
        expect(result.stderr,).toContain('XDG_RUNTIME_DIR',);
        expect(result.stderr,).toContain(TROUBLESHOOTING_DOC,);
      },
    },),

    //endregion What it does without a user manager

    //region What it says when the daemon does not come up

    it({
      name: 'names the unit and its journal when the started daemon never answers',
      fn: async () => {
        await using sandbox = await createSandbox({ sessionDaemon: 'never', },);
        await installFakeSystemdRun({ sandbox, },);
        const result = await runCli({
          args: ['list',],
          env: {
            ...fakeVirshEnv(sandbox,),
            [START_TIMEOUT_ENV]: String(SHORT_START_TIMEOUT_MS,),
          },
          sandbox,
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain(`mvm-libvirt-session.service`,);
        expect(result.stderr,).toContain(`journalctl --user-unit=${SESSION_DAEMON_UNIT}.service`,);
        expect(result.stderr,).toContain(`systemctl --user status ${SESSION_DAEMON_UNIT}.service`,);
        expect(result.stderr,).toContain('virtqemud --verbose',);
      },
    },),

    //endregion What it says when the daemon does not come up

    //region What it says when it cannot tell what the daemon command is

    it({
      name: 'says how to name the daemon command when nothing in the virsh command says virsh',
      fn: async () => {
        await using sandbox = await createSandbox({ sessionDaemon: 'absent', },);
        await writeExecutable({
          body: [
            `process.env.MVM_FAKE_VIRSH_DIR = ${JSON.stringify(
              sandbox.state,
            )};`,
            `import(${JSON.stringify(
              pathToFileURL(FAKE_VIRSH_PATH,)
                .href,
            )});`,
          ].join('\n',),
          name: 'vm-host-client',
          sandbox,
        },);
        const result = await runCli({
          args: ['list',],
          env: {
            MVM_VIRSH_COMMAND: JSON.stringify([
              join(
                sandbox.bin,
                'vm-host-client',
              ),
            ],),
          },
          sandbox,
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain('mvm cannot tell which command runs libvirt\'s session daemon',);
        expect(result.stderr,).toContain('MVM_VIRTQEMUD_COMMAND',);
      },
    },),

    //endregion What it says when it cannot tell what the daemon command is
  ],
},);
