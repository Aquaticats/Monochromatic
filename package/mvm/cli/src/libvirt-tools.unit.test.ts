/**
 Tests for choosing the commands that run `virsh` and `qemu-img`, and for the
 host command runner they go through, all from the built package.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import nanoSpawn from 'nano-spawn';

import {
  chooseLibvirtTools,
  CommandFailedError,
  CommandTimedOutError,
  ExecutableNotFoundError,
  isMissingExecutable,
  LibvirtUnresponsiveError,
  parseToolCommand,
  spawn,
  ToolCommandConfigError,
  virsh,
} from '../dist/final/node/index.mjs';

/**
 Command prefix the virt-manager Flatpak runs a tool with.
 */
function flatpakArgv(tool: string,): readonly string[] {
  return [
    'flatpak',
    'run',
    `--command=${tool}`,
    'org.virt_manager.virt-manager',
  ];
}

/**
 Sets the virsh command variable for the scope, restoring the prior value on disposal.
 */
function withVirshCommand(argv: readonly string[],): Disposable {
  const prior = process.env.MVM_VIRSH_COMMAND;
  process.env.MVM_VIRSH_COMMAND = JSON.stringify(argv,);
  return {
    [Symbol.dispose]() {
      if (prior === undefined)
        Reflect.deleteProperty(process.env, 'MVM_VIRSH_COMMAND',);
      else
        process.env.MVM_VIRSH_COMMAND = prior;
    },
  };
}

/**
 Runs `operation` and returns what it threw; fails when it did not throw.
 */
async function caught(operation: () => unknown,): Promise<unknown> {
  try {
    await operation();
  }
  catch (error) {
    return error;
  }
  throw new Error('expected the operation to throw',);
}

/**
 Values of the virsh command variable that are not a JSON array of non-empty strings.
 */
const UNREADABLE_VALUES = [
  'flatpak run --command=virsh org.virt_manager.virt-manager',
  '"virsh"',
  '[]',
  '["virsh", 1]',
  '["virsh", ""]',
  '{"command":"virsh"}',
];

