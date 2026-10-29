/**
 Corpus loaders for the fuzz properties.
 
 Two tiers feed the properties:
 
 1. The committed fixture package
    (`package/test-fixture/toml-edit/src/{valid,invalid}`), always loaded and
    fully deterministic, so bounded unit runs stay reproducible.
 2. Live repository TOML, discovered only in campaign mode. Bounded runs must
    not depend on whatever `.toml` files happen to be on disk, so discovery is
    gated behind the campaign budget; generated, dependency, build-output, and
    secret-looking paths are excluded.
 
 @module
 */

import {
  glob,
  readFile,
} from 'node:fs/promises';
import {
  join,
  relative,
} from 'node:path';
import { fileURLToPath, } from 'node:url';

/**
 One corpus document: a human-readable label and its source text.
 */
export type CorpusEntry = {
  /**
   Label for diagnostics, typically a repo-relative path.
   */
  readonly name: string;
  /**
   TOML source text.
   */
  readonly source: string;
};

/**
 Absolute path to the shared TOML edit fixture root.
 */
const fixturesRootDir = fileURLToPath(
  new URL(
    '../../../test-fixture/toml-edit/src/',
    import.meta.url,
  ),
);

/**
 Absolute path to the monorepo root, four levels above this sidecar's `src`.
 */
const repoRootDir = fileURLToPath(new URL(
  '../../../../',
  import.meta.url,
),);

/**
 Path fragments that disqualify a discovered file: dependencies, build output,
 VCS internals, caches, curated TOML fixture sets (which deliberately include
 invalid and version-specific inputs, so they are loaded explicitly rather
 than swept up as "real" config), and anything that looks like a secret.
 */
const EXCLUDED_FRAGMENTS: readonly string[] = [
  'node_modules',
  '/dist/',
  '/target/',
  '/.git/',
  '/coverage/',
  '/.cache/',
  '/.out-of-scope/',
  'test-fixture',
  'toml-test',
  '/fixtures/',
  'secret',
  'credential',
  'password',
  '.env',
];

/**
 Read every `-input.toml` file in `directory` as a corpus entry.
 
 @returns Corpus entries in stable lexical order.
 */
async function loadInputDir({ directory, }: { readonly directory: string; },): Promise<readonly CorpusEntry[]> {
  /**
   Matching file names, sorted so the corpus order is reproducible.
   */
  const names: string[] = [];
  for await (const name of glob(
    '*-input.toml',
    { cwd: directory, },
  )) {
    names.push(name,);
  }
  return Promise.all(
    names.toSorted()
      .map(async function read(name,) {
        return {
          name,
          source: await readFile(
            join(
              directory,
              name,
            ),
            'utf8',
          ),
        };
      },),
  );
}

/**
 Load the committed valid TOML fixtures.
 
 @returns Valid-fixture corpus entries.
 
 @example
 ```ts
 const valid = await loadValidFixtures();
 ```
 */
export function loadValidFixtures(): Promise<readonly CorpusEntry[]> {
  return loadInputDir({ directory: join(
    fixturesRootDir,
    'valid',
  ), },);
}

/**
 Load the committed invalid TOML fixtures.
 
 @returns Invalid-fixture corpus entries.
 
 @example
 ```ts
 const invalid = await loadInvalidFixtures();
 ```
 */
export function loadInvalidFixtures(): Promise<readonly CorpusEntry[]> {
  return loadInputDir({ directory: join(
    fixturesRootDir,
    'invalid',
  ), },);
}

/**
 Whether `path` is admissible repo corpus (not dependency, build, or secret).
 
 @returns Whether the discovered path should be read as corpus.
 */
function isAdmissiblePath({ path, }: { readonly path: string; },): boolean {
  return !EXCLUDED_FRAGMENTS.some(function present(fragment,) {
    return path.includes(fragment,);
  },);
}

/**
 Discover live repository TOML files, excluding noise and secret-looking paths.
 
 Campaign-only: bounded runs pass `campaign: false` and receive an empty list
 so their corpus is exactly the committed fixtures.
 
 @param campaign - Whether the campaign budget is active.
 
 @param limit - Upper bound on discovered files so a campaign run stays bounded.
 
 @returns Discovered corpus entries, or an empty list in bounded mode.
 
 @example
 ```ts
 const live = await discoverRepoToml({ campaign: true, limit: 200, },);
 ```
 */
export async function discoverRepoToml(
  {
    campaign,
    limit,
  }: {
    readonly campaign: boolean;
    readonly limit: number;
  },
): Promise<readonly CorpusEntry[]> {
  if (!campaign) return [];
  /**
   Admissible absolute paths discovered under the repo root.
   */
  const paths: string[] = [];
  for await (const path of glob(
    '**/*.toml',
    { cwd: repoRootDir, },
  )) {
    /**
     Absolute path so the exclusion check sees the full fragment context.
     */
    const absolute = join(
      repoRootDir,
      path,
    );
    if (isAdmissiblePath({ path: absolute, },)) paths.push(absolute,);
    if (paths.length >= limit) break;
  }
  return Promise.all(
    paths.map(async function read(absolute,) {
      return {
        name: relative(
          repoRootDir,
          absolute,
        ),
        source: await readFile(
          absolute,
          'utf8',
        ),
      };
    },),
  );
}
