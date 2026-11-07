/**
 The kernel lock of the earlier release on a slot file, taken with the claim already held: when the lock cannot be taken, the
 claim goes back before the error reaches the caller.
 */
import {
  holdLegacyLock,
  type LegacyHold,
} from './legacy-flock.ts';
import type { OwnerClaim } from './owner-claim.ts';

/**
 Take the kernel lock a gate of the earlier release would poll for, and give the claim back when the lock cannot be taken.

 @param path - the slot's lock file

 @param claim - the claim just taken on the slot

 @returns what {@link holdLegacyLock} found

 @throws CargoMutantsToolingError when a directory or a pipe stands in place of the slot file, after the claim went back, so
 the refused gate does not hold the slot for as long as it lives

 @example
 ```ts
 await holdLegacyLockOrGiveBack({ path: '/l/slot-1.lock', claim });
 ```
 */
export async function holdLegacyLockOrGiveBack({
  path,
  claim,
}: {
  readonly path: string;
  readonly claim: OwnerClaim;
}): Promise<LegacyHold> {
  try {
    return await holdLegacyLock({
      path,
      create: true,
    });
  }
  catch (error) {
    await claim.release();
    throw error;
  }
}
