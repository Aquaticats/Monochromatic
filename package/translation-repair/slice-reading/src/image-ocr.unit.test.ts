/**
 Tests for reading a picture without a model: counting what OCR yields, and
 every path of `readImageWithOcr`, which decodes a picture with one program
 and reads it with another.

 WHAT THIS PINS. `solidCharacters` is the count `readImageWithOcr` compares
 against `MIN_READING_CHARS` to decide `read` from `no-text`, so every shape
 of whitespace the raw reading might carry has to be discounted the same
 way, whether that is plain padding or the tabs and newlines a `.txt` file
 can carry. `readImageWithOcr` is pinned whole: which programs it runs, in
 which order and with which arguments, what it hands back, and every line
 it logs, for a picture the first decoder takes, one only the fallback
 takes, one neither takes, a reader that is not installed, a reader that
 fails, a reader that leaves nothing, and readings on each side of the
 floor.

 NO DECODER AND NO OCR READER RUNS HERE. `readImageWithOcr` takes the
 function that runs a program as a required parameter, and every case about
 it hands in `scriptedPrograms` (`scripted-programs.test-fixture.ts`), which
 starts no process. Its stand-ins do real file work at the paths the reader
 gives them, inside the scratch directory the reader makes for itself: the
 stand-in decoder copies its input to its output and the stand-in OCR reader
 writes its input out as text. So the bytes a case hands in as a picture are
 the words it reads back, and they come back only when the picture was
 written, decoded and read at the paths the programs were given. The scratch
 directory's own path is random, so it is read off the first run and shown
 as `<scratch>` in the runs and lines a case compares.

 THE ONE GROUP THAT STARTS A PROCESS is `runInstalledProgram`'s, the runner a
 run hands in. It starts Node itself (`process.execPath`), the program
 running this file and so never absent, and a path in a throwaway directory
 where no program stands. Those cases hold the two rejections the stand-ins
 raise against the ones Node raises, field for field.

 WHAT THE REAL TOOLS MAKE OF A REAL PICTURE IS NOT EXERCISED HERE. That
 depends on `tesseract` actually transcribing a picture, which needs its
 `chi_sim` language data installed, and a unit test must not depend on that.
 It was verified at the user boundary, through the built artifact, on
 2026-08-19:

 ```
 wangzihao980/Word1.webp       71288 bytes   read      405 chars
 zheermao101/photo3.webp       33038 bytes   read      557 chars
 dogesir_/intro.webp           95094 bytes   read      930 chars
 Zha_Ke/letter.webp           628180 bytes   read     1718 chars
 DarlinChit/photo1.webp       151352 bytes   read      139 chars
 wangzihao980/picture4.webp    13728 bytes   no-text     0 chars
 Uekawakuyuurei/img231.webp   169776 bytes   no-text     0 chars
 ```

 `extensionOf`, which names the scratch copy's extension, has its cases in
 `image-asset.unit.test.ts`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { stat, } from 'node:fs/promises';
import {
  basename,
  dirname,
  isAbsolute,
  join,
} from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isMissingPathError, } from '@monochromatic-dev/translation-repair-typography';
import {
  MIN_READING_CHARS,
  ocrReaderOver,
  type OcrReading,
  type ProgramRunner,
  readImageWithOcr,
  runInstalledProgram,
  solidCharacters,
} from '../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  copyingDecoder,
  damagingDecoder,
  failingProgram,
  missingProgram,
  missingProgramFailure,
  programExitFailure,
  type ProgramRun,
  scriptedPrograms,
  silentProgram,
  transcribingReader,
} from './scripted-programs.test-fixture.ts';

/**
 What a case reads where the reader's scratch directory stood, whose real
 path differs on every reading.
 */
const SCRATCH = '<scratch>';

/**
 Start of the name the reader gives its scratch directory.
 */
const SCRATCH_PREFIX = 'translation-repair-ocr-';

/**
 Asset most cases read: a letter in the corpus's usual picture format.
 */
const LETTER = 'mittens-letter.webp';

