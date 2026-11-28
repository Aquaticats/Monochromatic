/**
 The slot-state guest driver's build of the arm's test target: the paths its script names, the script itself, and the
 detached start whose end the host waits for.

 The build takes minutes (6m 07s on 2026-11-27), and a guest agent can lose a long command's result
 (`doc/troubleshooting/mvm-guest-agent-forgets-a-long-commands-result.md`), so the script is started under
 `setsid nohup`, appends the line that says how it ended, and the host waits for that line and pulls the log. It lives
 here rather than in the driver because that driver is at the oxlint line budget, and its arm reads the paths from here
 so neither file names one twice.

 @example
 ```ts
 await buildDetached({ chosen, });
 ```
 */
import {
  awaitDetachedGuestScript,
  startDetachedGuestScript,
} from '../bin/mvm-guest-detached-script.ts';
import type { GuestFacts, } from '../bin/mvm-guest-guest-facts.ts';

/**
 Where the crate is unpacked and built inside the guest, which every account may traverse.
 */
export const CRATE_DIR = '/opt/git-policy-cli-slot-state';

/**
 The pool this driver makes for the arm, and the file-backed vdev it stands on.
 */
export const POOL = 'git-policy-cli-slot-state';

/**
 The file-backed vdev the pool stands on.
 */
const POOL_IMAGE = '/home/git-policy-cli-slot-state-fixture/pool.img';

/**
 The folder the pool's vdev stands in, which the guest makes before it truncates the vdev.
 */
const POOL_IMAGE_FOLDER = POOL_IMAGE
  .slice(
  0,
  POOL_IMAGE
    .lastIndexOf('/',),
  );

/**
 The checkout the arm registers, clones and removes, under the guest's own root.
 */
export const CHECKOUT = '/root/slot-state-checkout';

/**
 The OpenZFS userspace the guest built, which every `zfs` the arm runs comes from.
 */
export const SBIN = '/opt/openzfs-2.4.4/sbin';

/**
 Milliseconds the guest's build of the test target may take.
 */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;

/**
 Milliseconds the host waits past the build's own bound, so the guest can write the status line the host reads.
 */
const BUILD_FINISH_MARGIN_MS = 180_000;

/**
 The log the build writes into the guest's shared folder, which the host reads its status from.
 */
const BUILD_LOG = 'slot-state-crate-build.log';

/**
 The script that unpacks the pushed archive, makes the arm's pool and builds the one test target the arm starts.

 @returns the shell script the guest runs, its lines in the order the guest runs them

 @example
 ```ts
 const script = theCrateBuildScript();
 ```
 */
export function theCrateBuildScript(): string {
  return [
    'set -eu',
    // `mvm push` lands the tarball in the guest's shared folder, not at the path it was given.
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${CRATE_DIR}`,
    `mkdir --parents ${CRATE_DIR}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${CRATE_DIR}`,
    // The arm needs a pool of its own; the checkout dataset and its clone are made on it by the arm.
    `mkdir --parents ${POOL_IMAGE_FOLDER}`,
    `if ! ${SBIN}/zpool list -H -o name ${POOL} > /dev/null 2>&1; then`,
    `  truncate --size 1G ${POOL_IMAGE}`,
    `  ${SBIN}/zpool create -f -O mountpoint=none ${POOL} ${POOL_IMAGE}`,
    'fi',
    `mkdir --parents ${CHECKOUT}`,
    `cd ${CRATE_DIR}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    // The one target the arm starts, two jobs at a time: a release rustc of this crate's tests took 6.9 GiB of this
    // guest's 8 GiB, and the kernel killed it at four jobs on 2026-11-27 (issue 6581; cargo exited 101). The lib test
    // harness the arm never runs is not built here.
    'cargo build --release --test native_binary --locked --features test-only-switches -j 2 > /root/slot-state-build.log 2>&1',
    'tail --lines=3 /root/slot-state-build.log',
  ]
    .join('\n',);
}

/**
 Start the guest's build detached and wait for the status line it appends to its own log.

 @param chosen - the run's guest and log folder

 @returns what the build wrote, which is the whole of its pulled log

 @throws Error naming the status the build ended with, or the wait that ended

 @example
 ```ts
 await buildDetached({ chosen, });
 ```
 */
export async function buildDetached({ chosen, }: { readonly chosen: GuestFacts; },): Promise<string> {
  /**
   The script this run builds the crate with, made here so the driver names no path of its own.
   */
  const script = theCrateBuildScript();
  await startDetachedGuestScript({
    chosen,
    script,
    log: BUILD_LOG,
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
    nameCheck: 'launcher',
  },);
  return await awaitDetachedGuestScript({
    chosen,
    log: BUILD_LOG,
    finishTimeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS + BUILD_FINISH_MARGIN_MS,
  },);
}
