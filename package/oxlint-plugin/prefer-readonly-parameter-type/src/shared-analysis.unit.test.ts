import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  dirname,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildEffectSummaryIndex,
  clearEffectSummaryCache,
  clearFinalEffectIndexCache,
  closeSemanticBridge,
  createSharedKeyResolver,
  effectProjectFingerprint,
  type EffectProjectFingerprint,
  effectSummaryCacheStats,
  openSemanticFile,
  type SharedKey,
  VERIFY_SHARED_DIRECTORY_ENVIRONMENT_VARIABLE,
  VERIFY_SHARED_ENVIRONMENT_VARIABLE,
  verifySharedHit,
} from '../dist/final/node/index.mjs';

/** Helper source analysed once and shared by every project that agrees on its inputs. */
const HELPER_SOURCE = 'export function inspect(value: { text: string; },): string { return value.text; }\n';

/** Helper source whose parameter type comes from a path alias. */
const ALIAS_HELPER_SOURCE = "import type { Shape, } from '@shape';\nexport function area(value: Shape,): number { return value.size; }\n";

/** Helper source whose callee comes from a path alias. */
const ALIAS_CALLEE_HELPER_SOURCE = "import { dependency, } from '@dependency';\nexport function relay(value: { text: string; },): string { return dependency(value); }\n";

/** Active source in each project importing the shared helper. */
const INPUT_SOURCE = "import { inspect, } from '../shared/helper.js';\nexport function caller(value: { text: string; },): string { return inspect(value); }\n";

/** Active source in each project importing the alias-typed helper. */
const ALIAS_INPUT_SOURCE = "import { area, } from '../shared/helper.js';\nexport function measure(value: { size: number; },): number { return area(value); }\n";

/** Active source in each project importing the alias-callee helper. */
const ALIAS_CALLEE_INPUT_SOURCE = "import { relay, } from '../shared/helper.js';\nexport function forward(value: { text: string; },): string { return relay(value); }\n";

/** Helper source whose only dependency does not resolve to any file. */
const UNRESOLVED_HELPER_SOURCE = "import { missing, } from './missing.js';\nexport function inspect(value: { text: string; },): string { return missing(value); }\n";

/** Helper source naming a module through an `import()` type whose argument is not a string literal. */
const NON_LITERAL_IMPORT_TYPE_HELPER_SOURCE = "export function inspect(value: { text: string; },): string { return value.text; }\nexport type Loaded = import(SpecifierName);\n";

/** Edited dependency body that keeps the callable shape but mutates its parameter. */
const EDITED_DEPENDENCY_SOURCE = 'export function dependency(value: { text: string; },): string { value.text = value.text.trim(); return value.text; }\n';

/** Helper source that is a global script rather than a module. */
const SCRIPT_HELPER_SOURCE = 'function globalInspect(value: { text: string; },): string { return value.text; }\n';

/** Active source that reaches the global script helper without importing it. */
const SCRIPT_INPUT_SOURCE = 'export function globalCaller(value: { text: string; },): string { return globalInspect(value); }\n';

/** Observing dependency body written under two names so projects can resolve the alias to either. */
const DEPENDENCY_SOURCE = 'export function dependency(value: { text: string; },): string { return value.text; }\n';

/** Helper source whose only dependency is reached through `export ... from`. */
const EXPORT_FROM_HELPER_SOURCE = "export { dependency, } from '@dependency';\nexport function relay(value: { text: string; },): string { return value.text; }\n";

/** Helper source whose only dependency is reached through `import x = require(...)`. */
const IMPORT_EQUALS_HELPER_SOURCE = "import dependency = require('@dependency');\nexport function relay(value: { text: string; },): string { return value.text; }\nexport const loaded: typeof dependency = dependency;\n";

/** Helper source whose only reference to a JSX-capable file is a type import. */
const IMPLICIT_IMPORT_HELPER_SOURCE = "import type { Props, } from './view.js';\nexport function inspect(value: Props,): string { return value.text; }\n";

/** JSX-capable file the helper imports, which the parser lists no runtime import for. */
const VIEW_SOURCE = 'export type Props = { text: string; };\n';

/** Helper source reading a global that only a triple-slash type reference can bring into the program. */
const GLOBAL_READING_HELPER_SOURCE = 'export function inspect(value: { text: string; },): string { return value.text + String(sharedAnalysisReferenced,); }\n';