/**
 Words on the fixture letter: 30 characters once whitespace is dropped.
 */
const WORDS = 'Mittens naps on the warm windowsill';

/**
 The information line a reading of {@link WORDS} off {@link LETTER} logs.
 */
const LETTER_READ_LINE = `info [readImageWithOcr] ${LETTER}: read 30 characters without a model`;

/**
 The first decoder's run for a `.webp` asset, scratch masked.
 */
const DWEBP_RUN: ProgramRun = {
  program: 'dwebp',
  args: [
    `${SCRATCH}/asset.webp`,
    '-o',
    `${SCRATCH}/decoded.png`,
  ],
};

/**
 The fallback decoder's run for a `.webp` asset, scratch masked.
 */
const MAGICK_RUN: ProgramRun = {
  program: 'magick',
  args: [
    `${SCRATCH}/asset.webp`,
    `${SCRATCH}/decoded.png`,
  ],
};

/**
 The OCR reader's run, the same for every asset, scratch masked.
 */
const TESSERACT_RUN: ProgramRun = {
  program: 'tesseract',
  args: [
    `${SCRATCH}/decoded.png`,
    `${SCRATCH}/reading`,
    '-l',
    'chi_sim+eng',
  ],
};

/**
 A machine where the first decoder takes the picture and the reader reads
 it. The fallback decoder has no stand-in, so a run of it is recorded and
 rejected.
 */
const FIRST_DECODER_READS: ReadonlyMap<string, ProgramRunner> = new Map([
  [
    'dwebp',
    copyingDecoder,
  ],
  [
    'tesseract',
    transcribingReader,
  ],
],);

/**
 A decoder stand-in that refuses the picture as a real decoder refuses a
 format it does not read: it runs and exits unhappy.
 */
const REFUSING_DECODER = failingProgram({
  exitCode: 1,
  stderr: 'this is not a picture a cat would recognise\n',
},);

/**
 A program stand-in on a machine whose scratch device is full: it rejects as
 Node does, with the filesystem code and a message that quotes a path.

 @returns Never

 @throws {@link Error} carrying the code `ENOSPC`, always

 @example
 ```ts
 const programs = new Map([['tesseract', fullDisk,],],);
 ```
 */
async function fullDisk(): Promise<void> {
  /**
   Rejection a disk with no space left raises.
   */
  const failure = new Error('ENOSPC: no space left on device, write \'/scratch/reading.txt\'',);
  Object.defineProperty(
    failure,
    'code',
    { value: 'ENOSPC', },
  );
  throw failure;
}

/**
 Bytes a damaged decode leaves behind: 32 characters of words that are not
 the letter's, enough to come back as a reading were they ever read.
 */
const HAIRBALL = 'hairball hairball hairball hairball';

/**
 A picture whose bytes are its words, which is what the stand-in decoder
 and reader make of any file.

 @param words - what the picture says, whitespace and all

 @returns Its bytes

 @example
 ```ts
 const bytes = pictureOf({ words: 'Mittens naps', },);
 ```
 */
function pictureOf({ words, }: { readonly words: string; },): Uint8Array {
  return new TextEncoder().encode(words,);
}

/**
 What one reading shows a case, with the scratch directory's random path
 masked so the whole of it compares against a literal.
 */
type Shown = {
  /**
   What the reader handed back.
   */
  readonly reading: OcrReading;

  /**
   Every program it ran, in order, with every argument.
   */
  readonly runs: readonly ProgramRun[];

  /**
   Every line it logged, each behind its level.
   */
  readonly lines: readonly string[];
};

/**
 Reads one picture through scripted programs and gathers what the reading
 shows.

 @param assetName - name the picture is read under

 @param words - what the picture says, which becomes its bytes

 @param programs - stand-in per program name

 @returns What the reading shows with scratch masked, and the scratch
 directory's real path

 @throws {@link Error} when the reading ran no program, since the scratch
 directory is read off the first run

 @example
 ```ts
 const { shown, scratch, } = await observedReading({ assetName: LETTER, words: WORDS, programs: FIRST_DECODER_READS, },);
 ```
 */
