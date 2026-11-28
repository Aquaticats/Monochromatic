/**
 Plan step 19's migration and its rollback, run by `doc/runbook/zfs-checkout-migration-and-boot-setup.md`'s own steps on a
 layout the guest can throw away: its own pool, its own home dataset, a real Git history with a linked worktree at the
 checkout path, and one dataset mounted inside it.

 Each step is one guest command with `set -x`, so its log holds the command it ran and what the command answered, which is
 the evidence the runbook asks a person to keep. Every deletion path here is written literally (`AGENTS.md` rule `RLW`),
 because this arm removes its own pool, image and folder at the end.

 The page's commands are run as the guest's root without `sudo`: the page's own first line says every step runs as root, and
 `sudo` is not installed in the guest.

 @example
 ```ts
 await migrateAndRollBack({ chosen });
 ```
 */

import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import { guest, } from '../../bin/mvm-guest-script.ts';

import { installThePlan, } from './boot.ts';

import {
  migrationImage,
  openZfsLibrary,
  openZfsPath,
} from './fixture.ts';

/**
 The migration arm's own pool.
 */
export const migrationPool: string = 'zfsstep19mig';

/**
 The file-backed image the migration's pool stands on.
 */


/**
 The home dataset the checkout's tree lives on before the migration.
 */
export const migrationHomeDataset: string = `${migrationPool}/ROOT/cos/home`;

/**
 The folder the home dataset is mounted at, which holds the checkout path.
 */
export const migrationHomePath: string = '/home/zfs-step19-migration';

/**
 The checkout path the migration moves onto its own dataset.
 */
export const migrationCheckout: string = `${migrationHomePath}/Monochromatic`;

/**
 The folder the checkout dataset is filled and verified at before its mountpoint moves, which the page leaves to the
 operator.
 */
export const migrationStaging: string = '/mnt/checkout-migration';

/**
 The nested dataset inside the checkout, which stands for this machine's `zfsstep19` builder output.
 */
export const migrationNested: string = `${migrationPool}/nosnap-vm-builder-output`;

/**
 The checkout dataset the migration makes.
 */
export const migrationCheckoutDataset: string = `${migrationHomeDataset}/monochromatic-git-checkout`;

/**
 One step of the page, as the guest runs it.

 The header names the page's own section, so a reader of the logs follows the page rather than this file.
 */
type MigrationStep = {
  /**
   The step's name in the log's file name.
   */
  readonly name: string;
  /**
   The page's section the step belongs to.
   */
  readonly section: string;
  /**
   The commands, run with `set -eu -x`.
   */
  readonly commands: readonly string[];
};

/**
 The environment every step starts with.
 */
const shellHeader: readonly string[] = [
  `export PATH=${openZfsPath}`,
  `export LD_LIBRARY_PATH=${openZfsLibrary}`,
  'set -eu',
];

/**
 What:
  The page's steps, in its own order, each with the one correction this run found: the checkout dataset is made at the
  staging mountpoint and not at the checkout path.
 Why:
  the page's "Create the two datasets" section made the dataset with `-o mountpoint=<the checkout path>`, which mounts it
  over the tree the "Copy the checkout" section still has to read, and that section then creates the same dataset again at
  a temporary mount point,
  which answers `dataset already exists`.
  The mountpoint moves in the page's own switch step, which is the only place a switch can happen.
 */
