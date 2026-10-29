import type { ClaudeAttemptAsk, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { ModelTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { ProviderOff, } from './providers-off.ts';
import type { RunClient, } from './run-client-contract.ts';
import {
  createRunClientWithFloors,
  runClientWithFloors,
} from './run-config.ts';
import { unmetBenchFloors, } from './run-seats-give-up-floors.ts';

//region Run client creation
// THE RUN CLIENT EVERY RUNNER CALLS OVER, WITH THE SEAT READER'S BENCH FLOORS WIRED IN. `run-config.ts` builds the client
// and needs the floors an entry's give-up of a provider must leave met (`unmetFloors`), which are the seat reader's own
// (`run-seats-give-up-floors.ts`). The seat reader reads `run-config.ts` for the roster, so `run-config.ts` cannot
// import it back (a dependency cycle, which the lint forbids): it takes the floors as a REQUIRED parameter, so no client
// is built with a floor nobody chose, and these two functions, the ones the package exports, hand them over.

/**
 Builds the client every run calls from every configured provider, counting every call, over the seat reader's bench
 floors.

 @param options - what {@link runClientWithFloors} of `run-config.ts` takes but the floors

 @returns Ready client, routed across configured providers and counted per seat

 @throws {@link RunConfigError} when every provider key variable is unset or empty

 @example
 ```ts
 const client = runClientFrom({
   env: { TRANSLATION_REPAIR_SYNTHETIC_API_KEY: 'test', },
   transport,
   maxHoldMs: 1_800_000,
 },);
 ```
 */
export function runClientFrom(
  options: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly transport: ModelTransport;
    readonly promptPayloadDir?: string;
    readonly off?: readonly ProviderOff[];
    readonly maxHoldMs: number;
    readonly admitClaudeAttempt?: (attempt: ClaudeAttemptAsk,) => void;
  },
): RunClient {
  return runClientWithFloors({
    ...options,
    unmetFloors: unmetBenchFloors,
  },);
}

/**
 The run client a runner calls over: the process's environment and the live transport, over the seat reader's bench
 floors.

 @param options - longest hold a stated wait may set, which is required: the runner's hard cap, or the probe bound
 where its command has none; and the payload checkpoint, when the runner keeps one

 @returns Ready client, routed across configured providers and counted per seat

 @throws {@link RunConfigError} when every provider key variable is unset or empty

 @example
 ```ts
 const client = createRunClient({ maxHoldMs: 25_200_000, },);
 ```
 */
export function createRunClient(
  options: {
    readonly promptPayloadDir?: string;
    readonly maxHoldMs: number;
  },
): RunClient {
  return createRunClientWithFloors({
    ...options,
    unmetFloors: unmetBenchFloors,
  },);
}

//endregion Run client creation
