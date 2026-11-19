/**
 The file Pi keeps a tool's whole output in: how a path is shown to be one of Pi's own, how it is read without waiting on anything
 that is not an ordinary file, and how the masked text is put in its place.

 Pi writes those files with mode 0600 in the folder `os.tmpdir()` names, which for one of this package's runs is
 `temporary-files-pi-wrote/` in the run folder, named `pi-output-<random hex><extension>`
 (`dist/utils/output-files.js` and `dist/core/tools/output-accumulator.js:81` in Pi 1.5.2). A path is accepted only when both of
 those hold, so a file somebody else made is never rewritten on the word of a tool result. The masked text goes to a file of its
 own beside the original and is renamed over it, so the path holds either the whole original or the whole masked text and never a
 half-written one.

 @module
 */

import { replaceAFileWithItsMaskedText, } from './replacing-a-file-only-if-it-is-the-one-that-was-read.ts';
// oxlint-disable-next-line no-restricted-syntax/no-system-tmpdir -- this names the folder Pi writes its own output files in; nothing of this package is written there.
import { tmpdir, } from 'node:os';
import {
  basename,
  dirname,
} from 'node:path';
import { textOfAnOrdinaryFile, } from './ordinary-file-opened-without-waiting.ts';
import { fieldsOf, } from './read-pi-fields.ts';
import type { SecretValueMasking, } from './secret-value-masking.ts';

//region Constants

/**
 The whole of the name Pi gives a file it keeps a tool's whole output in: `pi-output-`, sixteen random hex characters, then an
 extension (`dist/utils/output-files.js` in Pi 1.5.2, `randomBytes(8).toString("hex")`). A file whose name is not that is somebody
 else's, whatever the prefix says.
 */
// oxlint-disable-next-line no-restricted-syntax/no-regex -- the whole name Pi gives one file is a shape, and the pattern is anchored with bounded runs.
const PI_OUTPUT_FILE_NAME_PATTERN = /^pi-output-[0-9a-f]{16}(?:\.[^/]*)?$/u;

/**
 How many random bytes the file of its own is named with, which is what keeps two rewrites of one folder apart.
 */
const REPLACEMENT_NAME_BYTES = 8;

/**
 The mode every file Pi wrote is given, and the one its replacement keeps.
 */
const PRIVATE_FILE_MODE = 0o600;

//endregion Constants

//region Which file is Pi's

/**
 The folder Pi writes the files it keeps a tool's whole output in.

 For one of this package's runs that folder is `temporary-files-pi-wrote/` in the run folder, because the driver hands Pi the
 variables that name it (`temporary-folder-of-a-run.ts`), and this function runs inside Pi, so it asks Pi's own process.

 Reading this is the one thing this package does with that folder: it writes nothing of its own there, and it uses the folder only
 to tell Pi's own output file from a file some other program named.

 @returns Pi's output folder, as this machine's operating system names it
 */
function folderOfPiOutputFiles(): string {
  /**
   The folder, as this machine's operating system names it.
   */
  const folder = tmpdir();
  return folder;
}

/**
 Reads the path of the file Pi keeps a tool's whole output in, when the result names one Pi could have made.

 @param details - the tool result's own details, of any shape

 @param piOutputFolder - the folder Pi writes its own output files in, which a test names for itself

 @returns the path, or that the result named no file of Pi's
 
 @example
 ```ts
 const spareCopy = spareCopyOfToolOutput({ details: event.details, },);
 ```
 */
export function spareCopyOfToolOutput({
  details,
  piOutputFolder = folderOfPiOutputFiles(),
}: {
  readonly details: unknown;
  readonly piOutputFolder?: string;
},): { readonly path: string } | { readonly absent: true } {
  /**
   The field Pi puts the path in.
   */
  const path = fieldsOf(details,)
    .fullOutputPath;
  if ((typeof path) !== 'string') {
    return { absent: true, };
  }
  /**
   Whether the path is where Pi could have written the file: its own folder, and the whole of its own naming rule, which is not the
   prefix alone.
   */
  const isPiOwn = (dirname(path,) === piOutputFolder)
    && PI_OUTPUT_FILE_NAME_PATTERN.test(basename(path,),);
  return isPiOwn ? { path, } : { absent: true, };
}

//endregion Which file is Pi's

//region Reading it

/**
 Reads the text of a file Pi saved a tool's whole output in.

 @param path - the file

 @returns its text, or that it is not a file this code may rewrite
 */
async function textOfTheSpareCopy({ path, }: { readonly path: string },): Promise<
  | { readonly text: string }
    | { readonly absent: true }
  > {
  /**
   What the path gave, which is its text only when it holds an ordinary file.
   */
  const read = await textOfAnOrdinaryFile({ path, },);
  return ('text' in read) ? { text: read.text, } : { absent: true, };
}

//endregion Reading it

//region Putting the masked text in its place

/**
 Masks the file Pi saved a truncated tool output in, so a value masked in what the model reads is not left in what Pi wrote.

 The masked text goes to a file of its own beside the original and is renamed over it, and only while the path still names the file
 that was read: a file another command wrote in between is read and masked on the next turn, not lost.

 @param path - the file Pi saved the output in

 @param masking - the masking

 @returns why the file could not be rewritten, when it could not

 @example
 ```ts
 const problem = await maskSpareCopyOfToolOutput({ path, masking, },);
 ```
 */
export async function maskSpareCopyOfToolOutput({
  path,
  masking,
}: {
  readonly path: string;
  readonly masking: SecretValueMasking;
},): Promise<{ readonly problem?: string }> {
  /**
   What replacing the file found.
   */
  const replaced = await replaceAFileWithItsMaskedText({
    path,
    maskText: masking.maskText,
    // Pi's own writer of this file is gone: its bash tool finishes the output and closes the file
    // (`finishOutput` awaits `output.closeTempFile()`) before the tool returns the result this hook is masking.
    writer: { gone: true, },
  },);
  return ('problem' in replaced)
    ? { problem: `the whole output of a masked tool result could not be rewritten, so the file keeps what was written: ${replaced.problem}` }
    : {};
}

//endregion Putting the masked text in its place
