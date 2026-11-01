import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  type BudgetView,
  type ChatTextReply,
  createRoutingClient,
} from '../dist/final/node/index.mjs';
import {
  holdNothing,
  markNothingRefused,
} from './provider-router-wet-budgets.test-fixture.ts';

//region Provider router recording
// A ROUTER OVER STUB CALLERS THAT RECORD WHICH PROVIDER EACH CALL REACHED, over a reading of the meters the case hands in.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The re-ask hold test and the first-ask hold test both ask which provider the router
// sends a held model to, so the router they build is one function and not two copies that could drift.

/**
 What the router under test wrote and where its calls went.
 */
export type RecordingRouter = {
  /**
   Router over the stub callers.
   */
  readonly client: ReturnType<typeof createRoutingClient>;

  /**
   Providers the calls reached, in order.
   */
  readonly reached: ProviderName[];

  /**
   Lines the router wrote, behind their levels.
   */
  readonly lines: string[];
};

/**
 Builds a router over stub callers that answer and record their provider.

 @param read - reading of the meters the router chooses by, at every decision

 @returns Router, the providers its calls reached and the lines it wrote

 @example
 ```ts
 const { client, reached, } = routerRecordingProviders({ read: readOlderProvidersWet, },);
 ```
 */
export function routerRecordingProviders(
  { read, }: { readonly read: () => Promise<BudgetView>; },
): RecordingRouter {
  /**
   Providers the calls reached.
   */
  const reached: ProviderName[] = [];

  /**
   Lines the router wrote.
   */
  const lines: string[] = [];

  /**
   Builds one provider's caller.

   @param provider - provider this caller stands for

   @returns Caller that records its provider and answers
   */
  function callerFor(provider: ProviderName,): { readonly chatText: () => Promise<ChatTextReply>; } {
    return {
      chatText: function chatText(): Promise<ChatTextReply> {
        reached.push(provider,);
        return Promise.resolve({ text: `${provider} answers`, },);
      },
    };
  }

  return {
    reached,
    lines,
    client: createRoutingClient({
      callers: {
        neuralwatt: callerFor('neuralwatt',),
        synthetic: callerFor('synthetic',),
        hyper: callerFor('hyper',),
        openrouter: callerFor('openrouter',),
      },
      budgets: {
        read,
        markRefused: markNothingRefused,
        holds: holdNothing,
      },
      l: levelCapturingLogger({ lines, },),
    },),
  };
}

//endregion Provider router recording
