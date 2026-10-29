import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { createLogger, } from '../dist/final/neutral/index.mjs';

//region Public logger completion ownership survives sharing, deadlines, and rejection

await describe({
  name: 'shared sink completion',
  children: [
    it({
      name: 'delivers every record and reports one shared rejection before accepting later writes',
      fn: async ({ sinon, },) => {
        const warnings = sinon.stub(console, 'warn',);
        const operation = Promise.withResolvers<void>();
        const completed = Promise.resolve();
        const state = { delivered: 0, flushed: false, };
        const { logger, initPromise, } = createLogger({
          sinks: [{
            verify: async function verify(): Promise<boolean> {
              return true;
            },
            write: function write(): Promise<void> {
              state.delivered += 1;
              return state.delivered <= 2 ? operation.promise : completed;
            },
          },],
        },);
        await initPromise;
        logger.info('first',);
        logger.info('second',);
        /** Consumer completion must wait for the shared backend operation. */
        async function flush(): Promise<void> {
          await logger.flush();
          state.flushed = true;
        }
        const flushing = flush();
        await wait(1,);
        expect(state.flushed,).toBe(false,);
        operation.reject(new Error('shared backend failure',),);
        await flushing;
        logger.info('after rejection',);
        await logger.flush();
        expect(state.delivered,).toBe(3,);
        expect(warnings.getCalls().flatMap(function argumentsOf(call,): unknown[] {
          return call.args;
        },).filter(function sharedFailure(value,) {
          return ((typeof value) === 'string')
            && value.includes('sink write promise rejected while being tracked',);
        },),).toHaveLength(1,);
      },
    },),
    it({
      name: 'retracks the same pending operation after an earlier flush deadline abandons it',
      fn: async ({ sinon, },) => {
        sinon.stub(console, 'warn',);
        const operation = Promise.withResolvers<void>();
        const state = { delivered: 0, flushed: false, };
        const { logger, initPromise, } = createLogger({
          flushDeadlineMs: 100,
          sinks: [{
            verify: async function verify(): Promise<boolean> {
              return true;
            },
            write: function write(): Promise<void> {
              state.delivered += 1;
              return operation.promise;
            },
          },],
        },);
        await initPromise;
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            operation.resolve();
            await logger.flush();
          },
        };
        logger.info('abandoned observation',);
        await logger.flush();
        logger.info('new observation of the same operation',);
          /** Re-admitted work must remain part of the next flush until its operation settles. */
        async function flush(): Promise<void> {
          await logger.flush();
          state.flushed = true;
        }
        const flushing = flush();
        await wait(1,);
        expect(state.flushed,).toBe(false,);
        operation.resolve();
        await flushing;
        expect(state.delivered,).toBe(2,);
      },
    },),
  ],
},);

//endregion Public logger completion ownership survives sharing, deadlines, and rejection
