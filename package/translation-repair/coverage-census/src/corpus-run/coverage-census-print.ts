import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  ColdSince,
  EditedClaim,
  EmptyClaim,
  StretchStatus,
} from './coverage-census-baseline.ts';
import {
  invariantThrowCountLine,
  invariantThrowListLines,
  type InvariantThrowRow,
} from './coverage-census-invariant.ts';
import {
  type CensusStretch,
  type KindTotal,
  ownSourceOf,
  type SourceKind,
  type SourceRow,
} from './coverage-census-report.ts';
import type { MappedFunction, } from './coverage-lines.ts';

//region Coverage census print
// Ledger T8: the census's report as lines, apart from the entry that runs the
// suite, so what an operator reads is tested without a subprocess. Every line
// names package files, lines, bundle names and counts; no corpus text reaches
// a coverage report. The invariant throws stand apart from the cold code in
// it: their count beside the cold counts, and each by source and line
// (`coverage-census-invariant.ts`).

/**
 A source a runner bundle carries that no test loaded.

 @example
 ```ts
 const unloaded: UnloadedSource = { source: 'src/corpus-run/corpus-pass.ts', kind: 'entry file', lines: 400, };
 ```
 */
export type UnloadedSource = {
  /**
   Source file relative to the package.
   */
  readonly source: string;

  /**
   Where it sits.
   */
  readonly kind: SourceKind;

  /**
   Its physical lines.
   */
  readonly lines: number;
};

/**
 What the census found, ready to print.

 @example
 ```ts
 const lines = censusReportLines({ census, },);
 ```
 */
export type CensusSummary = {
  /**
   Commit the build was made from.
   */
  readonly head: string;

  /**
   Whether the package's files matched that commit.
   */
  readonly clean: boolean;

  /**
   Test files asked for, empty for the whole unit suite.
   */
  readonly testFiles: readonly string[];

  /**
   `[PASS]` markers the suite printed.
   */
  readonly passes: number;

  /**
   Totals per kind.
   */
  readonly totals: readonly KindTotal[];

  /**
   One row per source holding cold code.
   */
  readonly rows: readonly SourceRow[];

  /**
   One row per stretch that is nothing but invariant throws, which no total
   and no row of cold code counts, in report order.
   */
  readonly invariantThrows: readonly InvariantThrowRow[];

  /**
   Uncalled functions with their lines.
   */
  readonly uncalled: readonly MappedFunction[];

  /**
   Bundles no test process loaded.
   */
  readonly unloadedBundles: readonly string[];

  /**
   Sources only those bundles carry.
   */
  readonly unloadedSources: readonly UnloadedSource[];

  /**
   Where the census file was written.
   */
  readonly censusPath: string;
};

/**
 The report an operator reads after a census.

 @param census - what the census found

 @returns Lines to print, in order

 @example
 ```ts
 for (const line of censusReportLines({ census, },)) console.log(line,);
 ```
 */
export function censusReportLines({ census, }: { readonly census: CensusSummary; },): readonly string[] {
  /**
   Test files the census ran, where none means the whole unit suite.
   */
  const testFileCount = census.testFiles
    .length;

  /**
   Which tests ran.
   */
  const scope = (testFileCount === 0) ? 'the unit suite' : `${String(testFileCount,)} test ${
    wordForCount({
      count: testFileCount,
      one: 'file',
      many: 'files',
    },)
  }`;
  /**
   Uncalled functions in package source that no uncalled function holds.
   */
  const outermost = census.uncalled
    .flatMap(function outermostInPackage(fn,): readonly {
      readonly source: string;
      readonly line: number;
      readonly name: string;
    }[] {
      if (fn.nested || (fn.at
        .kind
        === 'unmapped')
        || (ownSourceOf({ source: fn.at
          .source, },)
          .kind
          === 'foreign'))
        return [];
      return [{
        source: fn.at
          .source,
        line: fn.at
          .line,
        name: (fn.name === '') ? '(anonymous)' : fn.name,
      },];
    },)
    .toSorted(function bySourceThenLine(
      left,
      right,
    ): number {
      return compareCodePoints({
        left: left.source,
        right: right.source,
      },)
        || (left.line - right.line);
    },)
    .map(function placed(fn,): string {
      return `  ${fn.source}:${String(fn.line,)} ${fn.name}`;
    },);
  /**
   Physical lines the unloaded sources hold.
   */
  const unloadedLines = census.unloadedSources
    .reduce(
    function sum(
      total,
      unloaded,
    ): number {
      return total + unloaded.lines;
    },
    0,
    );
  return [
    `coverage-census at ${census.head}${census.clean ? '' : ' with uncommitted changes'}: ${scope}, ${String(census.passes,)} ${
      wordForCount({
        count: census.passes,
        one: 'pass',
        many: 'passes',
      },)
    }`,
    ...census.totals
      .map(function totalLine(total,): string {
        return `${total.kind}: ${String(total.files,)} ${
        wordForCount({
          count: total.files,
          one: 'file',
          many: 'files',
        },)
      }, ${String(total.stretches,)} ${
        wordForCount({
          count: total.stretches,
          one: 'stretch',
          many: 'stretches',
        },)
      } over ${String(total.lines,)} ${
        wordForCount({
          count: total.lines,
          one: 'line',
          many: 'lines',
        },)
      }, `
          + `${String(total.uncalled,)} ${
          wordForCount({
            count: total.uncalled,
            one: 'function',
            many: 'functions',
          },)
        } never called`;
      },),
    invariantThrowCountLine({ rows: census.invariantThrows, },),
    `bundles no test loaded: ${String(census.unloadedBundles
      .length,)}, carrying ${String(census.unloadedSources
        .length,)} ${
      wordForCount({
        count: census.unloadedSources
          .length,
        one: 'source',
        many: 'sources',
      },)
    } `
      + `and ${String(unloadedLines,)} physical ${
      wordForCount({
        count: unloadedLines,
        one: 'line',
        many: 'lines',
      },)
    }`,
    ...census.unloadedSources
      .filter(function isLibrary(unloaded,): boolean {
        return unloaded.kind === 'library source';
      },)
      .map(function unloadedLine(unloaded,): string {
        return `  library source only those bundles carry: ${unloaded.source}`;
      },),
    'library source by cold lines:',
    ...census.rows
      .filter(function isLibrary(row,): boolean {
        return row.kind === 'library source';
      },)
      .map(function rowLine(row,): string {
        return `  ${row.source}: ${String(row.stretches,)} ${
          wordForCount({
            count: row.stretches,
            one: 'stretch',
            many: 'stretches',
          },)
        }, ${String(row.lines,)} ${
          wordForCount({
            count: row.lines,
            one: 'line',
            many: 'lines',
          },)
        }, ${String(row.uncalled,)} never called`;
      },),
    ...invariantThrowListLines({ rows: census.invariantThrows, },),
    'functions never called in package source, outermost:',
    ...outermost,
    `census written to ${census.censusPath}`,
  ];
}