async function observedReading(
  {
    assetName,
    words,
    programs,
  }: {
    readonly assetName: string;
    readonly words: string;
    readonly programs: ReadonlyMap<string, ProgramRunner>;
  },
): Promise<{
  readonly shown: Shown;
  readonly scratch: string;
}> {
  /**
   Lines the reader logs, each behind its level.
   */
  const lines: string[] = [];
  /**
   Runner that starts no process, and the runs it is asked for.
   */
  const scripted = scriptedPrograms({ programs, },);
  /**
   What the reader handed back.
   */
  const reading = await readImageWithOcr({
    bytes: pictureOf({ words, },),
    assetName,
    l: levelCapturingLogger({ lines, },),
    runProgram: scripted.runProgram,
  },);
  /**
   The first run, whose first argument is the scratch copy of the picture.
   */
  const [first,] = scripted.runs;
  if (first === undefined)
    throw new Error('the reading ran no program, so its scratch directory cannot be read off a run',);
  /**
   Directory the scratch copy was written in.
   */
  const scratch = dirname(nonNullishOrThrow(first.args[0],),);
  return {
    scratch,
    shown: {
      reading,
      runs: scripted.runs.map((run,) => ({
        program: run.program,
        args: run.args.map((arg,) =>
          arg.replaceAll(
            scratch,
            SCRATCH,
          )
        ),
      })),
      lines: lines.map((line,) =>
        line.replaceAll(
          scratch,
          SCRATCH,
        )
      ),
    },
  };
}

/**
 What a spawn failure carries, as a plain record a case can compare whole:
 its name, its message and every enumerable field but `signal`, which Node
 sets to `null` on a plain exit and the scripted rejection does not declare.

 @param error - caught value

 @returns Its name, message and fields

 @throws {@link Error} when the caught value is no error at all

 @example
 ```ts
 const fields = spawnFailureFields({ error, },);
 ```
 */
