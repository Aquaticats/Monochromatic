/**
 Tests of the built `mvm` executable against stand-in host tools: which
 command it runs `virsh` through, and what it says when libvirt is out of reach.

 Each test has its own sandbox, so the tests run side by side.

 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  createSandbox,
  fakeVirshEnv,
  installFakeFlatpak,
  runCli,
} from './fixture.cli-harness.ts';

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

await describe({
  name: 'mvm with configured or detected libvirt tools',
  children: [
    //region Configured command

    it({
      name: 'runs virsh through the command in MVM_VIRSH_COMMAND',
      fn: async () => {
        await using sandbox = await createSandbox({ domains: DOMAINS, },);
        const result = await runCli({
          args: ['list',],
          env: fakeVirshEnv(sandbox,),
          sandbox,
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toContain('dev',);
        expect(result.stdout,).toContain('running',);
        expect(result.stdout,).not.toContain('someone-elses-vm',);
        expect(await sandbox.calls(),).toEqual([
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
      name: 'explains the expected form when MVM_VIRSH_COMMAND is not a JSON array',
      fn: async () => {
        await using sandbox = await createSandbox({},);
        const result = await runCli({
          args: ['list',],
          env: { MVM_VIRSH_COMMAND: 'flatpak run --command=virsh org.virt_manager.virt-manager', },
          sandbox,
        },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('MVM_VIRSH_COMMAND must be a JSON array',);
        expect(result.stderr,).toContain('Example: MVM_VIRSH_COMMAND=',);
      },
    },),

    //endregion Configured command

    //region Detected Flatpak

    it({
      name: 'runs virsh inside the virt-manager Flatpak when PATH has no virsh and the Flatpak is installed',
      fn: async () => {
        await using sandbox = await createSandbox({ domains: DOMAINS, },);
        await installFakeFlatpak(sandbox,);
        const result = await runCli({
          args: ['list',],
          sandbox,
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toContain('dev',);
        const flatpakCalls = await readFile(
          join(
            sandbox.state,
            'flatpak-calls.jsonl',
          ),
          'utf8',
        );
        expect(flatpakCalls,).toContain(
          JSON.stringify([
            'run',
            '--command=virsh',
            'org.virt_manager.virt-manager',
            '--connect',
            'qemu:///session',
            'list',
            '--all',
          ],),
        );
      },
    },),

    it({
      name: 'says how to start the session daemon inside the Flatpak when virsh cannot connect',
      fn: async () => {
        await using sandbox = await createSandbox({ connectFailure: true, },);
        await installFakeFlatpak(sandbox,);
        const result = await runCli({
          args: ['list',],
          sandbox,
        },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('virsh could not connect to libvirt\'s session daemon',);
        expect(result.stderr,).toContain('virtqemud-sock',);
        expect(result.stderr,).toContain(
          'flatpak run --command=virtqemud org.virt_manager.virt-manager --verbose',
        );
      },
    },),

    //endregion Detected Flatpak

    //region Missing tools

    it({
      name: 'names virsh and how to configure it when neither virsh nor the Flatpak exists, without a stack',
      fn: async () => {
        await using sandbox = await createSandbox({},);
        const result = await runCli({
          args: ['list',],
          sandbox,
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain('The executable `virsh` was not found in any directory of PATH',);
        expect(result.stderr,).toContain('mvm runs virsh as ["virsh"]',);
        expect(result.stderr,).toContain('set MVM_VIRSH_COMMAND to a JSON array',);
        expect(result.stderr,).toContain('org.virt_manager.virt-manager Flatpak',);
        expect(result.stderr,).toContain('The MCP server reads MVM_VIRSH_COMMAND from its own environment',);
        expect(result.stderr,).not.toContain('dist/final/node',);
        expect(result.stderr,).not.toContain('\n    at ',);
      },
    },),

    it({
      name: 'keeps the stack and the cause when --verbose is given',
      fn: async () => {
        await using sandbox = await createSandbox({},);
        const result = await runCli({
          args: [
            '--verbose',
            'list',
          ],
          sandbox,
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain('ExecutableNotFoundError',);
        expect(result.stderr,).toContain('\n    at ',);
        expect(result.stderr,).toContain('ENOENT',);
      },
    },),

    it({
      name: 'says that the configured virsh executable does not exist',
      fn: async () => {
        await using sandbox = await createSandbox({},);
        const result = await runCli({
          args: ['list',],
          env: { MVM_VIRSH_COMMAND: '["/no/such/dir/virsh","--quiet"]', },
          sandbox,
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain('The executable /no/such/dir/virsh does not exist',);
        expect(result.stderr,).toContain('because MVM_VIRSH_COMMAND names it',);
      },
    },),

    it({
      name: 'names qemu-img and its variable when only qemu-img is missing',
      fn: async () => {
        await using sandbox = await createSandbox({},);
        const images = join(
          sandbox.home,
          '.local',
          'share',
          'mvm',
          'images',
        );
        await mkdir(
          images,
          { recursive: true, },
        );
        await writeFile(
          join(
            images,
            'template-ubuntu.qcow2',
          ),
          '',
        );
        const result = await runCli({
          args: [
            'create',
            'dev',
          ],
          env: fakeVirshEnv(sandbox,),
          sandbox,
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain('The executable `qemu-img` was not found in any directory of PATH',);
        expect(result.stderr,).toContain('set MVM_QEMU_IMG_COMMAND to a JSON array',);
      },
    },),

    it({
      name: 'reports a missing virsh from the interactive shell command instead of exiting quietly',
      fn: async () => {
        await using sandbox = await createSandbox({},);
        const result = await runCli({
          args: [
            'shell',
            'dev',
          ],
          sandbox,
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain('The executable `virsh` was not found in any directory of PATH',);
      },
    },),

    //endregion Missing tools
  ],
},);
