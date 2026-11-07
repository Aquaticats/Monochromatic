/**
 The stages of `thaw` and `rollback`: undo the freeze before the swap, and the fast rollback after it. Nothing is
 deleted; folders and files are renamed aside.

 @module
 */

import { realArguments, } from './guided-prepare-stages.data.ts';
import type {
  GuidedConfig,
  Stage,
} from './guided-types.ts';

/**
 The stages that undo the freeze: thaw the subvolumes, unmount the external binds, restart the stopped units.

 @param c - the configuration
 @returns the stages
 */
export function thawStages(c: GuidedConfig,): readonly Stage[] {
  const args = realArguments({ c, inventory: `${c.wiz}/inventory-final.txt`, },);
  return [
    {
      id: 't1-thaw-subvolumes',
      title: 'Thaw the subvolumes',
      says: `Thaw: ro=false on exactly the subvolumes the freeze recorded (${c.moveState}/freeze-manifest.tsv). It does not read the inventory, so worktrees that changed since the freeze cannot block it. Idempotent.`,
      command: `${c.nodePath} ${c.copyTool} --run --pass=thaw ${args}`,
      check: `grep --fixed-strings 'thaw finished' "$OUTPUT_FILE"`,
      checkText: 'the tool thawed every recorded subvolume and read ro=false back',
      rollback: 'freeze again by running the cut-over',
      noMark: true,
      root: 'no',
      unprivileged: 'this is the unprivileged route: the owner of a subvolume sets ro on it without root; a subvolume the user does not own makes the tool stop and name it',
    },
    {
      id: 't2-umount-binds',
      title: 'Unmount external binds',
      says: 'Unmount the read-only binds over the external worktrees (only those still mounted).',
      command: `touch ${c.wiz}/ext-binds.txt
while read -r p; do if mountpoint --quiet "$p"; then sudo umount "$p"; fi; done < ${c.wiz}/ext-binds.txt`,
      check: `while read -r p; do ! mountpoint --quiet "$p" || { echo "$p still mounted"; exit 1; }; done < ${c.wiz}/ext-binds.txt`,
      checkText: 'no external folder is still a mount',
      rollback: 'freeze again by running the cut-over',
      noMark: true,
      root: 'yes',
      unprivileged: 'none: umount of a bind mount made by root needs root',
    },
    {
      id: 't3-restart-units',
      title: 'Restart the stopped unit',
      says: 'Start again what the cut-over stopped: the commit sockets (restarting them is harmless; whether the wrapper would start them itself was not verified) and claude-cpu-watch.service.',
      command: `touch ${c.wiz}/stopped-units.txt
{ grep '\\.socket$' ${c.wiz}/stopped-units.txt || true; } | xargs --no-run-if-empty systemctl --user start || echo 'WARNING: a commit socket did not restart; list-units cli-git@* shows which'
if grep --fixed-strings --line-regexp --quiet claude-cpu-watch.service ${c.wiz}/stopped-units.txt; then systemctl --user start claude-cpu-watch.service; else echo 'claude-cpu-watch.service was not stopped by the cut-over'; fi`,
      check: 'true',
      checkText: 'the command exited 0',
      rollback: 'systemctl --user stop claude-cpu-watch.service',
      noMark: true,
      root: 'no',
      unprivileged: 'not needed: user units',
    },
  ];
}

/**
 The stages of the fast rollback after the swap, before the thaw stages.

 @param c - the configuration
 @param stamp - the swap's timestamp
 @returns the stages
 */
export function rollbackStages(
  { c, stamp, }: {
    readonly c: GuidedConfig;
    readonly stamp: string;
  },
): readonly Stage[] {
  const oldRepo = `${c.repo}.nvme-before-860-${stamp}`;
  const oldExternal = `${c.externalWorktrees}.nvme-before-860-${stamp}`;
  const identity = `${c.homeDir}/.config/monochromatic-dev/repository-link.json`;
  return [
    {
      id: 'r1-old-still-frozen',
      title: 'Old tree still frozen',
      says: 'The old trees must still be frozen (nothing wrote to them since the freeze).',
      command: `while IFS=$'\\t' read -r uuid p; do echo "$p: $(btrfs property get -ts "${oldRepo}\${p#${c.repo}}" ro)"; done < ${c.moveState}/freeze-manifest.tsv`,
      check: `! grep --fixed-strings 'ro=false' "$OUTPUT_FILE"`,
      checkText: 'every old subvolume still reads ro=true',
      rollback: 'read-only',
      noMark: true,
      root: 'no',
      unprivileged: 'not needed: property get runs as the user',
    },
    {
      id: 'r2-restore-names',
      title: 'Put the old folders back',
      says: 'Put the links aside (renamed, not deleted) and the old folders back. Refuses if either old path is occupied by something else.',
      command: `test -L ${c.repo} && test -L ${c.externalWorktrees}
test ! -e ${c.repo}.link-rolled-back-${stamp} && test ! -e ${c.externalWorktrees}.link-rolled-back-${stamp}
mv --no-clobber --no-target-directory ${c.repo} ${c.repo}.link-rolled-back-${stamp}
mv --no-clobber --no-target-directory ${oldRepo} ${c.repo}
mv --no-clobber --no-target-directory ${c.externalWorktrees} ${c.externalWorktrees}.link-rolled-back-${stamp}
mv --no-clobber --no-target-directory ${oldExternal} ${c.externalWorktrees}`,
      check: `test -d ${c.repo} && test ! -L ${c.repo} && test -d ${c.externalWorktrees} && test ! -L ${c.externalWorktrees}`,
      checkText: 'both old paths are real folders again',
      rollback: `the links are beside them with the suffix .link-rolled-back-${stamp}`,
      noMark: true,
      root: 'no',
      unprivileged: 'not needed: renames in the user-owned home folder',
    },
    {
      id: 'r3-identity-back',
      title: 'Record the repository link identity, phase before',
      says: 'The identity file said phase after; it is renamed aside (not deleted) and phase before is recorded again, because the repository is a real folder on the NVMe once more.',
      command: `if [ -e ${identity} ]; then mv --no-clobber ${identity} ${identity}.rolled-back-${stamp}; fi
${c.nodePath} ${c.linkBinFolder}/record.mjs --phase=before --link-path=${c.repo} --mount-point=${c.drive}`,
      check: `${c.nodePath} ${c.linkBinFolder}/check.mjs | grep --fixed-strings 'ready (before the move)'`,
      checkText: 'the installed check says ready (before the move)',
      rollback: 'the old identity file is beside the new one with the suffix .rolled-back',
      noMark: true,
      root: 'no',
      unprivileged: 'not needed: written as the user',
    },
  ];
}
