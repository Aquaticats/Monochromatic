/**
 Environment-adaptive file access.
 
 Checks the in-memory {@link fsRegistry} first (browser path),
 then falls back to `node:fs` when running under Node/Bun.
 This lets the same build pipeline work on the server (disk I/O)
 and in the browser (pre-populated memory Map).
 The module is loaded once, behind a computed specifier, so no bundler
 resolves `node:fs` from a static import site.
 */
import type NodeFs from 'node:fs';
import { dirname, } from '@monochromatic-dev/fs-path-core/ts';
import { fsRegistry, } from './fs-registry.ts';

//region Runtime module loading

/**
 Whether the current runtime has Node-compatible `fs` APIs.
 Bun and Node both set `process.versions.node`.
 */
const hasNodeFs = ((typeof process) !== 'undefined')
  && (process.versions
    ?.node
    !== undefined);

/**
 Computed specifier so browser bundlers cannot statically resolve the import
 */
const nodeFsSpecifier = `node:fs`;

/**
 Sentinel marking runtimes (e.g. browser) where `node:fs` is unavailable.
 A unique Symbol keeps {@link nodeFs} free of a banned nullish union while
 still encoding genuine absence; identity comparison narrows back to the module.
 */
const NO_NODE_FS = Symbol('runtime node:fs import unavailable',);

/**
 Eagerly loaded `node:fs` module, or {@link NO_NODE_FS} in browser.
 Uses a variable specifier so browser bundlers cannot statically resolve the import.
 Loaded eagerly (not lazily) because sync functions like
 {@link readTextFileSync} and {@link existsSync} cannot await.
 */
const nodeFs: typeof NodeFs | typeof NO_NODE_FS = hasNodeFs
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- dynamic import lacks static type
  ? await import(nodeFsSpecifier) as typeof NodeFs
  : NO_NODE_FS;

/**
 Promise-based filesystem API of the module loaded above, or
 {@link NO_NODE_FS} in the browser. Reading it once keeps every call site
 down to a single member access, so none of them names `node:fs` itself.
 */
const nodeFsPromises: typeof NodeFs.promises | typeof NO_NODE_FS = nodeFs
  === NO_NODE_FS
  ? NO_NODE_FS
  : nodeFs.promises;

//endregion Runtime module loading

//region File reading

/**
 Reads a text file by absolute path.
 Checks {@link fsRegistry} first (browser path); otherwise reads from disk
 through `node:fs/promises` when running server-side. Nothing about the
 fallback is CSS-specific: callers pass stylesheets and package manifests
 alike, so the name states the boundary rather than one caller's file type.
 
 @param absolutePath - Absolute file path to read
 
 @returns File contents as a UTF-8 string
 
 @throws When the file is missing from both the registry and disk
 
 @example
 ```ts
 const css = await readTextFile('/project/src/styles.css');
 ```
 */
export async function readTextFile(absolutePath: string,): Promise<string> {
  /**
   Registry hit shortcuts the filesystem fallback for browser builds.
   */
  const cached = fsRegistry.get(absolutePath,);
  if (cached !== undefined)
    return cached;

  if (nodeFsPromises !== NO_NODE_FS) {
    return await nodeFsPromises.readFile(
      absolutePath,
      'utf8',
    );
  }

  throw new Error(
    `File not found in registry and no filesystem available: ${absolutePath}`,
  );
}

/**
 Synchronous variant of {@link readTextFile} for contexts that cannot be async
 (e.g. inside PostCSS plugin walkers that don't support promises).
 Same registry-then-disk order: {@link fsRegistry} first, disk second.
 
 @param absolutePath - Absolute file path to read
 
 @returns File contents as a UTF-8 string
 
 @throws When the file is missing from both the registry and disk
 
 @example
 ```ts
 const css = readTextFileSync('/project/src/styles.css');
 ```
 */
export function readTextFileSync(absolutePath: string,): string {
  /**
   Registry hit shortcuts the filesystem fallback for browser builds.
   */
  const cached = fsRegistry.get(absolutePath,);
  if (cached !== undefined)
    return cached;

  if (nodeFs !== NO_NODE_FS) {
    return nodeFs.readFileSync(
      absolutePath,
      'utf8',
    );
  }

  throw new Error(
    `File not found in registry and no filesystem available: ${absolutePath}`,
  );
}

//endregion File reading

//region File writing

/**
 Writes a text file, creating the directories above it when they are absent.
 Uses the module loaded once in {@link nodeFs}, so no call site names
 `node:fs` as a static import specifier.
 Writes reach only the disk: the browser path cannot write, because
 {@link fsRegistry} is a read-only view of files the host already loaded.
 
 @param absolutePath - Absolute path of the file to write
 
 @param content - Text written verbatim, UTF-8 encoded
 
 @throws When the runtime has no filesystem, or the write itself fails
 
 @example
 ```ts
 await writeTextFileEnsuringParentDirectory({
   absolutePath: '/project/dist/main.css',
   content: '.a { top: 0; }',
 },);
 ```
 */
export async function writeTextFileEnsuringParentDirectory({
  absolutePath,
  content,
}: {
  readonly absolutePath: string;
  readonly content: string;
},): Promise<void> {
  if (nodeFsPromises === NO_NODE_FS)
    throw new Error(`No filesystem available to write: ${absolutePath}`,);

  await nodeFsPromises.mkdir(
    dirname(absolutePath,),
    { recursive: true, },
  );
  await nodeFsPromises.writeFile(
    absolutePath,
    content,
  );
}

//endregion File writing

//region Existence probing

/**
 Checks whether a file exists at the given absolute path.
 Checks {@link fsRegistry} first, then disk.
 
 @param absolutePath - Absolute file path to check
 
 @returns Whether the file exists
 
 @example
 ```ts
 if (existsSync('/project/src/theme.css')) { /* process *\/ }
 ```
 */
export function existsSync(absolutePath: string,): boolean {
  if (fsRegistry.has(absolutePath,))
    return true;

  if (nodeFs !== NO_NODE_FS)
    return nodeFs.existsSync(absolutePath,);

  return false;
}

//endregion Existence probing
