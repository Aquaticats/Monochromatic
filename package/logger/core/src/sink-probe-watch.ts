/**
 Verifies every sink backend under the verify time limit, keeping a slow probe's
 answer instead of retiring its sink. @module

 A verify deadline says how long startup waits, never what a sink is worth: a
 backend that a busy host delayed answers late, and whatever it answers decides
 the sink from then on. Keeping that rule here leaves the logger assembly with
 sink state and record delivery, and this module with probes and their verdicts.
 */

import {
  reportVerificationDeadline,
  reportVerificationFailure,
} from './sink-verification-reports.ts';
import { settleWithinHostTimeout, } from './with-host-timeout.ts';

import type { Sink, } from './types.ts';

/**
 What one probe's verdict does to its sink, supplied by the logger that owns the
 sink entries and the records kept for them.
 */
type VerdictPorts = {
  /**
   Records one sink's verdict and replays the records kept for it.
   */
  readonly conclude: (args: {
    readonly entryIndex: number;
    readonly available: boolean;
  }) => void;
  /**
   Records held for sinks that have no verdict yet.
   */
  readonly keptCount: () => number;
  /**
   Retires a sink whose verifier threw, rejected, or could not start.
   */
  readonly markUnavailable: (args: { readonly entryIndex: number; }) => void;
};

/**
 Runs one sink's verifier so a synchronous throw becomes a rejection, which the
 logger reports like any other failed probe instead of losing it.

 @param sink - Sink whose backend is about to be verified.

 @returns The sink's own verification promise.

 @example
 ```ts
 const probe = startProbe({ sink: createConsoleSink() });
 ```
 */
async function startProbe({ sink, }: { readonly sink: Sink; },): Promise<boolean> {
  return await sink.verify();
}

/**
 Applies a probe's verdict whenever it arrives after the deadline, so a sink
 that was merely slow still receives the records kept for it.

 @param entryIndex - Sink entry index whose probe is still running.

 @param probe - The sink's own verification promise.

 @param ports - Sink-state callbacks of the logger that started the probe.
 */
async function applyLateVerdict(
  {
    entryIndex,
    probe,
    ports,
    sink,
  }: {
    readonly entryIndex: number;
    readonly ports: VerdictPorts;
    readonly probe: Promise<boolean>;
    readonly sink: Sink;
  },
): Promise<void> {
  try {
    /**
     Verdict the probe reported after the deadline elapsed.
     */
    ports.conclude({
      available: await probe,
      entryIndex,
    },);
  }
  catch (error: unknown) {
    reportVerificationFailure({
      entryIndex,
      error,
      kept: ports.keptCount(),
      sink,
    },);
    ports.markUnavailable({ entryIndex, },);
  }
}

/**
 Runs one sink's verification, applying its verdict through `ports`. A verify
 that rejects or throws retires its sink. A verify that runs past `ms` retires
 nothing: startup stops waiting for it and its answer is applied whenever it
 arrives. An answer that never arrives leaves the sink undecided until the
 process ends, and its records stay bounded by the logger's buffer cap.

 The deadline arrives as the wait's outcome, so a verifier that rejects with
 text resembling a timeout is still retired as the failing backend it is.

 @param entryIndex - Sink entry index to verify.

 @param ms - Milliseconds startup waits before it stops waiting for this probe.

 @param ports - Sink-state callbacks of the logger that started the probe.

 @param sink - Sink whose backend to verify.

 @example
 ```ts
 await watchOneProbe({ entryIndex: 0, ms: 5_000, ports, sink });
 ```
 */
async function watchOneProbe(
  {
    entryIndex,
    ms,
    ports,
    sink,
  }: {
    readonly entryIndex: number;
    readonly ms: number;
    readonly ports: VerdictPorts;
    readonly sink: Sink;
  },
): Promise<void> {
  /**
   The sink's own probe, started once and observed even after the deadline.
   */
  const probe = startProbe({ sink, },);

  try {
    /**
     Which side of the wait finished first: the probe, or the deadline.
     */
    const outcome = await settleWithinHostTimeout({
      ms,
      promise: probe,
    },);

    if (outcome === 'expired') {
      reportVerificationDeadline({
        entryIndex,
        sink,
        timeoutMs: ms,
      },);
      void applyLateVerdict({
        entryIndex,
        ports,
        probe,
        sink,
      },);
      return;
    }

    ports.conclude({
      available: await probe,
      entryIndex,
    },);
  }
  catch (error: unknown) {
    reportVerificationFailure({
      entryIndex,
      error,
      kept: ports.keptCount(),
      sink,
    },);
    ports.markUnavailable({ entryIndex, },);
  }
}

/**
 Verifies every sink backend concurrently and resolves once each probe answered
 or its deadline elapsed. A probe still running past its deadline keeps running:
 its answer still decides its sink, so nothing a slow host delays is lost.

 @param ms - Milliseconds startup waits for each probe.

 @param ports - Sink-state callbacks of the owning logger.

 @param sinks - Sinks to verify, in startup order.

 @example
 ```ts
 await watchSinkProbes({ ms: 5_000, ports, sinks });
 ```
 */
export async function watchSinkProbes(
  {
    ms,
    ports,
    sinks,
  }: {
    readonly ms: number;
    readonly ports: VerdictPorts;
    readonly sinks: readonly Sink[];
  },
): Promise<void> {
  await Promise.all(sinks.map(function watchSink(
    sink,
    entryIndex,
  ) {
    return watchOneProbe({
      entryIndex,
      ms,
      ports,
      sink,
    },);
  },),);
}
