import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { compileRegex, } from '../cli-helpers.ts';
import { HashCache, } from '../hash-cache.ts';
import type {
  WatchCtx,
  WatchEvent,
} from '../types.ts';
import { regexFilter, } from './regex.ts';

/**
 Logger root for watch-restart after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: defaultLogger, },);
 ```
 */
const defaultLogger = tagged({ tag: 'watch-restart', },);

/**
 Builds a minimal {@link WatchCtx}; regexFilter ignores everything but
 `event.relativePath`.
 
 @returns context object suitable for handing to regexFilter
 
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
 to `file.ts`.
 
 @param overrides - partial event fields to merge over the default
 
 @returns fully-populated event
 
 @example
 ```ts
 const event = makeEvent({ relativePath: 'src/foo.ts', },);
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
  name: regexFilter.name,
  children: [
    it({
      name: 'no patterns is a vacuous pass-all',
      fn: async function defaultsPassAll() {
        const filter = regexFilter({},);
        const passed = await filter({
          event: makeEvent({ relativePath: 'anywhere/file.weird', },),
          ctx: makeCtx(),
        },);
        expect(passed,).toBe(true,);
      },
    },),
    it({
      name: 'include only: passes when at least one include matches',
      fn: async function includeMatches() {
        const filter = regexFilter({
          include: [compileRegex(String.raw`^src/.*\.ts$`,),],
        },);
        const matched = await filter({
          event: makeEvent({ relativePath: 'src/server/index.ts', },),
          ctx: makeCtx(),
        },);
        const unmatched = await filter({
          event: makeEvent({ relativePath: 'doc/index.md', },),
          ctx: makeCtx(),
        },);

        expect(matched,).toBe(true,);
        expect(unmatched,).toBe(false,);
      },
    },),
    it({
      name: 'include only: multiple includes OR together',
      fn: async function multipleIncludesOr() {
        const filter = regexFilter({
          include: [
            compileRegex(String.raw`\.ts$`,),
            compileRegex(String.raw`\.tsx$`,),
          ],
        },);
        const ts = await filter({
          event: makeEvent({ relativePath: 'src/index.ts', },),
          ctx: makeCtx(),
        },);
        const tsx = await filter({
          event: makeEvent({ relativePath: 'src/comp.tsx', },),
          ctx: makeCtx(),
        },);
        const css = await filter({
          event: makeEvent({ relativePath: 'src/index.css', },),
          ctx: makeCtx(),
        },);

        expect(ts,).toBe(true,);
        expect(tsx,).toBe(true,);
        expect(css,).toBe(false,);
      },
    },),
    it({
      name: 'exclude only: rejects when an exclude matches, passes otherwise',
      fn: async function excludeOnly() {
        const filter = regexFilter({
          exclude: [compileRegex(String.raw`\.test\.ts$`,),],
        },);
        const test = await filter({
          event: makeEvent({ relativePath: 'src/foo.test.ts', },),
          ctx: makeCtx(),
        },);
        const real = await filter({
          event: makeEvent({ relativePath: 'src/foo.ts', },),
          ctx: makeCtx(),
        },);

        expect(test,).toBe(false,);
        expect(real,).toBe(true,);
      },
    },),
    it({
      name: 'both include and exclude: exclude wins on overlap',
      fn: async function excludeWins() {
        const filter = regexFilter({
          include: [compileRegex(String.raw`^src/.*\.ts$`,),],
          exclude: [compileRegex(String.raw`\.test\.ts$`,),],
        },);
        const overlap = await filter({
          event: makeEvent({ relativePath: 'src/foo.test.ts', },),
          ctx: makeCtx(),
        },);
        const includedOnly = await filter({
          event: makeEvent({ relativePath: 'src/foo.ts', },),
          ctx: makeCtx(),
        },);
        const neither = await filter({
          event: makeEvent({ relativePath: 'doc/foo.md', },),
          ctx: makeCtx(),
        },);

        expect(overlap,).toBe(false,);
        expect(includedOnly,).toBe(true,);
        expect(neither,).toBe(false,);
      },
    },),
    it({
      name: 'anchored alternation not expressible in picomatch is expressible here',
      fn: async function anchoredAlternation() {
        const filter = regexFilter({
          exclude: [
            compileRegex(String.raw`\.(test|spec|fixture)\.[jt]sx?$`,),
          ],
        },);
        const test = await filter({
          event: makeEvent({ relativePath: 'src/foo.test.ts', },),
          ctx: makeCtx(),
        },);
        const spec = await filter({
          event: makeEvent({ relativePath: 'src/foo.spec.tsx', },),
          ctx: makeCtx(),
        },);
        const fixture = await filter({
          event: makeEvent({ relativePath: 'src/foo.fixture.jsx', },),
          ctx: makeCtx(),
        },);
        const real = await filter({
          event: makeEvent({ relativePath: 'src/foo.ts', },),
          ctx: makeCtx(),
        },);

        expect(test,).toBe(false,);
        expect(spec,).toBe(false,);
        expect(fixture,).toBe(false,);
        expect(real,).toBe(true,);
      },
    },),
  ],
},);
