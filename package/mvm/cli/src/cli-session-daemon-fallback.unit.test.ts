/**
 Tests of the built `mvm` executable when it has no systemd user manager to start
 libvirt's session daemon in: it lets `virsh` start the daemon as it did before
 and warns, or keeps the old answer when `systemd-run` failed for another reason.

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
  runCli,
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
 Text the stand-in `systemd-run` writes when it stands for a user manager the
 caller cannot reach, as recorded here on 2026-11-21 from a caller with a private
 `XDG_RUNTIME_DIR`.
 */
const UNREACHABLE_BUS_TEXT =
  'Failed to connect to user scope bus via local transport: No such file or directory';

//endregion Fixtures

await describe({
  name: 'mvm without a systemd user manager',
  children: [
    //region Letting virsh start the daemon

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

    it({
      name: 'asks for libvirt\'s own on-demand start even when the caller disabled it',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'on-demand',
        },);
        const result = await runCli({
          args: ['list',],
          env: {
            ...fakeVirshEnv(sandbox,),
            LIBVIRT_AUTOSTART: '0',
          },
          sandbox,
        },);
        // The fake answers only when libvirt may start a daemon, so a clean run shows mvm asked for that.
        expect(result.exitCode,).toBe(0,);
        expect(result.stderr,).toContain('mvm found no systemd-run on PATH',);
      },
    },),

    it({
      name: 'lets virsh start the daemon when the virsh command says nothing about the daemon and there is no systemd-run',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'on-demand',
        },);
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
        // With no unit to start, the daemon command is not needed, so the opaque command is no reason to fail.
        expect(result.exitCode,).toBe(0,);
        expect(result.stderr,).toContain('mvm found no systemd-run on PATH',);
        expect(result.stderr,).not.toContain('mvm cannot tell which command runs libvirt\'s session daemon',);
      },
    },),

    it({
      name: 'does not let virsh start the daemon when systemd-run fails for another reason',
      fn: async () => {
        await using sandbox = await createSandbox({
          domains: DOMAINS,
          sessionDaemon: 'on-demand',
        },);
        await writeExecutable({
          body: [
            `if (process.argv.includes('--version')) { process.stdout.write('systemd 262\n'); process.exit(0); }`,
            "process.stderr.write('Failed to start transient service unit: Invalid argument\n');",
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
        // A failure that says nothing about a loaded name or an unreachable manager keeps the old, safe answer.
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain('and it is not answering',);
        expect(result.stderr,).not.toContain('mvm found no systemd-run on PATH',);
      },
    },),

    //endregion Letting virsh start the daemon
  ],
},);
