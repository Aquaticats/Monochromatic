//region Roster quorum size

/**
 Computes exact-half quorum, rounded up for odd rosters and never below one.

 @param rosterSize - requested independent seat count

 @returns Seats needed to settle without requiring unreliable whole roster

 @example
 ```ts
 const quorum = rosterQuorumSize({ rosterSize: 8, });
 ```
 */
export function rosterQuorumSize(
  { rosterSize, }: { readonly rosterSize: number; },
): number {
  return Math.max(
    1,
    Math.ceil(rosterSize / 2,),
  );
}

//endregion Roster quorum size
