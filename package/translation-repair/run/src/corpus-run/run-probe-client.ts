import {
  fetchTransport,
  type ModelTransport,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { runClientFrom, } from './run-config.ts';
import type { RunClient, } from './run-client-contract.ts';

//region Probe run client
// THE RUN CLIENT A PROBE COMMAND BUILDS, with the hold bound its command lacks. The corpus pass has a hard cap and
// hands it to the budgets as the longest a stated wait can matter; a probe, a bench or a census has none, and a budgets
// instance built without a bound once held a provider out for as long as a refusal stated: `Retry-After: 99999999999`
// is about 3,170 years, and the wait for the hold polls once a second.

/**
 Longest hold, in milliseconds, a refusal's stated wait may set on a provider in a probe, bench or census command.

 THIRTY MINUTES. These commands are attended and last minutes to an hour, and have no hard cap to borrow. A provider
 that states a longer return (Hyper's daily limit answered "try again in 2h25m18s" on 2026-09-07; Anthropic's
 `retry-after` has stated an hour) is not one a command waits for: the operator reruns it when the provider is back.
 The cut is logged once for each provider with the value the provider stated, so it shows in the run's log.
 */
export const PROBE_MAX_HOLD_MS = 1_800_000;

/**
 The run client of a probe command over the given environment and transport, cut at {@link PROBE_MAX_HOLD_MS}.

 @param env - environment the provider keys are read from

 @param transport - HTTP every provider client calls over

 @returns Ready client, routed across configured providers and counted per seat

 @throws {@link RunConfigError} when every provider key variable is unset or empty

 @example
 ```ts
 const client = probeRunClientFrom({ env: { TRANSLATION_REPAIR_SYNTHETIC_API_KEY: 'test', }, transport, },);
 ```
 */
export function probeRunClientFrom(
  {
    env,
    transport,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly transport: ModelTransport;
  },
): RunClient {
  return runClientFrom({
    env,
    transport,
    maxHoldMs: PROBE_MAX_HOLD_MS,
  },);
}

/**
 The run client a probe, bench or census command calls over: the process's environment and the live transport, cut at
 {@link PROBE_MAX_HOLD_MS}. A command with a hard cap uses `createRunClient` and hands it.

 @returns Ready client, routed across configured providers and counted per seat

 @throws {@link RunConfigError} when every provider key variable is unset or empty

 @example
 ```ts
 const client = createProbeRunClient();
 ```
 */
export function createProbeRunClient(): RunClient {
  return probeRunClientFrom({
    env: process.env,
    transport: fetchTransport,
  },);
}

//endregion Probe run client
