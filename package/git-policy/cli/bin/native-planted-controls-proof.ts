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
  createHash,
  randomUUID,
} from 'node:crypto';
import {
  open,
  readdir,
  rename,
  rm,
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
import {
  type PlantedControl,
  plantedControls,
  type PlantedControlSetting,
} from './native-planted-controls.ts';

/**
 Name of the record inside a campaign's evidence directory.
 */
export const plantedControlsRecordName = 'planted-controls-passed.json';

/**
 Environment variable through which `native:mutation:parallel` tells a campaign unit that another unit runs the planted
 controls of its image beside it: comma-separated `<full image ID>@<unit name>` entries.
 */
export const plantedUnitsEnvironmentVariable = 'NATIVE_MUTATION_PLANTED_UNITS';

/**
 Write the value of {@link plantedUnitsEnvironmentVariable}.

 @param entries - the image each controls unit runs the controls of, and the unit

 @returns the comma-separated entries

 @example
 ```ts
 plantedUnitsText([{ image: 'a'.repeat(64), unit: 'cli-git-mutpar-w5-planted-aaaaaaaaaaaa' }]);
 ```
 */
export function plantedUnitsText(entries: readonly {
  readonly image: string;
  readonly unit: string;
}[]): string {
  return entries.map(function entryText(entry) {
    return `${entry.image}@${entry.unit}`;
  })
    .join(',');
}

/**
 Find the unit that runs the planted controls of an image, in the value of {@link plantedUnitsEnvironmentVariable}.

 @param image - full ID of the image the campaign runs over

 @param text - the variable's value, empty when it is not set

 @returns the unit as the only element, or no element when the text names none for this image

 @example
 ```ts
 plantedUnitFor({ image, text: process.env.NATIVE_MUTATION_PLANTED_UNITS ?? '' });
 ```
 */
export function plantedUnitFor({
  image,
  text,
}: {
  readonly image: string;
  readonly text: string;
}): readonly string[] {
  return text.split(',')
    .flatMap(function unitOfEntry(entry) {
      /**
       Position of the separator between the image ID and the unit name.
       */
      const at = entry.indexOf('@');
      return ((at > 0) && (entry.slice(
        0,
        at,
      ) === image)
        && (entry.length > (at
          + 1))) ? [entry.slice(at + 1)] : [];
    })
    .slice(
    0,
    1,
    );
}

/**
 Digest that identifies the full definition of every control, whatever order the table lists them in.

 @param controls - the controls a run planted

 @returns the hex SHA-256 over the controls sorted by name, each as its name, file, replaced text, new text and the test
 that must fail, so that editing any of them changes the digest

 @example
 ```ts
 plantedControlsDigest(plantedControls);
 ```
 */
export function plantedControlsDigest(controls: readonly PlantedControl[]): string {
  return createHash('sha256')
    .update(JSON.stringify(controls.toSorted(function byName(
      left,
      right
    ) {
      return left.name
        .localeCompare(right.name);
    })
      .map(function definitionOf(control) {
        return [
          control.name,
          control.file,
          control.from,
          control.to,
          control.failing,
        ];
      })))
    .digest('hex');
}

/**
 Build the record written once every planted control was noticed.

 @param image - full ID of the image the controls ran over

 @param limits - the Podman arguments the controls' containers ran with, which carry the memory and process limit

 @param controls - the controls that ran

 @returns the record, which stands alone: it names the image, the limits and the controls

 @example
 ```ts
 plantedControlsPassedRecord({ image, limits: ['--memory=3g', '--pids-limit=512'], controls: plantedControls });
 ```
 */
export function plantedControlsPassedRecord({
  image,
  limits,
  controls,
}: {
  readonly image: string;
  readonly limits: readonly string[];
  readonly controls: readonly PlantedControl[];
}): Record<string, unknown> {
  return {
    plantedControls: true,
    image,
    limits,
    controls: controls.map(function nameOf(control) {
      return control.name;
    }),
    controlsSha256: plantedControlsDigest(controls),
  };
}

/**
 Write a file so that a reader sees all of it or none of it, and a crash leaves no torn record.

 @param file - destination path

 @param text - file text

 @example
 ```ts
 await writeFileAtomically({ file: '/e/native-mutation-abc/planted-controls-passed.json', text: '{}\n' });
 ```
 */
async function writeFileAtomically({
  file,
  text,
}: {
  readonly file: string;
  readonly text: string;
}): Promise<void> {
  /**
   Sibling path that only this writer uses, so a reader looking for the destination name never sees it.
   */
  const temporary = `${file}.${String(process.pid)}.${randomUUID()}.tmp`;
  try {
    /**
     The temporary file, flushed before the rename so a power loss cannot publish an empty destination.
     */
    await using handle = await open(
      temporary,
      'wx',
    );
    await handle.writeFile(
      text,
      'utf8',
    );
    await handle.sync();
  }
  catch (error) {
    await rm(
      temporary,
      { force: true },
    );
    throw error;
  }
  try {
    await rename(
      temporary,
      file,
    );
  }
  catch (error) {
    await rm(
      temporary,
      { force: true },
    );
    throw error;
  }
}

/**
 Write the record that the planted controls passed, in the evidence directory they wrote their logs to.

 @param evidence - the campaign's evidence directory (or its pass-1 directory)

 @param record - the record from {@link plantedControlsPassedRecord}

 @returns the path of the record

 @example
 ```ts
 await writePlantedControlsRecord({ evidence, record });
 ```
 */
export async function writePlantedControlsRecord({
  evidence,
  record,
}: {
  readonly evidence: string;
  readonly record: Readonly<Record<string, unknown>>;
}): Promise<string> {
  /**
   Where the record goes.
   */
  const file = join(
    evidence,
    plantedControlsRecordName,
  );
  await writeFileAtomically({
    file,
    text: `${JSON.stringify(
      record,
      null,
      2,
    )}\n`,
  });
  return file;
}

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
   The names the record lists, sorted; an entry that is not a text stays and makes the lists differ.
   */
  const recorded = record.controls
    .map(String)
    .toSorted(function byText(
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

 @returns true when the record says the controls passed on this image, lists these controls, and records limits the
 campaign has at least; a record of this image that fails the last two checks is named in the log

 @example
 ```ts
 await provesControls({ file, image, limits, controls: plantedControls });
 ```
 */
async function provesControls({
  file,
  image,
  limits,
  controls,
}: {
  readonly file: string;
  readonly image: string;
  readonly limits: ExecutionLimits;
  readonly controls: readonly PlantedControl[];
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
}: {
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly image: string;
  readonly limits: ExecutionLimits;
  readonly controls?: readonly PlantedControl[];
}): Promise<readonly string[]> {
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

/**
 Print one line of the decision to the console.

 @param message - the line

 @example
 ```ts
 printLine('Planted controls skipped.');
 ```
 */
function printLine(message: string): void {
  console.log(message);
}

/**
 Decide whether a campaign runs the planted controls, and record a pass so that later campaigns need not.

 The controls are skipped, in this order, when an earlier attempt of the same campaign already scored mutants, when
 `NATIVE_MUTATION_SKIP_PLANTED=1` is set (a person's claim that another campaign over this snapshot ran them), or when
 a record proves they passed on this image at no more than this campaign's limits, or when a controls unit named by
 `besideUnit` runs them (no record yet, so this campaign's results hold only once that unit ends with one). Otherwise
 they run, and the record is written only after every control was noticed; a failure leaves none.

 @param setting - tested image ID, container bounds, scratch and evidence directories

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @param evidencePrefix - name prefix of the campaigns' directories

 @param scoredMutants - mutants that earlier attempts of this campaign scored

 @param skipRequested - whether the person set `NATIVE_MUTATION_SKIP_PLANTED=1`

 @param besideUnit - the unit that runs these controls beside this campaign, when `native:mutation:parallel` started one
 and no record proves them yet; this campaign then skips them without proof and says which unit holds it

 @param run - runs the controls and throws when one was not noticed

 @param controls - the controls `run` plants; defaults to the table the campaign runs

 @param log - prints one line of the decision; defaults to the console, and tests collect the lines

 @example
 ```ts
 await ensurePlantedControls({ setting, evidenceRoot, evidencePrefix, scoredMutants: 0, skipRequested: false, run: runPlantedControls });
 ```
 */
export async function ensurePlantedControls({
  setting,
  evidenceRoot,
  evidencePrefix,
  scoredMutants,
  skipRequested,
  besideUnit,
  run,
  controls = plantedControls,
  log = printLine,
}: {
  readonly setting: PlantedControlSetting;
  readonly evidenceRoot: string;
  readonly evidencePrefix: string;
  readonly scoredMutants: number;
  readonly skipRequested: boolean;
  readonly besideUnit?: string;
  readonly run: (setting: PlantedControlSetting) => Promise<void>;
  readonly controls?: readonly PlantedControl[];
  readonly log?: (message: string) => void;
}): Promise<void> {
  // A resumed campaign scored mutants only after an earlier attempt passed the controls over this same snapshot.
  if (scoredMutants > 0) {
    log('Planted controls skipped: an earlier attempt of this campaign passed them before scoring.');
    return;
  }
  if (skipRequested) {
    log('Planted controls skipped: another campaign over this snapshot ran them.');
    return;
  }
  /**
   The memory and process limit this campaign's containers run with, as the only element, or none.
   */
  const [limits] = containerLimitsOf(setting.limits);
  if (limits !== undefined) {
    /**
     Path of a record that proves the controls passed on this image, as the only element, or none.
     */
    const [proof] = await passingPlantedControls({
      evidenceRoot,
      evidencePrefix,
      image: setting.base,
      limits,
      controls,
    });
    if (proof !== undefined) {
      log(`Planted controls skipped, because ${proof} proves they already passed on this image.`);
      return;
    }
    if (besideUnit !== undefined) {
      log(`Planted controls skipped here, without proof: unit ${besideUnit} runs them for image ${setting.base} beside this campaign and has not recorded a pass. The results of this campaign hold only once that unit ends with a planted-controls-passed.json for this image (its log ends with the record path, or with the control that was not noticed).`);
      return;
    }
    log('Planted controls: run, because no campaign evidence proves they already passed on this image.');
  }
  await run(setting);
  if (limits === undefined) {
    console.error('The planted controls passed, but their container arguments hold no readable --memory and --pids-limit, so no record was written.');
    return;
  }
  try {
    log(`Planted controls passed; recorded in ${await writePlantedControlsRecord({
      evidence: setting.evidence,
      record: plantedControlsPassedRecord({
        image: setting.base,
        limits: setting.limits,
        controls,
      }),
    })}.`);
  }
  catch (error) {
    // The record only saves a later campaign time; failing to write it must not discard a pass that took minutes.
    console.error(`The planted controls passed, but their record could not be written, so a later campaign runs them again: ${String(error)}`);
  }
}
