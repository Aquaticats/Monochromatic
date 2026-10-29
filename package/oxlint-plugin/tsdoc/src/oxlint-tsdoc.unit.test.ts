import { readFileSync, } from 'node:fs';
import { resolve, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createTempFixtureFile,
  fixtureConfigPath,
  fixtureSourceRoot,
  type OxlintRuleDiagnostic as OxlintDiagnostic,
  resolveFixtureTarget,
  runOxlintFixture,
  uniqueRuleCodes as uniqueRules,
} from '@monochromatic-dev/oxlint-plugin-test-support/ts';

//region Helpers

/** Fixture source root. */
const FIXTURES = fixtureSourceRoot({
  fixturePackageName: 'oxlint-tsdoc',
},);

/**
 Fixture-specific oxlint config with all tsdoc rules enabled and no ignorePatterns
 that would skip test-fixture or invalid paths.
 */
const FIXTURE_CONFIG = fixtureConfigPath({
  fixturePackageName: 'oxlint-tsdoc',
  fileName: '.oxlintrc.fixture.json',
},);

/** Diagnostic message emitted by tsdoc/multiline-blocks. */
const MULTILINE_BLOCKS_MESSAGE = 'TSDoc comments must use multiline format.';

/**
 Runs oxlint with the project config against a fixture path and returns parsed diagnostics.
 
 @param fixturePath - path relative to fixture root, or absolute temp fixture path
 
 @returns array of diagnostics from tsdoc rules only
 */
async function lint(fixturePath: string,): Promise<readonly OxlintDiagnostic[]> {
  /** Resolved lint target; temp fixtures already arrive as absolute paths. */
  const target = resolveFixtureTarget({
    fixtureSourceRoot: FIXTURES,
    fixturePath,
  },);

  return runOxlintFixture({
    codePrefix: 'tsdoc(',
    configFlag: '--config',
    fixtureConfig: FIXTURE_CONFIG,
    target,
  },);
}

/**
 Extracts unique rule codes from a set of diagnostics.
 
 @param diagnostics - array of oxlint diagnostics
 
 @returns sorted array of unique `tsdoc(rule-name)` codes
 */
//endregion Helpers

