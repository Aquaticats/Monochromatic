import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type CommandLineOf,
  reportingRefusals,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  packageCommit,
  sourcesEditedSince,
} from './coverage-census-commit.ts';
import { censusScopeOf, } from './coverage-census-family.ts';
import { placeTally, } from './coverage-census-place.ts';
import { runCoverageCensus, } from './coverage-census-run.ts';
import {
  runSuite,
  tallyCoverage,
} from './coverage-census-steps.ts';

//region Coverage census
// LEDGER T8: which package code the unit suite never runs, as a command: the
// wiring only. What it does, and how it reads the command line, is
// `coverage-census-run.ts`, which says what it measures and how a batch proves
// its reach; the pieces are `coverage-census-build.ts`,
// `coverage-census-reading.ts` and `coverage-census-baseline-lines.ts`. What a
// real process has is the package directory it stands in, its environment and
// the steps that touch git, the suite and the coverage.
//
// RUN FROM THE PACKAGE DIRECTORY, which the task does:
// `mise run //package/translation-repair/coverage-census:coverage-census`. A package
// with a coverage build config measures the family of packages beside it; any
// other directory is measured alone (`coverage-census-family.ts`).

/**
 Logger for the census's progress lines, apart from the report on stdout.
 */
const censusLog = contextRoot({ tag: 'coverage-census', },);

if (import.meta.main)
  await reportingRefusals({
    what: 'coverage-census',
    argv: process.argv,
    env: process.env,
    run: async function runCensus({ line, }: { readonly line: CommandLineOf<'coverage-census'>; },): Promise<void> {
      /**
       The family the directory stands in, or the directory alone.
       */
      const scope = await censusScopeOf({ packageDirectory: process.cwd(), },);
      await runCoverageCensus({
        line,
        packageDirectory: process.cwd(),
        ...((scope.kind === 'family') ? { family: scope.family, } : {}),
        env: process.env,
        steps: {
          packageCommit,
          runSuite,
          tallyCoverage,
          placeTally,
          sourcesEditedSince,
        },
        l: censusLog,
      },);
    },
  },);

//endregion Coverage census
