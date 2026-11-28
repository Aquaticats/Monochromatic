/**
 Push this checkout into the guest and build the two things the step 19 arms run: the release executable a unit's
 `ExecStart` names, and the library test binary the arms are.

 The build runs detached, because it takes minutes: the guest agent has lost the status of a long command
 (`doc/troubleshooting/mvm-guest-agent-forgets-a-long-commands-result.md`), so the work is a script started with
 `setsid nohup` whose own last line says how it ended. That helper is `e2e/adoption-guest-arms/detached-step.ts`, which
 the adoption driver and the step 22 driver already share, so this driver adds none of its own.

 Two builds and not one, because they are two shapes: the executable is built as this release ships it, and the tests
 read the switches a test build carries (`--features test-only-switches`). A unit that started a test build would run
 bytes no release installs.

 @example
 ```ts
 await buildInGuest({ chosen });
 ```
 */

import {
  SWAP_GIBIBYTES,
} from '../../bin/mvm-guest-bounds.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  pushWorktreeSnapshot,
} from '../../bin/mvm-guest-host-command.ts';

import {
  detachedStep,
} from '../adoption-guest-arms/detached-step.ts';
import {
  crateBuildTimeoutMilliseconds,
  crateDirectory,
  cratePackage,
  finishMarginMilliseconds,
} from './fixture.ts';

/**
 The log the guest's build writes inside the guest's shared folder.
 */
const buildLog = 'guest-step19-build.log';

/**
 Push this checkout into the guest and build the release executable and the library tests there.

 @param chosen - the run's guest and log folder

 @returns the pulled build log

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
},): Promise<string> {
  await pushWorktreeSnapshot({ chosen, },);
  /**
   The two builds, their one job each, and the swap file that keeps them inside the guest.
   */
  const script = [
    'set -eu',
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${crateDirectory}`,
    `mkdir --parents ${crateDirectory}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${crateDirectory}`,
    `cd ${cratePackage}`,
    'export PATH=/root/.cargo/bin:$PATH',
    // One job and a swap file keep the build inside the guest: two jobs of this crate reached 7.45 GiB of the guest's
    // 7.75 GiB, and one run ended with a rustc killed at 5.15 GiB (issue 6458).
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${SWAP_GIBIBYTES} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'CARGO_BUILD_JOBS=1 cargo build --release --locked --bins > /root/step19-release-build.log 2>&1',
    'tail --lines=3 /root/step19-release-build.log',
    'CARGO_BUILD_JOBS=1 cargo build --release --locked --lib --tests --features test-only-switches > /root/step19-test-build.log 2>&1',
    'tail --lines=3 /root/step19-test-build.log',
    `test -x ${cratePackage}/target/release/monochromatic-git-policy-cli`,
  ].join('\n',);
  return await detachedStep({
    chosen,
    script,
    log: buildLog,
    timeoutMilliseconds: crateBuildTimeoutMilliseconds,
    finishTimeoutMilliseconds: crateBuildTimeoutMilliseconds + finishMarginMilliseconds,
  },);
}
