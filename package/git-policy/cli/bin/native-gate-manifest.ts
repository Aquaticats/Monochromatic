/**
 The manifest one gate run leaves in its evidence directory: the image it tested, the limits its containers ran under,
 and the run's own id with every container it started.

 The runner writes it once the image exists and before its containers start, again as each container's ID is read while
 that container runs, and once they ended, because a container's ID exists only while its `--rm` container does. The new
 `run` field is what lets a person prove which container of a machine running several gates is theirs (issue 6334).
 */
import {
  rename,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Everything one gate's manifest records, as the run observed it.
 */
export type GateManifest = {
  /**
   The directory the manifest is written into, which is the evidence directory itself and not a field of the file.
   */
  readonly evidenceDirectory: string;
  /**
   Content-addressed ID of the base image the gate image was built from.
   */
  readonly base: string;
  /**
   Tag the derived base image carries.
   */
  readonly baseTag: string;
  /**
   CachyOS image the derived base was built on, and the ID the host holds for it.
   */
  readonly cachyosImage: string;
  readonly cachyosImageId: string;
  /**
   Content-addressed ID of the image whose suite and Clippy ran.
   */
  readonly image: string;
  /**
   The fixture root the suite's containers mount, and the tmpfs options they mount it with.
   */
  readonly fixtureRoot: {
    readonly path: string;
    readonly mount: string;
  };
  /**
   Bounds of every container: removal on exit, no network, and the memory, CPUs and processes of the container limits.
   */
  readonly limits: readonly string[];
  /**
   The run's id, the label its containers carry, and one record per container.
   */
  readonly run: {
    readonly id: string;
    readonly label: string;
    readonly containers: readonly Readonly<Record<string, string>>[];
  };
  /**
   The user and group the containers run as.
   */
  readonly user: string;
};

/**
 Write one gate's manifest into its evidence directory.

 @param manifest - every field the manifest records, the evidence directory among them

 One writer owns one run's manifest (`gateManifestWriterOf`), so two writes never race for the file this one moves into
 place.

 @example
 ```ts
 await writeGateManifest({ ...fields, evidenceDirectory: 'target/verification/native-HZPg8O' });
 ```
 */
export async function writeGateManifest(manifest: GateManifest,): Promise<void> {
  /**
   Every field but the directory, which is where the file goes.
   */
  const {
    evidenceDirectory,
    ...recorded
  } = manifest;
  /**
   The file this write produces before it is moved into place, so a reader sees the manifest as it stood before or as it
   stands after this write and never a half-written one.
   */
  const partial = join(
    evidenceDirectory,
    'manifest.json.partial',
  );
  await writeFile(
    partial,
    `${JSON.stringify(
      recorded,
      null,
      2,
    )}\n`,
  );
  await rename(
    partial,
    join(
      evidenceDirectory,
      'manifest.json',
    ),
  );
}
