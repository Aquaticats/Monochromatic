/**
 Content identities and paths for persistent effect-summary cache.
 
 @module
 */

import { createHash, } from 'node:crypto';
import {
  type Dirent,
  existsSync,
  readFileSync,
  readdirSync,
} from 'node:fs';
import {
  dirname,
  extname,
  join,
} from 'node:path';
import { fileURLToPath, } from 'node:url';

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { version as typescriptVersion, } from 'typescript';

import { ancestorDirectories, } from './ancestor-directories.ts';

/**
 Persistent cache schema identity.
 
 Schema 4 removes documented-uncertainty state because authored contracts
 cannot discharge unresolved implementation effects.
 
 Schema 5 keys every effect by slot rather than by parameter and persists the slot
 ownership beside them, because the numbers mean nothing without it.
 
 Schema 6 persists callable identities deliberately omitted after direct-summary failures.
 Without them a warm process rejects caller edges that the cold process accepted as
 fail-closed omissions.
 
 Schema 7 adds bounded omission reason category,
 so restored-cache warnings retain why coverage was narrowed.
 
 Schema 8 retains every distinct bounded reason category for a source,
 including the known TypeScript tuple serializer panic.
 
 Schema 9 adds `sharedKeyDigest` to the envelope of an entry shared across projects,
 and addresses such an entry by file and key digest instead of by project.
 
 Schema 10 adds `externalOmissions`,
 the omissions met inside installed package implementations while the entry's source was scanned,
 so a restore of the source reports what a fresh scan reported.
 */
export const EFFECT_CACHE_SCHEMA = 10;

/**
 Process memo for analyzer implementation digest.
 */
const analyzerDigestMemo = new Map<'digest', string>();

/**
 Hashes exact UTF-8 text through SHA-256.
 
 @param text - Text whose content identity is required.
 
 @returns lowercase hexadecimal digest.
 
 @example
 ```ts
 contentDigest('source');
 ```
 */
export function contentDigest(text: string,): string {
  return createHash('sha256',)
    .update(text,)
    .digest('hex',);
}

/**
 Finds package root containing current analyzer module.
 
 @param modulePath - Analyzer source or bundled module path.
 
 @returns nearest ancestor carrying package manifest.
 
 @throws Error when package root cannot be found.
 */
function analyzerPackageRoot(modulePath: string,): string {
  for (const directory of ancestorDirectories(dirname(modulePath,),)) {
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- Oxlint rule construction is synchronous and computes this digest once per process.
    if (existsSync(join(
      directory,
      'package.json',
    )))
      return directory;
  }
  throw new Error(`Cannot find analyzer package root from ${modulePath}.`,);
}

/**
 Collects analyzer TypeScript source paths in deterministic order.
 
 @param directory - Directory whose source descendants are required.
 
 @returns sorted TypeScript source paths.
 */
function analyzerSourcePaths(directory: string,): readonly string[] {
  /* oxlint-disable no-restricted-syntax/no-sync -- Oxlint rule construction is synchronous and scans package source once per process. */
  /**
   Foreign directory entries returned by Node filesystem boundary.
   */
  const entries: ForeignBorrowed<Dirent[]> = readdirSync(
    directory,
    { withFileTypes: true, },
  );
  /* oxlint-enable no-restricted-syntax/no-sync */
  /**
   Collected source paths for current structural subtree.
   */
  const paths: string[] = [];
  for (const entry of entries) {
    /**
     Absolute child path.
     */
    const path = join(
      directory,
      entry.name,
    );
    if (entry.isDirectory()) {
      paths.push(...analyzerSourcePaths(path,),);
      continue;
    }
    if (entry.isFile() && (extname(entry.name,) === '.ts'))
      paths.push(path,);
  }
  return paths.toSorted();
}

/**
 Computes digest for analyzer implementation and TypeScript runtime.
 
 Source execution hashes every package source file.
 Published bundled execution hashes the current bundle.
 
 @returns process-stable analyzer identity.
 
 @example
 ```ts
 const identity = analyzerDigest();
 ```
 */
export function analyzerDigest(): string {
  /**
   Prior implementation digest computed in this process.
   */
  const memoized = analyzerDigestMemo.get('digest',);
  if (memoized !== undefined)
    return memoized;
  /**
   Current source module or published bundle path.
   */
  const modulePath = import.meta.filename;
  /**
   Analyzer package root found from current module.
   */
  const packageRoot = analyzerPackageRoot(modulePath,);
  /**
   Whether workspace `/ts` execution loads unbundled analyzer modules.
   */
  const isSourceExecution = extname(modulePath,) === '.ts';
  /**
   Source roots contributing semantic behavior during `/ts` execution.
   */
  const sourceRoots = isSourceExecution
    ? [
      join(
        packageRoot,
        'src',
      ),
      join(
        analyzerPackageRoot(
          fileURLToPath(import.meta.resolve(
            '@monochromatic-dev/oxlint-plugin-shared/ts',
          ),),
        ),
        'src',
      ),
    ]
    : [];
  /**
   Source modules or published bundle contributing analyzer behavior.
   */
  const implementationPaths = isSourceExecution
    ? sourceRoots.flatMap(analyzerSourcePaths,)
    : [modulePath,];
  /**
   Streaming digest over schema,
   compiler version,
   paths,
   and exact implementation bytes.
   */
  const digest = createHash('sha256',)
    .update(String(EFFECT_CACHE_SCHEMA,),)
    .update('\0',)
    .update(typescriptVersion,);
  implementationPaths.forEach(function hashImplementation(path,): void {
    digest
      .update('\0',)
      .update(path.slice(packageRoot.length,),)
      .update('\0',);
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- Oxlint rule construction is synchronous and hashes each analyzer file once per process.
    digest.update(readFileSync(path,),);
  },);
  /**
   Final implementation digest retained by process memo.
   */
  const result = digest.digest('hex',);
  analyzerDigestMemo.set(
    'digest',
    result,
  );
  return result;
}

/**
 Finds dependency-root ancestor for configured project.
 
 @param projectKey - Configured TypeScript project path.
 
 @returns nearest lockfile ancestor or project directory.
 */
function dependencyRoot(projectKey: string,): string {
  /**
   Original project directory used when no lockfile exists.
   */
  const projectDirectory = dirname(projectKey,);
  for (const directory of ancestorDirectories(projectDirectory,)) {
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- Synchronous rule needs deterministic cache root before reporting diagnostics.
    if (existsSync(join(
      directory,
      'pnpm-lock.yaml',
    )))
      return directory;
  }
  return projectDirectory;
}

/**
 Resolves persistent cache root for one configured project.
 
 @param projectKey - Configured TypeScript project path.
 
 @param override - Explicit disposable root used by tests.
 
 @returns cache directory containing project entries.
 
 @example
 ```ts
 effectCacheRoot({ projectKey: '/repo/tsconfig.json' });
 ```
 */
export function effectCacheRoot({
  projectKey,
  override,
}: {
  readonly projectKey: string;
  readonly override?: string;
},): string {
  if (override !== undefined)
    return override;
  return join(
    dependencyRoot(projectKey,),
    'node_modules',
    '.cache',
    'prefer-readonly-parameter-type',
  );
}
