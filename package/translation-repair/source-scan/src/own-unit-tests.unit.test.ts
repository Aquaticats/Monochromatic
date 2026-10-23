/**
 Holds the package to the production modules found with no unit test of
 their own (ledger B102). A module added without `<stem>.unit.test.ts`
 beside it fails this scan until it gets one or a reviewed `ALLOWLIST` line,
 and a listed module that gains one, turns out exempt, or is removed fails
 it until its line goes, so the list only shrinks.

 WHAT THE SCAN READS: every `.ts` file `readPackageSource` returns under
 the `src` of every family package, each named by its package directory and
 its path under `src`. A production module is any that is not `*.unit.test.ts`,
 `*.test-fixture.ts` or `*.scratch.ts`, and it has an own test when
 `<dir>/<stem>.unit.test.ts` is among the files of its package, or, for a module whose path under
 `src` no other package repeats, among the files of any package (the split put a test in the lowest
 package that reaches every name it needs, so a test may sit beside neither its module nor its package).
 A path every package repeats (`index.ts`, `build-entries.ts`) needs the test in its own package.

 WHAT IT EXEMPTS, read off each module's own syntax tree, since none of
 these carries logic a test could lose:

 - a runner entry, one of the `'./src/...'` paths a family package's
   `src/build-entries.ts` lists, read from the package that lists it, the library index left out: a program
   the corpus harness runs, not a library export a unit test calls;
 - a barrel, every top-level statement an import, an `export * from` or an
   export list with no inline declaration, such as the generated `src/index.ts` of each package;
 - a types-only module, every top-level statement a type alias, an
   interface, an import, or a variable declaration whose every initialiser
   is absent or literal-like (a literal, a template with no expressions, a
   signed literal, an array or object of literal-like values, any of those
   under `as`, `satisfies`, `!` or parentheses).

 A bare export list is never types-only on its own, so a file mixing one
 with a declaration needs an `ALLOWLIST` line.

 OUT OF ITS REACH: whether a module's tests exercise every branch, which
 the coverage census measures; a module outside `src`; and a runner known
 only by a top-level `await` or a `process.argv` read, which shows here as
 a module to list rather than as a fourth exemption.

 THE FIXTURE CASE COMES FIRST, so the package case is read against a scan
 shown able to tell each shape apart and to find both failure kinds
 (ledger M21). Fixtures are cat-themed; the package case reads this
 package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  pathUnderSrc,
  readPackageSource,
} from './family-source.test-fixture.ts';
import {
  isTreeNode,
  literalText,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectRecordAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Own-test ratchet scan
// What a production module classifies as: own-tested, exempt by shape
// (runner, barrel, types-only), or needing a line on ALLOWLIST. The scan never decides WHICH test reaches a module,
// only whether `dir/stem.unit.test.ts` exists beside it and, when it does
// not, whether the module's own syntax rules it out of needing one.

/**
 Whether a file's path names a production module: not a unit test, not a
 shared test fixture, not a scratch file.

 @param path - file location, a package directory and a path under its `src`

 @returns Whether the path is production rather than test support

 @example
 ```ts
 const production = isProductionModulePath({ path: 'cited-reference.ts', },); // true
 ```
 */
function isProductionModulePath({ path, }: { readonly path: string; },): boolean {
  return !(path.endsWith('.unit.test.ts',)
    || path.endsWith('.test-fixture.ts',)
    || path.endsWith('.scratch.ts',));
}

/**
 Directory and stem (file name without its trailing `.ts`) a path splits
 into.

 @param path - file location, a package directory and a path under its `src`

 @returns Its directory, empty at the top of `src`, and its stem

 @example
 ```ts
 const split = dirAndStem({ path: 'corpus-run/bench-sample.ts', },); // { dir: 'corpus-run', stem: 'bench-sample', }
 ```
 */
function dirAndStem({ path, }: { readonly path: string; },): {
  readonly dir: string;
  readonly stem: string;
} {
  /**
   Index of the last path separator, absent at the top of `src`.
   */
  const slash = path.lastIndexOf('/',);
  /**
   File name alone, the directory cut away.
   */
  const fileName = (slash === (-1)) ? path : path.slice(slash + 1,);
  return {
    dir: (slash === (-1)) ? '' : path.slice(0, slash,),
    stem: fileName.endsWith('.ts',) ? fileName.slice(0, -'.ts'.length,) : fileName,
  };
}

/**
 Path a module's own unit test would sit at, beside it.

 @param path - module location, a package directory and a path under its `src`

 @returns The exact own-test path the scan looks for

 @example
 ```ts
 const own = ownTestPathFor({ path: 'corpus-run/bench-sample.ts', },); // 'corpus-run/bench-sample.unit.test.ts'
 ```
 */
