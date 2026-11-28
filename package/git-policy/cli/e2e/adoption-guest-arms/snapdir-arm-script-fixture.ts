/**
 The guest script the `snapdir` arm runs, kept where a unit test holds its text without a guest.

 `snapdir-arm.ts` runs this script and judges the log it leaves; what the script says is here, so a test can hand the
 pattern it greps the listing for to a real `grep` beside the names a listing of the checkout holds.

 @example
 ```ts
 const script = snapdirScript();
 ```
 */

import {
  delegationAccount,
  delegationPool,
  openZfsPath,
  openZfsSbin,
} from './adoption-fixture.ts';

/**
 The library folder the guest's OpenZFS 2.4.4 userspace needs.
 */
const openZfsLibrary = '/opt/openzfs-2.4.4/lib';

/**
 The checkout the arm mounts its own dataset at.
 */
const snapdirCheckout = '/home/snapdir-checkout';

/**
 The image the arm's own pool stands on.
 */
const snapdirImage = '/home/snapdir-matrix.img';

/**
 The script the `snapdir` arm runs inside the guest: one fresh pool, and each value of the property read as the account
 the design delegates to.

 @returns the whole shell script, one line per step

 @example
 ```ts
 const script = snapdirScript();
 ```
 */
export function snapdirScript(): string {
  return [
    'set -eu',
    `export PATH=${openZfsPath}`,
    `export LD_LIBRARY_PATH=${openZfsLibrary}`,
    `ACCOUNT=${delegationAccount}`,
    `POOL=${delegationPool}`,
    `IMAGE=${snapdirImage}`,
    `CHECKOUT=${snapdirCheckout}`,
    'DS="$POOL/ROOT/cos/home/monochromatic-git-checkout"',
    `as_account() { runuser -u "$ACCOUNT" -- env LD_LIBRARY_PATH=${openZfsLibrary} ${openZfsSbin}/zfs "$@"; }`,
    'as_account_ls() { runuser -u "$ACCOUNT" -- ls -a "$1"; }',
    'snapshot_rows() { findmnt --list --noheadings --output SOURCE --type zfs | grep --count "@" || true; }',
    String.raw`leaf_rows() { findmnt --list --noheadings --output TARGET,SOURCE --type zfs | grep --fixed-strings "monochromatic-git-checkout" | tr "\n" ";"; }`,
    'snapdir_rows_cleanup() { findmnt --list --noheadings --output TARGET,SOURCE --type zfs | grep --fixed-strings "@account-origin" | sed "s/ .*//" | xargs --no-run-if-empty umount || true; }',
    'zpool destroy -f "$POOL" > /dev/null 2>&1 || true',
    'rm --force "$IMAGE"',
    'truncate --size 512M "$IMAGE"',
    'zpool create -f -O mountpoint=none "$POOL" "$IMAGE"',
    'zfs create -o mountpoint=none "$POOL/ROOT"',
    'zfs create -o mountpoint=none "$POOL/ROOT/cos"',
    'zfs create -o mountpoint=none "$POOL/ROOT/cos/home"',
    'zfs create -o mountpoint="$CHECKOUT" "$DS"',
    'mount --make-private "$CHECKOUT"',
    'zfs snapshot "$DS@account-origin"',
    'zfs allow -l -u "$ACCOUNT" snapshot "$DS"',
    'zfs allow -d -u "$ACCOUNT" clone,destroy,mount "$DS"',
    // The guest's own release accepts which values?
    'if zfs set snapdir=disabled "$DS" > /root/snapdir-disabled-set.log 2>&1; then echo "SNAPDIR release has-disabled=true"; else echo "SNAPDIR release has-disabled=false"; cat /root/snapdir-disabled-set.log; exit 1; fi',
    'zfs inherit snapdir "$DS"',
    // Each value is measured from a fresh mount, because a `.zfs` dentry the kernel already resolved keeps the
    // directory reachable until the dataset is mounted again, whatever the property says afterwards.
    'for value in visible hidden disabled; do',
    '  zfs unmount "$DS" > /dev/null 2>&1 || true',
    '  snapdir_rows_cleanup',
    '  zfs set "snapdir=$value" "$DS"',
    '  zfs mount "$DS"',
    '  mount --make-private "$CHECKOUT"',
    '  read_value=$(zfs get -H -o value snapdir "$DS")',
    '  test "$read_value" = "$value"',
    '  if as_account_ls "$CHECKOUT" > /root/snapdir-list.log 2>&1 && grep --quiet "^.zfs$" /root/snapdir-list.log; then listed=true; else listed=false; fi',
    '  before=$(snapshot_rows)',
    '  if as_account_ls "$CHECKOUT/.zfs/snapshot/account-origin" > /root/snapdir-enter.log 2>&1; then entered=true; else entered=false; fi',
    '  after=$(snapshot_rows)',
    '  if test "$after" -gt "$before"; then grown=true; else grown=false; fi',
    '  echo "SNAPDIR $value listed=$listed entered=$entered mounts-grown=$grown enter-answer=$(head --lines=1 /root/snapdir-enter.log 2>/dev/null || true)"',
    '  # Entering a snapshot directory is what makes the kernel mount it, so the two answers must agree in every state.',
    '  test "$entered" = "$grown" || { echo "SNAPDIR $value entered=$entered grown=$grown disagree"; exit 1; }',
    '  snapdir_rows_cleanup',
    'done',
    // The account cannot set the property back: the route's grants do not name it.
    'if as_account set snapdir=visible "$DS" > /root/snapdir-account-set.log 2>&1; then echo "SNAPDIR account-cannot-set-it-back=false"; else echo "SNAPDIR account-cannot-set-it-back=true"; sed "s/^/  /" /root/snapdir-account-set.log; fi',
    'zfs get -H -o property,value snapdir "$DS"',
    // What the route's other release does: the distro module in this guest, if it is installed.
    'if test -f /usr/sbin/zfs; then /usr/sbin/zfs --version || true; else echo "SNAPDIR distro-userspace absent"; fi',
    'zpool destroy -f "$POOL"',
    'rm --force "$IMAGE"',
    'echo "SNAPDIR done"',
  ].join('\n',);
}
