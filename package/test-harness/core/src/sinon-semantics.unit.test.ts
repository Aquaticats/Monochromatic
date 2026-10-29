/** Supported method semantics and explicit ordinary-Sinon fallback behavior. @module */
import { EventEmitter, } from 'node:events';
import { describe, expect, it, type TestContext, } from '@monochromatic-dev/test-harness-core';

await describe({
  name: 'context mock semantics',
  children: [
    ...(['call', 'apply', 'bind',] as const).map(mode => it({
      name: `factory ${mode} preserves context routing and lifetime guards`,
      fn: async (): Promise<void> => {
        /** Standard Function helpers must invoke the guarded factory, not its raw target. */
        const target = { method: (): string => 'original', };
        const late: (() => unknown)[] = [];
        await it({ name: 'factory owner', fn: async ({ sinon, }: TestContext,): Promise<void> => {
          /** Read the actual callable namespace member with an explicit invocation receiver. */
          const helper: unknown = Reflect.get(sinon.stub, mode,);
          if ((typeof helper) !== 'function')
            throw new Error(`Missing function helper ${mode}`,);
          const args = mode === 'call' ? [sinon, target, 'method',]
            : mode === 'apply' ? [sinon, [target, 'method',],] : [sinon,];
          const result: unknown = Reflect.apply(helper, sinon.stub, args,);
          if ((mode === 'bind') && ((typeof result) === 'function')) {
            Reflect.apply(result, undefined, [target, 'method',],);
            late.push(() => Reflect.apply(result, undefined, [target, 'method',],),);
          }
          else
            late.push(() => Reflect.apply(helper, sinon.stub, args,),);
          expect(target.method(),).toBeUndefined();
          await it({ name: 'reader without a replacement', fn: async (): Promise<void> => {
            expect(target.method(),).toBe('original',);
          }, },);
        }, },);
        for (const invoke of late)
          expect(invoke,).toThrow('completed',);
        expect(target.method(),).toBe('original',);
      },
    },)),
    it({
      name: 'symbols, matching, call sequences, call-through, and async behavior remain Sinon-owned',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        /** A symbol is a distinct property even when another key has the same description. */
        const key = Symbol('method key reused by Sinon isolation fixture',);
        const other = Symbol('method key reused by Sinon isolation fixture',);
        const target = {
          prefix: 'receiver',
          [key](input: string,): string {
            return `${this.prefix}:${input}`;
          },
          [other](): string {
            return 'unrelated';
          },
        };
        const fake = sinon.stub(target, key,).callThrough();
        fake.withArgs('matched',).returns('match',);
        expect(target[key]('matched',),).toBe('match',);
        expect(target[key]('original',),).toBe('receiver:original',);
        fake.resetBehavior();
        fake.onFirstCall().returns('first',);
        fake.resetHistory();
        expect(target[key]('sequence',),).toBe('first',);
        fake.callsFake((input: string,): string => `custom:${input}`);
        expect(target[key]('custom',),).toBe('custom:custom',);
        expect(target[other](),).toBe('unrelated',);
        fake.resolves('resolved',);
        expect(await target[key]('promise',),).toBe('resolved',);
      },
    },),
    it({
      name: 'inherited, nonconfigurable, and proxy targets preserve ordinary Sinon behavior',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        /** Inherited methods are deliberately outside the new own-property contract. */
        const prototype = { method: (): string => 'inherited', };
        const inherited = { method: (): string => 'own', };
        Object.setPrototypeOf(inherited, prototype,);
        Reflect.deleteProperty(inherited, 'method',);
        sinon.stub(inherited, 'method',).returns('ordinary',);
        expect(inherited.method(),).toBe('ordinary',);
        sinon.restore();
        expect(Object.hasOwn(inherited, 'method',),).toBe(false,);
        /** Writable but nonconfigurable methods can still be replaced by ordinary Sinon. */
        const fixed = { method: (): string => 'fixed', };
        Object.defineProperty(fixed, 'method', { configurable: false, },);
        sinon.stub(fixed, 'method',).returns('ordinary fixed',);
        expect(fixed.method(),).toBe('ordinary fixed',);
        sinon.restore();
        expect(fixed.method(),).toBe('fixed',);
        /** Proxy targets are not virtualized because their descriptor traps have their own effects. */
        const proxy = new Proxy({ method: (): string => 'proxy', }, {},);
        sinon.stub(proxy, 'method',).returns('ordinary proxy',);
        expect(proxy.method(),).toBe('ordinary proxy',);
        sinon.restore();
        expect(proxy.method(),).toBe('proxy',);
      },
    },),
    it({
      name: 'captured fake references retain identity but unbound event callbacks read the emitting context',
      fn: async (): Promise<void> => {
        /** Event registration and emission deliberately belong to different attempts. */
        // oxlint-disable-next-line unicorn/prefer-event-target -- Proves Node EventEmitter read-context semantics; rationale in doc/troubleshooting/sinon-context-owned-stubs.md.
        const emitter = new EventEmitter();
        const target = { method: (): string => 'original', };
        const ready = Promise.withResolvers<() => string>();
        const emitted = Promise.withResolvers<void>();
        await describe({ name: 'event contexts', children: [
          it({ name: 'registering owner', fn: async ({ sinon, }: TestContext,): Promise<void> => {
            sinon.stub(target, 'method',).returns('registered',);
            emitter.once('event', () => {
              expect(target.method(),).toBe('emitted',);
            },);
            ready.resolve(target.method,);
            await emitted.promise;
          }, },),
          it({ name: 'emitting owner', fn: async ({ sinon, }: TestContext,): Promise<void> => {
            const captured = await ready.promise;
            using release = {
              [Symbol.dispose](): void {
                emitted.resolve();
              },
            };
            sinon.stub(target, 'method',).returns('emitted',);
            expect(captured(),).toBe('registered',);
            emitter.emit('event',);
          }, },),
        ], },);
      },
    },),
    it({
      name: 'existing accessor getters and setters retain ordinary Sinon behavior',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const data = { value: 'original', };
        const target = {
          get value(): string {
            return data.value;
          },
          set value(value: string) {
            data.value = value;
          },
        };
        const original = Object.getOwnPropertyDescriptor(target, 'value',);
        const fake = sinon.stub(target, 'value',);
        fake.get(() => 'private getter',);
        fake.set((value: string): void => {
          data.value = value;
        },);
        expect(target.value,).toBe('private getter',);
        target.value = 'changed';
        expect(data.value,).toBe('changed',);
        fake.restore();
        expect(target.value,).toBe('changed',);
        expect(Object.getOwnPropertyDescriptor(target, 'value',),).toEqual(original,);
      },
    },),
    it({
      name: 'deferred contextual getters preserve receiver and setter conversion rejects before mutation',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const target = { label: 'receiver', method: (): string => 'original', };
        const fake = sinon.stub(target, 'method',);
        fake.get(function ownedGetter(this: typeof target,): string {
          return this.label;
        },);
        expect(Reflect.get(target, 'method',),).toBe('receiver',);
        await it({ name: 'reader outside private getter', fn: async (): Promise<void> => {
          expect(target.method(),).toBe('original',);
        }, },);
        fake.value(() => 'private value',);
        expect(target.method(),).toBe('private value',);
        expect(() => fake.set(() => {},),).toThrow('context-owned data-method',);
        expect(target.method(),).toBe('private value',);
      },
    },),
    it({
      name: 'a context-selected constructor spy preserves new invocation and prototype identity',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const target = { Constructor: Date, };
        const spy = sinon.spy(target, 'Constructor',);
        const instance = new target.Constructor(0,);
        expect(instance,).toBeInstanceOf(Date,);
        expect(instance.getTime(),).toBe(0,);
        expect(spy.calledWithNew(),).toBe(true,);
        expect(spy.callCount,).toBe(1,);
      },
    },),
    it({
      name: 'detached function spying does not preflight unrelated static replacements',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        function callable(): string {
          return 'call result';
        }
        callable.method = (): string => 'original static';
        sinon.stub(callable, 'method',).returns('owned static',);
        const before = Object.getOwnPropertyDescriptor(callable, 'method',);
        const spy = sinon.spy(callable,);
        expect(spy(),).toBe('call result',);
        expect(spy.callCount,).toBe(1,);
        expect(callable.method(),).toBe('owned static',);
        expect(Object.getOwnPropertyDescriptor(callable, 'method',),).toEqual(before,);
      },
    },),
    it({
      name: 'numeric method keys use their ordinary JavaScript property spelling',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const target = { 0: (): string => 'original', };
        sinon.stub(target, 0,).returns('numeric',);
        expect(target[0](),).toBe('numeric',);
      },
    },),
    it({
      name: 'createStubInstance returns configurable local fakes guarded after completion',
      fn: async (): Promise<void> => {
        const late: (() => unknown)[] = [];
        await it({ name: 'stub-instance owner', fn: async ({ sinon, }: TestContext,): Promise<void> => {
          const instance = sinon.createStubInstance(Date,);
          const retained = instance.getTime.returns(1,);
          expect(instance.getTime(),).toBe(1,);
          late.push(() => retained.value(() => 2,),);
        }, },);
        for (const mutate of late)
          expect(mutate,).toThrow('completed',);
      },
    },),
    it({
      name: 'whole-object stubs cannot use deferred descriptor methods after completion',
      fn: async (): Promise<void> => {
        /** Ordinary whole-object stubbing still returns the same target object. */
        const target = { method: (): string => 'original', };
        const late: (() => unknown)[] = [];
        await it({ name: 'whole-object owner', fn: async ({ sinon, }: TestContext,): Promise<void> => {
          const stubbed = sinon.stub(target,);
          expect(stubbed,).toBe(target,);
          /** Capture the fake now: the target method itself is restored at completion. */
          const retained = stubbed.method;
          late.push(() => retained.value(() => 'late',));
        }, },);
        for (const mutate of late)
          expect(mutate,).toThrow('completed',);
        expect(target.method(),).toBe('original',);
      },
    },),
  ],
},);
