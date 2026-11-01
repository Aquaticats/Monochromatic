import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  fixtureConfigPath,
  fixturePackageRoot,
  type OxlintRuleDiagnostic,
  runOxlintFixture,
} from '@monochromatic-dev/oxlint-plugin-test-support/ts';
import { resolve, } from 'node:path';

/** Fixture package name under `package/test-fixture`. */
const FIXTURE_PACKAGE_NAME = 'oxlint-test-import';

/** Diagnostic code prefix identifying this plugin's findings. */
const CODE_PREFIX = 'test-import(';

/** Absolute path to the fixture oxlint config. */
const FIXTURE_CONFIG = fixtureConfigPath({
  fixturePackageName: FIXTURE_PACKAGE_NAME,
  fileName: '.oxlintrc.fixture.json',
},);

/** Absolute root of the nested pseudo-packages. */
const CASE_ROOT = resolve(
  fixturePackageRoot({ fixturePackageName: FIXTURE_PACKAGE_NAME, },),
  'case',
);

/**
 Absolute path to fixture config emptying `fixturePatterns`.
 
 Serves as positive control that rule options are read at all: with no globs
 exempting anything, an import the default list allows must become a finding.
 */
const NO_FIXTURES_CONFIG = fixtureConfigPath({
  fixturePackageName: FIXTURE_PACKAGE_NAME,
  fileName: '.oxlintrc.no-fixtures.fixture.json',
},);

/**
 Runs the rule over one fixture file through a real oxlint process.
 
 @param caseName - nested pseudo-package directory name
 
 @param fileName - file inside that case's `src`
 
 @param fixtureConfig - oxlint config driving this run; defaults to unconfigured rule
 
 @returns diagnostics this plugin emitted for that file
 
 @example
 ```ts
 await lintCase({ caseName: 'standard', fileName: 'rejected.test.ts' });
 ```
 */
async function lintCase({
  caseName,
  fileName,
  fixtureConfig = FIXTURE_CONFIG,
}: {
  /**
   Nested pseudo-package directory name.
   */
  readonly caseName: string;
  /**
   File inside that case's `src`.
   */
  readonly fileName: string;
  /**
   Oxlint config driving this run.
   */
  readonly fixtureConfig?: string;
},): Promise<readonly OxlintRuleDiagnostic[]> {
  return await runOxlintFixture({
    codePrefix: CODE_PREFIX,
    configFlag: '--config',
    fixtureConfig,
    target: resolve(
      CASE_ROOT,
      caseName,
      'src',
      fileName,
    ),
  },);
}

await describe({
  name: 'require-eventual-artifact end to end',
  children: [
    it({
      name: 'reports nothing for every allowed import form',
      fn: async () => {
        expect(await lintCase({
          caseName: 'standard',
          fileName: 'allowed.test.ts',
        },),).toEqual([],);
      },
    },),
    it({
      name: 'honors a configured fixturePatterns list rather than always using defaults',
      fn: async () => {
        // Positive control. `allowed.test.ts` imports `./fixture.data.ts`, exempt only
        // because `**/fixture.*` sits in the default glob list. Emptying the option must
        // move that import from allowed to reported. A rule that reads `context.options`
        // inside `createOnce`, where oxlint leaves it null, silently keeps the defaults
        // and reports nothing here, which is the failure this case exists to catch.
        expect((await lintCase({
          caseName: 'standard',
          fileName: 'allowed.test.ts',
          fixtureConfig: NO_FIXTURES_CONFIG,
        },)).length,).toBe(1,);
      },
    },),
    it({
      name: 'reports every rejected import form, including type-only imports',
      fn: async () => {
        /** Diagnostics for the file exercising all four rejected forms. */
        const diagnostics = await lintCase({
          caseName: 'standard',
          fileName: 'rejected.test.ts',
        },);
        expect(diagnostics.length,).toBe(4,);
      },
    },),
    it({
      name: 'names the offending specifier in the message',
      fn: async () => {
        /** Diagnostics for the file exercising all four rejected forms. */
        const diagnostics = await lintCase({
          caseName: 'standard',
          fileName: 'rejected.test.ts',
        },);
        expect(diagnostics.some(function mentionsSpecifier(diagnostic,): boolean {
          return diagnostic.message
            .includes('`./parse.ts`',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'checks allowlisted modules themselves, closing the re-export laundering path',
      fn: async () => {
        expect((await lintCase({
          caseName: 'standard',
          fileName: 'test-support.ts',
        },)).length,).toBe(1,);
      },
    },),
    it({
      name: 'allows siblings of a main entry outside dist/final',
      fn: async () => {
        /** Diagnostics for the Electron-shaped case. */
        const diagnostics = await lintCase({
          caseName: 'electron',
          fileName: 'entry-directory.test.ts',
        },);
        expect(diagnostics.length,).toBe(1,);
      },
    },),
    it({
      name: 'never blesses the bare dist root from a single asset entry',
      fn: async () => {
        /** Diagnostics for the asset-entry case. */
        const diagnostics = await lintCase({
          caseName: 'asset-dist',
          fileName: 'bare-dist-root.test.ts',
        },);
        expect(diagnostics.length,).toBe(2,);
      },
    },),
    it({
      name: 'discards manifest entries pointing into source',
      fn: async () => {
        /** Diagnostics for the source-entry case. */
        const diagnostics = await lintCase({
          caseName: 'src-entry',
          fileName: 'discarded-src-entry.test.ts',
        },);
        expect(diagnostics.length,).toBe(2,);
      },
    },),
    it({
      name: 'exempts a package declaring no build task',
      fn: async () => {
        expect(await lintCase({
          caseName: 'buildless',
          fileName: 'exempt.test.ts',
        },),).toEqual([],);
      },
    },),
  ],
},);
