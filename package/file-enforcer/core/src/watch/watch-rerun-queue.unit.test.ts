import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createWatchRerunQueue,
  type WatchRerunBatch,
  type WatchRerunReporterLogger,
} from '../../dist/final/node/index.mjs';

/**
 Test logger that suppresses expected reporter-failure output.
 */
const quietReporterLogger: WatchRerunReporterLogger = {
  error(message: string,): void {
    void message;
  },
};

await describe({
  name: createWatchRerunQueue.name,
  children: [
    it({
      name: 'serializes reruns queued while a rerun is active',
      fn: async function serializesQueuedReruns(): Promise<void> {
        /**
         Signal resolved when first rerun starts.
         */
        const firstRerunStarted = Promise.withResolvers<void>();
        /**
         Signal that keeps first rerun active while second batch is queued.
         */
        const releaseFirstRerun = Promise.withResolvers<void>();
        /**
         Rerun batch names in handler execution order.
         */
        const runOrder: string[] = [];
        /**
         Current number of active rerun handlers.
         */
        const activeRerunCount = new Map<'count', number>([
          [
            'count',
            0,
          ],
        ],);
        /**
         Highest active handler count observed during the test.
         */
        const maxActiveRerunCount = new Map<'count', number>([
          [
            'count',
            0,
          ],
        ],);
        /**
         Errors reported by queue error handling.
         */
        const reportedErrors: unknown[] = [];
        /**
         Serial rerun queue under test.
         */
        const queue = createWatchRerunQueue({
          run: async function recordRerun(batch: WatchRerunBatch,): Promise<void> {
            /**
             Updated active handler count after this handler starts.
             */
            const nextActiveCount = (activeRerunCount.get('count',) ?? 0) + 1;
            activeRerunCount.set(
              'count',
              nextActiveCount,
            );
            maxActiveRerunCount.set(
              'count',
              Math.max(
                maxActiveRerunCount.get('count',) ?? 0,
                nextActiveCount,
              ),
            );
            /**
             Name carried by this synthetic rerun batch.
             */
            const [batchName,] = batch.paths;
            if (batchName === undefined)
              throw new Error('Test rerun batch is missing its name path',);

            runOrder.push(batchName,);
            if (batchName === 'first') {
              firstRerunStarted.resolve();
              await releaseFirstRerun.promise;
            }
            activeRerunCount.set(
              'count',
              (activeRerunCount.get('count',) ?? 0) - 1,
            );
          },
          onError: function recordError(error: unknown,): void {
            reportedErrors.push(error,);
          },
        },);

        /**
         Completion promise for first queued rerun.
         */
        const firstRerunFinished = queue.enqueue({
          paths: ['first',],
          protectedPaths: [],
        },);
        await firstRerunStarted.promise;
        /**
         Completion promise for second queued rerun.
         */
        const secondRerunFinished = queue.enqueue({
          paths: ['second',],
          protectedPaths: [],
        },);

        expect(runOrder,).toEqual(['first',],);
        expect(queue.running(),).toBe(true,);
        expect(queue.pendingCount(),).toBe(1,);

        releaseFirstRerun.resolve();
        await Promise.all([
          firstRerunFinished,
          secondRerunFinished,
        ],);

        expect(runOrder,).toEqual([
          'first',
          'second',
        ],);
        expect(maxActiveRerunCount.get('count',),).toBe(1,);
        expect(reportedErrors,).toEqual([],);
      },
    },),

    it({
      name: 'continues draining after rerun and reporter failures',
      fn: async function continuesAfterRerunAndReporterFailures(): Promise<void> {
        /**
         Rerun batch names in handler execution order.
         */
        const runOrder: string[] = [];
        /**
         Errors received by configured error reporter.
         */
        const reportedErrors: unknown[] = [];
        /**
         Queue under test with failing first rerun and failing reporter.
         */
        const queue = createWatchRerunQueue({
          run: async function runMaybeFailingBatch(batch: WatchRerunBatch,): Promise<void> {
            /**
             Name carried by this synthetic rerun batch.
             */
            const [batchName,] = batch.paths;
            if (batchName === undefined)
              throw new Error('Test rerun batch is missing its name path',);

            runOrder.push(batchName,);
            if (batchName === 'first')
              throw new Error('synthetic first rerun failure',);
          },
          onError: function reportAndFail(error: unknown,): void {
            reportedErrors.push(error,);
            throw new Error('synthetic reporter failure',);
          },
          logger: quietReporterLogger,
        },);

        await Promise.all([
          queue.enqueue({
            paths: ['first',],
            protectedPaths: [],
          },),
          queue.enqueue({
            paths: ['second',],
            protectedPaths: [],
          },),
        ],);

        expect(runOrder,).toEqual([
          'first',
          'second',
        ],);
        expect(reportedErrors,).toHaveLength(1,);
        expect(reportedErrors[0],).toBeInstanceOf(Error,);
        expect(queue.running(),).toBe(false,);
        expect(queue.pendingCount(),).toBe(0,);
      },
    },),
  ],
},);
