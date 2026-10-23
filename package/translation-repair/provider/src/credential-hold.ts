import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ModelReach, } from './budget-routing.ts';
import { isCredentialRefusal, } from './provider-budget-refusal.ts';
import type {
  BudgetView,
  ProviderBudgets,
} from './provider-budget.ts';
import {
  reachPastBoundHolds,
  type StreamBoundHolds,
} from './stream-bound-hold.ts';

//region Credential hold
// A PROVIDER WHOSE CREDENTIAL WAS REFUSED IS HELD OUT FOR THE REST OF THE RUN,
// for every model it serves. zheermao19 (2026-10-20): the Bedrock key expired,
// every Bedrock call answered HTTP 401, and the seat tally withheld
// `gemma-4-26b-a4b-it` after four failed asks though OpenRouter, wet all run,
// serves it. The key is the provider's, one key serves every model there, and a
// key does not renew during a run, so the hold is the provider's and has no
// end: unlike the stream bound hold (`stream-bound-hold.ts`), which is one
// pair's and lapses, nothing a run does makes an expired token valid again.
//
// A MODEL THAT NO OTHER PROVIDER SERVES reads unreachable without a call, as it
// does under any hold, and is the one case where a seat is withheld for it.

/**
 Providers whose credential was refused this run.

 @example
 ```ts
 const holds = createCredentialHolds();
 holds.hold({ provider: 'bedrock', },);
 ```
 */
export type CredentialHolds = {
  /**
   Holds a provider out for the rest of the run; true when this call began
   the hold, false when it was already in force, so the one call that learns
   a provider's credential is refused is the one that says so.
   */
  readonly hold: (args: { readonly provider: ProviderName; },) => boolean;

  /**
   Whether a provider is held out.
   */
  readonly isHeld: (args: { readonly provider: ProviderName; },) => boolean;
};

/**
 Builds the credential holds one router keeps.

 @returns Holds shared by every call the router makes

 @example
 ```ts
 const holds = createCredentialHolds();
 ```
 */
export function createCredentialHolds(): CredentialHolds {
  /**
   Providers held out.
   */
  const held = new Set<ProviderName>();
  return {
    hold: function hold({ provider, },): boolean {
      if (held.has(provider,))
        return false;
      held.add(provider,);
      return true;
    },
    isHeld: function isHeld({ provider, },): boolean {
      return held.has(provider,);
    },
  };
}

/**
 A model's reach with the providers whose credential was refused removed.
 */
export type ReachPastCredentialHolds = {
  /**
   Providers that serve the model and have their credential accepted so far.
   */
  readonly reach: ModelReach;

  /**
   Providers that serve the model and are held out for a refused credential,
   in provider order.
   */
  readonly refused: readonly ProviderName[];
};

/**
 Removes from a model's reach every provider whose credential was refused.

 @param reach - providers that serve the model

 @param holds - the router's credential holds

 @returns Reach past the holds, and the serving providers taken out

 @example
 ```ts
 const { reach, refused, } = reachPastCredentialHolds({ reach, holds, },);
 ```
 */
export function reachPastCredentialHolds(
  {
    reach,
    holds,
  }: {
    readonly reach: ModelReach;
    readonly holds: CredentialHolds;
  },
): ReachPastCredentialHolds {
  /**
   Whether one provider serves the model and has its credential accepted.

   @param provider - provider read

   @returns True where it may take the call
   */
  function open(provider: ProviderName,): boolean {
    return reach[provider] && (!holds.isHeld({ provider, },));
  }

  return {
    reach: {
      neuralwatt: open('neuralwatt',),
      synthetic: open('synthetic',),
      hyper: open('hyper',),
      bedrock: open('bedrock',),
      deepseek: open('deepseek',),
      xiaomi: open('xiaomi',),
      anthropic: open('anthropic',),
      openrouter: open('openrouter',),
    },
    refused: PROVIDER_ORDER.filter(function servesAndRefused(provider,): boolean {
      return reach[provider] && holds.isHeld({ provider, },);
    },),
  };
}

/**
 Where a seat can go once a provider's credential is refused, among the
 providers that still serve it.
 */
