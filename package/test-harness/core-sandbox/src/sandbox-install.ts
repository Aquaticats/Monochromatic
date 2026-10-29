/**
 Whole-object installation owns its introduced descriptors even when Sinon throws. @module
 */
import { restoreSandboxSteps, } from './sandbox-cleanup.ts';
import { guardCollectionFakes, } from './sandbox-collection.ts';
import { SandboxCleanupError, } from './sandbox-error.ts';
import {
  invokeSandboxMethod,
  type SandboxInvocation,
} from './sandbox-guard.ts';
import { collectionFakes, } from './sandbox-member.ts';
import type { SandboxOwner, } from './sandbox-owner.ts';

/**
 Restores only the failed operation's new fakes, never unrelated earlier sandbox work.

 @param invocation - ordinary whole-object stub or spy call
 
 @param target - mutation destination validated by the operation dispatcher
 
 @param owner - attempt owning introduced fakes
 
 @param restoring - runner-owned cleanup phase
 
 @returns unchanged collection identity
 
 @throws SandboxCleanupError when partial rollback also fails

 @example
 ```ts
 return invokeCollectionFactory({ invocation, target, owner, restoring });
 ```
 */
export function invokeCollectionFactory({
  invocation,
  target,
  owner,
  restoring,
}: {
  readonly invocation: SandboxInvocation;
  readonly target: object;
  readonly owner: SandboxOwner;
  readonly restoring: () => boolean;
},): unknown {
  /**
   Snapshot descriptors rather than reading user getters.
   */
  const previous = Object.getOwnPropertyDescriptors(target,);
  try {
    /**
     Sinon registers a whole collection only after its complete construction succeeds.
     */
    const result = invokeSandboxMethod(invocation,);
    guardCollectionFakes({
      value: result,
      previous,
      owner,
      restoring,
    },);
    return result;
  }
  catch (error) {
    /**
     Retained partial members must share lifetime guards even after their rollback.
     */
    guardCollectionFakes({
      value: target,
      previous,
      owner,
      restoring,
    },);
    /**
     Each introduced member has independent cleanup, including accessor spies.
     */
    const steps = collectionFakes({
      value: target,
      previous,
    },)
      .map(function partialRestorer(fake: object,): () => void {
      /**
       Capture before another member's restoration changes the destination descriptor.
       */
        const restore: unknown = Reflect.get(
          fake,
          'restore',
        );
        return function restorePartialFake(): void {
          if ((typeof restore) === 'function')
            Reflect.apply(
              restore,
              fake,
              [],
            );
        };
      },);
    try {
      restoreSandboxSteps(steps,);
    }
    catch (rollbackError) {
      throw new SandboxCleanupError(
        [
          error,
          rollbackError,
        ],
        `${invocation.operation} failed and partial replacement restoration failed`,
      );
    }
    throw error;
  }
}
