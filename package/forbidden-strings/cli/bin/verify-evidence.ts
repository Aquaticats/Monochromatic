#!/usr/bin/env node
/**
 Compare retained manifests with current compiled inputs and summarize exact terminal evidence.
 */
import {
  mkdir,
  readdir,
  readFile,
  stat,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
  resolve,
} from 'node:path';

import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';
import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import { ScannerVerificationError } from './container-snapshot.ts';
import {
  compareGuards,
  endingField,
} from './evidence-guard-comparison.ts';
import {
  currentInputs,
  type InputDigest,
} from './evidence-inputs.ts';
import type { ManifestSource } from './guard-control-selection.ts';

/**
 A compiled input that differs between the retained snapshot and the working tree.
 */
type Difference = {
  readonly path: string;
  readonly tested?: string;
  readonly current?: string;
  readonly addedAfterSnapshot?: boolean;
};

/**
 One retained evidence directory with its comparison; the fields appear in the reconciliation file in this order.
 */
type EvidenceRecord = {
  readonly directory: string;
  readonly timestamp: string;
  readonly image: unknown;
  readonly sourceSha256: unknown;
  readonly command: unknown;
  readonly sources: readonly ManifestSource[];
  readonly differences: readonly Difference[];
  readonly endings: Readonly<Record<string, unknown>>;
};

/**
 One ending file of an evidence directory with what it holds.
 */
type Ending = {
  readonly file: string;
  readonly content: unknown;
};

/**
 Files that end an evidence directory, read when present.
 */
const endingFiles: readonly string[] = [
  'exit.json',
  'control.json',
  'mutants.out/outcomes.json',
];

/**
 Path prefix of the fuzz sidecar's sources in a manifest.
 */
const fuzzPrefix = 'package/forbidden-strings/cli.fuzz/';

/**
 Check that a value is a recorded source file.

 @param value - parsed JSON

 @returns true when the value has a path and a digest

 @example
 ```ts
 isSource({ path: 'a', sha256: 'b' });
 ```
 */
function isSource(value: unknown): value is ManifestSource {
  if (((typeof value) !== 'object') || (value === null)) return false;
  if (!(('path' in value) && ((typeof value.path) === 'string'))) return false;
  return ('sha256' in value) && ((typeof value.sha256) === 'string');
}

/**
 Check that a value is an object with named properties.

 @param value - parsed JSON

 @returns true when the value is a non-null object

 @example
 ```ts
 isRecord({});
 ```
 */
function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Read a snapshot manifest, reporting one that cannot be read instead of failing.

 @param manifestPath - path of the `manifest.json`

 @returns the manifest as the only item, or no item when the file is missing or not JSON

 @example
 ```ts
 const manifests = await readManifest(path);
 ```
 */
async function readManifest(manifestPath: string): Promise<readonly unknown[]> {
  try {
    /**
     The parsed manifest.
     */
    const manifest: unknown = JSON.parse(await readFile(
      manifestPath,
      'utf8',
    ));
    return [manifest];
  } catch (error) {
    console.log(`No readable manifest in ${dirname(manifestPath)}: ${caughtValueText(error)}`);
    return [];
  }
}

/**
 Read the ending files of an evidence directory.

 @param directory - the evidence directory

 @returns each ending file's parsed content, or an object naming why it is unavailable

 @example
 ```ts
 const endings = await readEndings(directory);
 ```
 */
async function readEndings(directory: string): Promise<Readonly<Record<string, unknown>>> {
  /**
   Each ending file with its content.
   */
  const entries = await mapInSequence({
    items: endingFiles,
    async run(file): Promise<Ending> {
      try {
        /**
         The parsed ending file.
         */
        const content: unknown = JSON.parse(await readFile(
          join(
            directory,
            file,
          ),
          'utf8',
        ));
        return {
          file,
          content,
        };
      } catch (error) {
        return {
          file,
          content: { unavailable: caughtValueText(error) },
        };
      }
    },
  });
  return Object.fromEntries(entries.map(function pairOf(ending) {
    return [
      ending.file,
      ending.content,
    ];
  }));
}

