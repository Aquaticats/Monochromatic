#!/usr/bin/env node

/**
 Tightens monorepo `pnpm-workspace.yaml` catalog `>=x.y.z` ranges
 to match the versions actually installed in `node_modules`.
 
 Only touches entries in the default `catalog` object
 whose range starts with `>=`. Entries using `*`, exact versions,
 GitHub references, or named catalogs are skipped.
 
 @example
 ```sh
 node package/dev-script/catalog-tighten/src/index.ts
 node package/dev-script/catalog-tighten/src/index.ts --dry-run
 ```
 */

import {
  readFile,
  stat,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  parseCatalogFromYaml,
} from '@monochromatic-dev/module-pnpm-workspace-catalog/ts';

import {
  isStrictlyGreater,
  NO_INSTALLED_VERSION,
  NOT_A_RANGE,
  type ParsedRange,
  parseRange,
  readInstalledVersion,
  resolveNpmNames,
} from './version.ts';
import {
  isDeclaredByLiveImporter,
} from './declared.ts';
import {
  readModulesDir,
} from './settings.ts';
import {
  firstStoreHit,
  NOT_IN_STORE,
} from './store-probe.ts';
import {
  rewriteCatalogRanges,
} from './yaml-rewrite.ts';
import { tightenRustCatalog, } from './cargo/run.ts';



//region Types

/**
 Result of comparing catalog range against installed version.
 */
type TightenResult = {
  /**
   Package name as it appears in the catalog key.
   */
  readonly name: string;
  /**
   Original catalog range string, e.g. `">=1.2.0"`.
   */
  readonly oldRange: string;
  /**
   New tightened range string, e.g. `">=1.3.0"`.
   */
  readonly newRange: string;
};

/**
 One npm-name probe paired with its resolved installed version.
 
 `version` is an optional field (bucket 1): the probe omits it when the name
 did not resolve, so callers narrow with `version !== undefined` rather than a
 sentinel. The {@link readInstalledVersion} return-sentinel is converted to this
 optional shape at the `.map` seam below.
 */
type ProbedCandidate = {
  /**
   npm name that was looked up in node_modules.
   */
  readonly name: string;
  /**
   Installed version; omitted when this name did not resolve.
   */
  readonly version?: string;
};

//endregion Types

//region Main

/**
 Whether `--dry-run` was passed on the command line.
 */
const dryRun = process.argv
  .includes('--dry-run',);

/**
 Absolute path to the monorepo root (where this script is invoked from).
 */
const monorepoRoot = resolve('.',);

/**
 Absolute path to pnpm-workspace.yaml.
 */
const workspaceYamlPath = join(
  monorepoRoot,
  'pnpm-workspace.yaml',
);

/**
 Reads pnpm-workspace.yaml, failing with a clear message when it is absent
 (run outside a pnpm workspace) instead of surfacing a raw `ENOENT`.
 
 @param path - absolute path to pnpm-workspace.yaml
 
 @returns file content
 
 @throws Error when the file cannot be read
 
 @example
 ```ts
 await readWorkspaceYaml("/repo/pnpm-workspace.yaml")
 ```
 */
async function readWorkspaceYaml(path: string,): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    throw new Error(
      `pnpm-workspace.yaml not found at ${path}; run catalog-tighten from a pnpm workspace root.`,
      { cause: error, },
    );
  }
}

/**
 Raw content of pnpm-workspace.yaml, preserved for minimal-diff rewriting.
 */
const workspaceYamlContent = await readWorkspaceYaml(workspaceYamlPath,);

/**
 Per-importer modules directory (the effective `modulesDir` setting; usually `node_modules`).
 Queried through pnpm so env and global config apply, then used by the install guard and resolver.
 */
const modulesDir = await readModulesDir(monorepoRoot,);

/**
 Returns true when every char in `s` is ASCII whitespace (space, tab,
 newline, carriage return, form feed, vertical tab). Empty strings are
 vacuously whitespace-only, matching `\s*$` semantics for a blank trailing
 tail.
 
 @param s - candidate string
 
 @returns whether `s` consists solely of whitespace
 */
function isWhitespaceOnly(s: string,): boolean {
  for (const c of s) {
    /**
     Whether the current char satisfies regex `\s`.
     */
    const ok = (c === ' ')
      || (c === '\t')
      || (c === '\n')
      || (c === '\r')
      || (c === '\f')
      || (c === '\v');
    if (!ok)
      return false;
  }
  return true;
}

