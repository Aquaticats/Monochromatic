/**
 Tests of the built `mvm` executable against a stand-in `virsh` and a stand-in
 `systemd-run`: whether it starts libvirt's session daemon in a user unit of its
 own before its first `virsh` call, and what it says when that fails.

 Each test has its own sandbox, so the tests run side by side.

 @module
 */

import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

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
 Name mvm gives the unit that runs libvirt's session daemon, without `.service`.
 */
const UNIT = 'mvm-libvirt-session';

/**
 Memory cap mvm gives that unit.
 */
const MEMORY_MAX = '36G';

/**
 Seconds mvm lets the session daemon stay idle before it exits.
 */
const IDLE_TIMEOUT_SECONDS = 120;

/**
 Variable that shortens the wait for a started daemon, so a test can watch it give up.
 */
const START_TIMEOUT_ENV = 'MVM_SESSION_DAEMON_START_TIMEOUT_MS';

/**
 Milliseconds the tests let mvm wait for a daemon that never answers.
 */
const SHORT_START_TIMEOUT_MS = 800;

/**
 Arguments the stand-in `systemd-run` must receive before the daemon command.

 @param daemon - Command that runs `virtqemud`, then its own arguments

 @returns Every argument after the executable name

 @example
 ```ts
 daemonStartArgs(['virtqemud']); // => ['--user', '--unit=mvm-libvirt-session', ..., 'virtqemud', '--timeout=120']
 ```
 */
function daemonStartArgs(daemon: readonly string[],): readonly string[] {
  return [
    '--user',
    `--unit=${UNIT}`,
    '--collect',
    `--property=MemoryMax=${MEMORY_MAX}`,
    '--property=OOMPolicy=continue',
    ...daemon,
    `--timeout=${String(IDLE_TIMEOUT_SECONDS,)}`,
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
        await installFakeSystemdRun(sandbox,);
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
        expect(starts[0]?.args,).toEqual(daemonStartArgs(DERIVED_DAEMON,),);
      },
    },),

    it({
      name: 'starts nothing when a session daemon already answers',
      fn: async () => {
        await using sandbox = await createSandbox({ domains: DOMAINS, },);
        await installFakeSystemdRun(sandbox,);
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
        await installFakeSystemdRun(sandbox,);
        const [first, second,] = await Promise.all([
          runCli({
            args: ['list',],
            env: fakeVirshEnv(sandbox,),
            sandbox,
          },),
          runCli({
            args: ['list',],
            env: fakeVirshEnv(sandbox,),
            sandbox,
          },),
        ],);
        expect(first.exitCode,).toBe(0,);
        expect(second.exitCode,).toBe(0,);
        const starts = await systemdRunCalls(sandbox,);
        expect(
          starts.filter(function claimed(call,) {
            return call.accepted;
          },),
        ).toHaveLength(1,);
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
        await installFakeSystemdRun(sandbox,);
        const result = await runCli({
          args: ['list',],
          env: { MVM_FAKE_VIRSH_DIR: sandbox.state, },
          sandbox,
        },);
        expect(result.exitCode,).toBe(0,);
        const starts = await systemdRunCalls(sandbox,);
        expect(starts[0]?.args,).toEqual(daemonStartArgs([
          'flatpak',
          'run',
          '--command=virtqemud',
          'org.virt_manager.virt-manager',
        ],),);
      },
    },),

    it({
      name: 'takes the daemon command from MVM_VIRTQEMUD_COMMAND when it is set',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'absent',
        },);
        await installFakeSystemdRun(sandbox,);
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
        expect(starts[0]?.args,).toEqual(daemonStartArgs([
          '/opt/libvirt/sbin/virtqemud-wrapper',
          '--verbose',
        ],),);
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
        await installFakeSystemdRun(sandbox,);
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

    //endregion Starting it again

    //region What it says when the daemon does not come up

    it({
      name: 'names the unit and its journal when the started daemon never answers',
      fn: async () => {
        await using sandbox = await createSandbox({ sessionDaemon: 'never', },);
        await installFakeSystemdRun(sandbox,);
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
        expect(result.stderr,).toContain(`journalctl --user-unit=${UNIT}.service`,);
        expect(result.stderr,).toContain(`systemctl --user status ${UNIT}.service`,);
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
