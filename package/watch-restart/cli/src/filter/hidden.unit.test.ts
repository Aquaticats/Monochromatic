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
import { hiddenFilter, } from './hidden.ts';

/**
 Logger root for watch-restart after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: defaultLogger, },);
 ```
 */
const defaultLogger = tagged({ tag: 'watch-restart', },);

/**
 Builds a minimal {@link WatchCtx}; hiddenFilter ignores everything but
 `event.relativePath`.
 
 @returns context object suitable for handing to hiddenFilter
 
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
 const event = makeEvent({ relativePath: '.cache/data', },);
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

/**
 Reports whether the default (allowHidden off) filter treats `relativePath`
 as a hidden segment, i.e. rejects it.
 
 Inverts the filter result: the filter returns `true` to admit a non-hidden
 path, so a rejected (hidden) path negates to `true` here. Keeps the
 equivalence assertions below readable as direct hidden/not-hidden facts.
 
 @param relativePath - event relative path under inspection
 
 @returns whether `relativePath` contains a hidden segment
 
 @example
 ```ts
 await isHidden('.config.ts'); // true
 await isHidden('src/foo.ts'); // false
 ```
 */
async function isHidden(relativePath: string,): Promise<boolean> {
  const filter = hiddenFilter({},);
  const passed = await filter({
    event: makeEvent({ relativePath, },),
    ctx: makeCtx(),
  },);
  return !passed;
}

await describe({
  name: hiddenFilter.name,
  children: [
    it({
      name: 'default (allowHidden off) rejects leading-dot segment at root',
      fn: async function rejectsRootDotfile() {
        const filter = hiddenFilter({},);
        const passed = await filter({
          event: makeEvent({ relativePath: '.config.ts', },),
          ctx: makeCtx(),
        },);
        expect(passed,).toBe(false,);
      },
    },),
    it({
      name: 'default rejects leading-dot segment nested in path',
      fn: async function rejectsNestedDotfile() {
        const filter = hiddenFilter({},);
        const swapFile = await filter({
          event: makeEvent({ relativePath: 'src/.foo.swp', },),
          ctx: makeCtx(),
        },);
        const gitIndex = await filter({
          event: makeEvent({ relativePath: 'src/.git/index', },),
          ctx: makeCtx(),
        },);
        const cacheDir = await filter({
          event: makeEvent({
            relativePath: 'src/.cache/data',
            entity: 'dir',
            kind: 'addDir',
          },),
          ctx: makeCtx(),
        },);

        expect(swapFile,).toBe(false,);
        expect(gitIndex,).toBe(false,);
        expect(cacheDir,).toBe(false,);
      },
    },),
    it({
      name: 'default admits paths without any hidden segment',
      fn: async function admitsNonHidden() {
        const filter = hiddenFilter({},);
        const plain = await filter({
          event: makeEvent({ relativePath: 'src/index.ts', },),
          ctx: makeCtx(),
        },);
        const deep = await filter({
          event: makeEvent({ relativePath: 'src/a/b/c.ts', },),
          ctx: makeCtx(),
        },);
        expect(plain,).toBe(true,);
        expect(deep,).toBe(true,);
      },
    },),
    it({
      name: 'allowHidden=true passes hidden paths through',
      fn: async function allowsHidden() {
        const filter = hiddenFilter({ allowHidden: true, },);
        const dotfile = await filter({
          event: makeEvent({ relativePath: '.config.ts', },),
          ctx: makeCtx(),
        },);
        const swap = await filter({
          event: makeEvent({ relativePath: 'src/.foo.swp', },),
          ctx: makeCtx(),
        },);
        const plain = await filter({
          event: makeEvent({ relativePath: 'src/index.ts', },),
          ctx: makeCtx(),
        },);
        expect(dotfile,).toBe(true,);
        expect(swap,).toBe(true,);
        expect(plain,).toBe(true,);
      },
    },),
    it({
      name: 'default does not treat extension-only dot as hidden',
      fn: async function dotInExtension() {
        const filter = hiddenFilter({},);
        const passed = await filter({
          event: makeEvent({ relativePath: 'src/file.config.ts', },),
          ctx: makeCtx(),
        },);
        expect(passed,).toBe(true,);
      },
    },),
    it({
      name: 'default matches Windows-style backslash separators',
      fn: async function windowsSeparators() {
        const filter = hiddenFilter({},);
        const passed = await filter({
          event: makeEvent({ relativePath: String.raw`src\.swp`, },),
          ctx: makeCtx(),
        },);
        expect(passed,).toBe(false,);
      },
    },),
    it({
      name: 'no-args call uses the default (allowHidden off)',
      fn: async function noArgsDefault() {
        const filter = hiddenFilter();
        const dotfile = await filter({
          event: makeEvent({ relativePath: '.config.ts', },),
          ctx: makeCtx(),
        },);
        expect(dotfile,).toBe(false,);
      },
    },),
    it({
      name: 'equivalence: empty, whitespace-only, and plain paths are not hidden',
      fn: async function notHiddenBaseline() {
        expect(await isHidden('',),).toBe(false,);
        expect(await isHidden('   ',),).toBe(false,);
        expect(await isHidden('src/foo.ts',),).toBe(false,);
        expect(await isHidden('src/a/b/c.ts',),).toBe(false,);
      },
    },),
    it({
      name: 'equivalence: leading separator opens a segment; a bare leading separator does not',
      fn: async function leadingSeparator() {
        expect(await isHidden('/.foo',),).toBe(true,);
        expect(await isHidden('/foo',),).toBe(false,);
      },
    },),
    it({
      name: 'equivalence: trailing separators',
      fn: async function trailingSeparator() {
        expect(await isHidden('foo/',),).toBe(false,);
        expect(await isHidden('foo/.',),).toBe(false,);
        expect(await isHidden('a/.',),).toBe(false,);
        expect(await isHidden('src/foo/.',),).toBe(false,);
      },
    },),
    it({
      name: 'equivalence: unmatched and doubled separators are not hidden',
      fn: async function unmatchedSeparators() {
        expect(await isHidden('foo//bar',),).toBe(false,);
        expect(await isHidden('src\\',),).toBe(false,);
        expect(await isHidden(String.raw`\\`,),).toBe(false,);
      },
    },),
    it({
      name: 'equivalence: both separator kinds in one path',
      fn: async function bothSeparators() {
        expect(await isHidden(String.raw`a/b\.c`,),).toBe(true,);
        expect(await isHidden(String.raw`a\b\c`,),).toBe(false,);
      },
    },),
    it({
      name: 'equivalence: dot-only and dotdot segments',
      fn: async function dotSegments() {
        expect(await isHidden('.',),).toBe(false,);
        expect(await isHidden('..',),).toBe(false,);
        expect(await isHidden('...',),).toBe(false,);
        expect(await isHidden('./foo.ts',),).toBe(false,);
        expect(await isHidden('a/..',),).toBe(false,);
        expect(await isHidden('a/../b',),).toBe(false,);
      },
    },),
    it({
      name: 'equivalence: long repeated separator run stays correct, linear, and stack-safe',
      fn: async function longRepeatedRun() {
        const sepCount = 50_000;
        const prefix = 'a/'.repeat(sepCount,);
        const longNoHit = `${prefix}b`;
        const longHit = `${prefix}.h`;
        expect(await isHidden(longNoHit,),).toBe(false,);
        expect(await isHidden(longHit,),).toBe(true,);
      },
    },),
  ],
},);
