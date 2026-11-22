import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  gateRunLabelName,
} from './native-gate-container-identity.ts';
import { gateRunContainersOf, } from './native-gate-run-containers.ts';

/**
 The evidence directory of one run, whose last segment is the run id every one of its containers carries.
 */
const evidenceDirectory = 'target/verification/native-HZPg8O';

/**
 The run id that directory names.
 */
const runId = 'native-HZPg8O';

/**
 One fresh run, so a check that reports an ID never changes what another check reads.
 */
function runOf() {
  return gateRunContainersOf({ evidenceDirectory, },);
}

await describe({
  name: 'the containers one gate run starts',
  children: [
    it({
      name: 'names both checks after the run id, each with an ID file of its own under the evidence directory',
      fn: async () => {
        /**
         The run these assertions read.
         */
        const run = runOf();
        expect(run.runId,).toBe(runId,);
        expect(run.suite.identity.name,).toBe(`gpc-gate-${runId}-suite`,);
        expect(run.clippy.identity.name,).toBe(`gpc-gate-${runId}-clippy`,);
        expect(run.suite.cidFile,).toBe(`target/verification/${runId}/suite.cid`,);
        expect(run.clippy.cidFile,).toBe(`target/verification/${runId}/clippy.cid`,);
      },
    }),
    it({
      name: 'gives both containers one label, which is the filter that lists this run alone',
      fn: async () => {
        /**
         The run these assertions read.
         */
        const run = runOf();
        expect(run.label,).toBe(`${gateRunLabelName}=${runId}`,);
        expect(run.suite.identity.label,).toBe(run.label,);
        expect(run.clippy.identity.label,).toBe(run.label,);
      },
    }),
    it({
      name: 'records a container without an ID until its client reports one',
      fn: async () => {
        /**
         The run whose records are read before either client reported an ID.
         */
        const run = runOf();
        expect(run.records(),).toEqual([
          {
            role: 'suite',
            name: `gpc-gate-${runId}-suite`,
            label: `${gateRunLabelName}=${runId}`,
            cidFile: `target/verification/${runId}/suite.cid`,
          },
          {
            role: 'clippy',
            name: `gpc-gate-${runId}-clippy`,
            label: `${gateRunLabelName}=${runId}`,
            cidFile: `target/verification/${runId}/clippy.cid`,
          },
        ],);
      },
    }),
    it({
      name: 'keeps each reported ID beside the container whose client reported it',
      fn: async () => {
        /**
         The run whose Clippy container reports an ID.
         */
        const run = runOf();
        /**
         The ID the Clippy container's client reported, which only its own record may carry.
         */
        const clippyId = 'b'.repeat(64);
        run.recorded('clippy',)(clippyId,);
        /**
         The records as the manifest then holds them.
         */
        const [suite, clippy] = run.records();
        expect(clippy?.id,).toBe(clippyId,);
        expect(suite?.id,).toBeUndefined();
      },
    }),
  ],
},);
