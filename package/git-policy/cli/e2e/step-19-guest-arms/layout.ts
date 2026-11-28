/**
 The guest side of the step 19 layout: the design's two datasets on a file-backed pool, the dataset whose mountpoint lies
 inside the checkout, the store the clone records live in, the root-owned configuration, the grants the add route's
 preflight reads, one Git history at the checkout path, and the provider units a boot imports and mounts the pool with.

 Every path here is written literally, because a fixture folder is removed by this driver and a deletion with a variable
 in it is the shape `AGENTS.md` rule `RLW` refuses.

 The provider units are the OpenZFS release's own texts, from the tree the recipe built: the design's provider is
 `zfs-mount.service` (V1), and a boot that never ran it is what the failed-boot arms are about.

 @example
 ```ts
 await guestLayout({ chosen });
 ```
 */

import type { GuestFacts, } from '../../bin/mvm-guest-guest-facts.ts';
import { guest, } from '../../bin/mvm-guest-script.ts';

import {
  checkoutDataset,
  checkoutPath,
  containerDataset,
  fixtureRoot,
  nestedDataset,
  nestedMountpoint,
  openZfsLibrary,
  openZfsPath,
  openZfsSource,
  pool,
  slot,
} from './fixture.ts';

/**
 The folder the store's slots recorded inside the guest, which the add route writes.
 */
export const storeFolder = '/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities';

/**
 The checkout record the root mode writes, which the descendant mode reads.
 */
export const checkoutRecord = '/var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json';

/**
 The root-owned handler configuration.
 */
export const configurationPath = '/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc';

/**
 Build the design's layout on the guest's own disk, replacing whatever an earlier run of this driver left.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await guestLayout({ chosen });
 ```
 */