function ownTestPathFor({ path, }: { readonly path: string; },): string {
  /**
   The module's directory and stem.
   */
  const { dir, stem, } = dirAndStem({ path, },);
  return (dir === '') ? `${stem}.unit.test.ts` : `${dir}/${stem}.unit.test.ts`;
}

/**
 Opening every runner entry's source path carries in
 `src/build-entries.ts`.
 */
const ENTRY_SOURCE_OPENING = './src/';

/**
 Name of the file in each package whose string literals name that package's runner entries, read off its
 parse: tests reach a package through its built bundle, never a relative import into `src`.
 */
const BUILD_ENTRIES_FILE_NAME = 'build-entries.ts';

/**
 The library index, which `nodeEntries` also holds but which names library
 source, not a runner the scan should exempt.
 */
const LIBRARY_INDEX_SOURCE = 'index.ts';

/**
 Runner entry sources every `src/build-entries.ts` lists, read as text, each located by the package that
 lists it (`run/corpus-run/...`).

 @param files - files read; only those named `build-entries.ts` (in a package or, in a fixture, with
 no package) matter, and their absence (the fixture case) yields no runner entries

 @returns Runner entry sources, the library indexes left out

 @example
 ```ts
 const runners = runnerEntrySourcePaths({ files, },);
 runners.has('run/corpus-run/corpus-pass.ts',); // true
 ```
 */
function runnerEntrySourcePaths({ files, }: { readonly files: readonly SourceText[]; },): ReadonlySet<string> {
  return new Set(files
    .filter(function isBuildEntriesFile({ path, },): boolean {
      return (path === BUILD_ENTRIES_FILE_NAME) || path.endsWith(`/${BUILD_ENTRIES_FILE_NAME}`,);
    },)
    .flatMap(function entriesOf(file,): readonly string[] {
      /**
       Location prefix of the package that lists them, empty for a file with no package.
       */
      const prefix = file.path.slice(
        0,
        -BUILD_ENTRIES_FILE_NAME.length,
      );
      return nodesUnder({ root: parseSource({ file, },).program, },)
        .map(function textOf(node,): string {
          return literalText({ node, },);
        },)
        .filter(function isEntrySource(text,): boolean {
          return text.startsWith(ENTRY_SOURCE_OPENING,) && text.endsWith('.ts',);
        },)
        .map(function underSrc(text,): string {
          return text.slice(ENTRY_SOURCE_OPENING.length,);
        },)
        .filter(function isRunnerSource(source,): boolean {
          return source !== LIBRARY_INDEX_SOURCE;
        },)
        .map(function located(source,): string {
          return `${prefix}${source}`;
        },);
    },),);
}

/**
 Whether a top-level statement is barrel-shaped: an import, an
 `export * from`, or an export list with no inline declaration, type-only
 or not.

 @param statement - top-level statement read

 @returns Whether the statement alone would let its file be a barrel

 @example
 ```ts
 const barrelShaped = isBarrelStatement({ statement, },); // true for `export { x } from './y.ts';`
 ```
 */
function isBarrelStatement({ statement, }: { readonly statement: TreeNode; },): boolean {
  if (statement.type === 'ImportDeclaration')
    return true;
  if (statement.type === 'ExportAllDeclaration')
    return true;
  return (statement.type === 'ExportNamedDeclaration') && (!isTreeNode(statement.declaration,));
}

/**
 Whether every top-level statement of a module is barrel-shaped.

 @param module - module read

 @returns Whether the module is a barrel, vacuously true for an empty file

 @example
 ```ts
 const barrel = isBarrelModule({ module, },);
 ```
 */
function isBarrelModule({ module, }: { readonly module: SourceText; },): boolean {
  return topLevelStatementsOf({ module, },)
    .every(function each(statement,): boolean { return isBarrelStatement({ statement, },); },);
}

/**
 Top-level statements of a parsed module, non-node entries filtered out.

 @param module - module read

 @returns Its program's top-level statements

 @example
 ```ts
 const statements = topLevelStatementsOf({ module, },);
 ```
 */
function topLevelStatementsOf({ module, }: { readonly module: SourceText; },): readonly TreeNode[] {
  /**
   The module's parsed program.
   */
  const { program, } = parseSource({ file: module, },);
  /**
   Its top-level statement list, possibly absent on an unexpected program
   shape.
   */
  const body: unknown = program.body;
  return (Array.isArray(body,) ? body : []).filter(function isNode(node,): node is TreeNode {
    return isTreeNode(node,);
  },);
}

/**
 The declaration an `ExportNamedDeclaration` carries inline, or the
 statement itself for any other kind.

 @param statement - top-level statement read

 @returns The node whose kind the types-only check reads

 @example
 ```ts
 const declaration = declarationOf({ statement, },); // the `type Cat = ...` node under `export type Cat = ...;`
 ```
 */
