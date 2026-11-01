/**
 * Per-probe timeout and execution entry point.
 *
 * Wraps {@link runProbeCore} with a 5-minute timeout covering all turns
 * (consistency runs + fix pass). On timeout the probe resolves with score=0
 * rather than throwing, so partial results from other probes can still be collected.
 */
import {
  MS_PER_SECOND,
  SECONDS_PER_MINUTE,
} from '@monochromatic-dev/const-core/ts';

import {
  l,
  tagged,
} from './log.ts';
import { runProbeCore, } from './runner-probe-core.ts';

import type { Probe, } from './probes.ts';
import type { RunnerConfig, } from './runner-config.ts';
import type { ProbeResult, } from './runner-types.ts';

/**
 * Minutes before a probe is considered timed out
 */
const PROBE_TIMEOUT_MINUTES = 5;

/**
 * 5 minutes per probe (all consistency runs + fix pass) -- slower inference is unusable
 */
const PROBE_TIMEOUT_MS = PROBE_TIMEOUT_MINUTES * SECONDS_PER_MINUTE
  * MS_PER_SECOND;

/**
 * Options for {@link createDisposableTimeout}.
 *
 * @example
 * ```ts
 * const opts: CreateDisposableTimeoutOptions = {
 *   callback: () => console.log('fire'),
 *   ms: 1000,
 * };
 * ```
 */
type CreateDisposableTimeoutOptions = {
  /**
   * Function to execute when the timeout fires
   */
  readonly callback: () => void;
  /**
   * Timeout duration in milliseconds
   */
  readonly ms: number;
};

/**
 * Creates a disposable timeout that auto-clears via `Symbol.dispose`.
 * Calling `unref()` on the timer prevents it from keeping the event loop alive.
 *
 * @param callback - function to execute when the timeout fires
 *
 * @param ms - timeout duration in milliseconds
 *
 * @returns disposable handle; timer is cleared when disposed
 *
 * @example
 * ```ts
 * using timer = createDisposableTimeout({ callback: onTick, ms: 1000 });
 * ```
 */
function createDisposableTimeout({
  callback,
  ms,
}: CreateDisposableTimeoutOptions,): Disposable {
  /**
   * Node timer handle retained so `clearTimeout` can run from the dispose hook.
   */
  const id = setTimeout(
    callback,
    ms,
  );
  id.unref();
  return { [Symbol.dispose](): void {
    clearTimeout(id,);
  }, };
}

/**
 * Options for {@link runProbe}.
 *
 * @example
 * ```ts
 * const opts: RunProbeOptions = {
 *   probe: cssMixinProbe,
 *   config: runnerConfig,
 *   timestamp: '2025-09-21T11:13:00Z',
 * };
 * ```
 */
type RunProbeOptions = {
  /**
   * Canary probe to execute
   */
  readonly probe: Probe;
  /**
   * Runner configuration
   */
  readonly config: RunnerConfig;
  /**
   * Authoritative server timestamp for artifact naming
   */
  readonly timestamp: string;
};

/**
 * Runs a single probe with a 5-minute timeout covering all turns (consistency + fix).
 *
 * Uses `AbortController` so the timeout doesn't just cancel the promise; it also
 * cancels in-flight HTTP streams (OpenAI SDK respects `signal`) and kills any live
 * container processes (`execBun` listens for abort). Without cancellation, orphaned
 * coroutines keep the Bun event loop alive well past the timeout.
 *
 * On timeout the probe resolves with score=0 and `timedOut: true` rather than throwing,
 * so partial results from other probes can still be collected and written to history.
 *
 * @param probe - canary probe to execute
 *
 * @param config - runner configuration
 *
 * @param timestamp - authoritative server timestamp for artifact naming
 *
 * @returns scored result; on timeout, a zero-score result with `timedOut: true`
 *
 * @example
 * ```ts
 * const result = await runProbe({ probe, config, timestamp });
 * result.meanScore; // 0 if timed out
 * ```
 */
export async function runProbe({
  probe,
  config,
  timestamp,
}: RunProbeOptions,): Promise<ProbeResult> {
  /**
   * Signals cancellation to in-flight HTTP streams and container processes when the timeout fires.
   */
  const controller = new AbortController();
  /**
   * Live probe execution; settled by either success, failure, or `controller.abort()`.
   */
  const corePromise = runProbeCore({
    probe,
    config,
    timestamp,
    signal: controller.signal,
  },);
  /**
   * Zero-score sentinel returned when the timeout fires.
   *
   * Score 0 is recorded in history so the overall model score reflects the failure
   * without discarding other probe results from the same run.
   */
  const timedOutResult: ProbeResult = {
    name: probe.name,
    category: probe.category,
    scores: [],
    meanScore: 0,
    consistent: true,
    timedOut: true,
  };

  /**
   * Wraps `corePromise` to absorb abort-triggered rejections.
   * After timeout wins the race, `controller.abort()` causes `corePromise` to reject
   * with no observer. This wrapper catches that expected rejection and returns
   * the timeout sentinel so the promise settles cleanly.
   *
   * @returns probe result, or the timeout sentinel if the abort was expected
   */
  async function observedCore(): Promise<ProbeResult> {
    try {
      return await corePromise;
    }
    catch (error) {
      if (controller.signal
        .aborted)
        return timedOutResult;
      throw error;
    }
  }

  /**
   * Promise resolved from the timeout callback; raced against {@link observedCore}.
   */
  const {
    promise: timeoutPromise,
    resolve: resolveTimeout,
  } = Promise.withResolvers<
    ProbeResult
  >();

  /**
   * Disposable timeout handle; auto-clears via `Symbol.dispose` when the function returns.
   */
  using _timer = createDisposableTimeout({
    callback: function onTimeout(): void {
      controller.abort();
      /**
       * Probe-specific logger for timeout message.
       */
      const rl = tagged({
        tag: probe.name,
        l: tagged({
          tag: config.label,
          l,
        },),
      },);
      rl.error(
        `timed out after ${String(PROBE_TIMEOUT_MINUTES,)} minutes`,
      );
      resolveTimeout(timedOutResult,);
    },
    ms: PROBE_TIMEOUT_MS,
  },);

  return await Promise.race([
    observedCore(),
    timeoutPromise,
  ],);
}
