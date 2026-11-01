import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { HashCache, } from '../hash-cache.ts';
import type {
  WatchCtx,
  WatchEvent,
} from '../types.ts';
import { gitignoreFilter, } from './gitignore.ts';

/**
 Logger root for watch-restart after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: defaultLogger, },);
 ```
 */
const defaultLogger = tagged({ tag: 'watch-restart', },);

/**
 Builds a minimal {@link WatchCtx}; gitignoreFilter ignores everything
 but `event.relativePath`.
 
 @returns context object suitable for handing to gitignoreFilter
 
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
 Builds a {@link WatchEvent} from overrides; defaults match a `change`
 to `file.ts`.
 
 @param overrides - partial event fields to merge over the default
 
 @returns fully-populated event
 
 @example
 ```ts
 const event = makeEvent({ relativePath: 'dist/build.js', },);
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
 Creates a fresh temp directory for a single test.
 
 @returns absolute path to the new directory
 
 @example
 ```ts
 const dir = await makeTmpDir();
 ```
 */
async function makeTmpDir(): Promise<string> {
  return mkdtemp(join(scratchRoot(), 'watch-restart-gitignore-',),);
}

await describe({
  name: gitignoreFilter.name,
  children: [
    it({
      name: 'no .gitignore at root: vacuous pass-all',
      fn: async function vacuousPassAll() {
        const dir = await makeTmpDir();
        const filter = await gitignoreFilter({ roots: [dir,], },);
        const passed = await filter({
          event: makeEvent({ relativePath: 'anything/at/all.ts', },),
          ctx: makeCtx(),
        },);
        expect(passed,).toBe(true,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'directory pattern rejects events under the directory',
      fn: async function dirPatternRejects() {
        const dir = await makeTmpDir();
        await writeFile(join(dir, '.gitignore',), 'dist/\nnode_modules/\n',);

        const filter = await gitignoreFilter({ roots: [dir,], },);
        const distFile = await filter({
          event: makeEvent({ relativePath: 'dist/build.js', },),
          ctx: makeCtx(),
        },);
        const nestedDist = await filter({
          event: makeEvent({ relativePath: 'dist/nested/inner.js', },),
          ctx: makeCtx(),
        },);
        const nodeMod = await filter({
          event: makeEvent({ relativePath: 'node_modules/pkg/index.js', },),
          ctx: makeCtx(),
        },);
        const sourceFile = await filter({
          event: makeEvent({ relativePath: 'src/index.ts', },),
          ctx: makeCtx(),
        },);

        expect(distFile,).toBe(false,);
        expect(nestedDist,).toBe(false,);
        expect(nodeMod,).toBe(false,);
        expect(sourceFile,).toBe(true,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'glob pattern matches files by name',
      fn: async function globPatternRejects() {
        const dir = await makeTmpDir();
        await writeFile(join(dir, '.gitignore',), '*.log\n*.tmp\n',);

        const filter = await gitignoreFilter({ roots: [dir,], },);
        const logFile = await filter({
          event: makeEvent({ relativePath: 'server.log', },),
          ctx: makeCtx(),
        },);
        const nestedLog = await filter({
          event: makeEvent({ relativePath: 'src/server.log', },),
          ctx: makeCtx(),
        },);
        const tsFile = await filter({
          event: makeEvent({ relativePath: 'src/server.ts', },),
          ctx: makeCtx(),
        },);
        expect(logFile,).toBe(false,);
        expect(nestedLog,).toBe(false,);
        expect(tsFile,).toBe(true,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'negation pattern reinstates a previously-ignored path',
      fn: async function negationReinstates() {
        const dir = await makeTmpDir();
        // Pattern `dist/*` (not `dist/`) so the parent directory is not
        // categorically excluded; git itself rejects re-including a file
        // whose parent directory was excluded, and the `ignore` library
        // mirrors that.  Documents the live constraint: negation only
        // works at the same nesting level as the exclude.
        await writeFile(
          join(dir, '.gitignore',),
          'dist/*\n!dist/important.js\n',
        );
        const filter = await gitignoreFilter({ roots: [dir,], },);
        const ignored = await filter({
          event: makeEvent({ relativePath: 'dist/build.js', },),
          ctx: makeCtx(),
        },);
        const reinstated = await filter({
          event: makeEvent({ relativePath: 'dist/important.js', },),
          ctx: makeCtx(),
        },);
        expect(ignored,).toBe(false,);
        expect(reinstated,).toBe(true,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'extraFiles patterns AND with .gitignore patterns',
      fn: async function extraFilesAnd() {
        const dir = await makeTmpDir();
        await writeFile(join(dir, '.gitignore',), 'dist/\n',);
        const watchIgnorePath = join(dir, '.watchignore',);
        await writeFile(watchIgnorePath, 'coverage/\n',);

        const filter = await gitignoreFilter({
          roots: [dir,],
          extraFiles: [watchIgnorePath,],
        },);
        const fromGitignore = await filter({
          event: makeEvent({ relativePath: 'dist/build.js', },),
          ctx: makeCtx(),
        },);
        const fromExtra = await filter({
          event: makeEvent({ relativePath: 'coverage/index.html', },),
          ctx: makeCtx(),
        },);
        const neither = await filter({
          event: makeEvent({ relativePath: 'src/index.ts', },),
          ctx: makeCtx(),
        },);
        expect(fromGitignore,).toBe(false,);
        expect(fromExtra,).toBe(false,);
        expect(neither,).toBe(true,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'empty roots + extraFiles only loads from the extras',
      fn: async function emptyRootsOnlyExtras() {
        const dir = await makeTmpDir();
        const watchIgnorePath = join(dir, '.watchignore',);
        await writeFile(watchIgnorePath, 'tmp/\n',);

        const filter = await gitignoreFilter({
          roots: [],
          extraFiles: [watchIgnorePath,],
        },);
        const ignored = await filter({
          event: makeEvent({ relativePath: 'tmp/cache.bin', },),
          ctx: makeCtx(),
        },);
        const passes = await filter({
          event: makeEvent({ relativePath: 'src/index.ts', },),
          ctx: makeCtx(),
        },);
        expect(ignored,).toBe(false,);
        expect(passes,).toBe(true,);
        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'missing extraFile (ENOENT) collapses to no-op without throwing',
      fn: async function missingExtraNoThrow() {
        const dir = await makeTmpDir();
        const missing = join(dir, 'does-not-exist.watchignore',);

        const filter = await gitignoreFilter({
          roots: [dir,],
          extraFiles: [missing,],
        },);
        const passed = await filter({
          event: makeEvent({ relativePath: 'src/index.ts', },),
          ctx: makeCtx(),
        },);
        expect(passed,).toBe(true,);
        await rm(dir, { recursive: true, },);
      },
    },),
  ],
},);
