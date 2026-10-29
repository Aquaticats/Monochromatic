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
import { typeFilter, } from './type.ts';

/**
 Logger root for watch-restart after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: defaultLogger, },);
 ```
 */
const defaultLogger = tagged({ tag: 'watch-restart', },);

/**
 Builds a minimal {@link WatchCtx}; typeFilter ignores everything but
 `event.entity`, so a default-everything ctx is sufficient.
 
 @returns context object suitable for handing to typeFilter
 
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
 to a file under `/abs/`. Tests vary `entity` and `kind`.
 
 @param overrides - partial event fields to merge over the default
 
 @returns fully-populated event
 
 @example
 ```ts
 const event = makeEvent({ entity: 'dir', kind: 'addDir', },);
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
  name: typeFilter.name,
  children: [
    it({
      name: 'admits file events when types is [file]',
      fn: async function admitsFiles() {
        const filter = typeFilter(['file',],);
        const result = await filter({
          event: makeEvent({ entity: 'file', kind: 'change', },),
          ctx: makeCtx(),
        },);
        expect(result,).toBe(true,);
      },
    },),
    it({
      name: 'rejects dir events when types is [file]',
      fn: async function rejectsDirs() {
        const filter = typeFilter(['file',],);
        const result = await filter({
          event: makeEvent({ entity: 'dir', kind: 'addDir', },),
          ctx: makeCtx(),
        },);
        expect(result,).toBe(false,);
      },
    },),
    it({
      name: 'admits dir events when types is [dir]',
      fn: async function admitsDirs() {
        const filter = typeFilter(['dir',],);
        const result = await filter({
          event: makeEvent({ entity: 'dir', kind: 'unlinkDir', },),
          ctx: makeCtx(),
        },);
        expect(result,).toBe(true,);
      },
    },),
    it({
      name: 'rejects file events when types is [dir]',
      fn: async function rejectsFiles() {
        const filter = typeFilter(['dir',],);
        const result = await filter({
          event: makeEvent({ entity: 'file', kind: 'add', },),
          ctx: makeCtx(),
        },);
        expect(result,).toBe(false,);
      },
    },),
    it({
      name: 'admits both when types is [file, dir]',
      fn: async function admitsBoth() {
        const filter = typeFilter(['file', 'dir',],);
        const fileResult = await filter({
          event: makeEvent({ entity: 'file', },),
          ctx: makeCtx(),
        },);
        const dirResult = await filter({
          event: makeEvent({ entity: 'dir', kind: 'addDir', },),
          ctx: makeCtx(),
        },);
        expect(fileResult,).toBe(true,);
        expect(dirResult,).toBe(true,);
      },
    },),
    it({
      name: 'admits every event when types is empty (vacuous pass-all)',
      fn: async function vacuousPass() {
        const filter = typeFilter([],);
        const fileResult = await filter({
          event: makeEvent({ entity: 'file', },),
          ctx: makeCtx(),
        },);
        const dirResult = await filter({
          event: makeEvent({ entity: 'dir', kind: 'addDir', },),
          ctx: makeCtx(),
        },);
        expect(fileResult,).toBe(true,);
        expect(dirResult,).toBe(true,);
      },
    },),
  ],
},);
