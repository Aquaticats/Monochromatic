/**
 Push this checkout into the guest and build the crate's test target and
 executable there.

 The snapshot is the same repository-shaped tree the container gate builds from
 (`bin/native-wrapper-snapshot.ts`), so the arm compiles against the path crates
 the wrapper links, the ZFS binding among them, rather than against a copy
 somebody remembered to add.

 One build job and a swap file are what keep the build inside the guest. The
 guest is a four-vCPU image, and this crate's release test build at Cargo's
 default parallelism fills it: measured on 2026-11-25 (issue 6458), two jobs
 peaked the guest agent's own unit at 7.45 GiB of the guest's 7.75 GiB, and one
 run ended with the kernel killing a rustc at 5.15 GiB inside that unit. A kill
 would show as a failing build with its own error, because mvm's template sets the
 agent's `OOMPolicy=continue`, so the agent survives and still reports it
 (`doc/troubleshooting/mvm-guest-agent-forgets-a-long-commands-result.md`).

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
  buildFinishMarginMilliseconds,
  crateBuildTimeoutMilliseconds,
  crateDirectory,
} from './adoption-fixture.ts';
import { detachedStep, } from './detached-step.ts';

/**
 The log the guest's build writes inside the guest's shared folder.
 */
const buildLog = 'guest-crate-build.log';

/**
 Push this checkout into the guest and build the crate's tests there.

 @param chosen - the run's guest and log folder

 @throws Error naming the step that refused

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
   The guest's whole build, which unpacks the pushed archive and compiles the
   tests the arms run.
   */
  const script = [
    'set -eu',
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${crateDirectory}`,
    `mkdir --parents ${crateDirectory}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${crateDirectory}`,
    `cd ${crateDirectory}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${SWAP_GIBIBYTES} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'CARGO_BUILD_JOBS=1 cargo build --release --tests --locked --features test-only-switches > /root/adoption-build.log 2>&1',
    'tail --lines=3 /root/adoption-build.log',
  ].join('\n',);
  await detachedStep({
    chosen,
    script,
    log: buildLog,
    timeoutMilliseconds: crateBuildTimeoutMilliseconds,
    finishTimeoutMilliseconds: crateBuildTimeoutMilliseconds + buildFinishMarginMilliseconds,
  },);
}
