/**
 Both module systems must reach the same surface.

 `import { Readable } from 'readable-stream'` is the ESM spelling of the CommonJS destructuring the shim
 already serves, and Node builds a CommonJS module's ESM named exports from static
 `module.exports.<name> =` writes alone. A property the entry copies in a loop therefore never reaches the
 module namespace, so a static import of that property fails at link time while `require` answers it.
 The cases read the published entry as a namespace, link one static import per name `node:stream` itself
 answers to inside a child process, and compare every binding against `node:stream`'s own, so a name the
 entry stops declaring fails here instead of in a consumer's first import.

 @module
 */

import { execFile, } from 'node:child_process';
import { readFile, } from 'node:fs/promises';
import { createRequire, } from 'node:module';
import { promisify, } from 'node:util';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/** Published entry point, at the URL a consumer's `import 'readable-stream'` resolves to. */
const ENTRY_URL = new URL('index.cjs', import.meta.url,);

/** CommonJS loader anchored at this test, so the ESM surface is compared against a plain `require` of the entry. */
const requireFromHere = createRequire(import.meta.url,);

/** Ceiling for a child process that starts Node and links `node:stream`; a hang must fail the case, not the suite. */
const CHILD_DEADLINE_MS = 60_000;

/** Text the linking child prints once its import statement resolved, so a silent early exit cannot pass the case. */
const LINKED_MARKER = 'linked';

/** Subprocess adapter that keeps Node's declared `execFile` promise contract. */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/** `execFile` as a promise, so a case awaits the child's output instead of a callback. */
const execFileAsync = promisifyExecFile(execFile,);

/** A module's exports read by name, which no module namespace type allows without an index signature. */
type NamedSurface = Record<string, unknown>;

/** `node:stream`, the module the entry re-exports and therefore the surface it has to answer to. */
const NODE_STREAM = await import('node:stream',);

/** Names `node:stream` answers to ESM importers, minus the default binding every module also carries. */
const NODE_STREAM_NAMES = Object.keys(NODE_STREAM,)
  .filter((name) => name !== 'default',)
  .toSorted();

/** `node:stream`'s CommonJS module object, the value the shim's export mirrors and the `Stream` alias points at. */
const NODE_STREAM_SURFACE = NODE_STREAM.default as unknown as NamedSurface;

/** The entry as Node's ESM loader builds it from the entry's static writes. */
const ENTRY_NAMESPACE = (await import('./index.cjs',)) as unknown as NamedSurface;

/** The entry as `require` returns it, the shape every CommonJS consumer of `readable-stream` destructures. */
const REQUIRED_ENTRY = requireFromHere('./index.cjs',) as unknown as NamedSurface;

/** The package manifest, whose `exports` map names every path a consumer can require by package name. */
const MANIFEST = JSON.parse(
  await readFile(new URL('package.json', import.meta.url,), 'utf8',),
) as { name: string; exports: Record<string, unknown> };

/**
 Deep paths the manifest exports, which are the `lib/_stream_*.js` subpaths upstream `readable-stream` served and
 `winston-transport`'s `modern.js` reached.
 */
const DEEP_PATH_KEYS = Object.keys(MANIFEST.exports,)
  .filter((key) => key.startsWith('./lib/',),)
  .toSorted();

/**
 Class each exported deep path must answer with, keyed by the subpath consumers write.

 Spelling differs from the class name for `passthrough`, so this table cannot be derived from the keys.
 */
const DEEP_PATH_CLASSES: Record<string, unknown> = {
  './lib/_stream_writable.js': NODE_STREAM_SURFACE.Writable,
  './lib/_stream_readable.js': NODE_STREAM_SURFACE.Readable,
  './lib/_stream_transform.js': NODE_STREAM_SURFACE.Transform,
  './lib/_stream_duplex.js': NODE_STREAM_SURFACE.Duplex,
  './lib/_stream_passthrough.js': NODE_STREAM_SURFACE.PassThrough,
};

/**
 Source a child process runs: one static import of every name, which Node resolves at link time, before any
 statement in the source runs.

 Derived from `node:stream`'s own namespace so no hand-written list can fall behind what a consumer imports.
 */
const LINKING_SOURCE = [
  `import { ${NODE_STREAM_NAMES.join(', ',)} } from ${JSON.stringify(ENTRY_URL.href,)};`,
  `console.log(${JSON.stringify(LINKED_MARKER,)});`,
].join('\n',);

await describe({
  name: '',
  children: [
    describe({
      name: 'ESM named exports',
      children: [
        it({
          name: 'declares every name node:stream answers to, so a static import of any of them resolves',
          fn: async () => {
            const undeclared = NODE_STREAM_NAMES.filter((name) => !(name in ENTRY_NAMESPACE),);

            expect(undeclared,).toEqual([],);
          },
        },),
        it({
          name: 'binds each declared name to the value node:stream exposes under it',
          fn: async () => {
            const diverged = NODE_STREAM_NAMES.filter(
              (name) => ENTRY_NAMESPACE[name] !== NODE_STREAM_SURFACE[name],
            );

            expect(diverged,).toEqual([],);
          },
        },),
        it({
          name: 'defaults to the Readable constructor the README documents, through both loaders',
          fn: async () => {
            expect(ENTRY_NAMESPACE.default,).toBe(NODE_STREAM_SURFACE.Readable,);
            expect(REQUIRED_ENTRY,).toBe(NODE_STREAM_SURFACE.Readable,);
          },
        },),
      ],
    },),
    describe({
      name: 'link time',
      children: [
        it({
          name: 'links one static named import per name node:stream answers to',
          fn: async () => {
            const { stdout, } = await execFileAsync(
              process.execPath,
              ['--input-type=module', '--eval', LINKING_SOURCE,],
              { timeout: CHILD_DEADLINE_MS, },
            );

            expect(stdout.trim(),).toBe(LINKED_MARKER,);
          },
        },),
      ],
    },),
    describe({
      name: 'CommonJS surface',
      children: [
        it({
          name: 'keeps the destructured reads, the Stream module alias, and the constructable default',
          fn: async () => {
            expect(REQUIRED_ENTRY.Writable,).toBe(NODE_STREAM_SURFACE.Writable,);
            expect(REQUIRED_ENTRY.pipeline,).toBe(NODE_STREAM_SURFACE.pipeline,);
            expect(REQUIRED_ENTRY.Stream,).toBe(NODE_STREAM_SURFACE.Stream,);

            const ReadableConstructor = REQUIRED_ENTRY as unknown as new(options?: object,) => object;

            expect(new ReadableConstructor({},),).toBeInstanceOf(NODE_STREAM.Readable,);
          },
        },),
      ],
    },),
    describe({
      name: 'deep imports',
      children: [
        it({
          name: 'serves every lib/_stream_*.js subpath the manifest exports, with the class node:stream has',
          fn: async () => {
            expect(Object.keys(DEEP_PATH_CLASSES,).toSorted(),).toEqual(DEEP_PATH_KEYS,);
            for (const [subpath, expected, ] of Object.entries(DEEP_PATH_CLASSES,)) {
              const resolved: unknown = requireFromHere(`${MANIFEST.name}/${subpath.slice(2,)}`,);

              expect(resolved,).toBe(expected,);
            }
          },
        },),
      ],
    },),
  ],
},);
