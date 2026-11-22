/**
 The two containers one gate run starts: the run id both carry in their names and labels, the ID file each client writes
 its ID to, and the records the run's manifest keeps.

 The run id is the evidence directory's random name (`bin/native-gate-container-identity.ts`), so two gates of this
 package, which several sessions run at once, never name a container the same way and a session stops only containers
 carrying its own label (issue 6334). A container's ID exists only while its `--rm` container does, so the run records
 each ID as its own `podman run` reads it, and the manifest keeps those.
 */
import { join, } from 'node:path';

import {
  type GateContainerIdentity,
  type GateContainerRole,
  gateContainerIdentityOf,
  gateRunIdOf,
} from './native-gate-container-identity.ts';

/**
 One container of a gate run, and the file its client writes the container's ID to.
 */
export type GateRunContainer = {
  /**
   The role the container runs, and the name and label that say so.
   */
  readonly identity: GateContainerIdentity;
  /**
   The evidence file the client writes the container's ID to, which `--rm` takes away with the container.
   */
  readonly cidFile: string;
};

/**
 What one gate run's containers are, and how the run reads their IDs back for its manifest.
 */
export type GateRunContainers = {
  /**
   The run id every container of this run carries.
   */
  readonly runId: string;
  /**
   The label every container of this run carries, in the `key=value` form `podman ps --filter` matches.
   */
  readonly label: string;
  /**
   The container the whole suite runs in.
   */
  readonly suite: GateRunContainer;
  /**
   The container Clippy runs in.
   */
  readonly clippy: GateRunContainer;
  /**
   The callback one container's client reports its ID to, which its role's record then carries, given the check whose
   container was handed an ID and answering with the callback that container's run takes.
   */
  readonly recorded: (role: GateContainerRole) => (id: string) => void;
  /**
   Every container of this run, as the manifest records it: role, name, label, ID file, and the ID when one was read.
   */
  readonly records: () => readonly Readonly<Record<string, string>>[];
};

/**
 The containers of one gate run, named from its evidence directory.

 @param evidenceDirectory - the directory `test-native-container.ts` made for this run, whose random name is the run id

 @returns the run id, both container identities with their ID files, and the way the run records and reads their IDs

 @example
 ```ts
 const run = gateRunContainersOf({ evidenceDirectory: 'target/verification/native-HZPg8O' });
 ```
 */
export function gateRunContainersOf({ evidenceDirectory }: { readonly evidenceDirectory: string; },): GateRunContainers {
  /**
   The run id every container of this run carries.
   */
  const runId = gateRunIdOf({ evidenceDirectory, },);
  /**
   The ID podman gave each container, by role, as this run read it while the container existed.
   */
  const ids = new Map<GateContainerRole, string>();
  /**
   One container's identity beside the ID file its client writes.

   @param role - the check this container runs

   @returns the container's identity and the evidence file its client writes the ID to
   */
  function containerOf(role: GateContainerRole,): GateRunContainer {
    return {
      identity: gateContainerIdentityOf({
        runId,
        role,
      },),
      cidFile: join(
        evidenceDirectory,
        `${role}.cid`,
      ),
    };
  }
  /**
   The suite's container.
   */
  const suite = containerOf('suite',);
  /**
   Clippy's container.
   */
  const clippy = containerOf('clippy',);
  /**
   One container as the manifest records it, with its ID only when this run read one.

   @param container - the container to record

   @returns its role, name, label and ID file, and its ID when the run read one
   */
  function recordOf(container: GateRunContainer,): Readonly<Record<string, string>> {
    /**
     The container's identity, whose role its ID was recorded under.
     */
    const { identity, } = container;
    /**
     The ID this run read for it, absent when no ID was read.
     */
    const id = ids.get(identity.role,);
    return {
      role: identity.role,
      name: identity.name,
      label: identity.label,
      cidFile: container
        .cidFile,
      ...(id === undefined ? {} : { id, }),
    };
  }
  /**
   The label every container of this run carries, which both identities agree on.
   */
  const { label, } = suite.identity;
  return {
    runId,
    label,
    suite,
    clippy,
    recorded: function recordedFor(role: GateContainerRole,) {
      return function record(id: string,): void {
        ids.set(
          role,
          id,
        );
      };
    },
    records: function records() {
      return [
        recordOf(suite,),
        recordOf(clippy,),
      ];
    },
  };
}