/** Module in the second project whose triple-slash directive adds the global type package to the program. */
const REFERENCING_MODULE_SOURCE = '/// <reference types="sharedreferenced" />\nexport const referencing: number = 1;\n';

/** Type package declaring the global as a script, outside `@types` so only the directive loads it. */
const REFERENCED_PACKAGE_DECLARATION = 'declare const sharedAnalysisReferenced: string;\n';

/** Manifest of the type package. */
const REFERENCED_PACKAGE_MANIFEST = '{"name":"sharedreferenced","version":"1.0.0","types":"index.d.ts"}\n';

/** Strict configuration shared by projects that must agree on checking options. */
const STRICT_COMPILER_OPTIONS = '"strict":true';

/** Configuration that permits `import x = require(...)` in a module source. */
const PRESERVE_COMPILER_OPTIONS = `${STRICT_COMPILER_OPTIONS},"module":"preserve","moduleResolution":"bundler"`;

/** Schema of entries written before shared envelopes carried a key digest. */
const PRECEDING_SCHEMA = 8;

/** The persisted reason an analysis was narrowed. */
const OMISSION_REASON_A = 'direct-summary-construction-failed';

/** Include list reaching the project directory, the shared directory and its JSX-capable files. */
const JSX_INCLUDE = '"include":["*.ts","../shared/*.ts","../shared/*.tsx"]';

/** Project whose helper reaches a JSX-capable file. */
const JSX_PROJECT: ProjectFixture = {
  name: 'first',
  compilerOptions: `${STRICT_COMPILER_OPTIONS},"jsx":"preserve"`,
  inputSource: INPUT_SOURCE,
  files: {},
  include: JSX_INCLUDE,
};

/** Include list reaching the project directory and the shared directory. */
const INCLUDE = '"include":["*.ts","../shared/*.ts"]';

/** Disposable fixture root. */
type DisposableFixtureRoot = {
  readonly path: string;
  readonly [Symbol.dispose]: () => void;
};

/**
 Creates disposable fixture root.
 
 @returns directory removed after the test scope.
 */
function disposableFixtureRoot(): DisposableFixtureRoot {
  const path = mkdtempSync(join(scratchRoot(), 'readonly-shared-',),);
  return {
    path,
    [Symbol.dispose](): void {
      rmSync(path, { recursive: true, force: true, },);
    },
  };
}

/** What one side of a shared-hit comparison knows about a source. */
type AnalysisSnapshot = Parameters<typeof verifySharedHit>[0]['cached'];

/** One shared-hit comparison, whose cached side defaults to the agreeing snapshot. */
type VerificationCase = {
  readonly fileName: string;
  readonly cached?: AnalysisSnapshot;
  readonly fresh: AnalysisSnapshot;
};

/** One project directory next to the shared directory. */
type ProjectFixture = {
  readonly name: string;
  readonly compilerOptions: string;
  readonly inputSource: string;
  readonly files: Readonly<Record<string, string>>;
  readonly include?: string;
};

/**
 Writes one project directory with its configuration, active source, and extra files.
 
 @param root - Disposable fixture root.
 
 @param project - Project name, configuration, and files.
 
 @returns absolute path of the project's active source.
 */
function writeProject({
  root,
  project,
}: {
  readonly root: string;
  readonly project: ProjectFixture;
},): string {
  const directory = join(root, project.name,);
  mkdirSync(directory, { recursive: true, },);
  writeFileSync(
    join(directory, 'tsconfig.json',),
    `{"compilerOptions":{${project.compilerOptions}},${project.include ?? INCLUDE}}\n`,
  );
  writeFileSync(join(directory, 'input.ts',), project.inputSource,);
  Object.entries(project.files,)
    .forEach(function writeExtra([name, source,],): void {
      mkdirSync(dirname(join(directory, name,),), { recursive: true, },);
      writeFileSync(join(directory, name,), source,);
    },);
  return join(directory, 'input.ts',);
}

/**
 Writes the shared directory every project includes.
 
 @param root - Disposable fixture root.
 
 @param files - File names and sources written under the shared directory.
 */
function writeSharedDirectory({
  root,
  files,
}: {
  readonly root: string;
  readonly files: Readonly<Record<string, string>>;
},): void {
  const directory = join(root, 'shared',);
  mkdirSync(directory, { recursive: true, },);
  Object.entries(files,)
    .forEach(function writeShared([name, source,],): void {
      writeFileSync(join(directory, name,), source,);
    },);
}

