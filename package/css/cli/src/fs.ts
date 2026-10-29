/**
 Environment-adaptive file reading.
 
 Checks the in-memory {@link fsRegistry} first (browser path),
 then falls back to `node:fs` when running under Node/Bun.
 This lets the same build pipeline work on the server (disk I/O)
 and in the browser (pre-populated memory Map).
 */
import type NodeFs from 'node:fs';
import { fsRegistry, } from './fs-registry.ts';

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
 {@link readCssFileSync} and {@link existsSync} cannot await.
 */
const nodeFs: typeof NodeFs | typeof NO_NODE_FS = hasNodeFs
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- dynamic import lacks static type
  ? await import(nodeFsSpecifier) as typeof NodeFs
  : NO_NODE_FS;

/**
 Reads a CSS file by absolute path.
 Looks in {@link fsRegistry} first; falls back to `node:fs/promises`
 when running server-side.
 
 @param absolutePath - Absolute file path to read
 
 @returns File contents as a UTF-8 string
 
 @throws When the file is missing from both the registry and disk
 
 @example
 ```ts
 const css = await readCssFile('/project/src/styles.css');
 ```
 */
export async function readCssFile(absolutePath: string,): Promise<string> {
  /**
   Registry hit shortcuts the filesystem fallback for browser builds.
   */
  const cached = fsRegistry.get(absolutePath,);
  if (cached !== undefined)
    return cached;

  if (nodeFs !== NO_NODE_FS) {
    /**
     Dynamic import keeps `node:fs/promises` out of browser bundles.
     */
    const { readFile, } = await import('node:fs/promises');
    return readFile(
      absolutePath,
      'utf8',
    );
  }

  throw new Error(
    `File not found in registry and no filesystem available: ${absolutePath}`,
  );
}

/**
 Synchronous variant of {@link readCssFile} for contexts that cannot be async
 (e.g. inside PostCSS plugin walkers that don't support promises).
 
 @param absolutePath - Absolute file path to read
 
 @returns File contents as a UTF-8 string
 
 @throws When the file is missing from both the registry and disk
 
 @example
 ```ts
 const css = readCssFileSync('/project/src/styles.css');
 ```
 */
export function readCssFileSync(absolutePath: string,): string {
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
