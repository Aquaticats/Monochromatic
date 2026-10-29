/** Deterministic scheduling and bounded-collection regression coverage. @module */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { operationFixture, } from './operation-test-fixture.ts';

await describe({ name: '', children: [
  it({ name: 'disabled overlap returns one successful review without grace or an alternate', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: { 'p/a': [{ after: 80, text: 'first', },], 'q/b': [{ after: 10, text: 'unused', },], }, },);
    const result = await fixture.run();
    expect(result.reviews.map(review => review.text),).toEqual(['first',],);
    expect(fixture.dispatched.map(call => call.model),).toEqual(['p/a',],);
    expect(fixture.now(),).toBe(80,);
  }, },),
  it({ name: 'authentication stalls do not postpone the configured hedge', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: {
      'p/a': [{ preparationMs: 100, after: 150, text: 'late initial', },],
      'q/b': [{ after: 10, text: 'usable alternate', },],
    }, },);
    const result = await fixture.run({ hedgeDelayMs: 10, },);
    expect(fixture.now(),).toBe(50,);
    expect(fixture.dispatched.map(call => call.model),).toEqual(['q/b',],);
    expect(result.reviews.map(review => review.text),).toEqual(['usable alternate',],);
    await fixture.flush();
    expect(fixture.dispatched,).toHaveLength(1,);
  }, },),
  it({ name: 'enabled overlap tolerates an unavailable alternate without a polling loop', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: { 'p/a': [{ after: 80, text: 'only reviewer', },], }, },);
    const result = await fixture.run({ hedgeDelayMs: 10, },);
    expect(fixture.dispatched,).toHaveLength(1,);
    expect(fixture.now(),).toBe(80,);
    expect(result.reviews,).toHaveLength(1,);
  }, },),
  it({ name: 'an independent provider abort does not discard the other completed review', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: {
      'p/a': [{ after: 20, stopReason: 'aborted', error: 'provider abort', },],
      'q/b': [{ after: 2, text: 'usable', },],
    }, },);
    const result = await fixture.run({ hedgeDelayMs: 1, },);
    expect(result.end,).toBe('complete',);
    expect(result.reviews.map(review => review.text),).toEqual(['usable',],);
  }, },),
  it({ name: 'no-text retry inherits the original operation deadline', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: {
      'p/a': [{ after: 150, text: '', }, { after: 100, text: 'too late', },],
      'q/b': [{ after: 1, text: 'must not launch after deadline', },],
    }, },);
    const result = await fixture.run();
    expect(result.end,).toBe('deadline',);
    expect(fixture.now(),).toBe(200,);
    expect(fixture.dispatched.map(call => call.model),).toEqual(['p/a', 'p/a',],);
    expect(result.reviews,).toHaveLength(0,);
    await fixture.flush();
  }, },),
  it({ name: 'completion before the hedge delay does not launch another request', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: { 'p/a': [{ after: 5, text: 'first', },], 'q/b': [{ after: 5, text: 'unused', },], }, },);
    await fixture.run({ hedgeDelayMs: 10, },);
    expect(fixture.dispatched,).toHaveLength(1,);
    expect(fixture.now(),).toBe(5,);
  }, },),
  it({ name: 'hedge finishes first but both usable reviews return together in dispatch order', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: { 'p/a': [{ after: 40, text: 'initial', },], 'q/b': [{ after: 10, text: 'alternate', },], }, },);
    const result = await fixture.run({ hedgeDelayMs: 10, },);
    expect(fixture.dispatched.map(call => call.at),).toEqual([0, 10,],);
    expect(result.reviews.map(review => review.text),).toEqual(['initial', 'alternate',],);
    expect(result.usage.totalTokens,).toBe(36,);
    expect(result.usage.reasoning,).toBe(6,);
    expect(fixture.now(),).toBe(40,);
  }, },),
  ...[
    { name: 'initial reviewer is the straggler', first: 100, second: 10, text: 'alternate', },
    { name: 'alternate reviewer is the straggler', first: 20, second: 100, text: 'initial', },
  ].map(test => it({ name: test.name, fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: { 'p/a': [{ after: test.first, text: 'initial', },], 'q/b': [{ after: test.second, text: 'alternate', },], }, },);
    const result = await fixture.run({ hedgeDelayMs: 10, },);
    expect(result.end,).toBe('collection',);
    expect(result.reviews.map(review => review.text),).toEqual([test.text,],);
    expect(fixture.now(),).toBe(50,);
    expect(result.attempts.filter(attempt => attempt.state === 'cancelled'),).toHaveLength(1,);
    expect(fixture.dispatched.every(call => call.signal.aborted),).toBe(true,);
    expect(result.usageIncomplete,).toBe(true,);
    const count = fixture.progress.length;
    await fixture.flush();
    expect(fixture.progress,).toHaveLength(count,);
    expect(result.reviews,).toHaveLength(1,);
  }, },),),
  it({ name: 'original deadline truncates grace and retains already usable reviews', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: { 'p/a': [{ after: 90, text: 'initial', },], 'q/b': [{ after: 150, text: 'late', },], }, },);
    const result = await fixture.run({ hedgeDelayMs: 10, deadlineAtMs: 100, },);
    expect(result.end,).toBe('deadline',);
    expect(result.reviews,).toHaveLength(1,);
    expect(fixture.now(),).toBe(100,);
    await fixture.flush();
  }, },),
  it({ name: 'all failures exhaust candidates without exceeding two running providers', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: {
      'p/a': [{ after: 20, stopReason: 'error', error: 'network error', },],
      'q/b': [{ after: 20, stopReason: 'aborted', error: 'provider abort', },],
      'r/c': [{ after: 10, stopReason: 'toolUse', text: 'not a review', },],
    }, },);
    const result = await fixture.run({ hedgeDelayMs: 10, },);
    expect(result.end,).toBe('exhausted',);
    expect(result.reviews,).toHaveLength(0,);
    expect(fixture.dispatched,).toHaveLength(3,);
    expect(fixture.progress.every(snapshot => snapshot.attempts.filter(attempt => attempt.state === 'running').length <= 2),).toBe(true,);
  }, },),
  it({ name: 'first usable result prevents replacing another failed reviewer', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: {
      'p/a': [{ after: 10, stopReason: 'error', error: 'network error', },],
      'q/b': [{ after: 2, text: 'usable', },], 'r/c': [{ after: 1, text: 'unused', },],
    }, },);
    const result = await fixture.run({ hedgeDelayMs: 1, },);
    expect(result.reviews,).toHaveLength(1,);
    expect(fixture.dispatched.map(call => call.model),).toEqual(['p/a', 'q/b',],);
    expect(fixture.now(),).toBe(10,);
  }, },),
  ...[5, 25,].map(at => it({ name: `caller cancellation at ${at} stops before or after partial success`, fn: async (): Promise<void> => {
    const controller = new AbortController();
    const fixture = operationFixture({ plans: { 'p/a': [{ after: 100, text: 'late', },], 'q/b': [{ after: 10, text: 'usable', },], }, events: [{ at, run: (): void => controller.abort(), },], },);
    const result = await fixture.run({ hedgeDelayMs: 10, signal: controller.signal, },);
    expect(result.end,).toBe('caller',);
    expect(fixture.now(),).toBe(at,);
    expect(fixture.dispatched.every(call => call.signal.aborted),).toBe(true,);
    await fixture.flush();
  }, },),),
  it({ name: 'preparation cannot dispatch after the deadline', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: { 'p/a': [{ preparationMs: 100, after: 150, text: 'late', },], }, },);
    const result = await fixture.run({ deadlineAtMs: 50, },);
    expect(result.end,).toBe('deadline',);
    expect(fixture.dispatched,).toHaveLength(0,);
    await fixture.flush();
    expect(fixture.dispatched,).toHaveLength(0,);
  }, },),
], },);
