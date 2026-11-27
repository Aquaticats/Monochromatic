/**
 The one shape every guest driver's `all` step performs, so each driver keeps only the steps it names.

 A run makes a guest, prepares it, performs its arms, and takes the guest away. The guest goes whatever the arms said,
 because a guest is a machine of its own that outlives the command that made it, and one left behind holds a name no
 later run of the same driver can take. Only a run that never made its guest leaves one behind, and that is the one case
 where nothing is destroyed.
 */

/**
 The four steps one `all` run performs, in the order it performs them.
 
 */
export type GuestRunSteps = {
  /**
   Make the guest, which is the only step whose refusal leaves this run owning nothing to destroy.
   */
  readonly create: () => Promise<void>;
  /**
   Prepare the guest the way its arms need it.
   */
  readonly prepare: () => Promise<void>;
  /**
   Perform the run's own arms inside the prepared guest.
   */
  readonly work: () => Promise<void>;
  /**
   Take the guest away, which runs even when the work refused.
   */
  readonly destroy: () => Promise<void>;
};

/**
 Destroy the guest a refused work step left behind, keeping a destruction that refused beside the work's own refusal.

 @param destroy - the step that takes the guest away

 @param name - the guest's name, which a logged destruction failure names

 @param refusal - what the work refused with, which a failed destruction never hides

 @throws whatever the destruction refused with, when the work itself refused nothing

 @example
 ```ts
 await destroyAfterRefusal({ destroy: steps.destroy, name: 'guest-a', refusal });
 ```
 */
async function destroyAfterRefusal({
  destroy,
  name,
  refusal,
}: {
  readonly destroy: () => Promise<void>;
  readonly name: string;
  readonly refusal: unknown;
},): Promise<void> {
  try {
    await destroy();
  }
  catch (destruction) {
    if (refusal === undefined)
      throw destruction;
    console.error(`git-policy guest run: destroying '${name}' also failed: ${String(destruction,)}`,);
  }
}

/**
 Create, prepare, work and destroy one throwaway guest, destroying it even when the work refused.

 @param label - how this driver names itself in the lines it prints, such as `git-policy lock guest`

 @param name - the guest's name, which every printed line and refusal names

 @param steps - the four steps of this run, in the order they are performed

 @throws whatever the first step that refused raised, after the guest it created is gone

 @example
 ```ts
 await runWholeGuestRun({ label: 'git-policy lock guest', name: 'guest-a', steps });
 ```
 */
export async function runWholeGuestRun({
  label,
  name,
  steps,
}: {
  readonly label: string;
  readonly name: string;
  readonly steps: GuestRunSteps;
},): Promise<void> {
  try {
    await steps.create();
  }
  catch (caught) {
    console.error(`${label}: no guest was destroyed, because this run did not create '${name}'`,);
    throw caught;
  }
  try {
    await steps.prepare();
    await steps.work();
  }
  catch (refusal) {
    await destroyAfterRefusal({
      destroy: steps.destroy,
      name,
      refusal,
    },);
    throw refusal;
  }
  await steps.destroy();
}
