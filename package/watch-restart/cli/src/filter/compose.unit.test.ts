import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { HashCache, } from '../hash-cache.ts';
import type {
  WatchCtx,
  WatchEvent,
  WatchFilter,
} from '../types.ts';
import {
  anyFilter,
  composeFilters,
} from './compose.ts';

/**
 Logger root for watch-restart after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: defaultLogger, },);
 ```
 */
const defaultLogger = tagged({ tag: 'watch-restart', },);

/**
 Milliseconds the "async filter resolves correctly" test holds before
 resolving; small enough not to slow the suite, large enough to be a
 real microtask boundary rather than synchronous.
 */
const ASYNC_FILTER_DELAY_MS = 5;

/**
 Builds a minimal {@link WatchCtx}; compose passes the ctx through
 to inner filters but never inspects it itself.
 
 @returns context object
 
 @example
 ```ts
 const ctx = makeCtx();
 ```
 */
function makeCtx(): WatchCtx {
  return {
    logger: defaultLogger,
    hashCache: new HashCache(),
    signal: new AbortController().signal,
  };
}

/**
 Builds a {@link WatchEvent}; defaults give a `change` to `file.ts`.
 
 @returns fully-populated event with `kind === 'change'`
 
 @example
 ```ts
 const event = makeEvent();
 ```
 */
function makeEvent(): WatchEvent {
  return {
    kind: 'change',
    entity: 'file',
    path: '/abs/file.ts',
    relativePath: 'file.ts',
    ext: '.ts',
  };
}

/**
 Builds a {@link WatchFilter} that records its name in `callLog` on each
 invocation and returns the configured result.
 
 The recording array (`callLog`) is a single const-bound `string[]`
 shared across the test body and the filter closure; tests inspect it
 after running compose to assert short-circuit (or its absence). Using
 an array of names instead of a counter keeps order of calls visible.
 
 @param name - identifier appended to `callLog` on each call
 
 @param result - boolean (or promise of boolean) the filter resolves to
 
 @param callLog - external array the filter appends to on each call
 
 @returns a recording filter suitable for handing to composeFilters/anyFilter
 
 @example
 ```ts
 const calls: string[] = [];
 const f = recordingFilter({ name: 'f', result: true, callLog: calls, },);
 ```
 */
function recordingFilter(
  {
    name,
    result,
    callLog,
  }: {
    readonly name: string;
    readonly result: boolean | Promise<boolean>;
    readonly callLog: string[];
  },
): WatchFilter {
  return async function recordingFilterFn(
    {
      event: _event,
      ctx: _ctx,
    }: {
      readonly event: WatchEvent;
      readonly ctx: WatchCtx;
    },
  ): Promise<boolean> {
    callLog.push(name,);
    return result;
  };
}

await describe({
  name: composeFilters.name,
  children: [
    it({
      name: 'returns true when every filter returns true',
      fn: async function allTrue() {
        const calls: string[] = [];
        const filter = composeFilters([
          recordingFilter({ name: 'a', result: true, callLog: calls, },),
          recordingFilter({ name: 'b', result: true, callLog: calls, },),
        ],);
        const passed = await filter({
          event: makeEvent(),
          ctx: makeCtx(),
        },);

        expect(passed,).toBe(true,);
        expect(calls,).toEqual(['a', 'b',],);
      },
    },),
    it({
      name: 'returns false on first false and short-circuits later filters',
      fn: async function shortCircuitsOnFalse() {
        const calls: string[] = [];
        const filter = composeFilters([
          recordingFilter({ name: 'a', result: true, callLog: calls, },),
          recordingFilter({ name: 'b', result: false, callLog: calls, },),
          recordingFilter({ name: 'c', result: true, callLog: calls, },),
        ],);
        const passed = await filter({
          event: makeEvent(),
          ctx: makeCtx(),
        },);

        expect(passed,).toBe(false,);
        // 'c' must not have run after 'b' said skip.
        expect(calls,).toEqual(['a', 'b',],);
      },
    },),
    it({
      name: 'empty filter list is a vacuous true',
      fn: async function emptyIsTrue() {
        const filter = composeFilters([],);
        const passed = await filter({
          event: makeEvent(),
          ctx: makeCtx(),
        },);
        expect(passed,).toBe(true,);
      },
    },),
    it({
      name: 'awaits async filters before deciding',
      fn: async function awaitsAsync() {
        const calls: string[] = [];
        const filter = composeFilters([
          recordingFilter({
            name: 'slow',
            result: wait(ASYNC_FILTER_DELAY_MS,).then(function resolveTrue() {
              return true;
            },),
            callLog: calls,
          },),
          recordingFilter({ name: 'fast', result: true, callLog: calls, },),
        ],);
        const passed = await filter({
          event: makeEvent(),
          ctx: makeCtx(),
        },);

        expect(passed,).toBe(true,);
        expect(calls,).toEqual(['slow', 'fast',],);
      },
    },),
  ],
},);

await describe({
  name: anyFilter.name,
  children: [
    it({
      name: 'returns true on first true and short-circuits later filters',
      fn: async function shortCircuitsOnTrue() {
        const calls: string[] = [];
        const filter = anyFilter([
          recordingFilter({ name: 'a', result: false, callLog: calls, },),
          recordingFilter({ name: 'b', result: true, callLog: calls, },),
          recordingFilter({ name: 'c', result: false, callLog: calls, },),
        ],);
        const passed = await filter({
          event: makeEvent(),
          ctx: makeCtx(),
        },);

        expect(passed,).toBe(true,);
        // 'c' must not have run after 'b' said fire.
        expect(calls,).toEqual(['a', 'b',],);
      },
    },),
    it({
      name: 'returns false when every filter returns false',
      fn: async function allFalse() {
        const calls: string[] = [];
        const filter = anyFilter([
          recordingFilter({ name: 'a', result: false, callLog: calls, },),
          recordingFilter({ name: 'b', result: false, callLog: calls, },),
        ],);
        const passed = await filter({
          event: makeEvent(),
          ctx: makeCtx(),
        },);

        expect(passed,).toBe(false,);
        expect(calls,).toEqual(['a', 'b',],);
      },
    },),
    it({
      name: 'empty filter list is a vacuous false',
      fn: async function emptyIsFalse() {
        const filter = anyFilter([],);
        const passed = await filter({
          event: makeEvent(),
          ctx: makeCtx(),
        },);
        expect(passed,).toBe(false,);
      },
    },),
  ],
},);
