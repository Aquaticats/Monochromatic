import type { ChatTextReply, } from './chat-contract.ts';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Provider router reply
// What a routed call and a re-ask on another provider come back with, apart
// from the router that performs them. Split from `provider-router.ts` at its
// line budget.

/**
 One answer plus the provider that produced it.

 THE PROVIDER TRAVELS WITH THE REPLY because the schema re-ask needs to know
 where NOT to ask again, and nothing in the reply itself records it.

 @example
 ```ts
 const answered: RoutedReply = { provider: 'hyper', reply, };
 ```
 */
export type RoutedReply = {
  /**
   Provider that served this call.
   */
  readonly provider: ProviderName;

  /**
   What it answered.
   */
  readonly reply: ChatTextReply;
};

/**
 What a re-ask came back with.

 A NAMED REFUSAL RATHER THAN AN ABSENT REPLY, because the two are different
 facts: the provider was asked and said no on budget, which the caller keeps
 the first answer over, as opposed to never having been asked.

 @example
 ```ts
 const asked: ReAskReply = { kind: 'budget-refused', };
 ```
 */
export type ReAskReply = {
  /**
   Provider answered.
   */
  readonly kind: 'replied';

  /**
   What it said.
   */
  readonly reply: ChatTextReply;
} | {
  /**
   Provider refused on budget, and its hold has started.
   */
  readonly kind: 'budget-refused';
};

//endregion Provider router reply
