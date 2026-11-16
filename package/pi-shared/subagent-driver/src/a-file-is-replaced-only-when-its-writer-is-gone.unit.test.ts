/**
 The class one walk: every call site that replaces a file says who can be writing it, and a replacement asked for while a writer may
 live is refused rather than tried.

 Every value and file here is made up, under the scratch folder the test harness names.

 @module
 */

import { appendFileSync, } from 'node:fs';
import {
  chmod,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createSecretValueMasking,
  maskASessionIntoACopy,
  maskedValueMarker,
  replaceAFileWithItsMaskedText,
  type SecretValue,
} from '../dist/final/node/index.mjs';

/** The stand-in value every control arms with. */
const SECRET = 'stand-in-writer-value-0123456789';
/** The name a marker says in its place. */
const SECRET_NAME = 'STAND_IN_WRITER_KEY';
/** The masking every control uses. */
const masking = createSecretValueMasking({ values: [{ name: SECRET_NAME, value: SECRET, },] satisfies readonly SecretValue[], },);

/**
 A folder this file makes, removed when it is done.
 
 @param prefix - what the folder holds
 
 @returns the folder's path and its disposer
 */
async function makeFolder({ prefix, }: { readonly prefix: string },): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  const path = await mkdtemp(join(scratchRoot(), prefix,),);
  await chmod(path, 0o700,);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 The three places this package replaces a file, and who can write each of them at that moment.
 */
const PLACES: readonly {
  readonly place: string;
  readonly writer: string;
  readonly answer: 'gone' | 'may-exist';
}[] = [
  {
    // Pi has not started: this run's Pi cannot write the file it resumes, but the file is another run's, and another Pi may.
    place: 'the session a run resumes, before Pi starts',
    writer: 'another run of the same session, which this code cannot see',
    answer: 'may-exist',
  },
  {
    // Pi has ended and been reaped: this run's Pi is not writing it, and another process is outside what this code can see.
    place: 'the session a run leaves behind, after Pi was reaped',
    writer: 'this run\'s Pi, which the driver waited for, and any process outside this package',
    answer: 'gone',
  },
  {
    // Pi's bash tool finishes and closes the output file before it returns the result this hook masks.
    place: 'the file Pi kept a tool\'s whole output in, while Pi lives',
    writer: 'Pi\'s own tool, which closed the file before the result existed',
    answer: 'gone',
  },
];

await describe({
  name: 'replacing a file only when its writer is gone',
  children: [
    it({
      name: 'refuses every replacement whose caller cannot say the writer is gone, and leaves the file untouched',
      fn: async () => {
        await using folder = await makeFolder({ prefix: 'pi-writer-refused-', },);
        /**
         The file, holding the value.
         */
        const path = join(folder.path, 'session.jsonl',);
        await writeFile(path, `${JSON.stringify({ entry: SECRET, },)}\n`,);
        /* oxlint-disable eslint/no-await-in-loop -- one place at a time: each is put in the file's place, answered, and read back. */
        for (const place of PLACES) {
          const answered = await replaceAFileWithItsMaskedText({
            path,
            maskText: masking.maskText,
            ...((place.answer === 'gone')
              ? { writer: { gone: true, }, }
              : { writer: { mayExist: `${place.writer} may still write it`, }, }),
          },);
          if (place.answer === 'gone') {
            expect(('replaced' in answered) || ('unchanged' in answered),).toBe(true,);
            // Put the file back for the next place, which reads the same file.
            await writeFile(path, `${JSON.stringify({ entry: SECRET, },)}\n`,);
            continue;
          }
          expect(answered,).toEqual({ refused: `${place.writer} may still write it`, },);
          expect(await readFile(path, 'utf8',),).toContain(SECRET,);
        }
        /* oxlint-enable eslint/no-await-in-loop */
      },
    },),
    it({
      name: 'masks a session a run resumes into a copy of its own, and leaves the file another Pi may write exactly as it is',
      fn: async () => {
        await using folder = await makeFolder({ prefix: 'pi-writer-resume-copy-', },);
        /**
         The session the run was told to resume.
         */
        const sessionFile = join(folder.path, 'old-session.jsonl',);
        await writeFile(sessionFile, `${JSON.stringify({ entry: SECRET, },)}\n`,);
        /**
         Where the masked copy goes.
         */
        const copyFile = join(folder.path, 'resumed-session.jsonl',);
        const copied = await maskASessionIntoACopy({
          sessionFile,
          copyFile,
          mask: masking,
        },);
        expect(copied,).toEqual({ path: copyFile, },);
        expect(await readFile(copyFile, 'utf8',),).toBe(`${JSON.stringify({ entry: maskedValueMarker(SECRET_NAME,), },)}\n`,);
        // The original is another run's file and is never replaced: that is the stated limit of a resumed session.
        expect(await readFile(sessionFile, 'utf8',),).toContain(SECRET,);
      },
    },),
    it({
      name: 'keeps an entry another command appended while the masked text was being prepared',
      fn: async () => {
        await using folder = await makeFolder({ prefix: 'pi-writer-append-', },);
        /**
         The file, holding the value.
         */
        const path = join(folder.path, 'session.jsonl',);
        await writeFile(path, `${JSON.stringify({ entry: SECRET, },)}\n`,);
        /**
         Whether the second writer has already written, so it appends once and the replacement settles.
         */
        const appended: { done: boolean } = { done: false, };
        const replaced = await replaceAFileWithItsMaskedText({
          path,
          writer: { gone: true, },
          maskText: function maskAndAppend(text: string,) {
            // A second writer appends once, between the read and the swap, which is the instant the replacement must notice.
            if (!appended.done) {
              appended.done = true;
              appendFileSync(path, `${JSON.stringify({ entry: 'another command wrote this', },)}\n`,);
            }
            return masking.maskText(text,);
          },
        },);
        expect(('replaced' in replaced) || ('unchanged' in replaced),).toBe(true,);
        /**
         What the file holds at the end.
         */
        const kept = await readFile(path, 'utf8',);
        expect(kept,).toContain('another command wrote this',);
        expect(kept,).not.toContain(SECRET,);
      },
    },),
  ],
},);
