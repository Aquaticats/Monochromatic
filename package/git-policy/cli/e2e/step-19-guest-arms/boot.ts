/**
 The plan's boot arms in a guest:

 - every file the plan writes, installed where systemd reads it and enabled, with the guest's own reboot after it, and
   what the manager made of it read back from the boot that followed;
 - one managed clone with durable intent, made through the release writers the add route uses, and the same reading
   after a second reboot, where the gate must have attached it again;
 - three boots whose provider did not deliver the checkout, one each for the pool's import, the checkout's key and the
   checkout's own mount, where nothing attaches a clone and nothing creates a mountpoint folder in the home underlay.

 A reboot ends the command that asked for it, so the request is sent for its status rather than for its answer, and the
 wait is a poll bounded by a deadline (`AGENTS.md` rule `BWL`, `PWT`).

 @example
 ```ts
 await installThePlan({ chosen });
 await rebootTheGuest({ chosen });
 await readTheBoot({ chosen, expect: 'healthy' });
 ```
 */

import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import { guest, } from '../../bin/mvm-guest-script.ts';
import {
  runStepCommand,
  statusOf,
} from '../../bin/mvm-guest-step-command.ts';
import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  checkoutPath,
  containerDataset,
  checkoutDataset,
  expectedFailedBootMarkers,
  expectedHealthyBootMarkers,
  fixtureRoot,
  nestedDataset,
  nestedMountpoint,
  nestedSlug,
  openZfsSbin,
  openZfsSource,
  pool,
  rebootTest,
  rebootTimeoutMilliseconds,
  slot,
  underlay,
} from './fixture.ts';
import {
  runOneGuestTest,
} from './guest-test-run.ts';
import {
  requireMarkers,
} from '../../bin/mvm-guest-arm-log.ts';
import { GuestDriverError, } from '../../bin/mvm-guest-error.ts';

/**
 The environment the boot arm's phases read, which every phase of it shares.

 @param chosen - the run's guest, whose name the arm's gate reads

 @returns the `export` lines every phase starts with

 @example
 ```ts
 bootEnvironment({ chosen }); // ['export CLI_GIT_ZFS_STEP19_CHECKOUT=/home/…', …]
 ```
 */
function bootEnvironment({
  chosen,
}: {
  readonly chosen: GuestFacts;
}): readonly string[] {
  /**
   The `export` lines every phase of the boot arm reads.
   */
  return [
    `export CLI_GIT_ZFS_STEP19_CHECKOUT=${checkoutPath}`,
    `export CLI_GIT_ZFS_STEP19_NESTED=${nestedDataset}`,
    `export CLI_GIT_ZFS_STEP19_NESTED_MOUNTPOINT=${nestedMountpoint}`,
    `export CLI_GIT_ZFS_STEP19_NESTED_SLUG=${nestedSlug}`,
    `export CLI_GIT_ZFS_STEP19_UNDERLAY=${underlay}`,
    `export CLI_GIT_ZFS_STEP19_SLOT=${slot}`,
    `export CLI_GIT_ZFS_STEP19_SBIN=${openZfsSbin}`,
    `export CLI_GIT_ZFS_STEP19_CHECKOUT_DATASET=${checkoutDataset}`,
    `export CLI_GIT_ZFS_STEP19_CONTAINER=${containerDataset}`,
    `export CLI_GIT_ZFS_STEP19_GUEST=${chosen.name}`,
  ];
}

/**
 Write every file of the plan where systemd reads it and enable the gate and the nested dataset's unit.

 The files stay: the guest is rebooted with them in place, and the phase after the reboot reads what the manager did.

 @param chosen - the run's guest and log folder

 @returns the phase's own log

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await installThePlan({ chosen });
 ```
 */
