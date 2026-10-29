/** Bounded operation waits at the built-artifact boundary. @module */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { ADVISOR_CLOCK_BOUNDARY, createAdvisorCancellation, waitForAdvisorEvent, } from '../dist/final/node/index.mjs';

await describe({ name: '', children: [
  it({ name: 'returns provider settlement before the local boundary', fn: async (): Promise<void> => {
    /** Settled provider fixture. */
    const pending = Promise.resolve('review',);
    expect(await waitForAdvisorEvent({ pending: [pending,], untilMs: Date.now() + 1_000, },),).toBe('review',);
  }, },),
  it({ name: 'returns the local boundary even when the provider never settles', fn: async (): Promise<void> => {
    /** Provider deliberately ignores cancellation. */
    const pending = Promise.withResolvers<string>();
    expect(await waitForAdvisorEvent({ pending: [pending.promise,], untilMs: 1, now: (): number => 0, },),).toBe(ADVISOR_CLOCK_BOUNDARY,);
    pending.resolve('late',);
  }, },),
  it({ name: 'honors already-cancelled and later-cancelled callers', fn: async (): Promise<void> => {
    /** Caller-owned cancellation for this fixture. */
    const controller = new AbortController();
    /** Wait has no provider outcome and must be released by cancellation. */
    const pending = waitForAdvisorEvent({ pending: [], untilMs: Date.now() + 1_000, signal: controller.signal, },);
    controller.abort();
    expect(await pending,).toBe(ADVISOR_CLOCK_BOUNDARY,);
    expect(await waitForAdvisorEvent({ pending: [], untilMs: Date.now() + 1_000, signal: controller.signal, },),).toBe(ADVISOR_CLOCK_BOUNDARY,);
  }, },),
  it({ name: 'requests provider cancellation when owned scope ends', fn: async (): Promise<void> => {
    /** Cancellation lifetime under test. */
    const cancellation = createAdvisorCancellation();
    expect(cancellation.controller.signal.aborted,).toBe(false,);
    cancellation[Symbol.dispose]();
    expect(cancellation.controller.signal.aborted,).toBe(true,);
  }, },),
], },);
