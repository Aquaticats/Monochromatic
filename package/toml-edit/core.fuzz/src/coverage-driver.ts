/**
 Deterministic reachability driver for the toml-edit fuzz coverage gate.
 
 Samples the shared fuzz generators and loads the committed corpus at a fixed
 seed and run count, then replays them through the {@link coverage-exercise}
 operation spread, which imports the package implementation from source. Run
 under `NODE_V8_COVERAGE`, this attributes coverage to the `src` files the gate
 measures, and the fixed seed keeps the covered-line set reproducible so the
 committed baseline is stable.
 
 This is the reachability counterpart to the property suite (which tests the
 built artifact for correctness with random seeds); it asserts nothing.
 
 CLI: `node coverage-driver.ts [seed] [numRuns]`. The optional arguments let the
 saturation check rerun the sweep under a second seed.
 
 @module
 */

import {
  type Arbitrary,
  array,
  boolean,
  constantFrom,
  dictionary,
  double,
  integer,
  jsonValue,
  oneof,
  sample,
  string,
} from 'fast-check';

import { emptyTomlEdit, } from '@monochromatic-dev/toml-edit-core/ts';
import {
  DOCUMENT_EXAMPLES,
  documentArbitrary,
} from './arb-documents.ts';
import { corruptedDocumentArbitrary, } from './mutators.ts';
import { scalarSampleArbitrary, } from './arb-values.ts';
import type { ValueSample, } from './arb-types.ts';
import {
  loadInvalidFixtures,
  loadValidFixtures,
} from './corpus.ts';
import { tallySnapshot, } from './coverage-harness.ts';
import { exerciseValueEncoding, } from './coverage-edits.ts';
import {
  exerciseEditsAndComments,
  exerciseInvalidSource,
  exerciseSeams,
  exerciseValidSource,
} from './coverage-exercise.ts';
import {
  exerciseEmptyBase,
  exerciseExistingResets,
  exerciseParserEdges,
  exercisePendingProjections,
  exercisePendingReads,
} from './coverage-probes.ts';

//region Configuration

/**
 Default fast-check seed. The saturation check overrides it via argv to confirm
 the covered-line set does not depend on the seed.
 */
const DEFAULT_SEED = 0xC0_FF_EE;

/**
 Default sampled cases per arbitrary, high enough to saturate the reachable
 branches the gate watches.
 */
const DEFAULT_NUM_RUNS = 600;

/**
 Bound on sources fed through the base-independent edit and comment machinery,
 so the expensive deep sweep stays fast without losing reachability.
 */
const DEEP_SUBSET_LIMIT = 30;

/**
 Maximum generated array length for the edit-value arbitrary.
 */
const MAX_VALUE_ARRAY_LENGTH = 3;

/**
 Maximum generated key-name length for the seam arbitrary.
 */
const MAX_KEY_NAME_LENGTH = 12;

/**
 Optional seed and run-count CLI arguments (positions two and three).
 */
const [
  seedArg,
  runsArg,
] = process.argv
  .slice(2,);

/**
 Resolved fast-check seed for this run.
 */
const seed = Number.isInteger(Number(seedArg,),) ? Number(seedArg,) : DEFAULT_SEED;

/**
 Resolved sampled-cases-per-arbitrary count for this run.
 */
const numRuns = (Number.isInteger(Number(runsArg,),) && (Number(runsArg,) > 0)) ? Number(runsArg,) : DEFAULT_NUM_RUNS;

/**
 Shared deterministic sampling parameters.
 */
const sampleParams = {
  numRuns,
  seed,
};

//endregion Configuration

//region Local arbitraries

/**
 TOML-representable scalar values for the edit sweeps.
 */
const scalarValueArbitrary: Arbitrary<unknown> = oneof(
  string(),
  integer(),
  boolean(),
  double({
    noNaN: true,
    noDefaultInfinity: true,
  },),
);

/**
 Flat object emitted as a table or inline table.
 */
const flatObjectArbitrary = dictionary(
  constantFrom(
    'x',
    'y',
  ),
  scalarValueArbitrary,
);

/**
 Edit value arbitrary: scalars, flat tables, scalar arrays, arrays-of-tables.
 */
