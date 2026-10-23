import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  ModelReach,
  ProviderChoice,
} from './budget-routing.ts';
import {
  PROVIDER_ORDER,
  type ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { RequestPace, } from './request-pace.ts';

/**
 Request windows of the providers that pace their calls; a provider absent
 here does not pace.

 @example
 ```ts
 const paces: ProviderPaces = { hyper: hyperClient.pace, };
 ```
 */
export type ProviderPaces = {
  /**
   NeuralWatt's window, when it paces; it does not today.
   */
  readonly neuralwatt?: Pick<RequestPace, 'waitMs'>;

  /**
   Synthetic's window, when it paces.
   */
  readonly synthetic?: Pick<RequestPace, 'waitMs'>;

  /**
   Hyper's window, when it paces.
   */
  readonly hyper?: Pick<RequestPace, 'waitMs'>;

  /**
   Bedrock's window, when it paces.
   */
  readonly bedrock?: Pick<RequestPace, 'waitMs'>;

  /**
   DeepSeek's window, when it paces; it states a concurrency limit and no
   request rate, so it does not.
   */
  readonly deepseek?: Pick<RequestPace, 'waitMs'>;

  /**
   Xiaomi's window, when it paces: 100 requests a minute.
   */
  readonly xiaomi?: Pick<RequestPace, 'waitMs'>;

  /**
   The Claude key's window, when it paces: 1000 requests a minute.
   */
  readonly anthropic?: Pick<RequestPace, 'waitMs'>;

  /**
   OpenRouter's window, when it paces.
   */
  readonly openrouter?: Pick<RequestPace, 'waitMs'>;
};

//region Pace saturation
// CLASS ONE HUNDRED FORTY-NINE (hulicaijia30, 2026-09-26). Hyper's thousand
// request starts in a rolling hour were spent in the pass's first 33
// minutes; its pacer then made the next call wait 1,640,012 ms, and the
// router kept sending Hyper the models OpenRouter serves too, because a wet
// meter was all it read. A request window that would make the call wait is
// the provider's capacity in use, the same state as a per-model slot limit
// taken: the call overflows to a usable provider behind it, and queues on the
// paced provider only when nobody usable stands behind. No wait is too short
// to count, so no threshold is chosen here.

/**
 Logger root for the pace saturation.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 How long each provider's request window would make a call wait now, zero
 for a provider that does not pace its calls.

 @param paces - request windows of the providers that pace their calls

 @returns Wait per provider

 @example
 ```ts
 const waits = paceWaitsOf({ paces: { hyper: hyperClient.pace, }, },);
 ```
 */
export function paceWaitsOf(
  { paces, }: { readonly paces: ProviderPaces; },
): ProviderRecord<number> {
  /**
   One provider's wait, zero where it does not pace.

   @param provider - provider read

   @returns Milliseconds its window would make a call wait
   */
  function waitOf(provider: keyof ProviderRecord<number>,): number {
    /**
     This provider's window, absent where it does not pace.
     */
    const pace = paces[provider];
    return (pace === undefined) ? 0 : pace.waitMs();
  }
  return {
    neuralwatt: waitOf('neuralwatt',),
    synthetic: waitOf('synthetic',),
    hyper: waitOf('hyper',),
    bedrock: waitOf('bedrock',),
    deepseek: waitOf('deepseek',),
    xiaomi: waitOf('xiaomi',),
    anthropic: waitOf('anthropic',),
    openrouter: waitOf('openrouter',),
  };
}

/**
 Saturation with every provider whose request window would make the call
 wait read as saturated too.

 @param saturated - providers whose per-model slots are all taken

 @param waits - how long each provider's window would make the call wait

 @returns Saturation the routing policy overflows on

 @example
 ```ts
 const saturated = withFullWindows({ saturated: ledger.saturated({ modelId, },), waits, },);
 ```
 */
export function withFullWindows(
  {
    saturated,
    waits,
  }: {
    readonly saturated: ProviderRecord<boolean>;
    readonly waits: ProviderRecord<number>;
  },
): ProviderRecord<boolean> {
  return {
    neuralwatt: saturated.neuralwatt || (waits.neuralwatt > 0),
    synthetic: saturated.synthetic || (waits.synthetic > 0),
    hyper: saturated.hyper || (waits.hyper > 0),
    bedrock: saturated.bedrock || (waits.bedrock > 0),
    deepseek: saturated.deepseek || (waits.deepseek > 0),
    xiaomi: saturated.xiaomi || (waits.xiaomi > 0),
    anthropic: saturated.anthropic || (waits.anthropic > 0),
    openrouter: saturated.openrouter || (waits.openrouter > 0),
  };
}

/**
 Logs each usable provider the call passed over because its request window
 was full, naming where the call went instead.

 @param modelId - model being routed

 @param reach - providers that serve the model

 @param dry - providers with nothing buyable

 @param waits - how long each provider's window would make the call wait

 @param chosen - provider the policy picked, or why none could be

 @example
 ```ts
 logWindowOverflow({ modelId, reach, dry, waits, chosen: choice.kind, },);
 ```
 */
export function logWindowOverflow(
  {
    modelId,
    reach,
    dry,
    waits,
    chosen,
  }: {
    readonly modelId: string;
    readonly reach: ModelReach;
    readonly dry: ProviderRecord<boolean>;
    readonly waits: ProviderRecord<number>;
    readonly chosen: ProviderChoice['kind'];
  },
): void {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: logWindowOverflow.name,
    l,
  },);
  if (chosen === 'unreachable')
    return;
  /**
   Usable providers the call passed over for a full window.
   */
  const passedOver = PROVIDER_ORDER
    .filter(function passed(provider,): boolean {
      return reach[provider]
        && (!dry[provider])
        && (waits[provider] > 0)
        && (provider !== chosen);
    },);
  for (const provider of passedOver) {
    rl.info(
      `${modelId}: ${provider}'s request window is full for another ${String(waits[provider],)}ms; `
        + `the call goes to ${chosen}`,
    );
  }
}

//endregion Pace saturation
