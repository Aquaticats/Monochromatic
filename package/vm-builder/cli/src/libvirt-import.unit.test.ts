/**
 The steps that put the built qcow2 into libvirt, checked against a runner that records the commands instead of running
 them: this host has no hypervisor, and the tests must not create or change a libvirt domain.
 */

import {
  mkdtempDisposable,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import type { CommandRunner, } from '../dist/final/node/exec.mjs';
import {
  copyDiskImageIntoLibvirt,
  grantFlatpakAccess,
  installDomainIntoLibvirt,
} from '../dist/final/node/libvirt-import.mjs';

//region Fixtures

/**
 One command a step ran, as the recording runner saw it.
 */
type RecordedCommand = {
  readonly cmd: string;
  readonly args: readonly string[];
};

/**
 Builds a runner that records every command and can refuse one, so a step's failure path is exercised without the command
 existing on the host.

 @param failOn - Predicate naming the command that must fail; none means every command succeeds

 @returns The recorded calls, filled as the step runs, and the runner to pass it
 */
function recordCommands(
  {
    failOn,
  }: {
    readonly failOn?: (command: RecordedCommand,) => boolean;
  } = {},
): {
  readonly commands: RecordedCommand[];
  readonly runner: CommandRunner;
} {
  const commands: RecordedCommand[] = [];
  const runner: CommandRunner = async ({ cmd, args = [], },) => {
    const command = {
      cmd,
      args: [...args,],
    };
    commands.push(command,);
    if (failOn?.(command,) === true) {
      throw new Error(`${cmd} refused by the recording runner`,);
    }
    return '';
  };
  return {
    commands,
    runner,
  };
}

/**
 Captures the rejection reason of an operation, so a case reads the error the step let through.

 @param operation - Call expected to reject

 @returns The rejection reason

 @throws Error when the operation fulfilled instead, because the case expected a failure
 */
async function rejectionOf(operation: Promise<unknown>,): Promise<Error> {
  try {
    await operation;
  }
  catch (error) {
    return error as Error;
  }
  throw new Error('expected the step to reject, and it did not',);
}

/**
 Disk image the copy cases hand the step, spelled as the entry scripts resolve it.
 */
const FIXTURE_DISK = {
  sourcePath: '/repo/package/vm-builder/cli/output/qcow2/disk.qcow2',
  destinationPath: '/var/lib/libvirt/images/monochromatic-dev.qcow2',
  owner: 'fixture-user',
};

/**
 Domain inputs of the install cases.
 */
const FIXTURE_DOMAIN = {
  name: 'fixture-domain',
  memoryMib: '4096',
  vcpus: '4',
  uri: 'qemu:///fixture',
  qcow2Path: '/var/lib/libvirt/images/fixture-domain.qcow2',
};

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: copyDiskImageIntoLibvirt.name,
      children: [
        it({
          name: 'copies the built disk as root and hands the copy to its owner',
          fn: async () => {
            const { commands, runner, } = recordCommands();
            await copyDiskImageIntoLibvirt({
              ...FIXTURE_DISK,
              runner,
            },);
            expect(commands,).toStrictEqual([
              {
                cmd: 'sudo',
                args: [
                  'cp',
                  FIXTURE_DISK.sourcePath,
                  FIXTURE_DISK.destinationPath,
                ],
              },
              {
                cmd: 'sudo',
                args: [
                  'chown',
                  `${FIXTURE_DISK.owner}:${FIXTURE_DISK.owner}`,
                  FIXTURE_DISK.destinationPath,
                ],
              },
            ],);
          },
        },),
        it({
          name: 'changes ownership of the copy and never of the build output',
          fn: async () => {
            const { commands, runner, } = recordCommands();
            await copyDiskImageIntoLibvirt({
              ...FIXTURE_DISK,
              runner,
            },);
            /**
             Every path the ownership change names.
             */
            const chownedPaths = commands
              .filter((command,) => command.args.includes('chown',),)
              .flatMap((command,) => command.args.slice(2,),);
            expect(chownedPaths,).toStrictEqual([FIXTURE_DISK.destinationPath,],);
          },
        },),
        it({
          name: 'lets a failed copy stop the step and keep its error',
          fn: async () => {
            const { commands, runner, } = recordCommands({
              failOn: (command,) => command.args.includes('cp',),
            },);
            const error = await rejectionOf(copyDiskImageIntoLibvirt({
              ...FIXTURE_DISK,
              runner,
            },),);
            expect(error.message,).toBe('sudo refused by the recording runner',);
            expect(commands,).toHaveLength(1,);
          },
        },),
        it({
          name: 'lets a failed ownership change keep its error',
          fn: async () => {
            const { commands, runner, } = recordCommands({
              failOn: (command,) => command.args.includes('chown',),
            },);
            const error = await rejectionOf(copyDiskImageIntoLibvirt({
              ...FIXTURE_DISK,
              runner,
            },),);
            expect(error.message,).toBe('sudo refused by the recording runner',);
            expect(commands,).toHaveLength(2,);
          },
        },),
      ],
    },),
    describe({
      name: installDomainIntoLibvirt.name,
      children: [
        it({
          name: 'writes the rendered document into the output directory',
          fn: async () => {
            await using fixture = await mkdtempDisposable(join(
              scratchRoot(),
              'vm-builder-install-',
            ),);
            const { runner, } = recordCommands();
            await installDomainIntoLibvirt({
              ...FIXTURE_DOMAIN,
              outputDirectory: fixture.path,
              runner,
            },);
            /**
             Document the step left on disk for the next run to inspect.
             */
            const xml = await readFile(
              join(
                fixture.path,
                'domain.xml',
              ),
              'utf8',
            );
            expect(xml,).toContain(`<name>${FIXTURE_DOMAIN.name}</name>`,);
            expect(xml,).toContain(`<memory unit="MiB">${FIXTURE_DOMAIN.memoryMib}</memory>`,);
            expect(xml,).toContain(`<vcpu>${FIXTURE_DOMAIN.vcpus}</vcpu>`,);
            expect(xml,).toContain(`<source file="${FIXTURE_DOMAIN.qcow2Path}" />`,);
          },
        },),
        it({
          name: 'defines the written document on the given connection',
          fn: async () => {
            await using fixture = await mkdtempDisposable(join(
              scratchRoot(),
              'vm-builder-define-',
            ),);
            const { commands, runner, } = recordCommands();
            await installDomainIntoLibvirt({
              ...FIXTURE_DOMAIN,
              outputDirectory: fixture.path,
              runner,
            },);
            expect(commands,).toStrictEqual([
              {
                cmd: 'virsh',
                args: [
                  '--connect',
                  FIXTURE_DOMAIN.uri,
                  'define',
                  join(
                    fixture.path,
                    'domain.xml',
                  ),
                ],
              },
            ],);
          },
        },),
        it({
          name: 'lets a refused definition keep its error',
          fn: async () => {
            await using fixture = await mkdtempDisposable(join(
              scratchRoot(),
              'vm-builder-define-refused-',
            ),);
            const { runner, } = recordCommands({
              failOn: (command,) => command.cmd === 'virsh',
            },);
            const error = await rejectionOf(installDomainIntoLibvirt({
              ...FIXTURE_DOMAIN,
              outputDirectory: fixture.path,
              runner,
            },),);
            expect(error.message,).toBe('virsh refused by the recording runner',);
          },
        },),
      ],
    },),
    describe({
      name: grantFlatpakAccess.name,
      children: [
        it({
          name: 'grants the output directory to virt-manager when the flatpak is installed',
          fn: async () => {
            const { commands, runner, } = recordCommands();
            await grantFlatpakAccess({
              outputDirectory: '/repo/package/vm-builder/cli/output',
              runner,
            },);
            expect(commands,).toStrictEqual([
              {
                cmd: 'flatpak',
                args: [
                  'info',
                  'org.virt_manager.virt-manager',
                ],
              },
              {
                cmd: 'flatpak',
                args: [
                  'override',
                  '--user',
                  '--filesystem=/repo/package/vm-builder/cli/output',
                  'org.virt_manager.virt-manager',
                ],
              },
            ],);
          },
        },),
        it({
          name: 'skips the override and reports the probe when virt-manager is not a flatpak',
          fn: async (context,) => {
            const warn = context.sinon.spy(
              console,
              'warn',
            );
            const { commands, runner, } = recordCommands({
              failOn: (command,) => command.cmd === 'flatpak',
            },);
            await grantFlatpakAccess({
              outputDirectory: '/repo/package/vm-builder/cli/output',
              runner,
            },);
            expect(commands,).toHaveLength(1,);
            expect(warn.calledOnce,).toBe(true,);
            expect(String(warn.firstCall.args[0],),).toContain('skipping filesystem override',);
          },
        },),
        it({
          name: 'lets a failed override keep its error instead of reporting a grant it did not make',
          fn: async (context,) => {
            const warn = context.sinon.spy(
              console,
              'warn',
            );
            const { commands, runner, } = recordCommands({
              failOn: (command,) => command.args.includes('override',),
            },);
            const error = await rejectionOf(grantFlatpakAccess({
              outputDirectory: '/repo/package/vm-builder/cli/output',
              runner,
            },),);
            expect(error.message,).toBe('flatpak refused by the recording runner',);
            expect(commands,).toHaveLength(2,);
            expect(warn.called,).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