/**
 List the compiled inputs that differ between a retained snapshot and the working tree.

 @param sources - the sources the snapshot recorded

 @param inputs - the digest of every current input

 @returns the changed, removed and newly added inputs

 @example
 ```ts
 const differences = differencesOf({ sources, inputs });
 ```
 */
function differencesOf({
  sources,
  inputs,
}: {
  readonly sources: readonly ManifestSource[];
  readonly inputs: readonly InputDigest[];
}): readonly Difference[] {
  // Task scripts/docs do not enter Cargo's artifact. Guard-injected source intentionally differs and remains separately labeled.
  /**
   The recorded sources that enter Cargo's artifact.
   */
  const tested = sources.filter(function entersArtifact(source) {
    if (source.path
      .includes('/bin/')) return false;
    return !source.path
      .endsWith('/mise.toml');
  });
  /**
   The current digest of each input path.
   */
  const currentByPath = new Map(inputs.map(function pairOf(input) {
    return [
      input.path,
      input.sha256,
    ] as const;
  }));
  /**
   Paths of the tested sources.
   */
  const testedPaths = new Set(tested.map(function pathOf(source) {
    return source.path;
  }));
  return [
    ...tested.flatMap(function changed(source): readonly Difference[] {
      /**
       The current digest of the same path.
       */
      const current = currentByPath.get(source.path);
      if (source.sha256 === current) return [];
      return [
        {
          path: source.path,
          tested: source.sha256,
          ...(current === undefined ? {} : { current }),
        },
      ];
    }),
    ...inputs
      .filter(function isNew(input) {
        return !testedPaths.has(input.path);
      })
      .map(function added(input): Difference {
        return {
          path: input.path,
          current: input.sha256,
          addedAfterSnapshot: true,
        };
      }),
  ];
}

/**
 Name the terminal status of an evidence directory: the exit status, else the control status.

 @param record - the evidence directory

 @returns the status as text, or a note that the directory has none

 @example
 ```ts
 terminalStatus(record);
 ```
 */
function terminalStatus(record: Pick<EvidenceRecord, 'endings'>): string {
  /**
   The status of the exit file, then of the control file.
   */
  const statuses = [
    endingField({
      record,
      file: 'exit.json',
      field: 'status',
    }),
    endingField({
      record,
      file: 'control.json',
      field: 'status',
    }),
  ];
  /**
   The first status that exists.
   */
  const status = statuses.find(function exists(candidate) {
    return (candidate !== undefined) && (candidate !== null);
  });
  return (typeof status) === 'number' ? String(status) : 'no terminal status';
}

/**
 Compare one retained snapshot with the current compiled inputs.

 @param repository - root of the repository

 @param directory - the evidence directory

 @returns the comparison as the only item, or no item when the directory holds no readable snapshot manifest with sources

 @throws ScannerVerificationError when the recorded sources are not a list of paths with digests

 @example
 ```ts
 const records = await compareDirectory({ repository, directory });
 ```
 */