await describe({
  name: '',
  children: [
    //region Valid fixtures: expect zero tsdoc violations

    describe({
      name: 'valid fixtures',
      children: [
        it({
          name: 'fully documented declarations produce no violations',
          fn: async () => {
            const diagnostics = await lint('valid/documented-declarations.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'complete TSDoc with params, returns, yields produce no violations',
          fn: async () => {
            const diagnostics = await lint('valid/complete-tsdoc.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: '.test.ts files are ignored by tsdoc rules',
          fn: async () => {
            const diagnostics = await lint('valid/ignored-extensions.test.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'documented local declarations produce no violations',
          fn: async () => {
            const diagnostics = await lint('valid/documented-locals.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
      ],
    },),

    //endregion Valid fixtures

    //region Invalid fixtures: expect specific violations

    describe({
      name: 'require-tsdoc',
      children: [
        it({
          name: 'reports missing TSDoc on undocumented declarations',
          fn: async () => {
            const diagnostics = await lint('invalid/missing-tsdoc.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('tsdoc(require-tsdoc)',);

            // Every diagnostic should be require-tsdoc since file has no TSDoc at all
            const requireTsdocCount = diagnostics
              .filter(
              function isRequireTsdoc(d,): boolean {
                return d.code === 'tsdoc(require-tsdoc)';
              },
              )
              .length;
            // function, arrow, class, type, interface, enum, const = at least 7
            expect(requireTsdocCount,).toBeGreaterThanOrEqual(7,);
          },
        },),
        it({
          name:
            'reports missing TSDoc on undocumented locals while exempting for-loop bindings',
          fn: async () => {
            const diagnostics = await lint('invalid/missing-tsdoc-locals.ts',);
            const requireTsdocDiags = diagnostics.filter(
              function isRequireTsdoc(d,): boolean {
                return d.code === 'tsdoc(require-tsdoc)';
              },
            );
            // out, count, next, empty: four locals; for-loop `const v` is exempt; outer fn has TSDoc.
            const EXPECTED_LOCAL_VIOLATIONS = 4;
            expect(requireTsdocDiags.length,).toBe(EXPECTED_LOCAL_VIOLATIONS,);
          },
        },),
      ],
    },),
    describe({
      name: 'structural rules',
      children: [
        it({
          name: 'reports structural formatting issues',
          fn: async () => {
            const diagnostics = await lint('invalid/structural-issues.ts',);
            const rules = uniqueRules(diagnostics,);

            expect(rules,).toContain('tsdoc(multiline-blocks)',);
            expect(rules,).toContain('tsdoc(no-multi-asterisks)',);
            expect(rules,).toContain('tsdoc(tag-lines)',);
            expect(rules,).toContain('tsdoc(empty-tags)',);

            const multilineDiagnostics = diagnostics.filter(
              function isMultilineBlocks(diagnostic,): boolean {
                return diagnostic.code === 'tsdoc(multiline-blocks)';
              },
            );
            expect(multilineDiagnostics.length,).toBe(3,);
            expect(multilineDiagnostics.map(function getMessage(diagnostic,): string {
              return diagnostic.message;
            },),).toEqual([
              MULTILINE_BLOCKS_MESSAGE,
              MULTILINE_BLOCKS_MESSAGE,
              MULTILINE_BLOCKS_MESSAGE,
            ],);
          },
        },),
      ],
    },),
    describe({
      name: 'autofix',
      children: [
        it({
          name: '--fix expands single-line TSDoc blocks to star-less bodies',
          fn: async () => {
            const fixableSrc = resolve(
              FIXTURES,
              'invalid',
              'single-line-tsdoc-fixable.ts',
            );
            await using fixableCopy = await createTempFixtureFile({
              fileName: 'single-line-tsdoc-fixable.ts',
              sourcePath: fixableSrc,
              tempPrefix: 'oxlint-tsdoc-autofix-',
            },);

            await runOxlintFixture({
              codePrefix: 'tsdoc(',
              configFlag: '--config',
              fixtureConfig: FIXTURE_CONFIG,
              target: fixableCopy.filePath,
              fix: 'fixes',
            },);

            const fixedContent = readFileSync(fixableCopy.filePath, 'utf8',);
            expect(fixedContent,).toContain('/**\n Description only.\n */',);
            expect(fixedContent,).toContain('/**\n @returns value\n */',);
            expect(fixedContent,).toContain(
              '  /**\n   Inner description.\n   */\n  const value = true;',
            );
            expect(fixedContent,).toContain(
              'type PropertyFixture = {\n  /**\n   Property description.\n   */\n  readonly value: string;\n};',
            );
            expect(fixedContent,).toContain('/**\n \n */\nconst emptyDoc = true;',);

            const diagnostics = await lint(fixableCopy.filePath,);
            expect(diagnostics,).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: 'tag validation rules',
      children: [
        it({
          name: 'reports invalid tag names and type annotations',
          fn: async () => {
            const diagnostics = await lint('invalid/tag-validation-issues.ts',);
            const rules = uniqueRules(diagnostics,);

            expect(rules,).toContain('tsdoc(check-tag-names)',);
            expect(rules,).toContain('tsdoc(check-access)',);
            expect(rules,).toContain('tsdoc(no-types)',);
          },
        },),
        it({
          name: 'reports specific JSDoc-only tags',
          fn: async () => {
            const diagnostics = await lint('invalid/tag-validation-issues.ts',);
            const tagNameDiags = diagnostics.filter(
              function isCheckTagNames(d,): boolean {
                return d.code === 'tsdoc(check-tag-names)';
              },
            );

            const messages = tagNameDiags.map(function getMsg(d,): string {
              return d.message;
            },);
            // @type, @typedef, @return, @foobar should all be flagged
            expect(messages.some(function hasType(m,): boolean {
              return m.includes('@type',);
            },),)
              .toBe(true,);
            expect(messages.some(function hasTypedef(m,): boolean {
              return m.includes('@typedef',);
            },),)
              .toBe(true,);
            expect(messages.some(function hasReturn(m,): boolean {
              return m.includes('@return',);
            },),)
              .toBe(true,);
            expect(messages.some(function hasFoobar(m,): boolean {
              return m.includes('@foobar',);
            },),)
              .toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: 'param rules',
      children: [
        it({
          name: 'reports parameter documentation issues',
          fn: async () => {
            const diagnostics = await lint('invalid/param-issues.ts',);
            const rules = uniqueRules(diagnostics,);

            expect(rules,).toContain('tsdoc(check-param-names)',);
            expect(rules,).toContain('tsdoc(require-param)',);
            expect(rules,).toContain('tsdoc(require-param-description)',);
          },
        },),
      ],
    },),
    describe({
      name: 'mutation contract rules',
      children: [
        it({
          name: 'reports malformed mutation contracts',
          fn: async () => {
            const diagnostics = await lint('invalid/mutates-issues.ts',);
            const mutatesDiagnostics = diagnostics.filter(
              function isCheckMutates(diagnostic,): boolean {
                return diagnostic.code === 'tsdoc(check-mutates)';
              },
            );

            expect(mutatesDiagnostics,).toHaveLength(4,);
            expect(mutatesDiagnostics.map(function message(diagnostic,): string {
              return diagnostic.message;
            },),).toEqual([
              'Mutation contract must name a parameter.',
              'Mutation contract for "value" must include a description.',
              'Mutation contract target "other" does not match any parameter.',
              'Mutation contract target "value" is duplicated.',
            ],);
          },
        },),
        it({
          name: 'accepts named and destructured mutation targets',
          fn: async () => {
            const diagnostics = await lint('valid/mutates-contracts.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'validates bodyless method, call, and ambient signatures',
          fn: async () => {
            const diagnostics = await lint('invalid/mutates-signature-issues.ts',);
            const mutatesDiagnostics = diagnostics.filter(
              function isCheckMutates(diagnostic,): boolean {
                return diagnostic.code === 'tsdoc(check-mutates)';
              },
            );
            expect(mutatesDiagnostics,).toHaveLength(3,);
          },
        },),
        it({
          name: 'accepts valid bodyless method and call signatures',
          fn: async () => {
            const diagnostics = await lint('valid/mutates-signatures.ts',);
            const mutatesDiagnostics = diagnostics.filter(
              function isCheckMutates(diagnostic,): boolean {
                return diagnostic.code === 'tsdoc(check-mutates)';
              },
            );
            expect(mutatesDiagnostics,).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: 'returns rules',
      children: [
        it({
          name: 'reports return documentation issues',
          fn: async () => {
            const diagnostics = await lint('invalid/returns-issues.ts',);
            const rules = uniqueRules(diagnostics,);

            expect(rules,).toContain('tsdoc(require-returns)',);
            expect(rules,).toContain('tsdoc(require-returns-check)',);
            expect(rules,).toContain('tsdoc(require-returns-description)',);
          },
        },),
      ],
    },),
    describe({
      name: 'yields rules',
      children: [
        it({
          name: 'reports yield documentation issues',
          fn: async () => {
            const diagnostics = await lint('invalid/yields-issues.ts',);
            const rules = uniqueRules(diagnostics,);

            expect(rules,).toContain('tsdoc(require-yields)',);
            expect(rules,).toContain('tsdoc(require-yields-check)',);
          },
        },),
      ],
    },),

    describe({
      name: 'require-example',
      children: [
        it({
          name: 'reports missing @example on exported functions',
          fn: async () => {
            const diagnostics = await lint('invalid/require-example-issues.ts',);
            const rules = uniqueRules(diagnostics,);

            expect(rules,).toContain('tsdoc(require-example)',);

            const requireExampleDiags = diagnostics.filter(
              function isRequireExample(d,): boolean {
                return d.code === 'tsdoc(require-example)';
              },
            );
            // add (direct), greet (specifier), negate (default), double (const direct), shout (const specifier)
            expect(requireExampleDiags.length,).toBeGreaterThanOrEqual(5,);
          },
        },),
        it({
          name: 'does not report on functions with @example or exempt tags',
          fn: async () => {
            const diagnostics = await lint('valid/require-example-valid.ts',);
            const requireExampleDiags = diagnostics.filter(
              function isRequireExample(d,): boolean {
                return d.code === 'tsdoc(require-example)';
              },
            );
            expect(requireExampleDiags,).toEqual([],);
          },
        },),
      ],
    },),
    //endregion Invalid fixtures
  ],
},);
