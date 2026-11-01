//region Provider identity
// WHO CAN SERVE A CALL, named once, in the order the owner prefers to spend.
//
// This file imports nothing on purpose, the way `roster-id.ts` does: the
// budget layer, the router, the seat reader and the spend readers all need
// these names, and any of them importing another for the sake of a union
// would be a cycle waiting to happen.
//
// THE ORDER IS THE POLICY. NeuralWatt's free daily energy is spent first
// (the paragraph headed NEURALWATT JOINED ON 2026-10-20 says why); Synthetic
// is a flat subscription and costs nothing at the margin; Charm Hyper is a
// prepaid balance the owner will not top up again (2026-09-03: Charm Hyper
// ended its bundle subsidization, so Hyper will not be recharged from the
// usual subscription, as
// `doc/decision/translation-repair-openrouter-fallback.md` records the owner's
// answer); OpenRouter is paid per token and is where the owner would rather
// spend. So a call goes to
// the first provider in this order that serves its model and still has budget,
// and the routing arithmetic in `budget-routing.ts` walks this list rather
// than naming providers one by one.
//
// AMAZON BEDROCK JOINED ON 2026-09-07, SECOND IN THE ORDER: the owner's 200 USD
// of credits there expire early next year and will never be topped up ("you're
// allowed to use it as much as you like. But I will NEVER top it up"), where
// OpenRouter's credits are the owner's to top up. So prepaid and expiring
// money is spent before money that would be bought. It landed third, behind
// Hyper, and the owner moved it ahead the same evening ("Yes Bedrock sit
// ahead"): the two seats both serve had gone to Hyper the moment Synthetic
// dried, spending the balance the Hyper-only seats run on, while the expiring
// money answered nothing.
//
// NEURALWATT JOINED ON 2026-10-20, FIRST IN THE ORDER. The owner's key there
// includes 550 Wh of free energy a day on DeepSeek V4.1 Flash and GLM 5.3
// Flash (500 Wh on the Max plan and 50 Wh for subscribing to product news),
// which resets at 00:00 UTC and "doesn't carry over"
// (`https://docs.neuralwatt.com/billing/daily-free-energy`, fetched
// 2026-10-20). Money that expires at the end of the day is the most expiring
// money there is, so by the rule above it is spent before everything else,
// and the owner said where it goes: "Use the neuralwatt free energy front and
// center." (2026-10-20). Past the 550 Wh a call bills the owner's normal plan,
// which the owner uses for other things ("overage eats into my normal
// plan"), so the provider reads dry the moment its own meter says the day's
// energy is spent (`neuralwatt-ledger.ts`) and the walk down this order goes
// on to Synthetic. Record: `doc/decision/translation-repair-neuralwatt-provider.md`.
//
// DEEPSEEK AND XIAOMI JOINED ON 2026-10-22, AFTER BEDROCK AND AHEAD OF HYPER AND
// OPENROUTER. OPEN TO THE OWNER'S VETO. The owner granted both keys that day
// for one model each: "You can also use the DEEPSEEK_API_KEY as you wish, but
// you may only use deepseek v4.1 flash 1017 with that key" and "You may only
// use v2.6-flash with this key" (the Xiaomi MiMo key). The reason for the
// place is the Bedrock precedent of 2026-09-07 over again: Hyper's balance is
// never renewed and is what the Hyper-only seats run on, so a seat another
// provider can serve spends elsewhere first, and OpenRouter is where the owner
// would rather buy. DeepSeek's prepaid balance was given to use "as you wish",
// so it is money already set aside and goes before Hyper's. Xiaomi bills per
// token and serves only the one seat OpenRouter otherwise serves, at the same
// listed price, so it goes ahead of OpenRouter and behind the balances that do
// not renew. DeepSeek sits ahead of Xiaomi because its money is prepaid and
// Xiaomi's is bought as it is used. Neither route promises zero data
// retention; the owner's ruling of 2026-10-05 that the corpus is public and
// PII is allowed, and the grants quoted here, are what permit it. Record:
// `doc/decision/translation-repair-direct-providers.md`.
//
// ANTHROPIC JOINED ON 2026-10-22, AFTER XIAOMI AND AHEAD OF HYPER. OPEN TO THE
// OWNER'S VETO. The owner added `TRANSLATION_REPAIR_CLAUDE_API_KEY` that day:
// "You may use latest Haiku and latest Sonnet with this key. This key would have
// $200 every month so be careful what you use." The place has no routing effect
// today, because no other provider serves a Claude model and a walk down this
// order only ever asks the providers that serve the model; it matters if one
// ever does. It sits with the other direct, owner-keyed routes and ahead of the
// balances that do not renew, since a monthly allowance that is not spent is
// gone. The owner ruled it needs no hard monthly cap of its own ("a month is
// defined very vaguely"), so its meter never reads dry (`claude-spend.ts`);
// exhaustion is learned from the billing refusal at the wire.