export type SeatOutlook = {
  /**
   Providers that serve the model and can take a call now, in the order they
   are tried: not credential-held, not dry, not held for the model.
   */
  readonly open: readonly ProviderName[];

  /**
   Providers that serve the model and read dry.
   */
  readonly dry: readonly ProviderName[];

  /**
   Providers that serve the model and are held out for it on its stream bound.
   */
  readonly heldForModel: readonly ProviderName[];
};

/**
 Says what becomes of the seat.

 @param modelId - model whose call met the refusal

 @param outlook - the providers still serving the model, by what each can do now

 @returns The sentence for the line

 @example
 ```ts
 const sentence = seatSentence({ modelId, outlook, },);
 ```
 */
function seatSentence(
  {
    modelId,
    outlook: {
      open,
      dry,
      heldForModel,
    },
  }: {
    readonly modelId: string;
    readonly outlook: SeatOutlook;
  },
): string {
  if (open.length > 0)
    return `${modelId} moves to ${open.join(', then ',)} (every provider that serves it and can take a call now).`;

  /**
   Providers that serve the model but cannot take a call now, each with why, in provider order.
   */
  const waiting = PROVIDER_ORDER.flatMap(function reasonOf(candidate,): readonly string[] {
    if (dry.includes(candidate,))
      return [`${candidate} is dry`,];
    if (heldForModel.includes(candidate,))
      return [`${candidate} is held for this model`,];
    return [];
  },);
  if (waiting.length === 0)
    return `No provider that serves ${modelId} is left, so the seat is withheld for the rest of the run.`;
  return `No provider that serves ${modelId} can take a call now (${waiting.join(', ',)}), `
    + 'so the seat is not withheld on this refusal: the router goes on by its rules for a dry or held provider.';
}

/**
 The line said once per provider, when its credential is first refused.

 THREE ENDINGS FOR THE SEAT, because they are three facts. A provider that can
 take the call now is named as where the seat moves to. Providers that serve
 the model but are dry or held for it are named by that reason and the seat is
 not withheld on this refusal, since the router's rules for a dry or held
 provider decide what follows. Only a model no provider serves any longer is
 withheld.

 @param modelId - model whose call met the refusal

 @param provider - provider that refused its credential

 @param error - what the call threw, whose words the line quotes

 @param outlook - the providers still serving the model, by what each can do now

 @returns The warn line

 @example
 ```ts
 rl.warn(credentialRefusalLine({ modelId, provider: 'bedrock', error, outlook, },),);
 ```
 */
export function credentialRefusalLine(
  {
    modelId,
    provider,
    error,
    outlook,
  }: {
    readonly modelId: string;
    readonly provider: ProviderName;
    readonly error: unknown;
    readonly outlook: SeatOutlook;
  },
): string {
  /**
   The failure and the provider's own words.
   */
  const said = exchangeFailureLogText({ error, },);
  /**
   What becomes of the seat.
   */
  const seat = seatSentence({
    modelId,
    outlook,
  },);
  return `SEAT CREDENTIAL REFUSED ${modelId} on ${provider}: ${said}. `
    + `${provider} is held out for every model for the rest of the run, because one credential serves them all. `
    + `${seat} Renew ${provider}'s credential before the next run, or leave it.`;
}

/**
 Holds a provider out for a refused credential, saying so the first time.

 THE ONE PLACE A REFUSED CREDENTIAL IS HELD, reached from the call itself
 (`provider-router.ts`), so a first ask and the cross-provider re-ask hold the
 provider the same way. The budgets are read only for the call that began the
 hold, to say which providers can take the seat now.

 @param rl - logger the line goes to

 @param provider - provider that refused its credential

 @param modelId - model whose call met the refusal

 @param error - what the call threw

 @param reach - providers that serve the model, before any hold

 @param holds - the router's credential holds

 @param boundHolds - the router's stream bound holds, which also close a
 provider to the model the line names the seat's next provider from

 @param budgets - the router's budget view, read for which providers are dry

 @param signal - the call's abort signal, which the budget read honours

 @example
 ```ts
 await holdOnCredentialRefusal({ rl, provider, modelId, error, reach, holds, boundHolds, budgets, signal, },);
 ```
 */
