/**
 What a killed commit service's claims must come to once a service runs again:
 the push and the upkeep of the landed commit each settled once, by the service that took them back.

 Empty work directories prove only that nothing is outstanding.
 A remote that holds the commit proves nothing about the push record either,
 because the push of a later commit carries its ancestors.
 The service writes the number of times it took an item back from a holder that did not renew its claim into the item,
 so a record that was settled with a count of one or more was claimed by the dead service, taken back and run by its successor.

 @module
 */

import type { Expectations, } from './scenario-model-fixture.ts';
import type { ScenarioRepository, } from './repository-fixture.ts';
import { workRecordPlacements, } from './work-record-fixture.ts';

//region Constants

/**
 The kinds of after-commit work a landed commit leaves, which the service claims together.
 */
const CLAIMED_KINDS = [
  'push',
  'maintenance',
] as const;

/**
 The work directory of a finished item.
 */
const DONE = 'done';

//endregion Constants

//region Expectations

/**
 Names the files of an item for a report.

 @param placements - the item's files

 @returns `state/name` pairs, or `none`

 @example
 ```ts
 describePlacements([{ state: 'done', name: 'push-abc', tries: 1 }]);
 ```
 */
function describePlacements(placements: Awaited<ReturnType<typeof workRecordPlacements>>,): string {
  if (placements.length === 0)
    return 'none';
  return placements.map(function describe(placement,) {
    return `${placement.state}/${placement.name} (tries ${String(placement.tries,)})`;
  },)
    .join(', ',);
}

/**
 Finds the files of the items a landed commit leaves for the service.

 @param repository - scenario repository

 @param commit - the landed commit

 @returns each item's id and files

 @example
 ```ts
 await itemsOf({ repository, commit });
 ```
 */
async function itemsOf({
  repository,
  commit,
}: Readonly<{
  repository: ScenarioRepository;
  commit: string;
}>,): Promise<readonly Readonly<{
  id: string;
  placements: Awaited<ReturnType<typeof workRecordPlacements>>
}>[]> {
  return await Promise.all(CLAIMED_KINDS.map(async function placementsOf(kind,) {
    return {
      id: `${kind}-${commit}`,
      placements: await workRecordPlacements({
        repository,
        id: `${kind}-${commit}`,
      },),
    };
  },),);
}

/**
 What the killed service held on the items of a landed commit.
 */
export type ClaimsAtKill = Readonly<{
  /**
   Expectation that the push was claimed when the service died,
   so the scenario tests the recovery of a claim and not a kill that came after the work.
   */
  expectation: Expectations[number];
  /**
   Ids of the items that were in `claimed`.
   The service claims its pending items one after another, so the kill can come between the push and the upkeep:
   an item still pending at the kill is run by the successor without having been taken back.
   */
  claimed: readonly string[];
}>;

/**
 Looks at what the dead service held on the items of a landed commit.

 @param repository - scenario repository

 @param commit - the landed commit

 @returns the expectation and the ids of the claimed items

 @example
 ```ts
 await claimsAtKill({ repository, commit });
 ```
 */
export async function claimsAtKill({
  repository,
  commit,
}: Readonly<{
  repository: ScenarioRepository;
  commit: string;
}>,): Promise<ClaimsAtKill> {
  /**
   Files of each item, by item.
   */
  const items = await itemsOf({
    repository,
    commit,
  },);
  /**
   Ids of the items found in `claimed`, once each.
   */
  const claimed = items.filter(function isClaimed(item,) {
    return (item.placements
      .length
      === 1) && (item.placements[0]
        ?.state
        === 'claimed');
  },)
    .map(function idOf(item,) {
      return item.id;
    },);
  return {
    claimed,
    expectation: {
      name: 'service-killed-holding-the-push-claim',
      holds: claimed.includes(`push-${commit}`,),
      detail: items.map(function describe(item,) {
        return `${item.id}: ${describePlacements(item.placements,)}`;
      },)
        .join('; ',),
    },
  };
}

/**
 Expectation that every item of the landed commit was settled exactly once,
 and that each one the dead service had claimed was taken back from it on the way.

 @param repository - scenario repository

 @param commit - the landed commit

 @param claimed - ids of the items the dead service held, from {@link claimsAtKill}

 @returns the expectation record

 @example
 ```ts
 await claimsRunExpectation({ repository, commit, claimed });
 ```
 */
export async function claimsRunExpectation({
  repository,
  commit,
  claimed,
}: Readonly<{
  repository: ScenarioRepository;
  commit: string;
  claimed: readonly string[];
}>,): Promise<Expectations[number]> {
  /**
   Files of each item, by item.
   */
  const items = await itemsOf({
    repository,
    commit,
  },);
  return {
    name: 'claimed-work-settled-once-by-the-successor',
    holds: items.every(function settledOnce(item,) {
      return (item.placements
        .length
        === 1) && (item.placements[0]
          ?.state
          === DONE)
        && (claimed.includes(item.id,) ? (item.placements[0]
          ?.tries
          ?? 0) >= 1 : true);
    },),
    detail: items.map(function describe(item,) {
      return `${item.id}: ${describePlacements(item.placements,)}`;
    },)
      .join('; ',),
  };
}

//endregion Expectations