/**
 Every provider, in the order the owner prefers to spend on them.

 THE TYPE IS DERIVED FROM THIS LIST, so a provider cannot be added to the type
 without being placed in the order, and every walk down the order reaches it.

 @example
 ```ts
 const [preferred,] = PROVIDER_ORDER;
 ```
 */
export const PROVIDER_ORDER = [
  'neuralwatt',
  'synthetic',
  'bedrock',
  'deepseek',
  'xiaomi',
  'anthropic',
  'hyper',
  'openrouter',
] as const;

/**
 One of the providers this pipeline can buy a call from.

 @example
 ```ts
 const provider: ProviderName = 'openrouter';
 ```
 */
export type ProviderName = typeof PROVIDER_ORDER[number];

/**
 One value per provider, keyed by name.

 A RECORD RATHER THAN A FIELD PER PROVIDER, so adding a provider is one union member
 and one order entry, and every site that reads all providers fails to
 compile until it reads the new one too.

 @example
 ```ts
 const dry: ProviderRecord<boolean> = { neuralwatt: false, synthetic: false, bedrock: false, deepseek: false, xiaomi: false, anthropic: false, hyper: true, openrouter: false, };
 ```
 */
export type ProviderRecord<ValueT,> = Readonly<Record<ProviderName, ValueT>>;

/**
 Builds one record by asking a function about each provider in order.

 @param of - value for one provider

 @returns Record with every provider filled

 @example
 ```ts
 const holds = providerRecord({ of: function none(): number { return 0; }, },);
 ```
 */
export function providerRecord<ValueT,>(
  { of, }: { readonly of: (provider: ProviderName,) => ValueT; },
): ProviderRecord<ValueT> {
  return {
    neuralwatt: of('neuralwatt',),
    synthetic: of('synthetic',),
    bedrock: of('bedrock',),
    deepseek: of('deepseek',),
    xiaomi: of('xiaomi',),
    anthropic: of('anthropic',),
    hyper: of('hyper',),
    openrouter: of('openrouter',),
  };
}

/**
 Every provider but one, in spending order.

 @param provider - provider to leave out

 @returns The others, in the order the owner prefers to spend on them

 @example
 ```ts
 otherProviders({ provider: 'hyper', },);
 // => ['neuralwatt', 'synthetic', 'bedrock', 'deepseek', 'xiaomi', 'anthropic', 'openrouter',]
 otherProviders({ provider: 'synthetic', },);
 // => ['neuralwatt', 'bedrock', 'deepseek', 'xiaomi', 'anthropic', 'hyper', 'openrouter',]
 ```
 */
export function otherProviders(
  { provider, }: { readonly provider: ProviderName; },
): readonly ProviderName[] {
  return PROVIDER_ORDER.filter(function isOther(candidate,): boolean {
    return candidate !== provider;
  },);
}

/**
 Whether a string names a provider.

 @param value - untrusted spelling, from a CLI flag or a log line

 @returns Whether it is one of the provider names

 @example
 ```ts
 if (isProviderName(value,)) route(value,);
 ```
 */
export function isProviderName(value: string,): value is ProviderName {
  return PROVIDER_ORDER.some(function names(candidate,): boolean {
    return candidate === value;
  },);
}

//endregion Provider identity
