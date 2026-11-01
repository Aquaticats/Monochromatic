import { readFileSync, } from 'node:fs';
import { resolve, } from 'node:path';

import type { Context, } from '@oxlint/plugins';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn from 'nano-spawn';

import {
  createTempFixtureFile,
  fixtureConfigPath,
  fixturePackageRoot,
  fixtureSourceRoot,
  type OxlintRuleDiagnostic as OxlintDiagnostic,
  OXLINT_PLUGIN_TEST_ROOT as ROOT,
  resolveFixtureTarget,
  runOxlintFixture,
  uniqueRuleCodes as uniqueRules,
} from '@monochromatic-dev/oxlint-plugin-test-support/ts';

import {
  type ChainNode,
  chainBreaks,
} from '../dist/final/node/index.mjs';

/** Minimal token stub the chain walk reads: a value to classify and a start offset. */
type TokenStub = {
  /** Token text, classified against `.`/`(`/`)`/operator by the walk. */
  readonly value: string;
  /** Byte offset of the token's start. */
  readonly start: number;
};

/** Inputs for linting one fixture with a selected configuration. */
type LintWithConfigParams = {
  /** Path relative to fixture source root, or absolute temp fixture path. */
  readonly fixturePath: string;
  /** Absolute fixture configuration path. */
  readonly fixtureConfig: string;
};

//endregion Types

//region Helpers

/** Fixture package root. */
const FIXTURE_PKG = fixturePackageRoot({
  fixturePackageName: 'oxlint-stylistic',
},);

/** Fixture source root. */
const FIXTURES = fixtureSourceRoot({
  fixturePackageName: 'oxlint-stylistic',
},);

/**
 Fixture-specific oxlint config with all stylistic rules enabled and no
 ignorePatterns that would skip test-fixture or invalid paths.
 */
const FIXTURE_CONFIG = fixtureConfigPath({
  fixturePackageName: 'oxlint-stylistic',
  fileName: '.oxlintrc.fixture.json',
},);

/** Fixture config that intentionally passes eslint.style-style semi options. */
const SEMI_CONFIGURED_FIXTURE_CONFIG = fixtureConfigPath({
  fixturePackageName: 'oxlint-stylistic',
  fileName: '.oxlintrc.semi-configured.fixture.json',
},);

/** Fixture config that intentionally passes eslint.style-style comma-dangle options. */
const COMMA_DANGLE_CONFIGURED_FIXTURE_CONFIG = fixtureConfigPath({
  fixturePackageName: 'oxlint-stylistic',
  fileName: '.oxlintrc.comma-dangle-configured.fixture.json',
},);

/** Fixture config selecting always mode for TSDoc asterisk prefixes. */
const ASTERISK_PREFIX_ALWAYS_FIXTURE_CONFIG = fixtureConfigPath({
  fixturePackageName: 'oxlint-stylistic',
  fileName: '.oxlintrc.asterisk-prefix-always.fixture.json',
},);

/** Fixture config omitting required TSDoc asterisk-prefix mode. */
const ASTERISK_PREFIX_MISSING_FIXTURE_CONFIG = fixtureConfigPath({
  fixturePackageName: 'oxlint-stylistic',
  fileName: '.oxlintrc.asterisk-prefix-missing.fixture.json',
},);

/** Fixture config supplying unsupported TSDoc asterisk-prefix mode. */
const ASTERISK_PREFIX_INVALID_FIXTURE_CONFIG = fixtureConfigPath({
  fixturePackageName: 'oxlint-stylistic',
  fileName: '.oxlintrc.asterisk-prefix-invalid.fixture.json',
},);

/** Maximum autofix passes needed for overlapping stylistic fixes to converge. */
const MAX_AUTOFIX_PASSES = 8;

/**
 Runs oxlint against one fixture with selected config.
 
 @param params - fixture path and configuration
 
 @returns array of diagnostics from stylistic rules only
 */
async function lintWithConfig(params: Readonly<LintWithConfigParams>,): Promise<readonly OxlintDiagnostic[]> {
  /** Selected fixture path and configuration. */
  const {
    fixturePath,
    fixtureConfig,
  } = params;
  /** Resolved lint target; temp fixtures already arrive as absolute paths. */
  const target = resolveFixtureTarget({
    fixtureSourceRoot: FIXTURES,
    fixturePath,
  },);

  return runOxlintFixture({
    codePrefix: 'stylistic(',
    configFlag: '--config',
    fixtureConfig,
    target,
  },);
}

/**
 Runs oxlint with complete fixture config against one fixture path.
 
 @param fixturePath - path relative to fixture `src/` root, or absolute temp path
 
 @returns array of diagnostics from stylistic rules only
 */
async function lint(fixturePath: string,): Promise<readonly OxlintDiagnostic[]> {
  return lintWithConfig({
    fixturePath,
    fixtureConfig: FIXTURE_CONFIG,
  },);
}


/**
 Runs oxlint --fix once on a fixture file.
 
 Oxlint exits non-zero while unfixable findings remain, which is expected here; a plugin that crashed is not,
 and fails the run, so a file read back afterwards was never left alone by a crash.
 
 @param params - file to rewrite and the config to fix it under
 
 @example
 ```ts
 await fixOnce({ filePath: fixture.filePath, fixtureConfig: FIXTURE_CONFIG });
 ```
 */
async function fixOnce(
  {
    filePath,
    fixtureConfig,
  }: {
    readonly filePath: string;
    readonly fixtureConfig: string;
  },
): Promise<void> {
  await runOxlintFixture({
    codePrefix: 'stylistic(',
    configFlag: '--config',
    fixtureConfig,
    target: filePath,
    fix: 'fixes',
  },);
}

/**
 Runs oxlint --fix on a fixture until content stops changing.
 
 Some stylistic fixes overlap, so oxlint applies only one fix for that source
 region per pass. Repeating until stable exercises the same boundary while
 avoiding fixture mutation.
 
 @param filePath - absolute path to temp fixture copy
 
 @example
 ```ts
 await fixUntilStable(fixture.filePath);
 ```
 */
async function fixUntilStable(filePath: string,): Promise<void> {
  for (
    let pass = 0;
    pass < MAX_AUTOFIX_PASSES;
    pass += 1
  ) {
    /** File content before this pass; unchanged output means convergence. */
    const before = readFileSync(filePath, 'utf8',);
    // oxlint-disable-next-line eslint/no-await-in-loop -- each pass must read previous pass output from disk
    await fixOnce({
      filePath,
      fixtureConfig: FIXTURE_CONFIG,
    },);
    if (readFileSync(filePath, 'utf8',) === before)
      return;
  }
}

