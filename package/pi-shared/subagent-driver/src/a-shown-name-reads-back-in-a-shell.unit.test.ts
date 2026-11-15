/**
 The name a brief shows reads back byte for byte when a shell is given it the way the brief says it can be: as a `$'...'` word.

 The names are built from every byte value a file name may hold (every byte but `/` and the NUL byte that cannot be in a name), plus
 one name holding all of them and one whose bytes are UTF-8. Each shown name is written into a Bash script as the word the brief
 says a shell reads, the script prints what each word gives back, and every result is compared with the name's bytes. Bash failing
 on a word is a finding.

 @module
 */

import {
  type ExecException,
  execFileSync,
} from 'node:child_process';
import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  filesToListItems,
  pathsInAListing,
} from '../dist/final/node/index.mjs';

/**
 The byte that separates the folders of a path, which a file's own name may not hold.
 */
const SLASH_BYTE = 47;

/**
 The bytes a file's own name may hold: every one but the slash and NUL.
 */
const BYTES_A_NAME_MAY_HOLD: readonly number[] = Array.from({ length: 255, }, function bytes(_, index,) {
  return index + 1;
},)
  .filter(function isNoSlash(byte,) {
    return byte !== SLASH_BYTE;
  },);

/**
 Every name this test gives a shell: one name per byte, the name holding all of them, and one whose bytes are UTF-8.

 @returns each name's bytes

 @example
 ```ts
 const names = theNames();
 ```
 */
function theNames(): readonly Buffer[] {
  /**
   The folder the names sit in, which is the same for all of them.
   */
  const folder = Buffer.from('package/a/src/', 'utf8',);
  return [
    ...BYTES_A_NAME_MAY_HOLD.map(function aNameOfOneByte(byte,) {
      return Buffer.concat([folder, Buffer.from('n',), Buffer.from([byte,],), Buffer.from('.ts',)],);
    },),
    Buffer.concat([folder, Buffer.from('every',), Buffer.from(BYTES_A_NAME_MAY_HOLD,), Buffer.from('.ts',)],),
    Buffer.concat([folder, Buffer.from('é.unit.test.ts', 'utf8',)],),
  ];
}

/**
 The bash script that prints what a shell gives back for each word, one NUL byte after each.

 @param words - the words, each a `$'...'` word a brief says a shell reads

 @returns the script's text

 @example
 ```ts
 const script = theScript({ words: [`$'a'`], },);
 ```
 */
function theScript({ words }: { readonly words: readonly string[] },): string {
  return `#!/usr/bin/env bash\nprintf '%s\\0' ${words.join(' ',)}\n`;
}

/**
 A folder a test throws away.

 @returns the folder, and how to remove it

 @example
 ```ts
 await using folder = await makeAFolder();
 ```
 */
async function makeAFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-shown-name-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: 'a name the brief shows',
      children: [
        it({
          name: 'is the bytes of the file when the shell is given it as the shell word the brief names, for every byte a name may hold, so the way the brief tells a reader to open a file gives that file back',
          fn: async () => {
            /**
             Every name this test proves.
             */
            const names = theNames();
            /**
             The name as Git prints it, one character per byte, and then as the brief shows it.
             */
            const shown = names.map(function shownNameOf(bytes,) {
              return pathsInAListing({ listing: `${bytes.toString('latin1',)}\0`, },)[0] ?? '';
            },);
            expect(shown.length,).toBe(names.length,);
            // Some names need an escape, and the apostrophe is one of the bytes, so a word that leaves it as it is fails here.
            expect(shown.some(function holdsAnEscape(word,) {
              return word.includes(String.raw`\x`,);
            },),).toBe(true,);
            /**
             The folder this test writes its script in, which it throws away when it ends.
             */
            await using folder = await makeAFolder();
            /**
             The script's path.
             */
            const script = join(folder.path, 'words.sh',);
            await writeFile(script, theScript({ words: shown.map(function asAShellWord(word,) {
              return `$'${word}'`;
            },), },), 'utf8',);
            /**
             What the shell gave back, one NUL byte after each word.
             */
            let printed: string;
            try {
              printed = execFileSync('/bin/bash', ['--noprofile', '--norc', script,], {
                encoding: 'latin1',
                timeout: 60_000,
                killSignal: 'SIGKILL',
              },);
            }
            catch (caught) {
              throw new Error(
                `a shell could not read the name back: ${String((caught as ExecException).stderr,)}`
                  + ` words with an apostrophe: ${shown.filter(function holdsAnApostrophe(word,) {
                    return word.includes('\'',);
                  },)
                    .join(', ',)}`, { cause: caught },
              );
            }
            /**
             What each word gave back, in the order the words were written.
             */
            const givenBack = printed.split('\0',)
              .slice(
              0,
              -1,
              );
            expect(givenBack.length,).toBe(names.length,);
            expect(givenBack.map(function asHex(name,) {
              return Buffer.from(name, 'latin1',)
                .toString('hex',);
            },),)
              .toEqual(names.map(function asHex(name,) {
                return name.toString('hex',);
              },),);
          },
        },),
        it({
          name: 'is shown with the apostrophe and the backslash escaped when the brief explains an escape, so the sentence about the shell word is true of every name it explains',
          fn: async () => {
            /**
             A name holding an apostrophe and one byte that is not UTF-8.
             */
            const name = Buffer.concat([
              Buffer.from('package/a/src/it', 'utf8',),
              Buffer.from('\'s',),
              Buffer.from([0xE9,],),
              Buffer.from('.ts',),
            ],);
            /**
             The name as the brief shows it.
             */
            const shown = pathsInAListing({ listing: `${name.toString('latin1',)}\0`, },)[0] ?? '';
            expect(shown,).toBe(String.raw`package/a/src/it\'s\xe9.ts`,);
            expect(filesToListItems({ files: [shown,], },),).toContain("a shell reads as a `$'...'` word",);
            /**
             A name holding a backslash, which the shell word needs doubled.
             */
            const backslash = pathsInAListing({ listing: 'package/a/src/a\\b.ts\0', },)[0] ?? '';
            expect(backslash,).toBe(String.raw`package/a/src/a\\b.ts`,);
          },
        },),
      ],
    },),
  ],
},);