export async function holdOnCredentialRefusal(
  {
    rl,
    provider,
    modelId,
    error,
    reach,
    holds,
    boundHolds,
    budgets,
    signal,
  }: {
    readonly rl: Logger;
    readonly provider: ProviderName;
    readonly modelId: string;
    readonly error: unknown;
    readonly reach: ModelReach;
    readonly holds: CredentialHolds;
    readonly boundHolds: StreamBoundHolds;
    readonly budgets: Pick<ProviderBudgets, 'read'>;
    readonly signal: AbortSignal;
  },
): Promise<void> {
  if (!holds.hold({ provider, },))
    return;

  /**
   Providers serving the model that have their credential accepted.
   */
  const { reach: accepted, } = reachPastCredentialHolds({
    reach,
    holds,
  },);

  /**
   Providers serving the model that the stream bound holds do not close to it.
   */
  const { reach: notHeld, } = reachPastBoundHolds({
    reach: accepted,
    modelId,
    holds: boundHolds,
  },);

  /**
   Which providers read dry now.
   */
  const dry = await budgets.read({ signal, },);

  rl.warn(credentialRefusalLine({
    modelId,
    provider,
    error,
    outlook: {
      open: PROVIDER_ORDER.filter(function canTakeNow(candidate,): boolean {
        return notHeld[candidate] && (!dry[candidate]);
      },),
      dry: PROVIDER_ORDER.filter(function servesAndDry(candidate,): boolean {
        return notHeld[candidate] && dry[candidate];
      },),
      heldForModel: PROVIDER_ORDER.filter(function servesAndHeld(candidate,): boolean {
        return accepted[candidate] && (!notHeld[candidate]);
      },),
    },
  },),);
}

/**
 Performs one provider call, holding the provider out when it answers that its
 credential was refused, and rethrowing every failure unchanged.

 @param call - the provider call to perform

 @param hold - everything `holdOnCredentialRefusal` needs but the failure

 @returns Whatever the call returned

 @throws Whatever the call threw, after a refused credential is held

 @example
 ```ts
 const reply = await callHoldingRefusedCredential({ call: askBedrock, hold, },);
 ```
 */
export async function callHoldingRefusedCredential<ReplyT,>(
  {
    call,
    hold,
  }: {
    readonly call: () => Promise<ReplyT>;
    readonly hold: Omit<Parameters<typeof holdOnCredentialRefusal>[0], 'error'>;
  },
): Promise<ReplyT> {
  try {
    return await call();
  }
  catch (error) {
    if (isCredentialRefusal({ error, },))
      await holdOnCredentialRefusal({
        ...hold,
        error,
      },);
    throw error;
  }
}

/**
 Reads the budget view with every provider whose credential was refused as
 dry, which is the one reading the seat readers, benches and quorum
 denominators are derived from.

 A METER SAYS WHETHER MONEY IS LEFT, NOT WHETHER THE KEY IS ACCEPTED. The
 Bedrock meter stayed wet through the run its token expired in, so a bench read
 from the meters alone counted its seats within reach until each was asked.

 @param dry - the budget view as the meters and refusal holds read it

 @param holds - the router's credential holds

 @returns The view with each held provider read dry

 @example
 ```ts
 const view = foldCredentialHolds({ dry: await budgets.read({ signal, },), holds, });
 ```
 */
export function foldCredentialHolds(
  {
    dry,
    holds,
  }: {
    readonly dry: BudgetView;
    readonly holds: CredentialHolds;
  },
): BudgetView {
  /**
   Whether one provider reads dry or has its credential refused.

   @param provider - provider read

   @returns True where nothing can be bought from it
   */
  function dryOrHeld(provider: ProviderName,): boolean {
    return dry[provider] || holds.isHeld({ provider, });
  }
  return {
    neuralwatt: dryOrHeld('neuralwatt',),
    synthetic: dryOrHeld('synthetic',),
    hyper: dryOrHeld('hyper',),
    bedrock: dryOrHeld('bedrock',),
    deepseek: dryOrHeld('deepseek',),
    xiaomi: dryOrHeld('xiaomi',),
    anthropic: dryOrHeld('anthropic',),
    openrouter: dryOrHeld('openrouter',),
  };
}

//endregion Credential hold
