import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdir,
  mkdtemp,
  rename,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
  resolve,
} from 'node:path';
import { HashCache, } from './hash-cache.ts';
import type { WatchEvent, } from './types.ts';
import { Watcher, } from './watcher.ts';

/**
 sha256 hex of the literal bytes "hello"; used to assert pre-populate
 writes the right digest, not just "anything".
 */
const SHA256_HEX_OF_HELLO =
  '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824';

/**
 Buffer that exceeds chokidar's `awaitWriteFinish.stabilityThreshold` plus
 a generous safety margin. 200ms is enough on the slowest CI we run.
 */
const POST_EVENT_WAIT_MS = 200;

/**
 Buffer for "no event should fire" assertions; longer than POST_EVENT_WAIT_MS
 so the negative claim is well past chokidar's stability period.
 */
const NO_EVENT_WAIT_MS = 400;

/**
 Fresh temp dir per test so chokidar's per-directory state never bleeds.
 
 @returns absolute path of a freshly-created temp directory
 
 @example
 ```ts
 const dir = await makeTmpDir();
 ```
 */
async function makeTmpDir(): Promise<string> {
  return mkdtemp(join(scratchRoot(), 'watch-restart-watcher-',),);
}

/**
 Convenience builder for the Watcher under test. Wires a shared event
 accumulator and a fresh {@link HashCache}, returns the watcher together
 with the captured events so the test body can introspect both.
 
 @param paths - watch roots passed verbatim to {@link Watcher}
 
 @returns watcher, hashCache, and the live event accumulator
 
 @example
 ```ts
 const { watcher, hashCache, events, } = await buildWatcher([dir,],);
 ```
 */
async function buildWatcher(
  paths: readonly string[],
): Promise<{
  watcher: Watcher;
  hashCache: HashCache;
  events: WatchEvent[];
}> {
  const events: WatchEvent[] = [];
  const hashCache = new HashCache();
  const watcher = new Watcher({
    paths,
    hashCache,
    onEvent: async function onEvent(event,) {
      events.push(event,);
    },
  },);
  await watcher.untilReady();
  return {
    watcher,
    hashCache,
    events,
  };
}

