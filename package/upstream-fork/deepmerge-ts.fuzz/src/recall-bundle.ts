/**
 Bundles and controls for the in-container historical-recall run
 (`./recall-runtime.ts`): build a ledger row's buggy and fixed bundles
 with the image's esbuild (the v8.0.2 tree with the fault re-applied, or the
 fix commit's parent tree and the fix commit's tree), and run the row's
 control on a bundle.

 Upstream trees are mounted read-only at `/trees/<name>/src`, output at
 `/out`.

 @module
 */

import {
  cp,
  mkdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { createRequire, } from 'node:module';
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import { applyEdits, } from './recall-edit.ts';
import type { RuntimeBug, } from './recall-ledger.ts';
import type { DeepmergeTarget, } from './target.ts';

/**
 Read-only mount of upstream source trees.
 */
export const TREES = '/trees';

/**
 Writable output mount.
 */
export const OUT = '/out';

/**
 Tree every transplant starts from, and the fixed side of every transplant.
 */
export const BASELINE_TREE = 'v8.0.2';

/**
 Error for a recall step that cannot produce trustworthy evidence.
 */
export class RecallRunError extends Error {
  /**
   @param message - What failed.
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'RecallRunError';
  }
}

/**
 The part of esbuild's API the run calls.
 */
type Esbuild = {
  readonly build: (options: Readonly<Record<string, unknown>>,) => Promise<unknown>;
};

/**
 Whether a loaded module exposes esbuild's `build`.

 @param value - Module from the image's install.

 @returns True when `build` is a function.

 @example
 ```ts
 isEsbuild(require('esbuild',),);
 ```
 */
function isEsbuild(value: unknown,): value is Esbuild {
  return ((typeof value) === 'object') && (value !== null)
    && ((typeof Reflect.get(
      value,
      'build',
    )) === 'function');
}

/**
 Bundle a source tree's `index.ts` into one ES module with the image's esbuild.

 @param srcDir - Directory holding the deepmerge-ts source.

 @param outfile - Bundle path.

 @throws {@link RecallRunError} When the image has no usable esbuild.

 @example
 ```ts
 await bundle({ outfile: '/out/bundles/baseline.mjs', srcDir: '/trees/v8.0.2/src', });
 ```
 */
export async function bundle(
  {
    srcDir,
    outfile,
  }: {
    readonly srcDir: string;
    readonly outfile: string;
  },
): Promise<void> {
  /**
   esbuild from the image's pinned install.
   */
  const esbuild: unknown = createRequire('/upstream/package.json',)('esbuild',);
  if (!isEsbuild(esbuild,))
    throw new RecallRunError('esbuild in /upstream has no build function; rebuild the image with mutation:image',);
  await esbuild.build({
    bundle: true,
    entryPoints: [join(
      srcDir,
      'index.ts',
    ),],
    format: 'esm',
    logLevel: 'error',
    outfile,
    platform: 'node',
  },);
}

/**
 Buggy and fixed bundles of one row.
 */
export type BundlePair = {
  readonly buggy: string;
  readonly fixed: string;
};

/**
 Source directory for one side of a `parent` row. Trees older than the
 FastUnsafe entry points get a shim `index.ts` that re-exports the
 default entry points under the FastUnsafe names, so the sidecar's target
 check loads them; both sides get the same shim, so it cancels out.

 @param tree - Tree name under `/trees`.

 @param shim - Whether to add the FastUnsafe aliases.

 @returns Directory whose `index.ts` is the bundle entry.

 @example
 ```ts
 await parentSource({ shim: true, tree: '3984927-parent', });
 ```
 */
async function parentSource(
  {
    tree,
    shim,
  }: {
    readonly tree: string;
    readonly shim: boolean;
  },
): Promise<string> {
  /**
   The tree's own source.
   */
  const src = join(
    TREES,
    tree,
    'src',
  );
  if (!shim)
    return src;
  /**
   Shim directory.
   */
  const dir = join(
    '/tmp/recall/shim',
    tree,
  );
  await mkdir(
    dir,
    { recursive: true, },
  );
  /**
   The tree's entry module.
   */
  const entry = JSON.stringify(join(
    src,
    'index.ts',
  ),);
  await writeFile(
    join(
      dir,
      'index.ts',
    ),
    [
      `export * from ${entry};`,
      `export { deepmerge as deepmergeFastUnsafe, deepmergeCustom as deepmergeFastUnsafeCustom, deepmergeInto as deepmergeIntoFastUnsafe, deepmergeIntoCustom as deepmergeIntoFastUnsafeCustom } from ${entry};`,
      '',
    ].join('\n',),
  );
  return dir;
}

/**
 Build a row's bundles.

 @param bug - Ledger row.

 @returns Bundle paths.

 @example
 ```ts
 const pair = await buildPair(RUNTIME_BUGS[0]);
 ```
 */
export async function buildPair(bug: RuntimeBug,): Promise<BundlePair> {
  /**
   Buggy bundle path.
   */
  const buggy = join(
    OUT,
    'bundles',
    `${bug.id}.mjs`,
  );
  if (bug.source
    .kind
    === 'parent') {
    /**
     Fixed bundle path.
     */
    const fixed = join(
      OUT,
      'bundles',
      `${bug.id}-fixed.mjs`,
    );
    await bundle({
      outfile: buggy,
      srcDir: await parentSource({
        shim: bug.source
          .shimFastUnsafe,
        tree: bug.source
          .buggyTree,
      },),
    },);
    await bundle({
      outfile: fixed,
      srcDir: await parentSource({
        shim: bug.source
          .shimFastUnsafe,
        tree: bug.source
          .fixedTree,
      },),
    },);
    return {
      buggy,
      fixed,
    };
  }
  /**
   Scratch copy the edits land in.
   */
  const scratch = join(
    '/tmp/recall',
    bug.id,
  );
  await rm(
    scratch,
    {
      force: true,
      recursive: true,
    },
  );
  await cp(
    join(
      TREES,
      BASELINE_TREE,
      'src',
    ),
    scratch,
    { recursive: true, },
  );
  /**
   Files the row edits, each once.
   */
  const files = [...new Set(bug.source
    .edits
    .map(function fileOf(edit,) {
      return edit.file;
    },),),];
  /**
   Edits of this row.
   */
  const { edits, } = bug.source;
  await Promise.all(files.map(async function editFile(file,) {
    /**
     Edited file path.
     */
    const path = join(
      scratch,
      file,
    );
    await writeFile(
      path,
      applyEdits({
        edits: edits.filter(function forFile(edit,) {
          return edit.file === file;
        },),
        source: await readFile(
          path,
          'utf8',
        ),
      },),
    );
  },),);
  await bundle({
    outfile: buggy,
    srcDir: scratch,
  },);
  return {
    buggy,
    fixed: join(
      OUT,
      'bundles',
      'baseline.mjs',
    ),
  };
}

/**
 Whether a loaded bundle provides the entry points `reproduces` calls.

 @param value - Module namespace.

 @returns True when every entry point is a function.

 @example
 ```ts
 isLibrary(await import(url));
 ```
 */
function isLibrary(value: unknown,): value is DeepmergeTarget {
  return ((typeof value) === 'object') && (value !== null)
    && [
      'deepmerge',
      'deepmergeCustom',
      'deepmergeFastUnsafe',
      'deepmergeInto',
      'deepmergeIntoCustom',
      'deepmergeIntoFastUnsafe',
    ].every(function exported(name,) {
      return (typeof Reflect.get(
        value,
        name,
      )) === 'function';
    },);
}

/**
 Run a row's control against one bundle.

 @param bug - Ledger row.

 @param path - Bundle path.

 @returns `true`, `false`, or `throw:<ErrorName>`.

 @example
 ```ts
 await controlOn({ bug, path: pair.buggy, }); // 'true'
 ```
 */
export async function controlOn(
  {
    bug,
    path,
  }: {
    readonly bug: RuntimeBug;
    readonly path: string;
  },
): Promise<string> {
  /**
   Bundle namespace.
   */
  const loaded: unknown = await import(pathToFileURL(path,)
    .href);
  if (!isLibrary(loaded,))
    throw new RecallRunError(`${path} does not export the deepmerge-ts entry points`,);
  try {
    return String(bug.reproduces(loaded,),);
  } catch (error) {
    return `throw:${Error.isError(error,) ? error.name : 'unknown'}`;
  }
}
