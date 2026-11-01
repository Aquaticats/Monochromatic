/**
 Cross-project cache key for one source's direct effect summaries.
 
 @module
 */

import type {
  Node,
  SourceFile,
} from 'typescript/ast';
import { isStringLiteral, } from 'typescript/ast/is';
import type { Project, } from 'typescript/sync';

import { isDeclarationFileName, } from './declaration-file-name.ts';
import type { EffectProjectFingerprint, } from './effect-project-fingerprint.ts';
import {
  analyzerDigest,
  contentDigest,
} from './effect-summary-cache-identity.ts';
import { configuredProjectConfigFileName, } from './semantic-file-name.ts';
import { isWorkspaceSourceFileName, } from './workspace-source-path.ts';
import {
  recordSharedClosure,
  recordSharedEligible,
  recordSharedFallback,
} from './shared-analysis-telemetry.ts';

/**
 Sentinel for a program file that has no content digest or metadata in the snapshot.
 */
const ENTRY_UNAVAILABLE: unique symbol = Symbol('closure entry unavailable because the fingerprint holds no digest or metadata for the program file',);

/**
 Key schema, bumped whenever the components below change meaning.
 */
const SHARED_KEY_SCHEMA = 1;

/**
 Extensions whose files can carry implicit module references that the parser does not list.
 
 A JSX source imports its runtime module from `jsxImportSource` without writing the import,
 so the closure cannot see that edge.
 */
const IMPLICIT_IMPORT_EXTENSIONS: readonly string[] = [
  '.tsx',
  '.jsx',
  '.astro',
  '.mdx',
  '.vue',
  '.svelte',
];

/**
 Key shared by every project whose inputs to one source's analysis agree,
 or the reason this source keeps its per-project entry.
 */
export type SharedKey =
  | {
    readonly eligible: true;
    readonly digest: string;
    readonly parts: Readonly<Record<string, string>>;
  }
    | {
      readonly eligible: false;
      readonly reason: string;
      readonly detail?: string;
    };

/**
 Resolver of cross-project keys for the sources of one project snapshot.
 */
export type SharedKeyResolver = {
  readonly keyFor: (sourceFile: SourceFile) => SharedKey;
};

/**
 Files one file's module references resolve to,
 or the reason they cannot all be proven.
 */
type ModuleTargets =
  | {
    readonly complete: true;
    readonly targets: readonly string[];
  }
    | {
      readonly complete: false;
      readonly reason: string;
      readonly detail?: string;
    };

/**
 Tests whether the effect analysis treats a program file as owned source it can inspect.
 
 This is the classification `indexedSourceFileMap` and the owned-callable resolution apply,
 not the raw library flag: a workspace package reached through a symlink in
 `node_modules` is classified as external while its source stays inspectable,
 so a package's own project and a project importing it must agree on it.
 
 @param fileName - Program file path.
 
 @param isFromExternalLibrary - Program metadata flag for the file.
 
 @param analysisRoot - External implementation root included despite library classification.
 
 @returns whether the analysis indexes the file.
 
 @example
 ```ts
 const owned = isOwnedSource({ fileName, isFromExternalLibrary: false });
 ```
 */
function isOwnedSource({
  fileName,
  isFromExternalLibrary,
  analysisRoot,
}: {
  readonly fileName: string;
  readonly isFromExternalLibrary: boolean;
  readonly analysisRoot?: string;
},): boolean {
  if (isDeclarationFileName(fileName,))
    return false;
  return (!isFromExternalLibrary)
    || isWorkspaceSourceFileName(fileName,)
    || ((analysisRoot !== undefined) && fileName
      .startsWith(analysisRoot,));
}

/**
 Renders one specifier node for a diagnostic.
 
 @param specifier - Specifier node the parser listed.
 
 @returns the quoted specifier text when the node carries it,
 otherwise the node kind.
 */
function specifierText(specifier: Node,): string {
  return isStringLiteral(specifier,)
    ? JSON.stringify(specifier.text,)
    : `a node of kind ${String(specifier.kind,)}`;
}

/**
 Resolves the module references one program file lists to program file paths.
 
 The parser lists static imports, re-exports, `import =` requires, literal dynamic imports and
 literal `import()` types. Triple-slash directives are not resolved here:
 a directive only adds a file to the program,
 and whether that file affects a source that does not import it is decided by the ambient
 surface.
 
 @param project - Configured project resolving module symbols.
 
 @param sourceFile - Decoded program file.
 
 @returns resolved paths,
 or the reason a reference does not resolve.
 */
