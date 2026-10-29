import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type CredentialHolds,
  credentialRefusalLine,
} from './credential-hold.ts';
import type {
  Decider,
  DecisionReply,
  DecisionRequest,
} from './decision-contract.ts';
import {
  isBudgetRefusal,
  isCredentialRefusal,
  isPaymentRefusal,
  statedWaitMsOf,
} from './provider-budget-refusal.ts';
import type { ProviderBudgets, } from './provider-budget.ts';

//region Provider router decide
// THE TYPED-DECISION EXCHANGE, served by OpenRouter alone. Split from
// `provider-router.ts` at its line budget.

/**
 Typed-decision exchange, served by OpenRouter alone: refused when the
 OpenRouter meter reads dry or its credential was refused, and a budget refusal
 marks that meter as a chat refusal would.

 @param request - state and questions to decide

 @param decider - client of OpenRouter's decisions endpoint, absent when the
 run has no OpenRouter key

 @param budgets - shared budget view every call is routed by

 @param credentialHolds - router's credential holds, whose OpenRouter entry
 the one credential of chat and decisions shares

 @param l - logger the credential line goes to

 @returns Answers as the endpoint gave them

 @throws {@link NoProviderForModelError} when no decisions client is
 configured, the OpenRouter meter reads dry, or its credential was refused

 @example
 ```ts
 const reply = await routedDecide({ request, decider, budgets, credentialHolds, l, },);
 ```
 */
export async function routedDecide(
  {
    request,
    decider,
    budgets,
    credentialHolds,
    l,
  }: {
    readonly request: ForeignBorrowed<DecisionRequest>;
    readonly decider?: Decider;
    readonly budgets: ProviderBudgets;
    readonly credentialHolds: CredentialHolds;
    readonly l: Logger;
  },
): Promise<DecisionReply> {
  if (decider === undefined) {
    throw new NoProviderForModelError({
      modelId: request.modelId,
      reason: 'no decisions client is configured',
    },);
  }

  // ONE OPENROUTER CREDENTIAL SERVES CHAT AND DECISIONS ALIKE, so a chat call
  // that found it refused ends the decisions too, without a call.
  if (credentialHolds.isHeld({ provider: 'openrouter', })) {
    throw new NoProviderForModelError({
      modelId: request.modelId,
      reason: 'the decisions endpoint had its credential refused this run (openrouter)',
    },);
  }

  /**
   Every meter as it reads now.
   */
  const dry = await budgets.read({ signal: request.signal, },);
  if (dry.openrouter) {
    throw new NoProviderForModelError({
      modelId: request.modelId,
      reason: 'the decisions endpoint reads dry',
    },);
  }
  try {
    /**
     What the decisions endpoint answered.
     */
    const reply = await decider.decide(request,);
    budgets.markServed?.({ provider: 'openrouter', },);
    return reply;
  }
  catch (error) {
    if (isCredentialRefusal({ error, },) && credentialHolds.hold({ provider: 'openrouter', },)) {
      /**
       Logger pre-tagged with this function's name.
       */
      const rl = tagged({
        tag: routedDecide.name,
        l,
      },);
      rl.warn(credentialRefusalLine({
        modelId: request.modelId,
        provider: 'openrouter',
        error,
        outlook: {
          open: [],
          dry: [],
          heldForModel: [],
        },
      },),);
    }
    if (isBudgetRefusal({ error, },)) {
      await budgets.markRefused({
        provider: 'openrouter',
        signal: request.signal,
        statedWaitMs: statedWaitMsOf({ error, },),
        paymentRequired: isPaymentRefusal({ error, },),
      },);
    }
    throw error;
  }
}

//endregion Provider router decide
