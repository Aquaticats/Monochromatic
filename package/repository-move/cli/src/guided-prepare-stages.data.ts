/**
 The stages of `prepare`: root login, the 860 mount option, the install of the repository link check, the fixture
 test, the inventory, the space check, the folders, and the first pass. The repository stays writable; nothing is
 frozen and no commit is refused.

 @module
 */

import { fixtureStages, } from './guided-loop-image-stages.data.ts';
import type {
  GuidedConfig,
  Stage,
} from './guided-types.ts';
import {
  hookPreflight,
  hookUnitEdit,
} from './guided-stage-hooks.ts';
import { externalNames, } from './guided-holders.ts';
import { SUBVOLUME_REMOVAL_OPTION, } from './guided-unit-edit.ts';

/**
 The argument list of the copy tool for the real move.

 @param c - the configuration
 
 @param inventory - the inventory file
 
 @returns the arguments shared by every pass

 @example
 ```ts
 import { realArguments, } from './guided-prepare-stages.data.ts';
 
 const result = realArguments({ c, inventory, },);
 ```
 */
export function realArguments(
  {
    c,
    inventory,
  }: {
    readonly c: GuidedConfig;
    readonly inventory: string;
  },
): string {
  return `--main=${c.repo} --snapshots=${c.snapshots} --received=${c.received} --final=${c.finalPath} --state=${c.moveState} --inventory=${inventory} --inventory-main=${c.inventoryMain} --min-free-gib=${String(c.minFreeGiB,)}`;
}

/**
 The stages of `prepare`.

 @param c - the configuration
 
 @param fixtureFolder - the fixture folder, or a placeholder when it does not exist yet
 
 @returns the stages in order

 @example
 ```ts
 import { prepareStages, } from '\@monochromatic-dev/repository-move-cli';
 
 const result = await prepareStages({ c, fixtureFolder, },);
 ```
 */
