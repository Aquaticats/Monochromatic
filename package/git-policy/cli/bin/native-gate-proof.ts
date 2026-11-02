/**
 The record a passing native gate leaves, and the reader that turns it into proof for a mutation campaign.

 The gate (`test-native-container.ts`) writes `manifest.json` with the full ID of the image it tests before it runs
 anything, and `passed.json` only after the test run and Clippy both exited zero. A campaign layers cargo-mutants over
 that same image ID, so a gate directory with both files for that ID is proof that the whole suite passed on the image
 the campaign's own baseline would run. The ID is content-addressed: a rebuilt snapshot has another ID and no proof.

 The proof also depends on the limits the gate ran under. The baseline of 2026-11-01 failed under 128 processes where the
 gate's tests pass under more, so a campaign with less memory or fewer processes than the gate does not get the proof:
 its infrastructure failures would be counted as caught mutants.
 */
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { memoryBytes } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/container-limits.ts';
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
export const campaignPrefix = 'native-mutation-';

/**
 The memory and process limits of a container.
 */
export type ExecutionLimits = {
  /**
   Podman memory size, such as `2g`.
   */
  readonly memory: string;
  readonly pidsLimit: number;
};

/**
 Whether a single-dash argument is a cluster of short flags that holds a given short flag after its first letter, such as
 `-itm`, whose value Podman would read from the next argument and this reader cannot place.

 @param text - one Podman argument

 @param name - the short option, such as `-m`

 @returns true for a cluster of letters that has the option's letter anywhere but first

 @example
 ```ts
 isHiddenInCluster({ text: '-itm', name: '-m' });
 ```
 */
function isHiddenInCluster({
  text,
  name,
}: {
  readonly text: string;
  readonly name: string;
}): boolean {
  if ((name.length !== 2) || (!text.startsWith('-'))
    || text.startsWith('--')
    || text.startsWith(name))
    return false;
  return text.slice(1)
    .split('')
    .every(function isLetter(character) {
      return character.toLowerCase() !== character.toUpperCase();
    })
    && text.includes(name.slice(1));
}

/**
 Find the value Podman uses for an option that was given more than once, split or attached.

 Podman reads `--option=value`, `--option value` and, for a short option, `-ovalue`, and the last occurrence wins.
 An occurrence this reader cannot take a value from (the option ends the list, or hides in a cluster of short flags)
 makes the answer none instead of falling back to an earlier occurrence.

 @param texts - the Podman arguments

 @param names - the option's spellings, with their leading hyphens

 @returns the value of the last occurrence as the only element, or no element when the option is absent or its last
 occurrence has no readable value

 @example
 ```ts
 effectiveOptionValue({ texts: ['--memory=1g', '-m4g'], names: ['--memory', '-m'] });
 ```
 */
function effectiveOptionValue({
  texts,
  names,
}: {
  readonly texts: readonly string[];
  readonly names: readonly string[];
}): readonly string[] {
  /**
   The value of every occurrence in command-line order, each as a list that is empty when the value is unreadable.
   */
  const occurrences = texts.flatMap(function occurrencesAt(
    text,
    index
  ) {
    return names.flatMap(function occurrencesOfName(name): readonly (readonly string[])[] {
      if (text.startsWith(`${name}=`))
        return [[text.slice(name.length + 1)]];
      if (text === name)
        return [texts.slice(
          index + 1,
          index + 2,
        )];
      if (isHiddenInCluster({
        text,
        name,
      }))
        return [[]];
      // Only a short option may carry its value attached, as in `-m4g`.
      return ((!name.startsWith('--')) && text.startsWith(name))
        ? [[text.slice(name.length)]]
        : [];
    });
  });
  return occurrences.at(-1) ?? [];
}

/**
 Read the memory and process limits a record keeps among the container arguments it ran under.

 @param limits - the record's `limits` field, the Podman arguments the gate (or the planted controls) ran with

 @returns the limits as the only element, or no element when the field is not a list or does not hold a readable
 finite `--memory` and `--pids-limit`; of repeated, split or attached options the last wins, as in Podman; an option whose last occurrence has no
 readable value gives none

 @example
 ```ts
 containerLimitsOf(['--rm', '--memory=2g', '--pids-limit=512']);
 ```
 */
export function containerLimitsOf(limits: unknown): readonly ExecutionLimits[] {
  if (!Array.isArray(limits))
    return [];
  /**
   The arguments that are texts.
   */
  const texts = limits.filter(function isText(argument: unknown): argument is string {
    return (typeof argument) === 'string';
  });
  /**
   The memory limit, as the only element, or none.
   */
  const [memory] = effectiveOptionValue({
    texts,
    names: [
      '--memory',
      '-m',
    ],
  });
  /**
   The process limit, as the only element, or none.
   */
  const [pids] = effectiveOptionValue({
    texts,
    names: ['--pids-limit'],
  });
  if ((memory === undefined) || (pids === undefined)
    || (!Number.isInteger(Number(pids))))
    return [];
  // Podman documents -1 as unlimited processes and says nothing certain about 0; neither is a bound a campaign can be
  // compared against, so a record made under one is not proof.
  if (Number(pids) < 1) {
    console.error(`The recorded process limit ${pids} is not a finite bound, so it cannot be compared.`);
    return [];
  }
  try {
    if (memoryBytes(memory) < 1)
      throw new Error('a memory limit of 0 means unlimited');
  }
  catch (error) {
    console.error(`The recorded memory limit ${memory} cannot be compared: ${String(error)}`);
    return [];
  }
  return [{
    memory,
    pidsLimit: Number(pids),
  }];
}

