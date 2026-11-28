/**
 One entry per step this driver answers, so the command line's word and the work it names live in one table, and each
 other file stays inside this package's line bound.

 The three steps that are the ZFS binding's guest recipe are here too, with the driver's own template words, so every name
 a person may type answers from this one map.

 @example
 ```ts
 await theSteps({ chosen, extraArgumentWords: [] }).get('layout')?.();
 ```
 */

import type { GuestFacts, } from '../../bin/mvm-guest-guest-facts.ts';
import { runGuestRecipeStep, } from '../../bin/mvm-guest-recipe-step.ts';

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
import {
  buildInGuest,
  harnessInGuest,
} from './build.ts';
import { releaseBinary, } from './fixture.ts';
import {
  buildAndInstall,
  guestLayout,
  installTheProgram,
  recordTheCheckout,
} from './layout.ts';
import { migrateAndRollBack, } from './migration.ts';

/**
 One step of the ZFS binding's guest recipe, run with the driver's own template words.

 @param chosen - the run's guest, whose name the recipe hands to every later step

 @param step - the recipe's own step

 @param extraArgumentWords - the template words the driver's command line gave

 @returns the step

 @example
 ```ts
 recipeStep({ chosen, step: 'create', extraArgumentWords: [] });
 ```
 */
function recipeStep({
  chosen,
  step,
  extraArgumentWords,
}: {
  readonly chosen: GuestFacts;
  readonly step: string;
  readonly extraArgumentWords: readonly string[];
},): () => Promise<void> {
  return async function runTheRecipeStep() {
    await runGuestRecipeStep({
      name: chosen.name,
      logDirectory: chosen.logDir,
      step,
      extraArgumentWords,
    },);
  };
}

/**
 The steps this driver knows, and nothing else.

 @param chosen - the run's guest and log folder

 @param extraArgumentWords - the template words the driver's command line gave, which the recipe steps take

 @returns one call per step name

 @example
 ```ts
 theSteps({ chosen, extraArgumentWords: ['--image', 'cachyos-7.3.5-1-zfs-2.4.4'] });
 ```
 */
export function theSteps({
  chosen,
  extraArgumentWords,
}: {
  readonly chosen: GuestFacts;
  readonly extraArgumentWords: readonly string[];
},): ReadonlyMap<string, () => Promise<void>> {
  return new Map([
    [
      'create',
      recipeStep({
        chosen,
        step: 'create',
        extraArgumentWords,
      },),
    ],
    [
      'prepare',
      recipeStep({
        chosen,
        step: 'prepare',
        extraArgumentWords,
      },),
    ],
    [
      'destroy',
      recipeStep({
        chosen,
        step: 'destroy',
        extraArgumentWords,
      },),
    ],
    [
      'layout',
      async function layout() {
        await guestLayout({ chosen, },);
      },
    ],
    [
      'build',
      async function build() {
        await buildAndInstall({ chosen, },);
      },
    ],
    [
      'rebuild',
      async function rebuild() {
        // A build over the tree the last one left, so cargo reuses every dependency artifact.
        await buildInGuest({
          chosen,
          fresh: false,
        },);
        await installTheProgram({
          chosen,
          releaseBinary,
        },);
        await recordTheCheckout({ chosen, },);
      },
    ],
    [
      'harness',
      async function harness() {
        await harnessInGuest({ chosen, },);
      },
    ],
    [
      'record',
      async function record() {
        await recordTheCheckout({ chosen, },);
      },
    ],
    [
      'units',
      async function units() {
        await theUnitsReadBack({ chosen, },);
      },
    ],
    [
      'refusals',
      async function refusals() {
        await theRefusals({ chosen, },);
      },
    ],
    [
      'underlay',
      async function underlay() {
        await theImmutableEntry({ chosen, },);
      },
    ],
    [
      'snapdir',
      async function snapdir() {
        await theSnapdirInheritance({ chosen, },);
      },
    ],
    [
      'boot',
      async function boot() {
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
      },
    ],
    [
      'boot-install',
      async function bootInstall() {
        await installThePlan({ chosen, },);
      },
    ],
    [
      'boot-clone',
      async function bootClone() {
        await makeTheClone({ chosen, },);
      },
    ],
    [
      'boot-reboot',
      async function bootReboot() {
        await rebootTheGuest({ chosen, },);
      },
    ],
    [
      'boot-read',
      async function bootRead() {
        await readTheBoot({
          chosen,
          expect: 'healthy',
        },);
      },
    ],
    [
      'boot-read-failed',
      async function bootReadFailed() {
        await readTheBoot({
          chosen,
          expect: 'failed',
        },);
      },
    ],
    [
      'migration',
      async function migration() {
        await migrateAndRollBack({ chosen, },);
      },
    ],
    [
      'remove',
      async function remove() {
        await removeThePlan({ chosen, },);
      },
    ],
  ],);
}
