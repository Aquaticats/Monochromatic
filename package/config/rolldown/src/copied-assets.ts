import { readFile, } from 'node:fs/promises';

import type { Plugin, } from 'rolldown';

//region Types: copied file description and its error

/**
 One file the build copies into its output directory.

 @example
 ```ts
 const file: CopiedAsset = { source: '/repo/src/data.jsonc', fileName: 'data.jsonc' };
 ```
 */
export type CopiedAsset = {
  /**
   Absolute path of the file to read as bytes.
   */
  readonly source: string;
  /**
   Output-directory-relative name the copy is written under.
   */
  readonly fileName: string;
};

/**
 Raised when a file the build must copy cannot be read.

 Names the source path so a moved or misspelled asset stops the build
 instead of shipping an output the runtime cannot find.

 @example
 ```ts
 throw new CopiedAssetReadError({ source: '/repo/src/data.jsonc', cause: error });
 ```
 */
export class CopiedAssetReadError extends Error {
  /**
   Path of the file that could not be read.
   */
  readonly source: string;

  /**
   Builds the failure naming the unreadable path.

   @param source - Path of the file that could not be read.

   @param cause - Error the read raised, kept for diagnosis.

   @example
   ```ts
   new CopiedAssetReadError({ source: '/repo/src/data.jsonc', cause: error, },);
   ```
   */
  public constructor(
    {
      source,
      cause,
    }: {
      readonly source: string;
      readonly cause: unknown;
    },
  ) {
    super(
      `Cannot copy "${source}" into the build output because it cannot be read.`
        + ' Check that the file exists at that path.',
      { cause, },
    );
    this.name = 'CopiedAssetReadError';
    this.source = source;
  }
}

//endregion Types: copied file description and its error

//region Plugin

/**
 Read one source file as bytes, naming the path when it cannot be read.

 @param source - Absolute path of the file to read.

 @returns The file's bytes.

 @throws CopiedAssetReadError when the file cannot be read.

 @example
 ```ts
 const bytes = await readCopiedAsset({ source: '/repo/src/data.jsonc' });
 ```
 */
async function readCopiedAsset({ source, }: {
  readonly source: string;
},): Promise<Uint8Array> {
  try {
    return await readFile(source,);
  }
  catch (error) {
    throw new CopiedAssetReadError({
      source,
      cause: error,
    },);
  }
}

/**
 Rolldown plugin copying listed files into the output directory.

 A module bundled into several packages cannot find a data file by walking
 up from its own location, because that location differs per bundle. A copy
 emitted beside the chunks is found wherever the bundle runs, and its bytes
 sit in the output directory that content digests hash.

 Bytes are emitted unchanged under the given name, relative to the output
 directory. Each source is also registered as a watched file, so watch
 rebuilds follow edits.

 @param files - Files to copy, each with an absolute source path and an output-relative name.

 @returns Rolldown plugin emitting one asset per file in `generateBundle`.

 @throws CopiedAssetReadError when a source file cannot be read, so the build fails.

 @example
 ```ts
 plugins: [copiedAssetsPlugin({ files: [{ source: '/repo/src/data.jsonc', fileName: 'data.jsonc' },], },),];
 ```
 */
export function copiedAssetsPlugin({ files, }: {
  readonly files: readonly CopiedAsset[];
},): Plugin {
  return {
    name: 'monochromatic:copied-assets',
    async generateBundle(): Promise<void> {
      /**
       Every listed file with its bytes, read together and in listed order.
       */
      const read = await Promise.all(files.map(
        async function readListed(file: CopiedAsset,): Promise<{
          readonly file: CopiedAsset;
          readonly bytes: Uint8Array;
        }> {
          return {
            file,
            bytes: await readCopiedAsset({ source: file.source, },),
          };
        },
      ),);
      for (
        const {
          file,
          bytes,
        } of read
      ) {
        this.addWatchFile(file.source,);
        this.emitFile({
          type: 'asset',
          fileName: file.fileName,
          originalFileName: file.source,
          source: bytes,
        },);
      }
    },
  };
}

//endregion Plugin
