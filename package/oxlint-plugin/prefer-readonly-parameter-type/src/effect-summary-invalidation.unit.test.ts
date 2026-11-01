import {
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { isFunctionLikeDeclaration, } from 'typescript/ast/is';

import {
  buildEffectSummaryIndex,
  clearEffectSummaryCache,
  clearFinalEffectIndexCache,
  closeSemanticBridge,
  effectSummaryCacheStats,
  NO_EFFECT_SUMMARY,
  openSemanticFile,
} from '../dist/final/node/index.mjs';

/** Demand-reachable fixture sources scanned and persisted on a cold build. */
const FIXTURE_SOURCE_COUNT = 2;

/** Source whose runtime-variable dynamic import must not join any closure. */
const DYNAMIC_IMPORT_SOURCE = 'export async function loadRuntime(specifier: string,): Promise<unknown> { return import(specifier); }\n';

/** Caller source whose dependency closure contains the helper module. */
const CALLER_SOURCE = "import { inspect, } from './helper.js';\nexport function caller(value: { text: string; },): string { return inspect(value); }\n";

/** Observing helper source written by cold fixture builds. */
const OBSERVING_HELPER_SOURCE = 'export function inspect(value: { text: string; },): string { return value.text; }\n';

/** Helper source mutating its parameter referent. */
const MUTATING_HELPER_SOURCE = 'export function inspect(value: { text: string; },): string { value.text = value.text.trim(); return value.text; }\n';

/** Standalone source outside every other file's dependency closure. */
const STANDALONE_SOURCE = 'export function standalone(value: { text: string; },): string { return value.text; }\n';

/** Edited standalone source preserving its single-callable shape. */
const EDITED_STANDALONE_SOURCE = 'export function standalone(value: { text: string; },): string { return value.text.trim(); }\n';

/** Ambient declaration participating only in the declaration surface. */
const AMBIENT_SOURCE = 'declare const effectIncrementalAmbient: string;\n';

/** Edited ambient declaration changing the declaration surface digest. */
const EDITED_AMBIENT_SOURCE = 'declare const effectIncrementalAmbient: number;\n';

/** Cold TypeScript configuration admitting every fixture source. */
const TSCONFIG_SOURCE = '{"compilerOptions":{"strict":true},"include":["*.ts"]}\n';

/** Configuration changing resolved compiler options without membership. */
const EDITED_TSCONFIG_SOURCE = '{"compilerOptions":{"strict":true,"noUnusedLocals":true},"include":["*.ts"]}\n';

/** Disposable fixture project root. */
type DisposableFixtureRoot = {
  readonly path: string;
  readonly [Symbol.dispose]: () => void;
};

/**
 Creates disposable fixture project root.
 
 @returns project directory removed after test scope.
 */
function disposableFixtureRoot(): DisposableFixtureRoot {
  const path = mkdtempSync(join(scratchRoot(), 'readonly-incremental-',),);
  return {
    path,
    [Symbol.dispose](): void {
      rmSync(path, { recursive: true, force: true, },);
    },
  };
}

/** Fixture paths shared by every incremental invalidation case. */
type IncrementalFixturePaths = {
  readonly cacheRoot: string;
  readonly tsconfigPath: string;
  readonly inputPath: string;
  readonly helperPath: string;
  readonly standalonePath: string;
  readonly ambientPath: string;
};

/**
 Writes the shared three-source fixture project.
 
 The caller imports the helper,
 the standalone source imports nothing,
 and the ambient declaration reaches analysis only through the
 declaration surface digest,
 so each case can edit exactly one invalidation channel.
 
 @param projectRoot - Disposable project directory.
 
 @returns fixture paths keyed by role.
 */
function writeIncrementalFixture(projectRoot: string,): IncrementalFixturePaths {
  const paths: IncrementalFixturePaths = {
    cacheRoot: join(projectRoot, '.effect-cache',),
    tsconfigPath: join(projectRoot, 'tsconfig.json',),
    inputPath: join(projectRoot, 'input.ts',),
    helperPath: join(projectRoot, 'helper.ts',),
    standalonePath: join(projectRoot, 'standalone.ts',),
    ambientPath: join(projectRoot, 'ambient.d.ts',),
  };
  writeFileSync(paths.tsconfigPath, TSCONFIG_SOURCE,);
  writeFileSync(paths.inputPath, CALLER_SOURCE,);
  writeFileSync(paths.helperPath, OBSERVING_HELPER_SOURCE,);
  writeFileSync(paths.standalonePath, STANDALONE_SOURCE,);
  writeFileSync(paths.ambientPath, AMBIENT_SOURCE,);
  return paths;
}

/** One bridge-lifecycle build over the caller source. */
type FixtureBuildCycle = {
  readonly session: ReturnType<typeof openSemanticFile>;
  readonly index: ReturnType<typeof buildEffectSummaryIndex>;
};

/**
 Opens the caller source and builds one effect index.
 
 @param paths - Fixture paths naming input and persistent cache root.
 
 @returns open session and built index for summary lookups.
 */
function buildFixtureIndex(paths: IncrementalFixturePaths,): FixtureBuildCycle {
  const session = openSemanticFile({
    fileName: paths.inputPath,
    sourceText: CALLER_SOURCE,
    hasBOM: false,
  },);
  const index = buildEffectSummaryIndex({
    project: session.project,
    activeSourceFile: session.sourceFile,
    cacheRootOverride: paths.cacheRoot,
  },);
  return {
    session,
    index,
  };
}

/**
 Runs the cold build cycle and proves every reached source persisted.
 
 @param paths - Fixture paths naming input and persistent cache root.
 */
function runColdCycle(paths: IncrementalFixturePaths,): void {
  clearEffectSummaryCache();
  clearFinalEffectIndexCache();
  buildFixtureIndex(paths,);
  /** Counters after uncached scan of demand-reachable fixture sources. */
  const coldStats = effectSummaryCacheStats();
  expect(coldStats.persistentSourceCacheHitCount,).toBe(0,);
  expect(coldStats.directSummaryBuildCount,).toBe(FIXTURE_SOURCE_COUNT,);
  expect(coldStats.persistentCacheWriteCount,).toBe(FIXTURE_SOURCE_COUNT,);
  closeSemanticBridge();
}

/**
 Runs one warm build cycle against the persisted cache.
 
 @param paths - Fixture paths naming input and persistent cache root.
 
 @returns warm cycle handles and counters observed after the build.
 */
function runWarmCycle(paths: IncrementalFixturePaths,): FixtureBuildCycle & {
  readonly stats: ReturnType<typeof effectSummaryCacheStats>;
} {
  clearEffectSummaryCache();
  /** Warm session and index built over persisted entries. */
  const cycle = buildFixtureIndex(paths,);
  return {
    ...cycle,
    stats: effectSummaryCacheStats(),
  };
}

/**
 Releases bridge and process caches after one case.
 */
function releaseCycle(): void {
  closeSemanticBridge();
  clearEffectSummaryCache();
  clearFinalEffectIndexCache();
}

await describe({
  name: 'incremental persistent-cache invalidation',
  concurrency: 1,
  children: [
    it({
      name: 'preserves persistent entries outside an edited file closure',
      fn: async () => {
        using projectRoot = disposableFixtureRoot();
        /** Shared fixture paths for the unrelated-edit case. */
        const paths = writeIncrementalFixture(projectRoot.path,);
        runColdCycle(paths,);
        writeFileSync(paths.standalonePath, EDITED_STANDALONE_SOURCE,);
        /** Warm counters after editing a file no other closure contains. */
        const { stats, } = runWarmCycle(paths,);
        /* Unreachable source has no summary entry to invalidate;
         * both demanded entries remain valid without rescanning edited code. */
        expect(stats.persistentSourceCacheHitCount,).toBe(FIXTURE_SOURCE_COUNT,);
        expect(stats.directSummaryBuildCount,).toBe(0,);
        expect(stats.persistentCacheWriteCount,).toBe(0,);
        releaseCycle();
      },
    },),
    it({
      name: 'invalidates dependents of an edited dependency and keeps unrelated entries',
      fn: async () => {
        using projectRoot = disposableFixtureRoot();
        /** Shared fixture paths for the dependency-edit case. */
        const paths = writeIncrementalFixture(projectRoot.path,);
        runColdCycle(paths,);
        writeFileSync(paths.helperPath, MUTATING_HELPER_SOURCE,);
        /** Warm cycle after editing the caller's imported implementation. */
        const { session, index, stats, } = runWarmCycle(paths,);
        /* The unchanged caller revalidates against its recorded dependency
         * closure, so the helper edit invalidates both demanded entries.
         * Unreachable standalone source has no cache entry to preserve. */
        expect(stats.persistentSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(FIXTURE_SOURCE_COUNT,);
        expect(stats.persistentCacheWriteCount,).toBe(FIXTURE_SOURCE_COUNT,);
        /** Unchanged caller declaration resolved in the warm session. */
        const declaration = session.nodeAtOffset(CALLER_SOURCE.indexOf('caller',),)
          .parent;
        if (!isFunctionLikeDeclaration(declaration,))
          throw new Error('Expected caller function declaration.',);
        /** Caller summary rebuilt from the mutated helper implementation. */
        const summary = index.get(declaration,);
        if (summary === NO_EFFECT_SUMMARY)
          throw new Error('Expected caller effect summary.',);
        expect([...summary.referentMutatedParameterIndexes,],).toEqual([0,],);
        releaseCycle();
      },
    },),
    it({
      name: 'invalidates a caller through a semantic edge without a module import',
      fn: async () => {
        using projectRoot = disposableFixtureRoot();
        /** Persistent cache isolated from source files. */
        const cacheRoot = join(projectRoot.path, '.effect-cache',);
        /** Active global-script caller source. */
        const inputPath = join(projectRoot.path, 'input.ts',);
        /** Global-script callee source reached without import syntax. */
        const helperPath = join(projectRoot.path, 'helper.ts',);
        /** Caller relying on global declaration from sibling script. */
        const inputSource = 'function globalCaller(value: { text: string; },): string { return globalInspect(value); }\n';
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["*.ts"]}\n',
        );
        writeFileSync(inputPath, inputSource,);
        writeFileSync(
          helperPath,
          'function globalInspect(value: { text: string; },): string { return value.text; }\n',
        );
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const coldSession = openSemanticFile({
          fileName: inputPath,
          sourceText: inputSource,
          hasBOM: false,
        },);
        buildEffectSummaryIndex({
          project: coldSession.project,
          activeSourceFile: coldSession.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        closeSemanticBridge();
        writeFileSync(
          helperPath,
          "function globalInspect(value: { text: string; },): string { value.text = 'changed'; return value.text; }\n",
        );
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const warmSession = openSemanticFile({
          fileName: inputPath,
          sourceText: inputSource,
          hasBOM: false,
        },);
        const warmIndex = buildEffectSummaryIndex({
          project: warmSession.project,
          activeSourceFile: warmSession.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        /** Caller declaration whose cached edge depends on helper semantics. */
        const declaration = warmSession.nodeAtOffset(inputSource.indexOf('globalCaller',),)
          .parent;
        if (!isFunctionLikeDeclaration(declaration,))
          throw new Error('Expected global caller declaration.',);
        /** Rebuilt caller summary receiving changed helper effect. */
        const summary = warmIndex.get(declaration,);
        if (summary === NO_EFFECT_SUMMARY)
          throw new Error('Expected global caller effect summary.',);
        /** Counters proving semantic edge invalidated both reached sources. */
        const stats = effectSummaryCacheStats();
        expect(stats.persistentSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        expect([...summary.referentMutatedParameterIndexes,],).toEqual([0,],);
        releaseCycle();
      },
    },),
    it({
      name: 'keeps a runtime-variable dynamic import out of every closure',
      fn: async () => {
        using projectRoot = disposableFixtureRoot();
        /** Shared fixture paths for the dynamic-import case. */
        const paths = writeIncrementalFixture(projectRoot.path,);
        writeFileSync(
          join(projectRoot.path, 'dynamic.ts',),
          DYNAMIC_IMPORT_SOURCE,
        );
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        buildFixtureIndex(paths,);
        /** Cold counters excluding unreachable dynamic-import source. */
        const coldStats = effectSummaryCacheStats();
        expect(coldStats.persistentCacheWriteCount,).toBe(FIXTURE_SOURCE_COUNT,);
        closeSemanticBridge();
        writeFileSync(paths.standalonePath, EDITED_STANDALONE_SOURCE,);
        /** Warm counters after an edit unrelated to the dynamic importer. */
        const { stats, } = runWarmCycle(paths,);
        /* Runtime-variable importer is unreachable from active effects,
         * so neither it nor edited standalone source enters analysis. */
        expect(stats.persistentSourceCacheHitCount,).toBe(FIXTURE_SOURCE_COUNT,);
        expect(stats.directSummaryBuildCount,).toBe(0,);
        expect(stats.persistentCacheWriteCount,).toBe(0,);
        releaseCycle();
      },
    },),
    it({
      name: 'invalidates every entry when the declaration surface changes',
      fn: async () => {
        using projectRoot = disposableFixtureRoot();
        /** Shared fixture paths for the declaration-surface case. */
        const paths = writeIncrementalFixture(projectRoot.path,);
        runColdCycle(paths,);
        writeFileSync(paths.ambientPath, EDITED_AMBIENT_SOURCE,);
        /** Warm counters after editing a file excluded from every closure. */
        const { stats, } = runWarmCycle(paths,);
        /* Declaration files never join dependency closures, so only the
         * whole-scope declaration surface digest can carry this edit;
         * every entry must fail revalidation. */
        expect(stats.persistentSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(FIXTURE_SOURCE_COUNT,);
        expect(stats.persistentCacheWriteCount,).toBe(FIXTURE_SOURCE_COUNT,);
        releaseCycle();
      },
    },),
    it({
      name: 'invalidates every entry when resolved compiler options change',
      fn: async () => {
        using projectRoot = disposableFixtureRoot();
        /** Shared fixture paths for the compiler-options case. */
        const paths = writeIncrementalFixture(projectRoot.path,);
        runColdCycle(paths,);
        writeFileSync(paths.tsconfigPath, EDITED_TSCONFIG_SOURCE,);
        /** Warm counters after changing resolved compiler options only. */
        const { stats, } = runWarmCycle(paths,);
        /* Compiler options change semantic meaning without changing any
         * source digest, so the compiler-options surface digest must
         * invalidate the whole scope. */
        expect(stats.persistentSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(FIXTURE_SOURCE_COUNT,);
        expect(stats.persistentCacheWriteCount,).toBe(FIXTURE_SOURCE_COUNT,);
        releaseCycle();
      },
    },),
  ],
},);
