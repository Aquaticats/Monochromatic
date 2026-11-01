import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { ModelClient, } from './chat-contract.ts';
import type { RouteTimeHoldsRule, } from './reask-alternate-models.ts';

//region Reading for a re-ask
// THE READING A RE-ASK LEAVES OUT THE WITHHELD MODELS BY (ledger B1330). A caller that can read which providers are dry
// now hands that reading to the list of alternates, which then leaves out only the models the run withholds in it; a
// caller that cannot, or whose read fails, gets the list that leaves out every model with a hold applied at seating
// time, so a withheld model is never asked on a guess.

/**
 The rule for the holds applied at seating time that a re-ask reads, from a reading taken now.

 @param readDry - reads which providers are dry, absent where the caller knows no reading

 @param signal - caller's abort signal, whose abort a failed read hands back instead of hiding

 @param l - logger the failed read is written to

 @returns The rule that applies the reading, or the rule that applies every such hold where there is no reading or it
 could not be read

 @throws Whatever the read threw when the caller aborted

 @example
 ```ts
 const routeTimeHolds = await rulePastReading({ readDry: client.providerDryness, signal, l, },);
 ```
 */
export async function rulePastReading(
  {
    readDry,
    signal,
    l,
  }: {
    readonly readDry: ModelClient['providerDryness'];
    readonly signal: AbortSignal;
    readonly l: Logger;
  },
): Promise<RouteTimeHoldsRule> {
  if (readDry === undefined)
    return { applied: 'always', };
  try {
    return {
      applied: 'in-reading',
      dry: await readDry({ signal, },),
    };
  }
  catch (error) {
    if (signal.aborted)
      throw error;
    l.warn(
      `the budget view could not be read (${exchangeFailureLogText({ error, },)}); `
        + 'leaving out every model that carries a hold the run applies at seating time',
    );
    return { applied: 'always', };
  }
}

//endregion Reading for a re-ask
