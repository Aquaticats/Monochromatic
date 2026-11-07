/**
 What each send starts from (coordinator's pick of 2026-11-07, owner's veto open; record in
 `doc/troubleshooting/btrfs-send-clone-sources-need-a-parent.md`).

 `btrfs send` with clone sources (`-c`) and no parent (`-p`) searches for a parent itself and stops unless the subvolume
 a snapshot was taken from is a clone source, or a clone source is another snapshot of it (btrfs-progs v7.1,
 `cmds/send.c`, `find_good_parent`). In a first pass no snapshot has such a relative, so clone sources alone never work.
 A parent given with `-p` skips that search, but the kernel pairs inodes of the sent snapshot and the parent by number
 and generation (`changed_inode` in `fs/btrfs/send.c`), so a parent that is not on the sent subvolume's own history can
 pair two different inodes made in one transaction (measured on the fixture: inode 313 was `.git/worktrees` in the
 snapshot of the main checkout and `big.copy` in the snapshot of a worktree, and the receive failed with a link to a
 directory).

 Every first-pass send is therefore `-p <empty parent>` with every earlier copy as `-c`: an empty parent has no inode to
 pair, and the kernel finds shared extents by their physical backreferences, so extent sharing with every earlier copy
 is kept. The one safe incremental is the final pass on the first-pass snapshot of the same subvolume.

 @module
 */

/**
 Name of the empty read-only snapshot every first-pass send is based on, in the snapshot folder and in the received
 folder.
 */
export const EMPTY_PARENT_NAME = 'empty-parent';

/**
 Name of the empty subvolume the empty parent is a snapshot of.
 */
export const EMPTY_PARENT_BASE_NAME = 'empty-parent-base';

/**
 What one send starts from, and the sentence that says why.
 */
export type SendPlan = {

  /**
   Name of the snapshot given to `-p`.
   */
  readonly parent: string;

  /**
   Names of the snapshots given to `-c`.
   */
  readonly clones: readonly string[];
  readonly reason: string;
};

/**
 Chooses what one send starts from.

 @param sent - the names of the copies this pass already sent, in order

 @param firstPassCopy - the name of this subvolume's first-pass snapshot in a final pass, the empty text when it has none

 @param cloneSources - `all` gives every sent copy as a clone source, `main` only the first

 @returns the plan

 @example
 ```ts
 import { planSend, } from './send-plan.ts';

 const plan = planSend({ sent: ['s000'], firstPassCopy: '', cloneSources: 'all', },);
 ```
 */
export function planSend(
  {
    sent,
    firstPassCopy,
    cloneSources,
  }: {
    readonly sent: readonly string[];
    readonly firstPassCopy: string;
    readonly cloneSources: 'all' | 'main';
  },
): SendPlan {
  /**
   The copies that may serve as clone sources.
   */
  const clones = cloneSources === 'main' ? sent.slice(
    0,
    1,
  ) : sent;
  if (firstPassCopy !== '') {
    return {
      parent: firstPassCopy,
      clones,
      reason: `incremental on ${firstPassCopy} (its first-pass copy, a snapshot of the same subvolume)`,
    };
  }
  return {
    parent: EMPTY_PARENT_NAME,
    clones,
    reason: `full send on the empty parent, ${String(clones.length,)} earlier copies as clone sources (no copy of another `
      + 'subvolume is a parent, because the kernel pairs inodes by number and generation)',
  };
}
