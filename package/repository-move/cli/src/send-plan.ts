/**
 What each send may start from. `btrfs send` with clone sources (`-c`) and no parent (`-p`) searches for a parent itself
 and stops with "parent determination failed" unless the subvolume a snapshot was taken from is one of the clone
 sources, or a clone source is another snapshot of that same subvolume (btrfs-progs v7.1, `cmds/send.c`,
 `find_good_parent`). In a first pass every snapshot is of a different subvolume, so clone sources alone never work.
 A parent given with `-p` skips that search, and the kernel only needs it to be a read-only snapshot present on both
 sides (`fs/btrfs/send.c`, `btrfs_ioctl_send`).

 So each subvolume is sent as an increment on the copy of the nearest ancestor (by the parent UUID chain) that this pass
 already sent, with the other copies as clone sources, or in full with no `-p` and no `-c` when none is related.

 @module
 */

import { PARENT_UNKNOWN, } from './inventory.ts';
import type { Subvolume, } from './inventory.ts';

/**
 A subvolume this pass already sent, with the name of its snapshot.
 */
export type SentCopy = Subvolume & {
  readonly name: string;
};

/**
 What one send starts from, and the sentence that says why.
 */
export type SendPlan = {

  /**
   Name of the snapshot given to `-p`, or the empty string for a full send.
   */
  readonly parent: string;

  /**
   Names of the snapshots given to `-c`.
   */
  readonly clones: readonly string[];
  readonly reason: string;
};

/**
 Parent UUID of a subvolume that was not snapshotted from another one.
 */
const NO_PARENT = '-';

/**
 Finds the nearest ancestor of a subvolume that this pass already sent.

 @param subvolume - the subvolume about to be sent

 @param everyone - every subvolume of the run, to follow the chain through ones not sent yet

 @param sent - the copies already sent

 @returns the copy and the subvolume it stands for, or undefined when no ancestor was sent

 @example
 ```ts
 import { nearestSentAncestor, } from './send-plan.ts';

 const found = nearestSentAncestor({ subvolume, everyone, sent, },);
 ```
 */
export function nearestSentAncestor(
  {
    subvolume,
    everyone,
    sent,
  }: {
    readonly subvolume: Subvolume;
    readonly everyone: readonly Subvolume[];
    readonly sent: readonly SentCopy[];
  },
): SentCopy | undefined {
  // The chain is at most as long as the list; the bound also stops a malformed inventory whose parents form a loop.
  for (
    let hops = 0, uuid = subvolume.parentUuid;
    (hops <= everyone.length) && (uuid !== NO_PARENT) && (uuid !== PARENT_UNKNOWN);
    hops += 1
  ) {
    /**
     The copy of the ancestor with this UUID, when this pass sent it.
     */
    const copy = sent.find(function isAncestor(candidate,) {
      return candidate.uuid === uuid;
    },);
    if (copy !== undefined) {
      return copy;
    }
    /**
     The ancestor itself, to read its own parent.
     */
    const ancestor = everyone.find(function hasUuid(candidate,) {
      return candidate.uuid === uuid;
    },);
    if (ancestor === undefined) {
      return undefined;
    }
    uuid = ancestor.parentUuid;
  }
  return undefined;
}

/**
 Chooses what one send starts from.

 @param subvolume - the subvolume about to be sent

 @param everyone - every subvolume of the run

 @param sent - the copies this pass already sent, in order

 @param firstPassCopy - the name of this subvolume's first-pass snapshot in a final pass, when it has one

 @param cloneSources - `all` gives every other sent copy as a clone source, `main` only the first

 @returns the plan

 @example
 ```ts
 import { planSend, } from './send-plan.ts';

 const plan = planSend({ subvolume, everyone, sent, firstPassCopy: '', cloneSources: 'all', },);
 ```
 */
export function planSend(
  {
    subvolume,
    everyone,
    sent,
    firstPassCopy,
    cloneSources,
  }: {
    readonly subvolume: Subvolume;
    readonly everyone: readonly Subvolume[];
    readonly sent: readonly SentCopy[];
    readonly firstPassCopy: string;
    readonly cloneSources: 'all' | 'main';
  },
): SendPlan {
  /**
   The copies that may serve as clone sources.
   */
  const available = cloneSources === 'main' ? sent.slice(
    0,
    1,
  ) : sent;
  if (firstPassCopy !== '') {
    return {
      parent: firstPassCopy,
      clones: available.map(function nameOf(copy,) {
        return copy.name;
      },),
      reason: `incremental on ${firstPassCopy} (its first-pass copy)`,
    };
  }
  /**
   The nearest ancestor this pass already sent.
   */
  const ancestor = nearestSentAncestor({
    subvolume,
    everyone,
    sent,
  },);
  if (ancestor !== undefined) {
    return {
      parent: ancestor.name,
      clones: available.filter(function isOther(copy,) {
        return copy.name !== ancestor.name;
      },)
        .map(function nameOf(copy,) {
          return copy.name;
        },),
      reason: `incremental on ${ancestor.name} (it was snapshotted from ${ancestor.path})`,
    };
  }
  return {
    parent: '',
    clones: [],
    reason: subvolume.parentUuid === PARENT_UNKNOWN
      ? 'full send (the inventory has no parent column, so no earlier copy can be named its parent)'
      : 'full send (no copy sent so far is an ancestor of it, and clone sources alone make btrfs send stop)',
  };
}
