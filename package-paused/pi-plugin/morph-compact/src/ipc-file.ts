/**
 File-based IPC for morph-compact.
 
 Writes compressed context to a temp file when it exceeds the argv
 length limit. The new pi session reads the file via the
 `--morph-compact-file` flag and deletes it after reading.
 
 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchParent, } from './scratch-parent.ts';
//region Types

/**
 Result of writing a compact file.
 */
export type WriteCompactFileResult = {
  /**
   Absolute path to the temp file.
   */
  filePath: string;
  /**
   Deletes the temp directory containing the file.
   */
  cleanup: () => Promise<void>;
};

//endregion

//region File operations

/**
 Write compressed context to a temp file.
 
 Creates a unique directory under the tool scratch directory
 (e.g. `node_modules/.monochromatic/morph-compact/morph-compact-<uuid>/data.txt`) to avoid
 name collisions and simplify cleanup.
 
 @param text - the compressed context string
 
 @returns the file path and a cleanup function
 
 @throws when the temp directory or file cannot be created
 
 @example
 ```typescript
 const { filePath, cleanup } = await writeCompactFile(compressedText);
 // pass filePath to new pi session via --morph-compact-file
 // cleanup is called after the session reads the file
 ```
 */
export async function writeCompactFile(
  text: string,
): Promise<WriteCompactFileResult> {
  /**
   Unique temp directory whose removal yields a single cleanup target.
   */
  const dir = await mkdtemp(
    join(
      await scratchParent(),
      'morph-compact-',
    ),
  );
  /**
   Final write target inside the temp directory; surfaced to the new session.
   */
  const filePath = join(
    dir,
    'data.txt',
  );
  await writeFile(
    filePath,
    text,
    'utf8',
  );

  return {
    filePath,
    cleanup: async function cleanup(): Promise<void> {
      await rm(
        dir,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Read compressed context from a temp file.
 
 @param filePath - absolute path to the temp file
 
 @returns the file contents as a string
 
 @throws when the file cannot be read
 
 @example
 ```typescript
 const text = await readCompactFile(filePath);
 // text contains the Morph-compressed conversation context
 ```
 */
export async function readCompactFile(
  filePath: string,
): Promise<string> {
  return await readFile(
    filePath,
    'utf8',
  );
}

//endregion
