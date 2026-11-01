//region Public index
// The names `@monochromatic-dev/translation-repair-coverage-census` gives its siblings and its own tests, each from the module that declares it.
// Production siblings import it through `/ts`; tests and fixtures through the built entry, so a process loads one copy.

export {
  FAMILY_ENTRY_LISTS,
  FamilyEntryConflictError,
  familyEntriesOf,
  nodeEntries,
  runnerEntrySources,
} from './family-build-entries.ts';
export {
  type BundleMaps,
  bundleMapsOf,
  type MapNeed,
  requireMapFor,
  requireUnminifiedBuild,
} from './corpus-run/coverage-bundle-maps.ts';
export {
  type Baseline,
  baselineReadingsOf,
} from './corpus-run/coverage-census-baseline-lines.ts';
export {
  type BaselineCensus,
  baselineStatusesOf,
  type ColdSince,
  coldSinceOf,
  type EditedClaim,
  editedClaimsOf,
  type EmptyClaim,
  emptyClaimsOf,
  type StretchStatus,
} from './corpus-run/coverage-census-baseline.ts';
export { requireCoverageBuild, } from './corpus-run/coverage-census-build.ts';
export {
  type CensusFamily,
  type CensusScope,
  censusScopeOf,
  familySuiteRunsOf,
  type FamilySuiteRun,
  isFamilyBundleName,
  requireFamilyCoverageBuild,
} from './corpus-run/coverage-census-family.ts';
export {
  packageCommit,
  sourcesEditedSince,
} from './corpus-run/coverage-census-commit.ts';
export {
  CENSUS_FORMAT,
  type CensusArguments,
  CensusBaselineError,
  censusFileText,
  FAIL_MARKER,
  markerCount,
  PASS_MARKER,
  readBaselineCensus,
  readBaselineFile,
  readCensusArguments,
} from './corpus-run/coverage-census-input.ts';
export {
  invariantThrowCountLine,
  invariantThrowListLines,
  type InvariantThrowRow,
  invariantThrowRowsOf,
  type InvariantThrowStretch,
} from './corpus-run/coverage-census-invariant.ts';
export {
  type PlacedTally,
  placeTally,
  readMappedBundles,
} from './corpus-run/coverage-census-place.ts';
export {
  baselineReportLines,
  censusReportLines,
  type CensusSummary,
  type UnloadedSource,
} from './corpus-run/coverage-census-print.ts';
export {
  type ReadText,
  readUtf8Text,
} from './corpus-run/coverage-census-read-text.ts';
export { reportCensus, } from './corpus-run/coverage-census-reading.ts';
export {
  type CensusStretch,
  censusStretchesOf,
  isUnmappedSource,
  type KindTotal,
  kindTotalsOf,
  type OwnSource,
  ownSourceOf,
  requirePlacedFunctions,
  type SourceKind,
  sourceKindOf,
  type SourceRow,
  sourceRowsOf,
  unmappedSourceOf,
} from './corpus-run/coverage-census-report.ts';
export {
  type CoverageCensusSteps,
  readBaselines,
  runCoverageCensus,
} from './corpus-run/coverage-census-run.ts';
export {
  coverageReadings,
  readBundle,
  runSuite,
  tallyCoverage,
  unloadedSourcesOf,
} from './corpus-run/coverage-census-steps.ts';
export {
  type DecoyCut,
  decoyCut,
} from './corpus-run/coverage-control-decoy.ts';
export { gatherCases, } from './corpus-run/coverage-control-probe-cases.ts';
export {
  controlLines,
  offeringLine,
} from './corpus-run/coverage-control-probe-lines.ts';
export { runCoverageControl, } from './corpus-run/coverage-control-probe-run.ts';
export {
  type CoverageControlCase,
  coverageControlHolds,
  type CoverageControlRefusal,
  type CoverageControlResult,
  type CoverageControlRow,
  withoutSpans,
} from './corpus-run/coverage-control.ts';
export {
  type BundleScript,
  bundleScriptsOf,
  CoverageFileError,
  type CoverageRange,
  type FunctionCoverage,
} from './corpus-run/coverage-file.ts';
export {
  type StretchReading,
  stretchReadingOf,
} from './corpus-run/coverage-invariant-throw.ts';
export {
  type BundleLines,
  bundleLinesOf,
  mapFunction,
  type MappedFunction,
  readSourceMap,
  type SourceEntry,
  type SourceLine,
  sourceLineAt,
  SourceMapFileError,
} from './corpus-run/coverage-lines.ts';
export {
  type MappedStretch,
  mapStretch,
  type SourceSpan,
  type StretchPiece,
} from './corpus-run/coverage-pieces.ts';
export {
  DEFAULT_CANDIDATE_CAP,
  readCoverageProbeArguments,
} from './corpus-run/coverage-probe-args.ts';
export {
  type PairRead,
  readPair,
} from './corpus-run/coverage-probe-pair.ts';
export {
  answeredRow,
  failedRow,
  type ProbeRow,
  whereOf,
} from './corpus-run/coverage-probe-row.ts';
export { runCoverageProbe, } from './corpus-run/coverage-probe-run.ts';
export {
  type Atom,
  atomAt,
} from './corpus-run/coverage-stretch-atoms.ts';
export {
  type ColdStretch,
  type CoverageTally,
  CoverageTallyError,
  createCoverageTally,
  type FunctionSite,
  type UncalledFunction,
} from './corpus-run/coverage-tally.ts';

//endregion Public index
