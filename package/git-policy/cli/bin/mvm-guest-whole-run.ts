/**
 The one shape every guest driver's `all` step performs, so each driver keeps only the steps it names.

 A run makes a guest, prepares it, performs its arms, and takes the guest away. The guest goes whatever the arms said,
 because a guest is a machine of its own that outlives the command that made it, and one left behind holds a name no
 later run of the same driver can take. Only a run that never made its guest leaves one behind, and that is the one case
 where nothing is destroyed.
 */
import type { GuestFacts, } from './mvm-guest-recipe-step.ts';

/**
 One step of a run, and the facts it names its guest and its logs by.
 */
export type GuestRunStep = (run: { readonly chosen: GuestFacts; }) => Promise<void>;

/**
 The four steps one `all` run performs, in the order it performs them.
 */
export type GuestRunSteps = {
  /**
   Make the guest, which is the only step whose refusal leaves this run owning nothing to destroy.
   */
  readonly create: GuestRunStep;
  /**
   Prepare the guest the way its arms need it.
   */
  readonly prepare: GuestRunStep;
  /**
   Perform the run's own arms inside the prepared guest.
   */
  readonly work: GuestRunStep;
  /**
   Take the guest away, which runs even when the work refused.
   */
  readonly destroy: GuestRunStep;
};

/**
 Destroy the guest a refused work step left behind, keeping a destruction that refused beside the work's own refusal.

 @param destroy - the step that takes the guest away

 @param chosen - the facts the guest is named and logged by

 @param refusal - what the work refused with, which a failed destruction never hides

 @throws whatever the destruction refused with, when the work itself refused nothing

 @example
 ```ts
 await destroyAfterRefusal({ destroy: steps.destroy, chosen, refusal });
 ```
 */
async function destroyAfterRefusal({
  destroy,
  chosen,
  refusal,
}: {
  readonly destroy: GuestRunStep;
  readonly chosen: GuestFacts;
  readonly refusal: unknown;
},): Promise<void> {
  try {
    await destroy({ chosen, },);
  }
  catch (destruction) {
    if (refusal === undefined)
      throw destruction;
    console.error(`git-policy guest run: destroying '${chosen.name}' also failed: ${String(destruction,)}`,);
  }
}

/**
 Create, prepare, work and destroy one throwaway guest, destroying it even when the work refused.

 @param label - how this driver names itself in the lines it prints, such as `git-policy lock guest`

 @param chosen - the facts the guest is named and logged by

 @param steps - the four steps of this run, in the order they are performed

 @throws whatever the first step that refused raised, after the guest it created is gone

 @example
 ```ts
 await runWholeGuestRun({ label: 'git-policy lock guest', chosen, steps });
 ```
 */
export async function runWholeGuestRun({
  label,
  chosen,
  steps,
}: {
  readonly label: string;
  readonly chosen: GuestFacts;
  readonly steps: GuestRunSteps;
},): Promise<void> {
  try {
    await steps.create({ chosen, },);
  }
  catch (caught) {
    console.error(`${label}: no guest was destroyed, because this run did not create '${chosen.name}'`,);
    throw caught;
  }
  try {
    await steps.prepare({ chosen, },);
    await steps.work({ chosen, },);
  }
  catch (refusal) {
    await destroyAfterRefusal({
      destroy: steps.destroy,
      chosen,
      refusal,
    },);
    throw refusal;
  }
  await steps.destroy({ chosen, },);
}
