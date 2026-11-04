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
}>,): Promise<readonly Readonly<{ id: string; placements: Awaited<ReturnType<typeof workRecordPlacements>>; }>[]> {
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
 Expectation that the service held a claim on every item of a landed commit when it was killed,
 so the scenario tests the recovery of a claim and not a kill that came after the work.

 @param repository - scenario repository

 @param commit - the landed commit

 @returns the expectation record

 @example
 ```ts
 await claimsHeldExpectation({ repository, commit });
 ```
 */
export async function claimsHeldExpectation({
  repository,
  commit,
}: Readonly<{
  repository: ScenarioRepository;
  commit: string;
}>,): Promise<Expectations[number]> {
  /**
   Files of each item, by item.
   */
  const items = await itemsOf({
    repository,
    commit,
  },);
  return {
    name: 'service-killed-holding-every-claim',
    holds: items.every(function claimedOnce(item,) {
      return (item.placements.length === 1) && (item.placements[0]?.state === 'claimed');
    },),
    detail: items.map(function describe(item,) {
      return `${item.id}: ${describePlacements(item.placements,)}`;
    },)
      .join('; ',),
  };
}

/**
 Expectation that every item the dead service claimed was taken back and settled exactly once.

 @param repository - scenario repository

 @param commit - the landed commit

 @returns the expectation record

 @example
 ```ts
 await claimsRunExpectation({ repository, commit });
 ```
 */
export async function claimsRunExpectation({
  repository,
  commit,
}: Readonly<{
  repository: ScenarioRepository;
  commit: string;
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
      return (item.placements.length === 1) && (item.placements[0]?.state === DONE)
        && ((item.placements[0]?.tries ?? 0) >= 1);
    },),
    detail: items.map(function describe(item,) {
      return `${item.id}: ${describePlacements(item.placements,)}`;
    },)
      .join('; ',),
  };
}

//endregion Expectations
