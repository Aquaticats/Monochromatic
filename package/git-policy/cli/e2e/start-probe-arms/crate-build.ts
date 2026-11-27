/**
 Build this crate's handler inside the guest from a snapshot of the worktree, so
 the arm compiles against the path crates the wrapper links rather than against a
 copy somebody remembered to add.

 Two builds happen here, because two arms need them: the release the shipped
 modes run, and the same build with the probe's raised-deadline feature, which is
 the only build the one control whose point is a step that finishes after the
 shipped bound may run.

 @example
 ```ts
 await buildInGuest({ chosen });
 ```
 */

import type { GuestRun, } from '../guest-host/guest-run.ts';
import { guest, } from '../guest-host/guest-side-scripts.ts';
import { pushWorktreeSnapshot, } from '../guest-host/host-side-commands.ts';
import {
  crateBuildTimeoutMilliseconds,
  crateDirectory,
} from './probe-fixture.ts';

/**
 The log the guest's build writes inside the run's log folder.
 */
const buildLog = 'guest-crate-build.log';

/**
 Push this checkout into the guest and build the handler there.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status

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
   The guest's two builds: the release the modes run, and the raised-deadline one
   into a target folder of its own, so neither replaces the other's bytes.
   */
  const script = [
    'set -eu',
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${crateDirectory}`,
    `mkdir --parents ${crateDirectory}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${crateDirectory}`,
    `cd ${crateDirectory}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'cargo build --release --locked --features test-only-switches --bins > /root/probe-build.log 2>&1',
    'cargo build --release --locked --bin monochromatic-git-policy-cli --features test-only-switches,start-probe-raised-deadline --target-dir target/raised-deadline > /root/probe-build-raised.log 2>&1',
    'tail --lines=3 /root/probe-build.log',
    'tail --lines=3 /root/probe-build-raised.log',
  ].join('\n',);
  await guest({
    chosen,
    script,
    log: buildLog,
    timeout: crateBuildTimeoutMilliseconds,
  },);
}