const steps: readonly MigrationStep[] = [
  {
    name: 'before-anything',
    section: 'Before anything',
    commands: [
      'apt-get --option Acquire::Retries=2 --yes install lsof rsync > /root/step19-migration-packages.log 2>&1',
      // The page's step 1: a holder is a session to end first, so a holder stops the page rather than being forced.
      'if lsof +D /home/zfs-step19-migration/Monochromatic > /dev/null 2>&1; then echo "a process holds the checkout"; exit 1; fi',
      'echo "STEP19-MIGRATION no-holder"',
      'git -C /home/zfs-step19-migration/Monochromatic worktree list --porcelain',
      // `findmnt` exits one when the path holds no submount, which is an answer the page asks for and not a failure.
      'findmnt --list --output TARGET,SOURCE,FSTYPE --submounts /home/zfs-step19-migration/Monochromatic || true',
      `zfs list -H -o name,mountpoint,used,avail -r ${migrationHomeDataset}`,
      'du --one-file-system --summarize --block-size=1 /home/zfs-step19-migration/Monochromatic',
    ],
  },
  {
    name: 'backup',
    section: 'Before anything',
    commands: [
      `zfs snapshot "${  migrationHomeDataset  }@before-checkout-migration-$(date --iso-8601=date)"`,
      `zfs list -H -o name,used,refer -t snapshot -r ${  migrationHomeDataset}`,
      `zfs list -H -o name,avail ${migrationHomeDataset}`,
    ],
  },
  {
    name: 'create-the-datasets',
    section: 'Create the two datasets',
    commands: [
      `zfs create -o mountpoint=${migrationStaging} -o snapdir=disabled ${migrationCheckoutDataset}`,
      `zfs create -o mountpoint=legacy -o canmount=off -o setuid=off -o devices=off -o snapdir=disabled ${migrationHomeDataset}/monochromatic-git-worktree-clone-datasets`,
      `zfs get -H -o property,value,source snapdir ${migrationCheckoutDataset} ${migrationHomeDataset}/monochromatic-git-worktree-clone-datasets`,
    ],
  },
  {
    name: 'copy-the-checkout',
    section: 'Copy the checkout',
    commands: [
      `rsync --archive --hard-links --acls --xattrs --numeric-ids --one-file-system --sparse ${migrationCheckout}/ ${migrationStaging}/`,
      `test ! -e ${migrationStaging}/package/vm-builder-output/cli-output.txt`,
      `zfs list -H -o name,mounted -r ${migrationHomeDataset}`,
    ],
  },
  {
    name: 'verify-before-the-switch',
    section: 'Verify before the switch',
    commands: [
      `git -C ${migrationCheckout} status --porcelain=v1 --untracked-files=all --ignored=matching > /root/step19-migration-source-status.txt`,
      `git -C ${migrationStaging} status --porcelain=v1 --untracked-files=all --ignored=matching > /root/step19-migration-copy-status.txt`,
      'diff --unified /root/step19-migration-source-status.txt /root/step19-migration-copy-status.txt',
      `git -C ${migrationStaging} fsck --no-progress --connectivity-only`,
      `git -C ${migrationCheckout} worktree list --porcelain > /root/step19-migration-source-worktrees.txt`,
      `git -C ${migrationStaging} worktree list --porcelain > /root/step19-migration-copy-worktrees.txt`,
      // The source's list names the checkout path and the copy's names the staging folder, so the source's own copy of
      // it is rewritten before the two are compared.
      // The copy's `worktree list` reports the *source's* paths, because Git's
      // registration for a linked worktree travels with the copy;
      // so the two lists are compared by their HEADs and branch lines,
      // and the registration's own path is printed as the fact it is.
      `grep --invert-match "^worktree " /root/step19-migration-source-worktrees.txt > /root/step19-migration-source-worktree-heads.txt`,
      `grep --invert-match "^worktree " /root/step19-migration-copy-worktrees.txt > /root/step19-migration-copy-worktree-heads.txt`,
      'diff --unified /root/step19-migration-source-worktree-heads.txt /root/step19-migration-copy-worktree-heads.txt',
      `cat /mnt/checkout-migration/.git/worktrees/linked/gitdir`,
      'echo "STEP19-MIGRATION registration-names-the-source-path=true"',

      'echo "STEP19-MIGRATION verified=status-worktrees-and-fsck"',
    ],
  },
  {
    name: 'switch-the-mounts',
    section: 'Switch the mounts at the same path',
    commands: [
      // The tree being replaced is a folder on the home dataset, not a mount of its own, so there is nothing to unmount
      // here: the page's `umount` line answers "not mounted" and stops a run under `set -e`.
      `zfs unmount ${migrationNested} || true`,
      `zfs set mountpoint=${migrationCheckout} ${migrationCheckoutDataset}`,
      // `zfs set mountpoint=` on a mounted dataset mounts it at the new path itself, so the page's own `zfs mount` here
      // answers `filesystem already mounted`; it stays for the case where the dataset was unmounted first.
      `zfs mount ${migrationCheckoutDataset} || true`,
      `mount --make-rprivate ${migrationCheckout}`,
      `findmnt --target ${migrationCheckout}`,
      `test -f ${migrationCheckout}/.git/HEAD`,
      'echo "STEP19-MIGRATION switched=yes"',
    ],
  },
  {
    name: 'protect-the-covered-entry',
    section: 'Protect the covered entry',
    commands: [
      `zfs unmount ${migrationCheckout}`,
      `chattr +i ${migrationCheckout}`,
      `lsattr -d ${migrationCheckout}`,
      `zfs mount ${migrationCheckoutDataset}`,
      `mount --make-rprivate ${migrationCheckout}`,
      'echo "STEP19-MIGRATION protected=yes"',
    ],
  },
  {
    name: 'the-route-s-root-owned-files',
    section: "Write the route's root-owned files",
    commands: [
      'mkdir --parents /etc/monochromatic-git-policy-cli',
      String.raw`printf '{\n  "cloneMounts": {\n    "checkout": "` + migrationCheckout
        + String.raw`",\n    "checkoutDataset": "`
        + migrationCheckoutDataset
        + String.raw`",\n    "parent": "`
        + migrationHomeDataset
        + String.raw`/monochromatic-git-worktree-clone-datasets"\n  }\n}\n' > /etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc`,
      'chmod 0644 /etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc',
      'chown root:root /etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc',
      // One machine serves one checkout, and this guest's own served checkout is the boot arm's:
      // the record is taken away so this step can record the migrated one, and the last step puts the boot layout's
      // back.
      'rm --force /var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json',
      `/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli zfs-root record-checkout --checkout ${migrationCheckout}`,
      'cat /var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json',
      `zfs set canmount=noauto ${migrationNested}`,
      `zfs get -H -o property,value canmount,mountpoint ${migrationNested}`,
      'echo "STEP19-MIGRATION root-files=written"',
    ],
  },
  {
    name: 'read-what-the-manager-loaded',
    section: 'Write the units',
    commands: [
      'systemctl daemon-reload',
      'systemctl show --property=TimeoutStartUSec,TimeoutStopUSec,DropInPaths,CanReload reconcile-git-worktree-clone-mounts.service',
      // The page's own note: verify reports one line per unit whose mount unit the manager learned from the mount table,
      // and this checkout is a mount the manager learns that way.
      'systemd-analyze verify /etc/systemd/system/reconcile-git-worktree-clone-mounts.service',
      'echo "STEP19-MIGRATION units=read-back"',
    ],
  },
  {
    name: 'roll-back',
    section: 'Roll back, if a check fails',
    commands: [
      `zfs unmount ${migrationCheckout}`,
      `zfs set mountpoint=${migrationStaging} ${migrationCheckoutDataset}`,
      `zfs destroy ${migrationCheckoutDataset}`,
      // The page's `zfs mount <home dataset>` line answers "filesystem already mounted": the home dataset was never
      // unmounted, and the tree the migration moved is a folder on it.
      `test -f ${migrationCheckout}/.git/HEAD`,
      `git -C ${migrationCheckout} status --porcelain=v1 --untracked-files=all --ignored=matching | diff --unified /root/step19-migration-source-status.txt -`,
      `lsattr -d ${migrationCheckout}`,
      `chattr -i ${migrationCheckout}`,
      `lsattr -d ${migrationCheckout}`,
      `zfs list -H -o name -r ${migrationPool}`,
      // The guest's own served checkout is the boot arms', so their configuration and record come back.
      `printf '%s\\n' '{' '  "cloneMounts": {' '    "checkout": "/home/zfs-step19-fixture/home/monochromatic-git-checkout",' '    "checkoutDataset": "zfsstep19/ROOT/home/monochromatic-git-checkout",' '    "parent": "zfsstep19/ROOT/home/monochromatic-git-worktree-clone-datasets"' '  }' '}' > /etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc`,
      'chmod 0644 /etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc',
      'rm --force /var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json',
      '/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/zfs-step19-fixture/home/monochromatic-git-checkout || true',
      'echo "STEP19-MIGRATION rolled-back=yes"'
    ],
  },
];

