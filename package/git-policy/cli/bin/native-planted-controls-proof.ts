/**
 The record a passing run of the planted controls leaves, and the reader that turns it into proof for a later campaign.

 The planted controls (`native-planted-controls.ts`) run the whole suite once per guard removal, which costs about
 24 minutes per campaign. They prove the harness can observe a missing guard in a given image, so a campaign over an
 image whose controls already passed has nothing new to learn from running them again.

 The record is `planted-controls-passed.json` in the evidence directory of the campaign that ran the controls, written
 only after every control was noticed. It names the full image ID, the container arguments the controls ran under and
 the controls themselves (their names, and a digest of their whole definitions). It proves a later campaign only when
 all of these hold: the same image ID, the same controls, and at least the recorded memory and process limit. The image
 ID is content-addressed, so a rebuilt snapshot has another ID and no proof; a control added, removed or edited changes
 the digest and also leaves no proof.

 Nothing here takes a lock. Two campaigns started together over one image, with no record yet, both run the controls
 and each writes its own record; neither waits for the other.
 */
import {
  readdir,
} from 'node:fs/promises';
import { join } from 'node:path';

import { passOneDirectoryName } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/campaign-passes.ts';
import { isMissing } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fs-guards.ts';

import {
  containerLimitsOf,
  type ExecutionLimits,
  limitsAtLeast,
  readGateRecord,
} from './native-gate-proof.ts';
import { plantedHarnessVersion } from './native-planted-controls-harness.ts';
import {
  plantedControlsDigest,
  plantedControlsRecordName,
} from './native-planted-controls-record.ts';
import {
  type PlantedControl,
  plantedControls,
} from './native-planted-controls.ts';

/**
 Check that a record is of the same controls as the ones about to run.

 @param record - the parsed record

 @param controls - the controls a campaign would plant

 @returns true when the record lists exactly these control names and their digest equals the digest of these definitions

 @example
 ```ts
 recordIsOfControls({ record, controls: plantedControls });
 ```
 */
function recordIsOfControls({
  record,
  controls,
}: {
  readonly record: Readonly<Record<string, unknown>>;
  readonly controls: readonly PlantedControl[];
}): boolean {
  if (record.controlsSha256 !== plantedControlsDigest(controls))
    return false;
  if (!Array.isArray(record.controls))
    return false;
  /**
   The entries of the record's list that are texts; one that is not makes the record unusable, and nothing is coerced.
   */
  const texts = record.controls
    .filter(function isText(entry: unknown): entry is string {
      return (typeof entry) === 'string';
    });
  if (texts.length
    !== record.controls
      .length)
    return false;
  /**
   The names the record lists, sorted.
   */
  const recorded = texts.toSorted(function byText(
    left,
    right
  ) {
    return left.localeCompare(right);
  });
  /**
   The names of the controls about to run, sorted.
   */
  const wanted = controls.map(function nameOf(control) {
    return control.name;
  })
    .toSorted(function byText(
      left,
      right
    ) {
      return left.localeCompare(right);
    });
  return JSON.stringify(recorded) === JSON.stringify(wanted);
}

/**
 Check one record file for proof that the planted controls passed on an image.

 @param file - path of a `planted-controls-passed.json`

 @param image - full ID of the image a campaign uses

 @param limits - the memory and process limits of the campaign

 @param controls - the controls the campaign would plant

 @param harness - version of the host-side runner and judge this campaign would use

 @returns true when the record says the controls passed on this image, lists these controls, was made by this runner and
 judge, and records limits the campaign has at least; a record of this image that fails one of these checks is named in
 the log

 @example
 ```ts
 await provesControls({ file, image, limits, controls: plantedControls, harness });
 ```
 */
