import { createHash } from 'node:crypto';
import {
  chmod,
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { isDigit } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

import { CargoMutantsToolingError } from './error.ts';
import {
  isMissing,
  isRecord,
} from './fs-guards.ts';

/**
 Directory below a report directory in which cargo-mutants keeps its output.
 */
export const reportDirectoryName = 'mutants.out';

/**
 List files cargo-mutants appends one mutant name to as it scores each mutant.
 */
export const scoredListNames = [
  'caught.txt',
  'missed.txt',
  'timeout.txt',
  'unviable.txt',
] as const;

/**
 Name of the file cargo-mutants reads with `--iterate` to learn which mutants to leave out.
 */
const seedFileName = 'previously_caught.txt';

/**
 Prefix of the per-attempt report directories.
 */
const attemptPrefix = 'attempt-';

/**
 Digits of an attempt number in a directory name.
 */
const attemptDigits = 3;

/**
 Characters of the campaign key, a prefix of a SHA-256 digest in hexadecimal.
 */
export const campaignKeyLength = 12;

/**
 Permission bits that let the container's user write a directory the host created.
 */
const worldWritableMode = 0o777;

/**
 Mutant names that earlier attempts scored, by outcome.
 */
export type ScoredNames = {
  readonly caught: readonly string[];
  readonly missed: readonly string[];
  readonly timeout: readonly string[];
  readonly unviable: readonly string[];
};

/**
 Derive the key that names one campaign's evidence directory.

 The key binds the directory to the tested image and the exact command, so a rerun of the same campaign finds its
 kept outcomes while a campaign over another snapshot or another scope starts clean. A product adds what else its kept
 outcomes depend on (`keyParts`), so a rerun after that changed starts clean too and no outcome is kept that was scored
 under other conditions.

 @param baseImage - full content-addressed ID of the tested image

 @param command - the complete campaign command

 @param keyParts - the product's further conditions, each a text; none for a product that has none, which keeps its keys

 @returns {@link campaignKeyLength} lowercase hexadecimal digits

 @example
 ```ts
 campaignKey({ baseImage: 'a'.repeat(64), command: ['cargo', 'mutants'] });
 ```
 */
export function campaignKey({
  baseImage,
  command,
  keyParts = [],
}: {
  readonly baseImage: string;
  readonly command: readonly string[];
  readonly keyParts?: readonly string[];
}): string {
  return createHash('sha256')
    .update(JSON.stringify(keyParts.length === 0
      ? [
        baseImage,
        command
      ]
      : [
        baseImage,
        command,
        keyParts
      ]))
    .digest('hex')
    .slice(
    0,
    campaignKeyLength,
    );
}

/**
 Record which image, command and product conditions a campaign directory belongs to, or check that an existing record
 agrees.

 The directory name carries only a prefix of a digest, so two campaigns could in principle share one name; this keeps
 such a clash from silently resuming the wrong campaign.

 @param evidence - the campaign's evidence directory

 @param baseImage - full content-addressed ID of the tested image

 @param command - the complete campaign command

 @param keyParts - the product's further conditions, recorded in full when there are any; none keeps the file as it always was

 @param passes - 2 for a two-pass campaign, absent for a one-pass campaign; a directory started in one mode refuses the other

 @throws CargoMutantsToolingError when the directory already belongs to another image, command or set of conditions, or was started in the other mode

 @example
 ```ts
 await recordCampaignIdentity({ evidence: '/e/x', baseImage: 'a'.repeat(64), command: ['cargo'] });
 ```
 */
export async function recordCampaignIdentity({
  evidence,
  baseImage,
  command,
  keyParts = [],
  passes,
}: {
  readonly evidence: string;
  readonly baseImage: string;
  readonly command: readonly string[];
  readonly keyParts?: readonly string[];
  readonly passes?: 2;
}): Promise<void> {
  /**
   Identity file path.
   */
  const identityPath = join(
    evidence,
    'campaign.json',
  );
  /**
   Text this campaign's identity file holds. A one-pass campaign writes no `passes` field, so its file is what it always
   was.
   */
  const wanted = `${JSON.stringify(
    {
      baseImage,
      command,
      ...(keyParts.length === 0 ? {} : { keyParts }),
      ...(passes === undefined ? {} : { passes }),
    },
    null,
    2,
  )}\n`;
  /**
   Text the identity file holds already, as the only element, or none when there is no file yet.
   */
  const [existing] = await readIdentity(identityPath);
  if (existing === undefined) {
    await writeFile(
      identityPath,
      wanted,
    );
    return;
  }
  if (existing === wanted)
    return;
  /**
   The record in the file, when it is one.
   */
  const parsed: unknown = JSON.parse(existing);
  if (isRecord(parsed) && (parsed.baseImage === baseImage)
    && (JSON.stringify(parsed.command) === JSON.stringify(command))
    && (JSON.stringify(parsed.keyParts ?? []) === JSON.stringify(keyParts)))
    throw new CargoMutantsToolingError(`${evidence} holds a ${parsed.passes === 2 ? 'two-pass' : 'one-pass'} run of this campaign, and running it as a ${passes === 2 ? 'two-pass' : 'one-pass'} one would score its mutants again. Run it the same way as before, or move ${evidence} away to start this campaign afresh.`);
  throw new CargoMutantsToolingError(`${evidence} belongs to another image, command or set of product conditions (${identityPath}); move it away to start this campaign afresh.`);
}

/**
 Read a campaign's identity file.

 @param identityPath - path of `campaign.json`

 @returns its text as the only element, or no element when the file does not exist

 @example
 ```ts
 await readIdentity('/e/native-mutation-0123456789ab/campaign.json');
 ```
 */
async function readIdentity(identityPath: string): Promise<readonly string[]> {
  try {
    return [await readFile(
      identityPath,
      'utf8',
    )];
  }
  catch (error) {
    if (!isMissing(error))
      throw error;
    return [];
  }
}

/**
 Read the names a cargo-mutants list file holds.

 A final line without its newline is dropped: cargo-mutants appends the name and the newline separately, so a kill
 between them leaves a torn line, and the mutant it names counts as not scored.

 @param file - list file path

 @returns the complete lines in order, or none when the file does not exist

 @throws Error when the file cannot be read for a reason other than being absent

 @example
 ```ts
 await readScoredList('/e/attempt-001/mutants.out/caught.txt');
 ```
 */
export async function readScoredList(file: string): Promise<readonly string[]> {
  try {
    return (await readFile(
      file,
      'utf8',
    )).split('\n')
      .slice(
      0,
      -1,
      );
  }
  catch (error) {
    if (isMissing(error))
      return [];
    throw error;
  }
}

/**
 List the attempt directories of a campaign in attempt order.

 @param evidence - the campaign's evidence directory

 @returns absolute paths of the `attempt-NNN` directories

 @example
 ```ts
 await attemptDirectories('/e/native-mutation-0123456789ab');
 ```
 */
export async function attemptDirectories(evidence: string): Promise<readonly string[]> {
  return (await readdir(evidence))
    .filter(function isAttempt(name) {
      /**
       Part of the name after the prefix, which must be the attempt number's digits.
       */
      const digits = name.slice(attemptPrefix.length);
      /**
       Whether every character of the rest is a digit.
       */
      const onlyDigits = Array.from(digits)
        .every(function checkDigit(character) {
          return isDigit(character);
        });
      /**
       Whether the name begins with the attempt prefix.
       */
      const prefixed = name.startsWith(attemptPrefix);
      return prefixed
        && (digits !== '')
        && onlyDigits;
    })
    .toSorted()
    .map(function attemptPath(name) {
      return join(
        evidence,
        name,
      );
    });
}

/**
 Create the report directory of the next attempt.

 @param evidence - the campaign's evidence directory

 @returns the new directory's number and path; the directory is writable by any user, because the container's user is
 not the host user in every image

 @example
 ```ts
 await createAttemptDirectory('/e/native-mutation-0123456789ab');
 ```
 */
export async function createAttemptDirectory(evidence: string): Promise<{
  readonly number: number;
  readonly path: string;
}> {
  /**
   Number of the new attempt.
   */
  const number = (await attemptDirectories(evidence)).length + 1;
  /**
   Path of the new attempt's directory.
   */
  const path = join(
    evidence,
    `${attemptPrefix}${String(number)
      .padStart(
        attemptDigits,
        '0',
      )}`,
  );
  await mkdir(path);
  await chmod(
    path,
    worldWritableMode,
  );
  return {
    number,
    path,
  };
}

/**
 Read every mutant that the attempts of a campaign scored, by outcome.

 @param evidence - the campaign's evidence directory

 @returns the names, each once, in the order they were scored

 @example
 ```ts
 await readScoredNames('/e/native-mutation-0123456789ab');
 ```
 */
export async function readScoredNames(evidence: string): Promise<ScoredNames> {
  /**
   Attempts in order.
   */
  const attempts = await attemptDirectories(evidence);
  /**
   Names of one list over all attempts, without repeats.

   @param listName - list file name

   @returns the names in the order they were scored
   */
  async function unionOf(listName: string): Promise<readonly string[]> {
    return Array.from(
      new Set((await Promise.all(attempts.map(function readList(attempt) {
        return readScoredList(join(
          attempt,
          reportDirectoryName,
          listName,
        ));
      }))).flat()),
    );
  }
  /**
   Names per list, in the order of {@link scoredListNames}.
   */
  const [
    caught = [],
    missed = [],
    timeout = [],
    unviable = []
  ] = await Promise.all(scoredListNames.map(unionOf));
  return {
    caught,
    missed,
    timeout,
    unviable,
  };
}

/**
 Count the mutants a campaign has scored.

 @param names - scored names

 @returns the number of names over all outcomes

 @example
 ```ts
 scoredCount({ caught: ['a'], missed: [], timeout: [], unviable: [] });
 ```
 */
export function scoredCount(names: ScoredNames): number {
  return names.caught
    .length
    + names.missed
      .length
    + names.timeout
      .length
    + names.unviable
      .length;
}

/**
 Leave the names of already scored mutants where `--iterate` reads them, in a report directory the container will use.

 cargo-mutants reads `mutants.out/previously_caught.txt` before it rotates that directory, and leaves out every mutant
 named there, whatever its outcome; caught, missed, timed-out and unviable mutants all count as scored here.

 @param attemptDirectory - report directory of the attempt about to start

 @param names - scored names

 @example
 ```ts
 await seedScoredNames({ attemptDirectory: '/e/attempt-002', names });
 ```
 */
export async function seedScoredNames({
  attemptDirectory,
  names,
}: {
  readonly attemptDirectory: string;
  readonly names: ScoredNames;
}): Promise<void> {
  /**
   Directory cargo-mutants reads and rotates.
   */
  const reportDirectory = join(
    attemptDirectory,
    reportDirectoryName,
  );
  await mkdir(reportDirectory);
  await chmod(
    reportDirectory,
    worldWritableMode,
  );
  await writeFile(
    join(
      reportDirectory,
      seedFileName,
    ),
    [
      ...names.caught,
      ...names.missed,
      ...names.timeout,
      ...names.unviable,
    ].map(function withNewline(name) {
      return `${name}\n`;
    })
      .join(''),
  );
}

/**
 Combine lists of scored names into one, each name once, in the order the lists give them.

 @param parts - scored names to combine, earliest first

 @returns the union per outcome

 @example
 ```ts
 mergeScoredNames([first, second]);
 ```
 */
export function mergeScoredNames(parts: readonly ScoredNames[]): ScoredNames {
  /**
   Union of one outcome over all parts.

   @param outcome - outcome to read from each part

   @returns the names, each once
   */
  function unionOf(outcome: keyof ScoredNames): readonly string[] {
    return Array.from(
      new Set(parts.flatMap(function namesOf(part) {
        return part[outcome];
      })),
    );
  }
  return {
    caught: unionOf('caught'),
    missed: unionOf('missed'),
    timeout: unionOf('timeout'),
    unviable: unionOf('unviable'),
  };
}
