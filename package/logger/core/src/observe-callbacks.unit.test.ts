import { caught, describe, expect, it, type TestContext, } from '@monochromatic-dev/test-harness-core/ts';
import { observeLoggerCallbacks, type Logger, } from '../dist/final/node/index.mjs';

const levels = ['debug', 'error', 'fatal', 'info', 'trace', 'warn'] as const;

// A suspended-start generator forwards arbitrary thrown values without Sinon's undefined exception sentinel.
function* faultCarrier(): Generator<undefined> {
  yield;
}

function fixture(ctx: TestContext) {
  const spies = {
    debug: ctx.sinon.stub(),
    error: ctx.sinon.stub(),
    fatal: ctx.sinon.stub(),
    flush: ctx.sinon.stub().resolves(),
    info: ctx.sinon.stub(),
    trace: ctx.sinon.stub(),
    warn: ctx.sinon.stub(),
  };
  const logger: Logger = spies;
  return { logger, spies };
}

await describe({ name: observeLoggerCallbacks.name, children: [
  it({ name: 'forwards every level and explicit flush once in caller order with the original receiver', fn: async ctx => {
    const f = fixture(ctx);
    const observed = observeLoggerCallbacks(f.logger);
    expect(observed.snapshot()).toEqual([]);
    for (const level of levels) {
      observed.logger[level](`[existing] ${level}`);
      expect(f.spies[level].callCount).toBe(1);
      expect(f.spies[level].firstCall.args).toEqual([`[existing] ${level}`]);
      expect(f.spies[level].firstCall.thisValue).toBe(f.logger);
    }
    expect(f.spies.flush.callCount).toBe(0);
    await observed.logger.flush();
    expect(f.spies.flush.callCount).toBe(1);
    expect(f.spies.flush.firstCall.args).toEqual([]);
    expect(f.spies.flush.firstCall.thisValue).toBe(f.logger);
    ctx.sinon.assert.callOrder(f.spies.debug, f.spies.error, f.spies.fatal, f.spies.info, f.spies.trace, f.spies.warn, f.spies.flush);
    expect(observed.snapshot()).toEqual([]);
  } }),
  ...levels.map(level => it({ name: `contains ${level} throws without inspecting the thrown value or dropping later messages`, fn: async ctx => {
    const f = fixture(ctx);
    const trapped = ctx.sinon.stub().throws(new Error('inspected private q7z9k2'));
    const fault = new Error('private q7z9k2');
    Object.defineProperties(fault, {
      name: { get: trapped },
      message: { get: trapped },
      cause: { get: trapped },
    });
    f.spies[level].onFirstCall().throws(function foreignFailure() {
      return fault;
    });
    const observed = observeLoggerCallbacks(f.logger);
    expect(() => observed.logger[level]('first')).not.toThrow();
    observed.logger[level]('distinct later message');
    expect(f.spies[level].callCount).toBe(2);
    expect(f.spies[level].secondCall.args).toEqual(['distinct later message']);
    expect(trapped.callCount).toBe(0);
    expect(observed.snapshot()).toEqual([level]);
    expect(
      JSON.stringify(observed.snapshot()),
    ).not.toContain('q7z9k2');
  } })),
  ...(['string', 'symbol', 'undefined', 'null', 'false', 'revoked-error'] as const).map(kind => it({ name: `contains a foreign ${kind} throw without requiring an Error brand`, fn: async ctx => {
    const f = fixture(ctx);
    const revoked = Proxy.revocable(new Error('private q7z9k2'), {});
    revoked.revoke();
    const fault: unknown = kind === 'string' ? 'private q7z9k2' : kind === 'symbol' ? Symbol('private callback fault q7z9k2')
      : kind === 'undefined' ? undefined : kind === 'null' ? null : kind === 'false' ? false : revoked.proxy;
    expect(caught(function nativeFaultControl() {
      return faultCarrier().throw(fault);
    })).toBe(fault);
    const entered = ctx.sinon.spy(function enteredCallback() {});
    Object.defineProperty(f.logger, 'warn', {
      value: function foreignFailure(): void {
        entered();
        const carrier = faultCarrier();
        carrier.throw(fault);
      },
    });
    const observed = observeLoggerCallbacks(f.logger);
    expect(() => observed.logger.warn('first')).not.toThrow();
    expect(observed.snapshot()).toEqual(['warn']);
    expect(entered.callCount).toBe(1);
  } })),
  it({ name: 'does not read unused callback getters and contains a getter when its callback is requested', fn: async ctx => {
    const f = fixture(ctx);
    const getter = ctx.sinon.stub().throws(new Error('private getter q7z9k2'));
    Object.defineProperty(f.logger, 'fatal', { get: getter });
    expect(() => observeLoggerCallbacks(f.logger)).not.toThrow();
    const observed = observeLoggerCallbacks(f.logger);
    expect(getter.callCount).toBe(0);
    observed.logger.info('normal');
    expect(observed.snapshot()).toEqual([]);
    expect(getter.callCount).toBe(0);
    expect(() => observed.logger.fatal('requested')).not.toThrow();
    expect(getter.callCount).toBe(1);
    expect(observed.snapshot()).toEqual(['fatal']);
  } }),
  it({ name: 'looks up each requested level again instead of retaining the first method', fn: async ctx => {
    const f = fixture(ctx);
    const initial = f.spies.warn;
    const replacement = ctx.sinon.stub();
    const getter = ctx.sinon.stub();
    getter.onFirstCall().returns(initial);
    getter.onSecondCall().returns(replacement);
    Object.defineProperty(f.logger, 'warn', { get: getter });
    const observed = observeLoggerCallbacks(f.logger);
    expect(getter.callCount).toBe(0);
    observed.logger.warn('first');
    observed.logger.warn('second');
    expect(getter.callCount).toBe(2);
    expect(initial.callCount).toBe(1);
    expect(getter.firstCall.thisValue).toBe(f.logger);
    expect(getter.secondCall.thisValue).toBe(f.logger);
    expect(replacement.callCount).toBe(1);
    expect(replacement.firstCall.args).toEqual(['second']);
    expect(replacement.firstCall.thisValue).toBe(f.logger);
    expect(observed.snapshot()).toEqual([]);
  } }),
  it({ name: 'looks up explicit flush again and observes the replacement callback', fn: async ctx => {
    const f = fixture(ctx);
    const initial = f.spies.flush;
    const replacement = ctx.sinon.stub().rejects(Symbol('replacement Logger.flush callback rejected'));
    const getter = ctx.sinon.stub();
    getter.onFirstCall().returns(initial);
    getter.onSecondCall().returns(replacement);
    Object.defineProperty(f.logger, 'flush', { get: getter });
    const observed = observeLoggerCallbacks(f.logger);
    expect(getter.callCount).toBe(0);
    await observed.logger.flush();
    expect(observed.snapshot()).toEqual([]);
    await observed.logger.flush();
    expect(getter.callCount).toBe(2);
    expect(initial.callCount).toBe(1);
    expect(getter.firstCall.thisValue).toBe(f.logger);
    expect(getter.secondCall.thisValue).toBe(f.logger);
    expect(replacement.callCount).toBe(1);
    expect(replacement.firstCall.thisValue).toBe(f.logger);
    expect(observed.snapshot()).toEqual(['flush']);
  } }),
  ...(['undefined', 'revoked-error'] as const).map(kind => it({ name: `contains native flush rejection with ${kind} without inspecting the value`, fn: async ctx => {
    const f = fixture(ctx);
    const revoked = Proxy.revocable(new Error('private flush q7z9k2'), {});
    revoked.revoke();
    const fault = kind === 'undefined' ? undefined : revoked.proxy;
    async function rejectFlush(): Promise<void> {
      await Promise.resolve();
      const carrier = faultCarrier();
      carrier.throw(fault);
    }
    const [control] = await Promise.allSettled([rejectFlush()]);
    expect(control?.status).toBe('rejected');
    if (control?.status !== 'rejected') throw new Error('Expected the native flush rejection control');
    expect(Object.is(control.reason, fault)).toBe(true);
    Object.defineProperty(f.logger, 'flush', { value: rejectFlush });
    const observed = observeLoggerCallbacks(f.logger);
    const [outcome] = await Promise.allSettled([observed.logger.flush()]);
    expect(outcome?.status).toBe('fulfilled');
    expect(observed.snapshot()).toEqual(['flush']);
  } })),
  it({ name: 'contains a revoked callable proxy without inspecting it', fn: async ctx => {
    const f = fixture(ctx);
    const callable = Proxy.revocable(f.spies.warn, {});
    callable.revoke();
    Object.defineProperty(f.logger, 'warn', { value: callable.proxy });
    const observed = observeLoggerCallbacks(f.logger);
    expect(() => observed.logger.warn('requested')).not.toThrow();
    expect(observed.snapshot()).toEqual(['warn']);
  } }),
  it({ name: 'does not inspect a revoked logger until a callback is requested', fn: async ctx => {
    const f = fixture(ctx);
    const revoked = Proxy.revocable(f.logger, {});
    revoked.revoke();
    const observed = observeLoggerCallbacks(revoked.proxy);
    expect(observed.snapshot()).toEqual([]);
    expect(() => observed.logger.error('requested')).not.toThrow();
    await observed.logger.flush();
    expect(observed.snapshot()).toEqual(['error', 'flush']);
  } }),
  ...(['getter', 'invocation', 'rejection', 'then-getter', 'then-rejection'] as const).map(kind => it({ name: `contains explicit flush ${kind} failure and resolves normally`, fn: async ctx => {
    const f = fixture(ctx);
    const fault = Symbol('private flush callback fault q7z9k2');
    const thrower = ctx.sinon.stub().throws(function foreignFailure() {
      return fault;
    });
    if (kind === 'getter') Object.defineProperty(f.logger, 'flush', { get: thrower });
    else if (kind === 'invocation') f.spies.flush.callsFake(thrower);
    else if (kind === 'rejection') f.spies.flush.rejects(fault);
    else {
      const rejectedThen = ctx.sinon.stub().callsArgWith(1, fault);
      // Fault an existing Promise's protocol lookup instead of defining a thenable application record.
      const foreign = new Proxy(Promise.resolve(), {
        get: function lookup(target, key, receiver): unknown {
          if (key === 'then') return kind === 'then-getter' ? thrower() : rejectedThen;
          return Reflect.get(target, key, receiver);
        },
      });
      f.spies.flush.returns(foreign);
    }
    const observed = observeLoggerCallbacks(f.logger);
    expect(observed.snapshot()).toEqual([]);
    await observed.logger.flush();
    expect(observed.snapshot()).toEqual(['flush']);
    expect(
      JSON.stringify(observed.snapshot()),
    ).not.toContain('q7z9k2');
    if (kind === 'getter') expect(thrower.callCount).toBe(1);
    else expect(f.spies.flush.callCount).toBe(1);
  } })),
  it({ name: 'contains an ordinary synchronous failure at the request boundary', fn: async ctx => {
    const f = fixture(ctx);
    f.spies.warn.throws(new Error('expected callback failure'));
    const observed = observeLoggerCallbacks(f.logger);
    expect(() => observed.logger.warn('requested')).not.toThrow();
    expect(observed.snapshot()).toEqual(['warn']);
  } }),
  it({ name: 'ignores a level callback return value without reading its then protocol', fn: async ctx => {
    const f = fixture(ctx);
    const getter = ctx.sinon.stub().throws(new Error('unexpected then lookup'));
    const foreign = new Proxy(Promise.resolve(), {
      get: function lookup(target, key, receiver): unknown {
        if (key === 'then') return getter();
        return Reflect.get(target, key, receiver);
      },
    });
    f.spies.info.returns(foreign);
    const observed = observeLoggerCallbacks(f.logger);
    observed.logger.info('synchronous contract');
    expect(f.spies.info.callCount).toBe(1);
    expect(getter.callCount).toBe(0);
    expect(observed.snapshot()).toEqual([]);
    expect(() => Reflect.get(foreign, 'then')).toThrow();
    expect(getter.callCount).toBe(1);
  } }),
  it({ name: 'deduplicates failures in canonical frozen snapshots detached from later activity', fn: async ctx => {
    const f = fixture(ctx);
    for (const level of levels) f.spies[level].throws(new Error('private q7z9k2'));
    f.spies.flush.rejects(new Error('private q7z9k2'));
    const observed = observeLoggerCallbacks(f.logger);
    const empty = observed.snapshot();
    observed.logger.warn('first');
    observed.logger.warn('second distinct request');
    const one = observed.snapshot();
    for (const level of levels.toReversed()) observed.logger[level]('distinct level');
    await observed.logger.flush();
    const all = observed.snapshot();
    expect(empty).toEqual([]);
    expect(one).toEqual(['warn']);
    expect(all).toEqual(['debug', 'error', 'fatal', 'flush', 'info', 'trace', 'warn']);
    expect(Object.isFrozen(empty)).toBe(true);
    expect(Object.isFrozen(one)).toBe(true);
    expect(Object.isFrozen(all)).toBe(true);
    expect(Reflect.set(all, '0', 'private')).toBe(false);
    expect(observed.snapshot()).not.toBe(all);
    expect(Object.isFrozen(observed.logger)).toBe(true);
    expect(Object.isFrozen(observed)).toBe(true);
  } }),
] });
