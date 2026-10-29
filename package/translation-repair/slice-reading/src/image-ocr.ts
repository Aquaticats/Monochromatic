import { execFile, } from 'node:child_process';
import {
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { isAsciiAlphanumeric, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  childEnvironment,
  isMissingPathError,
  rethrowUnlessMissingPath,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { extensionOf, } from './image-asset.ts';
import {
  MIN_READING_CHARS,
  solidCharacters,
} from './image-reading-sense.ts';
import type { OcrReader, } from './image-reading-pair.ts';
import {
  LocalProgramFailedError,
  localProgramFailureOf,
  readerFailureText,
} from './local-program-failure.ts';
import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';

import { scratchParent, } from './scratch-parent.ts';
//region Image OCR
// READING A PICTURE WITHOUT ASKING A MODEL, which is the first thing to try and
// on this corpus usually the last.
//
// THREE REASONS IT COMES FIRST, and the owner's ruling named the third.
//
// MOST PICTURES HAVE NOTHING TO READ. Measured with `tesseract` over all 191
// distinct source-referenced assets: 119 carry no text at all. They are
// photographs of people. A reader declining those is the RIGHT answer, not a
// miss, and asking two vision models about them spends quota to be told so.
//
// IT AGREES WITH THE MODELS. On the six assets where a model reading is on
// record, six of six agree in both directions: tesseract read the three they
// read, at 205, 388 and 540 characters against their 390/394, 448/454 and
// 590/632, and found nothing in the three they declined.
//
// IT IS DETERMINISTIC, which the models are not. `wangzihao980/Word1.webp`
// corroborated at 0.643 in one probe and disagreed at 0.087 in a corpus pass,
// same bytes and same two models. Putting a party that cannot vary on one side
// of the comparison removes half that variance.
//
// IT HAS NO CONTEXT CAP EITHER. 45 of the 191 assets are past the smaller
// reader's byte allowance and cannot be sent to a model at all without
// re-encoding. Tesseract reads them as they are.
//
// THIS MAKES `tesseract` A DEPLOYMENT DEPENDENCY. It is reported as a named
// unavailable reason rather than silently skipped, because a run that quietly
// stopped reading pictures would look exactly like a corpus that stopped
// carrying them.

/**
 How the OCR reader is invoked, including the language data it needs.

 BOTH SCRIPTS, since the corpus is Chinese and its pictures carry Latin
 handles, dates and place names inside otherwise Chinese text.
 */
const TESSERACT_LANGUAGES = 'chi_sim+eng';

// THE TEXTLESS LINE IS `MIN_READING_CHARS`, IMPORTED rather than copied: the
// copy this replaced was a literal 16 that could drift from the value it
// claimed to be drawn from.
// Shortest OCR yield treated as text rather than as noise, in characters after
// whitespace is removed.
//
// STATED AS A CHOICE, because the measurement does not make it for us. The
// yield over 191 assets runs 0, then 1, 2, 4 and up through 2965 with no gap
// anywhere: 119 assets return nothing and the remaining 72 form a continuum. So
// there is no boundary to discover, only a line to draw.
//
// DRAWN AT `MIN_READING_CHARS`, the value `image-reading-sense.ts` already uses
// for a model reading too short to be a reading. Reusing it keeps ONE notion of
// "too short to be a transcription" rather than inventing a second that would
// drift from it. At this line 60 assets read and 12 low-yield ones are called
// textless.
//
// THE ERRORS ARE NOT SYMMETRIC, which is why the line sits where it does rather
// than lower. Calling a caption textless loses a little evidence about a
// picture that 79 of 1260 slices mention. Calling noise text spends model calls
// on nothing and, worse, offers the corroboration stage two pieces of garbage
// that may agree with each other.

// THE PROGRAMS ARE RUN THROUGH A FUNCTION THE CALLER HANDS IN, never reached
// for. A decoder and an OCR reader are programs on a machine, so a function
// that starts them by itself can be tested only on a machine that carries
// them and their language data, and its tests then say what that machine
// does. The runner is REQUIRED (ledger M70): a default would be the
// production value, and a test that left it out would start the real
// programs. A run names the real runner, `runInstalledProgram`, once, where
// it builds its picture sources (`corpus-run/pass-visual-evidence.ts`).

/**
 The decoder tried first, which reads the format nearly every corpus picture
 is in.
 */
const WEBP_DECODER = 'dwebp';

/**
 The decoder tried second, for every format the first refuses.
 */
const FALLBACK_DECODER = 'magick';

/**
 The OCR reader.
 */
const OCR_READER = 'tesseract';

/**
 Runs one program to completion: the seam between this module and the
 programs installed on a machine.

 RESOLVES once the program has run and exited zero. Nothing it printed is
 handed back, since every program this module runs leaves its result in a
 file it was told to write.

 REJECTS otherwise, and a caller tells the two rejections apart: a program
 that is not installed rejects with the error `isMissingPathError`
 recognises, Node's `ENOENT` spawn failure, and a program that ran and
 exited unhappy rejects with anything else.

 @example
 ```ts
 const runProgram: ProgramRunner = async () => {};
 ```
 */
export type ProgramRunner = (
  run: {
    /**
     Name the system finds the program by, or its path.
     */
    readonly program: string;

    /**
     Its arguments, in order.
     */
    readonly args: readonly string[];
  },
) => Promise<void>;

/* oxlint-disable typescript/strict-void-return -- promisify deliberately ignores Node execFile's ChildProcess return while adapting its callback */
/**
 Node's `execFile` as a promise, which starts a program without a shell and
 waits for it.
 */
const execFileAsync = promisify(execFile,);
/* oxlint-enable typescript/strict-void-return */

/**
 Runs a program installed on this machine and waits for it: the runner a run
 hands to {@link readImageWithOcr}, since every reader this module reaches
 for is a program rather than a library.

 NODE'S OWN REJECTION IS PASSED ON UNCHANGED, which is what lets a caller
 tell a program that is not installed (`ENOENT`, a string code) from one that
 ran and exited unhappy (its exit code, a number).

 @param program - name the system finds the program by, or its path

 @param args - its arguments, handed over as they are and never through a
 shell

 @throws Node's spawn failure when the program is not installed, and its
 exit failure when the program exits with a code other than zero

 @example
 ```ts
 await runInstalledProgram({ program: 'dwebp', args: [source, '-o', png,], },);
 ```
 */
export async function runInstalledProgram(
  {
    program,
    args,
  }: {
    readonly program: string;
    readonly args: readonly string[];
  },
): Promise<void> {
  await execFileAsync(
    program,
    args,
    { env: childEnvironment({ parent: process.env, },), },
  );
}

/**
 What reading a picture without a model produced.

 @example
 ```ts
 const reading: OcrReading = { kind: 'no-text', characters: 0, };
 ```
 */
export type OcrReading = {
  /**
   Text was found and is long enough to be a transcription.
   */
  readonly kind: 'read';

  /**
   What the picture says, as the OCR read it.
   */
  readonly text: string;
} | {
  /**
   The picture carries nothing worth transcribing, which for two thirds of
   this corpus is the true answer rather than a failure.
   */
  readonly kind: 'no-text';

  /**
   How much the OCR did return, so a run can tell a clean nothing from a few
   characters of noise below the line.
   */
  readonly characters: number;
} | {
  /**
   The reading could not be attempted.
   */
  readonly kind: 'unavailable';

  /**
   Which step failed, so a missing tool is never mistaken for an empty
   picture.
   */
  readonly reason: 'undecodable' | 'ocr-tool-missing' | 'ocr-failed';
};

/**
 Directory that removes itself, so no cleanup depends on a `finally`.

 @example
 ```ts
 await using scratch = await scratchDirectory();
 ```
 */
type ScratchDirectory = {
  /**
   Where files may be written.
   */
  readonly path: string;

  /**
   Removes it and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a private directory that removes itself when it leaves scope.

 THROWAWAY BY CONSTRUCTION. Every intermediate this module writes is a decoded
 copy of somebody's photograph, so it lives under the system temporary
 directory for the length of one reading and no longer.

 @returns Directory and its disposer

 @throws Whatever `mkdtemp` raises when a temporary directory cannot be made

 @example
 ```ts
 await using scratch = await scratchDirectory();
 ```
 */
async function scratchDirectory(): Promise<ScratchDirectory> {
  /**
   Freshly made directory nothing else knows about.
   */
  const path = await mkdtemp(join(
    await scratchParent(),
    'translation-repair-ocr-',
  ),);

  return {
    path,
    [Symbol.asyncDispose]: async function removeScratch(): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

// THE SCRATCH NAMES ARE FIXED, AND NO TWO OF THEM CAN BE ONE FILE. Three files
// land in scratch: the picture as gathered, its decoded copy, and the reading.
// The asset's own name chooses none of them; it lends the first a short
// extension from a closed alphabet and nothing else.

/**
 Stem of the scratch copy of a picture as it was gathered.
 */
const SCRATCH_COPY_STEM = 'asset';

/**
 Longest extension the scratch copy takes from an asset's name, in
 characters.

 A CHOICE WITH HEADROOM. The longest extension the corpus's pictures carry
 is four characters (`MEDIA_TYPES` in `image-asset.ts`), and no picture
 format is told by one four times that. What the line buys is a scratch name
 of bounded length, so no asset name can make the write fail for being too
 long a file name.
 */
const MAX_SCRATCH_EXTENSION_CHARS = 16;

/**
 Name of the decoded copy, which the scratch copy can never share.

 A NAME OUTSIDE EVERY NAME `scratchCopyName` BUILDS, all of which open
 `asset.`. While this was `asset.png`, the decoded copy of every `.png`
 asset was the scratch copy itself: the fallback decoder rewrote its own
 input in place, and a first decoder that wrote part of its output before
 failing would have destroyed the picture the fallback is there to read.
 */
const DECODED_COPY_NAME = 'decoded.png';

/**
 Stem the OCR reader appends `.txt` to.
 */
const READING_STEM = 'reading';

/**
 Name of the scratch copy of a picture: `asset.` and the asset's extension
 where that extension is a short run of ASCII letters and digits, `asset.`
 alone for every other name.

 THE EXTENSION IS TAKEN ONLY FROM A CLOSED ALPHABET, since it is text from a
 file name on its way into a path. `extensionOf` hands back everything after
 a name's last dot, which for an asset called `../../etc/passwd` is
 `/etc/passwd`: joined into the scratch copy's name that pointed into a
 directory nobody made, and the write failed. No such text can climb out of
 scratch, since what follows a name's last dot holds no dot, but a name is
 fixed only when the asset chooses nothing of it beyond a suffix a decoder
 could want. An extension holding anything else is dropped, and the picture
 is still read, since a decoder tells a format by its bytes.

 @param assetName - file name the picture was gathered under

 @returns File name within scratch, never a path

 @example
 ```ts
 const name = scratchCopyName({ assetName: 'letter.WEBP', },); // 'asset.webp'
 ```
 */
function scratchCopyName({ assetName, }: { readonly assetName: string; },): string {
  /**
   What the asset's name carries after its last dot, lowercased.
   */
  const extension = extensionOf({ assetName, },);
  if (extension.length > MAX_SCRATCH_EXTENSION_CHARS)
    return `${SCRATCH_COPY_STEM}.`;
  for (const character of extension)
    if (!isAsciiAlphanumeric({ character, },))
      return `${SCRATCH_COPY_STEM}.`;
  return `${SCRATCH_COPY_STEM}.${extension}`;
}

/**
 The line recording why one decoder produced nothing: that it is not
 installed, or that it ran and refused the picture.

 TOLD APART, since they are different problems for whoever reads the run: a
 machine missing a decoder needs it installed, and a picture a decoder
 refuses is expected of every format that decoder does not read. Read off
 the failure's code, as the OCR reader's failure is: `isMissingPathError`
 for a decoder not installed, and `LocalProgramFailedError` for the exit
 code or filesystem code of one that ran, never the rejection's own words,
 which quote the scratch paths and whatever the decoder printed. Both used
 to print as `refused by Error`, under the scratch copy's path rather than
 the asset's name, which told a reader neither which picture nor which
 problem; until 2026-10-06 the second still printed `refused by Error`, its
 exit code lost.

 @param assetName - picture the decoder was run on

 @param program - decoder that produced nothing

 @param error - what running it raised

 @returns Line naming the asset, the decoder and which of the two it
 was, with the code a decoder that ran failed with

 @example
 ```ts
 l.debug(decoderFailureLine({ assetName, program: 'dwebp', error, },),);
 ```
 */
function decoderFailureLine(
  {
    assetName,
    program,
    error,
  }: {
    readonly assetName: string;
    readonly program: string;
    readonly error: unknown;
  },
): string {
  if (isMissingPathError({ error, },))
    return `${assetName}: ${program} is not installed`;

  /**
   The failure as an account authored from its code.
   */
  const failure = new LocalProgramFailedError({
    program,
    failure: localProgramFailureOf({ error, },),
    cause: error,
  },);
  return `${assetName}: ${program} did not decode it (${refusalText({ error: failure, },)})`;
}

/**
 Decodes a picture to PNG, which is what the OCR reader accepts.

 `dwebp` FIRST AND `magick` SECOND, which is the opposite of what it looks
 like it should be. ImageMagick on this machine has no working webp reader and
 fails outright on the format 187 of 191 corpus assets use, while `dwebp`
 handles exactly that format. So the specific tool leads and the general one
 covers the rest.

 @param source - picture as written to scratch

 @param png - where the decoded copy should land, never the same file as
 `source`, so a decoder that fails partway leaves the picture whole for the
 decoder tried after it

 @param assetName - picture's own name, for the lines the failures are
 recorded under

 @param l - logger the two decoder failures are recorded on

 @param runProgram - runner both decoders go through

 @returns Whether either decoder produced one

 @example
 ```ts
 const decoded = await decodeToPng({ source, png, assetName, l, runProgram, },);
 ```
 */
async function decodeToPng(
  {
    source,
    png,
    assetName,
    l,
    runProgram,
  }: {
    readonly source: string;
    readonly png: string;
    readonly assetName: string;
    readonly l: Logger;
    readonly runProgram: ProgramRunner;
  },
): Promise<boolean> {
  try {
    await runProgram({
      program: WEBP_DECODER,
      args: [
        source,
        '-o',
        png,
      ],
    },);
    return true;
  }
  catch (error) {
    // Expected for every format that is not webp, which is what the fallback is
    // for. Recorded rather than silent so a machine with one decoder missing
    // and one refusing the file is diagnosable from a log.
    l.debug(decoderFailureLine({
      assetName,
      program: WEBP_DECODER,
      error,
    },),);
  }

  try {
    await runProgram({
      program: FALLBACK_DECODER,
      args: [
        source,
        png,
      ],
    },);
    return true;
  }
  catch (error) {
    l.debug(decoderFailureLine({
      assetName,
      program: FALLBACK_DECODER,
      error,
    },),);
    return false;
  }
}

/**
 What the OCR reader left where it was told to write its reading.

 A NAMED OUTCOME rather than a nullish union, which this repository does not
 model absence with.

 @example
 ```ts
 const left: ReadingLeft = { kind: 'absent', };
 ```
 */
type ReadingLeft = {
  readonly kind: 'written';

  /**
   What it transcribed, whitespace and all.
   */
  readonly text: string;
} | {
  /**
   The reader exited without an error and wrote no file.
   */
  readonly kind: 'absent';
};

/**
 Reads the transcript the OCR reader was told to write, or says that it
 wrote none.

 THE FILE COMES FROM OUTSIDE THE PROCESS, so its absence is a state to
 answer for rather than one to assume away: a reader that exits without an
 error and leaves nothing has failed, and that is a fact about the reader,
 not a fault in this machine.

 @param path - where the reader was told to write

 @returns Transcript, or that no file stands there

 @throws Whatever reading the file raises other than its absence

 @example
 ```ts
 const left = await readingLeftAt({ path: `${stem}.txt`, },);
 ```
 */
async function readingLeftAt({ path, }: { readonly path: string; },): Promise<ReadingLeft> {
  try {
    return {
      kind: 'written',
      text: await readFile(
        path,
        'utf8',
      ),
    };
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    return { kind: 'absent', };
  }
}

/**
 Reads a picture with the deterministic OCR reader.

 @param bytes - picture as gathered from the corpus

 @param assetName - its file name, kept so the scratch copy carries the
 extension a decoder may want

 @param l - lane logger

 @param runProgram - runner the decoders and the OCR reader go through:
 `runInstalledProgram` in a run, a runner that starts no process in a test.
 Required, so that leaving it out is a type error rather than a reading
 that quietly depends on what the machine has installed

 @returns What it read, that it read nothing, or why it could not try

 @throws Whatever `mkdtemp` raises when scratch cannot be made, whatever
 writing the scratch copy raises, and whatever reading the transcript raises
 other than its absence, each a broken machine rather than an unreadable
 picture

 @example
 ```ts
 const reading = await readImageWithOcr({ bytes, assetName: 'letter.webp', l, runProgram: runInstalledProgram, },);
 ```
 */
export async function readImageWithOcr(
  {
    bytes,
    assetName,
    l,
    runProgram,
  }: {
    readonly bytes: Uint8Array;
    readonly assetName: string;
    readonly l: Logger;
    readonly runProgram: ProgramRunner;
  },
): Promise<OcrReading> {
  /**
   Logger pre-tagged with this function's name.
   */
  const ol = tagged({
    tag: readImageWithOcr.name,
    l,
  },);

  /**
   Directory every intermediate lands in, removed when this returns.
   */
  await using scratch = await scratchDirectory();

  /**
   Picture written where the command-line tools can reach it, under a fixed
   name so an asset called `../../etc/passwd` cannot direct a write anywhere.
   */
  const source = join(
    scratch.path,
    scratchCopyName({ assetName, },),
  );
  await writeFile(
    source,
    bytes,
  );

  /**
   Decoded copy the OCR reader accepts, a different file from `source` for
   every asset name.
   */
  const png = join(
    scratch.path,
    DECODED_COPY_NAME,
  );
  if (!await decodeToPng({
    source,
    png,
    assetName,
    l: ol,
    runProgram,
  },)) {
    ol.warn(`${assetName}: neither ${WEBP_DECODER} nor ${FALLBACK_DECODER} could decode it`,);
    return {
      kind: 'unavailable',
      reason: 'undecodable',
    };
  }

  /**
   Stem the OCR reader appends `.txt` to.
   */
  const stem = join(
    scratch.path,
    READING_STEM,
  );
  try {
    await runProgram({
      program: OCR_READER,
      args: [
        png,
        stem,
        '-l',
        TESSERACT_LANGUAGES,
      ],
    },);
  }
  catch (error) {
    /**
     The failure as an account authored from its code: not installed, a
     filesystem code, or the exit code of a run that failed. Read off the
     code, not the error's text, which carries the tool's own output and the
     paths it was handed.
     */
    const failure = new LocalProgramFailedError({
      program: OCR_READER,
      failure: localProgramFailureOf({ error, },),
      cause: error,
    },);
    /**
     Whether the reader is not installed rather than unhappy, which names the
     reason the reading carries: they are different problems for whoever reads
     the run, and a spawn of a program not installed fails with `ENOENT`.
     */
    const missing = isMissingPathError({ error, },);
    ol.warn(`${assetName}: ${refusalText({ error: failure, },)}`,);
    return {
      kind: 'unavailable',
      reason: missing ? 'ocr-tool-missing' : 'ocr-failed',
    };
  }

  /**
   What the reader left at its output path.
   */
  const left = await readingLeftAt({ path: `${stem}.txt`, },);
  if (left.kind === 'absent') {
    // A reader that says it succeeded and wrote nothing has failed. Reported
    // as the reader's failure, like its unhappy exit, so the picture goes on
    // to the readers that can still read it and the run log says why.
    ol.warn(`${assetName}: ${OCR_READER} exited without an error and left no reading`,);
    return {
      kind: 'unavailable',
      reason: 'ocr-failed',
    };
  }

  /**
   What it transcribed, whitespace and all.
   */
  const { text, } = left;

  /**
   How much of that is not whitespace, which is what the line is drawn on.
   */
  const characters = solidCharacters({ text, },);
  if (characters < MIN_READING_CHARS) {
    ol.info(`${assetName}: no text (${String(characters,)} ${
      wordForCount({
        count: characters,
        one: 'character',
        many: 'characters',
      },)
    }, under ${String(MIN_READING_CHARS,)})`,);
    return {
      kind: 'no-text',
      characters,
    };
  }

  ol.info(`${assetName}: read ${String(characters,)} ${
    wordForCount({
      count: characters,
      one: 'character',
      many: 'characters',
    },)
  } without a model`,);
  return {
    kind: 'read',
    text: text.trim(),
  };
}

/**
 Binds the OCR reader to the runner its programs go through, which gives the
 reader a picture stage asks for (`OcrReader` in `image-reading-pair.ts`):
 bytes, a name and a logger in, a reading out.

 A FUNCTION OF ITS OWN so that the one place a run names the real runner is
 a call made when the run's picture sources are built, and no function body
 exists that only a real run executes. A unit test drives the reader this
 returns with a runner that starts no process.

 @param runProgram - runner every program of every reading goes through

 @returns Reader that reads one picture through that runner

 @example
 ```ts
 const readOcr = ocrReaderOver({ runProgram: runInstalledProgram, },);
 ```
 */
export function ocrReaderOver(
  { runProgram, }: { readonly runProgram: ProgramRunner; },
): (
  picture: {
    readonly bytes: Uint8Array;
    readonly assetName: string;
    readonly l: Logger;
  },
) => Promise<OcrReading> {
  return function readOcrThroughRunner(
    {
      bytes,
      assetName,
      l,
    },
  ): Promise<OcrReading> {
    return readImageWithOcr({
      bytes,
      assetName,
      l,
      runProgram,
    },);
  };
}

/**
 Asks the deterministic reader about a picture, answering a reader that
 rejects with its own unavailable reading.

 CONTAINED FOR THE SAME REASON A MODEL READER IS. The deterministic reader
 documents some failures as raised (a scratch directory that cannot be made, a
 copy that cannot be written), each a broken machine for that reader's own
 caller; for this pair a reading is evidence and never a gate, so one picture's
 failure to be pre-screened must not end the entry. The models are then asked
 as they are where the tools are not installed.

 @param readOcr - deterministic reader to ask

 @param bytes - picture as read from disk

 @param assetName - its file name, which carries the media type

 @param signal - abort that must travel rather than be absorbed

 @param l - pair logger, told when the reader failed

 @returns What the reader made of the picture, or `unavailable` with reason
 `ocr-failed` when it rejected

 @throws {@link DOMException} when `signal` aborted by the time the reader
 rejected, since a stop is not a failure to absorb

 @example
 ```ts
 const ocr = await askDeterministicReader({ readOcr, bytes, assetName, signal, l, },);
 ```
 */
export async function askDeterministicReader(
  {
    readOcr,
    bytes,
    assetName,
    signal,
    l,
  }: {
    readonly readOcr: OcrReader;
    readonly bytes: Uint8Array;
    readonly assetName: string;
    readonly signal: AbortSignal;
    readonly l: Logger;
  },
): Promise<OcrReading> {
  try {
    return await readOcr({
      bytes,
      assetName,
      l,
    },);
  }
  catch (error) {
    // A STOP THAT KILLED THE READER IS NOT ITS FAILURE. Whatever the reader
    // raised, a signal already aborted means the run was told to stop.
    signal.throwIfAborted();
    l.warn(
      `${assetName}: the deterministic reader failed outright, so the models are asked without its gate (${
        readerFailureText({ error, },)
      })`,
    );
    return {
      kind: 'unavailable',
      reason: 'ocr-failed',
    };
  }
}

//endregion Image OCR
