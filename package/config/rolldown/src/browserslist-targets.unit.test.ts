import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  browserslistTargets,
  type BrowserslistImport,
  type BrowserslistTargetsOptions,
} from './browserslist-targets.ts';

//region Fixtures: generated and fallback target sources

/**
 File URL used for generated JSON import tests.
 
 @example
 ```ts
 const href = GENERATED_URL.href;
 ```
 */
const GENERATED_URL = new URL('file:///tmp/.browserslistrc.resolved.local.json',);

/**
 Targets returned by generated JSON fixture.
 
 @example
 ```ts
 expect(GENERATED_TARGETS).toContain('firefox 140');
 ```
 */
const GENERATED_TARGETS = [
  'and_chr 148',
  'android 148',
  'chrome 148',
  'firefox 140',
  'node 25.2.1',
] as const;

/**
 Rolldown-compatible targets expected from generated JSON fixture.
 
 @example
 ```ts
 expect(EXPECTED_GENERATED_TARGETS).toContain('chrome148');
 ```
 */
const EXPECTED_GENERATED_TARGETS = [
  'chrome148',
  'firefox140',
] as const;

/**
 Targets returned by Browserslist fallback fixture.
 
 @example
 ```ts
 expect(FALLBACK_TARGETS).toContain('node 25.2.1');
 ```
 */
const FALLBACK_TARGETS = [
  'node 25.2.1',
  'node 25.0.0',
] as const;

/**
 Rolldown-compatible targets expected from Browserslist fallback fixture.
 
 @example
 ```ts
 expect(EXPECTED_FALLBACK_TARGETS).toContain('node25.2.1');
 ```
 */
const EXPECTED_FALLBACK_TARGETS = [
  'node25.0.0',
] as const;

//endregion Fixtures: generated and fallback target sources

//region Fixture helpers: generated-file branch

/**
 Reports generated JSON as present.
 
 @param _fileUrl - Generated URL ignored by deterministic fixture.
 @returns Always true for generated-file branch tests.
 @example
 ```ts
 await generatedExists(GENERATED_URL);
 ```
 */
async function generatedExists(_fileUrl: URL,): Promise<boolean> {
  return true;
}

/**
 Reports generated JSON as absent.
 
 @param _fileUrl - Generated URL ignored by deterministic fixture.
 @returns Always false for fallback branch tests.
 @example
 ```ts
 await generatedMissing(GENERATED_URL);
 ```
 */
async function generatedMissing(_fileUrl: URL,): Promise<boolean> {
  return false;
}

/**
 Imports valid generated target JSON.
 
 @param _fileUrl - Generated URL ignored by deterministic fixture.
 @returns JSON namespace containing generated target array.
 @example
 ```ts
 const imported = await importGeneratedTargets(GENERATED_URL);
 ```
 */
async function importGeneratedTargets(_fileUrl: URL,): Promise<unknown> {
  return GENERATED_TARGETS;
}

/**
 Imports malformed generated target JSON.
 
 @param _fileUrl - Generated URL ignored by deterministic fixture.
 @returns JSON namespace whose default export is not string array.
 @example
 ```ts
 const imported = await importMalformedTargets(GENERATED_URL);
 ```
 */
async function importMalformedTargets(_fileUrl: URL,): Promise<unknown> {
  return { firefox: 140, };
}

/**
 Fails if Browserslist fallback is reached during generated-file tests.
 
 @returns Never returns because generated JSON should short-circuit fallback.
 @throws Error when fallback path runs unexpectedly.
 @example
 ```ts
 await importUnexpectedBrowserslist();
 ```
 */
async function importUnexpectedBrowserslist(): Promise<BrowserslistImport> {
  throw new Error('Browserslist fallback should not run when generated JSON exists.',);
}

//endregion Fixture helpers: generated-file branch

//region Fixture helpers: fallback branch

/**
 Browserslist resolver fixture matching package call signature.
 
 @returns Fallback target array.
 @example
 ```ts
 fallbackBrowserslistResolver();
 ```
 */
function fallbackBrowserslistResolver(): string[] {
  return [...FALLBACK_TARGETS,];
}

/**
 Imports Browserslist as default-export namespace.
 
 @returns Namespace shape produced by Node ESM dynamic import of CommonJS.
 @example
 ```ts
 const imported = await importFallbackBrowserslistDefault();
 ```
 */
async function importFallbackBrowserslistDefault(): Promise<BrowserslistImport> {
  return { default: fallbackBrowserslistResolver, };
}