/**
 Reports whether `path` exists on disk, async (the repo bans sync fs).
 
 @param path - absolute filesystem path to probe
 
 @returns whether an entry exists at `path`
 
 @example
 ```ts
 await pathExists("/repo/node_modules") // true
 ```
 */
async function pathExists(path: string,): Promise<boolean> {
  try {
    await stat(path,);
    return true;
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    return false;
  }
}

/**
 Workspace catalog mapping package names to version ranges.
 */
const parsedCatalog = parseCatalogFromYaml(workspaceYamlContent,);
/**
 Default catalog entries; named catalogs are intentionally outside tighten's scope.
 */
const catalog = parsedCatalog.defaultCatalog;
if (Object.keys(catalog,)
  .length
  === 0) {
  console.error('No catalog found in pnpm-workspace.yaml',);
  process.exitCode = 1;
  throw new Error('No catalog found in pnpm-workspace.yaml',);
}

/**
 Whether an install exists to resolve against: `node_modules` (node-modules linkers) or `.pnp.cjs` (PnP).
 Without one every entry would report MISS, so failing loud here points at the real cause.
 */
const hasInstall = (await pathExists(join(
  monorepoRoot,
  modulesDir,
),))
  || (await pathExists(join(
    monorepoRoot,
    '.pnp.cjs',
  ),));
if (!hasInstall) {
  throw new Error(
    'No install found (node_modules and .pnp.cjs both absent). Run `pnpm install` before catalog-tighten.',
  );
}

/**
 Aggregated outcome of processing every catalog entry.
 
 Folded by `Array.reduce` over `Object.entries(catalog)` so the per-category
 counters and `results` accumulator live on the same object instead of as
 module-root `let` bindings.
 */
type CatalogSummary = {
  /**
   Tightening results to write back to `pnpm-workspace.yaml`.
   */
  readonly results: readonly TightenResult[];
  /**
   Count of entries skipped (not `>=` ranges).
   */
  readonly skippedCount: number;
  /**
   Count of entries where the installed version matched the catalog range (already tight).
   */
  readonly alreadyTightCount: number;
  /**
   Count of entries present in the pnpm store as a transitive dependency but declared directly by no live package.
   */
  readonly undeclaredCount: number;
  /**
   Count of entries not installed anywhere in the workspace (no importer symlink and no store copy).
   */
  readonly notFoundCount: number;
};

/**
 Initial summary fed into the reduce; every counter starts at zero with an empty result list.
 */
const initialSummary: CatalogSummary = {
  results: [],
  skippedCount: 0,
  alreadyTightCount: 0,
  undeclaredCount: 0,
  notFoundCount: 0,
};

/**
 Catalog as `[name, value]` entry pairs, folded into the per-category summary below.
 */
const catalogEntries = Object.entries(catalog,);
/**
 Classifies and processes each catalog entry for tightening.
 */
