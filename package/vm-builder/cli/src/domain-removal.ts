/**
 Removes an existing libvirt domain so the pipeline can define a fresh one under the same name.

 Both entry scripts of this package import this module, so the decision of whether an existing domain has to be stopped
 before it is undefined has one owner and one unit test.
 */
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  type CommandRunner,
  exec,
} from './exec.ts';

//region Domain state classification

/**
 States `virsh domstate` prints while libvirt still holds the domain's QEMU process.

 These are the active `virDomainState` values as `virsh` spells them (`virshDomainState` in libvirt's
 `tools/virsh-domain-monitor.c`), where `blocked` prints as `idle`; only one of the five reads `running`.
 */
const ACTIVE_DOMAIN_STATES: readonly string[] = [
  'running',
  'idle',
  'paused',
  'in shutdown',
  'pmsuspended',
];

/**
 States `virsh domstate` prints once no QEMU process is left for the domain, where `virsh destroy` fails with
 "domain is not running".
 */
const INACTIVE_DOMAIN_STATES: readonly string[] = [
  'no state',
  'shut off',
  'crashed',
];

/**
 Raised when `virsh domstate` prints a state neither {@link ACTIVE_DOMAIN_STATES} nor {@link INACTIVE_DOMAIN_STATES}
 carries, so removal never guesses whether `virsh destroy` is required.
 */
export class UnknownDomainStateError extends Error {
  /**
   Builds a message naming the domain, the state read, and both classifications, so the reader can tell a libvirt
   change from a broken probe.

   @param name - domain whose state was read

   @param state - state `virsh domstate` printed
   */
  constructor(
    {
      name,
      state,
    }: {
      readonly name: string;
      readonly state: string;
    },
  ) {
    super(
      `virsh domstate printed '${state}' for libvirt domain '${name}', which is neither an active state (${ACTIVE_DOMAIN_STATES.join(', ',)}) nor an inactive one (${INACTIVE_DOMAIN_STATES.join(', ',)}); stopping before undefining a domain whose QEMU process may still run`,
    );
    this.name = 'UnknownDomainStateError';
  }
}

/**
 Decides whether an existing domain must be destroyed before `virsh undefine` can remove it.

 @param name - domain whose state was read, used in the error

 @param state - what `virsh domstate` printed, trimmed

 @returns `true` when libvirt still holds a QEMU process for the domain

 @throws `UnknownDomainStateError` when the state is in neither classification
 */
function domainStateIsActive(
  {
    name,
    state,
  }: {
    readonly name: string;
    readonly state: string;
  },
): boolean {
  if (ACTIVE_DOMAIN_STATES.includes(state,)) {
    return true;
  }
  if (INACTIVE_DOMAIN_STATES.includes(state,)) {
    return false;
  }
  throw new UnknownDomainStateError({
    name,
    state,
  },);
}

//endregion Domain state classification

/**
 Stops and undefines an existing libvirt domain if it exists.
 No-op when the domain is not defined; the existence probe's failure is logged via {@link caughtValueText}.

 A domain in any active state is destroyed first. `virsh undefine` alone does not stop one: libvirt deletes the
 definition, marks the domain transient, and leaves it running (`qemuDomainUndefineFlags` in `src/qemu/qemu_driver.c`),
 so the definition and disk image the pipeline installs next would sit behind a QEMU still writing the replaced image.
 Only the states with no QEMU process left skip `virsh destroy`, which fails there with "domain is not running".

 @param name - Libvirt domain name to remove

 @param uri - Libvirt connection URI the commands connect to, such as `qemu:///session`

 @param runner - Command runner; tests pass one that records the `virsh` calls instead of running them

 @throws `UnknownDomainStateError` when `virsh domstate` prints a state in neither {@link ACTIVE_DOMAIN_STATES} nor {@link INACTIVE_DOMAIN_STATES}

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
   Current domain state from `virsh domstate`, trimmed of the newline it prints.
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
  if (domainStateIsActive({
    name,
    state,
  },)) {
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