function declarationOf({ statement, }: { readonly statement: TreeNode; },): TreeNode {
  /**
   The statement's inline declaration, absent on a bare export list or any
   other statement kind.
   */
  const { declaration, } = statement;
  return ((statement.type === 'ExportNamedDeclaration') && isTreeNode(declaration,)) ? declaration : statement;
}

/**
 Node kinds `isLiteralLikeExpression` unwraps without changing whether the
 value underneath is literal-like, `ChainExpression` left out since a
 literal is never optionally chained.
 */
const LITERAL_WRAPPER_KINDS: ReadonlySet<string> = new Set([
  'TSAsExpression',
  'TSSatisfiesExpression',
  'TSNonNullExpression',
  'ParenthesizedExpression',
  'TSTypeAssertion',
],);

/**
 Whether an expression a `const` initialises with is literal-like, so the
 declaration carries no logic.

 @param node - initialiser expression read, absent for a bare `let x;`

 @returns Whether it is literal-like, true where there is no initialiser

 @example
 ```ts
 const literalLike = isLiteralLikeExpression({ node: declarator.init, },); // true for `{ tabby: 1, } as const`
 ```
 */
function isLiteralLikeExpression({ node, }: { readonly node: unknown; },): boolean {
  // No initialiser at all carries no logic.
  if (!isTreeNode(node,))
    return true;
  if (node.type === 'Literal')
    return true;
  if (node.type === 'TemplateLiteral') {
    /**
     Interpolated expressions the template holds, none for a plain string.
     */
    const expressions: unknown = node.expressions;
    return (Array.isArray(expressions,) ? expressions : []).length === 0;
  }
  if ((node.type === 'UnaryExpression') && ['-', '+',].includes(node.operator as string,))
    return isLiteralLikeExpression({ node: node.argument, },);
  if (node.type === 'ArrayExpression') {
    /**
     The array's elements, a hole read as `null`.
     */
    const elements: unknown = node.elements;
    return (Array.isArray(elements,) ? elements : [])
      .every(function each(element,): boolean {
        return (element === null) || isLiteralLikeExpression({ node: element, },);
      },);
  }
  if (node.type === 'ObjectExpression') {
    /**
     The object's properties, each a plain property or a spread.
     */
    const properties: unknown = node.properties;
    return (Array.isArray(properties,) ? properties : [])
      .every(function each(property,): boolean {
        return isTreeNode(property,)
          && ((property.type === 'SpreadElement')
            ? isLiteralLikeExpression({ node: property.argument, },)
            : isLiteralLikeExpression({ node: property.value, },));
      },);
  }
  return LITERAL_WRAPPER_KINDS.has(node.type,) && isLiteralLikeExpression({ node: node.expression, },);
}

/**
 Whether a top-level statement is types-only-shaped: a bare export list
 never qualifies on its own, so a file mixing one with a type declaration
 needs an `ALLOWLIST` line.

 @param statement - top-level statement read

 @returns Whether the statement alone would let its file be types-only

 @example
 ```ts
 const typesOnlyShaped = isTypesOnlyStatement({ statement, },); // true for `export const TABBY = 'tabby' as const;`
 ```
 */
function isTypesOnlyStatement({ statement, }: { readonly statement: TreeNode; },): boolean {
  /**
   The statement's inline declaration, or the statement itself.
   */
  const declaration = declarationOf({ statement, },);
  if (['TSTypeAliasDeclaration', 'TSInterfaceDeclaration',].includes(declaration.type,))
    return true;
  if (statement.type === 'ImportDeclaration')
    return true;
  if (declaration.type !== 'VariableDeclaration')
    return false;
  /**
   The declaration's individual declarators, each `const x = ...`.
   */
  const declarators: unknown = declaration.declarations;
  return (Array.isArray(declarators,) ? declarators : [])
    .every(function each(declarator,): boolean {
      return isTreeNode(declarator,) && isLiteralLikeExpression({ node: declarator.init, },);
    },);
}

/**
 Whether every top-level statement of a module is types-only-shaped.

 @param module - module read

 @returns Whether the module is types-only, vacuously true for an empty file

 @example
 ```ts
 const typesOnly = isTypesOnlyModule({ module, },);
 ```
 */
function isTypesOnlyModule({ module, }: { readonly module: SourceText; },): boolean {
  return topLevelStatementsOf({ module, },)
    .every(function each(statement,): boolean { return isTypesOnlyStatement({ statement, },); },);
}

/**
 Whether a production module without an own test is exempt by shape: a
 runner entry, a barrel, or types-only.

 @param module - module read, already known to lack an own test

 @param runnerEntrySources - runner entry sources every `src/build-entries.ts` lists

 @returns Whether the module needs no `ALLOWLIST` line despite lacking one

 @example
 ```ts
 const exempt = isExemptByShape({ module, runnerEntrySources, },);
 ```
 */
