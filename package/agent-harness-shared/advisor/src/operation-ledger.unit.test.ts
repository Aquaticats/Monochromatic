/** Built-artifact checks for ledger immutability and available usage. @module */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { createAdvisorOperationLedger, aggregateAdvisorUsage, copyAdvisorUsage, type AdvisorAttemptRecord, } from '../dist/final/node/index.mjs';

/** Usage contains subset counters so accidental double-counting is observable. */
const usage = {
  input: 7, output: 5, reasoning: 3, cacheRead: 2, cacheWrite: 4, cacheWrite1h: 1, totalTokens: 18,
  cost: { input: 1, output: 2, cacheRead: 3, cacheWrite: 4, total: 10, },
};
/** Attempt fixture. */
const attempt: AdvisorAttemptRecord = {
  id: 1, model: 'fixture/one', provider: 'fixture', attempt: 1, state: 'running', startedAtMs: 0,
  dispatchedAtMs: 1, contextChars: 20, estimatedInputTokens: 5, truncated: false, usageIncomplete: true,
};

await describe({ name: '', children: [
  it({ name: 'aggregates optional subsets without adding them to total tokens', fn: async (): Promise<void> => {
    /** Combined usage for two actual attempts. */
    const total = aggregateAdvisorUsage([usage, usage,],);
    expect(total.totalTokens,).toBe(36,);
    expect(total.output,).toBe(10,);
    expect(total.reasoning,).toBe(6,);
    expect(total.cacheWrite,).toBe(8,);
    expect(total.cacheWrite1h,).toBe(2,);
    expect(total.cost.total,).toBe(20,);
    expect(aggregateAdvisorUsage([],).reasoning,).toBeUndefined();
    expect(aggregateAdvisorUsage([],).cacheWrite1h,).toBeUndefined();
    /** Provider does not report optional subsets. */
    const withoutSubsets = { input: 1, output: 1, cacheRead: 0, cacheWrite: 0, totalTokens: 2, cost: usage.cost, };
    expect(aggregateAdvisorUsage([withoutSubsets, withoutSubsets,],).reasoning,).toBeUndefined();
  }, },),
  it({ name: 'copies provider counters including nested cost', fn: async (): Promise<void> => {
    /** Provider-owned mutable data. */
    const mutable = { ...usage, cost: { ...usage.cost, }, };
    /** Snapshot retained before provider writes another event. */
    const copy = copyAdvisorUsage(mutable,);
    mutable.input = 100;
    mutable.cost.total = 100;
    expect(copy.input,).toBe(7,);
    expect(copy.cost.total,).toBe(10,);
  }, },),
  it({ name: 'replaces usage snapshots and seals late updates after local cancellation', fn: async (): Promise<void> => {
    /** Operation under test. */
    const ledger = createAdvisorOperationLedger({ startedAtMs: 0, deadlineAtMs: 100, },);
    ledger.record(attempt,);
    /** Earlier progress remains unchanged after later observations. */
    const earlier = ledger.snapshot();
    ledger.record({ ...attempt, usage, },);
    ledger.record({ ...attempt, usage: { ...usage, input: 8, totalTokens: 19, }, },);
    expect(earlier.attempts[0]?.usage,).toBeUndefined();
    expect(ledger.snapshot().usage.input,).toBe(8,);
    ledger.blockProvider('fixture',);
    ledger.finish({ end: 'collection', now: 50, },);
    ledger.record({ ...attempt, state: 'succeeded', },);
    ledger.blockProvider('late-provider',);
    ledger.collect({ review: { attemptId: 1, model: 'fixture/one', text: 'late', lengthLimited: false, }, collectionEndsAtMs: 80, },);
    ledger.finish({ end: 'complete', now: 60, },);
    expect(ledger.attempt(1,).state,).toBe('cancelled',);
    expect(ledger.snapshot().end,).toBe('collection',);
    expect(ledger.snapshot().blockedProviders,).toEqual(['fixture',],);
    expect(ledger.snapshot().reviews,).toHaveLength(0,);
    expect(ledger.snapshot().usageIncomplete,).toBe(true,);
    expect(() => ledger.attempt(2,),).toThrow('unknown attempt',);
  }, },),
  it({ name: 'retains all reviews in attempt order and caps collection at the original deadline', fn: async (): Promise<void> => {
    /** Operation under test. */
    const ledger = createAdvisorOperationLedger({ startedAtMs: 0, deadlineAtMs: 100, },);
    ledger.record({ ...attempt, state: 'succeeded', usage, usageIncomplete: false, },);
    ledger.record({ ...attempt, id: 2, model: 'other/two', state: 'succeeded', usage, usageIncomplete: false, },);
    ledger.collect({ review: { attemptId: 2, model: 'other/two', text: 'second', lengthLimited: true, }, collectionEndsAtMs: 120, },);
    ledger.collect({ review: { attemptId: 1, model: 'fixture/one', text: 'first', lengthLimited: false, }, collectionEndsAtMs: 200, },);
    ledger.finish({ end: 'complete', now: 80, },);
    expect(ledger.snapshot().reviews.map(review => review.model,),).toEqual(['fixture/one', 'other/two',],);
    expect(ledger.snapshot().collectionEndsAtMs,).toBe(100,);
    expect(ledger.snapshot().usageIncomplete,).toBe(false,);
    expect(ledger.attempt(1,).state,).toBe('succeeded',);
  }, },),
], },);
