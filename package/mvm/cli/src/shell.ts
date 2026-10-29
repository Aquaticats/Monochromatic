import nanoSpawn from 'nano-spawn';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  LIBVIRT_URI,
  validateName,
  VM_PREFIX,
} from './config.ts';
import {
  libvirtTools,
  toolNotFoundRemedy,
  VIRSH_COMMAND_ENV,
} from './libvirt-tools.ts';
import {
  ExecutableNotFoundError,
  isMissingExecutable,
} from './spawn-errors.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

/**
 Opens an interactive serial console session to a running VM via `virsh console`,
 through the command {@link libvirtTools} chose for this host.
 The VM is configured with auto-login on ttyS0, so no credentials are needed.
 Press `Ctrl+]` to disconnect from the console.

 @param name - VM name without the mvm- prefix

 @throws {@link ExecutableNotFoundError} when the command that runs virsh does not exist

 @example
 ```ts
 await shell({ name: 'dev-01' });
 ```
 */
export async function shell({ name, }: { readonly name: string; },): Promise<void> {
  validateName(name,);
  /**
   Tagged logger so console-session messages name the call site.
   */
  const rl = tagged({
    tag: shell.name,
    l,
  },);
  /**
   Fully prefixed VM name expected by virsh commands.
   */
  const fullName = `${VM_PREFIX}${name}`;
  /**
   Command that runs virsh on this host.
   */
  const { virsh: tool, } = await libvirtTools();
  /**
   Executable and the arguments that precede virsh's own.
   */
  const [command, ...leading] = tool.argv;

  rl.info(`connecting to VM ${name} via console (press Ctrl+] to disconnect, not exit)`,);

  try {
    await nanoSpawn(
      command,
      [
        ...leading,
        '--connect',
        LIBVIRT_URI,
        'console',
        fullName,
      ],
      {
        stderr: 'inherit',
        stdin: 'inherit',
        stdout: 'inherit',
      },
    );
  }
  catch (error: unknown) {
    if (isMissingExecutable(error,)) {
      rl.debug(`executable ${command} does not exist`,);
      throw new ExecutableNotFoundError({
        cause: error,
        executable: command,
        remedy: toolNotFoundRemedy({
          command: tool,
          tool: 'virsh',
          variable: VIRSH_COMMAND_ENV,
        },),
      },);
    }
    if ((error !== null)
      && (error !== undefined)
      && ((typeof error) === 'object')
      && ('exitCode' in error))
      {
      /**
       Forwarded so the shell exit code reflects the virsh console outcome.
       */
      const exitCode = ((typeof error.exitCode) === 'number')
        ? error.exitCode
        : undefined;
      if (exitCode !== undefined)
        process.exitCode = exitCode;
    }
  }
}
