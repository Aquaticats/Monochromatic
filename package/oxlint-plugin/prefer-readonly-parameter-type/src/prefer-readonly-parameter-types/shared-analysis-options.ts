/**
 Compiler option classification for the cross-project analysis key.
 
 @module
 */

import { createHash, } from 'node:crypto';

import {
  updateHashPlainValue,
  updateHashString,
} from './effect-hash-value.ts';

/**
 Options whose resolved values name project-specific paths or only steer module resolution.
 
 Their effect on one file's analysis reaches it through which files its module references
 resolve to, which the dependency closure records file by file, and through which declaration
 files the program loads, which the ambient surface records. Leaving them out of the key is
 what lets two projects whose `paths` or output directories differ share an entry whose
 resolutions agree.
 */
const RESOLUTION_ONLY_OPTIONS: ReadonlySet<string> = new Set([
  'baseUrl',
  'configFilePath',
  'declarationDir',
  'mapRoot',
  'outDir',
  'paths',
  'pathsBasePath',
  'project',
  'rootDir',
  'rootDirs',
  'sourceRoot',
  'tsBuildInfoFile',
  'typeRoots',
],);

/**
 Options whose resolved values are part of the key.
 
 Checking options belong here because they change what the checker concludes. Emit and
 diagnostic options are keyed as well, although no analysis input is known to read them:
 keying a value that is equal in every project costs nothing, and it keeps the unknown-option
 refusal below for names nobody has classified.
 */
const KEYED_OPTIONS: ReadonlySet<string> = new Set([
  'allowArbitraryExtensions',
  'allowImportingTsExtensions',
  'allowJs',
  'allowNonTsExtensions',
  'allowSyntheticDefaultImports',
  'allowUmdGlobalAccess',
  'allowUnreachableCode',
  'allowUnusedLabels',
  'assumeChangesOnlyAffectDirectDependencies',
  'checkJs',
  'composite',
  'customConditions',
  'declaration',
  'declarationMap',
  'deduplicatePackages',
  'diagnostics',
  'disableReferencedProjectLoad',
  'disableSizeLimit',
  'disableSolutionSearching',
  'disableSourceOfProjectReferenceRedirect',
  'downlevelIteration',
  'emitBOM',
  'emitDecoratorMetadata',
  'emitDeclarationOnly',
  'erasableSyntaxOnly',
  'esModuleInterop',
  'exactOptionalPropertyTypes',
  'experimentalDecorators',
  'explainFiles',
  'extendedDiagnostics',
  'forceConsistentCasingInFileNames',
  'ignoreConfig',
  'ignoreDeprecations',
  'importHelpers',
  'incremental',
  'init',
  'inlineSourceMap',
  'inlineSources',
  'isolatedDeclarations',
  'isolatedModules',
  'jsx',
  'jsxFactory',
  'jsxFragmentFactory',
  'jsxImportSource',
  'lib',
  'libReplacement',
  'listEmittedFiles',
  'listFiles',
  'locale',
  'maxNodeModuleJsDepth',
  'module',
  'moduleDetection',
  'moduleResolution',
  'moduleSuffixes',
  'newLine',
  'noCheck',
  'noEmit',
  'noEmitHelpers',
  'noEmitOnError',
  'noErrorTruncation',
  'noFallthroughCasesInSwitch',
  'noImplicitAny',
  'noImplicitOverride',
  'noImplicitReturns',
  'noImplicitThis',
  'noLib',
  'noPropertyAccessFromIndexSignature',
  'noResolve',
  'noUncheckedIndexedAccess',
  'noUncheckedSideEffectImports',
  'noUnusedLocals',
  'noUnusedParameters',
  'preserveConstEnums',
  'preserveSymlinks',
  'preserveWatchOutput',
  'pretty',
  'reactNamespace',
  'removeComments',
  'resolveJsonModule',
  'resolvePackageJsonExports',
  'resolvePackageJsonImports',
  'rewriteRelativeImportExtensions',
  'skipDefaultLibCheck',
  'skipLibCheck',
  'sourceMap',
  'stableTypeOrdering',
  'strict',
  'strictBindCallApply',
  'strictBuiltinIteratorReturn',
  'strictFunctionTypes',
  'strictNullChecks',
  'strictPropertyInitialization',
  'stripInternal',
  'suppressOutputPathCheck',
  'target',
  'traceResolution',
  'types',
  'useDefineForClassFields',
  'useUnknownInCatchVariables',
  'verbatimModuleSyntax',
],);

/**
 Outcome of classifying a project's resolved compiler options.
 
 An unclassified option makes the whole project refuse sharing, so a TypeScript option added
 after this list was written cannot be shared by accident.
 */
export type CheckingOptionsKey =
  | {
    readonly eligible: true;
    readonly digest: string;
  }
    | {
      readonly eligible: false;
      readonly reason: string;
    };

/**
 Digests the keyed compiler options, or names the first option nobody classified.
 
 @param compilerOptions - Resolved compiler options as the TypeScript API reports them.
 
 @returns digest over the keyed options that are set,
 or the reason sharing is refused.
 
 @example
 ```ts
 const key = checkingOptionsKey({ strict: true });
 ```
 */
export function checkingOptionsKey(compilerOptions: object,): CheckingOptionsKey {
  /**
   Option values by name.
   */
  const options: Readonly<Record<string, unknown>> = Object.fromEntries(Object.entries(compilerOptions,),);
  /**
   Option names the project sets, in a stable order.
   */
  const names = Object.keys(options,)
    .filter(function isSet(name,): boolean {
      return options[name] !== undefined;
    },)
    .toSorted();
  /**
   First option on neither list.
   */
  const unclassified = names.find(function isUnclassified(name,): boolean {
    return (!KEYED_OPTIONS.has(name,)) && (!RESOLUTION_ONLY_OPTIONS.has(name,));
  },);
  if (unclassified !== undefined) {
    return {
      eligible: false,
      reason: `unknown-option:${unclassified}`,
    };
  }
  /**
   Digest over keyed option names and values.
   */
  const digest = createHash('sha256',);
  names
    .filter(function isKeyed(name,): boolean {
      return KEYED_OPTIONS.has(name,);
    },)
    .forEach(function hashOption(name,): void {
      updateHashString({
        digest,
        value: name,
      },);
      updateHashPlainValue({
        digest,
        value: options[name],
      },);
    },);
  return {
    eligible: true,
    digest: digest.digest('hex',),
  };
}
