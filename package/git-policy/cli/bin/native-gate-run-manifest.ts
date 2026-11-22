/**
 The manifest of one gate run, built from the fields the run knows and written as each container's ID arrives.

 A container's ID exists only while its `--rm` container does, so the run cannot wait for its scope to end to record one:
 `writeManifest` is called once the image exists, again as each client reports its container's ID, and once more when
 the run's scope ends (`bin/test-native-container.ts`). Every ID reaches the file through `gateManifestWriterOf`, which
 writes one at a time, so the suite's and Clippy's IDs never interleave.
 */
import { gateManifestWriterOf, } from './native-gate-manifest-writer.ts';
import type { GateManifest } from './native-gate-manifest.ts';
import type { GateContainerRole } from './native-gate-container-identity.ts';
import type { GateRunContainers } from './native-gate-run-containers.ts';

/**
 What one gate run's manifest is, and the two ways the run changes it.
 */
export type GateRunManifest = {
  /**
   Every field the manifest records as the run stands now, the containers' IDs among them.
   */
  readonly manifestOf: () => GateManifest;
  /**
   Write the manifest, answering once this write is on disk.
   */
  readonly writeManifest: () => Promise<void>;
  /**
   The callback one container's client reports its ID to: it records the ID under that container's role and writes the
   manifest before it answers.
   */
  readonly recorded: (role: GateContainerRole) => (id: string) => Promise<void>;
};

/**
 One gate run's manifest, from the fields the run knows.

 @param fields - every field the manifest records, the run's own containers among them

 @returns the manifest as the run holds it now, its writer, and the callbacks that record each container's ID

 @example
 ```ts
 const evidenceManifest = gateRunManifestOf({ ...fields, run });
 await evidenceManifest.writeManifest();
 ```
 */
export function gateRunManifestOf(fields: {
  readonly evidenceDirectory: string;
  readonly base: string;
  readonly baseTag: string;
  readonly cachyosImage: string;
  readonly cachyosImageId: string;
  readonly image: string;
  readonly fixtureRoot: {
    readonly path: string;
    readonly mount: string;
  };
  readonly limits: readonly string[];
  readonly run: GateRunContainers;
  readonly user: string;
},): GateRunManifest {
  /**
   Everything but the run, which contributes its id, its label and its container records.
   `run` is read on every write, so this holds only the fields that never change.
   */
  const fixed = {
    evidenceDirectory: fields.evidenceDirectory,
    base: fields.base,
    baseTag: fields.baseTag,
    cachyosImage: fields.cachyosImage,
    cachyosImageId: fields.cachyosImageId,
    image: fields.image,
    fixtureRoot: fields.fixtureRoot,
    limits: fields.limits,
    user: fields.user,
  };
  /**
   The run whose containers this manifest records.
   */
  const { run } = fields;
  /**
   This run's manifest as it stands now, which every write builds again so a write asked for after an ID was read
   carries it.

   @returns every field the manifest records at this moment
   */
  function manifestOf(): GateManifest {
    return {
      ...fixed,
      run: {
        id: run.runId,
        label: run.label,
        containers: run.records(),
      },
    };
  }
  /**
   This run's writer, whose writes land one after the other.
   */
  const writeManifest = gateManifestWriterOf({ manifestOf, },);
  return {
    manifestOf,
    writeManifest,
    recorded: function recordedFor(role: GateContainerRole,): (id: string,) => Promise<void> {
      return async function record(id: string,): Promise<void> {
        run.recorded(role,)(id,);
        await writeManifest();
      };
    },
  };
}