/**
 Says what this run left in one source, which is all that speaks for a
 source the baseline cannot.

 @param loadedNow - whether a bundle this run loaded carried it

 @param coldNow - cold stretches this run left in it

 @returns Clause for a report line

 @example
 ```ts
 const now = standingNow({ loadedNow: true, coldNow: 0, },);
 ```
 */
function standingNow(
  {
    loadedNow,
    coldNow,
  }: {
    readonly loadedNow: boolean;
    readonly coldNow: number;
  },
): string {
  return loadedNow
    ? `this run loaded it and left ${String(coldNow,)} cold ${
      wordForCount({
        count: coldNow,
        one: 'stretch',
        many: 'stretches',
      },)
    }`
    : 'this run did not load it';
}

/**
 The report of a batch read against an earlier census.

 @param path - earlier census read

 @param head - commit it was taken at

 @param statuses - each of its stretches in the claimed sources with its
 status, outside sources edited since that commit

 @param coldSince - this run's stretches in lines none of its stretches
 held, which its statuses cannot speak for (ledger B61)

 @param emptyClaims - claimed sources it holds no stretch in, which the
 statuses cannot speak for, so each is printed with both censuses' standing

 @param editedClaims - sources edited since its commit, whose baseline lines
 name other code now, so each is printed with this run's standing alone

 @returns Lines to print, in order: counts, then every stretch not proven
 run, then every stretch cold since, then every claimed source holding no
 baseline stretch, then every edited source

 @example
 ```ts
 for (const line of baselineReportLines({ path, head, statuses, coldSince, emptyClaims, editedClaims, },)) console.log(line,);
 ```
 */
export function baselineReportLines(
  {
    path,
    head,
    statuses,
    coldSince,
    emptyClaims,
    editedClaims,
  }: {
    readonly path: string;
    readonly head: string;
    readonly statuses: readonly {
      readonly stretch: CensusStretch;
      readonly status: StretchStatus;
    }[];
    readonly coldSince: readonly ColdSince[];
    readonly emptyClaims: readonly EmptyClaim[];
    readonly editedClaims: readonly EditedClaim[];
  },
): readonly string[] {
  /**
   How many stretches read as one status.

   @param status - status counted

   @returns Stretches reading as it
   */
  function countOf(status: StretchStatus,): number {
    return statuses.filter(function hasStatus(read,): boolean {
      return read.status === status;
    },)
      .length;
  }
  /**
   The counts the first line states, in order.
   */
  const counts = [
    `ran ${String(countOf('ran',),)}`,
    `still cold ${String(countOf('still cold',),)}`,
    `cold since then ${String(coldSince.length,)}`,
    `not loaded ${String(countOf('not loaded',),)}`,
    `claimed sources with no stretch there ${String(emptyClaims.length,)}`,
    `sources edited since then ${String(editedClaims.length,)}`,
  ];
  return [
    `against ${path} at ${head}: ${counts.join(', ',)}`,
    ...statuses
      .filter(function unproven(read,): boolean {
        return read.status !== 'ran';
      },)
      .map(function unprovenLine(read,): string {
        return `  ${read.status}: ${read.stretch
          .source}:${String(read.stretch
            .startLine,)}-${String(read.stretch
              .endLine,)}`;
      },),
    ...coldSince.map(function coldSinceLine({
      stretch,
      loadedAtBaseline,
    },): string {
      /**
       What the baseline says of the stretch's lines.
       */
      const then = loadedAtBaseline ? 'ran there' : 'not loaded there';
      /**
       Where the stretch sits.
       */
      const {
        source,
        startLine,
        endLine,
      } = stretch;
      return `  cold since then (${then}): ${source}:${String(startLine,)}-${String(endLine,)}`;
    },),
    ...emptyClaims.map(function emptyClaimLine({
      source,
      loadedAtBaseline,
      loadedNow,
      coldNow,
    },): string {
      /**
       What the baseline says of the source.
       */
      const then = loadedAtBaseline
        ? 'ran whole there'
        : 'not loaded there, so the baseline proves nothing of it';
      return `  no baseline stretch (${then}); ${
        standingNow({
          loadedNow,
          coldNow,
        },)
      }: ${source}`;
    },),
    ...editedClaims.map(function editedClaimLine({
      source,
      loadedNow,
      coldNow,
    },): string {
      return `  edited since ${head}, so its baseline lines name other code; ${
        standingNow({
          loadedNow,
          coldNow,
        },)
      }: ${source}`;
    },),
  ];
}

//endregion Coverage census print
