/**
 Selection of the retained control run that the guard observer builds on.
 */
import {
  readdir,
  readFile,
  stat,
} from 'node:fs/promises';
import { join } from 'node:path';

import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';
import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import { ScannerVerificationError } from './container-snapshot.ts';

/**
 One source file recorded in a snapshot manifest.
 */
export type ManifestSource = {
  readonly path: string;
  readonly sha256: string;
};

/**
 The parts of a snapshot manifest the observer reads.
 */
export type ControlManifest = {
  readonly image: string;
  readonly limits: readonly string[];
  readonly sources: readonly ManifestSource[];
};

/**
 A completed, expected control run.
 */
export type Control = {
  readonly directory: string;
  readonly manifest: ControlManifest;
  readonly time: number;
};

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
 Check that a value is a list of texts.

 @param value - parsed JSON

 @returns true when the value is an array of strings

 @example
 ```ts
 isTexts(['a']);
 ```
 */
function isTexts(value: unknown): value is readonly string[] {
  return Array.isArray(value) && value.every(function isText(item) {
    return (typeof item) === 'string';
  });
}

/**
 Check that a value is a recorded source file.

 @param value - parsed JSON

 @returns true when the value has a path and a digest

 @example
 ```ts
 isManifestSource({ path: 'a', sha256: 'b' });
 ```
 */
function isManifestSource(value: unknown): value is ManifestSource {
  if (!isRecord(value)) return false;
  /**
   The recorded path and digest.
   */
  const {
    path,
    sha256,
  } = value;
  return ((typeof path) === 'string') && ((typeof sha256) === 'string');
}

/**
 Read the parts of a manifest the observer needs.

 @param value - parsed `manifest.json`

 @returns the image, the container limits and the recorded sources

 @throws ScannerVerificationError when a part is missing or has the wrong shape

 @example
 ```ts
 const manifest = parseManifest(JSON.parse(text));
 ```
 */
export function parseManifest(value: unknown): ControlManifest {
  if (!isRecord(value)) throw new ScannerVerificationError('The manifest is not an object.');
  /**
   The recorded image ID, container limits and source files.
   */
  const {
    image,
    limits,
    sources,
  } = value;
  if ((typeof image) !== 'string') throw new ScannerVerificationError('The manifest has no image.');
  if (!isTexts(limits)) throw new ScannerVerificationError('The manifest has no container limits.');
  if (!(Array.isArray(sources) && sources.every(isManifestSource)))
    throw new ScannerVerificationError('The manifest has no recorded sources.');
  return {
    image,
    limits,
    sources,
  };
}

/**
 Read one evidence directory as a completed control, or explain why it does not qualify.

 @param directory - an evidence directory named for the variant

 @returns the control as the only item, or no item when its verdict was not the expected one

 @throws Error when a file is missing, malformed or lacks a part the observer needs

 @example
 ```ts
 const control = await readControl(directory);
 ```
 */
async function readControl(directory: string): Promise<readonly Control[]> {
  /**
   The verdict the control run wrote.
   */
  const verdict: unknown = JSON.parse(await readFile(
    join(
      directory,
      'control.json',
    ),
    'utf8',
  ));
  if (!isRecord(verdict)) return [];
  /**
   Whether the control run proved its boundary.
   */
  const { expected } = verdict;
  if (expected !== true) return [];
  /**
   The manifest the control run wrote.
   */
  const manifest: unknown = JSON.parse(await readFile(
    join(
      directory,
      'manifest.json',
    ),
    'utf8',
  ));
  /**
   Status of the verdict file, whose modification time orders the controls.
   */
  const verdictFile = await stat(join(
    directory,
    'control.json',
  ));
  return [
    {
      directory,
      manifest: parseManifest(manifest),
      time: verdictFile.mtimeMs,
    },
  ];
}

/**
 Select only a terminal, expected control result; incomplete or previously failed campaigns do not qualify.

 @param root - directory holding the evidence directories

 @param variant - the control variant

 @returns the newest completed control of the variant

 @throws ScannerVerificationError when no completed control exists

 @example
 ```ts
 const selected = await selectControl({ root, variant: 'protected' });
 ```
 */
export async function selectControl({
  root,
  variant,
}: {
  readonly root: string;
  readonly variant: string;
}): Promise<Control> {
  /**
   Evidence directories of this variant's control runs.
   */
  const directories = (await readdir(
    root,
    { withFileTypes: true },
  ))
    .filter(function isVariantRun(entry) {
      if (!entry.isDirectory()) return false;
      return entry.name
        .startsWith(`guard-${variant}-`);
    })
    .map(function directoryOf(entry) {
      return join(
        root,
        entry.name,
      );
    });
  /**
   The controls that qualified, in directory order.
   */
  const candidates = (await mapInSequence({
    items: directories,
    async run(directory): Promise<readonly Control[]> {
      try {
        return await readControl(directory);
      } catch (error) {
        console.log(`Control not eligible: ${directory}: ${caughtValueText(error)}`);
        return [];
      }
    },
  }))
    .flat()
    .toSorted(function newestFirst(
      first,
      second,
    ) {
      return second.time - first.time;
    });
  /**
   The newest qualifying control.
   */
  const [newest] = candidates;
  if (newest === undefined) throw new ScannerVerificationError(`No completed ${variant} control exists.`);
  return newest;
}
