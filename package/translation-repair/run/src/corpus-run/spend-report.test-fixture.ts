//region Spend report fixture
// Log lines a cat-themed run wrote, as the spend report reads them: the logger's
// own prefix, then the record the spend writer returns.
//
// MODEL IDS ARE THE ONES THE PRICE TABLE KNOWS, or one it does not, since what
// the report does with a seat depends on its provider and on whether a rate
// is on file for its model.

/**
 Puts the logger's prefix in front of a spend record.

 @param record - record text, from the marker word onward

 @returns Line shaped the way a run log holds it

 @example
 ```ts
 const line = loggedSpend({ record: 'SPEND provider=hyper model=qwen3.8-max prompt=1 completion=2', },);
 ```
 */
function loggedSpend({ record, }: { readonly record: string; },): string {
  return `[info] [2026-08-25T01:28:57.289Z] [translation-repair] [reportSpend] ${record}`;
}

/**
 A metered call on a model the price table prices dear: a million prompt
 tokens at forty credits and half a million completion tokens at a hundred
 and twenty come to a hundred credits.
 */
export const HYPER_DEAR: string = loggedSpend({
  record: 'SPEND provider=hyper model=qwen3.8-max prompt=1000000 completion=500000',
},);

/**
 A metered call on a model the price table prices cheap: a million of each
 come to 8.64 credits at the rates read 2026-10-11 (10.84 at those of 2026-09-11).
 */
export const HYPER_CHEAP: string = loggedSpend({
  record: 'SPEND provider=hyper model=gemma-4-26b-a4b-it prompt=1000000 completion=1000000',
},);

/**
 A metered call on a model no row of the price table names.
 */
export const HYPER_UNPRICED: string = loggedSpend({
  record: 'SPEND provider=hyper model=whisker-mini-9 prompt=10 completion=20',
},);

/**
 A call on the flat subscription, which bills no credits.
 */
export const SUBSCRIPTION_CALL: string = loggedSpend({
  record: 'SPEND provider=synthetic model=hf:zai-org/GLM-5.3-Flash prompt=300 completion=40',
},);

/**
 A call on OpenRouter whose line carried its cost in USD.
 */
export const OPENROUTER_COSTED: string = loggedSpend({
  record: 'SPEND provider=openrouter model=minimax/minimax-m3 prompt=10 completion=20 cost=0.0625',
},);

/**
 A call on the same seat whose line carried no cost.
 */
export const OPENROUTER_UNCOSTED: string = loggedSpend({
  record: 'SPEND provider=openrouter model=minimax/minimax-m3 prompt=5 completion=5',
},);

/**
 A metered call whose provider reported no usage block.
 */
export const HYPER_QUIET: string = loggedSpend({
  record: 'SPEND provider=hyper model=qwen3.8-max prompt=unreported completion=unreported',
},);

/**
 A metered call written as a reckoning: an attempt abandoned before it finished.
 */
export const HYPER_RECKONED: string = loggedSpend({
  record: 'SPEND provider=hyper model=qwen3.8-max prompt=1000000 completion=500000 estimated=abandoned',
},);

/**
 A line that carries the marker and is cut off before its fields end.
 */
export const CUT_SPEND_LINE: string = loggedSpend({ record: 'SPEND provider=hyper mod', },);

//endregion Spend report fixture