/**
 Casts a structural stub to a `ChainNode` for synthetic-AST construction.
 
 The chain walk reads only `type`, the receiver links, `start`/`end`, and a
 member's `property`/`computed`; a hand-built stub supplies exactly those, so
 one assertion at the boundary keeps the builders free of per-field casts.
 
 @param stub - object carrying the chain-walk fields for one node
 
 @returns stub viewed as a `ChainNode`
 */
function asChainNode(stub: object,): ChainNode {
  return stub as unknown as ChainNode;
}

/**
 Builds a mock rule context whose token lookups satisfy the chain walk for
 synthetic nodes, so the flatten can run on chains far deeper than oxlint feeds
 a plugin in practice (where its own deep-AST handling fails first).
 
 `getTokenBefore` returns the dot punctuator for a property stub (no `type`)
 and a non-`(` neighbour for a node; `getTokenAfter` returns the `+` operator
 when a filter is supplied (the operator-token lookup) and a non-`)` neighbour
 otherwise (the grouping-paren probe). No node is ever treated as grouped.
 
 @returns context stub exposing the two token accessors the walk uses
 */
function mockChainContext(): Context {
  /**
   @param target - node or property whose preceding token is wanted
   
   @returns dot for a property stub, a non-`(` marker for a node
   */
  function getTokenBefore(target: {
    readonly type?: string;
    readonly start: number;
  },): TokenStub {
    return (target.type === undefined)
      ? {
        value: '.',
        start: target.start - 1,
      }
      : {
        value: 'x',
        start: target.start - 1,
      };
  }
  return asChainContext({
    sourceCode: {
      getTokenBefore,
      // `getTokenAfter` mirrors oxlint's positional `SourceCode.getTokenAfter(
      // nodeOrToken, skipOptions?)`, which the chain walk calls positionally.
      // A named function expression as a property value, not a `function`
      // declaration: `require-destructured-params` fires only on declarations
      // and would reject this external-API 2-parameter shape, which cannot
      // collapse to a single destructured object.
      // `target` is the left operand or node whose following token is wanted;
      // `options` is present for the operator-token lookup, absent for the
      // paren probe. Returns the `+` operator token when filtered, a non-`)`
      // marker otherwise.
      getTokenAfter: function getTokenAfter(
        target: { readonly end: number; },
        options?: { readonly filter: (token: TokenStub,) => boolean; },
      ): TokenStub {
        return (options === undefined)
          ? {
            value: 'x',
            start: target.end,
          }
          : {
            value: '+',
            start: target.end + 1,
          };
      },
    },
  },);
}

/**
 Casts a token-accessor stub to a `Context` for the synthetic chain walk.
 
 @param stub - object carrying the two `sourceCode` token accessors
 
 @returns stub viewed as a `Context`
 */
function asChainContext(stub: object,): Context {
  return stub as unknown as Context;
}

/**
 Builds a synthetic member chain `x.a.a...` of the given step count.
 
 @param steps - number of `.a` member steps past the leaf
 
 @returns outermost `MemberExpression` node of the chain
 */
function buildMemberChain(steps: number,): ChainNode {
  /** Leaf identifier `x` at the head of the chain. */
  const leaf = asChainNode({
    type: 'Identifier',
    start: 0,
    end: 1,
  },);
  return Array.from({ length: steps, },).reduce(
    function addStep(object: ChainNode, _step, index,): ChainNode {
      /** Byte offset of this step's dot punctuator. */
      const dotStart = 1 + (index * 2);
      return asChainNode({
        type: 'MemberExpression',
        object,
        computed: false,
        property: {
          start: dotStart + 1,
          end: dotStart + 2,
        },
        start: 0,
        end: dotStart + 2,
      },);
    },
    leaf,
  );
}

/**
 Builds a synthetic left-associative operator chain `a + a + ...`.
 
 @param operators - number of `+` operators in the chain
 
 @returns outermost `BinaryExpression` node of the chain
 */
function buildOperatorChain(operators: number,): ChainNode {
  /** Leftmost operand `a`. */
  const leaf = asChainNode({
    type: 'Identifier',
    start: 0,
    end: 1,
  },);
  return Array.from({ length: operators, },).reduce(
    function addOperator(left: ChainNode, _operator, index,): ChainNode {
      /** Byte offset of this operand's `a`, four columns per `+ a` step. */
      const operandStart = 4 * (index + 1);
      return asChainNode({
        type: 'BinaryExpression',
        operator: '+',
        left,
        right: asChainNode({
          type: 'Identifier',
          start: operandStart,
          end: operandStart + 1,
        },),
        start: 0,
        end: operandStart + 1,
      },);
    },
    leaf,
  );
}

/**
 Extracts unique rule codes from a set of diagnostics.
 
 @param diagnostics - array of oxlint diagnostics
 
 @returns sorted array of unique `stylistic(rule-name)` codes
 */
//endregion Helpers

