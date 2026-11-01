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
} from '../types.ts';
import { extFilter, } from './ext.ts';

/**
 Logger root for watch-restart after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: defaultLogger, },);
 ```
 */
const defaultLogger = tagged({ tag: 'watch-restart', },);

/**
 Builds a minimal {@link WatchCtx}; extFilter ignores everything but
 `event.ext`, so a default-everything ctx is sufficient.
 
 @returns context object suitable for handing to extFilter
 
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
 Builds a {@link WatchEvent} from overrides; defaults give a `change`
 to `/abs/file.ts`. The `ext` override is the only field the tests
 vary.
 
 @param overrides - partial event fields to merge over the default
 
 @returns fully-populated event
 
 @example
 ```ts
 const event = makeEvent({ ext: '.css', },);
 ```
 */
function makeEvent(
  overrides: {
    readonly kind?: WatchEvent['kind'];
    readonly entity?: WatchEvent['entity'];
    readonly path?: WatchEvent['path'];
    readonly relativePath?: WatchEvent['relativePath'];
    readonly ext?: WatchEvent['ext'];
  } = {},
): WatchEvent {
  return {
    kind: overrides.kind ?? 'change',
    entity: overrides.entity ?? 'file',
    path: overrides.path ?? '/abs/file.ts',
    relativePath: overrides.relativePath ?? 'file.ts',
    ext: overrides.ext ?? '.ts',
  };
}

await describe({
  name: extFilter.name,
  children: [
    it({
      name: 'matches when event.ext is in the configured list',
      fn: async function matchesListed() {
        const filter = extFilter(['.ts', '.tsx',],);
        const passed = await filter({
          event: makeEvent({ ext: '.ts', },),
          ctx: makeCtx(),
        },);
        expect(passed,).toBe(true,);
      },
    },),
    it({
      name: 'rejects when event.ext is not in the list',
      fn: async function rejectsUnlisted() {
        const filter = extFilter(['.ts',],);
        const passed = await filter({
          event: makeEvent({ ext: '.css', },),
          ctx: makeCtx(),
        },);
        expect(passed,).toBe(false,);
      },
    },),
    it({
      name: 'is case-insensitive on both sides',
      fn: async function caseInsensitive() {
        const filter = extFilter(['.TS',],);
        const lower = await filter({
          event: makeEvent({ ext: '.ts', },),
          ctx: makeCtx(),
        },);
        const upper = await filter({
          event: makeEvent({ ext: '.TS', },),
          ctx: makeCtx(),
        },);
        const mixed = await filter({
          event: makeEvent({ ext: '.Ts', },),
          ctx: makeCtx(),
        },);

        expect(lower,).toBe(true,);
        expect(upper,).toBe(true,);
        expect(mixed,).toBe(true,);
      },
    },),
    it({
      name: 'leading dot on the configured ext is optional',
      fn: async function dotOptional() {
        const withDot = extFilter(['.ts',],);
        const withoutDot = extFilter(['ts',],);

        const a = await withDot({
          event: makeEvent({ ext: '.ts', },),
          ctx: makeCtx(),
        },);
        const b = await withoutDot({
          event: makeEvent({ ext: '.ts', },),
          ctx: makeCtx(),
        },);

        expect(a,).toBe(true,);
        expect(b,).toBe(true,);
      },
    },),
    it({
      name: 'empty extension list is a vacuous pass-all',
      fn: async function emptyPassesAll() {
        const filter = extFilter([],);
        const passedTs = await filter({
          event: makeEvent({ ext: '.ts', },),
          ctx: makeCtx(),
        },);
        const passedCss = await filter({
          event: makeEvent({ ext: '.css', },),
          ctx: makeCtx(),
        },);
        const passedNone = await filter({
          event: makeEvent({ ext: '', },),
          ctx: makeCtx(),
        },);

        expect(passedTs,).toBe(true,);
        expect(passedCss,).toBe(true,);
        expect(passedNone,).toBe(true,);
      },
    },),
  ],
},);