const editValueArbitrary: Arbitrary<unknown> = oneof(
  scalarValueArbitrary,
  flatObjectArbitrary,
  array(
    scalarValueArbitrary,
    { maxLength: MAX_VALUE_ARRAY_LENGTH, },
  ),
  array(
    flatObjectArbitrary,
    {
      minLength: 1,
      maxLength: MAX_VALUE_ARRAY_LENGTH,
    },
  ),
);

/**
 Adversarial key-name arbitrary spanning bare, empty, dotted-looking,
 numeric-looking, quote-bearing, and unicode names.
 */
const keyNameArbitrary: Arbitrary<string> = oneof(
  string({
    unit: constantFrom(
      'a',
      'Z',
      '0',
      '_',
      '-',
      '.',
      ' ',
      '"',
      '\\',
      '#',
      'é',
      '😀',
    ),
    maxLength: MAX_KEY_NAME_LENGTH,
  },),
  constantFrom(
    '',
    'a.b',
    '123',
    '3.14',
    'has space',
    'quote"x',
    'é€',
  ),
);

//endregion Local arbitraries

//region Sampling

/**
 Generated documents, pinned examples first so they always run.
 */
const documents: readonly string[] = [
  ...DOCUMENT_EXAMPLES,
  ...sample(
    documentArbitrary,
    sampleParams,
  ),
];

/**
 Mutator-corrupted documents for the parser error paths.
 */
const corrupted: readonly string[] = sample(
  corruptedDocumentArbitrary,
  sampleParams,
);

/**
 Edit values for the from-scratch value-encoder sweep.
 */
const editValues: readonly unknown[] = sample(
  editValueArbitrary,
  sampleParams,
);

/**
 Key names for the `_encodeKey` seam.
 */
const keyNames: readonly string[] = sample(
  keyNameArbitrary,
  sampleParams,
);

/**
 Scalar value spellings for the node re-emission seams.
 */
const scalarTexts: readonly string[] = sample(
  scalarSampleArbitrary,
  sampleParams,
)
  .map(function text(value: ValueSample,) { return value.text; },);

/**
 Arbitrary JSON values for the `_jsValueToTomlText` seam.
 */
const jsonValues: readonly unknown[] = sample(
  jsonValue(),
  sampleParams,
);

/**
 Committed valid fixtures, loaded deterministically (bounded mode).
 */
const validCorpus = await loadValidFixtures();

/**
 Committed invalid fixtures for the parser rejection paths.
 */
const invalidCorpus = await loadInvalidFixtures();

/**
 Canonical options drawn once from a fresh empty state, fed to the seams.
 */
const canonicalOptions = emptyTomlEdit()
  .canonical;

//endregion Sampling

//region Sweep

exerciseEmptyBase();

exerciseParserEdges();

exercisePendingReads();

exercisePendingProjections();

exerciseValueEncoding({ values: editValues, },);

exerciseSeams({
  keyNames,
  jsonValues,
  scalarTexts,
  canonicalOptions,
},);

for (const source of documents) {
  exerciseValidSource({ source, },);
}

for (const entry of validCorpus) {
  exerciseValidSource({ source: entry.source, },);
}

/**
 Bounded, diverse subset for the base-independent edit and comment machinery.
 */
const deepSources: readonly string[] = [
  ...documents.slice(
    0,
    DEEP_SUBSET_LIMIT,
  ),
  ...validCorpus.slice(
    0,
    DEEP_SUBSET_LIMIT,
  )
    .map(function pick(entry,) { return entry.source; },),
];

for (const source of deepSources) {
  exerciseEditsAndComments({ source, },);
  exerciseExistingResets({ source, },);
}

for (const source of corrupted) {
  exerciseInvalidSource({ source, },);
}

for (const entry of invalidCorpus) {
  exerciseInvalidSource({ source: entry.source, },);
}

/**
 Accepted and rejected operation counts after the sweep.
 */
const snapshot = tallySnapshot();

console.log(
  `coverage driver: seed=${String(seed,)} numRuns=${String(numRuns,)} ok=${String(snapshot.ok,)} `
    + `rejected=${String(snapshot.rejected,)} docs=${String(documents.length,)} corrupted=${String(corrupted.length,)} `
    + `validCorpus=${String(validCorpus.length,)} invalidCorpus=${String(invalidCorpus.length,)}`,
);

//endregion Sweep
