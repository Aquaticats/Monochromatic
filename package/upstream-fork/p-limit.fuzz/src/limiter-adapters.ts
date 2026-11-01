/**
 Limiter adapters exposing one uniform workload surface over this package's
 fork and upstream `p-limit`, so the same generated workload can drive both.
 
 @module
 */

import pLimitUpstream from 'p-limit';

import { pLimit, } from '@monochromatic-dev/upstream-fork-p-limit/ts';

import type { LimiterAdapter, } from './workload.ts';

//region Adapters

/**
 Adapts this package's fork to the workload surface.
 
 @param options - Concurrency bound and `rejectOnClear` for the limiter.
 
 @returns Adapter driving the fork's `pLimit` limiter.
 
 @example
 ```ts
 const adapter = createForkAdapter({
   concurrency: 2,
   rejectOnClear: false,
 });
 ```
 */
export function createForkAdapter(options: {
  readonly concurrency: number;
  readonly rejectOnClear: boolean;
},): LimiterAdapter {
  /**
   Fork limiter under test.
   */
  const limit = pLimit(options,);
  return {
    schedule: function scheduleFork(task: () => string | Promise<string>,): Promise<string> {
      return limit({
        fn: task,
        args: [],
      },);
    },
    activeCount: function readForkActiveCount(): number {
      return limit.activeCount;
    },
    pendingCount: function readForkPendingCount(): number {
      return limit.pendingCount;
    },
    concurrency: function readForkConcurrency(): number {
      return limit.concurrency;
    },
    setConcurrency: function writeForkConcurrency(concurrency: number,): void {
      limit.concurrency = concurrency;
    },
    clearQueue: function clearForkQueue(): void {
      limit.clearQueue();
    },
  };
}

/**
 Adapts upstream `p-limit` to the workload surface.
 
 @param options - Concurrency bound and `rejectOnClear` for the limiter.
 
 @returns Adapter driving an upstream `p-limit` limiter.
 
 @example
 ```ts
 const adapter = createUpstreamAdapter({
   concurrency: 2,
   rejectOnClear: false,
 });
 ```
 */
export function createUpstreamAdapter(options: {
  readonly concurrency: number;
  readonly rejectOnClear: boolean;
},): LimiterAdapter {
  /**
   Upstream limiter under comparison.
   */
  const limit = pLimitUpstream(options,);
  return {
    schedule: function scheduleUpstream(task: () => string | Promise<string>,): Promise<string> {
      return limit(task,);
    },
    activeCount: function readUpstreamActiveCount(): number {
      return limit.activeCount;
    },
    pendingCount: function readUpstreamPendingCount(): number {
      return limit.pendingCount;
    },
    concurrency: function readUpstreamConcurrency(): number {
      return limit.concurrency;
    },
    setConcurrency: function writeUpstreamConcurrency(concurrency: number,): void {
      limit.concurrency = concurrency;
    },
    clearQueue: function clearUpstreamQueue(): void {
      limit.clearQueue();
    },
  };
}

//endregion Adapters