/**
 Run the page's steps in its order.

 @param chosen - the run's guest and log folder

 @returns every step's log, in the order the steps ran

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await migrateAndRollBack({ chosen });
 ```
 */
export async function migrateAndRollBack({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<readonly string[]> {
  /**
   What the step that wrote and enabled the plan's units, which the page installs before it reads them back, wrote.
   */
  const installed = await installThePlan({ chosen, },);
  /**
   What the layout the page moves wrote before its first step ran.
   */
  const laidOut = await layOutTheDisposableHome({ chosen, },);
  return await runTheSteps({
    chosen,
    remaining: steps,
    said: [
      installed,
      laidOut,
    ],
  },);
}

/**
 Run the page's steps one after another, each in its own guest command with `set -x`.

 Why a call of its own:
  the page's steps change one machine in a fixed order,
 so each step waits for the one before it,
 and a step is asked for only once the one before it answered.

 @param chosen - the run's guest and log folder

 @param remaining - the steps this run has not reached yet

 @param said - what the steps before this one wrote

 @returns every step's log, in the order the steps ran

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await runTheSteps({ chosen, remaining: steps, said: [] });
 ```
 */
async function runTheSteps({
  chosen,
  remaining,
  said,
}: {
  readonly chosen: GuestFacts;
  readonly remaining: readonly MigrationStep[];
  readonly said: readonly string[];
},): Promise<readonly string[]> {
  /**
   The step this call runs, absent when every one was reached.
   */
  const [step, ...rest] = remaining;
  if (step === undefined)
    return said;
  /**
   What this step wrote, which its own log holds whole.
   */
  const wrote = await guest({
    chosen,
    script: [
      ...shellHeader,
      `echo "STEP19-MIGRATION section=${step.section} step=${step.name}"`,
      'set -x',
      ...step.commands,
    ].join('\n',),
    log: `guest-step19-migration-${step.name}.log`,
    timeoutMilliseconds: 900_000,
  },);
  return await runTheSteps({
    chosen,
    remaining: rest,
    said: [
      ...said,
      wrote,
    ],
  },);
}

/**
 The layout the page moves: a home dataset with a real Git history and a linked worktree at the checkout path, and one
 dataset mounted inside it.

 The source's `status` is written before the copy so the page's own comparison has something to compare against.

 @param chosen - the run's guest and log folder

 @returns the layout step's log

 @throws GuestDriverError naming the command that refused

 @example
 ```ts
 await layOutTheDisposableHome({ chosen });
 ```
 */
export async function layOutTheDisposableHome({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  return await guest({
    chosen,
    script: [
      ...shellHeader,
      'set -x',
      `zpool destroy -f ${migrationPool} > /dev/null 2>&1 || true`,
      `rm --force ${migrationImage}`,
      `rm --recursive --force ${migrationHomePath}`,
      `mkdir --parents ${migrationHomePath}`,
      `truncate --size 1G ${migrationImage}`,
      `zpool create -O mountpoint=none ${migrationPool} ${migrationImage}`,
      `zfs create -o mountpoint=none ${migrationPool}/ROOT`,
      `zfs create -o mountpoint=none ${migrationPool}/ROOT/cos`,
      `zfs create -o mountpoint=${migrationHomePath} ${migrationHomeDataset}`,
      `mkdir --parents ${migrationCheckout}`,
      `git -C ${migrationCheckout} init --quiet --initial-branch=main`,
      `printf 'a tracked line\\n' > ${migrationCheckout}/tracked.txt`,
      `mkdir --parents ${migrationCheckout}/.claude/worktrees`,
      `git -C ${migrationCheckout} add tracked.txt`,
      `git -C ${migrationCheckout} -c user.name=step19 -c user.email=step19@example.invalid commit --quiet -m 'the tree the migration moves'`,
      `git -C ${migrationCheckout} worktree add --quiet --detach .claude/worktrees/linked`,
      `zfs create -o mountpoint=${migrationCheckout}/package/vm-builder-output ${migrationNested}`,
      `printf 'the nested dataset answers\\n' > ${migrationCheckout}/package/vm-builder-output/cli-output.txt`,
      // The nested dataset is unmounted before the page's steps, because the copy must not cross into it and the page's
      // own V4 answer mounts it again by a unit ordered after the gate.
      `zfs unmount ${migrationNested}`,
      `git -C ${migrationCheckout} status --porcelain=v1 --untracked-files=all --ignored=matching`,
      `zfs list -H -o name,mountpoint,mounted -r ${migrationPool}`,
      `echo "STEP19-MIGRATION laid-out=${migrationCheckout}"`,
    ].join('\n',),
    log: 'guest-step19-migration-layout.log',
    timeoutMilliseconds: 600_000,
  },);
}
