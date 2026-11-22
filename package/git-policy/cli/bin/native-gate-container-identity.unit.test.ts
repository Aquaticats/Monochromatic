import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  gateContainerArgumentsOf,
  gateContainerIdLineOf,
  gateContainerIdentityOf,
  gateContainerLineOf,
  gateContainerNoIdLineOf,
  gateContainerNotCreatedLineOf,
  gateRunIdOf,
  gateRunLabelName,
  gateRunLineOf,
} from './native-gate-container-identity.ts';

/**
 A run id in the shape the evidence directory gives one: the runner's prefix and six random characters.
 */
const runId = 'native-HZPg8O';

/**
 The suite's identity in every check that needs one.
 */
const suite = gateContainerIdentityOf({
  runId,
  role: 'suite',
});

await describe({
  name: 'the identity one gate container carries',
  children: [
    it({
      name: 'takes the run id from the evidence directory\'s random name, never from a clock',
      fn: async () => {
        expect(gateRunIdOf({ evidenceDirectory: `/repo/target/verification/${runId}`, }),).toBe(runId,);
        expect(gateRunIdOf({ evidenceDirectory: `/repo/target/verification/${runId}/`, }),).toBe(runId,);
      },
    }),
    it({
      name: 'names a container after the run and the check it runs, and labels it with the run alone',
      fn: async () => {
        expect(suite,).toEqual({
          role: 'suite',
          name: `gpc-gate-${runId}-suite`,
          label: `${gateRunLabelName}=${runId}`,
        },);
        expect(gateContainerIdentityOf({
          runId,
          role: 'clippy',
        },),).toEqual({
          role: 'clippy',
          name: `gpc-gate-${runId}-clippy`,
          label: `${gateRunLabelName}=${runId}`,
        },);
      },
    }),
    it({
      name: 'gives the two checks two names and one label, so a name stops one container and a filter lists the run',
      fn: async () => {
        /**
         The other check's identity.
         */
        const clippy = gateContainerIdentityOf({
          runId,
          role: 'clippy',
        },);
        expect(suite.name,).not.toBe(clippy.name,);
        expect(suite.label,).toBe(clippy.label,);
      },
    }),
    it({
      name: 'writes the name and label as the two podman options that carry them',
      fn: async () => {
        expect(gateContainerArgumentsOf({ identity: suite, },),).toEqual([
          '--name',
          `gpc-gate-${runId}-suite`,
          '--label',
          `${gateRunLabelName}=${runId}`,
        ],);
      },
    }),
    it({
      name: 'prints the run id with the filter that lists only its containers, and the name and label before the container starts',
      fn: async () => {
        /**
         The line a gate prints once, before any container.
         */
        const runLine = gateRunLineOf({ runId, },);
        expect(runLine,).toContain(`Gate run id ${runId}`,);
        expect(runLine,).toContain(`podman ps --filter label=${gateRunLabelName}=${runId}`,);
        /**
         The line a gate prints before one container's client starts.
         */
        const containerLine = gateContainerLineOf({
          identity: suite,
          cidFile: 'target/verification/native-HZPg8O/suite.cid',
        },);
        expect(containerLine,).toContain(`Gate container suite: name gpc-gate-${runId}-suite`,);
        expect(containerLine,).toContain(`label ${gateRunLabelName}=${runId}`,);
        expect(containerLine,).toContain('ID file target/verification/native-HZPg8O/suite.cid',);
      },
    }),
    it({
      name: 'prints the container\'s ID beside the name `podman stop` takes',
      fn: async () => {
        /**
         An ID in the shape podman writes to the ID file.
         */
        const id = 'a'.repeat(64);
        expect(gateContainerIdLineOf({
          identity: suite,
          id,
        },),).toBe(`Gate container suite: ID ${id} (name gpc-gate-${runId}-suite, label ${gateRunLabelName}=${runId})`,);
      },
    }),
    it({
      name: 'says a client that created no container left only podman\'s own failure to read',
      fn: async () => {
        expect(gateContainerNotCreatedLineOf({
          identity: suite,
          cidFile: 'target/verification/native-HZPg8O/suite.cid',
        },),).toContain('created no container',);
      },
    }),
    it({
      name: 'says which name and label remain when no ID arrived in the time the gate waited',
      fn: async () => {
        /**
         The line a gate prints when its wait for an ID used its whole bound.
         */
        const line = gateContainerNoIdLineOf({
          identity: suite,
          cidFile: 'target/verification/native-HZPg8O/suite.cid',
          waitedMilliseconds: 120_000,
        },);
        expect(line,).toContain('no ID reached',);
        expect(line,).toContain(`name gpc-gate-${runId}-suite`,);
        expect(line,).toContain(`label ${gateRunLabelName}=${runId}`,);
        expect(line,).toContain('120000 ms',);
      },
    }),
  ],
},);
