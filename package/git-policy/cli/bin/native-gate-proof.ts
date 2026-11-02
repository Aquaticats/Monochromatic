/**
 The record a passing native gate leaves, and the reader that turns it into proof for a mutation campaign.

 The gate (`test-native-container.ts`) writes `manifest.json` with the full ID of the image it tests before it runs
 anything, and `passed.json` only after the test run and Clippy both exited zero. A campaign layers cargo-mutants over
 that same image ID, so a gate directory with both files for that ID is proof that the whole suite passed on the image
 the campaign's own baseline would run. The ID is content-addressed: a rebuilt snapshot has another ID and no proof.
 */
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  isMissing,
  isRecord,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fs-guards.ts';

/**
 Name prefix of the gate's evidence directories (`mkdtemp` adds six random characters).
 */
const gatePrefix = 'native-';

/**
 Name prefix of the mutation campaigns' evidence directories, which start with the gate prefix but are not gate runs.
 */
const campaignPrefix = 'native-mutation-';

/**
 Build the record the gate writes as `passed.json` once the tests and Clippy passed.

 @param image - full ID of the image the gate tested

 @returns the record, which names the image so the file proves the pass without the manifest

 @example
 ```ts
 gatePassedRecord('a'.repeat(64));
 ```
 */
export function gatePassedRecord(image: string): Record<string, unknown> {
  return {
    tests: true,
    clippy: true,
    image,
  };
}

/**
 Read a JSON object file of a gate directory.

 @param file - path of the file

 @returns the object, or nothing when the file is absent, torn or not an object

 @example
 ```ts
 await readGateRecord('/e/native-abc123/passed.json');
 ```
 */
async function readGateRecord(file: string): Promise<Readonly<Record<string, unknown>> | undefined> {
  try {
    /**
     The parsed file before it is known to be an object.
     */
    const parsed: unknown = JSON.parse(await readFile(
      file,
      'utf8',
    ));
    return isRecord(parsed) ? parsed : undefined;
  }
  catch (error) {
    // A gate stopped before it passed has no passed record; a torn file means a kill landed mid-write. Either is no proof.
    if (!isMissing(error))
      console.error(`Gate record ${file} is not usable as proof: ${String(error)}`);
    return undefined;
  }
}

/**
 Check one gate directory for proof that the gate passed on an image.

 @param directory - a gate evidence directory

 @param image - full ID of the image a campaign uses

 @returns true when the manifest names the image and the passed record says the tests and Clippy passed (and, when it
 names an image itself, the same one)

 @example
 ```ts
 await provesImage({ directory: '/e/native-abc123', image });
 ```
 */
async function provesImage({
  directory,
  image,
}: {
  readonly directory: string;
  readonly image: string;
}): Promise<boolean> {
  const [manifest, passed] = await Promise.all([
    readGateRecord(join(
      directory,
      'manifest.json',
    )),
    readGateRecord(join(
      directory,
      'passed.json',
    )),
  ]);
  if ((manifest === undefined) || (passed === undefined))
    return false;
  if ((passed.tests !== true) || (passed.clippy !== true))
    return false;
  return (manifest.image === image) && ((!('image' in passed)) || (passed.image === image));
}

/**
 Find a gate run that passed on an image.

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run (`target/verification`)

 @param image - full ID of the image the campaign resolved from the gate's tag

 @returns the path of the first (in name order) gate directory that proves the pass, or nothing when there is none and
 the campaign must run its own baseline

 @example
 ```ts
 await passingGateEvidence({ evidenceRoot: 'target/verification', image });
 ```
 */
export async function passingGateEvidence({
  evidenceRoot,
  image,
}: {
  readonly evidenceRoot: string;
  readonly image: string;
}): Promise<string | undefined> {
  /**
   Names of the gate directories, in name order.
   */
  let names: readonly string[];
  try {
    names = (await readdir(evidenceRoot)).filter(function isGateRun(name) {
      return name.startsWith(gatePrefix) && (!name.startsWith(campaignPrefix));
    })
      .toSorted();
  }
  catch (error) {
    if (!isMissing(error))
      throw error;
    return undefined;
  }
  /**
   Whether each gate directory proves the image, in name order.
   */
  const proofs = await Promise.all(names.map(function provesName(name) {
    return provesImage({
      directory: join(
        evidenceRoot,
        name,
      ),
      image,
    });
  }));
  /**
   Name of the first directory that proves it.
   */
  const proving = names.find(function isProof(
    _name,
    index
  ) {
    return proofs[index] === true;
  });
  return proving === undefined ? undefined : join(
    evidenceRoot,
    proving,
  );
}
