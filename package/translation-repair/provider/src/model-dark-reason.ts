import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  PROVIDER_ORDER,
  type ProviderName,
  type ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Model dark reason
// WHY ONE MODEL HAS NO PROVIDER, in words a read of the run can tell from the run-ending stop. A model whose own
// providers are all out of budget while another provider reads wet is a lost seat (the router raises
// `NoProviderForModelError` for it); every provider out of budget at once is the stop that ends the run
// (`EveryProviderDryError`). The two read alike in a log unless the reason names which providers serve the model and
// how each reads, so this does.

/**
 Reading of one provider in the words the run-ending refusal's measured clause uses.

 @param provider - provider read

 @param dry - which providers read dry

 @returns Provider's name and `dry` or `wet`

 @example
 ```ts
 readingOf({ provider: 'hyper', dry, },); // => 'hyper dry'
 ```
 */
function readingOf(
  {
    provider,
    dry,
  }: {
    readonly provider: ProviderName;
    readonly dry: ProviderRecord<boolean>;
  },
): string {
  return `${provider} ${dry[provider] ? 'dry' : 'wet'}`;
}

/**
 The reason a model that no usable provider can take gives for it.

 @param reach - providers that serve the model at all

 @param dry - which providers have nothing buyable, budget reading and anything learned at the wire taken together

 @returns Either that no provider serves the model, or that every provider serving it is out of budget with how each
 provider reads, the ones serving the model first and then the ones that do not

 @example
 ```ts
 modelDarkReason({ reach, dry, },);
 // => 'every provider serving this model is out of budget (hyper dry); the providers that do not serve it read ...'
 ```
 */
export function modelDarkReason(
  {
    reach,
    dry,
  }: {
    readonly reach: ProviderRecord<boolean>;
    readonly dry: ProviderRecord<boolean>;
  },
): string {
  /**
   Providers that serve the model, in spending order.
   */
  const serving = PROVIDER_ORDER.filter(function serves(provider,): boolean {
    return reach[provider];
  },);
  if (serving.length === 0)
    return 'no provider serves this model';

  /**
   Providers that do not serve the model, in spending order.
   */
  const notServing = PROVIDER_ORDER.filter(function doesNotServe(provider,): boolean {
    return !reach[provider];
  },);
  /**
   The sentence about the providers serving the model.
   */
  const servingClause = `every provider serving this model is out of budget (${
    serving.map(function readingOfServing(provider,): string {
      return readingOf({
        provider,
        dry,
      },);
    },)
      .join(', ',)
  })`;
  if (notServing.length === 0)
    return servingClause;
  return `${servingClause}; the providers that do not serve it read ${
    notServing.map(function readingOfOther(provider,): string {
      return readingOf({
        provider,
        dry,
      },);
    },)
      .join(', ',)
  }`;
}

/**
 How long a model that went dark is not warned about again: the budget layer's refusal cooldown, 300 s, since a dark
 model stays dark at least that long and a fleet asked for it every round, repeating the same line hundreds of times.
 */
export const DARK_MODEL_WARN_COOLDOWN_MS = 300_000;

/**
 Decides, once per model per cooldown, whether the warn for a dark model is written.

 @param now - clock the cooldown is read by, injectable for tests

 @param cooldownMs - how long a model is not warned about again

 @returns Check that is true the first time a model is asked about and again once the cooldown has passed

 @example
 ```ts
 const due = createDarkWarnings({ now: monotonicMs, },);
 due({ modelId: 'minimax-m3', },); // => true, then false until the cooldown has passed
 ```
 */
export function createDarkWarnings(
  {
    now,
    cooldownMs = DARK_MODEL_WARN_COOLDOWN_MS,
  }: {
    readonly now: () => number;
    readonly cooldownMs?: number;
  },
): (args: { readonly modelId: string; },) => boolean {
  /**
   When each model was last warned about.
   */
  const warnedAt = new Map<string, number>();
  return function isDue({ modelId, },): boolean {
    /**
     The instant of the last warn for this model.
     */
    const last = warnedAt.get(modelId,);
    if ((last !== undefined) && ((now() - last) < cooldownMs))
      return false;
    warnedAt.set(
      modelId,
      now(),
    );
    return true;
  };
}

/**
 Refuses a call whose model has no usable provider, saying in the log which model went dark and how each provider read,
 so a read of the run can tell a lost seat from the stop that ends the run.

 @param modelId - model that has no usable provider

 @param reason - what {@link modelDarkReason} gave for it

 @param l - logger the line goes to

 @param warn - whether the line is written; the refusal is thrown either way (default: it is)

 @throws {@link NoProviderForModelError} always, which the stage layers read as one seat's lost voice

 @example
 ```ts
 refuseDarkModel({ modelId, reason: modelDarkReason({ reach, dry, },), l, },);
 ```
 */
export function refuseDarkModel(
  {
    modelId,
    reason,
    l,
    warn = true,
  }: {
    readonly modelId: string;
    readonly reason: string;
    readonly l: Logger;
    readonly warn?: boolean;
  },
): never {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: refuseDarkModel.name,
    l,
  },);
  if (warn)
    rl.warn(`${modelId} went dark: ${reason}`,);
  throw new NoProviderForModelError({
    modelId,
    reason,
  },);
}

//endregion Model dark reason