await describe({
  name: '',
  children: [
    describe({
      name: chooseLibvirtTools.name,
      children: [
        it({
          name: 'uses the bare tool names when virsh is on PATH, even with the Flatpak installed',
          fn: async () => {
            expect(chooseLibvirtTools({
              flatpakInstalled: true,
              qemuImgValue: '',
              virshOnPath: true,
              virshValue: '',
            },),).toEqual({
              qemuImg: {
                argv: ['qemu-img',],
                origin: 'path',
              },
              virsh: {
                argv: ['virsh',],
                origin: 'path',
              },
            },);
          },
        },),
        it({
          name: 'runs both tools inside the virt-manager Flatpak when virsh is not on PATH and the Flatpak is installed',
          fn: async () => {
            expect(chooseLibvirtTools({
              flatpakInstalled: true,
              qemuImgValue: '',
              virshOnPath: false,
              virshValue: '',
            },),).toEqual({
              qemuImg: {
                argv: flatpakArgv('qemu-img',),
                origin: 'flatpak',
              },
              virsh: {
                argv: flatpakArgv('virsh',),
                origin: 'flatpak',
              },
            },);
          },
        },),
        it({
          name: 'falls back to the bare tool names when neither virsh nor the Flatpak exists',
          fn: async () => {
            const tools = chooseLibvirtTools({
              flatpakInstalled: false,
              qemuImgValue: '',
              virshOnPath: false,
              virshValue: '',
            },);
            expect(tools.virsh.argv,).toEqual(['virsh',],);
            expect(tools.qemuImg.argv,).toEqual(['qemu-img',],);
          },
        },),
        it({
          name: 'takes a configured tool from its variable and still detects the other one',
          fn: async () => {
            const tools = chooseLibvirtTools({
              flatpakInstalled: true,
              qemuImgValue: '',
              virshOnPath: false,
              virshValue: '["/opt/my tools/virsh","--debug","4"]',
            },);
            expect(tools.virsh,).toEqual({
              argv: [
                '/opt/my tools/virsh',
                '--debug',
                '4',
              ],
              origin: 'environment',
            },);
            expect(tools.qemuImg.origin,).toBe('flatpak',);
          },
        },),
        it({
          name: 'takes both tools from their variables',
          fn: async () => {
            const tools = chooseLibvirtTools({
              flatpakInstalled: false,
              qemuImgValue: '["/opt/qemu/bin/qemu-img"]',
              virshOnPath: true,
              virshValue: '["/opt/libvirt/bin/virsh"]',
            },);
            expect(tools.virsh.argv,).toEqual(['/opt/libvirt/bin/virsh',],);
            expect(tools.qemuImg,).toEqual({
              argv: ['/opt/qemu/bin/qemu-img',],
              origin: 'environment',
            },);
          },
        },),
      ],
    },),

    describe({
      name: parseToolCommand.name,
      children: [
        it({
          name: 'keeps an argument with spaces as one argument',
          fn: async () => {
            expect(parseToolCommand({
              value: '["env","LIBVIRT_DEBUG=1","/opt/my tools/virsh"]',
              variable: 'MVM_VIRSH_COMMAND',
            },),).toEqual([
              'env',
              'LIBVIRT_DEBUG=1',
              '/opt/my tools/virsh',
            ],);
          },
        },),
        ...UNREADABLE_VALUES.map(function rejects(value,) {
          return it({
            name: `rejects ${value} and names the variable with an example`,
            fn: async () => {
              const error = await caught(() =>
                parseToolCommand({
                  value,
                  variable: 'MVM_VIRSH_COMMAND',
                },)
              );
              expect(error,).toBeInstanceOf(ToolCommandConfigError,);
              expect((error as Error).message,).toContain('MVM_VIRSH_COMMAND must be a JSON array',);
              expect((error as Error).message,).toContain(value,);
              expect((error as Error).message,).toContain(
                '["flatpak","run","--command=virsh","org.virt_manager.virt-manager"]',
              );
            },
          },);
        },),
      ],
    },),

    describe({
      name: spawn.name,
      children: [
        it({
          name: 'returns when the command exits although a process it left behind keeps the output open',
          fn: async () => {
            const started = performance.now();
            const output = await spawn({
              args: [
                '--eval',
                [
                  'const { spawn } = require("node:child_process");',
                  'const lingering = spawn(process.execPath, ["--eval", "setTimeout(() => {}, 60000)"], { detached: true, stdio: "inherit" });',
                  'lingering.unref();',
                  'console.log(String(lingering.pid));',
                ].join('\n',),
              ],
              command: process.execPath,
            },);
            const elapsed = performance.now() - started;
            process.kill(
              Number(output,),
              'SIGKILL',
            );
            expect(Number(output,),).toBeGreaterThan(0,);
            // The lingering process holds the output for 60 seconds; a runner that waits for the streams to close would take that long.
            expect(elapsed,).toBeLessThan(30_000,);
          },
        },),
        it({
          name: 'stops a command at its deadline',
          fn: async () => {
            const error = await caught(() =>
              spawn({
                args: [
                  '--eval',
                  'setTimeout(() => {}, 60000)',
                ],
                command: process.execPath,
                deadlineMs: 300,
              },)
            );
            expect(error,).toBeInstanceOf(CommandTimedOutError,);
            expect((error as InstanceType<typeof CommandTimedOutError>).deadlineMs,).toBe(300,);
          },
        },),
        it({
          name: 'reports exit status and both output streams of a failed command',
          fn: async () => {
            const error = await caught(() =>
              spawn({
                args: [
                  '--eval',
                  'console.log("partial output"); console.error("what went wrong"); process.exit(7)',
                ],
                command: process.execPath,
              },)
            );
            expect(error,).toBeInstanceOf(CommandFailedError,);
            const failed = error as InstanceType<typeof CommandFailedError>;
            expect(failed.exitCode,).toBe(7,);
            expect(failed.stdout,).toBe('partial output\n',);
            expect(failed.stderr,).toBe('what went wrong\n',);
            expect(failed.message,).toContain('exited with status 7',);
            expect(failed.message,).toContain('what went wrong',);
          },
        },),
        it({
          name: 'names the signal that ended a command',
          fn: async () => {
            const error = await caught(() =>
              spawn({
                args: [
                  '--eval',
                  'process.kill(process.pid, "SIGTERM")',
                ],
                command: process.execPath,
              },)
            );
            expect(error,).toBeInstanceOf(CommandFailedError,);
            expect((error as InstanceType<typeof CommandFailedError>).signal,).toBe('SIGTERM',);
            expect((error as Error).message,).toContain('ended by SIGTERM',);
          },
        },),
        it({
          name: 'names an executable that does not exist',
          fn: async () => {
            const error = await caught(() =>
              spawn({
                args: [],
                command: 'mvm-test-no-such-executable',
              },)
            );
            expect(error,).toBeInstanceOf(ExecutableNotFoundError,);
            expect((error as InstanceType<typeof ExecutableNotFoundError>).executable,).toBe(
              'mvm-test-no-such-executable',
            );
            expect((error as Error).message,).toContain('`mvm-test-no-such-executable` was not found',);
          },
        },),
        it({
          name: 'returns all of a large output, trimmed',
          fn: async () => {
            const output = await spawn({
              args: [
                '--eval',
                String.raw`process.stdout.write("x".repeat(5 * 1024 * 1024) + "\n")`,
              ],
              command: process.execPath,
            },);
            expect(output.length,).toBe(5 * 1_024 * 1_024,);
          },
        },),
        it({
          name: 'shortens a very long argument in the failure message',
          fn: async () => {
            const error = await caught(() =>
              spawn({
                args: [
                  '--eval',
                  'process.exit(1)',
                  'y'.repeat(5_000,),
                ],
                command: process.execPath,
              },)
            );
            expect((error as Error).message,).toContain('...(4700 more characters)',);
            expect((error as Error).message.length,).toBeLessThan(1_000,);
          },
        },),
      ],
    },),

    describe({
      name: isMissingExecutable.name,
      children: [
        it({
          name: 'recognizes the error node:child_process emits for a missing executable',
          fn: async () => {
            expect(
              isMissingExecutable(Object.assign(
                new Error('spawn virsh ENOENT',),
                { code: 'ENOENT', },
              ),),
            ).toBe(true,);
          },
        },),
        it({
          name: 'recognizes a nano-spawn failure for a missing executable, which carries the cause',
          fn: async () => {
            const error = await caught(() => nanoSpawn('mvm-test-no-such-executable',));
            expect(isMissingExecutable(error,),).toBe(true,);
          },
        },),
        it({
          name: 'does not take a command that ran and failed for a missing executable',
          fn: async () => {
            const error = await caught(() =>
              nanoSpawn(
                process.execPath,
                [
                  '--eval',
                  'process.exit(2)',
                ],
              )
            );
            expect(isMissingExecutable(error,),).toBe(false,);
            expect(
              isMissingExecutable(new Error('other',),),
            ).toBe(false,);
            expect(isMissingExecutable('ENOENT',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: virsh.name,
      // Sequential: the test sets the process-wide command variable.
      concurrency: 1,
      children: [
        it({
          name: 'says the session daemon is not answering when virsh outlasts its deadline',
          fn: async () => {
            using _command = withVirshCommand([
              process.execPath,
              '--eval',
              'setTimeout(() => {}, 60000)',
              '--',
            ],);
            const error = await caught(() =>
              virsh({
                args: [
                  'list',
                  '--all',
                ],
                deadlineMs: 300,
              },)
            );
            expect(error,).toBeInstanceOf(LibvirtUnresponsiveError,);
            expect((error as Error).message,).toContain('session daemon is missing or not answering',);
            expect((error as Error).message,).toContain('virtqemud --verbose',);
          },
        },),
      ],
    },),
  ],
},);
