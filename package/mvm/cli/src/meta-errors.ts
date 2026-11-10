/**
 Errors raised when a VM's stored metadata exists but cannot be used.

 A `meta.json` that is there holds the only record of which guest the VM was
 made for. Answering a failed read or a failed parse with the legacy `image`
 fallback would rebuild a different guest, so these failures stop the caller
 with the file named.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region Errors

/**
 The VM's metadata file exists but its contents could not be read.

 @example
 ```ts
 try {
   await readVmMeta(vmDir);
 }
 catch (error) {
   if (error instanceof UnreadableVmMetaError) console.error(error.metaPath);
 }
 ```
 */
export class UnreadableVmMetaError extends Error {
  /**
   Path of the metadata file that could not be read.
   */
  readonly metaPath: string;

  /**
   @param cause - Read failure reported by the operating system

   @param metaPath - Absolute path of the metadata file
   */
  constructor({
    cause,
    metaPath,
  }: {
    readonly cause: unknown;
    readonly metaPath: string;
  },) {
    super(
      [
        `The VM metadata at ${metaPath} is there but could not be read, `
          + `so this VM's guest is unknown: ${caughtValueText(cause,)}`,
        'mvm falls back to the legacy `image` file only when `meta.json` is absent, '
          + 'so this stops here instead of treating the VM as one made before `meta.json`.',
        `Fix the file's permissions or replace its contents, then run the command again. `
          + `Deleting ${metaPath} also works, if you accept that the guest is then rebuilt `
          + 'from the legacy `image` file and the recorded file-transfer route is lost.',
      ].join('\n',),
      { cause, },
    );
    this.name = 'UnreadableVmMetaError';
    this.metaPath = metaPath;
  }
}

/**
 The VM's metadata file exists but does not hold usable VM metadata.

 @example
 ```ts
 try {
   await readVmMeta(vmDir);
 }
 catch (error) {
   if (error instanceof InvalidVmMetaError) console.error(error.problem);
 }
 ```
 */
export class InvalidVmMetaError extends Error {
  /**
   Path of the metadata file that could not be used.
   */
  readonly metaPath: string;

  /**
   What is wrong with the file's contents, as one sentence.
   */
  readonly problem: string;

  /**
   @param cause - Parse failure, when the text is not JSON at all

   @param metaPath - Absolute path of the metadata file

   @param problem - What the contents fail to provide
   */
  constructor({
    cause,
    metaPath,
    problem,
  }: {
    readonly cause?: unknown;
    readonly metaPath: string;
    readonly problem: string;
  },) {
    super(
      [
        `The VM metadata at ${metaPath} cannot be used: ${problem}.`,
        `mvm falls back to the legacy \`image\` file only when \`meta.json\` is absent, `
          + 'so this stops here instead of treating the VM as one made before `meta.json`.',
        `Rewrite ${metaPath} as the JSON object \`mvm create\` writes, or delete it `
          + 'to rebuild the guest from the legacy `image` file.',
      ].join('\n',),
      cause === undefined ? undefined : { cause, },
    );
    this.name = 'InvalidVmMetaError';
    this.metaPath = metaPath;
    this.problem = problem;
  }
}

//endregion Errors

//region Legacy image file

/**
 The legacy `image` file of a VM made before `meta.json` exists but cannot be read.

 The file is the only record of which image such a VM came from, so answering
 an unreadable file with the default image would silently move the VM to a
 different guest.

 @example
 ```ts
 try {
   await readVmMeta(vmDir);
 }
 catch (error) {
   if (error instanceof UnreadableLegacyImageError) console.error(error.legacyImagePath);
 }
 ```
 */
export class UnreadableLegacyImageError extends Error {
  /**
   Path of the legacy image file that could not be read.
   */
  readonly legacyImagePath: string;

  /**
   @param cause - Read failure reported by the operating system

   @param legacyImagePath - Absolute path of the legacy image file
   */
  constructor({
    cause,
    legacyImagePath,
  }: {
    readonly cause: unknown;
    readonly legacyImagePath: string;
  },) {
    super(
      [
        `The legacy VM image file at ${legacyImagePath} is there but could not be read, `
          + `so this VM's guest is unknown: ${caughtValueText(cause,)}`,
        'mvm assumes a default image only when the file is absent, so this stops here '
          + 'instead of moving the VM to another guest.',
        `Fix the file's permissions or replace its contents with an image identifier, `
          + `then run the command again. Deleting ${legacyImagePath} makes mvm assume its default image.`,
      ].join('\n',),
      { cause, },
    );
    this.name = 'UnreadableLegacyImageError';
    this.legacyImagePath = legacyImagePath;
  }
}

//endregion Legacy image file