async function provesControls({
  file,
  image,
  limits,
  controls,
  harness,
}: {
  readonly file: string;
  readonly image: string;
  readonly limits: ExecutionLimits;
  readonly controls: readonly PlantedControl[];
  readonly harness: number;
}): Promise<boolean> {
  /**
   The parsed record as the only element, or none when the file is absent, torn or not an object.
   */
  const [record] = await readGateRecord(file);
  if ((record === undefined) || (record.plantedControls !== true)
    || (record.image !== image))
    return false;
  if (!recordIsOfControls({
    record,
    controls,
  })) {
    console.log(`Planted-controls record ${file} passed on this image, but for another set of controls than the ones this campaign has, so the controls run.`);
    return false;
  }
  if (record.harnessVersion !== harness) {
    console.log(`Planted-controls record ${file} passed on this image, but another version of the code that runs and judges the controls made it (or it records none), so the controls run.`);
    return false;
  }
  /**
   The limits the controls ran under, as the only element, or none when the record does not hold readable ones.
   */
  const [recorded] = containerLimitsOf(record.limits);
  if (recorded === undefined) {
    console.log(`Planted-controls record ${file} passed on this image but does not record its memory and process limits, so it is not used as proof.`);
    return false;
  }
  if (!limitsAtLeast({
    campaign: limits,
    recorded,
  })) {
    console.log(`Planted-controls record ${file} passed on this image at --memory ${recorded.memory} --pids-limit ${String(recorded.pidsLimit)}, but this campaign runs at --memory ${limits.memory} --pids-limit ${String(limits.pidsLimit)}; with less than the controls had, a failure may be the limit and not the removed guard, so the controls run.`);
    return false;
  }
  return true;
}

/**
 List the campaigns' evidence directories.

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @param evidencePrefix - name prefix of the campaigns' directories

 @returns the names in name order, or none when the directory does not exist

 @example
 ```ts
 await campaignDirectoryNames({ evidenceRoot: 'target/verification', evidencePrefix: 'native-mutation-' });
 ```
 */
async function campaignDirectoryNames({
  evidenceRoot,
  evidencePrefix,
}: {
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
}): Promise<readonly string[]> {
  try {
    return (await readdir(evidenceRoot)).filter(function isCampaign(name) {
      return name.startsWith(evidencePrefix);
    })
      .toSorted();
  }
  catch (error) {
    if (!isMissing(error))
      throw error;
    return [];
  }
}

/**
 Find a campaign whose planted controls passed on an image.

 A two-pass campaign runs the controls before its first pass, so its record sits in its pass-1 directory; that
 directory is looked at as well.

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run (`target/verification`)

 @param evidencePrefix - name prefix of the campaigns' directories (`native-mutation-`)

 @param image - full ID of the image the campaign resolved

 @param limits - the memory and process limits of the campaign, which must be at least the recorded ones

 @param controls - the controls the campaign would plant; defaults to the table the campaign runs

 @param harness - version of the runner and judge the campaign would use; defaults to the loaded one

 @returns the path of the first (in name order) record that proves the pass as the only element, or no element when
 there is none and the campaign must run the controls

 @example
 ```ts
 await passingPlantedControls({ evidenceRoot, evidencePrefix: 'native-mutation-', image, limits: { memory: '2g', pidsLimit: 512 } });
 ```
 */
export async function passingPlantedControls({
  evidenceRoot,
  evidencePrefix,
  image,
  limits,
  controls = plantedControls,
  harness,
}: {
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly image: string;
  readonly limits: ExecutionLimits;
  readonly controls?: readonly PlantedControl[];
  readonly harness?: number;
}): Promise<readonly string[]> {
  /**
   Version of the runner and judge this campaign would use.
   */
  const harnessVersion = harness ?? plantedHarnessVersion;
  /**
   Every place a record can be, in name order, the campaign's own directory before its pass-1 directory.
   */
  const files = (await campaignDirectoryNames({
    evidenceRoot,
    evidencePrefix,
  })).flatMap(function recordPaths(name) {
    return [
      join(
        evidenceRoot,
        name,
        plantedControlsRecordName,
      ),
      join(
        evidenceRoot,
        name,
        passOneDirectoryName,
        plantedControlsRecordName,
      ),
    ];
  });
  /**
   Whether each place holds a record that proves the image.
   */
  const proofs = await Promise.all(files.map(function provesFile(file) {
    return provesControls({
      file,
      image,
      limits,
      controls,
      harness: harnessVersion,
    });
  }));
  /**
   Path of the first record that proves it.
   */
  const proving = files.find(function isProof(
    _file,
    index,
  ) {
    return proofs[index] === true;
  });
  return proving === undefined ? [] : [proving];
}
