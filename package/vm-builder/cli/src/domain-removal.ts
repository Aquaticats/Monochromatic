/**
 Removes an existing libvirt domain so the pipeline can define a fresh one under the same name.

 Both entry scripts of this package import this module, so the decision of whether an existing domain has to be stopped
 before it is undefined has one owner and one unit test.
 */
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  exec,
  type ExecOptions,
} from './exec.ts';

/**
 Runs one external command and resolves with its captured stdout.

 @param options - command and its arguments

 @returns Stdout of the command

 @example
 ```ts
 const state = await runner({ cmd: 'virsh', args: ['domstate', 'dev'], });
 ```
 */
export type CommandRunner = (options: ExecOptions,) => Promise<string>;

/**
 Destroys and undefines an existing libvirt domain if it exists.
 No-op when the domain is not defined; the existence probe's failure is logged via {@link caughtValueText}.
 Forcibly destroys a running domain before undefining to avoid an "active domain" error.

 @param name - Libvirt domain name to remove

 @param uri - Libvirt connection URI the commands connect to, such as `qemu:///session`

 @param runner - Command runner; tests pass one that records the `virsh` calls instead of running them

 @throws `SubprocessError` from {@link exec} when a `virsh` call other than the existence probe fails

 @example
 ```ts
 await undefineVmIfExists({ name: 'monochromatic-dev', uri: 'qemu:///session', });
 ```
 */
export async function undefineVmIfExists(
  {
    name,
    uri,
    runner = exec,
  }: {
    readonly name: string;
    readonly uri: string;
    readonly runner?: CommandRunner;
  },
): Promise<void> {
  try {
    await runner({
      cmd: 'virsh',
      args: [
        '--connect',
        uri,
        'dominfo',
        name,
      ],
    },);
  }
  catch (error) {
    console.warn(
      `[vm-builder] libvirt domain '${name}' existence probe failed; skipping removal: ${caughtValueText(error,)}`,
    );
    // Domain not defined; nothing to remove.
    return;
  }
  console.log(`[vm-builder] removing existing VM '${name}'...`,);
  /**
   Current domain state from `virsh domstate`; `'running'` requires `destroy` before `undefine`.
   */
  const state = (await runner({
    cmd: 'virsh',
    args: [
      '--connect',
      uri,
      'domstate',
      name,
    ],
  },))
    .trim();
  if (state === 'running') {
    await runner({
      cmd: 'virsh',
      args: [
        '--connect',
        uri,
        'destroy',
        name,
      ],
    },);
  }
  await runner({
    cmd: 'virsh',
    args: [
      '--connect',
      uri,
      'undefine',
      name,
      '--nvram',
    ],
  },);
}