/**
 Compare a campaign's memory and process limits with the ones a record was made under.

 @param campaign - the limits of the campaign that asks for proof

 @param recorded - the limits the proving run had

 @returns true when the campaign has at least the recorded memory and at least the recorded process limit; with less, a
 failure may be the limit and not the thing under test

 @example
 ```ts
 limitsAtLeast({ campaign: { memory: '3g', pidsLimit: 512 }, recorded: { memory: '2g', pidsLimit: 512 } });
 ```
 */
export function limitsAtLeast({
  campaign,
  recorded,
}: {
  readonly campaign: ExecutionLimits;
  readonly recorded: ExecutionLimits;
}): boolean {
  return (memoryBytes(campaign.memory) >= memoryBytes(recorded.memory))
    && (campaign.pidsLimit >= recorded.pidsLimit);
}

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
 Read a JSON object file of a gate or campaign evidence directory.

 @param file - path of the file

 @returns the object as the only element, or no element when the file is absent, torn or not an object

 @example
 ```ts
 await readGateRecord('/e/native-abc123/passed.json');
 ```
 */
export async function readGateRecord(file: string): Promise<readonly Readonly<Record<string, unknown>>[]> {
  try {
    /**
     The parsed file before it is known to be an object.
     */
    const parsed: unknown = JSON.parse(await readFile(
      file,
      'utf8',
    ));
    return isRecord(parsed) ? [parsed] : [];
  }
  catch (error) {
    // A gate stopped before it passed has no passed record; a torn file means a kill landed mid-write. Either is no proof.
    if (!isMissing(error))
      console.error(`Evidence record ${file} is not usable as proof: ${String(error)}`);
    return [];
  }
}

/**
 Check one gate directory for proof that the gate passed on an image.

 @param directory - a gate evidence directory

 @param image - full ID of the image a campaign uses

 @param limits - the memory and process limits of the campaign

 @returns true when the manifest names the image, the passed record says the tests and Clippy passed (and, when it names
 an image itself, the same one), and the campaign has at least the gate's memory and process limit

 @example
 ```ts
 await provesImage({ directory: '/e/native-abc123', image });
 ```
 */
async function provesImage({
  directory,
  image,
  limits,
}: {
  readonly directory: string;
  readonly image: string;
  readonly limits: ExecutionLimits;
}): Promise<boolean> {
  /**
   The manifest and the passed record, each as the only element of a list, or an empty list when it is not usable.
   */
  const [[manifest], [passed]] = await Promise.all([
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
  if ((manifest.image !== image) || (('image' in passed) && (passed.image !== image)))
    return false;
  /**
   The limits the gate ran under, as the only element, or none when its manifest does not record them.
   */
  const [gate] = containerLimitsOf(manifest.limits);
  if (gate === undefined) {
    console.log(`Gate evidence ${directory} passed on this image but does not record its memory and process limits, so it is not used as proof.`);
    return false;
  }
  if (!limitsAtLeast({
    campaign: limits,
    recorded: gate,
  })) {
    console.log(`Gate evidence ${directory} passed on this image at --memory ${gate.memory} --pids-limit ${gate.pidsLimit}, but this campaign runs at --memory ${limits.memory} --pids-limit ${limits.pidsLimit}; with less than the gate had, its failures may be limits and not mutants, so the baseline runs.`);
    return false;
  }
  return true;
}

/**
 List the gate's evidence directories.

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run

 @returns the names in name order, or none when the directory does not exist

 @example
 ```ts
 await gateDirectoryNames('target/verification');
 ```
 */
async function gateDirectoryNames(evidenceRoot: string): Promise<readonly string[]> {
  try {
    return (await readdir(evidenceRoot)).filter(function isGateRun(name) {
      return name.startsWith(gatePrefix) && (!name.startsWith(campaignPrefix));
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
 Find a gate run that passed on an image.

 @param evidenceRoot - directory that keeps the evidence of every gate and campaign run (`target/verification`)

 @param image - full ID of the image the campaign resolved from the gate's tag

 @param limits - the memory and process limits of the campaign, which must be at least the gate's

 @returns the path of the first (in name order) gate directory that proves the pass as the only element, or no element
 when there is none and the campaign must run its own baseline

 @example
 ```ts
 await passingGateEvidence({ evidenceRoot: 'target/verification', image, limits: { memory: '2g', pidsLimit: 512 } });
 ```
 */
export async function passingGateEvidence({
  evidenceRoot,
  image,
  limits,
}: {
  readonly evidenceRoot: string;
  readonly image: string;
  readonly limits: ExecutionLimits;
}): Promise<readonly string[]> {
  /**
   Names of the gate directories, in name order.
   */
  const names = await gateDirectoryNames(evidenceRoot);
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
      limits,
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
  return proving === undefined ? [] : [join(
    evidenceRoot,
    proving,
  )];
}