function spawnFailureFields({ error, }: { readonly error: unknown; },): Readonly<Record<string, unknown>> {
  if (!Error.isError(error,))
    throw new Error(`expected a spawn failure, and caught ${String(error,)}`,);
  return {
    ...Object.fromEntries(Object.entries(error,).filter(([key,],) => key !== 'signal'),),
    name: error.name,
    message: error.message,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: solidCharacters.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS ZERO FOR AN EMPTY STRING, the base case every other whitespace rule in this '
            + 'function has to agree with',
          fn: async () => {
            expect(solidCharacters({ text: '', },),).toBe(0,);
          },
        },),

        it({
          name: 'COUNTS ZERO FOR TEXT THAT IS ONLY SPACES, so a picture that yields nothing but padding '
            + 'reads as bare rather than as a few characters of noise',
          fn: async () => {
            expect(solidCharacters({ text: '    ', },),).toBe(0,);
          },
        },),

        it({
          name: 'COUNTS ZERO FOR TEXT MADE ENTIRELY OF NEWLINES AND TABS, since the reading this counts '
            + 'comes from a raw `.txt` file read whitespace and all, and line breaks alone must never '
            + 'register as a character of transcript',
          fn: async () => {
            expect(solidCharacters({ text: '\n\t\n\t', },),).toBe(0,);
          },
        },),

        it({
          name: 'COUNTS ONLY THE NON-WHITESPACE RUN WHEN SPACES, TABS AND NEWLINES SIT BETWEEN LETTERS, '
            + 'since the raw reading is counted whitespace and all and only the solid characters decide '
            + 'whether a picture crosses `MIN_READING_CHARS`',
          fn: async () => {
            expect(solidCharacters({ text: 'a b\tc\nd', },),).toBe(4,);
          },
        },),

        it({
          name: 'COUNTS EVERY CHARACTER WHEN THE TEXT CARRIES NO WHITESPACE AT ALL, so a dense '
            + 'transcript is never undercounted for having nothing to strip',
          fn: async () => {
            expect(solidCharacters({ text: 'abcdef', },),).toBe(6,);
          },
        },),

        it({
          name: 'COUNTS NON-WHITESPACE CHARACTERS IN CHINESE TEXT THE SAME WAY AS LATIN, since the '
            + 'corpus this reads is `chi_sim` and a count that only worked on Latin script would '
            + 'silently break on every real reading',
          fn: async () => {
            expect(solidCharacters({ text: '喵喵 喵', },),).toBe(3,);
          },
        },),
      ],
    },),

    describe({
      name: readImageWithOcr.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a picture the first decoder takes: runs that decoder and the OCR reader once each '
            + 'on fixed names under scratch, never the fallback decoder, hands back the words trimmed as '
            + '`read`, and logs one information line counting their solid characters',
          fn: async () => {
            /**
             The letter, padded as a raw reading is.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words: `\n  ${WORDS} \n\n`,
              programs: FIRST_DECODER_READS,
            },);

            expect(shown,).toEqual({
              reading: {
                kind: 'read',
                text: WORDS,
              },
              runs: [
                DWEBP_RUN,
                TESSERACT_RUN,
              ],
              lines: [LETTER_READ_LINE,],
            },);
          },
        },),

        it({
          name: 'FALLS BACK to the second decoder when the first refuses the picture: runs both in '
            + 'order, logs at debug which decoder refused and the exit code it refused with, and reads what '
            + 'the fallback decoded',
          fn: async () => {
            /**
             A reading on a machine whose first decoder refuses the letter.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words: WORDS,
              programs: new Map([
                [
                  'dwebp',
                  REFUSING_DECODER,
                ],
                [
                  'magick',
                  copyingDecoder,
                ],
                [
                  'tesseract',
                  transcribingReader,
                ],
              ],),
            },);

            expect(shown,).toEqual({
              reading: {
                kind: 'read',
                text: WORDS,
              },
              runs: [
                DWEBP_RUN,
                MAGICK_RUN,
                TESSERACT_RUN,
              ],
              lines: [
                `debug [readImageWithOcr] ${LETTER}: dwebp did not decode it (dwebp exited with code 1)`,
                LETTER_READ_LINE,
              ],
            },);
          },
        },),

        it({
          name: 'REPORTS `undecodable` when both decoders refuse the picture: never runs the OCR reader, '
            + 'logs each decoder\'s refusal at debug, and warns once naming the asset and both decoders',
          fn: async () => {
            /**
             A reading of bytes no decoder takes.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words: WORDS,
              programs: new Map([
                [
                  'dwebp',
                  REFUSING_DECODER,
                ],
                [
                  'magick',
                  REFUSING_DECODER,
                ],
              ],),
            },);

            expect(shown,).toEqual({
              reading: {
                kind: 'unavailable',
                reason: 'undecodable',
              },
              runs: [
                DWEBP_RUN,
                MAGICK_RUN,
              ],
              lines: [
                `debug [readImageWithOcr] ${LETTER}: dwebp did not decode it (dwebp exited with code 1)`,
                `debug [readImageWithOcr] ${LETTER}: magick did not decode it (magick exited with code 1)`,
                `warn [readImageWithOcr] ${LETTER}: neither dwebp nor magick could decode it`,
              ],
            },);
          },
        },),

        it({
          name: 'LOGS a decoder that is not installed apart from one that refused the picture, so a '
            + 'machine missing a decoder is told from a picture no decoder reads',
          fn: async () => {
            /**
             A reading on a machine with no first decoder, whose fallback
             refuses the letter.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words: WORDS,
              programs: new Map([
                [
                  'dwebp',
                  missingProgram,
                ],
                [
                  'magick',
                  REFUSING_DECODER,
                ],
              ],),
            },);

            expect(shown.lines,).toEqual([
              `debug [readImageWithOcr] ${LETTER}: dwebp is not installed`,
              `debug [readImageWithOcr] ${LETTER}: magick did not decode it (magick exited with code 1)`,
              `warn [readImageWithOcr] ${LETTER}: neither dwebp nor magick could decode it`,
            ],);
          },
        },),

        it({
          name: 'REPORTS `ocr-tool-missing` when the OCR reader is not installed, warning with the name '
            + 'of the reader it ran and the spawn failure',
          fn: async () => {
            /**
             A reading on a machine that decodes and has no OCR reader.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words: WORDS,
              programs: new Map([
                [
                  'dwebp',
                  copyingDecoder,
                ],
                [
                  'tesseract',
                  missingProgram,
                ],
              ],),
            },);

            expect(shown,).toEqual({
              reading: {
                kind: 'unavailable',
                reason: 'ocr-tool-missing',
              },
              runs: [
                DWEBP_RUN,
                TESSERACT_RUN,
              ],
              lines: [
                `warn [readImageWithOcr] ${LETTER}: tesseract is not installed`,
              ],
            },);
          },
        },),

        it({
          name: 'REPORTS `ocr-failed` when the OCR reader runs and exits unhappy, warning with its exit code and '
            + 'never its command line or what it complained of',
          fn: async () => {
            /**
             A reading on a machine whose OCR reader gives up on the letter.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words: WORDS,
              programs: new Map([
                [
                  'dwebp',
                  copyingDecoder,
                ],
                [
                  'tesseract',
                  failingProgram({
                    exitCode: 1,
                    stderr: 'the cat sat on the scanner\n',
                  },),
                ],
              ],),
            },);

            expect(shown,).toEqual({
              reading: {
                kind: 'unavailable',
                reason: 'ocr-failed',
              },
              runs: [
                DWEBP_RUN,
                TESSERACT_RUN,
              ],
              lines: [
                `warn [readImageWithOcr] ${LETTER}: tesseract exited with code 1`,
              ],
            },);
          },
        },),

        it({
          name: 'REPORTS `ocr-failed` naming the filesystem code when the OCR reader fails with one, such as a full '
            + 'disk, and never the message that quotes a path',
          fn: async () => {
            /**
             A reading on a machine whose scratch device fills up under the OCR reader.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words: WORDS,
              programs: new Map([
                [
                  'dwebp',
                  copyingDecoder,
                ],
                [
                  'tesseract',
                  fullDisk,
                ],
              ],),
            },);

            expect(shown.lines,).toEqual([
              `warn [readImageWithOcr] ${LETTER}: tesseract failed with filesystem code ENOSPC`,
            ],);
          },
        },),

        it({
          name: 'LOGS the filesystem code a decoder failed with, and never the message that quotes a path, '
            + 'as it logs the OCR reader\'s',
          fn: async () => {
            /**
             A reading on a machine whose scratch device fills up under the
             first decoder.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words: WORDS,
              programs: new Map([
                [
                  'dwebp',
                  fullDisk,
                ],
                [
                  'magick',
                  copyingDecoder,
                ],
                [
                  'tesseract',
                  transcribingReader,
                ],
              ],),
            },);

            expect(shown.lines,).toEqual([
              `debug [readImageWithOcr] ${LETTER}: dwebp did not decode it (dwebp failed with filesystem code `
                + 'ENOSPC)',
              LETTER_READ_LINE,
            ],);
          },
        },),

        it({
          name: 'REPORTS `ocr-failed` when the OCR reader exits without an error and leaves no reading '
            + 'behind, with a warning that says so, rather than rejecting with the missing file',
          fn: async () => {
            /**
             A reading on a machine whose OCR reader writes nothing.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words: WORDS,
              programs: new Map([
                [
                  'dwebp',
                  copyingDecoder,
                ],
                [
                  'tesseract',
                  silentProgram,
                ],
              ],),
            },);

            expect(shown,).toEqual({
              reading: {
                kind: 'unavailable',
                reason: 'ocr-failed',
              },
              runs: [
                DWEBP_RUN,
                TESSERACT_RUN,
              ],
              lines: [`warn [readImageWithOcr] ${LETTER}: tesseract exited without an error and left no reading`,],
            },);
          },
        },),

        it({
          name: 'READS `no-text` with a count of zero off a reading that is only whitespace, and logs '
            + 'the count against the floor',
          fn: async () => {
            /**
             A reading of a picture with nothing on it.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words: ' \n\t\n',
              programs: FIRST_DECODER_READS,
            },);

            expect(shown,).toEqual({
              reading: {
                kind: 'no-text',
                characters: 0,
              },
              runs: [
                DWEBP_RUN,
                TESSERACT_RUN,
              ],
              lines: [
                `info [readImageWithOcr] ${LETTER}: no text (0 characters, under ${String(MIN_READING_CHARS,)})`,
              ],
            },);
          },
        },),

        it({
          name: 'READS `no-text` one solid character under the floor, counted by code point, so a '
            + 'reading of astral characters is not doubled into a transcript',
          fn: async () => {
            /**
             One cat short of the floor, each cat two UTF-16 units wide.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words: '🐈'.repeat(MIN_READING_CHARS - 1,),
              programs: FIRST_DECODER_READS,
            },);

            expect(shown,).toEqual({
              reading: {
                kind: 'no-text',
                characters: MIN_READING_CHARS - 1,
              },
              runs: [
                DWEBP_RUN,
                TESSERACT_RUN,
              ],
              lines: [
                `info [readImageWithOcr] ${LETTER}: no text (${String(MIN_READING_CHARS - 1,)} characters, under ${
                  String(MIN_READING_CHARS,)
                })`,
              ],
            },);
          },
        },),

        it({
          name: 'READS `read` at exactly the floor, the shortest reading taken as text',
          fn: async () => {
            /**
             Exactly as many solid characters as the floor asks for.
             */
            const words = '喵'.repeat(MIN_READING_CHARS,);
            /**
             A reading of them.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words,
              programs: FIRST_DECODER_READS,
            },);

            expect(shown,).toEqual({
              reading: {
                kind: 'read',
                text: words,
              },
              runs: [
                DWEBP_RUN,
                TESSERACT_RUN,
              ],
              lines: [
                `info [readImageWithOcr] ${LETTER}: read ${String(MIN_READING_CHARS,)} characters without a model`,
              ],
            },);
          },
        },),

        it({
          name: 'KEEPS a `.png` picture whole for the fallback decoder when the first decoder leaves a '
            + 'damaged file at its target before refusing: both decoders are given a target that is not '
            + 'the scratch copy, and the words read are the picture\'s',
          fn: async () => {
            /**
             A reading of a `.png` letter on a machine whose first decoder
             writes part of its output before giving up.
             */
            const { shown, } = await observedReading({
              assetName: 'mittens-letter.png',
              words: WORDS,
              programs: new Map([
                [
                  'dwebp',
                  damagingDecoder({ garbage: pictureOf({ words: HAIRBALL, },), },),
                ],
                [
                  'magick',
                  copyingDecoder,
                ],
                [
                  'tesseract',
                  transcribingReader,
                ],
              ],),
            },);

            expect(shown,).toEqual({
              reading: {
                kind: 'read',
                text: WORDS,
              },
              runs: [
                {
                  program: 'dwebp',
                  args: [
                    `${SCRATCH}/asset.png`,
                    '-o',
                    `${SCRATCH}/decoded.png`,
                  ],
                },
                {
                  program: 'magick',
                  args: [
                    `${SCRATCH}/asset.png`,
                    `${SCRATCH}/decoded.png`,
                  ],
                },
                TESSERACT_RUN,
              ],
              lines: [
                'debug [readImageWithOcr] mittens-letter.png: dwebp did not decode it (dwebp exited with code 1)',
                'info [readImageWithOcr] mittens-letter.png: read 30 characters without a model',
              ],
            },);
          },
        },),

        it({
          name: 'KEEPS a `.webp` picture whole for the fallback decoder when the first decoder leaves a '
            + 'damaged file at its target before refusing',
          fn: async () => {
            /**
             A reading of the `.webp` letter on a machine whose first
             decoder writes part of its output before giving up.
             */
            const { shown, } = await observedReading({
              assetName: LETTER,
              words: WORDS,
              programs: new Map([
                [
                  'dwebp',
                  damagingDecoder({ garbage: pictureOf({ words: HAIRBALL, },), },),
                ],
                [
                  'magick',
                  copyingDecoder,
                ],
                [
                  'tesseract',
                  transcribingReader,
                ],
              ],),
            },);

            expect(shown,).toEqual({
              reading: {
                kind: 'read',
                text: WORDS,
              },
              runs: [
                DWEBP_RUN,
                MAGICK_RUN,
                TESSERACT_RUN,
              ],
              lines: [
                `debug [readImageWithOcr] ${LETTER}: dwebp did not decode it (dwebp exited with code 1)`,
                LETTER_READ_LINE,
              ],
            },);
          },
        },),

        it({
          name: 'NAMES the scratch copy `asset.` plus the asset\'s extension, lowercased, only when that '
            + 'extension is at most 16 ASCII letters and digits, and `asset.` alone otherwise, so no '
            + 'asset name chooses where the picture is written or makes the write fail',
          fn: async () => {
            /**
             Asset names, each with the scratch copy the first decoder was
             handed for it.
             */
            const copies = await Promise.all(
              [
                'MITTENS.WEBP',
                'kitten.tar.gz',
                'mittens-letter',
                'whiskers.p ng',
                '../../etc/passwd',
                'tabby.png/../../paw',
                `purr.${'r'.repeat(16,)}`,
                `purr.${'r'.repeat(17,)}`,
                `purr.${'r'.repeat(300,)}`,
              ].map(async (assetName,) => {
                /**
                 A reading under that name.
                 */
                const { shown, } = await observedReading({
                  assetName,
                  words: WORDS,
                  programs: FIRST_DECODER_READS,
                },);
                return [
                  assetName,
                  nonNullishOrThrow(shown.runs[0],).args[0],
                ];
              },),
            );

            expect(copies,).toEqual([
              [
                'MITTENS.WEBP',
                `${SCRATCH}/asset.webp`,
              ],
              [
                'kitten.tar.gz',
                `${SCRATCH}/asset.gz`,
              ],
              [
                'mittens-letter',
                `${SCRATCH}/asset.`,
              ],
              [
                'whiskers.p ng',
                `${SCRATCH}/asset.`,
              ],
              [
                '../../etc/passwd',
                `${SCRATCH}/asset.`,
              ],
              [
                'tabby.png/../../paw',
                `${SCRATCH}/asset.`,
              ],
              [
                `purr.${'r'.repeat(16,)}`,
                `${SCRATCH}/asset.${'r'.repeat(16,)}`,
              ],
              [
                `purr.${'r'.repeat(17,)}`,
                `${SCRATCH}/asset.`,
              ],
              [
                `purr.${'r'.repeat(300,)}`,
                `${SCRATCH}/asset.`,
              ],
            ],);
          },
        },),

        it({
          name: 'READS an asset called `../../etc/passwd` like any other: every argument of every '
            + 'program is a fixed name inside the one scratch directory, and the words come back',
          fn: async () => {
            /**
             A reading under a name that climbs out of its directory.
             */
            const { shown, } = await observedReading({
              assetName: '../../etc/passwd',
              words: WORDS,
              programs: FIRST_DECODER_READS,
            },);

            expect(shown,).toEqual({
              reading: {
                kind: 'read',
                text: WORDS,
              },
              runs: [
                {
                  program: 'dwebp',
                  args: [
                    `${SCRATCH}/asset.`,
                    '-o',
                    `${SCRATCH}/decoded.png`,
                  ],
                },
                TESSERACT_RUN,
              ],
              lines: ['info [readImageWithOcr] ../../etc/passwd: read 30 characters without a model',],
            },);
          },
        },),

        it({
          name: 'REMOVES its scratch directory once the reading returns: a directory at an absolute '
            + 'path whose name starts `translation-repair-ocr-`, gone by the time the caller has the '
            + 'reading',
          fn: async () => {
            /**
             A reading that worked, so every intermediate was written.
             */
            const { scratch, } = await observedReading({
              assetName: LETTER,
              words: WORDS,
              programs: FIRST_DECODER_READS,
            },);
            /**
             What asking the filesystem about the directory raises now.
             */
            const asked = await rejectionOf(async function statScratch(): Promise<void> {
              await stat(scratch,);
            },);

            expect({
              absolute: isAbsolute(scratch,),
              prefix: basename(scratch,).slice(
                0,
                SCRATCH_PREFIX.length,
              ),
              gone: isMissingPathError({ error: asked, },),
            },).toEqual({
              absolute: true,
              prefix: SCRATCH_PREFIX,
              gone: true,
            },);
          },
        },),
      ],
    },),

    describe({
      name: ocrReaderOver.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'BINDS the runner it is handed: the reader it returns runs its programs through that '
            + 'runner and hands back the reading',
          fn: async () => {
            /**
             Runner that starts no process, and the runs it is asked for.
             */
            const scripted = scriptedPrograms({ programs: FIRST_DECODER_READS, },);
            /**
             What the bound reader made of the letter.
             */
            const reading = await ocrReaderOver({ runProgram: scripted.runProgram, },)({
              bytes: pictureOf({ words: WORDS, },),
              assetName: LETTER,
              l: levelCapturingLogger({ lines: [], },),
            },);

            expect({
              reading,
              programs: scripted.runs.map((run,) => run.program),
            },).toEqual({
              reading: {
                kind: 'read',
                text: WORDS,
              },
              programs: [
                'dwebp',
                'tesseract',
              ],
            },);
          },
        },),
      ],
    },),

    describe({
      name: runInstalledProgram.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RESOLVES with nothing when the program exits zero',
          fn: async () => {
            await expect(runInstalledProgram({
              program: process.execPath,
              args: [
                '--eval',
                '',
              ],
            },),).resolves
              .toBeUndefined();
          },
        },),

        it({
          name: 'REJECTS a path where no program stands with the missing-path error, field for field '
            + 'the rejection `missingProgramFailure` scripts',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'image-ocr-no-program-', },);
            /**
             A program that is certainly not installed: a path in the empty
             scratch directory this case owns.
             */
            const run: ProgramRun = {
              program: join(
                scratch.path,
                'no-cat-reader',
              ),
              args: [
                '--purr',
                'loudly',
              ],
            };
            /**
             What the real runner raises for it.
             */
            const refusal = await rejectionOf(async function runMissing(): Promise<void> {
              await runInstalledProgram(run,);
            },);

            expect({
              missing: isMissingPathError({ error: refusal, },),
              fields: spawnFailureFields({ error: refusal, },),
            },).toEqual({
              missing: true,
              fields: spawnFailureFields({ error: missingProgramFailure(run,), },),
            },);
          },
        },),

        it({
          name: 'REJECTS a program that exits unhappy with its exit code as a number, which is not the '
            + 'missing-path error, field for field the rejection `programExitFailure` scripts',
          fn: async () => {
            /**
             Node itself, told to complain and exit with a code of its own.
             */
            const run: ProgramRun = {
              program: process.execPath,
              args: [
                '--eval',
                String.raw`process.stderr.write("the cat knocked it over\n",); process.exit(3,);`,
              ],
            };
            /**
             What the real runner raises for it.
             */
            const refusal = await rejectionOf(async function runUnhappy(): Promise<void> {
              await runInstalledProgram(run,);
            },);

            expect({
              missing: isMissingPathError({ error: refusal, },),
              fields: spawnFailureFields({ error: refusal, },),
            },).toEqual({
              missing: false,
              fields: spawnFailureFields({
                error: programExitFailure({
                  ...run,
                  exitCode: 3,
                  stderr: 'the cat knocked it over\n',
                },),
              },),
            },);
          },
        },),
      ],
    },),
  ],
},);
