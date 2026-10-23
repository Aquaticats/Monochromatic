import type { Logger, } from '@monochromatic-dev/module-logger/ts';

import type {
  MeterState,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  type ProviderRecord,
  providerRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { RunClient, } from './run-client-contract.ts';

//region Seat reading meter states
// THE WORDS A `JUDGE SEATS` LINE GIVES EACH PROVIDER'S METER, and the read of the states that feeds them, split out of
// `run-seats-read.ts` at its line budget.

/**
 The state of a meter whose states could not be read: nothing was learned.

 @returns Unreadable

 @example
 ```ts
 const meters = providerRecord({ of: unreadableMeter, },);
 ```
 */
function unreadableMeter(): MeterState {
  return 'unreadable';
}

/**
 Names how a provider reads in the `JUDGE SEATS` line: dry when the router
 holds it out, otherwise what its meter said, an unreadable meter spelled out
 as the spendable provider the router treats it as.

 NOT `wet` FOR A METER THAT SAID NOTHING. The routed bit counts an unreadable
 meter as spendable (`provider-budget.ts`), which is the recorded policy and
 stays; the line only stops claiming the meter answered. TWO WORDS, NO SPACE,
 so each `name=state` token still reads on its own.

 @param routedDry - whether the router holds the provider out, holds and payment refusals included

 @param meter - what the provider's meter said

 @returns State word the line prints

 @example
 ```ts
 const word = stateWordOf({ routedDry: false, meter: 'unreadable', },);
 // => 'unreadable-treated-as-spendable'
 ```
 */
export function stateWordOf(
  {
    routedDry,
    meter,
  }: {
    readonly routedDry: boolean;
    readonly meter: MeterState;
  },
): string {
  if (routedDry)
    return 'dry';
  return (meter === 'unreadable') ? 'unreadable-treated-as-spendable' : 'wet';
}

/**
 Reads what each meter said, or that none could be read when the states
 cannot be had.

 @param client - run client whose meter states are the router's own

 @param signal - entry abort

 @param l - entry logger, which records a failed read

 @returns What each meter said, every meter unreadable when the read failed

 @example
 ```ts
 const meters = await readMeterStates({ client, signal, l, },);
 ```
 */
export async function readMeterStates(
  {
    client,
    signal,
    l,
  }: {
    readonly client: Pick<RunClient, 'providerMeterStates'>;
    readonly signal: AbortSignal;
    readonly l: Logger;
  },
): Promise<ProviderRecord<MeterState>> {
  try {
    return await client.providerMeterStates({ signal, },);
  } catch (error) {
    l.warn(
      `judge seats: the meter states could not be read (${exchangeFailureLogText({ error, },)}); `
        + 'naming every meter unreadable',
    );
    return providerRecord({ of: unreadableMeter, },);
  }
}

//endregion Seat reading meter states