/**
 Builds the effect index of one project's active source and reads the cache counters.
 
 @param inputPath - Active source path.
 
 @param inputSource - Active source text.
 
 @param cacheRoot - Persistent cache root shared by both projects.
 
 @returns counters observed after the build.
 */
function buildProject({
  inputPath,
  inputSource,
  cacheRoot,
}: {
  readonly inputPath: string;
  readonly inputSource: string;
  readonly cacheRoot: string;
},): ReturnType<typeof effectSummaryCacheStats> {
  clearEffectSummaryCache();
  clearFinalEffectIndexCache();
  const session = openSemanticFile({
    fileName: inputPath,
    sourceText: inputSource,
    hasBOM: false,
  },);
  buildEffectSummaryIndex({
    project: session.project,
    activeSourceFile: session.sourceFile,
    cacheRootOverride: cacheRoot,
  },);
  const stats = effectSummaryCacheStats();
  closeSemanticBridge();
  return stats;
}

/**
 Builds the first project cold, then the second against the cache the first wrote.
 
 @param root - Disposable fixture root.
 
 @param first - First project.
 
 @param second - Second project.
 
 @returns counters observed while building the second project.
 */
function buildBothProjects({
  root,
  first,
  second,
}: {
  readonly root: string;
  readonly first: ProjectFixture;
  readonly second: ProjectFixture;
},): ReturnType<typeof effectSummaryCacheStats> {
  /** Persistent cache root outside every project. */
  const cacheRoot = join(root, '.effect-cache',);
  buildProject({
    inputPath: writeProject({
      root,
      project: first,
    },),
    inputSource: first.inputSource,
    cacheRoot,
  },);
  return buildProject({
    inputPath: writeProject({
      root,
      project: second,
    },),
    inputSource: second.inputSource,
    cacheRoot,
  },);
}

/**
 Lists the shared entries a cache root holds.
 
 @param cacheRoot - Persistent cache root.
 
 @returns absolute paths of every shared entry.
 */
function sharedEntryPaths(cacheRoot: string,): readonly string[] {
  return readdirSync(cacheRoot, { recursive: true, encoding: 'utf8', },)
    .filter(function isSharedEntry(name,): boolean {
      return name.startsWith('shared',) && name.endsWith('.json',);
    },)
    .map(function absolute(name,): string {
      return join(cacheRoot, name,);
    },);
}

/** Environment values set for one scope. */
type EnvironmentOverride = {
  readonly [Symbol.dispose]: () => void;
};

/**
 Sets environment variables for the scope that holds the result.
 
 @param values - Variable names and the values they hold in the scope.
 
 @returns override that puts every variable back the way it was,
 removing the ones that were unset.
 */
function overrideEnvironment(values: Readonly<Record<string, string>>,): EnvironmentOverride {
  /** Values the variables held before, for those that were set. */
  const previous = new Map<string, string>();
  /** Names of the variables that were unset. */
  const unset: string[] = [];
  Object.entries(values,)
    .forEach(function setVariable([name, value,],): void {
      const before = process.env[name];
      if (before === undefined)
        unset.push(name,);
      else
        previous.set(name, before,);
      process.env[name] = value;
    },);
  return {
    [Symbol.dispose](): void {
      previous.forEach(function restoreVariable(value, name,): void {
        process.env[name] = value;
      },);
      unset.forEach(function removeVariable(name,): void {
        Reflect.deleteProperty(process.env, name,);
      },);
    },
  };
}

/**
 Computes the cross-project key of one shared source as one project sees it.
 
 @param root - Disposable fixture root.
 
 @param project - Project whose snapshot resolves the key.
 
 @param sharedName - File name under the shared directory whose key is computed.
 
 @param adjustFingerprint - Rewrites the fingerprint before the key is computed,
 to model a snapshot that lacks something.
 
 @returns the key or its fallback reason.
 */