function resolveModuleTargets({
  project,
  sourceFile,
}: {
  readonly project: Project;
  readonly sourceFile: SourceFile;
},): ModuleTargets {
  if (IMPLICIT_IMPORT_EXTENSIONS.some(function hasExtension(extension,): boolean {
    return sourceFile.fileName
      .endsWith(extension,);
  },)) {
    return {
      complete: false,
      reason: 'implicit-import-file',
    };
  }
  /**
   Specifier nodes the parser listed for the file.
   */
  const specifiers = sourceFile.imports;
  if (specifiers.length === 0) {
    return {
      complete: true,
      targets: [],
    };
  }
  /**
   Module symbols of every listed specifier, resolved in one request.
   */
  const symbols = project.checker
    .getSymbolAtLocation(specifiers,);
  /**
   Resolved target paths.
   */
  const targets = new Set<string>();
  for (const [index, specifier,] of specifiers.entries()) {
    /**
     Module symbol resolved for the specifier at the same position.
     */
    const symbol = symbols[index];
    /**
     Declaration handle naming the module's file.
     */
    const handle = symbol?.valueDeclaration
      ?? symbol?.declarations
        .at(0,);
    /**
     Resolved declaration whose file the reference reaches.
     */
    const declaration = handle?.resolve(project,);
    if (declaration === undefined) {
      return {
        complete: false,
        reason: 'unresolved-specifier',
        detail: `${sourceFile.fileName} names ${specifierText(specifier,)}`,
      };
    }
    targets.add(
      declaration.getSourceFile()
        .fileName,
    );
  }
  return {
    complete: true,
    targets: [...targets,].toSorted(),
  };
}

/**
 Entries of every file one source's module references reach,
 or the reason the closure cannot be proven complete.
 */
type ClosureEntries =
  | {
    readonly complete: true;
    readonly entries: readonly string[];
  }
    | {
      readonly complete: false;
      readonly reason: string;
      readonly detail?: string;
    };

/**
 Builds the cross-project key resolver for one project snapshot.
 
 The key names everything that can change one source's direct summaries:
 the analyzer, the source text, every file its module references reach
 (declaration and external files included, each with its content digest and its package
 module format), the ambient and augmentation surfaces, the lockfile, and the keyed
 compiler options.
 It leaves out the project's file list, its declaration files that nothing reaches,
 and the resolution-only options, so projects that agree on all of the above share.
 A reference that does not resolve to a program file, a source that is not a module,
 or an unclassified compiler option keeps the per-project entry.
 
 @param project - Configured project resolving module references.
 
 @param fingerprint - Whole-program identities of the same snapshot.
 
 @param analysisRoot - External implementation root that changes the analysis scope.
 
 @returns resolver memoized per source for this snapshot.
 
 @example
 ```ts
 const keys = createSharedKeyResolver({ project, fingerprint });
 ```
 */