export async function guestLayout({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The design's layout, dataset by dataset, with the provider's own units and the history a clone may be made of.
   */
  const script = [
    'set -eu',
    `export PATH=${openZfsPath}`,
    `export LD_LIBRARY_PATH=${openZfsLibrary}`,
    `zpool destroy -f ${pool} > /dev/null 2>&1 || true`,
    `rm --recursive --force ${fixtureRoot}`,
    `mkdir --parents ${fixtureRoot}/home`,
    `chmod 0755 ${fixtureRoot} ${fixtureRoot}/home`,
    `truncate --size 2G ${fixtureRoot}/pool.img`,
    `head --bytes 32 /dev/urandom > ${fixtureRoot}/checkout-key`,
    `sync ${fixtureRoot}/pool.img ${fixtureRoot}/checkout-key`,
    // The cachefile is the one the release's own `zfs-import-cache.service` reads, so a boot imports this pool without a
    // person naming it.
    `zpool create -o cachefile=${openZfsSource}/etc/zfs/zpool.cache -O mountpoint=none ${pool} ${fixtureRoot}/pool.img`,
    `zfs create -o mountpoint=none ${pool}/ROOT`,
    `zfs create -o mountpoint=none ${pool}/ROOT/home`,
    // The checkout dataset carries the project's own `snapdir` value (2026-11-23) and the key of the encrypted pool's
    // dataset, whose file a boot loads without a person.
    `zfs create -o mountpoint=${checkoutPath} -o snapdir=disabled -o encryption=aes-256-gcm -o keyformat=raw -o keylocation=file://${fixtureRoot}/checkout-key ${checkoutDataset}`,
    `mount --make-rprivate ${checkoutPath}`,
    `zfs create -o mountpoint=legacy -o canmount=off -o setuid=off -o devices=off -o snapdir=disabled ${containerDataset}`,
    // The dataset inside the checkout is `canmount=noauto`, which V4 sets so `zfs mount -a` leaves it to the unit that
    // runs after the gate.
    `zfs create -o mountpoint=${nestedMountpoint} -o canmount=noauto ${nestedDataset}`,
    // `canmount=noauto` already kept it unmounted, and an unmount of a dataset that is not mounted is a refusal.
    `zfs unmount ${nestedDataset} || true`,
    `zfs create -o mountpoint=/var/lib/monochromatic-git-policy-cli ${pool}/store`,
    `git -C ${checkoutPath} init --quiet --initial-branch=main`,
    `git -C ${checkoutPath} -c user.name=step19 -c user.email=step19@example.invalid commit --quiet --allow-empty -m 'the step 19 guest layout'`,
    `mkdir --parents ${checkoutPath}/.claude/worktrees`,
    `chmod 0755 ${checkoutPath}/.claude ${checkoutPath}/.claude/worktrees`,
    // The design's scoped delegation, which the add route's preflight refuses an add without.
    `zfs allow -l -u 0 snapshot ${checkoutDataset}`,
    `zfs allow -d -u 0 clone,mount,destroy ${checkoutDataset}`,
    `zfs allow -l -u 0 create,mount ${containerDataset}`,
    `zfs allow -d -u 0 mount,destroy,userprop ${containerDataset}`,
    'mkdir --parents /etc/monochromatic-git-policy-cli',
    `cat > ${configurationPath} <<'JSONC'`,
    '{',
    `  // written by plan step 19's guest layout`,
    '  "cloneMounts": {',
    `    "checkout": "${checkoutPath}",`,
    `    "checkoutDataset": "${checkoutDataset}",`,
    `    "parent": "${containerDataset}"`,
    '  }',
    '}',
    'JSONC',
    `chmod 0644 ${configurationPath}`,
    `chown root:root ${configurationPath}`,
    // The provider: the release's own texts, and the targets that pull them at boot.
    `install --mode=0644 ${openZfsSource}/etc/systemd/system/zfs-mount.service /etc/systemd/system/zfs-mount.service`,
    `install --mode=0644 ${openZfsSource}/etc/systemd/system/zfs-import-cache.service /etc/systemd/system/zfs-import-cache.service`,
    `install --mode=0644 ${openZfsSource}/etc/systemd/system/zfs-import.target /etc/systemd/system/zfs-import.target`,
    `install --mode=0644 ${openZfsSource}/etc/systemd/system/zfs.target /etc/systemd/system/zfs.target`,
    String.raw`printf "zfs\n" > /etc/modules-load.d/zfs.conf`,
    'systemctl daemon-reload',
    'systemctl enable zfs.target zfs-import-cache.service zfs-mount.service',
    `test -s ${openZfsSource}/etc/zfs/zpool.cache`,
    `zfs list -H -o name,mountpoint,mounted,canmount -r ${pool}`,
    `zfs get -H -o property,value,source snapdir ${containerDataset} ${checkoutDataset}`,
    `findmnt --list --noheadings --output TARGET,SOURCE --type zfs | grep --fixed-strings "${checkoutPath}"`,
    `echo "STEP19-LAYOUT checkout=${checkoutPath} container=${containerDataset} nested=${nestedMountpoint} slot=${slot}"`,
  ].join('\n',);
  await guest({
    chosen,
    script,
    log: 'guest-step19-layout.log',
    timeoutMilliseconds: 600_000,
  },);
}

/**
 Install the release build at the root-owned path every unit line names, link it for ZED, and print the words that carry
 no mode.

 This is the runbook's own step, run against the build this guest made, so the units are parsed and started against the
 bytes a person would install.

 @param chosen - the run's guest and log folder

 @param releaseBinary - the built release executable inside the guest

 @returns the words that carry no mode, which name the release the installed bytes came from

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await installTheProgram({ chosen, releaseBinary: '/opt/…/target/release/monochromatic-git-policy-cli' });
 ```
 */
export async function installTheProgram({
  chosen,
  releaseBinary,
}: {
  readonly chosen: GuestFacts;
  readonly releaseBinary: string;
},): Promise<string> {
  /**
   The runbook's install step, run against the build this guest made.
   */
  const script = [
    'set -eu',
    'install --directory --mode=0755 /usr/libexec/monochromatic-git-policy-cli',
    'install --directory --mode=0755 /etc/zfs/zed.d',
    `install --mode=0755 --owner=root --group=root ${releaseBinary} /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets`,
    'ln --symbolic --force /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets /etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets',
    '/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli zfs-root --version',
  ].join('\n',);
  return await guest({
    chosen,
    script,
    log: 'guest-step19-program.log',
    timeoutMilliseconds: 300_000,
  },);
}

/**
 Record the checkout with the wrapper's own root mode, which proves the path, dataset, GUID, container and pool against
 the kernel before it writes them.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await recordTheCheckout({ chosen });
 ```
 */
export async function recordTheCheckout({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The root mode that proves the checkout against the kernel before it writes what it found.
   */
  const script = [
    'set -eu',
    `/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli zfs-root record-checkout --checkout ${checkoutPath}`,
    `cat ${checkoutRecord}`,
    `ls -la ${storeFolder} 2>/dev/null || true`,
  ].join('\n',);
  await guest({
    chosen,
    script,
    log: 'guest-step19-record-checkout.log',
    timeoutMilliseconds: 300_000,
  },);
}