const entrySummaries = await Promise.all(catalogEntries.map(
  async function processEntry([name, value,]: readonly [
    string,
    string,
  ],): Promise<CatalogSummary> {
    /**
     Parsed range prefix and version, or `NOT_A_RANGE` if not a `>=` range.
     */
    const parsed = parseRange(value,);
    if (parsed === NOT_A_RANGE) {
      console.info(`SKIP  ${name}: ${value} (not a >= range)`,);
      return {
        ...initialSummary,
        skippedCount: 1,
      };
    }

    /**
     Candidate npm package names to probe in node_modules.
     */
    const npmNames = resolveNpmNames({
      catalogKey: name,
      catalogValue: value,
    },);
    /**
     Installed-version probes for every candidate npm name.
     */
    const probes = await Promise.all(npmNames.map(async function probeCandidate(candidate,): Promise<ProbedCandidate> {
      /**
       Installed version for this candidate; `NO_INSTALLED_VERSION` when it did not resolve.
       */
      const installed = await readInstalledVersion({
        npmName: candidate,
        monorepoRoot,
        modulesDir,
      },);
      return installed === NO_INSTALLED_VERSION
        ? { name: candidate, }
        : {
          name: candidate,
          version: installed,
        };
    },),);
    /**
     First npm name candidate whose installed version resolves.
     */
    const resolved = probes.find(function hasVersion(r: Readonly<ProbedCandidate>,): boolean {
      return r.version
        !== undefined;
    },);

    if ((resolved === undefined) || (resolved.version
      === undefined)) {
      // No direct-dependency symlink resolved. Classify the miss: a package
      // installed only as a transitive dependency lives in the pnpm store with
      // no top-level symlink, so probe the store, then confirm no live importer
      // declares it before calling it undeclared (symlinks also vanish under
      // `symlink: false` or a broken `.pnp.cjs`, where the package is declared).
      /**
       Store versions found for the first candidate name that has any store copy, or `NOT_IN_STORE`.
       */
      const storeVersions = await firstStoreHit({
        npmNames,
        monorepoRoot,
        modulesDir,
      },);
      if (storeVersions !== NOT_IN_STORE) {
        /**
         Whether a live importer declares this entry directly; only an undeclared store copy is `UNDCL`.
         */
        const declared = await isDeclaredByLiveImporter({
          npmNames,
          monorepoRoot,
        },);
        if (!declared) {
          console.warn(
            `UNDCL ${name}: in pnpm store as ${
              storeVersions.join(', ',)
            } but no live package declares it directly -- add it as a direct dependency of its user, or drop the catalog entry`,
          );
          return {
            ...initialSummary,
            undeclaredCount: 1,
          };
        }
        console.warn(
          `MISS  ${name}: declared but no installed version resolved -- present in pnpm store as ${
            storeVersions.join(', ',)
          }, but no importer symlink or PnP entry resolved it (broken install?)`,
        );
        return {
          ...initialSummary,
          notFoundCount: 1,
        };
      }
      console.warn(
        `MISS  ${name}: not installed in this workspace -- no importer symlink, PnP entry, or store copy (tried ${
          npmNames.join(', ',)
        })`,
      );
      return {
        ...initialSummary,
        notFoundCount: 1,
      };
    }

    if (!isStrictlyGreater({
      cataloged: parsed.version,
      installed: resolved.version,
    },)) {
      console.info(
        `OK    ${name}: >=${parsed.version} -- installed ${resolved.version} (already tight)`,
      );
      return {
        ...initialSummary,
        alreadyTightCount: 1,
      };
    }

    /**
     Tightened version range using the installed version as the lower bound.
     */
    const newRange = `${parsed.prefix}>=${resolved.version}`;
    console.info(
      `TIGHT ${name}: ${value} -> ${newRange} (installed ${resolved.version})`,
    );
    return {
      ...initialSummary,
      results: [{
        name,
        oldRange: value,
        newRange,
      },],
    };
  },
),);
/**
 Aggregated outcome merged from per-entry summaries.
 */
const summary: CatalogSummary = entrySummaries.reduce(
  function mergeSummary(
    acc: Readonly<CatalogSummary>,
    entrySummary: Readonly<CatalogSummary>,
  ): CatalogSummary {
    return {
      results: [
        ...acc.results,
        ...entrySummary.results,
      ],
      skippedCount: acc.skippedCount + entrySummary.skippedCount,
      alreadyTightCount: acc.alreadyTightCount + entrySummary.alreadyTightCount,
      undeclaredCount: acc.undeclaredCount + entrySummary.undeclaredCount,
      notFoundCount: acc.notFoundCount + entrySummary.notFoundCount,
    };
  },
  initialSummary,
);

//region Write results

if (summary.results
  .length
  === 0)
  console.info('\nNo catalog entries to tighten.',);
else if (dryRun) {
  console.info(
    `\nDry run: ${String(summary.results
      .length,)} entries would be tightened.`,
  );
}
else {
  /**
   Rewritten file with tightened ranges; surgical string replacement preserves
   formatting, comments, ordering, and the file's single-quote style.
   */
  const rewritten = rewriteCatalogRanges({
    content: workspaceYamlContent,
    results: summary.results,
  },);

  await writeFile(
    workspaceYamlPath,
    rewritten,
  );
  console.info(
    `\nWrote ${
      String(summary.results
        .length,)
    } tightened entries to pnpm-workspace.yaml.`,
  );
}

//endregion Write results

//region Summary

console.info(`\nSummary:`,);
console.info(`  Tightened: ${String(summary.results
  .length,)}`,);
console.info(`  Already tight: ${String(summary.alreadyTightCount,)}`,);
console.info(`  Skipped (not >=): ${String(summary.skippedCount,)}`,);
console.info(`  Undeclared (in store, not tightened): ${String(summary.undeclaredCount,)}`,);
console.info(`  Not installed: ${String(summary.notFoundCount,)}`,);

//endregion Summary

//region Cargo catalog

// The Rust catalog (rust-catalog.toml) is tightened by the same command; see src/cargo/run.ts.
await tightenRustCatalog({
  root: monorepoRoot,
  dryRun,
},);

//endregion Cargo catalog

//endregion Main
