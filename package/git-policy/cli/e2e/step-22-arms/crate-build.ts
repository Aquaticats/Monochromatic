/**
 Build this crate's test target inside the guest from a snapshot of the worktree,
 so the arm compiles against the path crates the wrapper links rather than
 against a copy somebody remembered to add.

 The tree is unpacked over whatever the guest already holds, so a second run of
 the arm reuses the build's target folder instead of compiling the whole crate
 again: a renamed source file would linger, and this arm only adds files. One
 `rustc` for a crate this size wants more than five gigabytes and the guest has
 eight, so one job and a swap file are what make the build fit; the other guest
 drivers measure the same bound.

 @example
 ```ts
 await buildInGuest({ chosen });
 ```
 */

import type { GuestRun, } from '../guest-host/guest-run.ts';
import { swapGibibytes, } from '../guest-host/guest-host-bounds.ts';
import { pushWorktreeSnapshot, } from '../guest-host/host-side-commands.ts';
import { detachedStep, } from './detached-step.ts';
import {
  crateBuildTimeoutMilliseconds,
  crateDirectory,
  longFinishMarginMilliseconds,
} from './measurement-fixture.ts';

/**
 The log the guest's build writes inside the guest's shared folder.
 */
const buildLog = 'guest-crate-build.log';

/**
 Push this checkout into the guest and build the crate's test target there.

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
  readonly chosen: GuestRun;
},): Promise<void> {
  await pushWorktreeSnapshot({ chosen, },);
  /**
   The guest's whole build, which unpacks the pushed archive and compiles the
   test target the arm runs.
   */
  const script = [
    'set -eu',
    // `mvm push` lands every file in the guest's shared folder, so the copy the
    // build unpacks is taken from there and a missing one refuses.
    'test -f /mnt/shared/subject.tar.gz',
    'cp /mnt/shared/subject.tar.gz /root/subject.tar.gz',
    // The tree is unpacked over whatever the guest already holds, so a second
    // run of the arm reuses the build's target folder instead of compiling the
    // whole crate again; a renamed source file would linger, and this arm only
    // adds files.
    `mkdir --parents ${crateDirectory}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${crateDirectory}`,
    `cd ${crateDirectory}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'df --human-readable / /mnt/shared',
    'free',
    // One rustc for a crate this size wants more than five gigabytes and the guest has eight, so one job and a swap
    // file are what make the build fit; the customer's own guest-arm drivers measure the same bound.
    `if ! swapon --show | grep --quiet /swapfile; then fallocate --length ${swapGibibytes} /swapfile; chmod 600 /swapfile; mkswap /swapfile; swapon /swapfile; fi`,
    'free',
    'CARGO_BUILD_JOBS=1 cargo build --release --tests --locked > /root/step22-build.log 2>&1',
    'tail --lines=3 /root/step22-build.log',
  ].join('\n',);
  await detachedStep({
    chosen,
    script,
    log: buildLog,
    timeout: crateBuildTimeoutMilliseconds,
    finishTimeout: crateBuildTimeoutMilliseconds + longFinishMarginMilliseconds,
  },);
}