export async function installThePlan({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  return await runOneGuestTest({
    chosen,
    test: rebootTest,
    environment: [
      ...bootEnvironment({ chosen, }),
      'export CLI_GIT_ZFS_STEP19_PHASE=install',
    ],
    log: 'guest-step19-boot-install.log',
  },);
}

/**
 Make one managed clone of the checkout dataset's snapshot, with the durable intent the add route writes and the slot
 registration Git writes, so the next boot has a clone to attach again.

 @param chosen - the run's guest and log folder

 @returns the phase's own log

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await makeTheClone({ chosen });
 ```
 */
export async function makeTheClone({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  return await runOneGuestTest({
    chosen,
    test: rebootTest,
    environment: [
      ...bootEnvironment({ chosen, }),
      'export CLI_GIT_ZFS_STEP19_PHASE=clone',
    ],
    log: 'guest-step19-boot-clone.log',
  },);
}

/**
 How often the wait asks whether the guest answers again, in milliseconds: a boot of this guest takes about a minute, and
 a closer poll only spends the agent's own answers.
 */
const rebootPollMilliseconds = 10_000;

/**
 Ask the guest to reboot and wait until it answers again.

 @param chosen - the run's guest and log folder

 @example
 ```ts
 await rebootTheGuest({ chosen });
 ```
 */
export async function rebootTheGuest({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  await runStepCommand({
    logDirectory: chosen.logDir,
    program: 'mvm',
    argumentWords: [
      '--backend',
      'libvirt',
      'exec',
      chosen.name,
      '--',
      `test "$(hostname)" = ${chosen.name} && (systemctl reboot || true)`,
    ],
    log: 'guest-step19-reboot.log',
    timeoutMilliseconds: 180_000,
  },);
  /**
   When the wait stops asking, which is the reboot's whole bound.
   */
  const deadline: number = Date.now() + rebootTimeoutMilliseconds;
  await waitForTheGuest({
    chosen,
    deadline,
  },);
}

/**
 Ask the guest whether it answers again, and wait once more when it does not.

 Why a call of its own:
  a reboot ends the command that asked for it,
 so the wait is a question asked until it is answered,
 and each answer ends before the next one is asked.

 @param chosen - the run's guest and log folder

 @param deadline - when the wait stops asking

 @throws GuestDriverError when the guest did not answer inside the deadline

 @example
 ```ts
 await waitForTheGuest({ chosen, deadline: Date.now() + 600_000 });
 ```
 */
async function waitForTheGuest({
  chosen,
  deadline,
}: {
  readonly chosen: GuestFacts;
  readonly deadline: number;
},): Promise<void> {
  /**
   The status the probe exited with, which only zero says the guest answers again.
   */
  const answered = await statusOf({
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'exec',
      chosen.name,
      '--',
      `test "$(hostname)" = ${chosen.name} && hostname`,
    ],
    timeoutMilliseconds: 60_000,
  },);
  if (answered === 0)
    return;
  if (Date.now() > deadline)
    throw new GuestDriverError(`guest '${chosen.name}' did not answer within ${String(rebootTimeoutMilliseconds,)} ms of its reboot`,);
  await wait(rebootPollMilliseconds,);
  await waitForTheGuest({
    chosen,
    deadline,
  },);
}

/**
 Read what the boot did with the plan and judge it against what the driver said the boot was.

 @param chosen - the run's guest and log folder

 @param expect - `healthy` for a boot that had its checkout, `failed` for one that did not

 @returns the phase's own log

 @throws GuestDriverError naming the marker the log lacks, or the step that refused

 @example
 ```ts
 await readTheBoot({ chosen, expect: 'healthy' });
 ```
 */
export async function readTheBoot({
  chosen,
  expect,
}: {
  readonly chosen: GuestFacts;
  readonly expect: 'healthy' | 'failed';
},): Promise<string> {
  /**
   The reading's own log, whose name says which boot it read.
   */
  const log = `guest-step19-boot-read-${expect}.log`;
  /**
   The phase's own log, which the markers are read from.
   */
  const pulled = await runOneGuestTest({
    chosen,
    test: rebootTest,
    environment: [
      ...bootEnvironment({ chosen, }),
      'export CLI_GIT_ZFS_STEP19_PHASE=read',
      `export CLI_GIT_ZFS_STEP19_BOOT_EXPECT=${expect}`,
    ],
    log,
    timeoutMilliseconds: 600_000,
  },);
  requireMarkers({
    text: pulled,
    markers: expect === 'healthy' ? expectedHealthyBootMarkers : expectedFailedBootMarkers,
    subject: `the pulled ${expect} boot log`,
    because: 'the reading after the reboot did not reach the state it is named for',
  },);
  return pulled;
}

/**
 Leave the plan's files and take the units out of the boot.

 @param chosen - the run's guest and log folder

 @returns the phase's own log

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await removeThePlan({ chosen });
 ```
 */
export async function removeThePlan({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  return await runOneGuestTest({
    chosen,
    test: rebootTest,
    environment: [
      ...bootEnvironment({ chosen, }),
      'export CLI_GIT_ZFS_STEP19_PHASE=remove',
    ],
    log: 'guest-step19-boot-remove.log',
  },);
}

/**
 What one failing provider looks like between two boots: the guest side that breaks the piece this arm is about, and the
 guest side that puts it back so the next arm starts from the healthy layout.
 */
type FailingProvider = {
  /**
   The arm's own name, which its logs carry.
   */
  readonly name: string;
  /**
   What breaks, run before the reboot.
   */
  readonly breaks: readonly string[];
  /**
   What puts it back, run after the reading.
   */
  readonly restores: readonly string[];
};

/**
 The three providers a boot can be refused by, in the order the plan names them: the pool's import, the checkout's key
 and the checkout's own mount.
 */
const failingProviders: readonly FailingProvider[] = [
  {
    name: 'import',
    breaks: [
      `zfs unmount ${nestedMountpoint} > /dev/null 2>&1 || true`,
      `zpool export -f ${pool}`,
      `rm --force ${openZfsSource}/etc/zfs/zpool.cache`,
    ],
    restores: [
      `zpool import -d ${fixtureRoot} ${pool}`,
      `zpool set cachefile=${openZfsSource}/etc/zfs/zpool.cache ${pool}`,
      'zfs mount -a',
      `mount --make-rprivate ${checkoutPath}`,
    ],
  },
  {
    name: 'key',
    breaks: [
      `zfs unmount ${nestedMountpoint} > /dev/null 2>&1 || true`,
      `zfs unmount ${checkoutPath}`,
      `mv ${fixtureRoot}/checkout-key ${fixtureRoot}/checkout-key.aside`,
    ],
    restores: [
      `mv ${fixtureRoot}/checkout-key.aside ${fixtureRoot}/checkout-key`,
      `zfs load-key ${checkoutDataset}`,
      `zfs mount ${checkoutDataset}`,
      `mount --make-rprivate ${checkoutPath}`,
    ],
  },
  {
    name: 'checkout-mount',
    breaks: [
      `zfs unmount ${nestedMountpoint} > /dev/null 2>&1 || true`,
      `zfs set canmount=off ${checkoutDataset}`,
    ],
    restores: [
      `zfs set canmount=on ${checkoutDataset}`,
      `zfs mount ${checkoutDataset}`,
      `mount --make-rprivate ${checkoutPath}`,
    ],
  },
];

/**
 The guest script one failing provider's half runs.

 @param lines - the commands the half runs

 @param header - which half it is, for the log's own first line

 @returns the shell script

 @example
 ```ts
 failingHalf({ lines: ['zpool export -f zfsstep19'], header: 'break import' });
 ```
 */
function failingHalf({
  lines,
  header,
}: {
  readonly lines: readonly string[];
  readonly header: string;
}): string {
  return [
    `export PATH=${[openZfsSbin,].concat([
      '/usr/local/sbin',
      '/usr/local/bin',
      '/usr/sbin',
      '/usr/bin',
      '/sbin',
      '/bin',
    ],)
      .join(':')}`,
    'export LD_LIBRARY_PATH=/opt/openzfs-2.4.4/lib',
    'set -eu',
    `echo "STEP19-FAILING half=${header}"`,
    ...lines,
  ].join('\n',);
}

/**
 Run the plan's three failing boots: break one provider, reboot, read, put it back.

 @param chosen - the run's guest and log folder

 @returns what every step wrote, one entry per line

 @throws GuestDriverError naming the step that refused, or the marker a reading lacks

 @example
 ```ts
 const said = await theFailingBoots({ chosen });
 ```
 */
export async function theFailingBoots({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<readonly string[]> {
  return await failEachProvider({
    chosen,
    providers: failingProviders,
    said: [],
  },);
}

/**
 Break one provider, reboot, read the boot that was refused and put the provider back, then the next one.

 Why a call of its own:
  a failing boot is a reboot,
 so one arm ends before the next one starts,
 and each provider's layout is restored before the next is broken.

 @param chosen - the run's guest and log folder

 @param providers - the providers this run has not reached yet

 @param said - what the steps before this one wrote

 @returns every step's log, in the order the boots ran

 @throws GuestDriverError naming the step that refused, or the marker a reading lacks

 @example
 ```ts
 await failEachProvider({ chosen, providers: failingProviders, said: [] });
 ```
 */
async function failEachProvider({
  chosen,
  providers,
  said,
}: {
  readonly chosen: GuestFacts;
  readonly providers: readonly FailingProvider[];
  readonly said: readonly string[];
},): Promise<readonly string[]> {
  /**
   The provider this call breaks and puts back, absent when every one was reached.
   */
  const [provider, ...rest] = providers;
  if (provider === undefined)
    return said;
  /**
   What the step that broke the provider wrote.
   */
  const broke = await guest({
    chosen,
    script: failingHalf({
      lines: provider.breaks,
      header: `break-${provider.name}`,
    },),
    log: `guest-step19-break-${provider.name}.log`,
    timeoutMilliseconds: 300_000,
  },);
  await rebootTheGuest({ chosen, },);
  /**
   What the reading after the refusing boot wrote.
   */
  const read = await readTheBoot({
    chosen,
    expect: 'failed',
  },);
  /**
   What the step that put the provider back wrote.
   */
  const restored = await guest({
    chosen,
    script: failingHalf({
      lines: provider.restores,
      header: `restore-${provider.name}`,
    },),
    log: `guest-step19-restore-${provider.name}.log`,
    timeoutMilliseconds: 300_000,
  },);
  return await failEachProvider({
    chosen,
    providers: rest,
    said: [
      ...said,
      broke,
      read,
      restored,
    ],
  },);
}