export async function prepareStages(
  {
    c,
    fixtureFolder,
  }: {
    readonly c: GuidedConfig;
    readonly fixtureFolder: string;
  },
): Promise<readonly Stage[]> {
  /**
   The external worktree folder names.
   */
  const names = await externalNames(c,);
  /**
   The external worktree folder names, space separated.
   */
  const nameList = names.join(' ',);
  /**
   The arguments shared by the commands of this step.
   */
  const args = realArguments({
    c,
    inventory: c.inventory,
  },);
  /**
   The copy tool command, run as the user.
   */
  const copy = `${c.nodePath} ${c.copyTool}`;
  /**
   The copy tool command, run through sudo with the absolute path of node.
   */
  const rootCopy = `sudo ${c.nodePath} ${c.copyTool}`;
  /**
   The installed link check command.
   */
  const link = `${c.nodePath} ${c.linkBinFolder}`;
  return [
    {
      id: 'sudo-prepare',
      title: 'Root access',
      says: 'The root steps use sudo, with the absolute path of node. This checks that sudo works without a prompt; nothing else runs.',
      command: 'sudo --non-interactive true',
      check: 'sudo --non-interactive true',
      checkText: 'sudo works without a prompt',
      rollback: 'nothing changed',
      noMark: true,
      root: 'yes',
      unprivileged: 'none: this stage only proves the root route used by the stages that need it',
    },
    {
      id: 'i1-install-link-check',
      title: 'Install the repository link check outside the repository',
      says: 'Build the repository link check (work item W1) and copy its bins outside the repository, so the check itself never dangles when the repository is away.',
      command: `cd ${c.repo} && mise run //package/repository-link/check:build && mise run //package/repository-link/check:install:outside-repository`,
      check: `test -e ${c.linkBinFolder}/check.mjs && head --lines=1 ${c.linkBinFolder}/check.mjs | grep --fixed-strings --quiet '#!/usr/bin/env node' && test -L ${c.linkBinFolder}/repository-link-check`,
      checkText: 'the installed copy exists, starts with a node shebang, and the bin links exist',
      rollback: `the install only adds files below ${c.linkBinFolder}; remove that folder yourself to undo`,
      root: 'no',
      unprivileged: 'not needed: installed as the user below the data folder',
    },
    {
      id: 'i2-record-before',
      title: 'Record the repository link identity, phase before',
      says: 'Record the identity the check enforces while the repository is still a real folder on the NVMe (phase before), so tools can be installed ahead of the move. The cut-over records phase after.',
      command: `${link}/record.mjs --phase=before --link-path=${c.repo} --mount-point=${c.drive}`,
      check: `${link}/check.mjs | grep --fixed-strings 'ready (before the move)'`,
      checkText: 'the installed check says ready (before the move)',
      rollback: 'the identity file is rewritten by the next record run',
      root: 'no',
      unprivileged: 'not needed: written as the user below the config folder',
    },
    ...fixtureStages({
      c,
      fx: fixtureFolder,
    },),
    {
      id: 'r0-inventory',
      title: 'Inventory R0',
      says: "R0: the filesystem's own list of every subvolume of the NVMe filesystem, with UUIDs, and any mount inside the checkout. The copy tool rebuilds each path from the list's parent column and keeps the ones below the checkout.",
      command: `sudo btrfs subvolume list -u -q /var/home > ${c.inventory}
sudo findmnt --list --output TARGET,SOURCE,FSTYPE | grep --fixed-strings ${c.repo} || echo 'no mount inside the tree'`,
      check: `test -s ${c.inventory} && grep --fixed-strings --quiet ' uuid ' ${c.inventory} && grep --fixed-strings 'no mount inside the tree' "$OUTPUT_FILE"`,
      checkText: 'the list is not empty, has UUID columns, and no mount sits inside the tree (a mount needs your written disposition first)',
      rollback: 'read-only; the list file can be regenerated',
      root: 'yes',
      unprivileged: 'none: btrfs subvolume list needs root (tree search is refused to users, measured: Operation not permitted)',
    },
    {
      id: 'r0-reconcile',
      title: 'Inventory reconciles with Git',
      says: "Reconcile: the copy tool compares the filesystem's list with Git's registered worktrees and refuses on a difference in either direction. This is print mode: nothing is copied.",
      command: `${copy} --pass=first ${args} > ${c.wiz}/reconcile-print.txt`,
      check: `grep --extended-regexp 'inventory agrees: [0-9]+ nested subvolumes' "$OUTPUT_FILE"`,
      checkText: "the tool reports 'inventory agrees'",
      rollback: 'read-only. On a refusal the tool names each subvolume Git does not know or each registered one that is not a subvolume; give each a written disposition before going on',
      root: 'no',
      unprivileged: 'not needed: print mode runs as the user',
    },
    {
      id: 'p3a-footprint',
      title: 'Pre-flight: checkout footprint',
      says: 'Pre-flight 1 of 3: the footprint of the nested checkout with extent sharing counted once, as root so unreadable files are included. The plan measured about 12.5 minutes for it.',
      command: `sudo systemd-run ${c.rootScope} timeout --kill-after=${String(c.killGraceSeconds,)} ${String(c.measureTimeoutSeconds,)} btrfs filesystem du --summarize --raw ${c.repo} | tee ${c.wiz}/main-du.txt`,
      check: `awk 'END { exit !(NF >= 4 && $2 + $3 > 0) }' ${c.wiz}/main-du.txt`,
      checkText: 'the output has a total row with exclusive plus set shared above zero',
      rollback: 'read-only',
      timer: {
        file: 'du-seconds',
        mode: 'set',
      },
      root: 'yes',
      unprivileged: 'none: btrfs filesystem du needs root; a plain du as the user counts shared extents once per file and overestimates, which is safe for the space rule but is not the plan\'s figure',
    },
    {
      id: 'p3b-external',
      title: 'Pre-flight: external worktrees',
      says: 'Pre-flight 2 of 3: the on-disk size of each external worktree (plain directories on the NVMe, nothing shared on the 860).',
      command: `for p in ${names.map(function toPath(name,) {
        return `${c.externalWorktrees}/${name}`;
      },)
        .join(' ',)}; do du --summarize --one-file-system --block-size=1 -- "$p"; done | tee ${c.wiz}/external-du.txt`,
      check: `test -s ${c.wiz}/external-du.txt`,
      checkText: 'one size row per external folder',
      rollback: 'read-only',
      root: 'no',
      unprivileged: 'not needed: du of the user\'s own folders runs as the user (a root-owned file inside is counted only if readable; the root run is the plan\'s figure)',
    },
    {
      id: 'p3c-rule',
      title: 'Pre-flight: the rule',
      says: `Pre-flight 3 of 3: the rule from the plan. The 860's free space after the estimate plus a quarter margin must stay at least ${String(c.minFreeGiB,)} GiB, and the NVMe at least ${String(c.nvmeMinFreeGiB,)} GiB. A refusal here is the rule working: free space or the estimate changed since the plan was written.`,
      command: 'true',
      internal: hookPreflight,
      check: `grep --fixed-strings 'pre-flight passed' "$OUTPUT_FILE"`,
      checkText: "the rule prints 'pre-flight passed'",
      rollback: "read-only. If refused, free space on the 860 or lower the scope (the plan's scope four) before running prepare again",
      root: 'no',
      unprivileged: 'not needed: statfs as the user',
    },
    {
      id: 'p4a-folders',
      title: 'Create the folders',
      says: 'The folders from the plan: the snapshot folder on the NVMe, and on the 860 the 0700 repository folder, the received folder and the external-worktrees folder. The 860 must be mounted first.',
      command: `mountpoint --quiet ${c.drive}
test "$(findmnt --noheadings --output UUID --target ${c.drive})" != "$(findmnt --noheadings --output UUID --target ${c.homeDir})"
[ -d ${c.snapshots} ] || mkdir --mode=0700 ${c.snapshots}
[ -d ${c.destRoot} ] || sudo mkdir --mode=0700 ${c.destRoot}
sudo chown ${c.userName}:${c.groupName} ${c.destRoot}
[ -d ${c.received} ] || mkdir --mode=0700 ${c.received}
[ -d ${c.externalFinal} ] || mkdir --mode=0700 ${c.externalFinal}
stat --format='%U:%G %a %n' ${c.snapshots} ${c.destRoot} ${c.received} ${c.externalFinal}`,
      check: `test "$(stat --format='%U:%G %a' ${c.snapshots} ${c.destRoot} ${c.received} ${c.externalFinal} | sort --unique)" = '${c.userName}:${c.groupName} 700'`,
      checkText: `all four folders are ${c.userName}:${c.groupName} mode 700`,
      rollback: `empty folders only: rmdir ${c.received} ${c.externalFinal} ${c.destRoot} ${c.snapshots} (yours to run; this runner deletes nothing)`,
      root: 'yes',
      unprivileged: `only the first folder below the 860's mount point needs root, because the 860's top level is user:wheel mode 777 (measured in the plan), so the user could make ${c.destRoot} too; sudo is used only for the chown that makes it user-owned and 0700`,
    },
    {
      id: 'm1a-unit-edit',
      title: 'Prepare the edited 860 mount unit',
      says: `The 860 mount gains ${SUBVOLUME_REMOVAL_OPTION} (the plan's adopted decision) so the wrapper can remove a worktree subvolume with BTRFS_IOC_SNAP_DESTROY. This step only writes the edited copy of ${c.unitPath} and keeps the unit as it is, both in ${c.wiz}; nothing live changes.`,
      command: 'true',
      internal: hookUnitEdit,
      check: `test -e ${c.wiz}/mount-860.service.new || grep --fixed-strings --quiet ${SUBVOLUME_REMOVAL_OPTION} ${c.unitPath}`,
      checkText: 'the edited copy exists, or the unit already has the option',
      rollback: 'read-only',
      root: 'no',
      unprivileged: 'not needed: the unit is world-readable and the edit is made on a copy',
    },
    {
      id: 'm1b-mount-option',
      title: 'Apply the 860 mount option',
      says: `Install the edited unit (a root-owned file), reload systemd, and remount the 860 with ${SUBVOLUME_REMOVAL_OPTION}. The remount does not unmount anything: files stay open. Deletion of an empty subvolume works without the option (rmdir_subvol); the option makes the wrapper's fast path available.`,
      command: `if [ -e ${c.wiz}/mount-860.service.new ]; then sudo install --mode=0644 --owner=root --group=root ${c.wiz}/mount-860.service.new ${c.unitPath}; fi
sudo systemctl daemon-reload
sudo mount -o remount,${SUBVOLUME_REMOVAL_OPTION} ${c.drive}
findmnt --target ${c.drive} --output OPTIONS`,
      check: `grep --fixed-strings --quiet ${SUBVOLUME_REMOVAL_OPTION} ${c.unitPath} && findmnt --noheadings --target ${c.drive} --output OPTIONS | grep --fixed-strings --quiet ${SUBVOLUME_REMOVAL_OPTION} && systemctl cat mount-860.service | grep --fixed-strings --quiet ${SUBVOLUME_REMOVAL_OPTION}`,
      checkText: `the unit file, systemctl cat and the live mount options all name ${SUBVOLUME_REMOVAL_OPTION}`,
      rollback: `sudo cp ${c.wiz}/mount-860.service.before ${c.unitPath} && sudo systemctl daemon-reload (the live mount keeps the option until the next mount; a remount without it is: sudo mount -o remount,nouser_subvol_rm_allowed ${c.drive})`,
      root: 'yes',
      unprivileged: 'none: a remount and a file under /etc/systemd/system both need root',
    },
    {
      id: 'p4b-print',
      title: 'First pass: print mode',
      says: `First pass, print mode: every step the real run will perform, saved to ${c.wiz}/first-pass-print.txt. READ IT. The lines below are the first and last of it.`,
      command: `${copy} --pass=first ${args} --deadline-seconds=${String(c.transferDeadlineSeconds,)} > ${c.wiz}/first-pass-print.txt
head --lines=6 ${c.wiz}/first-pass-print.txt
echo ...
tail --lines=2 ${c.wiz}/first-pass-print.txt
echo "snapshots printed: $(grep --count '^btrfs subvolume snapshot -r' ${c.wiz}/first-pass-print.txt); sends printed: $(grep --count 'btrfs send' ${c.wiz}/first-pass-print.txt)"`,
      check: `test "$(grep --count '^btrfs subvolume snapshot -r' ${c.wiz}/first-pass-print.txt)" = "$(grep --count 'btrfs send' ${c.wiz}/first-pass-print.txt)" && grep --fixed-strings 'inventory agrees' "$OUTPUT_FILE"`,
      checkText: 'one send per snapshot and the inventory agrees',
      rollback: 'read-only',
      root: 'no',
      unprivileged: 'not needed: print mode runs as the user',
    },
    {
      id: 'p4c-first',
      title: 'First pass: run',
      says: `First pass, for real. The repository stays writable; no window, no freeze. Each transfer has the tool's own deadline (${String(c.transferDeadlineSeconds,)} s) and its own systemd scope (CPUQuota 200%, MemoryMax 2G, TasksMax 32, IOWeight 10), and the whole pass has a total deadline (${String(c.firstPassTotalSeconds,)} s, the author's bound; a rerun continues). Records are keyed by subvolume UUID and path, so a rerun after worktrees were added or removed continues each one's own records and states what changed. The inventory is taken again at the start of this stage, because worktrees come and go. The coordinator states the print was read.`,
      assertion: `${c.wiz}/first-pass-print.txt was read and is what you expect.`,
      command: `sudo btrfs subvolume list -u -q /var/home > ${c.inventory}
${rootCopy} --run --pass=first ${args} --deadline-seconds=${String(c.transferDeadlineSeconds,)} --total-deadline-seconds=${String(c.firstPassTotalSeconds,)}`,
      check: `grep --fixed-strings 'pass first finished' "$OUTPUT_FILE" && while IFS=$'\\t' read -r name path uuid; do test -e ${c.moveState}/$name.received.json || { echo "no received record for $name $path"; exit 1; }; done < ${c.moveState}/first-pass-manifest.tsv`,
      checkText: "'pass first finished' and a received record for every row of the manifest",
      rollback: "nothing on the NVMe tree changed (snapshots only). A partial receive is named by the tool with the exact 'btrfs subvolume delete' for you to run",
      timeoutSeconds: c.firstPassTotalSeconds + (c.killGraceSeconds
        * 2),
      timer: {
        file: 'first-pass-seconds',
        mode: 'add',
      },
      root: 'yes',
      unprivileged: "none for send and receive (CAP_SYS_ADMIN). The plan's method M2 (unprivileged copy plus per-worktree snapshots) needs no root but writes every extent again, so the copy would not share space between worktrees and would not fit",
    },
    {
      id: 'p4d-external-first',
      title: 'External worktrees: pass one',
      says: `External worktrees (plain directories under ${c.externalWorktrees}): pass one is a new subvolume per folder and a root rsync into it while everything runs. Folders: ${nameList}`,
      command: `${rootCopy} --run --pass=external-first ${args} --external-source=${c.externalWorktrees} --external-final=${c.externalFinal} --external-deadline-seconds=${String(c.externalDeadlineSeconds,)} --total-deadline-seconds=${String(c.firstPassTotalSeconds,)}`,
      check: `for name in ${nameList}; do test -d ${c.externalFinal}/$name || exit 1; done`,
      checkText: 'every external folder has a copy',
      rollback: `the sources are untouched; the copies are new subvolumes under ${c.externalFinal} (delete them yourself with rmdir after emptying, or btrfs subvolume delete as root)`,
      timeoutSeconds: c.firstPassTotalSeconds + (c.killGraceSeconds
        * 2),
      root: 'yes',
      unprivileged: 'a user-level rsync of the user\'s own worktrees works (the 860 top level is user-writable, and btrfs subvolume create works unprivileged on a user-owned folder); root is used to read files the user cannot (the plan found root-owned files inside some worktrees)',
    },
    {
      id: 'p4e-sizes',
      title: 'Size table',
      says: 'Size table: the on-disk size of every subvolume, taken now (outside any window) so the cut-over verification can choose the six largest and six smallest.',
      command: `cut --fields=2 ${c.moveState}/first-pass-manifest.tsv | sudo systemd-run ${c.rootScope} timeout --kill-after=${String(c.killGraceSeconds,)} ${String(c.sizeTableTimeoutSeconds,)} xargs --delimiter='\\n' du --summarize --one-file-system --block-size=1M -- > ${c.wiz}/subvolume-sizes.txt`,
      check: `test "$(wc --lines < ${c.wiz}/subvolume-sizes.txt)" = "$(wc --lines < ${c.moveState}/first-pass-manifest.tsv)"`,
      checkText: 'one size row per manifest row',
      rollback: 'read-only',
      root: 'yes',
      unprivileged: 'du of the checkout works as the user except for root-owned files; root is used so every file is counted',
    },
    {
      id: 'p4f-verify-first',
      title: 'Verify every first-pass copy',
      says: `Every first-pass copy against the read-only snapshot it was sent from, with rsync --dry-run by path, type, size, time and mode (no data is read), outside any window because both sides are immutable. The nested subvolume folders a snapshot shows empty are the only excused differences. A difference stops the run and names the copy.`,
      command: `${rootCopy} --run --pass=verify-first ${args} --deadline-seconds=${String(c.transferDeadlineSeconds,)} --total-deadline-seconds=${String(c.firstPassTotalSeconds,)}`,
      check: `grep --fixed-strings 'verify-first finished' "$OUTPUT_FILE"`,
      checkText: "'verify-first finished': every copy matches its snapshot",
      rollback: 'read-only comparison; nothing changed. A copy that differs is deleted by you (the tool names it) and sent again',
      timeoutSeconds: c.firstPassTotalSeconds + (c.killGraceSeconds
        * 2),
      root: 'yes',
      unprivileged: 'none: the comparison reads files only root can read (root-owned logs inside some worktrees) and the copy tool runs the same root scopes as the transfers',
    },
  ];
}