async function compareDirectory({
  repository,
  directory,
}: {
  readonly repository: string;
  readonly directory: string;
}): Promise<readonly EvidenceRecord[]> {
  /**
   Path of the snapshot manifest.
   */
  const manifestPath = join(
    directory,
    'manifest.json',
  );
  /**
   The manifest, when it can be read.
   */
  const [manifest] = await readManifest(manifestPath);
  if (!isRecord(manifest)) return [];
  /**
   The recorded fields the comparison reads.
   */
  const {
    sources,
    image,
    sourceSha256,
    command,
  } = manifest;
  if ((sources === undefined) || (sources === null)) return [];
  if (!(Array.isArray(sources) && sources.every(isSource)))
    throw new ScannerVerificationError(`The recorded sources in ${directory} are not paths with digests.`);
  /**
   The paths and digests of the current inputs.
   */
  const inputs = await currentInputs({
    repository,
    fuzzing: sources.some(function isFuzz(source) {
      return source.path
        .startsWith(fuzzPrefix);
    }),
  });
  /**
   The record before its endings.
   */
  const record: EvidenceRecord = {
    directory,
    timestamp: (await stat(manifestPath)).mtime
      .toISOString(),
    image,
    sourceSha256,
    command,
    sources,
    differences: differencesOf({
      sources,
      inputs,
    }),
    endings: await readEndings(directory),
  };
  /**
   The terminal status of the directory, as text.
   */
  const status = terminalStatus(record);
  /**
   The directory's name.
   */
  const name = basename(directory);
  /**
   The number of compiled inputs that differ.
   */
  const { length: differenceCount } = record.differences;
  console.log(`${name}: ${differenceCount} compiled-input differences; ${status}`);
  return [record];
}

/**
 List the evidence directories below a verification root.

 @param root - a `target/verification` directory

 @returns the directories that are not the rendered documents

 @example
 ```ts
 const directories = await evidenceDirectories(root);
 ```
 */
async function evidenceDirectories(root: string): Promise<readonly string[]> {
  return (await readdir(
    root,
    { withFileTypes: true },
  ))
    .filter(function isEvidence(entry) {
      if (!entry.isDirectory()) return false;
      return entry.name !== 'docs';
    })
    .map(function directoryOf(entry) {
      return join(
        root,
        entry.name,
      );
    });
}

/**
 Report every mismatch rather than treating a Git HEAD as source-snapshot proof.

 @throws Error when a disabled guard run's main module differs by more than the test registration

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Root of the repository.
   */
  const repository = resolve(
    import.meta.dirname,
    '../../../..',
  );
  /**
   The verification directories of the scanner and of its fuzz sidecar.
   */
  const roots: readonly string[] = [
    resolve(
      import.meta.dirname,
      '../target/verification',
    ),
    resolve(
      import.meta.dirname,
      '../../cli.fuzz/target/verification',
    ),
  ];
  /**
   Every retained evidence directory with its comparison.
   */
  const records = (await mapInSequence({
    items: (await mapInSequence({
      items: roots,
      run: evidenceDirectories,
    })).flat(),
    run: function compareOf(directory) {
      return compareDirectory({
        repository,
        directory,
      });
    },
  })).flat();
  /**
   The newest protected guard run that proved its boundary.
   */
  const [guarded] = records
    .filter(function isProtectedControl(record): boolean {
      return (endingField({
        record,
        file: 'control.json',
        field: 'variant',
      }) === 'protected') && Boolean(endingField({
        record,
        file: 'control.json',
        field: 'expected',
      }));
    })
    .toSorted(function newestFirst(
      first,
      second,
    ) {
      return second.timestamp
        .localeCompare(first.timestamp);
    });
  /**
   The verification directory that receives the reconciliation files.
   */
  const [primaryRoot = ''] = roots;
  /**
   Directory that receives the copies of main modules.
   */
  const extraction = join(
    primaryRoot,
    'guard-source-comparison',
  );
  await mkdir(
    extraction,
    { recursive: true },
  );
  /**
   The comparisons of the disabled guard runs with the protected run.
   */
  const guardComparisons = guarded === undefined ? [] : await compareGuards({
    guarded,
    records,
    extraction,
  });
  await writeFile(
    join(
      primaryRoot,
      'reconciliation.json',
    ),
    `${JSON.stringify(
      records,
      null,
      2,
    )}\n`,
  );
  await writeFile(
    join(
      primaryRoot,
      'guard-input-comparisons.json',
    ),
    `${JSON.stringify(
      guardComparisons,
      null,
      2,
    )}\n`,
  );
}

await main();
