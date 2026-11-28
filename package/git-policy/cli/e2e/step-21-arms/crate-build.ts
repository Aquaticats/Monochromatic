/**
 Push this checkout into the guest and build the crate's test target there, so the arms compile against the tree this run
 has rather than against a copy somebody remembered to add.

 The tree is unpacked over whatever the guest already holds, so a second run reuses the build's target folder instead of
 compiling the whole crate again: a renamed source file would linger, and these arms only add files. One `rustc` for a
 crate this size wants more than five gigabytes and the guest has eight, so one job and a swap file are what make the
 build fit; the other guest drivers measure the same bound.

 @example
 ```ts
 await buildInGuest({ chosen });
 ```
 */

import {
  SWAP_GIBIBYTES,
} from '../../bin/mvm-guest-bounds.ts';
import {
  awaitDetachedGuestScript,
  startDetachedGuestScript,
} from '../../bin/mvm-guest-detached-script.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  pushWorktreeSnapshot,
} from '../../bin/mvm-guest-host-command.ts';
import {
  buildLog,
  crateBuildTimeoutMilliseconds,
  crateDirectory,
  finishMarginMilliseconds,
} from './step-21-fixture.ts';

/**
 Push this checkout into the guest and build the crate's test target there.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await buildInGuest({ chosen });
 ```
 */
export async function buildInGuest({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  await pushWorktreeSnapshot({ chosen, },);
  /**
   The guest's whole build, which unpacks the pushed archive and compiles the test target the arms run.
   */
  const script = [
    'set -eu',
    // `mvm push` lands every file in the guest's shared folder, so the copy the build unpacks is taken from there and a
    // missing one refuses.
    'test -f /mnt/shared/subject.tar.gz',
    'cp /mnt/shared/subject.tar.gz /root/subject.tar.gz',
    // The tree is unpacked over whatever the guest already holds, so a second run reuses the build's target folder.
    `mkdir --parents ${crateDirectory}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${crateDirectory}`,
    `cd ${crateDirectory}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'df --human-readable / /mnt/shared',
    'free',
    // One rustc for a crate this size wants more than five gigabytes and the guest has eight, so one job and a swap file
    // are what make the build fit.
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${SWAP_GIBIBYTES} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'free',
    'CARGO_BUILD_JOBS=1 cargo build --release --tests --locked > /root/step21-build.log 2>&1',
    'tail --lines=3 /root/step21-build.log',
  ].join('\n',);
  await startDetachedGuestScript({
    chosen,
    script,
    log: buildLog,
    timeoutMilliseconds: crateBuildTimeoutMilliseconds,
    nameCheck: 'step',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: buildLog,
    finishTimeoutMilliseconds: crateBuildTimeoutMilliseconds + finishMarginMilliseconds,
  },);
}
