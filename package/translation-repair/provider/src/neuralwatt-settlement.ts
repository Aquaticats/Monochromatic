import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import type { ExtractedCompletion, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { readNeuralWattComments, } from './neuralwatt-comments.ts';
import type { NeuralWattReservation, } from './neuralwatt-ledger.ts';
import { reportSpend, } from './spend-line.ts';

//region NeuralWatt settlement
// WHAT A FINISHED CALL TELLS THE METER. The stream's two comment lines
// (`neuralwatt-comments.ts`) settle the reservation the call held: the measured
// energy replaces the estimate, a missing energy comment leaves the estimate
// standing and marked.
//
// A COST IS REPORTED AND NEVER ACTED ON. The `: cost` comment is still parsed
// and its figure goes on the SPEND line, but it does not reach the ledger and
// ends nothing: NeuralWatt reports a nominal `request_cost_usd` on free-energy
// requests too. The owner, 2026-10-20: a call with 0.0167 Wh measured came back
// with 0.000107 USD, and the dashboard said "Your call only used 60.3J, which
// isn't even 0.0%", counted against the free energy. What did leave the free
// energy shows only in the plan-energy difference a run prints at its end
// (`neuralwatt-quota-session.ts`).

/**
 Logger root for the settlement.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Decimals an energy figure is logged to: a call is a few thousandths of a
 watt-hour.
 */
const ENERGY_LOG_DECIMALS = 4;

/**
 Settles one finished call against the day's energy and reports it.

 @param reservation - hold the call took before it was sent

 @param servedId - model the call went to

 @param extracted - completion reassembled from the stream

 @param bodyText - whole drained stream, comment lines included

 @param estimateWh - energy the call held

 @param promptTokens - prompt tokens the estimate was drawn for

 @throws {@link Error} when the ledger line cannot be written; the hold is
 released first

 @example
 ```ts
 await settleNeuralWattCall({ reservation, servedId, extracted, bodyText, estimateWh, promptTokens, },);
 ```
 */
export async function settleNeuralWattCall(
  {
    reservation,
    servedId,
    extracted,
    bodyText,
    estimateWh,
    promptTokens,
  }: {
    readonly reservation: NeuralWattReservation;
    readonly servedId: string;
    readonly extracted: ExtractedCompletion;
    readonly bodyText: string;
    readonly estimateWh: number;
    readonly promptTokens: number;
  },
): Promise<void> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: settleNeuralWattCall.name,
    l,
  },);

  /**
   What NeuralWatt said about this request's energy and cost.
   */
  const {
    energyWh,
    requestCostUsd,
  } = readNeuralWattComments({ bodyText, },);
  reportSpend({
    provider: 'neuralwatt',
    label: servedId,
    extracted,
    // Conditional spread keeps an unreported cost absent instead of zero.
    ...((requestCostUsd === undefined) ? {} : { costUsd: requestCostUsd, }),
  },);

  /**
   The energy half of the log line: measured against the estimate, or the
   estimate standing.
   */
  const energyNote = (energyWh === undefined)
    ? `not reported, ${estimateWh.toFixed(ENERGY_LOG_DECIMALS,)} Wh estimated stands`
    : `${energyWh.toFixed(ENERGY_LOG_DECIMALS,)} Wh measured against ${estimateWh.toFixed(ENERGY_LOG_DECIMALS,)} Wh estimated`;
  rl.info(
    `NeuralWatt energy ${servedId}: ${energyNote} for ${String(promptTokens,)} prompt ${
      wordForCount({
        count: promptTokens,
        one: 'token',
        many: 'tokens',
      },)
    }`,
  );
  // An unreported energy figure stays absent, so the estimate stands, marked.
  await reservation.settle((energyWh === undefined) ? {} : { measuredWh: energyWh, },);
}

//endregion NeuralWatt settlement
