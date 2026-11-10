/**
 Disposable session observations for Pi goal runtime verification.
 
 @module
 */

import type { SessionManager, } from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Recorded goal event queries

/**
 Count persisted goal events of selected kind.
 
 @param sessionManager - real disposable session manager
 
 @param kind - goal event kind under test
 
 @returns number of matching selected-branch events
 
 @example
 ```ts
 goalEventCount({ sessionManager, kind: 'run_cleared' });
 ```
 */
function goalEventCount(
  {
    sessionManager,
    kind,
  }: {
    readonly sessionManager: SessionManager;
    readonly kind: string;
  },
): number {
  return sessionManager
    .getBranch()
    .filter(function matchesGoalEvent(
      entry: ForeignBorrowed<ReturnType<SessionManager['getBranch']>[number]>,
    ): boolean {
      if ((entry.type !== 'custom') || (entry.customType !== 'goal:state'))
        return false;
      return (entry.data !== null)
        && ((typeof entry.data) === 'object')
        && ('kind' in entry.data)
        && (entry.data
          .kind
          === kind);
    },)
    .length;
}

/**
 Collect persisted goal event kinds from supplied entries.
 
 @param entries - selected branch or complete session entries
 
 @returns ordered goal event kinds
 
 @example
 ```ts
 goalEventKinds(sessionManager.getBranch());
 ```
 */
function goalEventKinds(
  entries: ForeignBorrowed<ReturnType<SessionManager['getEntries']>>,
): readonly string[] {
  return entries.flatMap(function goalEventKind(entry,) {
    if ((entry.type !== 'custom') || (entry.customType !== 'goal:state'))
      return [];
    if ((entry.data === null)
      || ((typeof entry.data) !== 'object')
      || (!('kind' in entry.data))) {
      return [];
    }
    return [String(entry.data
      .kind,),];
  },);
}

/**
 Count task continuation messages in selected branch.
 
 @param sessionManager - real disposable session manager
 
 @returns persisted continuation-message count
 
 @example
 ```ts
 goalContinuationMessageCount(sessionManager);
 ```
 */
function goalContinuationMessageCount(sessionManager: SessionManager,): number {
  return sessionManager
    .getBranch()
    .filter(function matchesContinuation(
      entry: ForeignBorrowed<ReturnType<SessionManager['getBranch']>[number]>,
    ): boolean {
      if ((entry.type !== 'custom_message') || (entry.customType !== 'goal'))
        return false;
      return (entry.details !== null)
        && ((typeof entry.details) === 'object')
        && ('kind' in entry.details)
        && (entry.details
          .kind
          === 'continuation');
    },)
    .length;
}

//endregion Recorded goal event queries

export {
  goalContinuationMessageCount,
  goalEventCount,
  goalEventKinds,
};
