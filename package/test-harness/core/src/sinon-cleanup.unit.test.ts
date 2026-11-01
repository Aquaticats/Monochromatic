/** Completed attempts and restoration failures through the public artifact. @module */
import { describe, expect, it, type TestContext, } from '@monochromatic-dev/test-harness-core';

await describe({
  name: 'sandbox cleanup boundaries',
  children: [
    it({
      name: 'a timeout tail cannot install new mocks or enter a later owners fake',
      fn: async (): Promise<void> => {
        /** Work finishes only after another attempt owns the same method. */
        const release = Promise.withResolvers<void>();
        /** Explicitly observe the original body after its timeout. */
        const finished = Promise.withResolvers<void>();
        /** The fixture is local even though timed-out work retains it. */
        const target = { method: (): string => 'original', };
        await it({
          name: 'expired owner',
          timeout: 1,
          fails: true,
          fn: async ({ sinon, }: TestContext,): Promise<void> => {
            sinon.stub(target, 'method',).returns('expired',);
            await release.promise;
            try {
              expect(target.method(),).toBe('original',);
              expect(() => sinon.stub(target, 'method',),).toThrow('completed',);
              expect(() => sinon.spy(target, 'method',),).toThrow('completed',);
              expect(() => sinon.fake.returns('late',),).toThrow('completed',);
              expect(() => sinon.replace(target, 'method', () => 'late',),).toThrow('completed',);
              finished.resolve();
            }
            catch (error) {
              // A timeout's original promise already has a rejection observer; explicitly report tail assertions.
              finished.reject(error);
            }
          },
        },);
        await it({
          name: 'later owner',
          fn: async ({ sinon, }: TestContext,): Promise<void> => {
            /** Expired work must not contribute calls to the later fake. */
            const fake = sinon.stub(target, 'method',).returns('later',);
            release.resolve();
            await finished.promise;
            expect(fake.callCount,).toBe(0,);
            expect(target.method(),).toBe('later',);
          },
        },);
        expect(target.method(),).toBe('original',);
      },
    },),
    it({
      name: 'retained raw accessor configuration and mock controllers cannot reinstall target state',
      fn: async (): Promise<void> => {
        /** Ordinary Sinon data-property behavior still needs late-mutation guards. */
        const target = { value: 'original', method: (): string => 'original', };
        /** Callables are retained exactly as a consumer can retain them. */
        const late: (() => unknown)[] = [];
        await it({
          name: 'controller owner',
          fn: async ({ sinon, }: TestContext,): Promise<void> => {
            /** This data property is deliberately outside contextual method routing. */
            const stub = sinon.stub(target, 'value',).value('fake',);
            /** Mocking installs the method when expects is called, not at controller creation. */
            const mock = sinon.mock(target,);
            /** Saved factory namespaces must enforce completion at invocation. */
            const returns: unknown = Reflect.get(sinon.fake, 'returns',);
            if ((typeof returns) !== 'function')
              throw new Error('Sinon fake namespace did not provide returns',);
            /** A saved plain method must carry the same guard as a property lookup. */
            const {spy} = sinon;
            late.push(
              () => stub.value('late',),
              () => stub.get(() => 'late',),
              () => stub.set(() => {},),
              () => mock.expects('method',),
              () => Reflect.apply(returns, undefined, ['late',],),
              () => spy(target, 'method',),
            );
          },
        },);
        for (const mutate of late)
          expect(mutate,).toThrow('completed',);
        expect(target.value,).toBe('original',);
        expect(target.method(),).toBe('original',);
      },
    },),
    it({
      name: 'old fake restoration cannot remove a later generation',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        /** Manual restoration followed by restubbing shares only the original descriptor. */
        const target = { method: (): string => 'original', };
        const first = sinon.stub(target, 'method',).returns('first',);
        first.restore();
        const second = sinon.stub(target, 'method',).returns('second',);
        first.restore();
        expect(target.method(),).toBe('second',);
        expect(second.callCount,).toBe(1,);
        sinon.restore();
        expect(target.method(),).toBe('original',);
      },
    },),
    it({
      name: 'manual ordinary data restoration retires the old fake before restubbing',
      fn: async ({ sinon, }: TestContext,): Promise<void> => {
        const target = { value: 'original', };
        const first = sinon.stub(target, 'value',).value('first',);
        first.restore();
        sinon.stub(target, 'value',).value('second',);
        first.restore();
        expect(target.value,).toBe('second',);
        expect(() => first.value('late',),).toThrow('restored',);
        sinon.restore();
        expect(target.value,).toBe('original',);
      },
    },),
    it({
      name: 'registry cleanup survives an overwritten fake restorer and retains both failures',
      fn: async (): Promise<void> => {
        /** Every contextual property must be restored even if raw Sinon restoration throws. */
        const target = { first: (): string => 'first', second: (): string => 'second', };
        /** Capture the thrown error without a legacy rejects matcher. */
        const failures: unknown[] = [];
        try {
          await it({
            name: 'failing owner',
            fn: async ({ sinon, }: TestContext,): Promise<void> => {
              const first = sinon.stub(target, 'first',);
              sinon.stub(target, 'second',);
              first.restore = () => {
                throw new Error('restore failure',);
              };
              throw new Error('body failure',);
            },
          },);
        }
        catch (error) {
          failures.push(error,);
        }
        expect(failures.length,).toBe(1,);
        /** The outer runner preserves the aggregate as the test's cause. */
        const [failure,] = failures;
        expect(failure,).toBeInstanceOf(Error,);
        if ((!(Error.isError(failure,))) || (!(failure.cause instanceof AggregateError)))
          throw new Error('Expected body and cleanup aggregate',);
        expect(failure.cause.errors.length,).toBe(2,);
        expect(target.first(),).toBe('first',);
        expect(target.second(),).toBe('second',);
      },
    },),
  ],
},);
