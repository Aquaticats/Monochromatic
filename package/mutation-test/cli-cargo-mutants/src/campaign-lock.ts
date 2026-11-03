/**
 The right to run one campaign: a numbered claim file in the campaign's evidence directory that names the campaign process.

 The claim is the one `src/owner-claim.ts` implements after `package/git-policy/cli/src/native/single_owner_claim.rs`:
 `campaign.owner.<generation>` is published with one exclusive `link`, the newest generation is the claim, and it names its
 owner by process id, birth (start time and boot) and PID namespace. A second campaign on the same directory is refused with
 the owner named, never made to wait. A run killed by `systemctl stop`, a reboot or `SIGKILL` leaves its claim behind and the
 next run takes the next generation, because no process has that id with that birth any more; a run that ends cleanly
 rewrites its claim as released. The newest generation is kept as a tombstone, so no later run can create the same number
 twice.

 No lock stays in this file (the repository rule is lock-free first; a lock that must stay covers the smallest unit and says
 why). The unit is one campaign's evidence directory and one owner per item. The kernel lock this file used to take on the
 directory is gone; what remains is a single non-waiting probe of that old lock, for a campaign that an earlier release of
 this package started and that still runs (`src/legacy-flock.ts`). That probe takes the lock for an instant and gives it
 straight back; nothing holds or waits for it, and it can be removed with the probe once no such campaign can run.

 What the claim judges less exactly than the kernel did: whether its owner is alive. It knows only the recorded process id,
 start time and boot, which is exact for a run that was killed, stopped or lost to a reboot, but a campaign process that
 handed its work to another process and exited would read as finished.

 It is needed even though the parallel launcher runs each campaign as a uniquely named systemd unit: a run started by hand,
 or by a second launcher under another label for the same file, names the same evidence directory, and the runner removes
 an orphaned container of the campaign's name before it creates its own, which would kill a live run's container.
 */
import { stat } from 'node:fs/promises';

import { probeLegacyLock } from './legacy-flock.ts';
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
    // share the directory, whichever release they are.
    if ((await probeLegacyLock({
      path: evidence,
      retireWhenFree: false,
    })) === 'held')
      throw new CargoMutantsToolingError(`A run of this campaign started by the earlier release of this package is alive (it holds the kernel lock on ${evidence}); wait for it or stop it, then run this again.`);
  }
  catch (error) {
    // Nothing is granted: give the claim back, so a refused run does not lock out the run that is alive.
    await release();
    throw error;
  }
  return { [Symbol.asyncDispose]: release };
}