export function createSharedKeyResolver({
  project,
  fingerprint,
  analysisRoot,
}: {
  readonly project: Project;
  readonly fingerprint: EffectProjectFingerprint;
  readonly analysisRoot?: string;
},): SharedKeyResolver {
  /**
   Module targets memoized per program file.
   */
  const targetsByFile = new Map<string, ModuleTargets>();
  /**
   Closure entry text memoized per program file,
   unavailable when the snapshot lacks the file's digest or metadata.
   */
  const entryByFile = new Map<string, string | typeof ENTRY_UNAVAILABLE>();
  /**
   Key memoized per source path.
   */
  const keyByFile = new Map<string, SharedKey>();

  /**
   Resolves every module reference of one program file.
   
   @param fileName - Program file path.
   
   @returns resolved file paths,
   or the reason a reference could not be resolved.
   */
  function targetsOf(fileName: string,): ModuleTargets {
    /**
     Targets resolved earlier in this snapshot.
     */
    const memoized = targetsByFile.get(fileName,);
    if (memoized !== undefined)
      return memoized;
    /**
     Decoded program file whose references are listed.
     */
    const sourceFile = project.program
      .getSourceFile(fileName,);
    /**
     Resolution outcome retained for later closures.
     */
    const targets = sourceFile === undefined
      ? {
        complete: false as const,
        reason: 'file-not-in-program',
      }
      : resolveModuleTargets({
        project,
        sourceFile,
      },);
    targetsByFile.set(
      fileName,
      targets,
    );
    return targets;
  }

  /**
   Describes one program file inside a closure.
   
   @param fileName - Program file path.
   
   @returns path, content digest, and package module format,
   or the unavailable sentinel when the snapshot lacks either.
   */
  function entryOf(fileName: string,): string | typeof ENTRY_UNAVAILABLE {
    /**
     Entry computed earlier in this snapshot.
     */
    const memoized = entryByFile.get(fileName,);
    if (memoized !== undefined)
      return memoized;
    /**
     Content digest of the file in this snapshot.
     */
    const digest = fingerprint.sourceDigests
      .get(fileName,);
    /**
     Program metadata deciding indexing and module format.
     */
    const metadata = project.program
      .getSourceFileMetadata(fileName,);
    /**
     Entry text, unavailable when the snapshot does not know the file.
     */
    const entry = (digest === undefined) || (metadata === undefined)
      ? ENTRY_UNAVAILABLE
      : [
        fileName,
        digest,
        String(isOwnedSource({
          fileName,
          isFromExternalLibrary: metadata.isFromExternalLibrary,
          ...(analysisRoot === undefined) ? {} : { analysisRoot, },
        },),),
        String(metadata.isDefaultLibrary,),
        String(metadata.impliedNodeFormat,),
        metadata.packageJsonType,
      ].join('\0',);
    entryByFile.set(
      fileName,
      entry,
    );
    return entry;
  }

  /**
   Walks every file reachable through module references from one source.
   
   @param sourceFile - Source whose closure is walked.
   
   @returns sorted closure entries,
   or the reason the closure cannot be proven complete.
   */
  function closureEntries(sourceFile: SourceFile,): ClosureEntries {
    /**
     Files already walked.
     */
    const visited = new Set<string>([sourceFile.fileName,],);
    /**
     Work stack of files whose references remain to resolve.
     */
    const pending: string[] = [sourceFile.fileName,];
    /**
     Closure entries accumulated over walked files.
     */
    const entries: string[] = [];
    while (pending.length > 0) {
      /**
       Next file to expand.
       */
      const current = pending.pop();
      if (current === undefined)
        continue;
      /**
       Closure entry describing the file.
       */
      const entry = entryOf(current,);
      if ((typeof entry) === 'symbol') {
        return {
          complete: false,
          reason: 'no-source-digest',
        };
      }
      entries.push(entry,);
      /**
       Files the current file's references reach.
       */
      const resolved = targetsOf(current,);
      if (!resolved.complete) {
        return {
          complete: false,
          reason: resolved.reason,
          ...(resolved.detail === undefined) ? {} : { detail: resolved.detail, },
        };
      }
      resolved.targets
        .forEach(function enqueueTarget(target,): void {
          if (!visited.has(target,)) {
            visited.add(target,);
            pending.push(target,);
          }
        },);
    }
    return {
      complete: true,
      entries: entries.toSorted(),
    };
  }

  /**
   Computes the key for one source without memoization.
   
   @param sourceFile - Source whose key is computed.
   
   @returns shared key or fallback reason.
   */
  function computeKey(sourceFile: SourceFile,): SharedKey {
    /**
     Whole-program inputs binding the key.
     */
    const {
      shared,
      surfaces,
    } = fingerprint;
    /**
     Checking options of the project.
     */
    const { checkingOptions, } = shared;
    if (!checkingOptions.eligible) {
      return {
        eligible: false,
        reason: checkingOptions.reason,
      };
    }
    if (sourceFile.externalModuleIndicator === undefined) {
      return {
        eligible: false,
        reason: 'script-source',
      };
    }
    /**
     Closure entries or the reason none can be proven.
     */
    const closure = closureEntries(sourceFile,);
    if (!closure.complete) {
      return {
        eligible: false,
        reason: closure.reason,
        ...(closure.detail === undefined) ? {} : { detail: closure.detail, },
      };
    }
    /**
     Digest of each component, kept apart so a split between projects can be traced to one.
     */
    const parts = {
      schema: String(SHARED_KEY_SCHEMA,),
      analyzer: analyzerDigest(),
      fileName: sourceFile.fileName,
      source: contentDigest(sourceFile.text,),
      closure: contentDigest(JSON.stringify(closure.entries,),),
      ambient: shared.ambientSurfaceDigest,
      augmentation: surfaces.augmentationSurfaceDigest,
      lockfile: surfaces.lockfileDigest,
      options: checkingOptions.digest,
      analysisRoot: analysisRoot ?? '',
    };
    recordSharedClosure({
      fileName: sourceFile.fileName,
      entries: closure.entries,
    },);
    return {
      eligible: true,
      digest: contentDigest(JSON.stringify(parts,),),
      parts,
    };
  }

  return {
    keyFor(sourceFile: SourceFile,): SharedKey {
      /**
       Key computed earlier for this source.
       */
      const memoized = keyByFile.get(sourceFile.fileName,);
      if (memoized !== undefined)
        return memoized;
      /**
       Freshly computed key.
       */
      const key = computeKey(sourceFile,);
      keyByFile.set(
        sourceFile.fileName,
        key,
      );
      if (key.eligible) {
        recordSharedEligible({
          fileName: sourceFile.fileName,
          project: configuredProjectConfigFileName({ project, },),
          digest: key.digest,
          parts: key.parts,
        },);
      }
      else {
        recordSharedFallback({
          fileName: sourceFile.fileName,
          reason: key.reason,
          ...(key.detail === undefined) ? {} : { detail: key.detail, },
        },);
      }
      return key;
    },
  };
}
