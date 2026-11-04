/**
 The right to run one campaign: a numbered claim file in the campaign's evidence directory that names the campaign process.

 The claim is the one `src/owner-claim.ts` implements after `package/git-policy/cli/src/native/lock/single_owner_claim.rs`:
 `campaign.owner.<generation>` is published with one exclusive `link`, the newest generation is the claim, and it names its
 owner by process id, birth (start time and boot) and PID namespace. A second campaign on the same directory is refused with
 the owner named, never made to wait. A run killed by `systemctl stop`, a reboot or `SIGKILL` leaves its claim behind and the
 next run takes the next generation, because no process has that id with that birth any more; a run that ends cleanly
 rewrites its claim as released. The newest generation is kept as a tombstone, so no later run can create the same number
 twice.

 The one lock that stays in this file is the kernel lock on the evidence directory that the earlier release of this package took
 (the repository rule is lock-free first; a lock that must stay covers the smallest unit and says why). The unit is one
 campaign's evidence directory. It stays because a campaign of the earlier release takes that lock without waiting and never
 looks at a claim file, so a campaign of this release that holds only its claim would let an old one start on the same
 directory. The lock is taken once without waiting, after the claim, held for as long as the claim, and given back with it;
 nothing waits for it, and it can be removed (`src/legacy-flock.ts`) once no such campaign can run. A campaign of the earlier
 release that already holds the lock refuses this run, as before.

 What the claim judges less exactly than the kernel did: whether its owner is alive. It knows only the recorded process id,
 start time and boot, which is exact for a run that was killed, stopped or lost to a reboot, but a campaign process that
 handed its work to another process and exited would read as finished.

 It is needed even though the parallel launcher runs each campaign as a uniquely named systemd unit: a run started by hand,
 or by a second launcher under another label for the same file, names the same evidence directory, and the runner removes
 an orphaned container of the campaign's name before it creates its own, which would kill a live run's container.
 */
import { stat } from 'node:fs/promises';

import { holdLegacyLock } from './legacy-flock.ts';
import { CargoMutantsToolingError } from './error.ts';
import { takeClaim } from './owner-claim.ts';

/**
 The claim series' name in the evidence directory: files named `campaign.owner.<generation>`.
 */
export const campaignClaimPrefix = 'campaign.owner';

/**
 Tell whether a path is an existing directory.

 @param path - the path

 @returns false for a path that does not exist or is not a directory

 @example
 ```ts
 await isDirectory('/e/native-mutation-abc');
 ```
 */
async function isDirectory(path: string): Promise<boolean> {
  try {
    return (await stat(path)).isDirectory();
  }
  catch (error) {
    console.error(`Cannot examine ${path}: ${String(error)}`);
    return false;
  }
}

/**
 Take the right to run one campaign, so that no second run writes the same evidence directory.

 @param evidence - the campaign's evidence directory, which must exist

 @returns the held claim; declare it with `await using` so it is released when the campaign ends, and keep it for as long as
 the campaign runs

 @throws CargoMutantsToolingError when another run of the same campaign holds the claim (a campaign of the earlier release
 holding its kernel lock counts), the directory does not exist, or the claim cannot be judged or written

 @example
 ```ts
 await using lock = await acquireCampaignLock('/e/native-mutation-0123456789ab');
 ```
 */
export async function acquireCampaignLock(evidence: string): Promise<AsyncDisposable> {
  if (!(await isDirectory(evidence)))
    throw new CargoMutantsToolingError(`${evidence} is not a directory, so there is nothing to claim.`);
  /**
   What taking the claim came to.
   */
  const outcome = await takeClaim({
    directory: evidence,
    prefix: campaignClaimPrefix,
  });
  if (outcome.kind === 'busy') {
    /**
     The process that holds the claim and the file that says so.
     */
    const { owner } = outcome;
    /**
     The owner's process id as text.
     */
    const ownerPid = String(owner.pid);
    throw new CargoMutantsToolingError(`Another run of this campaign is alive (process ${ownerPid}, claim file ${owner.path}); wait for it or stop it, then run this again.`);
  }
  /**
   The claim this run holds.
   */
  const { claim } = outcome;
  /**
   Give the claim back, once.
   */
  async function release(): Promise<void> {
    try {
      await claim.release();
    }
    catch (error) {
      // The claim then names this process until the process ends, when it stops counting as alive.
      console.error(`The claim ${claim.path} could not be released: ${String(error)}`);
    }
  }
  try {
    // A campaign of the earlier release holds a kernel lock on the directory instead of a claim, and two runs must never
    // share the directory, whichever release they are; this run takes the same lock, so a campaign of the earlier release
    // that starts while this one runs is refused in its turn.
    /**
     What taking that lock came to.
     */
    const hold = await holdLegacyLock({
      path: evidence,
      create: false,
    });
    if (hold.kind === 'busy')
      throw new CargoMutantsToolingError(`A run of this campaign started by the earlier release of this package is alive (it holds the kernel lock on ${evidence}); wait for it or stop it, then run this again.`);
    if (hold.kind === 'kept')
      return {
        async [Symbol.asyncDispose](): Promise<void> {
          // The kernel lock goes first, so a run that sees the claim given back also finds the lock free.
          await hold.lock[Symbol.asyncDispose]();
          await release();
        },
      };
  }
  catch (error) {
    // Nothing is granted: give the claim back, so a refused run does not lock out the run that is alive.
    await release();
    throw error;
  }
  return { [Symbol.asyncDispose]: release };
}