/**
 Imports Browserslist as callable namespace.
 
 @returns Callable shape accepted for runtimes that expose CommonJS directly.
 @example
 ```ts
 const imported = await importFallbackBrowserslistCallable();
 ```
 */
async function importFallbackBrowserslistCallable(): Promise<BrowserslistImport> {
  return fallbackBrowserslistResolver;
}

/**
 Fails if generated JSON import is reached during fallback tests.
 
 @param _fileUrl - Generated URL ignored by deterministic fixture.
 @returns Never returns because missing generated JSON should skip import.
 @throws Error when generated JSON importer runs unexpectedly.
 @example
 ```ts
 await importUnexpectedJson(GENERATED_URL);
 ```
 */
async function importUnexpectedJson(_fileUrl: URL,): Promise<unknown> {
  throw new Error('Generated JSON import should not run when file is absent.',);
}

//endregion Fixture helpers: fallback branch

//region Assertion helpers

/**
 Captures error thrown by Browserslist target resolution.
 
 @param options - Resolution options expected to throw.
 @returns Error thrown by resolution.
 @throws Error when resolution succeeds unexpectedly.
 @example
 ```ts
 const error = await captureBrowserslistTargetsError({ exists: generatedExists });
 ```
 */
async function captureBrowserslistTargetsError(
  options: BrowserslistTargetsOptions,
): Promise<unknown> {
  try {
    await browserslistTargets(options,);
  }
  catch (error) {
    return error;
  }

  throw new Error('Expected browserslistTargets to throw.',);
}

//endregion Assertion helpers

await describe({
  name: browserslistTargets.name,
  children: [
    //region Generated JSON branch

    it({
      name: 'uses generated JSON when local file exists',
      fn: async function usesGeneratedJson(): Promise<void> {
        const targets = await browserslistTargets({
          exists: generatedExists,
          generatedFileUrl: GENERATED_URL,
          importBrowserslist: importUnexpectedBrowserslist,
          importJson: importGeneratedTargets,
        },);

        expect(targets,).toEqual(EXPECTED_GENERATED_TARGETS,);
      },
    },),

    it({
      name: 'rejects malformed generated JSON instead of falling back',
      fn: async function rejectsMalformedGeneratedJson(): Promise<void> {
        const error = await captureBrowserslistTargetsError({
          exists: generatedExists,
          generatedFileUrl: GENERATED_URL,
          importBrowserslist: importUnexpectedBrowserslist,
          importJson: importMalformedTargets,
        },);

        expect(error,).toBeInstanceOf(TypeError,);
        expect((error as Error).message,).toContain(GENERATED_URL.href,);
      },
    },),

    //endregion Generated JSON branch

    //region Browserslist package fallback

    it({
      name: 'falls back to Browserslist default export when generated JSON is absent',
      fn: async function fallsBackToDefaultExport(): Promise<void> {
        const targets = await browserslistTargets({
          exists: generatedMissing,
          generatedFileUrl: GENERATED_URL,
          importBrowserslist: importFallbackBrowserslistDefault,
          importJson: importUnexpectedJson,
          runtime: 'node',
        },);

        expect(targets,).toEqual(EXPECTED_FALLBACK_TARGETS,);
      },
    },),

    it({
      name: 'falls back to callable Browserslist import when generated JSON is absent',
      fn: async function fallsBackToCallableImport(): Promise<void> {
        const targets = await browserslistTargets({
          exists: generatedMissing,
          generatedFileUrl: GENERATED_URL,
          importBrowserslist: importFallbackBrowserslistCallable,
          importJson: importUnexpectedJson,
          runtime: 'node',
        },);

        expect(targets,).toEqual(EXPECTED_FALLBACK_TARGETS,);
      },
    },),

    it({
      name: 'falls back to actual Browserslist package when generated JSON is absent',
      fn: async function fallsBackToActualBrowserslist(): Promise<void> {
        const targets = await browserslistTargets({
          exists: generatedMissing,
          generatedFileUrl: GENERATED_URL,
          importJson: importUnexpectedJson,
        },);

        expect(targets.length,).toBeGreaterThan(0,);
        expect(targets.some(function includesSpace(target,): boolean {
          return target.includes(' ',);
        },),).toBe(false,);
        expect(targets.some(function isNodeTarget(target,): boolean {
          return target.startsWith('node',);
        },),).toBe(false,);
      },
    },),

    //endregion Browserslist package fallback
  ],
},);