function isExemptByShape(
  {
    module,
    runnerEntrySources,
  }: {
    readonly module: SourceText;
    readonly runnerEntrySources: ReadonlySet<string>;
  },
): boolean {
  return runnerEntrySources.has(module.path,)
    || isBarrelModule({ module, },)
    || isTypesOnlyModule({ module, },);
}

/**
 Locations, sorted, of every production module that lacks an
 own test and is not exempt by shape: the live set `ALLOWLIST` tracks.

 @param files - files read, production modules and their tests alike

 @returns Locations needing an `ALLOWLIST` line, sorted

 @example
 ```ts
 const needing = modulesNeedingAllowlistEntry({ files, },); // ['stage/nap-schedule.ts']
 ```
 */
function modulesNeedingAllowlistEntry({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Locations of every `.unit.test.ts` file read, own-test lookups check this.
   */
  const unitTestPaths = new Set(files
    .filter(function isUnitTest({ path, },): boolean { return path.endsWith('.unit.test.ts',); },)
    .map(function pathOf({ path, },): string { return path; },),);
  /**
   Paths under `src` of those, whichever package holds them.
   */
  const unitTestsUnderSrc = new Set([...unitTestPaths,]
    .map(function underSrc(path,): string { return pathUnderSrc({ path, },); },),);
  /**
   How many production modules sit at each path under `src`: more than one for a path every package repeats.
   */
  const modulesAt = Map.groupBy(
    files.filter(function isProduction({ path, },): boolean { return isProductionModulePath({ path, },); },),
    function underSrc({ path, },): string { return pathUnderSrc({ path, },); },
  );
  /**
   Runner entry sources, read once so every module's `isExemptByShape` call
   reuses them.
   */
  const runnerEntrySources = runnerEntrySourcePaths({ files, },);
  return files
    .filter(function isProduction({ path, },): boolean { return isProductionModulePath({ path, },); },)
    .filter(function lacksOwnTest({ path, },): boolean {
      return !(unitTestPaths.has(ownTestPathFor({ path, },),)
        || (((modulesAt.get(pathUnderSrc({ path, },),) ?? []).length === 1)
          && unitTestsUnderSrc.has(ownTestPathFor({ path: pathUnderSrc({ path, },), },),)));
    },)
    .filter(function notExempt(module,): boolean { return !isExemptByShape({ module, runnerEntrySources, },); },)
    .map(function toPath({ path, },): string { return path; },)
    .toSorted();
}

/**
 `ALLOWLIST` entries now stale, and live-set entries still missing from
 `ALLOWLIST`: the ratchet's two failure kinds.
 */
type RatchetFindings = {
  /**
   Locations the live set holds that `ALLOWLIST` does not: a
   module lacking an own test and not exempt by shape, left unlisted.
   */
  readonly unlisted: readonly string[];

  /**
   `ALLOWLIST` entries the live set no longer holds: the module gained an
   own test, turned out exempt by shape, or no longer exists.
   */
  readonly stale: readonly string[];
};

/**
 Compares the live set of modules needing an `ALLOWLIST` line against
 `ALLOWLIST` itself, in both directions.

 @param files - files read, production modules and their tests alike

 @param allowlist - the ratchet list compared against

 @returns Unlisted live entries and stale allowlist entries, each sorted

 @example
 ```ts
 const findings = ownTestRatchetFindings({ files, allowlist: ALLOWLIST, },);
 ```
 */
function ownTestRatchetFindings(
  {
    files,
    allowlist,
  }: {
    readonly files: readonly SourceText[];
    readonly allowlist: readonly string[];
  },
): RatchetFindings {
  /**
   The live set: paths needing an `ALLOWLIST` line right now.
   */
  const live = modulesNeedingAllowlistEntry({ files, },);
  /**
   The live set, for the stale-entry direction's membership check.
   */
  const liveSet = new Set(live,);
  /**
   `ALLOWLIST` itself, for the unlisted-entry direction's membership check.
   */
  const allowedSet = new Set(allowlist,);
  return {
    unlisted: live.filter(function notAllowed(path,): boolean { return !allowedSet.has(path,); },),
    stale: allowlist.filter(function notLive(path,): boolean { return !liveSet.has(path,); },),
  };
}

//endregion Own-test ratchet scan

/**
 Production modules under `src` that lack an own test and are not exempt by
 shape, sorted: the 300 found when this scan was added (ledger B102), each
 logic some other module's test reaches or no test reaches. An entry goes
 when its module gains an own test, turns out exempt by shape, or is
 removed; adding one is a reviewed decision.
 */
const ALLOWLIST: readonly string[] = [
  'shared-contract/active-footnote-markers.ts',
  'document-structure/align-blocks-walk.ts',
  'page-pass/apply-patch-markup.ts',
  'archive/archive-block-naturalness.ts',
  'archive/archive-block-selection-evidence.ts',
  'archive/archive-replacement-candidates.ts',
  'artifact/artifact-schema-version.ts',
  'artifact/artifact-v1-read.ts',
  'assembly/assembly-container-halves.ts',
  'assembly/assembly-contract-fault.ts',
  'assembly/assembly-regressions.ts',
  'provider/bedrock-bound-ledger.ts',
  'translate-floor/bench-seating.ts',
  // The split gave each runner package its own build-entries.ts, and the one build-entries.unit.test.ts went with
  // the coverage census, the first package to hold the file. The five copies listed here have no test of their own.
  'bench/build-entries.ts',
  'corpus-pass/build-entries.ts',
  'corpus-pass/claude-schema-producers.ts',
  'probe/build-entries.ts',
  'run/build-entries.ts',
  'slice-cache/build-entries.ts',
  'house-rule/bilingual-line-clause.ts',
  'house-rule/candidate-judge-rules.ts',
  'adjudication/candidate-select-count.ts',
  'house-rule/candidate-select-model.ts',
  'translate-stage/candidate-select-record.ts',
  'provider/chat-contract.ts',
  'shared-contract/chunk-insertion.ts',
  'reference/cited-reference-lookup.ts',
  'slice-reading/cited-reference-rule.ts',
  'reference/cited-reference-scan.ts',
  'repair/claim-panel-voters.ts',
  'slice-reading/closing-punctuation.ts',
  'glossary/community-glossary-words.ts',
  'artifact/consolidate-archive-stand-in.ts',
  'corpus-pass/consolidate-driver-acquire.ts',
  'corpus-pass/consolidate-driver-index.ts',
  'corpus-pass/consolidate-driver-records.ts',
  'corpus-pass/consolidate-driver-subject.ts',
  'consolidate/consolidate-persistence.ts',
  'consolidate/consolidate-settle-context.ts',
  'consolidate/consolidate-settle-gate.ts',
  'consolidate/consolidation-ledger-gap.ts',
  'consolidate/consolidation-naturalness-state.ts',
  'consolidate/consolidation-polish-gate-wire.ts',
  'consolidate/consolidation-polish-round.ts',
  'consolidate/consolidation-polish-skip.ts',
  'house-rule/contributor-translation-guard.ts',
  'page-check/corpus-run/archive-block-source-context.ts',
  'page-check/corpus-run/archive-italic-spans.ts',
  'page-check/corpus-run/archive-name-runs.ts',
  'artifact-pool/corpus-run/artifact-generation.ts',
  'artifact-pool/corpus-run/artifact-pool.ts',
  'artifact/corpus-run/artifact-producer-read.ts',
  'artifact-pool/corpus-run/artifact-two-lane-comparison-fault.ts',
  'artifact/corpus-run/artifact-two-lane-contract.ts',
  'artifact/corpus-run/artifact-two-lane-derive.ts',
  'artifact/corpus-run/artifact-two-lane-read-consolidate-slice.ts',
  'artifact/corpus-run/artifact-two-lane-read-contest-ballot.ts',
  'artifact/corpus-run/artifact-two-lane-read-contest-eligibility.ts',
  'artifact/corpus-run/artifact-two-lane-read-contest-verdict.ts',
  'artifact/corpus-run/artifact-two-lane-read-evidence.ts',
  'artifact/corpus-run/artifact-two-lane-read-fields.ts',
  'artifact/corpus-run/artifact-two-lane-read-lanes.ts',
  'artifact/corpus-run/artifact-two-lane-read-naturalness-confirmation.ts',
  'artifact/corpus-run/artifact-two-lane-read-naturalness-digest.ts',
  'artifact/corpus-run/artifact-two-lane-read-naturalness-round.ts',
  'artifact/corpus-run/artifact-two-lane-read-naturalness-seat.ts',
  'artifact/corpus-run/artifact-two-lane-read-polish-gate.ts',
  'artifact/corpus-run/artifact-two-lane-read-rows.ts',
  'artifact-pool/corpus-run/artifact-two-lane-row-equality.ts',
  'artifact/corpus-run/artifact-vote-read.ts',
  'probe/corpus-run/attribution-decode.ts',
  'rendering-audit/corpus-run/attribution-line.ts',
  'bench/corpus-run/bench-sample.ts',
  'page-check/corpus-run/blockquote-paragraphs.ts',
  'page-check/corpus-run/canadian-date-parts.ts',
  'page-check/corpus-run/canadian-date-read-leading.ts',
  'page-check/corpus-run/canadian-date-read.ts',
  'page-check/corpus-run/canadian-date-words.ts',
  'page-check/corpus-run/canadian-date.ts',
  'page-check/corpus-run/canadian-spelling-capital.ts',
  'page-check/corpus-run/canadian-spelling-context.ts',
  'page-check/corpus-run/canadian-spelling-pairs-doubled-l.ts',
  'page-check/corpus-run/canadian-spelling-pairs-word.ts',
  'page-check/corpus-run/canadian-spelling-pairs.ts',
  'page-check/corpus-run/canadian-spelling-words.ts',
  'run/corpus-run/cap-census-read.ts',
  'run/corpus-run/cap-census-rule.ts',
  'coverage-census/corpus-run/coverage-census-baseline.ts',
  'coverage-census/corpus-run/coverage-file.ts',
  'coverage-census/corpus-run/coverage-pieces.ts',
  'page-check/corpus-run/destination-completeness.ts',
  'probe/corpus-run/draw-reconcile.ts',
  'page-check/corpus-run/final-naturalness-completeness.ts',
  'insertion/corpus-run/insertion-carried-anchor.ts',
  'insertion/corpus-run/insertion-carried-decide.ts',
  'insertion/corpus-run/insertion-carried-neighbours.ts',
  'insertion/corpus-run/insertion-carried-shift.ts',
  'insertion/corpus-run/insertion-container-blocks.ts',
  'insertion/corpus-run/insertion-coverage-model.ts',
  'run/corpus-run/ledger-parse.ts',
  'page-check/corpus-run/line-ending-fold.ts',
  'page-check/corpus-run/page-agreement.ts',
  'page-check/corpus-run/page-assembly-passes.ts',
  'page-check/corpus-run/page-slice-rewrite.ts',
  'corpus-pass/corpus-run/pass-archive.ts',
  'corpus-pass/corpus-run/pass-carried-fold.ts',
  'corpus-pass/corpus-run/pass-consolidate-reseat.ts',
  'corpus-pass/corpus-run/pass-consolidate.ts',
  'corpus-pass/corpus-run/pass-contest-reseat.ts',
  'corpus-pass/corpus-run/pass-contest.ts',
  'corpus-pass/corpus-run/pass-entry-artifact.ts',
  'corpus-pass/corpus-run/pass-entry-caches.ts',
  'corpus-pass/corpus-run/pass-entry-persist.ts',
  'corpus-pass/corpus-run/pass-entry-settle.ts',
  'corpus-pass/corpus-run/pass-footnote-relabel-read.ts',
  'corpus-pass/corpus-run/pass-insertion-reseat.ts',
  'corpus-pass/corpus-run/pass-lanes.ts',
  'corpus-pass/corpus-run/pass-page-assembly.ts',
  'corpus-pass/corpus-run/pass-page-guards.ts',
  'corpus-pass/corpus-run/pass-pictures-reseat.ts',
  'corpus-pass/corpus-run/pass-prepare-reseat.ts',
  'corpus-pass/corpus-run/pass-prepare.ts',
  'corpus-pass/corpus-run/pass-seated-pictures.ts',
  'corpus-pass/corpus-run/pass-visual-evidence.ts',
  'page-check/corpus-run/published-page-disagreement.ts',
  'probe/corpus-run/rendering-audit-settled-band.ts',
  'probe/corpus-run/rendering-audit-settled-buy.ts',
  'probe/corpus-run/rendering-audit-settled-digest.ts',
  'probe/corpus-run/rendering-audit-settled-relocation.ts',
  'probe/corpus-run/rendering-audit-settled-runs.ts',
  // The repeat census splits (the command `repeat-census.ts` is a runner entry and exempt):
  // a read-only measurement of the pinned corpus. Their logic runs in the command's own
  // pass, whose positive control (the live case's refrain and sentence lines must be found
  // by the line reading and by the package's detection) ends the command at the stated
  // refusal code when it reads false (`repeat-census-control.unit.test.ts` and
  // `repeat-census-report.unit.test.ts` hold that decision), and the detection it runs has
  // its own cases in `repeat-groups.unit.test.ts`. Each line goes when its module gains an
  // own test.
  'run/corpus-run/repeat-census-lines.ts',
  'run/corpus-run/repeat-census-maximal.ts',
  'run/corpus-run/repeat-census-package.ts',
  'run/corpus-run/repeat-census-page.ts',
  'run/corpus-run/repeat-census-refrain.ts',
  'run/corpus-run/repeat-census-summary.ts',
  'bench/corpus-run/roster-card-ask.ts',
  'run/corpus-run/run-config-error.ts',
  'run/corpus-run/run-providers.ts',
  'run/corpus-run/run-timing-parse.ts',
  'run/corpus-run/run-timing-read.ts',
  'slice-cache/corpus-run/slice-cache-claims.ts',
  'slice-cache/corpus-run/slice-cache-dir-read.ts',
  'page-check/corpus-run/text-runs.ts',
  'page-pass/corpus-run/title-reference-locate.ts',
  'page-pass/corpus-run/title-reference-rewrite.ts',
  'archive/coverage-foreign-region.ts',
  'provider/decision-context-refusal.ts',
  'adjudication/declared-identity-rule.ts',
  'adjudication/declared-names-evidence.ts',
  'repair/derive-seeds.ts',
  'corpus-source/displacement-ratio.ts',
  'document-structure/document-node.ts',
  'repair/editor-envelope-context.ts',
  'repair/editor-selection-sheet.ts',
  'probe/fidelity-reference-build.ts',
  'probe/fidelity-reference-error.ts',
  'probe/fidelity-reference-manifest.ts',
  'probe/fidelity-reference-select.ts',
  'probe/fidelity-reference-text.ts',
  'probe/fidelity-reference-trials.ts',
  'repair/fidelity-window-positions.ts',
  'page-pass/footnote-closure-input.ts',
  'page-pass/footnote-protected-ranges.ts',
  'page-pass/footnote-retained-labels.ts',
  'document-structure/footnote-rewrite-error.ts',
  'page-pass/footnote-rewrite-map.ts',
  'shared-contract/footnote-slice-labels.ts',
  'slice-reading/front-matter-comment-authority.ts',
  'repair/front-matter-repair.ts',
  'translate-floor/front-matter-translation.ts',
  'translate-floor/group-merge.ts',
  'translate-floor/group-nodes.ts',
  'translate-floor/group-source-anchor.ts',
  'markup/han-title-read.ts',
  'house-rule/house-form-corrections.ts',
  'house-rule/house-policy.ts',
  'house-rule/identity-han-items.ts',
  'markup/inline-container-tags.ts',
  'lane-contest/lane-comparison-fault.ts',
  'translate-floor/lane-slice-coverage-error.ts',
  'markup/lexical-restoration.ts',
  'slice-reading/line-starts.ts',
  'slice-reading/line-structure-addendum.ts',
  'slice-reading/line-structure-inherit.ts',
  'house-rule/linked-title-declared-name.ts',
  'document-structure/markdown-blocks.ts',
  'adjudication/naturalness-completeness-error.ts',
  'adjudication/naturalness-quorum.ts',
  'adjudication/naturalness-repair-interrupted-error.ts',
  'markup/no-provider-for-model-error.ts',
  'provider/openrouter-chunk-scan.ts',
  'provider/pace-saturation.ts',
  'shared-contract/page-apparatus-clause.ts',
  'rendering-audit/page-headings.ts',
  'artifact/page-title-lexicon-stage.ts',
  'artifact/page-title-lexicon-wire.ts',
  'rendering-audit/page-visible-text.ts',
  'stage/pair-contested-target.ts',
  'slice-reading/pairing-pictures.ts',
  'slice-reading/pairing-question-key.ts',
  'adjudication/panel-stage.ts',
  'document-structure/parse-slice-body.ts',
  'shared-contract/patch-model.ts',
  'shared-contract/per-model-limiter.ts',
  'house-rule/polish-gate-house-rules.ts',
  'corpus-source/preparation-seal.ts',
  'corpus-source/preparation-unclaimed.ts',
  'archive/prepare-block-pairing-finish.ts',
  'run/probe-issue-index.ts',
  'document-structure/protected-atom.ts',
  'provider/neuralwatt-account-read.ts',
  'provider/neuralwatt-day-anchor.ts',
  'provider/neuralwatt-day-session.ts',
  'provider/provider-meters.ts',
  'run/corpus-run/run-seats-short-bench.ts',
  'run/corpus-run/run-seats-states.ts',
  'provider/provider-router-reask.ts',
  'slice-reading/queried-block-pairing-details.ts',
  'provider-stream/quote-depth-clamp.ts',
  'typography/quote-neighbours.ts',
  'typography/read-text-if-present.ts',
  'reference/reference-attest-claims.ts',
  'reference/reference-attest-confirm-wire.ts',
  'reference/reference-attest-match.ts',
  'reference/reference-attest-stage.ts',
  'reference/reference-attest-wire.ts',
  'repair/refine-phase-slice.ts',
  'repair/refine-selection-context.ts',
  'assembly/repair-assemble.ts',
  'repair/repair-chunk-evidence.ts',
  'repair/repair-chunk-proof.ts',
  'repair/repair-chunk-settle.ts',
  'repair/repair-chunk.ts',
  'repair/repair-entry.ts',
  'assembly/repair-replacements.ts',
  'repair/repair-scorecard.ts',
  'repair/repair-slice-settle.ts',
  'repair/repair-stage-findings.ts',
  'house-rule/resolution-authorship.ts',
  'page-pass/restore-ellipsis.ts',
  'provider/retry-stated-wait.ts',
  'stage/roster-quorum-size.ts',
  'lane-contest/sample-draw-identity.ts',
  'adjudication/sealed-node-ids.ts',
  'lane-contest/seed-detection.ts',
  'adjudication/select-decline-consequence.ts',
  'adjudication/severity-scale.ts',
  'translate-floor/slice-cost-log.ts',
  'translate-floor/slice-delivery-decide.ts',
  'translate-floor/slice-delivery-fault.ts',
  'translate-floor/soft-break-fold.ts',
  'slice-reading/span-edge-match.ts',
  'provider/stream-bound-hold.ts',
  'provider-stream/stream-delivered-chars.ts',
  'adjudication/tally-claim.ts',
  'adjudication/tally-issue.ts',
  'translate-floor/translate-absence.ts',
  'glossary/translate-address-original.ts',
  'translate-lane/translate-alignment-refusals.ts',
  'translate-lane/translate-archive-floor.ts',
  'translate-floor/translate-atom-floor.ts',
  'translate-floor/translate-candidates.ts',
  'translate-floor/translate-community-term.ts',
  'translate-floor/translate-declared-link-name.ts',
  'translate-floor/translate-definition-leak.ts',
  'translate-floor/translate-escape-leak.ts',
  'translate-floor/translate-han-residue.ts',
  'translate-floor/translate-han-title.ts',
  'translate-floor/translate-marker-drop.ts',
  'translate-stage/translate-produce.ts',
  'translate-stage/translate-runoff-tie.ts',
  'translate-floor/translate-sheet-leak.ts',
  'translate-floor/translate-signer-handle.ts',
  'glossary/translate-skeleton-page.ts',
  'translate-stage/translate-slate-evidence.ts',
  'translate-lane/translate-slice-buy.ts',
  'translate-lane/translate-slice-settle.ts',
  'translate-floor/translate-source-carry.ts',
  'translate-stage/translate-stage-repair.ts',
  'glossary/translate-suicide-drop.ts',
  'translate-stage/translate-unfloored.ts',
  'translate-floor/translate-unwrapped-link.ts',
  'glossary/translate-validate-blocks.ts',
  'translate-stage/translated-slate-criteria.ts',
  'translate-stage/translation-repair-interrupted-error.ts',
  'stage/unpartnered-gap-steps.ts',
  'stage/unprepared-slice.ts',
  'provider/upstream-model-hold.ts',
  'reference/work-title-scan.ts',
];

await describe({
  name: 'own unit tests ratchet (ledger B102)',
  children: [
    it({
      name: 'FINDS a logic module with no own test missing from ALLOWLIST, the library index among them, and an '
        + 'ALLOWLIST entry whose module already has an own test; PASSES a module with an own test, a barrel, a '
        + 'types-only module, and a runner entry build-entries.ts lists',
      fn: async () => {
        expect(ownTestRatchetFindings({
          files: [
            {
              path: 'whisker-count.ts',
              isTest: false,
              text: 'export function whiskerCount(cat: string): number { return cat.length; }',
            },
            {
              path: 'whisker-count.unit.test.ts',
              isTest: true,
              text: 'export const tabby = 1;',
            },
            {
              path: 'cat-barrel.ts',
              isTest: false,
              text: [
                'export { Purr, } from \'./purr.ts\';',
                'export * from \'./meow.ts\';',
                'export type { CatBreed, } from \'./cat-breed.ts\';',
              ].join('\n',),
            },
            {
              path: 'cat-breed.ts',
              isTest: false,
              text: [
                'export type CatBreed = \'tabby\' | \'calico\';',
                'export const DEFAULT_BREED = \'tabby\' as const;',
                'export const WHISKER_COUNT = 12;',
                'export const FAVOURITE_NAPS = [\'sunbeam\', \'laundry basket\',] as const;',
              ].join('\n',),
            },
            {
              path: 'nap-schedule.ts',
              isTest: false,
              text: 'export function napDuration(hours: number): number { return hours * 60; }',
            },
            {
              path: 'build-entries.ts',
              isTest: false,
              text: 'export const nodeEntries = { purr: \'./src/purr-cli.ts\', index: \'./src/index.ts\', } as const;',
            },
            {
              path: 'purr-cli.ts',
              isTest: false,
              text: 'export function purrLoudly(times: number): string { return \'purr\'.repeat(times); }',
            },
            {
              path: 'index.ts',
              isTest: false,
              text: 'export function meow(): string { return \'meow\'; }',
            },
          ],
          allowlist: ['whisker-count.ts',],
        },),).toEqual({
          unlisted: [
            'index.ts',
            'nap-schedule.ts',
          ],
          stale: ['whisker-count.ts',],
        },);
      },
    },),
    it({
      name: 'HOLDS EVERY MODULE WITHOUT AN OWN TEST in ALLOWLIST and NO STALE ALLOWLIST ENTRY in this package',
      fn: async () => {
        /**
         Every package file, tests among them, which own-test lookups need.
         */
        const files = await readPackageSource();
        expect(files.length,).toBeGreaterThan(0,);
        expectRecordAsListed({
          found: ownTestRatchetFindings({ files, allowlist: ALLOWLIST, },),
          listed: {
            unlisted: [],
            stale: [],
          },
        },);
      },
    },),
  ],
},);
