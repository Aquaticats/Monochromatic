/**
 Answer the owner's question of 2026-11-22 by a run: as the account the design
 delegates to, try every route that could give a dataset named
 `monochromatic-git-checkout` a row in the host's own mount table.

 The design's grants are applied to the exact datasets it names
 (`doc/planning/cli-git-zfs-worktree-design-proposal.md`, "Scoped delegation"):
 `snapshot` on the checkout dataset, `clone` on its descendants, `create,mount` on
 the clone container, `mount,userprop` on the container's descendants, and
 `mount,destroy` on the checkout's descendants. Then the account tries to create,
 rename, mount and property-set a dataset of the design's leaf name, to make a
 pool of its own, and to import one; root then runs `zfs mount -a`, which is what
 a boot's `zfs-mount.service` does, and the arm reads the mount table afterwards.

 Nothing here runs on the host: the pool is this guest's file-backed one.

 @example
 ```ts
 const pulled = await delegation({ chosen });
 ```
 */

import {
  requireMarkers,
} from '../../bin/mvm-guest-arm-log.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  guest,
} from '../../bin/mvm-guest-script.ts';



import {
  armTimeoutMilliseconds,
  delegationAccount,
  delegationCheckout,
  delegationImage,
  delegationPool,
  expectedDelegationMarkers,
  openZfsPath,
  openZfsSbin,
} from './adoption-fixture.ts';

/**
 The log the delegation arm writes inside the run's log folder.
 */
const delegationLog = 'guest-delegation.log';

/**
 The library folder the guest's OpenZFS 2.4.4 userspace needs.
 */
const openZfsLibrary = '/opt/openzfs-2.4.4/lib';

/**
 Run the delegation attempts and refuse a log that does not prove them.

 @param chosen - the run's guest and log folder

 @returns the arm's log, as the guest wrote it

 @throws Error naming the marker the pulled log lacks

 @example
 ```ts
 const pulled = await delegation({ chosen });
 ```
 */
