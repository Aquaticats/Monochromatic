/**
 The plan's arms in the order a boot would meet them, and the one run every arm of this driver as `all` runs.

 The order is the plan's: the texts are parsed and read back first, then the two refusals that changed them, then the
 immutable entry and the `snapdir` value the setup sets, then the boots themselves, and the migration last, because it
 writes a checkout record for a checkout of its own.

 @example
 ```ts
 await everyArm({ chosen });
 ```
 */

import type { GuestFacts, } from '../../bin/mvm-guest-guest-facts.ts';

import {
  theImmutableEntry,
  theRefusals,
  theSnapdirInheritance,
  theUnitsReadBack,
} from './arms.ts';
import {
  installThePlan,
  makeTheClone,
  readTheBoot,
  rebootTheGuest,
  removeThePlan,
  theFailingBoots,
} from './boot.ts';
import { migrateAndRollBack, } from './migration.ts';

/**
 Every arm of this driver, in the order the plan names them.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError naming the step that refused, or the marker a pulled log lacks

 @example
 ```ts
 await everyArm({ chosen });
 ```
 */
export async function everyArm({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  await theUnitsReadBack({ chosen, },);
  await theRefusals({ chosen, },);
  await theImmutableEntry({ chosen, },);
  await theSnapdirInheritance({ chosen, },);
  await installThePlan({ chosen, },);
  await rebootTheGuest({ chosen, },);
  await readTheBoot({
    chosen,
    expect: 'healthy',
  },);
  await makeTheClone({ chosen, },);
  await rebootTheGuest({ chosen, },);
  await readTheBoot({
    chosen,
    expect: 'healthy',
  },);
  await theFailingBoots({ chosen, },);
  await migrateAndRollBack({ chosen, },);
  await removeThePlan({ chosen, },);
}
