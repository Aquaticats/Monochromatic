/** Default recovery respects exhausted-credit exclusions and bounded no-text retries. @module */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { operationFixture, } from './operation-test-fixture.ts';

/** Reported Hyper failure, not a generic HTTP 402 assumption. */
const creditError = `402: {"message":"You're out of credits. Add more at https://hyper.charm.land","type":"billing_error","code":null}`;

await describe({ name: '', children: [
  it({ name: 'non-hedged default call skips all models of an exhausted provider', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: {
      'hyper/a': [{ after: 5, stopReason: 'error', error: creditError, },],
      'hyper/b': [{ after: 1, text: 'must not dispatch', },],
      'other/c': [{ after: 5, text: 'usable', },],
    }, },);
    const result = await fixture.run();
    expect(fixture.dispatched.map(call => call.model),).toEqual(['hyper/a', 'other/c',],);
    expect(result.blockedProviders,).toEqual(['hyper',],);
    expect(result.reviews.map(review => review.text),).toEqual(['usable',],);
    expect(result.attempts[0]?.diagnostic,).toBe(creditError,);
  }, },),
  it({ name: 'credit block is rechecked after another same-provider candidate finishes preparation', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: {
      'hyper/a': [{ after: 15, stopReason: 'error', error: creditError, },],
      'hyper/b': [{ preparationMs: 20, after: 25, text: 'must not dispatch', },],
    }, },);
    const result = await fixture.run({ hedgeDelayMs: 10, },);
    expect(fixture.prepared,).toEqual(['hyper/a', 'hyper/b',],);
    expect(fixture.dispatched.map(call => call.model),).toEqual(['hyper/a',],);
    expect(result.end,).toBe('exhausted',);
    expect(result.attempts[1]?.diagnostic,).toContain('blocked for this call',);
  }, },),
  it({ name: 'credit exclusion preserves a usable result from an already-running same-provider request', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: {
      'p/a': [{ after: 15, stopReason: 'error', error: creditError, },],
      'p/b': [{ after: 10, text: 'already-running review', },],
      'p/c': [{ after: 1, text: 'must not dispatch', },],
    }, },);
    const result = await fixture.run({ hedgeDelayMs: 10, },);
    expect(fixture.dispatched.map(call => call.model),).toEqual(['p/a', 'p/b',],);
    expect(result.blockedProviders,).toEqual(['p',],);
    expect(result.reviews.map(review => review.text),).toEqual(['already-running review',],);
  }, },),
  it({ name: 'successful review text mentioning credits does not block its provider', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: { 'p/a': [{ after: 10, text: 'The message says out of credits.', },], }, },);
    const result = await fixture.run();
    expect(result.blockedProviders,).toHaveLength(0,);
    expect(result.reviews,).toHaveLength(1,);
  }, },),
  it({ name: 'no-text recovery is bounded to one retry before moving to another model', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: {
      'p/a': [{ after: 1, text: '  ', }, { after: 1, text: '', },],
      'q/b': [{ after: 1, text: 'usable', stopReason: 'length', },],
    }, },);
    const result = await fixture.run();
    expect(fixture.dispatched.map(call => call.model),).toEqual(['p/a', 'p/a', 'q/b',],);
    expect(result.reviews[0]?.lengthLimited,).toBe(true,);
    expect(result.usage.totalTokens,).toBe(54,);
  }, },),
  it({ name: 'no no-text retry starts after another model has supplied a usable review', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: { 'p/a': [{ after: 10, text: '', },], 'q/b': [{ after: 2, text: 'usable', },], }, },);
    const result = await fixture.run({ hedgeDelayMs: 1, },);
    expect(fixture.dispatched,).toHaveLength(2,);
    expect(result.reviews.map(review => review.text),).toEqual(['usable',],);
  }, },),
  it({ name: 'already-started preparation receives the collection grace without launching replacements', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: {
      'p/a': [{ after: 15, text: 'usable', },],
      'q/b': [{ preparationMs: 20, after: 30, text: 'second review', },],
      'r/c': [{ after: 1, text: 'must not launch', },],
    }, },);
    const result = await fixture.run({ hedgeDelayMs: 10, },);
    expect(result.reviews.map(review => review.text),).toEqual(['usable', 'second review',],);
    expect(fixture.now(),).toBe(40,);
    expect(fixture.dispatched.map(call => call.model),).toEqual(['p/a', 'q/b',],);
  }, },),
  it({ name: 'already-started preparation cannot dispatch after the collection cutoff', fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: {
      'p/a': [{ after: 15, text: 'usable', },],
      'q/b': [{ preparationMs: 40, after: 50, text: 'too late', },],
    }, },);
    const result = await fixture.run({ hedgeDelayMs: 10, },);
    expect(result.reviews,).toHaveLength(1,);
    expect(result.end,).toBe('collection',);
    expect(fixture.now(),).toBe(45,);
    await fixture.flush();
    expect(fixture.dispatched,).toHaveLength(1,);
  }, },),
  ...(['pending', 'deferred', 'toolUse', 'error', 'aborted',] as const).map(stopReason => it({ name: `does not accept visible text from ${stopReason}`, fn: async (): Promise<void> => {
    const fixture = operationFixture({ plans: { 'p/a': [{ after: 1, stopReason, text: 'not a completed usable review', },], }, },);
    const result = await fixture.run();
    expect(result.reviews,).toHaveLength(0,);
    expect(fixture.dispatched,).toHaveLength(1,);
  }, },),),
], },);
