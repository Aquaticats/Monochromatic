/**
 Removing what a failed step made, and saying what is left when that removal
 itself fails.

 A create that cannot finish deletes its instance before it reports the failure,
 so a failed create leaves nothing billed. When the delete itself fails nothing
 can be done from here, so the error names the instance, says that it keeps being
 billed, and prints the one command that removes it.

 A clone's snapshot is deleted the same way, but its failure is only logged: a
 snapshot left behind is swept by `destroy --all`, and it must never mask the
 clone's own failure.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import {
  deleteInstance,
  deleteSnapshot,
} from './api-resources.ts';
import type { OvhInstance, } from './types.ts';

//region Instance

/**
 Deletes an instance and reports what deleting it said.
 
 @param instanceId - instance to delete
 
 @returns `''` when the delete worked, otherwise what it answered
 
 @example
 ```ts
 const failure = await deleteQuietly({ instanceId: 'abc' });
 ```
 */
async function deleteQuietly({
  instanceId,
}: {
  readonly instanceId: string;
},): Promise<string> {
  try {
    await deleteInstance({ instanceId, },);
    return '';
  }
  catch (error: unknown) {
    return caughtValueText(error,);
  }
}

/**
 Deletes an instance whose provisioning failed, then builds the error to throw:
 one saying nothing is left billed, or one naming the command that removes an
 instance the delete could not remove.
 
 @param failure - what went wrong after the instance existed
 
 @param instance - instance to delete
 
 @param name - VM name without the mvm- prefix, named in the message and in the command
 
 @param rl - logger for progress
 
 @returns the error the caller throws
 
 @example
 ```ts
 catch (error) { throw await failedProvisionError({ failure: error, instance, name, rl }); }
 ```
 */
export async function failedProvisionError({
  failure,
  instance,
  name,
  rl,
}: {
  readonly failure: unknown;
  readonly instance: OvhInstance;
  readonly name: string;
  readonly rl: Logger;
},): Promise<Error> {
  /**
   What deleting the instance answered, `''` when the delete worked.
   */
  const deleteFailure = await deleteQuietly({ instanceId: instance.id, },);
  if (deleteFailure === '') {
    rl.info(`deleted ${instance.name} after the failed create; nothing is left billed`,);
    return new Error(
      `creating ${name} failed: ${
        caughtValueText(failure,)
      }. The instance was deleted, so nothing is left billed.`,
    );
  }
  rl.info(`could not delete ${instance.name}: ${deleteFailure}`,);
  return new Error(
    `creating ${name} failed: ${
      caughtValueText(failure,)
    }. Deleting the instance also failed: ${deleteFailure}. The instance keeps being billed until it is deleted; remove it with:\n  mvm --backend ovh destroy ${name}`,
  );
}

//endregion Instance

//region Snapshot

/**
 Deletes a clone's snapshot, logging rather than throwing: a snapshot that
 cannot be removed is swept by `destroy --all`.
 
 @param imageId - snapshot image identifier
 
 @param rl - logger for the cleanup outcome
 
 @example
 ```ts
 await safeDeleteSnapshot({ imageId: 'abc', rl });
 ```
 */
export async function safeDeleteSnapshot({
  imageId,
  rl,
}: {
  readonly imageId: string;
  readonly rl: Logger;
},): Promise<void> {
  rl.debug(`deleting intermediate snapshot ${imageId}`,);
  try {
    await deleteSnapshot({ imageId, },);
  }
  catch (error: unknown) {
    rl.info(
      `snapshot ${imageId} cleanup failed: ${caughtValueText(error,)}`,
    );
  }
}

//endregion Snapshot
