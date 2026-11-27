/**
 The one shape every guest driver's `all` step performs, so each driver keeps only the steps it names.

 A run makes a guest, prepares it, performs its arms, and takes the guest away. The guest goes whatever the arms said,
 because a guest is a machine of its own that outlives the command that made it, and one left behind holds a name no
 later run of the same driver can take. Only a run that never made its guest leaves one behind, and that is the one case
 where nothing is destroyed.

 `--keep-guest` is the driver's own exception: a person inspecting a failed run asks for the guest to stay, and the run
 says so once instead of destroying it. Two of the three sets of guest helpers held this shape
 (`bin/mvm-guest-whole-run.ts` and `e2e/guest-host/guest-run-lifecycle.ts`); the surviving one is `mvm-guest`'s, whose
 order and destruction are held by a unit test, with the keep decision the other set's drivers needed.
 */
import type { GuestFacts, } from './mvm-guest-guest-facts.ts';

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
 What one stretch of a run refused, so the guest is destroyed before it is thrown.
 */
type RunFailure = {
  /**
   True for a stretch that refused, whose error is kept for after the destruction.
   */
  readonly kind: 'refused';
  /**
   What the stretch threw.
   */
  readonly error: unknown;
} | {
  /**
   True for a stretch that ran to its end.
   */
  readonly kind: 'none';
};

/**
 Run one stretch of a run and keep what it refused, so the caller can answer after the guest is gone.

 @param step - the steps to run

 @returns what the stretch refused, or that it ended

 @example
 ```ts
 const failure = await keepFailure({ step: prepareAndWork });
 ```
 */
async function keepFailure({ step }: { readonly step: () => Promise<void> },): Promise<RunFailure> {
  try {
    await step();
    return { kind: 'none', };
  }
  catch (caught) {
    return {
      kind: 'refused',
      error: caught,
    };
  }
}

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
  readonly refusal: RunFailure;
},): Promise<void> {
  try {
    await destroy({ chosen, },);
  }
  catch (destruction) {
    console.error(`git-policy guest run: destroying '${chosen.name}' also failed: ${String(destruction,)}`,);
  }
}

/**
 Create, prepare, work and destroy one throwaway guest, destroying it even when the work refused.

 @param label - how this driver names itself in the lines it prints, such as `git-policy lock guest`

 @param chosen - the facts the guest is named and logged by

 @param keepGuest - whether this run keeps the guest for a person to inspect instead of destroying it

 @param steps - the four steps of this run, in the order they are performed

 @throws whatever the first step that refused raised, after the guest it created is gone

 @example
 ```ts
 await runWholeGuestRun({ label: 'git-policy lock guest', chosen, keepGuest: false, steps });
 ```
 */
export async function runWholeGuestRun({
  label,
  chosen,
  keepGuest,
  steps,
}: {
  readonly label: string;
  readonly chosen: GuestFacts;
  readonly keepGuest: boolean;
  readonly steps: GuestRunSteps;
},): Promise<void> {
  try {
    await steps.create({ chosen, },);
  }
  catch (caught) {
    console.error(`${label}: no guest was destroyed, because this run did not create '${chosen.name}'`,);
    throw caught;
  }
  /**
   What the preparation and the work refused, absent when they ran to their end.
   */
  const failure = await keepFailure({
    step: async function prepareAndWork(): Promise<void> {
      await steps.prepare({ chosen, },);
      await steps.work({ chosen, },);
    },
  },);
  if (keepGuest) {
    console.log(`${label}: --keep-guest kept '${chosen.name}' to inspect and destroy`,);
    if (failure.kind === 'refused')
      throw failure.error;
    return;
  }
  if (failure.kind === 'refused') {
    await destroyAfterRefusal({
      destroy: steps.destroy,
      chosen,
      refusal: failure,
    },);
    throw failure.error;
  }
  await steps.destroy({ chosen, },);
}
