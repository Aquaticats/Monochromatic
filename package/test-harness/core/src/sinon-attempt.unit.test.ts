/** Attempt lifetime regression tests through the public harness artifact. @module */
import {
  describe,
  expect,
  it,
  type TestContext,
} from '@monochromatic-dev/test-harness-core';

await describe({
  name: 'sandbox attempt lifetime',
  children: [
    it({
      name: 'retained sandbox factories reject after successful completion',
      fn: async (): Promise<void> => {
        /** Fixture survives its owner so late mutation can be observed. */
        const target = { method: (): string => 'original', };
        /** Completed context deliberately retained by a consumer. */
        const captured = Promise.withResolvers<TestContext>();
        await it({
          name: 'finished owner',
          fn: async (ctx: TestContext,): Promise<void> => {
            captured.resolve(ctx);
            ctx.sinon.stub(target, 'method',).returns('fake',);
          },
        },);
        /** Wait for the exact context supplied by the completed inner test. */
        const completed = await captured.promise;
        expect(target.method(),).toBe('original',);
        expect(() => completed.sinon.stub(target, 'method',),).toThrow('completed',);
        expect(target.method(),).toBe('original',);
      },
    },),
    it({
      name: 'repeats get distinct contexts and reject factories retained from an older attempt',
      fn: async (): Promise<void> => {
        /** Retained contexts distinguish lifetime identity from descriptor identity. */
        const contexts: TestContext[] = [];
        await it({
          name: 'repeated owner',
          repeats: 1,
          fn: async (ctx: TestContext,): Promise<void> => {
            /** First attempt must not remain an authority during the next one. */
            const [previous,] = contexts;
            if (previous !== undefined) {
              expect(previous,).not.toBe(ctx,);
              expect(previous.sinon,).not.toBe(ctx.sinon,);
              expect(() => previous.sinon.stub(),).toThrow('completed',);
            }
            contexts.push(ctx,);
            expect(ctx.sinon.stub().callCount,).toBe(0,);
          },
        },);
        expect(contexts.length,).toBe(2,);
      },
    },),
  ],
},);