function sharedKeyOf({
  root,
  project,
  sharedName,
  adjustFingerprint,
}: {
  readonly root: string;
  readonly project: ProjectFixture;
  readonly sharedName: string;
  readonly adjustFingerprint?: (fingerprint: EffectProjectFingerprint) => EffectProjectFingerprint;
},): SharedKey {
  /** Active source path of the project. */
  const inputPath = writeProject({
    root,
    project,
  },);
  clearEffectSummaryCache();
  clearFinalEffectIndexCache();
  const session = openSemanticFile({
    fileName: inputPath,
    sourceText: project.inputSource,
    hasBOM: false,
  },);
  /** Whole-program identities the key resolver reads. */
  const fingerprint = effectProjectFingerprint({
    project: session.project,
    activeSourceFile: session.sourceFile,
  },);
  /** Program file the key is computed for. */
  const sharedFile = session.project.program
    .getSourceFile(join(realpathSync(root,), 'shared', sharedName,),);
  if (sharedFile === undefined)
    throw new Error(`Expected ${sharedName} in the program of ${project.name}.`,);
  /** Key or fallback reason, read before the bridge releases its snapshot. */
  const key = createSharedKeyResolver({
    project: session.project,
    fingerprint: adjustFingerprint === undefined ? fingerprint : adjustFingerprint(fingerprint,),
  },).keyFor(sharedFile,);
  closeSemanticBridge();
  return key;
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
  name: 'cross-project sharing of one file analysis',
  concurrency: 1,
  children: [
    it({
      name: 'shares a file between projects that differ only in an unreachable file',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': HELPER_SOURCE, },
        },);
        /** Counters of the second project, whose own unreachable files differ from the first's. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: INPUT_SOURCE,
            files: { 'unrelated.ts': 'export const unrelated: number = 1;\n', },
          },
          second: {
            name: 'second',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: INPUT_SOURCE,
            files: { 'other.d.ts': 'export type Other = { readonly other: string; };\n', },
          },
        },);
        /* The helper is analysed once; only the second project's own active source is built. */
        expect(stats.sharedSourceCacheHitCount,).toBe(1,);
        expect(stats.directSummaryBuildCount,).toBe(1,);
        releaseCycle();
      },
    },),
    it({
      name: 'does not share a file whose reachable declaration file differs',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': ALIAS_HELPER_SOURCE, },
        },);
        /** Counters of the second project, whose alias reaches a different declaration. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"paths":{"@shape":["./shape.d.ts"]}`,
            inputSource: ALIAS_INPUT_SOURCE,
            files: { 'shape.d.ts': 'export type Shape = { readonly size: number; };\n', },
          },
          second: {
            name: 'second',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"paths":{"@shape":["./shape.d.ts"]}`,
            inputSource: ALIAS_INPUT_SOURCE,
            files: { 'shape.d.ts': 'export type Shape = { size: number; readonly extra: string; };\n', },
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        releaseCycle();
      },
    },),
    it({
      name: 'does not share a file between projects that differ in a checking option',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': HELPER_SOURCE, },
        },);
        /** Counters of the second project, whose checking options differ from the first's. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: INPUT_SOURCE,
            files: {},
          },
          second: {
            name: 'second',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"exactOptionalPropertyTypes":true`,
            inputSource: INPUT_SOURCE,
            files: {},
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        releaseCycle();
      },
    },),
    it({
      name: 'does not share a file when paths change what one import resolves to',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: {
            'helper.ts': ALIAS_CALLEE_HELPER_SOURCE,
            'dependency-a.ts': DEPENDENCY_SOURCE,
            'dependency-b.ts': DEPENDENCY_SOURCE,
          },
        },);
        /** Counters of the second project, whose alias resolves to a sibling with equal content. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-a.ts"]}`,
            inputSource: ALIAS_CALLEE_INPUT_SOURCE,
            files: {},
          },
          second: {
            name: 'second',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-b.ts"]}`,
            inputSource: ALIAS_CALLEE_INPUT_SOURCE,
            files: {},
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(3,);
        releaseCycle();
      },
    },),
    it({
      name: 'shares a file between projects whose paths differ without changing any resolution',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: {
            'helper.ts': ALIAS_CALLEE_HELPER_SOURCE,
            'dependency-a.ts': DEPENDENCY_SOURCE,
          },
        },);
        /** Counters of the second project, whose extra alias is never imported. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-a.ts"]}`,
            inputSource: ALIAS_CALLEE_INPUT_SOURCE,
            files: {},
          },
          second: {
            name: 'second',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-a.ts"],"@unused":["./unused.ts"]}`,
            inputSource: ALIAS_CALLEE_INPUT_SOURCE,
            files: { 'unused.ts': 'export const unused: number = 1;\n', },
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(2,);
        expect(stats.directSummaryBuildCount,).toBe(1,);
        releaseCycle();
      },
    },),
    it({
      name: 'does not share a file when an ambient declaration differs',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': HELPER_SOURCE, },
        },);
        /** Counters of the second project, whose global script differs from the first's. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: INPUT_SOURCE,
            files: { 'ambient.d.ts': 'declare const sharedAnalysisAmbient: string;\n', },
          },
          second: {
            name: 'second',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: INPUT_SOURCE,
            files: { 'ambient.d.ts': 'declare const sharedAnalysisAmbient: number;\n', },
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        releaseCycle();
      },
    },),
    it({
      name: 'does not share a file whose module reference does not resolve',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': UNRESOLVED_HELPER_SOURCE, },
        },);
        /** Counters of the second project, whose helper cannot prove its closure complete. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: INPUT_SOURCE,
            files: {},
          },
          second: {
            name: 'second',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: INPUT_SOURCE,
            files: {},
            /* The second program holds every file the first held, so the whole-scope snapshot an
             * unresolved closure takes cannot be what keeps the entry from being read back. */
            include: '"include":["*.ts","../shared/*.ts","../first/*.ts"]',
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        releaseCycle();
      },
    },),
    it({
      name: 'does not share a file that is a global script',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': SCRIPT_HELPER_SOURCE, },
        },);
        /** Counters of the second project, whose helper applies to every file without an import. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: SCRIPT_INPUT_SOURCE,
            files: {},
          },
          second: {
            name: 'second',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: SCRIPT_INPUT_SOURCE,
            files: {},
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        releaseCycle();
      },
    },),
    it({
      name: 'does not share a file when a compiler option has not been classified',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': HELPER_SOURCE, },
        },);
        /** Counters of the second project, whose configuration sets an option on neither list. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"generateTrace":"first-trace"`,
            inputSource: INPUT_SOURCE,
            files: {},
          },
          second: {
            name: 'second',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"generateTrace":"first-trace"`,
            inputSource: INPUT_SOURCE,
            files: {},
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        releaseCycle();
      },
    },),
    it({
      name: 'does not share a file whose import type names its module without a string literal',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': NON_LITERAL_IMPORT_TYPE_HELPER_SOURCE, },
        },);
        /** Counters of the second project, whose helper has a reference no closure can resolve. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: INPUT_SOURCE,
            files: {},
          },
          second: {
            name: 'second',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: INPUT_SOURCE,
            files: {},
            /* The second program holds every file the first held, which is the one case where an
             * entry that snapshots its whole scope could still validate. */
            include: '"include":["*.ts","../shared/*.ts","../first/*.ts"]',
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        releaseCycle();
      },
    },),
    it({
      name: 'does not share a file whose reachable dependency changed content at the same path',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: {
            'helper.ts': ALIAS_CALLEE_HELPER_SOURCE,
            'dependency-a.ts': DEPENDENCY_SOURCE,
          },
        },);
        /** Persistent cache root outside every project. */
        const cacheRoot = join(root.path, '.effect-cache',);
        /** Project configuration shared by both builds. */
        const project: ProjectFixture = {
          name: 'first',
          compilerOptions: `${STRICT_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-a.ts"]}`,
          inputSource: ALIAS_CALLEE_INPUT_SOURCE,
          files: {},
        };
        buildProject({
          inputPath: writeProject({
            root: root.path,
            project,
          },),
          inputSource: project.inputSource,
          cacheRoot,
        },);
        writeSharedDirectory({
          root: root.path,
          files: { 'dependency-a.ts': EDITED_DEPENDENCY_SOURCE, },
        },);
        /** Counters of a second project reaching the dependency at the same path with new content. */
        const stats = buildProject({
          inputPath: writeProject({
            root: root.path,
            project: {
              ...project,
              name: 'second',
            },
          },),
          inputSource: project.inputSource,
          cacheRoot,
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(3,);
        releaseCycle();
      },
    },),
    it({
      name: 'rejects a shared entry stamped with the schema that preceded the key digest',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': HELPER_SOURCE, },
        },);
        /** Persistent cache root outside every project. */
        const cacheRoot = join(root.path, '.effect-cache',);
        buildProject({
          inputPath: writeProject({
            root: root.path,
            project: {
              name: 'first',
              compilerOptions: STRICT_COMPILER_OPTIONS,
              inputSource: INPUT_SOURCE,
              files: {},
            },
          },),
          inputSource: INPUT_SOURCE,
          cacheRoot,
        },);
        /** Shared entries the first project wrote. */
        const entries = sharedEntryPaths(cacheRoot,);
        expect(entries.length > 0,).toBe(true,);
        entries.forEach(function restampEntry(entryPath,): void {
          /** Entry as the first project wrote it. */
          const entry = JSON.parse(readFileSync(entryPath, 'utf8',),) as { schema: number; };
          /* Entries written before shared envelopes carried a key digest used schema 8. */
          expect(entry.schema > PRECEDING_SCHEMA,).toBe(true,);
          entry.schema = PRECEDING_SCHEMA;
          writeFileSync(entryPath, JSON.stringify(entry,),);
        },);
        /** Counters of a second project that would otherwise read the restamped entries. */
        const stats = buildProject({
          inputPath: writeProject({
            root: root.path,
            project: {
              name: 'second',
              compilerOptions: STRICT_COMPILER_OPTIONS,
              inputSource: INPUT_SOURCE,
              files: {},
            },
          },),
          inputSource: INPUT_SOURCE,
          cacheRoot,
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        releaseCycle();
      },
    },),
    it({
      name: 'names the fallback reason of a source that is a global script',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': SCRIPT_HELPER_SOURCE, },
        },);
        expect(
          sharedKeyOf({
            root: root.path,
            sharedName: 'helper.ts',
            project: {
              name: 'first',
              compilerOptions: STRICT_COMPILER_OPTIONS,
              inputSource: SCRIPT_INPUT_SOURCE,
              files: {},
            },
          },),
        ).toEqual({
          eligible: false,
          reason: 'script-source',
        },);
      },
    },),
    it({
      name: 'names the fallback reason of a compiler option on neither list',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': HELPER_SOURCE, },
        },);
        expect(
          sharedKeyOf({
            root: root.path,
            sharedName: 'helper.ts',
            project: {
              name: 'first',
              compilerOptions: `${STRICT_COMPILER_OPTIONS},"generateTrace":"first-trace"`,
              inputSource: INPUT_SOURCE,
              files: {},
            },
          },),
        ).toEqual({
          eligible: false,
          reason: 'unknown-option:generateTrace',
        },);
      },
    },),
    it({
      name: 'names the fallback reason of a specifier that does not resolve',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': UNRESOLVED_HELPER_SOURCE, },
        },);
        /** Key of the helper, whose import names no file. */
        const key = sharedKeyOf({
          root: root.path,
          sharedName: 'helper.ts',
          project: {
            name: 'first',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: INPUT_SOURCE,
            files: {},
          },
        },);
        expect(key.eligible,).toBe(false,);
        expect(key.eligible ? undefined : key.reason,).toBe('unresolved-specifier',);
      },
    },),
    it({
      name: 'names the fallback reason of a closure file the snapshot holds no digest for',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': HELPER_SOURCE, },
        },);
        /** Key of the helper in a snapshot whose digests omit the helper itself. */
        const key = sharedKeyOf({
          root: root.path,
          sharedName: 'helper.ts',
          project: {
            name: 'first',
            compilerOptions: STRICT_COMPILER_OPTIONS,
            inputSource: INPUT_SOURCE,
            files: {},
          },
          adjustFingerprint(fingerprint,): EffectProjectFingerprint {
            return {
              ...fingerprint,
              sourceDigests: new Map(
                [...fingerprint.sourceDigests,]
                  .filter(function keepOthers([fileName,],): boolean {
                    return !fileName.endsWith('helper.ts',);
                  },),
              ),
            };
          },
        },);
        expect(key,).toEqual({
          eligible: false,
          reason: 'no-source-digest',
        },);
      },
    },),
    it({
      name: 'names the fallback reason of a file that reaches a JSX-capable file',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: {
            'helper.ts': IMPLICIT_IMPORT_HELPER_SOURCE,
            'view.tsx': VIEW_SOURCE,
          },
        },);
        /** Key of the helper, which reaches a file whose implicit import no closure lists. */
        const key = sharedKeyOf({
          root: root.path,
          sharedName: 'helper.ts',
          project: JSX_PROJECT,
        },);
        expect(key.eligible,).toBe(false,);
        expect(key.eligible ? undefined : key.reason,).toBe('implicit-import-file',);
      },
    },),
    it({
      name: 'does not share a file that reaches a JSX-capable file the parser lists no runtime import for',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: {
            'helper.ts': IMPLICIT_IMPORT_HELPER_SOURCE,
            'view.tsx': VIEW_SOURCE,
          },
        },);
        /** Project fixture identical in both builds. */
        const project = JSX_PROJECT;
        /** Counters of the second project, which agrees with the first on every input. */
        const stats = buildBothProjects({
          root: root.path,
          first: project,
          second: {
            ...project,
            name: 'second',
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        releaseCycle();
      },
    },),
    it({
      name: 'shares a file whose export-from target is the same file in both projects',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: {
            'helper.ts': EXPORT_FROM_HELPER_SOURCE,
            'dependency-a.ts': DEPENDENCY_SOURCE,
          },
        },);
        /** Counters of the second project, which resolves the re-export to the first's file. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-a.ts"]}`,
            inputSource: ALIAS_CALLEE_INPUT_SOURCE,
            files: {},
          },
          second: {
            name: 'second',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-a.ts"]}`,
            inputSource: ALIAS_CALLEE_INPUT_SOURCE,
            files: { 'unrelated.ts': 'export const unrelated: number = 1;\n', },
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(1,);
        expect(stats.directSummaryBuildCount,).toBe(1,);
        releaseCycle();
      },
    },),
    it({
      name: 'does not share a file whose only dependency is reached through export-from and differs',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: {
            'helper.ts': EXPORT_FROM_HELPER_SOURCE,
            'dependency-a.ts': DEPENDENCY_SOURCE,
            'dependency-b.ts': DEPENDENCY_SOURCE,
          },
        },);
        /** Counters of the second project, whose alias resolves the re-export to a sibling. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-a.ts"]}`,
            inputSource: ALIAS_CALLEE_INPUT_SOURCE,
            files: {},
          },
          second: {
            name: 'second',
            compilerOptions: `${STRICT_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-b.ts"]}`,
            inputSource: ALIAS_CALLEE_INPUT_SOURCE,
            files: {},
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        releaseCycle();
      },
    },),
    it({
      name: 'shares a file whose import-equals target is the same file in both projects',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: {
            'helper.ts': IMPORT_EQUALS_HELPER_SOURCE,
            'dependency-a.ts': DEPENDENCY_SOURCE,
          },
        },);
        /** Counters of the second project, which resolves the require to the first's file. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: `${PRESERVE_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-a.ts"]}`,
            inputSource: ALIAS_CALLEE_INPUT_SOURCE,
            files: {},
          },
          second: {
            name: 'second',
            compilerOptions: `${PRESERVE_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-a.ts"]}`,
            inputSource: ALIAS_CALLEE_INPUT_SOURCE,
            files: { 'unrelated.ts': 'export const unrelated: number = 1;\n', },
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(1,);
        expect(stats.directSummaryBuildCount,).toBe(1,);
        releaseCycle();
      },
    },),
    it({
      name: 'does not share a file whose only dependency is reached through import-equals and differs',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: {
            'helper.ts': IMPORT_EQUALS_HELPER_SOURCE,
            'dependency-a.ts': DEPENDENCY_SOURCE,
            'dependency-b.ts': DEPENDENCY_SOURCE,
          },
        },);
        /** Counters of the second project, whose alias resolves the require to a sibling. */
        const stats = buildBothProjects({
          root: root.path,
          first: {
            name: 'first',
            compilerOptions: `${PRESERVE_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-a.ts"]}`,
            inputSource: ALIAS_CALLEE_INPUT_SOURCE,
            files: {},
          },
          second: {
            name: 'second',
            compilerOptions: `${PRESERVE_COMPILER_OPTIONS},"paths":{"@dependency":["../shared/dependency-b.ts"]}`,
            inputSource: ALIAS_CALLEE_INPUT_SOURCE,
            files: {},
          },
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        releaseCycle();
      },
    },),
    it({
      name: 'does not share a file when one project adds a triple-slash type reference the source uses without importing',
      fn: async () => {
        using root = disposableFixtureRoot();
        writeSharedDirectory({
          root: root.path,
          files: { 'helper.ts': GLOBAL_READING_HELPER_SOURCE, },
        },);
        /** Project that never loads the type package. */
        const first: ProjectFixture = {
          name: 'first',
          compilerOptions: STRICT_COMPILER_OPTIONS,
          inputSource: INPUT_SOURCE,
          files: {},
        };
        /** Project identical to the first except for a module whose directive loads the type package. */
        const second: ProjectFixture = {
          name: 'second',
          compilerOptions: STRICT_COMPILER_OPTIONS,
          inputSource: INPUT_SOURCE,
          files: {
            'referencing.ts': REFERENCING_MODULE_SOURCE,
            'node_modules/sharedreferenced/index.d.ts': REFERENCED_PACKAGE_DECLARATION,
            'node_modules/sharedreferenced/package.json': REFERENCED_PACKAGE_MANIFEST,
          },
        };
        /** Key of the helper in the project that never loads the type package. */
        const firstKey = sharedKeyOf({
          root: root.path,
          sharedName: 'helper.ts',
          project: first,
        },);
        /** Key of the helper in the project whose directive loads it. */
        const secondKey = sharedKeyOf({
          root: root.path,
          sharedName: 'helper.ts',
          project: second,
        },);
        if ((!firstKey.eligible) || (!secondKey.eligible))
          throw new Error('Expected both projects to key the helper.',);
        /* The ambient surface is the only component the directive can change, so the keys split on it. */
        expect(
          Object.keys(firstKey.parts,)
            .filter(function differs(part,): boolean {
              return firstKey.parts[part] !== secondKey.parts[part];
            },)
            .toSorted(),
        ).toEqual(['ambient',],);
        /** Counters of the second project, built against the first's cache. */
        const stats = buildBothProjects({
          root: root.path,
          first,
          second,
        },);
        expect(stats.sharedSourceCacheHitCount,).toBe(0,);
        expect(stats.directSummaryBuildCount,).toBe(2,);
        releaseCycle();
      },
    },),
    it({
      name: 'reports a mismatch when omitted callables, omission reasons or external omissions differ although the payload agrees',
      fn: async () => {
        using root = disposableFixtureRoot();
        /** Directory receiving this process's verification counter file. */
        const directory = join(root.path, 'verification',);
        using environment = overrideEnvironment({
          [VERIFY_SHARED_ENVIRONMENT_VARIABLE]: '1',
          [VERIFY_SHARED_DIRECTORY_ENVIRONMENT_VARIABLE]: directory,
        },);
        /** Snapshot both sides share, with no omissions. */
        const agreeing: AnalysisSnapshot = {
          summaries: new Map(),
          omittedCallableKeys: [],
          omissionReasons: [],
          externalOmissions: [],
        };
        /** Comparisons run, the first of which must not be reported. */
        const cases: readonly VerificationCase[] = [
          {
            fileName: 'agreeing.ts',
            fresh: agreeing,
          },
          {
            fileName: 'omitted-keys.ts',
            fresh: {
              ...agreeing,
              omittedCallableKeys: ['omitted-keys.ts:inspect',],
            },
          },
          {
            fileName: 'omission-reasons.ts',
            fresh: {
              ...agreeing,
              omissionReasons: [OMISSION_REASON_A,],
            },
          },
          {
            fileName: 'external-omissions.ts',
            fresh: {
              ...agreeing,
              externalOmissions: [{
                sourceFileName: 'node_modules/package/index.js',
                omittedCount: 1,
                reasons: [OMISSION_REASON_A,],
              },],
            },
          },
        ];
        cases.forEach(function verifyCase({ fileName, cached, fresh, },): void {
          verifySharedHit({
            fileName,
            project: 'reading-project',
            writerProject: 'writing-project',
            cached: cached ?? agreeing,
            fresh,
          },);
        },);
        /** Counter files this process wrote. */
        const counterFiles = readdirSync(directory,)
          .filter(function isCounterFile(name,): boolean {
            return name.endsWith('.json',);
          },);
        expect(counterFiles.length,).toBe(1,);
        /** Mismatch records of this process, one per differing case. */
        const { mismatchRecords, } = JSON.parse(
          readFileSync(join(directory, counterFiles[0] ?? '',), 'utf8',),
        ) as { readonly mismatchRecords: readonly { readonly fileName: string; readonly parts: readonly string[]; }[]; };
        expect(mismatchRecords.map(function describeRecord({ fileName, parts, },): string {
          return `${fileName}: ${parts.join('; ',)}`;
        },),).toEqual([
          'omitted-keys.ts: omittedCallableKeys differ',
          'omission-reasons.ts: omissionReasons differ',
          'external-omissions.ts: externalOmissions differ',
        ],);
      },
    },),
  ],
},);