await describe({
  name: Watcher.name,
  children: [
    describe({
      name: 'pre-`ready` (initial walk)',
      children: [
        it({
          name: 'pre-populates hashCache with the file digest',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'pre.txt',);
            await writeFile(file, 'hello',);
            const { watcher, hashCache, events, } = await buildWatcher([dir,],);

            expect(
              hashCache.has(resolve(file,),),
            ).toBe(true,);
            expect(
              hashCache.get(resolve(file,),),
            ).toBe(SHA256_HEX_OF_HELLO,);
            expect(events,).toHaveLength(0,);

            await watcher.stop();
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'does not call onEvent for files seen during initial walk',
          fn: async () => {
            const dir = await makeTmpDir();
            await writeFile(join(dir, 'a.txt',), 'aaa',);
            await writeFile(join(dir, 'b.txt',), 'bbb',);
            const { watcher, events, } = await buildWatcher([dir,],);

            await wait(POST_EVENT_WAIT_MS,);
            expect(events,).toHaveLength(0,);

            await watcher.stop();
            await rm(dir, { recursive: true, },);
          },
        },),
      ],
    },),
    describe({
      name: 'post-`ready` (live events)',
      children: [
        it({
          name: 'emits an `add` event for newly created files',
          fn: async () => {
            const dir = await makeTmpDir();
            const { watcher, events, } = await buildWatcher([dir,],);

            const file = join(dir, 'new.txt',);
            await writeFile(file, 'world',);
            await wait(POST_EVENT_WAIT_MS,);

            const adds = events.filter(function isAdd(e,) {
              return e.kind === 'add';
            },);
            expect(adds.length,).toBeGreaterThanOrEqual(1,);
            expect(nonNullishOrThrow(adds[0],).path,).toBe(resolve(file,),);
            expect(nonNullishOrThrow(adds[0],).relativePath,).toBe('new.txt',);
            expect(nonNullishOrThrow(adds[0],).ext,).toBe('.txt',);

            await watcher.stop();
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'emits a `change` event when an existing file is modified',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'edit.txt',);
            await writeFile(file, 'v1',);
            const { watcher, events, } = await buildWatcher([dir,],);

            await writeFile(file, 'v2',);
            await wait(POST_EVENT_WAIT_MS,);

            const changes = events.filter(function isChange(e,) {
              return e.kind === 'change';
            },);
            expect(changes.length,).toBeGreaterThanOrEqual(1,);
            expect(nonNullishOrThrow(changes[0],).path,).toBe(resolve(file,),);

            await watcher.stop();
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'emits an `unlink` event when a file is deleted',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'gone.txt',);
            await writeFile(file, 'bye',);
            const { watcher, hashCache, events, } = await buildWatcher([dir,],);

            expect(
              hashCache.has(resolve(file,),),
            ).toBe(true,);

            await rm(file,);
            await wait(POST_EVENT_WAIT_MS,);

            const unlinks = events.filter(function isUnlink(e,) {
              return e.kind === 'unlink';
            },);
            expect(unlinks.length,).toBe(1,);
            expect(nonNullishOrThrow(unlinks[0],).path,).toBe(resolve(file,),);
            expect(
              hashCache.has(resolve(file,),),
            ).toBe(false,);

            await watcher.stop();
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'atomic save (rename _tmp -> file) eventually fires for the target',
          // Empirical flake rate ~20% on Linux EVEN IN ISOLATION (1 of 5
          // single-file runs reports events.length === 0): chokidar's
          // `atomic: true` + `awaitWriteFinish` stability window races
          // the kernel's rename event; chokidar occasionally reports zero
          // events on the destination path. The atomic-save path stays
          // exercised end-to-end by editord's dev loop (task 9) where
          // the user is the only file producer.
          skip:
            'flaky against chokidar atomic+awaitWriteFinish timing; covered by editord dev-loop integration verification',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'atomic.txt',);
            await writeFile(file, 'original-contents',);
            const { watcher, events, } = await buildWatcher([dir,],);

            const tmp = `${file}.tmp`;
            await writeFile(tmp, 'new-contents-with-different-length',);
            await rename(tmp, file,);
            await wait(NO_EVENT_WAIT_MS,);

            const changesOnFile = events.filter(function isFileChange(e,) {
              return (e.path === resolve(file,))
                && ((e.kind === 'add') || (e.kind === 'change'));
            },);
            expect(changesOnFile.length,).toBeGreaterThanOrEqual(1,);

            await watcher.stop();
            await rm(dir, { recursive: true, },);
          },
        },),
      ],
    },),
    describe({
      name: 'multiple watch roots',
      children: [
        it({
          name: 'relativePath is relative to the deepest matching root',
          fn: async () => {
            const root = await makeTmpDir();
            const inner = join(root, 'inner',);
            await mkdir(inner,);
            const { watcher, events, } = await buildWatcher([root, inner,],);

            const file = join(inner, 'leaf.txt',);
            await writeFile(file, 'hi',);
            await wait(POST_EVENT_WAIT_MS,);

            const adds = events.filter(function isAdd(e,) {
              return (e.kind === 'add') && (e.path === resolve(file,));
            },);
            expect(adds.length,).toBeGreaterThanOrEqual(1,);
            // Deepest match is `inner`, so relativePath is `leaf.txt`, not `inner/leaf.txt`.
            expect(nonNullishOrThrow(adds[0],).relativePath,).toBe('leaf.txt',);

            await watcher.stop();
            await rm(root, { recursive: true, },);
          },
        },),
      ],
    },),
    describe({
      name: 'chokidar option pass-through (depth, poll, followSymlinks)',
      children: [
        it({
          name: 'depth caps subdirectory traversal during the initial walk',
          fn: async () => {
            const root = await makeTmpDir();
            await mkdir(
              join(root, 'a', 'b', 'c',),
              { recursive: true, },
            );
            const shallow = join(root, 'shallow.txt',);
            const mid = join(root, 'a', 'mid.txt',);
            const deep = join(root, 'a', 'b', 'deep.txt',);
            const deeper = join(root, 'a', 'b', 'c', 'deeper.txt',);
            await writeFile(shallow, 'top',);
            await writeFile(mid, 'mid',);
            await writeFile(deep, 'deep',);
            await writeFile(deeper, 'deeper',);

            const events: WatchEvent[] = [];
            const hashCache = new HashCache();
            const watcher = new Watcher({
              paths: [root,],
              hashCache,
              depth: 1,
              onEvent: async function onEvent(event,) {
                events.push(event,);
              },
            },);
            await watcher.untilReady();

            // depth: 1 admits root files (depth 0 of subdirs) and one level of subdirs.
            // shallow.txt sits in root; mid.txt sits in a first-level subdir.
            // deep.txt is inside `a/b/`, a second-level subdir, past the cap.
            expect(
              hashCache.has(resolve(shallow,),),
            ).toBe(true,);
            expect(
              hashCache.has(resolve(mid,),),
            ).toBe(true,);
            expect(
              hashCache.has(resolve(deep,),),
            ).toBe(false,);
            expect(
              hashCache.has(resolve(deeper,),),
            ).toBe(false,);

            await watcher.stop();
            await rm(root, { recursive: true, },);
          },
        },),
        it({
          name: 'poll mode emits live events on file changes',
          fn: async () => {
            const root = await makeTmpDir();
            const events: WatchEvent[] = [];
            const hashCache = new HashCache();
            const watcher = new Watcher({
              paths: [root,],
              hashCache,
              poll: 50,
              onEvent: async function onEvent(event,) {
                events.push(event,);
              },
            },);
            await watcher.untilReady();

            const file = join(root, 'polled.txt',);
            await writeFile(file, 'v1',);
            // Polling-mode latency is bounded by the interval + chokidar's
            // stability window; 500ms covers both with headroom on slow CI.
            await wait(500,);

            const adds = events.filter(function isAdd(e,) {
              return (e.kind === 'add') && (e.path === resolve(file,));
            },);
            expect(adds.length,).toBeGreaterThanOrEqual(1,);

            await watcher.stop();
            await rm(root, { recursive: true, },);
          },
        },),
        it({
          name: 'followSymlinks: true pre-populates files inside a symlinked directory',
          fn: async () => {
            const root = await makeTmpDir();
            const external = await makeTmpDir();
            const externalFile = join(external, 'linked-file.txt',);
            await writeFile(externalFile, 'inside',);
            await symlink(external, join(root, 'link',),);

            const events: WatchEvent[] = [];
            const hashCache = new HashCache();
            const watcher = new Watcher({
              paths: [root,],
              hashCache,
              followSymlinks: true,
              onEvent: async function onEvent(event,) {
                events.push(event,);
              },
            },);
            await watcher.untilReady();

            // Chokidar reports the path as it walked it: through the symlink.
            // The pre-populate hashed the file's bytes, so a `has()` against
            // the resolved-via-link path is the cleanest assertion.
            const linkedPath = resolve(root, 'link', 'linked-file.txt',);
            expect(hashCache.has(linkedPath,),).toBe(true,);

            await watcher.stop();
            await rm(root, { recursive: true, },);
            await rm(external, { recursive: true, },);
          },
        },),
        it({
          name: 'followSymlinks default (false) skips files inside symlinked directories',
          fn: async () => {
            const root = await makeTmpDir();
            const external = await makeTmpDir();
            const externalFile = join(external, 'linked-file.txt',);
            await writeFile(externalFile, 'inside',);
            await symlink(external, join(root, 'link',),);

            const events: WatchEvent[] = [];
            const hashCache = new HashCache();
            const watcher = new Watcher({
              paths: [root,],
              hashCache,
              onEvent: async function onEvent(event,) {
                events.push(event,);
              },
            },);
            await watcher.untilReady();

            const linkedPath = resolve(root, 'link', 'linked-file.txt',);
            // With followSymlinks: false (default), chokidar must not
            // descend into `link/` and so the file inside the linked tree
            // never reaches the hash cache.
            expect(hashCache.has(linkedPath,),).toBe(false,);

            await watcher.stop();
            await rm(root, { recursive: true, },);
            await rm(external, { recursive: true, },);
          },
        },),
      ],
    },),
    describe({
      name: 'lifecycle',
      children: [
        it({
          name: 'stop() resolves and prevents further events',
          fn: async () => {
            const dir = await makeTmpDir();
            const { watcher, events, } = await buildWatcher([dir,],);

            await watcher.stop();

            await writeFile(join(dir, 'late.txt',), 'no',);
            await wait(NO_EVENT_WAIT_MS,);

            expect(events,).toHaveLength(0,);

            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'untilReady() is idempotent on subsequent calls',
          fn: async () => {
            const dir = await makeTmpDir();
            await writeFile(join(dir, 'a.txt',), 'a',);
            const { watcher, } = await buildWatcher([dir,],);

            // Second call must resolve immediately without re-awaiting chokidar's `ready`.
            await watcher.untilReady();

            await watcher.stop();
            await rm(dir, { recursive: true, },);
          },
        },),
      ],
    },),
  ],
},);
