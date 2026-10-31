/**
 Comparison of the protected guard run with each disabled guard run.
 */
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  run,
  ScannerVerificationError,
} from './container-snapshot.ts';
import type { ManifestSource } from './guard-control-selection.ts';

/**
 One retained evidence directory, as far as the comparison reads it.
 */
export type GuardRecord = {
  readonly directory: string;
  readonly image: unknown;
  readonly sources: readonly ManifestSource[];
  readonly endings: Readonly<Record<string, unknown>>;
};

/**
 The comparison of one disabled guard run with the protected run.
 */
export type GuardComparison = {
  readonly protected: string;
  readonly disabled: string;
  readonly differences: readonly string[];
  readonly mainDifferenceOnlyTestRegistration?: boolean;
};

/**
 Source path of the scanner's main module.
 */
const mainPath = 'package/forbidden-strings/cli/src/main.rs';

/**
 Test registration that the process-isolated filter controls add to the main module and that production does not hold.
 */
const testRegistration = '\n/// Process-isolated filter controls do not alter production startup or the embedding host\'s environment.\n#[cfg(test)]\n#[path = "main_logging_tests.rs"]\nmod logging_tests;\n';

/**
 Read one field of a retained ending file.

 @param record - the evidence directory

 @param file - the ending file, such as `control.json`

 @param field - the field to read

 @returns the field, or nothing when the file is unavailable or has no such field

 @example
 ```ts
 const variant = endingField({ record, file: 'control.json', field: 'variant' });
 ```
 */
export function endingField({
  record,
  file,
  field,
}: {
  readonly record: Pick<GuardRecord, 'endings'>;
  readonly file: string;
  readonly field: string;
}): unknown {
  /**
   The ending file's parsed content.
   */
  const ending = record.endings[file];
  if (((typeof ending) !== 'object') || (ending === null)) return undefined;
  return Reflect.get(
    ending,
    field,
  );
}

/**
 Paths and digests of the sources that enter Cargo's artifact, without task scripts and documents.

 @param record - the evidence directory

 @returns each source path with its digest

 @example
 ```ts
 const digests = compiledSources(record);
 ```
 */
function compiledSources(record: Pick<GuardRecord, 'sources'>): ReadonlyMap<string, string> {
  return new Map(record.sources
    .filter(function entersArtifact(source) {
      if (source.path
        .includes('/bin/')) return false;
      return !source.path
        .endsWith('/mise.toml');
    })
    .map(function digestOf(source) {
      return [
        source.path,
        source.sha256,
      ] as const;
    }));
}

/**
 Create a container from a retained image, copy the scanner's main module out of it and remove the container.

 @param image - content-addressed ID of the retained image

 @param extraction - directory that receives the copy

 @returns the text of the main module the image holds

 @example
 ```ts
 const source = await extractMain({ image, extraction });
 ```
 */
async function extractMain({
  image,
  extraction,
}: {
  readonly image: string;
  readonly extraction: string;
}): Promise<string> {
  /**
   The created container.
   */
  const container = (await run({
    command: 'podman',
    args: [
      'create',
      '--network=none',
      '--memory=2g',
      '--cpus=2',
      '--pids-limit=128',
      image,
    ],
    capture: true,
  })).stdout
    .trim();
  /**
   Removal of the container when the copy ends.
   */
  await using cleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      await run({
        command: 'podman',
        args: [
          'rm',
          container,
        ],
        capture: true,
      });
    },
  };
  void cleanup;
  /**
   Where the copy lands.
   */
  const destination = join(
    extraction,
    `${container}.main.rs`,
  );
  await run({
    command: 'podman',
    args: [
      'cp',
      `${container}:/work/package/forbidden-strings/cli/src/main.rs`,
      destination,
    ],
    capture: true,
  });
  return await readFile(
    destination,
    'utf8',
  );
}

/**
 Compare each disabled guard run's compiled inputs with the protected run's.

 @param guarded - the newest expected protected run

 @param records - every retained evidence directory

 @param extraction - directory that receives the copies of main modules

 @returns one comparison per expected disabled guard run

 @throws ScannerVerificationError when a main module differs by more than the test registration

 @example
 ```ts
 const comparisons = await compareGuards({ guarded, records, extraction });
 ```
 */
export async function compareGuards({
  guarded,
  records,
  extraction,
}: {
  readonly guarded: GuardRecord;
  readonly records: readonly GuardRecord[];
  readonly extraction: string;
}): Promise<readonly GuardComparison[]> {
  /**
   The main modules read so far, by image.
   */
  const mainSources = new Map<string, string>();
  /**
   Read the main module of a record's image once.

   @param record - the evidence directory

   @returns the main module text
   */
  async function nativeMain(record: GuardRecord): Promise<string> {
    if ((typeof record.image) !== 'string')
      throw new ScannerVerificationError(`No image is recorded in ${record.directory}.`);
    /**
     The module read earlier, if any.
     */
    const cached = mainSources.get(record.image);
    if (cached !== undefined) return cached;
    /**
     The module read now.
     */
    const source = await extractMain({
      image: record.image,
      extraction,
    });
    mainSources.set(
      record.image,
      source,
    );
    return source;
  }
  /**
   The disabled guard runs that proved their boundary.
   */
  const disabledRuns = records.filter(function isExpectedDisabled(record) {
    return record.directory
      .includes('/guard-without-')
      && (endingField({
        record,
        file: 'control.json',
        field: 'expected',
      }) === true);
  });
  /**
   The protected run's compiled sources.
   */
  const first = compiledSources(guarded);
  return await mapInSequence({
    items: disabledRuns,
    async run(record): Promise<GuardComparison> {
      /**
       The disabled run's compiled sources.
       */
      const second = compiledSources(record);
      /**
       Every path either run holds.
       */
      const paths = new Set([
        ...first.keys(),
        ...second.keys(),
      ]);
      /**
       Paths whose digests differ.
       */
      const differences = [...paths].filter(function differs(path) {
        return first.get(path) !== second.get(path);
      });
      /**
       Whether the main module differs, outside the variant that removes the output hook.
       */
      const mainDiffers = differences.includes(mainPath) && (endingField({
        record,
        file: 'control.json',
        field: 'variant',
      }) !== 'without-output-hook');
      if (mainDiffers) {
        /**
         Whether the disabled main module equals the protected one once the test registration is removed.
         */
        const onlyRegistration = (await nativeMain(guarded)).replace(
          testRegistration,
          '',
        ) === (await nativeMain(record)).replace(
          testRegistration,
          '',
        );
        if (!onlyRegistration)
          throw new Error(`Unexpected main implementation difference in ${record.directory}.`);
      }
      console.log(`Guard input comparison ${record.directory}: ${differences.join(', ')}`);
      return {
        protected: guarded.directory,
        disabled: record.directory,
        differences,
        ...(mainDiffers ? { mainDifferenceOnlyTestRegistration: true } : {}),
      };
    },
  });
}
