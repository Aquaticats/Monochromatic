/**
 Differential property tests:
 the fork must produce the same observable store behavior as upstream
 the vendored upstream `conf` snapshot (`upstream-conf/`, commit `83e267178f`) on the same generated operation sequences.
 
 The oracle is upstream itself:
 per-operation results,
 thrown error diagnostics,
 final store contents,
 item counts,
 and config-file `JSON` are checked against the implementation this fork was
 derived from.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  assert,
  property,
} from 'fast-check';

import { fuzzRuns, } from './fuzz-budget.ts';
import {
  type StoreOperationSequence,
  sequenceArb,
} from './store-operations.ts';
import {
  type OperationOutcome,
  type StoreObservation,
  runOnFork,
  runOnUpstream,
} from './store-adapters.ts';

//region Helpers

/**
 Settled error-class deviation:
 the fork throws typed error classes where upstream throws bare
 `TypeError` or `Error`,
 each carrying upstream's message text verbatim
 (see `package/upstream-fork/conf/README.md`,
 "Error types").
 This map normalizes fork error names back to the class name upstream
 reports,
 so only message texts and base classes must match exactly.
 */
const SETTLED_ERROR_NAMES: Readonly<Record<string, string>> = {
  InvalidKeyError: 'TypeError',
  MissingValueError: 'TypeError',
  ReservedKeyError: 'TypeError',
  UnsupportedValueTypeError: 'TypeError',
  NonArrayValueError: 'TypeError',
  InvalidCallbackError: 'TypeError',
  InvalidEncryptionAlgorithmError: 'TypeError',
  InvalidSchemaError: 'TypeError',
  RootSchemaPropertiesError: 'TypeError',
  SchemaViolationError: 'Error',
  MissingProjectNameError: 'Error',
  MissingProjectVersionError: 'Error',
  DecryptionFailedError: 'Error',
  InvalidAuthenticationTagError: 'Error',
  MigrationFailedError: 'Error',
};

/**
 Normalizes one outcome for cross-implementation comparison:
 thrown error names pass through the settled error-class map,
 everything else is compared exactly.
 
 @param outcome - Outcome recorded from one store under comparison.
 
 @returns Comparable projection of the outcome.
 
 @example
 ```ts
 normalizeOutcome({ kind: 'threw', errorName: 'NonArrayValueError', ... });
 ```
 */
function normalizeOutcome(outcome: OperationOutcome,): OperationOutcome {
  if (outcome.kind !== 'threw')
    return outcome;

  return {
    kind: 'threw',
    errorName: SETTLED_ERROR_NAMES[outcome.errorName]
      ?? outcome.errorName,
    errorBase: outcome.errorBase,
    errorMessage: outcome.errorMessage,
  };
}

/**
 Projects one observation into a directly comparable record.
 
 @param observation - Observation from one workload run.
 
 @returns Comparable projection of the observation.
 
 @example
 ```ts
 expect(comparable(forkObservation,)).toEqual(comparable(upstreamObservation,),);
 ```
 */
function comparable(observation: StoreObservation,): Record<string, unknown> {
  return {
    outcomes: observation.outcomes.map(normalizeOutcome,),
    finalStore: observation.finalStore,
    size: observation.size,
    fileContents: observation.fileContents,
  };
}

//endregion Helpers

await describe({
  name: 'upstream conf parity',
  children: [
    it({
      name: 'store operation sequences produce identical observations in the vendored upstream snapshot and the fork',
      fn: async () => {
        assert(
          property(sequenceArb, (sequence: StoreOperationSequence,) => {
            expect(
              comparable(runOnFork(sequence,)),
            ).toEqual(
              comparable(runOnUpstream(sequence,)),
            );
          },),
          {
            numRuns: fuzzRuns,
          },
        );
      },
    },),
  ],
},);