await describe({
  name: '',
  children: [
    //region Valid fixtures: expect zero stylistic violations

    describe({
      name: 'valid fixtures',
      children: [
        it({
          name: 'star-less TSDoc and literal-leading asterisks pass never mode',
          fn: async () => {
            const diagnostics = await lint('valid/asterisk-prefix.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'starred TSDoc passes always mode',
          fn: async () => {
            const diagnostics = await lintWithConfig({
              fixturePath: 'invalid/asterisk-prefix.ts',
              fixtureConfig: ASTERISK_PREFIX_ALWAYS_FIXTURE_CONFIG,
            },);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'already-per-line constructs produce no violations',
          fn: async () => {
            const diagnostics = await lint('valid/already-per-line.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'single-item constructs produce no violations',
          fn: async () => {
            const diagnostics = await lint('valid/single-item.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'empty constructs produce no violations',
          fn: async () => {
            const diagnostics = await lint('valid/empty-constructs.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'no-mixed-operators valid cases produce no violations',
          fn: async () => {
            const diagnostics = await lint('valid/no-mixed-operators.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'chain-per-line valid cases produce no violations',
          fn: async () => {
            const diagnostics = await lint('valid/chain-per-line.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'semi valid cases produce no violations',
          fn: async () => {
            const diagnostics = await lint('valid/semi.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'comma-dangle valid cases produce no violations',
          fn: async () => {
            const diagnostics = await lint('valid/comma-dangle.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'block-body-newline valid cases produce no violations',
          fn: async () => {
            const diagnostics = await lint('valid/block-body-newline.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: 'indentation valid cases produce no violations',
          fn: async () => {
            const diagnostics = await lint('valid/indentation.ts',);
            expect(diagnostics,).toEqual([],);
          },
        },),
      ],
    },),

    //endregion Valid fixtures

    //region Invalid fixtures: expect specific violations

    describe({
      name: 'require-asterisk-prefix',
      children: [
        it({
          name: 'reports every canonical prefix in never mode',
          fn: async () => {
            const diagnostics = await lint('invalid/asterisk-prefix.ts',);
            const prefixDiagnostics = diagnostics.filter(
              function isPrefixDiagnostic(diagnostic,): boolean {
                return diagnostic.code === 'stylistic(require-asterisk-prefix)';
              },
            );
            expect(prefixDiagnostics,).toHaveLength(12,);
            expect(prefixDiagnostics[0]?.message,).toBe(
              'TSDoc body line cannot have an asterisk prefix in never mode.',
            );
          },
        },),
        it({
          name: 'reports every unprefixed body line in always mode',
          fn: async () => {
            const diagnostics = await lintWithConfig({
              fixturePath: 'valid/asterisk-prefix.ts',
              fixtureConfig: ASTERISK_PREFIX_ALWAYS_FIXTURE_CONFIG,
            },);
            const prefixDiagnostics = diagnostics.filter(
              function isPrefixDiagnostic(diagnostic,): boolean {
                return diagnostic.code === 'stylistic(require-asterisk-prefix)';
              },
            );
            expect(prefixDiagnostics,).toHaveLength(11,);
            expect(prefixDiagnostics[0]?.message,).toBe(
              'TSDoc body line requires an asterisk prefix in always mode.',
            );
          },
        },),
        it({
          name: 'rejects an omitted mode',
          fn: async () => {
            const caught = await (async function catchMissingModeError(): Promise<unknown> {
              try {
                await spawn(
                  'oxlint',
                  [
                    '--format',
                    'json',
                    '--config',
                    ASTERISK_PREFIX_MISSING_FIXTURE_CONFIG,
                    resolve(
                      FIXTURES,
                      'valid',
                      'asterisk-prefix.ts',
                    ),
                  ],
                  { cwd: ROOT, },
                );
                return undefined;
              }
              catch (error: unknown) {
                return error;
              }
            })();
            expect(caught,).toBeDefined();
            const { stdout, } = caught as { readonly stdout: string; };
            expect(stdout,).toContain('stylistic/require-asterisk-prefix',);
          },
        },),
        it({
          name: 'rejects an unsupported mode',
          fn: async () => {
            const caught = await (async function catchInvalidModeError(): Promise<unknown> {
              try {
                await spawn(
                  'oxlint',
                  [
                    '--format',
                    'json',
                    '--config',
                    ASTERISK_PREFIX_INVALID_FIXTURE_CONFIG,
                    resolve(
                      FIXTURES,
                      'valid',
                      'asterisk-prefix.ts',
                    ),
                  ],
                  { cwd: ROOT, },
                );
                return undefined;
              }
              catch (error: unknown) {
                return error;
              }
            })();
            expect(caught,).toBeDefined();
            const { stdout, } = caught as { readonly stdout: string; };
            expect(stdout,).toContain('stylistic/require-asterisk-prefix',);
          },
        },),
      ],
    },),

    describe({
      name: 'param-per-line',
      children: [
        it({
          name: 'reports params on the same line',
          fn: async () => {
            const diagnostics = await lint('invalid/param-per-line.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(param-per-line)',);
          },
        },),
      ],
    },),
    describe({
      name: 'argument-per-line',
      children: [
        it({
          name: 'reports arguments on the same line',
          fn: async () => {
            const diagnostics = await lint('invalid/argument-per-line.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(argument-per-line)',);
          },
        },),
      ],
    },),
    describe({
      name: 'array-element-per-line',
      children: [
        it({
          name: 'reports array elements on the same line',
          fn: async () => {
            const diagnostics = await lint('invalid/array-element-per-line.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(array-element-per-line)',);
          },
        },),
      ],
    },),
    describe({
      name: 'object-property-per-line',
      children: [
        it({
          name: 'reports object properties on the same line',
          fn: async () => {
            const diagnostics = await lint('invalid/object-property-per-line.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(object-property-per-line)',);
          },
        },),
      ],
    },),
    describe({
      name: 'import-per-line',
      children: [
        it({
          name: 'reports import specifiers on the same line',
          fn: async () => {
            const diagnostics = await lint('invalid/import-per-line.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(import-per-line)',);
          },
        },),
      ],
    },),
    describe({
      name: 'export-per-line',
      children: [
        it({
          name: 'reports export specifiers on the same line',
          fn: async () => {
            const diagnostics = await lint('invalid/export-per-line.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(export-per-line)',);
          },
        },),
      ],
    },),
    describe({
      name: 'type-property-per-line',
      children: [
        it({
          name: 'reports type members on the same line',
          fn: async () => {
            const diagnostics = await lint('invalid/type-property-per-line.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(type-property-per-line)',);
          },
        },),
      ],
    },),
    describe({
      name: 'tuple-per-line',
      children: [
        it({
          name: 'reports tuple elements on the same line',
          fn: async () => {
            const diagnostics = await lint('invalid/tuple-per-line.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(tuple-per-line)',);
          },
        },),
      ],
    },),
    describe({
      name: 'destructure-per-line',
      children: [
        it({
          name: 'reports destructured properties on the same line',
          fn: async () => {
            const diagnostics = await lint('invalid/destructure-per-line.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(destructure-per-line)',);
          },
        },),
      ],
    },),
    describe({
      name: 'no-mixed-operators',
      children: [
        it({
          name: 'reports mixed operators and chained exponentiation without parens',
          fn: async () => {
            const diagnostics = await lint('invalid/no-mixed-operators.ts',);
            const rules = uniqueRules(diagnostics,);
            /** Diagnostics carrying the dedicated chained-exponentiation message. */
            const exponentiationDiagnostics = diagnostics.filter(
              function isChainedExponentiation(diagnostic,): boolean {
                return diagnostic.message
                  === 'Chained exponentiation requires parentheses that expose its grouping.';
              },
            );
            expect(rules,).toContain('stylistic(no-mixed-operators)',);
            expect(exponentiationDiagnostics,).toHaveLength(3,);
          },
        },),
      ],
    },),
    describe({
      name: 'block-body-newline',
      children: [
        it({
          name: 'reports dense non-empty brace-delimited bodies',
          fn: async () => {
            const diagnostics = await lint('invalid/block-body-newline.ts',);
            /** block-body-newline diagnostics isolated from sibling fixture violations. */
            const blockDiagnostics = diagnostics.filter(function isBlockBodyNewline(
              diagnostic,
            ): boolean {
              return diagnostic.code === 'stylistic(block-body-newline)';
            },);
            // The fixture has eighteen dense non-empty bodies: function, arrow,
            // if/else, loop, do-while, try/catch/finally, switch, class body,
            // method, static block, module block, and comment boundary variants.
            // Fully dense nested method/static/module blocks and line comments
            // add more covered boundary shapes. Most report both boundaries;
            // a line comment directly before an existing newline reports only the
            // opening boundary because the closing brace is already after it.
            expect(blockDiagnostics.length,).toBe(51,);
            expect(
              blockDiagnostics.some(function hasOpeningMessage(diagnostic,): boolean {
                return diagnostic.message
                  === 'Put the first body token on the line after the opening brace.';
              },),
            ).toBe(true,);
            expect(
              blockDiagnostics.some(function hasClosingMessage(diagnostic,): boolean {
                return diagnostic.message
                  === 'Put the closing brace on the line after the final body token.';
              },),
            ).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: 'chain-per-line',
      children: [
        it({
          name: 'reports each non-canonical chain exactly once',
          fn: async () => {
            const diagnostics = await lint('invalid/chain-per-line.ts',);
            /** chain-per-line diagnostics isolated from any unrelated fixture violations. */
            const chainDiagnostics = diagnostics.filter(function isChain(diagnostic,): boolean {
              return diagnostic.code === 'stylistic(chain-per-line)';
            },);
            // The fixture has ten non-canonical chains (b1..b10); each root fires once.
            expect(chainDiagnostics.length,).toBe(10,);
            expect(chainDiagnostics[0]?.message,).toBe(
              'Put each operator, member, or method step in this chain on its own line.',
            );
          },
        },),
        it({
          name: 'reports a chain with an interior comment without dropping other rules',
          fn: async () => {
            const diagnostics = await lint('invalid/chain-comment.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(chain-per-line)',);
          },
        },),
        it({
          name:
            'reports both chain-per-line and no-mixed-operators on a combined fixture',
          fn: async () => {
            const diagnostics = await lint(
              'invalid/chain-and-mixed-operators.ts',
            );
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(chain-per-line)',);
            expect(rules,).toContain('stylistic(no-mixed-operators)',);
          },
        },),
        it({
          name: 'flattens a very long chain without stack overflow',
          fn: async () => {
            // doc/audit/chain-flatten-skewed-tree.md: the prior recursive flatteners
            // overflowed near member n=12000 and binary n=8000 because a member or
            // left-associative operator chain is a left-nested spine whose depth equals
            // its length. The iterative walk has O(1) extra stack depth, so flattening a
            // chain far past those thresholds returns instead of throwing RangeError.
            // Exercised directly on the flatten because oxlint's own deep-AST handling
            // fails first end-to-end (drops the rule near n=4000, then crashes), so a
            // reintroduced recursion would not be caught through a real lint run.
            /** Step count well past both audited overflow thresholds and the JS stack. */
            const steps = 50_000;
            /** Shared mock context; the synthetic nodes carry their own offsets. */
            const context = mockChainContext();
            /** Breaks of a `steps`-deep member spine. */
            const memberBreaks = chainBreaks({
              context,
              node: buildMemberChain(steps,),
            },);
            /** Breaks of a `steps`-operator left-associative spine. */
            const operatorBreaks = chainBreaks({
              context,
              node: buildOperatorChain(steps,),
            },);
            // A member chain breaks every step past the head's two segments; an operator
            // chain breaks every operator past the source-first one. Both reaching the
            // expected count proves the flatten ran to completion without overflowing.
            expect(memberBreaks.length,).toBe(steps
              - 1,);
            expect(operatorBreaks.length,).toBe(steps
              - 1,);
          },
        },),
      ],
    },),
    describe({
      name: 'invocation-depth-per-line',
      children: [
        it({
          name: 'reports every spine over depth two',
          fn: async () => {
            const diagnostics = await lint('invalid/invocation-depth-per-line.ts',);
            /** invocation-depth-per-line diagnostics isolated from the fixture. */
            const invocationDiagnostics = diagnostics.filter(
              function isInvocationDepth(diagnostic,): boolean {
                return diagnostic.code === 'stylistic(invocation-depth-per-line)';
              },
            );
            // The fixture has fifteen failing spines (f1..f14 plus the yield case);
            // each reports the outermost invocation on its violating line once.
            expect(invocationDiagnostics.length,).toBe(15,);
            expect(invocationDiagnostics[0]?.message,).toBe(
              'No more than two nested invocations may start on one line; split the operand onto its own line.',
            );
          },
        },),
        it({
          name: 'does not report compliant spines',
          fn: async () => {
            const diagnostics = await lint('valid/invocation-depth-per-line.ts',);
            expect(
              diagnostics.filter(function isInvocationDepth(diagnostic,): boolean {
                return diagnostic.code === 'stylistic(invocation-depth-per-line)';
              },),
            ).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: 'one-var-declaration-per-line',
      children: [
        it({
          name: 'reports multi-declarator declarations on the same line',
          fn: async () => {
            const diagnostics = await lint('invalid/one-var-declaration-per-line.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(one-var-declaration-per-line)',);
          },
        },),
      ],
    },),
    describe({
      name: 'max-statements-per-line',
      children: [
        it({
          name: 'reports multiple statements on the same line',
          fn: async () => {
            const diagnostics = await lint('invalid/max-statements-per-line.ts',);
            const rules = uniqueRules(diagnostics,);
            expect(rules,).toContain('stylistic(max-statements-per-line)',);
          },
        },),
      ],
    },),

    describe({
      name: 'semi',
      children: [
        it({
          name: 'reports missing semicolons',
          fn: async () => {
            const diagnostics = await lint('invalid/semi.ts',);
            const semiDiagnostics = diagnostics.filter(function isSemi(diagnostic,): boolean {
              return diagnostic.code === 'stylistic(semi)';
            },);
            expect(semiDiagnostics.length,).toBe(15,);
            expect(semiDiagnostics[0]?.message,).toBe('Missing semicolon.',);
          },
        },),
        it({
          name: 'rejects eslint-style options',
          fn: async () => {
            const caught = await (async function catchConfiguredSemiError(): Promise<unknown> {
              try {
                await spawn(
                  'oxlint',
                  [
                    '--format',
                    'json',
                    '--config',
                    SEMI_CONFIGURED_FIXTURE_CONFIG,
                    resolve(
                      FIXTURES,
                      'valid',
                      'semi.ts',
                    ),
                  ],
                  { cwd: ROOT, },
                );
                return undefined;
              }
              catch (error: unknown) {
                return error;
              }
            })();
            const { stdout, } = caught as { readonly stdout: string; };
            expect(stdout,).toContain(
              "Rule 'stylistic/semi' does not accept options",
            );
          },
        },),
      ],
    },),

    describe({
      name: 'comma-dangle',
      children: [
        it({
          name: 'reports missing trailing commas',
          fn: async () => {
            const diagnostics = await lint('invalid/comma-dangle.ts',);
            const commaDiagnostics = diagnostics.filter(function isCommaDangle(
              diagnostic,
            ): boolean {
              return diagnostic.code === 'stylistic(comma-dangle)';
            },);
            expect(commaDiagnostics.length,).toBeGreaterThan(0,);
            expect(commaDiagnostics[0]?.message,).toBe('Missing trailing comma.',);
          },
        },),
        it({
          name: 'rejects eslint-style options',
          fn: async () => {
            const caught = await (async function catchConfiguredCommaDangleError(): Promise<unknown> {
              try {
                await spawn(
                  'oxlint',
                  [
                    '--format',
                    'json',
                    '--config',
                    COMMA_DANGLE_CONFIGURED_FIXTURE_CONFIG,
                    resolve(
                      FIXTURES,
                      'valid',
                      'comma-dangle.ts',
                    ),
                  ],
                  { cwd: ROOT, },
                );
                return undefined;
              }
              catch (error: unknown) {
                return error;
              }
            })();
            const { stdout, } = caught as { readonly stdout: string; };
            expect(stdout,).toContain(
              "Rule 'stylistic/comma-dangle' does not accept options",
            );
          },
        },),
        it({
          name: 'reports missing trailing commas on type-annotated patterns',
          fn: async () => {
            const diagnostics = await lint(
              'invalid/comma-dangle-pattern-annotation.ts',
            );
            const commaDiagnostics = diagnostics.filter(
              function isCommaDangle(diagnostic,): boolean {
                return diagnostic.code === 'stylistic(comma-dangle)';
              },
            );
            expect(commaDiagnostics,).toHaveLength(3,);
            expect(commaDiagnostics[0]?.message,).toBe('Missing trailing comma.',);
          },
        },),
        it({
          name: 'valid type-annotated pattern cases produce no violations',
          fn: async () => {
            const diagnostics = await lint(
              'valid/comma-dangle-pattern-annotation.ts',
            );
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: '--fix adds pattern commas and leaves type annotations alone',
          fn: async () => {
            /** Source fixture copied so --fix never mutates original fixture. */
            const annotationSrc = resolve(
              FIXTURES,
              'invalid',
              'comma-dangle-pattern-annotation.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using annotationCopy = await createTempFixtureFile({
              fileName: 'comma-dangle-pattern-annotation.ts',
              sourcePath: annotationSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            await fixUntilStable(annotationCopy.filePath,);

            const fixedContent = readFileSync(annotationCopy.filePath, 'utf8',);
            expect(fixedContent,).toContain('function destructured({ salt, }: { readonly salt: number; },): void',);
            expect(fixedContent,).toContain('function destructuredPresent({ salt, }: { readonly salt: number; },): void',);
            expect(fixedContent,).toContain('function destructuredTuple([only,]: [number,],): void',);
            expect(fixedContent,).not.toContain(';,');

            const diagnostics = await lint(annotationCopy.filePath,);
            expect(diagnostics,).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: 'indentation',
      children: [
        it({
          name: 'reports one violation per deviating checked line',
          fn: async () => {
            const diagnostics = await lint('invalid/indentation.ts',);
            const indentationDiagnostics = diagnostics.filter(
              function isIndentationDiagnostic(diagnostic,): boolean {
                return diagnostic.code === 'stylistic(indentation)';
              },
            );
            expect(indentationDiagnostics,).toHaveLength(6,);
            expect(indentationDiagnostics[0]?.message,).toBe(
              'Expected indentation of 2 spaces but found 6.',
            );
          },
        },),
      ],
    },),

    //endregion Invalid fixtures

    //region Autofix tests

    describe({
      name: 'autofix',
      children: [
        it({
          name: '--fix normalizes indentation idempotently',
          fn: async () => {
            const sourcePath = resolve(
              FIXTURES,
              'invalid',
              'indentation.ts',
            );
            await using fixtureCopy = await createTempFixtureFile({
              fileName: 'indentation.ts',
              sourcePath,
              tempPrefix: 'oxlint-stylistic-indentation-',
            },);

            await fixUntilStable(fixtureCopy.filePath,);
            const fixedOnce = readFileSync(fixtureCopy.filePath, 'utf8',);
            expect(fixedOnce,).toContain(
              'function hang(\n  parameter: number,\n): number {\n  return parameter;',
            );
            expect(fixedOnce,).toContain('  value = 1;',);
            expect(fixedOnce,).toContain(
              '  method(): number {\n    return this.value;',
            );
            expect(fixedOnce,).toContain('  else {\n    return 2;',);

            await fixUntilStable(fixtureCopy.filePath,);
            expect(readFileSync(fixtureCopy.filePath, 'utf8',),).toBe(fixedOnce,);
            expect(await lint(fixtureCopy.filePath,),).toEqual([],);
          },
        },),
        it({
          name: '--fix converges indentation with argument, chain, and body fixers',
          fn: async () => {
            const sourcePath = resolve(
              FIXTURES,
              'invalid',
              'indentation-mixed.ts',
            );
            await using fixtureCopy = await createTempFixtureFile({
              fileName: 'indentation-mixed.ts',
              sourcePath,
              tempPrefix: 'oxlint-stylistic-indentation-mixed-',
            },);

            await fixUntilStable(fixtureCopy.filePath,);
            const fixedOnce = readFileSync(fixtureCopy.filePath, 'utf8',);
            expect(fixedOnce,).toContain('  calls(\n    1,\n    2,\n  );',);
            expect(fixedOnce,).toContain(
              '  const chained = value.then(1, )\n    .finally(2, );',
            );
            expect(fixedOnce,).toContain('  if (true) {\n    mixed();\n  }',);

            await fixUntilStable(fixtureCopy.filePath,);
            expect(readFileSync(fixtureCopy.filePath, 'utf8',),).toBe(fixedOnce,);
            expect(await lint(fixtureCopy.filePath,),).toEqual([],);
          },
        },),
        it({
          name: '--fix removes every canonical TSDoc prefix in never mode idempotently',
          fn: async () => {
            const sourcePath = resolve(
              FIXTURES,
              'invalid',
              'asterisk-prefix.ts',
            );
            await using fixtureCopy = await createTempFixtureFile({
              fileName: 'asterisk-prefix.ts',
              sourcePath,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            await fixUntilStable(fixtureCopy.filePath,);
            const fixedOnce = readFileSync(fixtureCopy.filePath, 'utf8',);
            expect(fixedOnce,).toContain(
              '/**\n Starred description.\n \n **Leading bold** remains literal content.\n \n *through* remains literal content.',
            );
            expect(fixedOnce,).toContain(
              '  /**\n   Nested property description.\n   */',
            );

            await fixUntilStable(fixtureCopy.filePath,);
            expect(readFileSync(fixtureCopy.filePath, 'utf8',),).toBe(fixedOnce,);
            expect(await lint(fixtureCopy.filePath,),).toEqual([],);
          },
        },),
        it({
          name: '--fix adds every canonical TSDoc prefix in always mode idempotently',
          fn: async () => {
            const sourcePath = resolve(
              FIXTURES,
              'valid',
              'asterisk-prefix.ts',
            );
            await using fixtureCopy = await createTempFixtureFile({
              fileName: 'asterisk-prefix.ts',
              sourcePath,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            await fixOnce({
              filePath: fixtureCopy.filePath,
              fixtureConfig: ASTERISK_PREFIX_ALWAYS_FIXTURE_CONFIG,
            },);
            const fixedOnce = readFileSync(fixtureCopy.filePath, 'utf8',);
            expect(fixedOnce,).toContain(
              '/**\n * Star-less description.\n *\n * **Leading bold** remains literal content.\n *\n * *through* remains literal content.',
            );
            expect(fixedOnce,).toContain(
              '/**\n * Description whose closing delimiter shares its final line.\n */',
            );

            await fixOnce({
              filePath: fixtureCopy.filePath,
              fixtureConfig: ASTERISK_PREFIX_ALWAYS_FIXTURE_CONFIG,
            },);
            expect(readFileSync(fixtureCopy.filePath, 'utf8',),).toBe(fixedOnce,);
            expect(await lintWithConfig({
              fixturePath: fixtureCopy.filePath,
              fixtureConfig: ASTERISK_PREFIX_ALWAYS_FIXTURE_CONFIG,
            },),).toEqual([],);
          },
        },),
        it({
          name: '--fix inserts missing semicolons',
          fn: async () => {
            /** Source fixture copied so --fix never mutates original fixture. */
            const semiSrc = resolve(
              FIXTURES,
              'invalid',
              'semi.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using semiCopy = await createTempFixtureFile({
              fileName: 'semi.ts',
              sourcePath: semiSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            await fixOnce({
              filePath: semiCopy.filePath,
              fixtureConfig: FIXTURE_CONFIG,
            },);

            const fixedContent = readFileSync(semiCopy.filePath, 'utf8',);
            expect(fixedContent,).toContain('const value = 1;',);
            expect(fixedContent,).toContain('return value;',);
            expect(fixedContent,).toContain('field = value;',);
            expect(fixedContent,).toContain('type Alias = number;',);
            expect(fixedContent,).toContain('declare function ambient(): void;',);
            expect(fixedContent,).toContain('export default value;',);

            const diagnostics = await lint(semiCopy.filePath,);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: '--fix produces zero violations',
          fn: async () => {
            /** Source fixture copied so --fix never mutates original fixture. */
            const fixableSrc = resolve(
              FIXTURES,
              'invalid',
              'fixable.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using fixableCopy = await createTempFixtureFile({
              fileName: 'fixable.ts',
              sourcePath: fixableSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            await fixUntilStable(fixableCopy.filePath,);

            // Re-lint the fixed copy
            const diagnostics = await lint(fixableCopy.filePath,);
            const stylisticDiags = diagnostics.filter(
              function isStylistic(d,): boolean {
                return ((typeof d.code) === 'string')
                  && d.code.startsWith('stylistic(',);
              },
            );
            expect(stylisticDiags,).toEqual([],);
          },
        },),
        it({
          name: '--fix inserts missing trailing commas',
          fn: async () => {
            /** Source fixture copied so --fix never mutates original fixture. */
            const commaSrc = resolve(
              FIXTURES,
              'invalid',
              'comma-dangle.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using commaCopy = await createTempFixtureFile({
              fileName: 'comma-dangle.ts',
              sourcePath: commaSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            await fixUntilStable(commaCopy.filePath,);

            const fixedContent = readFileSync(commaCopy.filePath, 'utf8',);
            expect(fixedContent,).toContain('import { one as importedOne, }',);
            expect(fixedContent,).toContain("with { type: 'json', }",);
            expect(fixedContent,).toContain('const values = [one,];',);
            expect(fixedContent,).toContain('  one,\n  two,',);
            expect(fixedContent,).toContain('const value = { one: 1, };',);
            expect(fixedContent,).toContain('const [first,] = sourceValues;',);
            expect(fixedContent,).toContain('const { one: picked, } = sourceObject;',);
            expect(fixedContent,).toContain('function identity(value: string,): string',);
            expect(fixedContent,).toContain('function generic<T,>(value: T,): T',);
            expect(fixedContent,).toContain('function named(value: string,): string',);
            expect(fixedContent,).toContain('const arrow = (value: string,): string => value;',);
            expect(fixedContent,).toContain('identity(one,);',);
            expect(fixedContent,).toContain('new Thing(one,);',);
            expect(fixedContent,).toContain('import(one,);',);
            expect(fixedContent,).toContain("with: { type: 'json', },",);
            expect(fixedContent,).toContain('enum Value {\n  One,\n}',);
            expect(fixedContent,).toContain('type StringPair = [string,];',);
            expect(fixedContent,).toContain('type Generic<T,> = T;',);
            expect(fixedContent,).toContain('type Fn = (value: string,) => void;',);
            expect(fixedContent,).toContain('method(value: string,): void;',);
            expect(fixedContent,).toContain('(value: string,): void;',);
            expect(fixedContent,).toContain('new(value: string,): Thing;',);
            expect(fixedContent,).toContain('type Ctor = new(value: string,) => Thing;',);
            expect(fixedContent,).toContain('declare function declared(value: string,): void;',);
            expect(fixedContent,).toContain('one: 1, // keep comment',);
            expect(fixedContent,).toContain('export { one, };',);
            expect(fixedContent,).toContain('export * from',);

            const diagnostics = await lint(commaCopy.filePath,);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: '--fix preserves trailing commas',
          fn: async () => {
            /** Source fixture copied so --fix never mutates original fixture. */
            const trailingSrc = resolve(
              FIXTURES,
              'invalid',
              'fixable-trailing-comma.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using trailingCopy = await createTempFixtureFile({
              fileName: 'fixable-trailing-comma.ts',
              sourcePath: trailingSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            await fixOnce({
              filePath: trailingCopy.filePath,
              fixtureConfig: FIXTURE_CONFIG,
            },);

            const fixedContent = readFileSync(trailingCopy.filePath, 'utf8',);

            // Trailing commas should be preserved on all items including the last
            expect(fixedContent,).toContain('  name: string,',);
            expect(fixedContent,).toContain('  age: number,',);
            expect(fixedContent,).toContain('  3,\n',);
            expect(fixedContent,).toContain('  port: 3000,',);
            expect(fixedContent,).toContain('  port,',);
          },
        },),
        it({
          name: '--fix splits dense block body boundaries and preserves comments',
          fn: async () => {
            /** Source fixture copied so --fix never mutates original fixture. */
            const blockSrc = resolve(
              FIXTURES,
              'invalid',
              'block-body-newline.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using blockCopy = await createTempFixtureFile({
              fileName: 'block-body-newline.ts',
              sourcePath: blockSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            await fixUntilStable(blockCopy.filePath,);

            /** File content after all block-boundary fixes converge. */
            const fixedContent = readFileSync(blockCopy.filePath, 'utf8',);
            expect(fixedContent,).toContain('function denseFunction(): number {\n  return 1;\n}',);
            expect(fixedContent,).toContain('const denseArrow = (): number => {\n  return denseFunction();\n};',);
            expect(fixedContent,).toContain('if (condition) {\n  doThing();\n}',);
            expect(fixedContent,).toContain('catch (error) {\n  throw error;\n}',);
            expect(fixedContent,).toContain('switch (denseFunction()) {\n  case 1:',);
            expect(fixedContent,).toContain('class DenseClassBody {\n  static value = 0;\n}',);
            expect(fixedContent,).toContain('  static {\n    DenseClassBody.value = 1;\n  }',);
            expect(fixedContent,).toContain('class FullyDenseMethod {\n  method(): number {\n    return 1;\n  }\n}',);
            expect(fixedContent,).toContain('class FullyDenseStatic {\n  static {\n    DenseClassBody.value = 2;\n  }\n}',);
            expect(fixedContent,).toContain('namespace DenseNamespace {\n  export const namespaceValue = 1;\n}',);
            expect(fixedContent,).toContain('namespace OuterNamespace {\n  export namespace InnerNamespace {\n    export const inner = 1;\n  }\n}',);
            expect(fixedContent,).toContain('function commentAtStart(): void {\n  /* keep start */doThing();\n}',);
            expect(fixedContent,).toContain('function commentAtEnd(): void {\n  doThing();/* keep end */\n}',);
            expect(fixedContent,).toContain('function commentOnly(): void {\n  /* keep only */\n}',);
            expect(fixedContent,).toContain('function lineCommentAtStart(): void {\n  // keep line start\n  doThing();\n}',);
            expect(fixedContent,).toContain('function lineCommentAtEnd(): void {\n  doThing(); // keep line end\n}',);

            const diagnostics = await lint(blockCopy.filePath,);
            expect(
              diagnostics.filter(function isBlockBodyNewline(diagnostic,): boolean {
                return diagnostic.code === 'stylistic(block-body-newline)';
              },),
            ).toEqual([],);
          },
        },),
        it({
          name: '--fix renders each chain in canonical layout, idempotently and without trailing whitespace',
          fn: async () => {
            /** Source fixture copied so --fix never mutates original fixture. */
            const chainSrc = resolve(
              FIXTURES,
              'invalid',
              'chain-per-line.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using chainCopy = await createTempFixtureFile({
              fileName: 'chain-per-line.ts',
              sourcePath: chainSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            await fixUntilStable(chainCopy.filePath,);
            /** File content after all overlapping fixes converge. */
            const fixedOnce = readFileSync(chainCopy.filePath, 'utf8',);

            /** Exact canonical layout expected for each chain in the fixture. */
            const expectedLayouts = [
              'const b1 = obj.foo\n  .bar;',
              'const b2 = ctx.sc\n  .getText();',
              'const b3 = obj.b\n  .c\n  .d;',
              'const b4 = foo()\n  .bar()[0];',
              'const b5 = items.map(a,)\n  .filter(b,)\n  .filter(c,);',
              'const b6 = a + b\n  + c\n  + d;',
              'const b7 = x && y\n  && z;',
              'const b8 = aa.b()\n  .c()\n  + dd.e()\n    .f();',
              'const b9 = obj.b\n  .c\n  .d\n  .toString()\n  .trim();',
              'const b10 = obj.a\n  .b\n  > c;',
            ];
            expectedLayouts.forEach(function assertLayout(layout,): void {
              expect(fixedOnce,).toContain(layout,);
            },);

            /** Lines of the fixed output, for whitespace-shape regression checks. */
            const lines = fixedOnce.split('\n',);
            expect(
              lines.some(function endsInWhitespace(line,): boolean {
                /** Last character of the line, or `undefined` when the line is empty. */
                const last = line.at(-1,);
                return (last === ' ') || (last === '\t');
              },),
            ).toBe(false,);
            expect(
              lines.some(function isWhitespaceOnly(line,): boolean {
                return (line.length > 0) && (line.trim() === '');
              },),
            ).toBe(false,);

            await fixUntilStable(chainCopy.filePath,);
            // Another convergence run changes nothing: the fix is idempotent.
            expect(readFileSync(chainCopy.filePath, 'utf8',),).toBe(fixedOnce,);

            const diagnostics = await lint(chainCopy.filePath,);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name: '--fix preserves a chain whose interior comment suppresses the fix',
          fn: async () => {
            /** Source fixture copied so --fix never mutates original fixture. */
            const commentSrc = resolve(
              FIXTURES,
              'invalid',
              'chain-comment.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using commentCopy = await createTempFixtureFile({
              fileName: 'chain-comment.ts',
              sourcePath: commentSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);
            /** Original content; the suppressed fix must leave it byte-for-byte. */
            const before = readFileSync(commentCopy.filePath, 'utf8',);

            await fixOnce({
              filePath: commentCopy.filePath,
              fixtureConfig: FIXTURE_CONFIG,
            },);
            expect(readFileSync(commentCopy.filePath, 'utf8',),).toBe(before,);
          },
        },),
        it({
          name: '--fix applies to a chain whose comment sits in trailing call args',
          fn: async () => {
            /** Source fixture copied so --fix never mutates the original fixture. */
            const argsSrc = resolve(
              FIXTURES,
              'invalid',
              'chain-comment-in-args.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using argsCopy = await createTempFixtureFile({
              fileName: 'chain-comment-in-args.ts',
              sourcePath: argsSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);
            /** Original content; unlike the head-comment case, the fix must rewrite it. */
            const before = readFileSync(argsCopy.filePath, 'utf8',);

            await fixOnce({
              filePath: argsCopy.filePath,
              fixtureConfig: FIXTURE_CONFIG,
            },);
            /** Content after the fix: the chain broke at the member step. */
            const after = readFileSync(argsCopy.filePath, 'utf8',);
            // The fix applied rather than being suppressed by the interior comment.
            expect(after,).not.toBe(before,);
            // The continuation-slice comment survived verbatim, never relocated or dropped.
            expect(after.includes('// keep this note inside the call',),).toBe(true,);
            // The member axis broke `obj.a` onto the head line.
            expect(after.includes('obj.a\n',),).toBe(true,);
          },
        },),
        it({
          name: '--fix exposes every chained-exponentiation grouping',
          fn: async () => {
            /** Source fixture copied so --fix never mutates original fixture. */
            const exponentiationSrc = resolve(
              FIXTURES,
              'invalid',
              'no-mixed-operators.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using exponentiationCopy = await createTempFixtureFile({
              fileName: 'no-mixed-operators.ts',
              sourcePath: exponentiationSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);
            /** Diagnostics before fixing, proving the fixture exercises the new branch. */
            const beforeDiagnostics = await lint(exponentiationCopy.filePath,);
            /** Chained-exponentiation findings before explicit grouping is inserted. */
            const beforeExponentiationDiagnostics = beforeDiagnostics.filter(
              function isChainedExponentiation(diagnostic,): boolean {
                return diagnostic.message
                  === 'Chained exponentiation requires parentheses that expose its grouping.';
              },
            );
            expect(beforeExponentiationDiagnostics,).toHaveLength(3,);

            await fixUntilStable(exponentiationCopy.filePath,);

            /** Fixed source, inspected to prove grouping became explicit. */
            const after = readFileSync(exponentiationCopy.filePath, 'utf8',);
            /** Diagnostics after all overlapping fixes converge. */
            const afterDiagnostics = await lint(exponentiationCopy.filePath,);
            expect(after,).toContain('const r6 = 2 ** (3',);
            expect(afterDiagnostics,).toEqual([],);
          },
        },),
        it({
          name:
            '--fix converges when chain-per-line and no-mixed-operators apply together',
          fn: async () => {
            /** Source fixture copied so --fix never mutates original fixture. */
            const combinedSrc = resolve(
              FIXTURES,
              'invalid',
              'chain-and-mixed-operators.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using combinedCopy = await createTempFixtureFile({
              fileName: 'chain-and-mixed-operators.ts',
              sourcePath: combinedSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            // oxlint applies at most one fix per overlapping byte region per pass, so
            // chain-per-line and the no-mixed-operators paren wrap settle over several
            // passes: pass 1 breaks the chain, pass 2 wraps the mixed operands, pass 3
            // re-indents the now-nested inner chain. Run --fix until the file stops
            // changing (capped well above the need) rather than hard-coding a count.
            await fixUntilStable(combinedCopy.filePath,);

            const diagnostics = await lint(combinedCopy.filePath,);
            const stylisticDiags = diagnostics.filter(
              function isStylistic(d,): boolean {
                return ((typeof d.code) === 'string')
                  && d.code.startsWith('stylistic(',);
              },
            );
            expect(stylisticDiags,).toEqual([],);
          },
        },),
        it({
          name: '--fix places each item on its own line',
          fn: async () => {
            /** Source fixture copied so --fix never mutates original fixture. */
            const fixableSrc = resolve(
              FIXTURES,
              'invalid',
              'fixable.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using fixableCopy = await createTempFixtureFile({
              fileName: 'fixable.ts',
              sourcePath: fixableSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            await fixUntilStable(fixableCopy.filePath,);

            const fixedContent = readFileSync(fixableCopy.filePath, 'utf8',);

            // After fix, multi-param function should have params on separate lines.
            expect(fixedContent,).toContain('  name: string,',);
            expect(fixedContent,).toContain('  age: number,',);

            // Array elements should be on separate lines with trailing comma.
            expect(fixedContent,).toContain('[\n  1,\n  2,\n  3,',);

            // Object properties should be on separate lines with trailing comma.
            expect(fixedContent,).toContain("  host: 'localhost',",);
            expect(fixedContent,).toContain('  port: 3000,',);

            // Multi-declarator declaration should be split across lines.
            expect(fixedContent,).toContain('const m = 1,\n  n = 2;',);

            // Two statements on a line should be split across lines.
            expect(fixedContent,).toContain('const p = 10;\nconst q = 20;',);
          },
        },),
        it({
          name:
            '--fix splits invocation spines, keeping trailing comments and grouping parens, idempotently',
          fn: async () => {
            /** Source fixture copied so --fix never mutates the original fixture. */
            const commentSrc = resolve(
              FIXTURES,
              'invalid',
              'invocation-depth-comment.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using commentCopy = await createTempFixtureFile({
              fileName: 'invocation-depth-comment.ts',
              sourcePath: commentSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            await fixUntilStable(commentCopy.filePath,);
            /** File content after all overlapping fixes converge. */
            const fixedOnce = readFileSync(commentCopy.filePath, 'utf8',);

            // The comma is placed before each trailing comment, the grouping
            // parentheses survive, and an existing comma is not doubled.
            expect(fixedOnce,).toContain('b(c(),), // keep line',);
            expect(fixedOnce,).toContain('b(c(),), /* keep block */',);
            expect(fixedOnce,).toContain('(b(c(),) /* keep grouped */),',);
            expect(fixedOnce,).not.toContain('b(c(),),,',);

            // Another convergence run changes nothing: the fix is idempotent.
            await fixUntilStable(commentCopy.filePath,);
            expect(readFileSync(commentCopy.filePath, 'utf8',),).toBe(fixedOnce,);

            const diagnostics = await lint(commentCopy.filePath,);
            expect(diagnostics,).toEqual([],);
          },
        },),
        it({
          name:
            '--fix converges invocation-depth with argument, object, and array per-line rules',
          fn: async () => {
            /** Source fixture copied so --fix never mutates the original fixture. */
            const convergeSrc = resolve(
              FIXTURES,
              'invalid',
              'invocation-depth-convergence.ts',
            );
            /** Temp fixture copy isolated from parallel autofix tests. */
            await using convergeCopy = await createTempFixtureFile({
              fileName: 'invocation-depth-convergence.ts',
              sourcePath: convergeSrc,
              tempPrefix: 'oxlint-stylistic-autofix-',
            },);

            // oxlint applies at most one fix per overlapping byte region per pass,
            // so a deep spine and the sibling per-line rules settle over several
            // passes. Run --fix until the file stops changing (capped well above
            // the need) rather than hard-coding a count.
            await fixUntilStable(convergeCopy.filePath,);

            const diagnostics = await lint(convergeCopy.filePath,);
            const stylisticDiags = diagnostics.filter(
              function isStylistic(d,): boolean {
                return ((typeof d.code) === 'string')
                  && d.code.startsWith('stylistic(',);
              },
            );
            expect(stylisticDiags,).toEqual([],);
          },
        },),
      ],
    },),
    //endregion Autofix tests
  ],
},);
