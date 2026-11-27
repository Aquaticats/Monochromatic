/**
 Create, run and destroy one guest, destroying it even when a step refused.

 A guest costs sixteen minutes of preparation and one build, so a run that fails
 in an arm still destroys what it created: the next round starts from nothing, and
 nobody has to find a guest by hand. A guest this run did not create is never
 destroyed, because `create` refuses a name `mvm list` already reports, and that
 guest belongs to somebody else.

 The keep decision is the driver's own: `--keep-guest` keeps a guest for a person
 to inspect and destroy, and says so once.

 @example
 ```ts
 await runWithAGuestThatIsDestroyed({ name, subject, keepGuest: false, create, run, destroy });
 ```
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

/**
 What one step of a run refused, so the guest is destroyed before it is thrown.
 */
export type RunFailure = {
  /**
   True for a step that refused, whose error is kept for after the destruction.
   */
  readonly kind: 'refused';
  /**
   What the step threw.
   */
  readonly error: Error;
} | {
  /**
   True for a step that ended without refusing.
   */
  readonly kind: 'none';
};

/**
 Read a caught value as the error a driver throws on.

 A driver keeps a caught value only to throw it after a guest is destroyed, and a
 value that is not an error still has to name what happened.

 @param caught - what was caught

 @returns the caught error, or a new one built from the caught value's text

 @example
 ```ts
 asError({ caught: new Error('the guest refused') });
 ```
 */
export function asError({
  caught,
}: {
  readonly caught: unknown;
},): Error {
  if (Error.isError(caught,)) return caught;
  return new Error(caughtValueText(caught,),);
}

/**
 Run one step and keep what it refused, so a caller can answer after the guest is
 gone.

 @param step - the step to run

 @returns what the step refused, or that it ended

 @example
 ```ts
 const failure = await keepFailure({ step: runEveryArm });
 ```
 */
export async function keepFailure({
  step,
}: {
  readonly step: () => Promise<void>;
},): Promise<RunFailure> {
  try {
    await step();
    return { kind: 'none', };
  }
  catch (caught) {
    return {
      kind: 'refused',
      error: asError({ caught, },),
    };
  }
}

/**
 Create one guest, run the work, and destroy the guest whatever the work did.

 @param name - the guest's name, which every message and the keep decision name

 @param subject - the words every message about this guest starts with

 @param keepGuest - whether the guest is kept for a person to inspect

 @param create - the step that creates the guest

 @param run - the work the guest was created for

 @param destroy - the step that destroys the guest

 @throws Error naming what ended the run, after the guest it created is gone

 @example
 ```ts
 await runWithAGuestThatIsDestroyed({ name: 'guest-one', subject: 'git-policy probe guest', keepGuest: false, create, run, destroy });
 ```
 */
export async function runWithAGuestThatIsDestroyed({
  name,
  subject,
  keepGuest,
  create,
  run,
  destroy,
}: {
  readonly name: string;
  readonly subject: string;
  readonly keepGuest: boolean;
  readonly create: () => Promise<void>;
  readonly run: () => Promise<void>;
  readonly destroy: () => Promise<void>;
},): Promise<void> {
  try {
    await create();
  }
  catch (caught) {
    console.error(`${subject}: no guest was destroyed, because this run did not create '${name}'`);
    throw asError({ caught, },);
  }
  /**
   What the work refused, which the destruction below answers.
   */
  const failure = await keepFailure({ step: run, },);
  if (keepGuest) {
    console.log(`${subject}: --keep-guest kept '${name}' to inspect and destroy`);
    if (failure.kind === 'refused') throw failure.error;
    return;
  }
  /**
   What the destruction itself refused, absent when it worked.
   */
  const destruction = await keepFailure({ step: destroy, },);
  if ((failure.kind === 'none') && (destruction.kind === 'refused')) throw destruction.error;
  if ((failure.kind === 'refused') && (destruction.kind === 'refused'))
    console.error(`${subject}: destroying '${name}' also failed: ${caughtValueText(destruction.error,)}`);
  if (failure.kind === 'refused') throw failure.error;
}
