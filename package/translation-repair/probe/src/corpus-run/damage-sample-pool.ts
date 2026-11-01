import {
  collectTwoLaneShippedRegions,
  type ShippedRegionCensus,
} from './damage-region-v2.ts';
import {
  artifactsDirOf,
  keepEligible,
  type PoolPolicy,
  resolvePool,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { listSettledNames, } from './sample-artifact-names.ts';

//region Damage sample pool
// Collects every shipped region across the settled artifacts of a runs
// directory.

/**
 Collects every shipped region across the settled artifacts.

 THE LISTING STAYS HERE and the reading moves out, because eligibility is a
 question about the POOL while shipping is a question about one artifact's
 ledger, and mixing them is what let the old reader describe a repair-lane-only
 population as the shipped regions.

 @param runsDir - runs directory holding the artifacts

 @param policy - generation policy the entry read from its environment, which
 decides which generations the pool admits (`readPoolPolicy`)

 @param repository - checkout a required commit is resolved in and every
 recorded commit is placed against, which the entry names: the package's own
 in a run, a throwaway one in a case

 @returns Regions the damage question can be asked about, and how many rows
 filled a passage that had no incumbent wording

 @throws {@link StatedRefusalError} When the runs directory holds no artifacts
 directory

 @example
 ```ts
 const { regions, } = await collectShippedRegions({ runsDir, policy, repository, },);
 ```
 */
export async function collectShippedRegions(
  {
    runsDir,
    policy,
    repository,
  }: {
    readonly runsDir: string;
    readonly policy: PoolPolicy;
    readonly repository: string;
  },
): Promise<ShippedRegionCensus> {
  /**
   Directory the settled artifacts sit in, named once so the listing, the
   census and the later reads cannot drift onto different paths.
   */
  const artifactsDir = artifactsDirOf({ runsDir, },);

  /**
   One directory listing, shared with the census.

   Taken once and threaded through, because a pass writes into this directory
   continuously: a second listing inside the census would classify a different
   set of files from the one this reader goes on to read.
   */
  const listed = await listSettledNames({ runsDir, },);

  return await collectTwoLaneShippedRegions({
    artifactsDir,
    files: keepEligible({
      names: listed,
      eligible: await resolvePool({
        artifactsDir,
        names: listed,
        policy,
        repository,
      },),
    },),
  },);
}

//endregion Damage sample pool