export async function delegation({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  /**
   The attempts, in the order the design's grants allow or refuse them: the
   layout and its grants first, then each route, then root's own `zfs mount -a`.
   */
  const attempts = [
    'set -eu',
    `export PATH=${openZfsPath}`,
    `export LD_LIBRARY_PATH=${openZfsLibrary}`,
    `ACCOUNT=${delegationAccount}`,
    `POOL=${delegationPool}`,
    `IMAGE=${delegationImage}`,
    'IMAGE_SECOND=/home/git-policy-cli-delegation-second.img',
    'POOL_SECOND="$POOL-second"',
    `CHECKOUT=${delegationCheckout}`,
    'HOME_DS="$POOL/ROOT/cos/home"',
    'CHECKOUT_DS="$HOME_DS/monochromatic-git-checkout"',
    'CONTAINER="$HOME_DS/monochromatic-git-worktree-clone-datasets"',
    `as_account() { runuser -u "$ACCOUNT" -- env LD_LIBRARY_PATH=${openZfsLibrary} ${openZfsSbin}/zfs "$@"; }`,
    'as_account_ls() { runuser -u "$ACCOUNT" -- ls -a "$1"; }',
    `as_account_zpool() { runuser -u "$ACCOUNT" -- env LD_LIBRARY_PATH=${openZfsLibrary} ${openZfsSbin}/zpool "$@"; }`,
    String.raw`rows() { findmnt --noheadings --output TARGET,SOURCE --type zfs | grep --fixed-strings monochromatic-git | sort | tr "\n" ";"; }`,
    'say() { echo "DELEGATION $1=$2 rows=$(rows)"; }',
    // The design's layout and its grants.
    'zpool destroy -f "$POOL" > /dev/null 2>&1 || true',
    'rm --force "$IMAGE"',
    'truncate --size 512M "$IMAGE"',
    'zpool create -f -O mountpoint=none "$POOL" "$IMAGE"',
    'zfs create -o mountpoint=none "$POOL/ROOT"',
    'zfs create -o mountpoint=none "$POOL/ROOT/cos"',
    'zfs create -o mountpoint=none "$HOME_DS"',
    'zfs create -o mountpoint="$CHECKOUT" "$CHECKOUT_DS"',
    'mount --make-private "$CHECKOUT"',
    'zfs create -o mountpoint=legacy -o canmount=off -o setuid=off -o devices=off "$CONTAINER"',
    // The design's grants exactly as "Scoped delegation" names them: local where it says local, descendants where it
    // says descendants. The integration pass found this arm's earlier form granted everything to descendants, which is
    // wider than the design.
    'zfs allow -l -u "$ACCOUNT" snapshot "$CHECKOUT_DS"',
    'zfs allow -d -u "$ACCOUNT" clone,destroy,mount "$CHECKOUT_DS"',
    'zfs allow -l -u "$ACCOUNT" create,mount "$CONTAINER"',
    'zfs allow -d -u "$ACCOUNT" mount,destroy,userprop "$CONTAINER"',
    'echo "DELEGATION grants:"; zfs allow "$CHECKOUT_DS"; zfs allow "$CONTAINER"',
    // A: create under the container, which `create` on it allows.
    'if as_account create "$CONTAINER/plain" > /root/delegation-create.log 2>&1; then say create-under-container allowed; else say create-under-container refused; sed "s/^/  /" /root/delegation-create.log; fi',
    // B: create the design's leaf name with a mountpoint the account chooses.
    'if as_account create -o mountpoint=/home/account-second "$CONTAINER/monochromatic-git-checkout" > /root/delegation-create-mountpoint.log 2>&1; then say create-with-mountpoint allowed; else say create-with-mountpoint refused; sed "s/^/  /" /root/delegation-create-mountpoint.log; fi',
    // C: snapshot the checkout, which it is delegated.
    'if as_account snapshot "$CHECKOUT_DS@account-origin" > /root/delegation-snapshot.log 2>&1; then say snapshot-of-checkout allowed; else say snapshot-of-checkout refused; sed "s/^/  /" /root/delegation-snapshot.log; fi',
    // D: clone it to the design's leaf name, which `clone` and `create` allow.
    'if as_account clone "$CHECKOUT_DS@account-origin" "$CONTAINER/monochromatic-git-checkout" > /root/delegation-clone.log 2>&1; then say clone-named-the-leaf allowed; else say clone-named-the-leaf refused; sed "s/^/  /" /root/delegation-clone.log; fi',
    'zfs get -H -o property,value mountpoint,canmount,origin "$CONTAINER/monochromatic-git-checkout" 2>/dev/null || true',
    // E: rename the plain dataset onto the design's leaf name.
    'if as_account rename "$CONTAINER/plain" "$CONTAINER/monochromatic-git-checkout-second" > /root/delegation-rename.log 2>&1; then say rename-onto-the-leaf allowed; else say rename-onto-the-leaf refused; sed "s/^/  /" /root/delegation-rename.log; fi',
    // F: mount it into the host's namespace as the account.
    'if as_account mount "$CONTAINER/monochromatic-git-checkout" > /root/delegation-mount.log 2>&1; then say mount-as-account allowed; else say mount-as-account refused; sed "s/^/  /" /root/delegation-mount.log; fi',
    // G and H: the properties a mount needs, which it is not delegated.
    'if as_account set mountpoint=/home/account-third "$CONTAINER/monochromatic-git-checkout" > /root/delegation-set-mountpoint.log 2>&1; then say set-mountpoint-as-account allowed; else say set-mountpoint-as-account refused; sed "s/^/  /" /root/delegation-set-mountpoint.log; fi',
    'if as_account set canmount=on "$CONTAINER/monochromatic-git-checkout" > /root/delegation-set-canmount.log 2>&1; then say set-canmount-as-account allowed; else say set-canmount-as-account refused; sed "s/^/  /" /root/delegation-set-canmount.log; fi',
    // I and J: a pool of its own, and one to import.
    // The image a pool of the account's own would stand on is made readable and writable by it first,
    // so the attempt answers the capability rather than a missing file.
    'rm --force "$IMAGE_SECOND"',
    'truncate --size 256M "$IMAGE_SECOND"',
    'chmod 0666 "$IMAGE_SECOND"',
    'if as_account_zpool create -f "$POOL_SECOND" "$IMAGE_SECOND" > /root/delegation-pool.log 2>&1; then say create-pool-as-account allowed; else say create-pool-as-account refused; sed "s/^/  /" /root/delegation-pool.log; fi',
    // Root makes the same second pool and exports it, so the account's import attempt is a capability question rather than a name one.
    'zpool create -f -O mountpoint=none "$POOL_SECOND" "$IMAGE_SECOND"',
    'zpool export "$POOL_SECOND"',
    'if as_account_zpool import -d /home "$POOL_SECOND" > /root/delegation-import.log 2>&1; then say import-pool-as-account allowed; else say import-pool-as-account refused; sed "s/^/  /" /root/delegation-import.log; fi',
    'zpool import -d /home "$POOL_SECOND" || true',
    'zpool destroy -f "$POOL_SECOND" || true',
    // K: root mounts every dataset the pool holds, which is what a boot does.
    'echo "DELEGATION before-root-mount-a: $(rows)"',
    'zfs mount -a',
    'if rows | grep --quiet --fixed-strings "/home/account"; then say root-mount-a mounted-the-accounts; else say root-mount-a mounted-nothing-of-the-accounts; fi',
    'zfs list -H -o name,mountpoint,canmount -r "$POOL" | sed "s/^/  /"',
    'echo "DELEGATION done"',
  ].join('\n',);
  /**
   The arm's log, written where the run's own logs go.
   */
  const pulled = await guest({
    chosen,
    script: attempts,
    log: delegationLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
  requireMarkers({
    text: pulled,
    markers: expectedDelegationMarkers,
    subject: 'the delegation log',
    because: 'the guest ran the attempt without proving it',
  },);
  return pulled;
}
